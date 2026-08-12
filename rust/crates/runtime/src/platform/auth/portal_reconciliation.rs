use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::sync::Notify;

use super::{
    browser_consent_proposal, resolve_portal_authority_selection, AccountRepository,
    ApplyIdentityAuthoritySelectionInput, AuthService, AuthorityEvidenceRepository,
    AuthorityRepository, AuthorityState, AuthorizationStateError, ContextRepository,
    IdempotencyResultRecord, OutboxRepository, PortalAuthoritySource, PortalBindingMutation,
    PortalRepository, ProviderLoginAttributes,
};
use crate::shutdown::StopHandle;

#[derive(Clone)]
pub(crate) struct PortalPolicyReconciliationHandle {
    notify: Arc<Notify>,
}

impl PortalPolicyReconciliationHandle {
    pub(crate) fn notify(&self) {
        self.notify.notify_one();
    }
}

pub(crate) struct PortalPolicyReconciliationWorker<R> {
    service: AuthService<R>,
    notify: Arc<Notify>,
}

pub(crate) fn portal_policy_reconciliation<R>(
    service: AuthService<R>,
) -> (
    PortalPolicyReconciliationHandle,
    PortalPolicyReconciliationWorker<R>,
) {
    let notify = Arc::new(Notify::new());
    (
        PortalPolicyReconciliationHandle {
            notify: notify.clone(),
        },
        PortalPolicyReconciliationWorker { service, notify },
    )
}

impl<R> PortalPolicyReconciliationWorker<R>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + AuthorityRepository
        + ContextRepository
        + OutboxRepository
        + PortalRepository
        + Clone
        + Send
        + Sync,
{
    pub(crate) async fn run(self, stop: StopHandle) -> Result<(), AuthorizationStateError> {
        self.reconcile_all().await?;
        loop {
            tokio::select! {
                () = stop.stopped() => return Ok(()),
                () = self.notify.notified() => self.reconcile_all().await?,
            }
        }
    }

    async fn reconcile_all(&self) -> Result<(), AuthorizationStateError> {
        for binding in self
            .service
            .repository()
            .list_portal_authority_bindings()
            .await?
        {
            self.reconcile_binding(binding).await?;
        }
        Ok(())
    }

    async fn reconcile_binding(
        &self,
        binding: super::PortalAuthorityBindingRecord,
    ) -> Result<(), AuthorizationStateError> {
        let Some(current) = self
            .service
            .repository()
            .get_identity_authority(&binding.principal_id, &binding.participant_id)
            .await?
        else {
            self.service
                .repository()
                .remove_portal_authority_binding(&binding.principal_id, &binding.participant_id)
                .await?;
            return Ok(());
        };
        let portal = self
            .service
            .repository()
            .get_login_portal(&binding.portal_id)
            .await?;
        let policy = self
            .service
            .repository()
            .get_portal_grant_override(&binding.portal_id, &binding.participant_id)
            .await?;
        let provider_allowed = portal.as_ref().is_some_and(|(portal, settings)| {
            portal.provider_ids.contains(&binding.provider_id)
                && (binding.provider_id != "local" || settings.local_login_enabled)
        });
        let now = super::reconciliation::unix_time_millis()?;
        let (state, grant_set, capabilities, replacement, semantic_key) = if portal
            .as_ref()
            .is_none_or(|(portal, _)| portal.disabled || portal.removed)
            || !provider_allowed
            || policy.is_none()
        {
            (
                AuthorityState::Revoked,
                current.desired_grant_set.clone(),
                current.desired_capabilities.clone(),
                None,
                None,
            )
        } else {
            let policy = match policy {
                Some(policy) => policy,
                None => unreachable!("checked above"),
            };
            let participant = self
                .service
                .repository()
                .get_participant_binding(
                    &current.participant_id,
                    &current.participant_artifact_digest,
                )
                .await?
                .ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord(
                        "portal-managed participant binding is missing".to_owned(),
                    )
                })?;
            let consent = browser_consent_proposal(&participant)?;
            let groups = self
                .service
                .repository()
                .list_capability_groups()
                .await?
                .into_iter()
                .map(|group| (group.group_key.clone(), group))
                .collect::<BTreeMap<_, _>>();
            let selection = resolve_portal_authority_selection(
                &policy,
                &groups,
                &consent,
                &ProviderLoginAttributes {
                    provider_id: binding.provider_id.clone(),
                    roles: binding.roles.clone(),
                },
            )?;
            if current.state == AuthorityState::Accepted
                && binding.authority_version == current.version
                && binding.effective_policy_digest == selection.effective_policy_digest
            {
                return Ok(());
            }
            let semantic_key = selection.effective_policy_digest.clone();
            (
                AuthorityState::Accepted,
                selection.grant_set,
                selection.capabilities,
                Some(PortalAuthoritySource {
                    portal_id: binding.portal_id.clone(),
                    provider_id: binding.provider_id.clone(),
                    roles: binding.roles.clone(),
                    effective_policy_digest: selection.effective_policy_digest,
                }),
                Some(semantic_key),
            )
        };
        let request = json!({
            "principalId": binding.principal_id, "participantId": binding.participant_id,
            "authorityId": binding.authority_id,
            "currentAuthorityVersion": current.version,
            "currentEffectivePolicyDigest": binding.effective_policy_digest,
            "targetEffectivePolicyDigest": semantic_key,
            "targetState": state,
        });
        let request_digest = trellis_protocol::digest_json(&request)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let result = self.service.apply_identity_authority_selection(ApplyIdentityAuthoritySelectionInput {
            principal_id: binding.principal_id.clone(), participant_id: binding.participant_id.clone(),
            participant_artifact_digest: current.participant_artifact_digest,
            participant_needs_digest: current.accepted_needs_digest,
            grant_set, capabilities, state, decided_by: "portal-policy-reconciler".to_owned(),
            source_payload: json!({ "source": "portal_policy_reconciliation", "semanticKey": semantic_key }),
            portal_binding: PortalBindingMutation::CompareAndSet {
                expected: Some(binding), replacement,
            }, decided_at: now,
            expires_at: current.expires_at,
            proposal_idempotency: idempotency("portal.policy.propose", &request_digest, &request, now)?,
            decision_idempotency: idempotency("portal.policy.accept", &request_digest, &request, now)?,
        }).await;
        if matches!(result, Err(AuthorizationStateError::StorageConflict)) {
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.notify.notify_one();
            return Ok(());
        }
        result?;
        Ok(())
    }
}

fn idempotency(
    purpose: &str,
    request_digest: &str,
    request: &Value,
    now: i64,
) -> Result<IdempotencyResultRecord, AuthorizationStateError> {
    Ok(IdempotencyResultRecord {
        scope_key: trellis_protocol::digest_json(&json!([purpose, request_digest]))
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        purpose: purpose.to_owned(),
        signer_id: "portal-policy-reconciler".to_owned(),
        request_id: request_digest.to_owned(),
        request_digest: trellis_protocol::digest_json(request)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        result: Value::Null,
        created_at: now,
        expires_at: now.saturating_add(86_400_000),
    })
}
