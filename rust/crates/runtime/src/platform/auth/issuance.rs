use trellis_protocol::{
    AuthorizationAuthorityRef, AuthorizationParticipant, AuthorizationPrincipal,
};

use super::authority::{AuthorityReconciliationOutcome, IssuanceSnapshot};
use super::materializer::{protocol_authority_kind, protocol_principal_kind};
use super::{
    AuthorityState, AuthorityTarget, AuthorizationStateError, ContextRepository,
    DesiredAuthorityRecord, IssuableAuthorizationState, MaterializationState, PrincipalKind,
    PrincipalState, RuntimeEvidence, SessionState,
};

const MAX_RECONCILIATION_ATTEMPTS: usize = 3;

/// Internal Rust facade that separates authority reconciliation from session issuance.
#[derive(Clone, Debug)]
pub(crate) struct AuthorizationStateService<S> {
    repositories: S,
}

impl<S> AuthorizationStateService<S> {
    /// Construct an authorization-state facade over one coherent repository set.
    #[must_use]
    pub(crate) fn new(repositories: S) -> Self {
        Self { repositories }
    }
}

impl<S> AuthorizationStateService<S>
where
    S: ContextRepository,
{
    /// Recompute or invalidate one authority-level projection.
    ///
    /// The repository owns the coherent unit of work. An optimistic backend may
    /// report a conflict, which is retried a bounded number of times.
    ///
    /// # Errors
    ///
    /// Returns a storage or validation error if no coherent replacement can be
    /// committed after the bounded retry budget.
    pub(crate) async fn reconcile_authority(
        &self,
        target: &AuthorityTarget,
        now: i64,
    ) -> Result<AuthorityReconciliationOutcome, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("now", now)?;
        for attempt in 0..MAX_RECONCILIATION_ATTEMPTS {
            match self.repositories.reconcile_authority(target, now).await {
                Err(AuthorizationStateError::StorageConflict)
                    if attempt + 1 < MAX_RECONCILIATION_ATTEMPTS => {}
                result => return result,
            }
        }
        Err(AuthorizationStateError::StorageConflict)
    }

    /// Reconcile every desired, missing, stale, invalid, or orphaned authority.
    ///
    /// # Errors
    ///
    /// Returns the first storage or validation failure. Callers must not report
    /// readiness after a partial startup reconciliation.
    pub(crate) async fn reconcile_all(
        &self,
        now: i64,
    ) -> Result<Vec<AuthorityReconciliationOutcome>, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("now", now)?;
        let targets = self.repositories.list_reconciliation_targets().await?;
        let mut outcomes = Vec::with_capacity(targets.len());
        for target in targets {
            outcomes.push(self.reconcile_authority(&target, now).await?);
        }
        Ok(outcomes)
    }

    /// Resolve complete unsigned issuable state without rewriting shared authority.
    ///
    /// # Errors
    ///
    /// Returns a stable expected denial for missing, inactive, expired, revoked,
    /// stale, digest-mismatched, or instance-specific ineligible state.
    pub(crate) async fn resolve_issuable_state(
        &self,
        session_id: &str,
        now: i64,
    ) -> Result<IssuableAuthorizationState, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("now", now)?;
        let snapshot = self.repositories.load_issuance_snapshot(session_id).await?;
        resolve_snapshot(snapshot, now)
    }
}

pub(super) fn resolve_snapshot(
    snapshot: IssuanceSnapshot,
    now: i64,
) -> Result<IssuableAuthorizationState, AuthorizationStateError> {
    let session = snapshot
        .session
        .ok_or(AuthorizationStateError::SessionMissing)?;
    match session.state {
        SessionState::Active => {}
        SessionState::Expired => return Err(AuthorizationStateError::SessionExpired),
        SessionState::Revoked => return Err(AuthorizationStateError::SessionRevoked),
    }
    if session.expires_at.is_some_and(|expiry| now >= expiry) {
        return Err(AuthorizationStateError::SessionExpired);
    }
    let principal = snapshot
        .principal
        .ok_or(AuthorizationStateError::PrincipalMissing)?;
    if principal.kind != session.principal_kind || principal.state != PrincipalState::Active {
        return Err(AuthorizationStateError::PrincipalInactive);
    }
    let binding = snapshot
        .participant
        .ok_or(AuthorizationStateError::ParticipantMissing)?;
    if binding.participant_id != session.participant_id
        || binding.participant_kind != session.participant_kind
        || binding.artifact_digest != session.participant_artifact_digest
    {
        return Err(AuthorizationStateError::ParticipantDigestMismatch);
    }
    if binding.needs_digest != session.participant_needs_digest {
        return Err(AuthorizationStateError::NeedsDigestMismatch);
    }
    binding.resolve()?;

    let runtime = snapshot
        .runtime
        .ok_or(AuthorizationStateError::DeploymentInactive)?;
    validate_runtime(principal.kind, &runtime, snapshot.deployment.as_ref(), now)?;
    let authority = snapshot
        .authority
        .ok_or(AuthorizationStateError::AuthorityMissing)?;
    validate_authority(&authority, now)?;
    if authority.participant_id() != session.participant_id
        || authority.participant_artifact_digest() != session.participant_artifact_digest
    {
        return Err(AuthorizationStateError::ParticipantDigestMismatch);
    }
    if authority.accepted_needs_digest() != session.participant_needs_digest {
        return Err(AuthorizationStateError::NeedsDigestMismatch);
    }
    let expected_target = authority.target();
    let (deployment_id, instance_id, delegation_expires_at) = runtime_identity(&runtime);
    match (&authority, &runtime) {
        (DesiredAuthorityRecord::Identity(record), RuntimeEvidence::User)
            if record.principal_id == principal.principal_id => {}
        (DesiredAuthorityRecord::Deployment(record), RuntimeEvidence::Service(evidence))
            if record.deployment_id == evidence.deployment_id => {}
        (DesiredAuthorityRecord::Deployment(record), RuntimeEvidence::Device(evidence))
            if record.deployment_id == evidence.deployment_id => {}
        _ => {
            return Err(AuthorizationStateError::InvalidRecord(
                "session eligibility does not match authority subject".to_owned(),
            ));
        }
    }

    let materialized = snapshot
        .materialization
        .ok_or(AuthorizationStateError::MaterializationStale)?;
    let resource_bindings = materialized.resources;
    let header = materialized.authority;
    let effective_authority_expires_at = match (&authority, snapshot.deployment.as_ref()) {
        (DesiredAuthorityRecord::Deployment(record), Some(deployment)) => {
            min_expiry(record.expires_at, deployment.expires_at)
        }
        _ => authority.expires_at(),
    };
    if header.state != MaterializationState::Available
        || header.authority_kind != expected_target.kind
        || header.authority_id != expected_target.authority_id
        || header.authority_version != authority.version()
        || header.subject_id != authority.subject_id()
        || header.participant_id != session.participant_id
        || header.participant_artifact_digest != session.participant_artifact_digest
        || header.participant_needs_digest != session.participant_needs_digest
        || header.expires_at != effective_authority_expires_at
        || header.expires_at.is_some_and(|expiry| now >= expiry)
    {
        tracing::warn!(
            materialization_state = ?header.state,
            materialization_error = ?header.error,
            materialization_authority_version = header.authority_version,
            authority_version = authority.version(),
            materialization_participant_id = %header.participant_id,
            session_participant_id = %session.participant_id,
            materialization_artifact_digest = %header.participant_artifact_digest,
            session_artifact_digest = %session.participant_artifact_digest,
            materialization_needs_digest = %header.participant_needs_digest,
            session_needs_digest = %session.participant_needs_digest,
            materialization_expires_at = ?header.expires_at,
            effective_authority_expires_at = ?effective_authority_expires_at,
            "issuable authorization materialization does not match current session authority"
        );
        return Err(AuthorizationStateError::MaterializationStale);
    }

    Ok(IssuableAuthorizationState {
        principal: AuthorizationPrincipal {
            kind: protocol_principal_kind(principal.kind),
            id: principal.principal_id,
        },
        session_id: session.session_id,
        session_public_key: session.session_public_key,
        session_key_id: session.session_key_id,
        inbox_prefix: session.inbox_prefix,
        participant: AuthorizationParticipant {
            kind: session.participant_kind,
            id: session.participant_id,
            artifact_digest: session.participant_artifact_digest,
            needs_digest: session.participant_needs_digest,
        },
        authority_ref: AuthorizationAuthorityRef {
            kind: protocol_authority_kind(expected_target.kind),
            id: expected_target.authority_id,
            version: header.authority_version,
        },
        deployment_id,
        instance_id,
        grant_set: header.effective_grant_set,
        resource_bindings,
        capabilities: header.effective_capabilities,
        session_expires_at: session.expires_at,
        effective_authority_expires_at: header.expires_at,
        delegation_expires_at,
        materialization_version: header.materialization_version,
    })
}

fn min_expiry(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn validate_authority(
    authority: &DesiredAuthorityRecord,
    now: i64,
) -> Result<(), AuthorizationStateError> {
    match authority.state() {
        AuthorityState::Accepted => {}
        AuthorityState::Pending => return Err(AuthorizationStateError::AuthorityPending),
        AuthorityState::Rejected => return Err(AuthorizationStateError::AuthorityRejected),
        AuthorityState::Revoked => return Err(AuthorizationStateError::AuthorityRevoked),
        AuthorityState::Stale => return Err(AuthorizationStateError::AuthorityStale),
    }
    if authority.expires_at().is_some_and(|expiry| now >= expiry) {
        return Err(AuthorizationStateError::AuthorityExpired);
    }
    Ok(())
}

fn validate_runtime(
    principal: PrincipalKind,
    evidence: &RuntimeEvidence,
    deployment: Option<&super::DeploymentRecord>,
    now: i64,
) -> Result<(), AuthorizationStateError> {
    match (principal, evidence) {
        (PrincipalKind::User, RuntimeEvidence::User) if deployment.is_none() => Ok(()),
        (PrincipalKind::Service, RuntimeEvidence::Service(value)) => {
            validate_deployment(deployment, &value.deployment_id, now)?;
            if !value.instance_active {
                return Err(AuthorizationStateError::InstanceInactive);
            }
            Ok(())
        }
        (PrincipalKind::Device, RuntimeEvidence::Device(value)) => {
            validate_deployment(deployment, &value.deployment_id, now)?;
            if !value.device_active {
                return Err(AuthorizationStateError::DeviceInactive);
            }
            if !value.instance_active {
                return Err(AuthorizationStateError::InstanceInactive);
            }
            if let Some(delegation) = &value.delegation {
                if delegation.required && !delegation.active {
                    return Err(AuthorizationStateError::ActivationMissing);
                }
                if delegation
                    .expires_at
                    .is_some_and(|expiry| delegation.required && now >= expiry)
                {
                    return Err(AuthorizationStateError::DelegationExpired);
                }
            }
            Ok(())
        }
        _ => Err(AuthorizationStateError::InvalidRecord(
            "runtime evidence does not match principal kind".to_owned(),
        )),
    }
}

fn validate_deployment(
    deployment: Option<&super::DeploymentRecord>,
    expected_id: &str,
    now: i64,
) -> Result<(), AuthorizationStateError> {
    let deployment = deployment.ok_or(AuthorizationStateError::DeploymentInactive)?;
    if deployment.deployment_id != expected_id
        || !deployment.active
        || deployment.expires_at.is_some_and(|expiry| now >= expiry)
    {
        return Err(AuthorizationStateError::DeploymentInactive);
    }
    Ok(())
}

fn runtime_identity(evidence: &RuntimeEvidence) -> (Option<String>, Option<String>, Option<i64>) {
    match evidence {
        RuntimeEvidence::User => (None, None, None),
        RuntimeEvidence::Service(value) => (
            Some(value.deployment_id.clone()),
            Some(value.instance_id.clone()),
            None,
        ),
        RuntimeEvidence::Device(value) => (
            Some(value.deployment_id.clone()),
            Some(value.instance_id.clone()),
            value
                .delegation
                .as_ref()
                .and_then(|delegation| delegation.expires_at),
        ),
    }
}
