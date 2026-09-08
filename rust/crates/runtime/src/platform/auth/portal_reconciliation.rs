use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{stream, StreamExt};
use serde_json::{json, Value};
use tokio::sync::{Mutex, Notify};

use super::{
    browser_consent_proposal, portal_policy_snapshot, resolve_portal_authority_selection,
    AccountRepository, AuthService, AuthorizationStateError, CapabilityGroupRecord, GrantBinding,
    GrantBindingReplacement, GrantBindingState, GrantOwnerKind, GrantRepository,
    IdempotencyResultRecord, LoginPortalRecord, LoginSettingsRecord, PortalGrantOverrideRecord,
    PortalGrantProvenance, PortalPolicySnapshot, PortalRepository, ProviderLoginAttributes,
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
    handle: PortalPolicyReconciliationHandle,
}

pub(crate) fn portal_policy_reconciliation<R>(
    service: AuthService<R>,
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
        PortalPolicyReconciliationWorker { service, handle },
    )
}

impl<R> PortalPolicyReconciliationWorker<R>
where
    R: AccountRepository + GrantRepository + PortalRepository + Clone + Send + Sync,
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
                .list_portal_grant_bindings()
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
            .list_portal_grant_bindings()
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
        bindings: Vec<super::PortalGrantBindingRecord>,
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
                if let Some(current) = self
                    .service
                    .repository()
                    .get_grant_binding(
                        GrantOwnerKind::User,
                        binding.principal_id.clone(),
                        binding.participant_id.clone(),
                    )
                    .await?
                {
                    current_bindings.push((binding, current));
                }
            }
            let batch = Arc::new(PortalPolicyBatch {
                portal,
                settings,
                policy,
                snapshot,
                groups: groups.clone(),
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
        binding: super::PortalGrantBindingRecord,
        current: GrantBinding,
        batch: Arc<PortalPolicyBatch>,
        _materialize_immediately: bool,
    ) -> Result<(), AuthorizationStateError> {
        let provider_allowed = super::policy::portal_allows_authenticated_provider(
            &batch.portal,
            &batch.settings,
            &binding.provider_id,
        );
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?
            .as_millis()
            .try_into()
            .map_err(|_| AuthorizationStateError::Storage("current time exceeds i64".to_owned()))?;
        let retry_portal_id = binding.portal_id.clone();
        let result = if !provider_allowed || batch.policy.is_none() {
            let request = json!({
                "principalId": binding.principal_id,
                "participantId": binding.participant_id,
                "expectedRevision": current.revision,
                "state": "revoked",
            });
            let request_digest = trellis_protocol::digest_json(&request)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            self.service
                .repository()
                .revoke_portal_grant_binding(
                    binding.principal_id,
                    binding.participant_id,
                    current.revision,
                    batch.snapshot.clone(),
                    idempotency("portal.policy.revoke", &request_digest, &request, now)?,
                )
                .await
                .map(|_| ())
        } else {
            let (_, participant) = self
                .service
                .repository()
                .get_installed_participant_record(
                    binding.participant_id.clone(),
                    Some(current.installed_revision),
                )
                .await?
                .ok_or(AuthorizationStateError::ParticipantMissing)?;
            let consent = browser_consent_proposal(&participant)?;
            let selection = resolve_portal_authority_selection(
                batch.policy.as_ref().expect("checked above"),
                &batch.groups,
                &consent,
                &ProviderLoginAttributes {
                    provider_id: binding.provider_id.clone(),
                    roles: binding.roles.clone(),
                },
            )?;
            let provenance = PortalGrantProvenance {
                portal_id: binding.portal_id.clone(),
                provider_id: binding.provider_id.clone(),
                roles: binding.roles.clone(),
                effective_policy_digest: selection.effective_policy_digest.clone(),
            };
            if current.state == GrantBindingState::Active
                && current.grants == selection.grant_set
                && current.platform_privileges.is_empty()
                && current.provenance.as_ref() == Some(&provenance)
            {
                return Ok(());
            }
            let request = json!({
                "principalId": binding.principal_id,
                "participantId": binding.participant_id,
                "expectedRevision": current.revision,
                "effectivePolicyDigest": selection.effective_policy_digest,
            });
            let request_digest = trellis_protocol::digest_json(&request)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            self.service
                .repository()
                .set_portal_grant_binding(
                    GrantBindingReplacement {
                        owner_kind: GrantOwnerKind::User,
                        owner_id: binding.principal_id,
                        participant_id: binding.participant_id,
                        installed_revision: current.installed_revision,
                        grants: selection.grant_set,
                        platform_privileges: Vec::new(),
                        state: GrantBindingState::Active,
                        expires_at: current.expires_at,
                        provenance: Some(provenance),
                        expected_revision: current.revision,
                    },
                    batch.snapshot.clone(),
                    idempotency("portal.policy.replace", &request_digest, &request, now)?,
                )
                .await
                .map(|_| ())
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
