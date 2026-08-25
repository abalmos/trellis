use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{stream, StreamExt};
use serde_json::{json, Value};
use tokio::sync::{Mutex, Notify};

use super::{
    browser_consent_proposal, portal_policy_snapshot, resolve_portal_authority_selection,
    AccountRepository, ApplyIdentityAuthoritySelectionInput, AuthService,
    AuthorityEvidenceRepository, AuthorityKind, AuthorityRepository, AuthorityState,
    AuthorityTarget, AuthorizationReconciliationHandle, AuthorizationStateError,
    CapabilityGroupRecord, ContextRepository, IdempotencyResultRecord, IdentityAuthorityRecord,
    LoginPortalRecord, LoginSettingsRecord, OutboxRepository, PortalAuthoritySource,
    PortalBindingMutation, PortalGrantOverrideRecord, PortalPolicySnapshot, PortalRepository,
    ProviderLoginAttributes, ReconciliationCause,
};
use crate::shutdown::StopHandle;

#[derive(Clone)]
pub(crate) struct PortalPolicyReconciliationHandle {
    notify: Arc<Notify>,
    all_pending: Arc<AtomicBool>,
    pending_portals: Arc<Mutex<BTreeSet<String>>>,
}

struct PortalPolicyBatch {
    portal: LoginPortalRecord,
    settings: LoginSettingsRecord,
    policy: Option<PortalGrantOverrideRecord>,
    snapshot: PortalPolicySnapshot,
    groups: Arc<BTreeMap<String, CapabilityGroupRecord>>,
    consents: BTreeMap<(String, String), super::ephemeral::BrowserConsentProposal>,
}

impl PortalPolicyReconciliationHandle {
    pub(crate) fn notify_all(&self) {
        self.all_pending.store(true, Ordering::Release);
        self.notify.notify_one();
    }

    pub(crate) async fn notify_portal(&self, portal_id: &str) {
        self.pending_portals
            .lock()
            .await
            .insert(portal_id.to_owned());
        self.notify.notify_one();
    }
}

pub(crate) struct PortalPolicyReconciliationWorker<R> {
    service: AuthService<R>,
    reconciliation: AuthorizationReconciliationHandle,
    handle: PortalPolicyReconciliationHandle,
}

pub(crate) fn portal_policy_reconciliation<R>(
    service: AuthService<R>,
    reconciliation: AuthorizationReconciliationHandle,
) -> (
    PortalPolicyReconciliationHandle,
    PortalPolicyReconciliationWorker<R>,
) {
    let notify = Arc::new(Notify::new());
    let handle = PortalPolicyReconciliationHandle {
        notify,
        all_pending: Arc::new(AtomicBool::new(false)),
        pending_portals: Arc::new(Mutex::new(BTreeSet::new())),
    };
    (
        handle.clone(),
        PortalPolicyReconciliationWorker {
            service,
            reconciliation,
            handle,
        },
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
        loop {
            tokio::select! {
                () = stop.stopped() => return Ok(()),
                () = self.handle.notify.notified() => self.reconcile_pending().await?,
            }
        }
    }

    pub(crate) async fn reconcile_startup(&self) -> Result<(), AuthorizationStateError> {
        self.reconcile_global(true).await
    }

    async fn reconcile_pending(&self) -> Result<(), AuthorizationStateError> {
        loop {
            self.reconcile_pending_portals().await?;
            if self.handle.all_pending.swap(false, Ordering::AcqRel) {
                self.reconcile_global(false).await?;
                continue;
            }
            return Ok(());
        }
    }

    async fn reconcile_pending_portals(&self) -> Result<(), AuthorizationStateError> {
        loop {
            let portal_ids = {
                let mut pending = self.handle.pending_portals.lock().await;
                std::mem::take(&mut *pending)
            };
            if portal_ids.is_empty() {
                return Ok(());
            }
            let bindings = self
                .service
                .repository()
                .list_portal_authority_bindings()
                .await?
                .into_iter()
                .filter(|binding| portal_ids.contains(&binding.portal_id))
                .collect();
            let groups = self.capability_groups().await?;
            self.reconcile_bindings(bindings, false, groups).await?;
        }
    }

    async fn reconcile_global(
        &self,
        materialize_immediately: bool,
    ) -> Result<(), AuthorizationStateError> {
        let groups = self.capability_groups().await?;
        let mut bindings = self
            .service
            .repository()
            .list_portal_authority_bindings()
            .await?
            .into_iter();
        loop {
            self.reconcile_pending_portals().await?;
            let chunk = bindings.by_ref().take(16).collect::<Vec<_>>();
            if chunk.is_empty() {
                return Ok(());
            }
            self.reconcile_bindings(chunk, materialize_immediately, groups.clone())
                .await?;
        }
    }

    async fn capability_groups(
        &self,
    ) -> Result<Arc<BTreeMap<String, CapabilityGroupRecord>>, AuthorizationStateError> {
        Ok(Arc::new(
            self.service
                .repository()
                .list_capability_groups()
                .await?
                .into_iter()
                .map(|group| (group.group_key.clone(), group))
                .collect(),
        ))
    }

    async fn reconcile_bindings(
        &self,
        bindings: Vec<super::PortalAuthorityBindingRecord>,
        materialize_immediately: bool,
        groups: Arc<BTreeMap<String, CapabilityGroupRecord>>,
    ) -> Result<(), AuthorizationStateError> {
        let mut grouped = BTreeMap::<(String, String), Vec<_>>::new();
        for binding in bindings {
            grouped
                .entry((binding.portal_id.clone(), binding.participant_id.clone()))
                .or_default()
                .push(binding);
        }
        let mut work = Vec::new();
        for ((portal_id, participant_id), bindings) in grouped {
            let (portal, settings) = self
                .service
                .repository()
                .get_login_portal(&portal_id)
                .await?
                .ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord(format!(
                        "portal-managed authority references missing portal {portal_id}"
                    ))
                })?;
            let policy = self
                .service
                .repository()
                .get_portal_grant_override(&portal_id, &participant_id)
                .await?;
            let snapshot = portal_policy_snapshot(
                &portal,
                &settings,
                &participant_id,
                policy.as_ref(),
                &groups,
            )?;
            let mut current_bindings = Vec::new();
            for binding in bindings {
                match self
                    .service
                    .repository()
                    .get_identity_authority(&binding.principal_id, &binding.participant_id)
                    .await?
                {
                    Some(current) => current_bindings.push((binding, current)),
                    None => {
                        self.service
                            .repository()
                            .remove_portal_authority_binding(
                                &binding.principal_id,
                                &binding.participant_id,
                            )
                            .await?;
                    }
                }
            }
            let mut consents = BTreeMap::new();
            if policy.is_some() {
                for (_, current) in &current_bindings {
                    let key = (
                        current.participant_artifact_digest.clone(),
                        current.accepted_needs_digest.clone(),
                    );
                    if consents.contains_key(&key) {
                        continue;
                    }
                    let participant = self
                        .service
                        .repository()
                        .get_participant_binding(&current.participant_id, &key.0)
                        .await?
                        .ok_or_else(|| {
                            AuthorizationStateError::InvalidRecord(
                                "portal-managed participant binding is missing".to_owned(),
                            )
                        })?;
                    let consent = browser_consent_proposal(&participant)?;
                    if consent.participant_needs_digest != key.1 {
                        return Err(AuthorizationStateError::StorageConflict);
                    }
                    consents.insert(key, consent);
                }
            }
            let batch = Arc::new(PortalPolicyBatch {
                portal,
                settings,
                policy,
                snapshot,
                groups: groups.clone(),
                consents,
            });
            work.extend(
                current_bindings
                    .into_iter()
                    .map(|(binding, current)| (binding, current, batch.clone())),
            );
        }
        let mut reconciliations = stream::iter(work)
            .map(|(binding, current, batch)| {
                self.reconcile_binding(binding, current, batch, materialize_immediately)
            })
            .buffer_unordered(16);
        while let Some(result) = reconciliations.next().await {
            result?;
        }
        Ok(())
    }

    async fn reconcile_binding(
        &self,
        binding: super::PortalAuthorityBindingRecord,
        current: IdentityAuthorityRecord,
        batch: Arc<PortalPolicyBatch>,
        materialize_immediately: bool,
    ) -> Result<(), AuthorizationStateError> {
        let provider_allowed = super::policy::portal_allows_authenticated_provider(
            &batch.portal,
            &batch.settings,
            &binding.provider_id,
        );
        let now = super::reconciliation::unix_time_millis()?;
        let (state, grant_set, capabilities, replacement, semantic_key) =
            if !provider_allowed || batch.policy.is_none() {
                (
                    AuthorityState::Revoked,
                    current.desired_grant_set.clone(),
                    current.desired_capabilities.clone(),
                    None,
                    None,
                )
            } else {
                let policy = match batch.policy.as_ref() {
                    Some(policy) => policy,
                    None => unreachable!("checked above"),
                };
                let consent = batch
                    .consents
                    .get(&(
                        current.participant_artifact_digest.clone(),
                        current.accepted_needs_digest.clone(),
                    ))
                    .ok_or_else(|| {
                        AuthorizationStateError::InvalidRecord(
                            "portal-managed participant consent is missing".to_owned(),
                        )
                    })?;
                let selection = resolve_portal_authority_selection(
                    policy,
                    &batch.groups,
                    consent,
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
        let retry_portal_id = binding.portal_id.clone();
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
        let input = ApplyIdentityAuthoritySelectionInput {
            principal_id: binding.principal_id.clone(),
            participant_id: binding.participant_id.clone(),
            participant_artifact_digest: current.participant_artifact_digest,
            participant_needs_digest: current.accepted_needs_digest,
            grant_set,
            capabilities,
            state,
            decided_by: "portal-policy-reconciler".to_owned(),
            source_payload: json!({ "source": "portal_policy_reconciliation", "semanticKey": semantic_key }),
            portal_binding: PortalBindingMutation::CompareAndSet {
                expected: Some(binding),
                replacement,
            },
            portal_policy_snapshot: Some(batch.snapshot.clone()),
            decided_at: now,
            expires_at: current.expires_at,
            proposal_idempotency: idempotency(
                "portal.policy.propose",
                &request_digest,
                &request,
                now,
            )?,
            decision_idempotency: idempotency(
                "portal.policy.accept",
                &request_digest,
                &request,
                now,
            )?,
        };
        let result = if materialize_immediately {
            self.service.apply_identity_authority_selection(input).await
        } else {
            self.service
                .commit_identity_authority_selection(input)
                .await
        };
        if matches!(
            result,
            Err(AuthorizationStateError::StorageConflict
                | AuthorizationStateError::PortalPolicyChanged)
        ) {
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.handle.notify_portal(&retry_portal_id).await;
            return Ok(());
        }
        let authority = result?;
        if !materialize_immediately {
            self.reconciliation
                .reconcile(
                    AuthorityTarget::new(AuthorityKind::Identity, authority.authority_id)?,
                    ReconciliationCause::DesiredAuthorityChanged,
                )
                .await?;
        }
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
