use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};

use super::domain::{
    canonical_capabilities, require_digest, require_nonempty, require_positive,
    require_protocol_timestamp, validate_principal_participant,
};
use super::materializer::{materialize_authority, transition_for_change};
use super::{
    AuthorityEvidenceScope, AuthorityKind, AuthorityState, AuthorityTarget,
    AuthorizationStateError, AuthorizationTransitionOutboxRecord, DelegationEvidence,
    DependencyEvidence, DeploymentAuthorityRecord, DeploymentRecord, DesiredAuthorityRecord,
    DeviceDelegationRecord, DeviceDelegationState, DeviceEvidence, DeviceRecord, DeviceState,
    IdentityAuthorityRecord, MaterializedAuthorityRecord, ParticipantBindingRecord,
    PrincipalAuthorizationChange, PrincipalKind, PrincipalRecord, PrincipalState,
    ProviderIdentityLink, ResourceBindingEvidence, RuntimeEvidence, RuntimeInstanceRecord,
    RuntimeInstanceState, ServiceEvidence, SessionRecord, SessionRuntimeBinding, SessionState,
};

/// Atomic materialized-authority replacement and its exact supporting evidence.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializationReplacement {
    /// New materialized-authority header and effective permissions.
    pub authority: MaterializedAuthorityRecord,
    /// Exact dependency evidence used by the result.
    pub dependencies: Vec<DependencyEvidence>,
    /// Exact resource-binding evidence used by the result.
    pub resources: Vec<ResourceBindingEvidence>,
}

/// Opaque digest of every authority-level input read by one coherent snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritySnapshotToken(pub String);

/// Principal or deployment record that owns one authority-level materialization.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "record")]
pub enum AuthoritySubjectRecord {
    /// Identity authority is owned by one user principal.
    Identity(PrincipalRecord),
    /// Deployment authority is owned by one service or device deployment.
    Deployment(DeploymentRecord),
}

/// Complete authority-level input read under one repository unit of work.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityMaterializationSnapshot {
    /// Typed authority target.
    pub target: AuthorityTarget,
    /// Revision token covering every field in this snapshot.
    pub token: AuthoritySnapshotToken,
    /// Desired authority, absent only for an orphaned materialization.
    pub authority: Option<DesiredAuthorityRecord>,
    /// Authority-level principal or deployment subject.
    pub subject: Option<AuthoritySubjectRecord>,
    /// Exact participant binding selected by desired authority.
    pub participant: Option<ParticipantBindingRecord>,
    /// Exact authority-scoped dependency evidence.
    pub dependencies: Vec<DependencyEvidence>,
    /// Persisted scope of dependency evidence when present.
    pub dependency_scope: Option<AuthorityEvidenceScope>,
    /// Exact authority-scoped resource evidence.
    pub resources: Vec<ResourceBindingEvidence>,
    /// Persisted scope of resource evidence when present.
    pub resource_scope: Option<AuthorityEvidenceScope>,
    /// Previous durable projection.
    pub previous: Option<MaterializationReplacement>,
}

/// Coherent session-level input used only to decide whether issuance is allowed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuanceSnapshot {
    /// Requested session when it exists.
    pub session: Option<SessionRecord>,
    /// Session principal when it exists.
    pub principal: Option<PrincipalRecord>,
    /// Exact participant binding when it exists.
    pub participant: Option<ParticipantBindingRecord>,
    /// Session-specific deployment, instance, and delegation eligibility.
    pub runtime: Option<RuntimeEvidence>,
    /// Current authority-level deployment state for service or device issuance.
    pub deployment: Option<DeploymentRecord>,
    /// Desired authority selected from the session and runtime evidence.
    pub authority: Option<DesiredAuthorityRecord>,
    /// Current shared authority materialization.
    pub materialization: Option<MaterializationReplacement>,
}

/// Result of one coherent authority reconciliation transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityReconciliationOutcome {
    /// Typed reconciled target.
    pub target: AuthorityTarget,
    /// Complete input revision used by the replacement.
    pub snapshot_token: AuthoritySnapshotToken,
    /// Current projection, or `None` when an orphan was removed.
    pub materialization: Option<MaterializationReplacement>,
    /// Whether the transaction changed durable semantics.
    pub changed: bool,
}

/// Durable principal repository port.
#[async_trait]
pub trait PrincipalRepository: Send + Sync {
    /// Load a principal by stable ID.
    async fn get_principal(
        &self,
        id: &str,
    ) -> Result<Option<PrincipalRecord>, AuthorizationStateError>;

    /// Create a principal at authorization version one.
    async fn create_principal(
        &self,
        record: PrincipalRecord,
    ) -> Result<PrincipalRecord, AuthorizationStateError>;

    /// Apply one authorization state transition with an optimistic version guard.
    async fn update_principal_authorization_state(
        &self,
        id: &str,
        expected_version: u64,
        change: PrincipalAuthorizationChange,
    ) -> Result<PrincipalRecord, AuthorizationStateError>;
}

/// External-provider identity linkage repository port.
#[async_trait]
pub trait ProviderIdentityRepository: Send + Sync {
    /// Load a provider identity by its exact provider and subject.
    async fn get_provider_identity(
        &self,
        provider: &str,
        subject: &str,
    ) -> Result<Option<ProviderIdentityLink>, AuthorizationStateError>;

    /// Create one unique provider identity link.
    async fn link_provider_identity(
        &self,
        link: ProviderIdentityLink,
    ) -> Result<(), AuthorizationStateError>;
}

/// Durable authenticated-session repository port.
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Load a session by stable ID.
    async fn get_session(&self, id: &str)
        -> Result<Option<SessionRecord>, AuthorizationStateError>;

    /// Create a validated participant-bound session.
    async fn create_session(
        &self,
        record: SessionRecord,
    ) -> Result<SessionRecord, AuthorizationStateError>;

    /// Update liveness only when the session remains active.
    async fn touch_session(
        &self,
        id: &str,
        observed_at: i64,
    ) -> Result<(), AuthorizationStateError>;

    /// Revoke a session with an optimistic version guard.
    async fn revoke_session(
        &self,
        id: &str,
        expected_version: u64,
        revoked_at: i64,
    ) -> Result<SessionRecord, AuthorizationStateError>;

    /// Mark a session expired with an optimistic version guard.
    async fn expire_session(
        &self,
        id: &str,
        expected_version: u64,
        expired_at: i64,
    ) -> Result<SessionRecord, AuthorizationStateError>;

    /// Replace the exact participant binding through an explicit versioned rebind.
    async fn rebind_session(
        &self,
        id: &str,
        expected_version: u64,
        participant: &ParticipantBindingRecord,
    ) -> Result<SessionRecord, AuthorizationStateError>;

    /// List sessions for one stable principal.
    async fn list_sessions_for_principal(
        &self,
        principal_id: &str,
    ) -> Result<Vec<SessionRecord>, AuthorizationStateError>;
}

/// Exact participant binding repository port.
#[async_trait]
pub trait ParticipantBindingRepository: Send + Sync {
    /// Load one exact participant artifact by ID and digest.
    async fn get_participant_binding(
        &self,
        participant_id: &str,
        artifact_digest: &str,
    ) -> Result<Option<ParticipantBindingRecord>, AuthorizationStateError>;

    /// Persist a binding only after exact protocol resolution succeeds.
    async fn put_participant_binding(
        &self,
        binding: ParticipantBindingRecord,
    ) -> Result<(), AuthorizationStateError>;
}

/// Desired identity-authority repository port.
#[async_trait]
pub trait IdentityAuthorityRepository: Send + Sync {
    /// Load current identity authority for one principal and participant.
    async fn get_identity_authority(
        &self,
        principal_id: &str,
        participant_id: &str,
    ) -> Result<Option<IdentityAuthorityRecord>, AuthorizationStateError>;

    /// Create or compare-and-swap the current identity authority.
    async fn put_identity_authority(
        &self,
        record: IdentityAuthorityRecord,
        expected_version: Option<u64>,
    ) -> Result<IdentityAuthorityRecord, AuthorizationStateError>;
}

/// Desired deployment-authority repository port.
#[async_trait]
pub trait DeploymentAuthorityRepository: Send + Sync {
    /// Load current deployment authority for one deployment and participant.
    async fn get_deployment_authority(
        &self,
        deployment_id: &str,
        participant_id: &str,
    ) -> Result<Option<DeploymentAuthorityRecord>, AuthorizationStateError>;

    /// Create or compare-and-swap the current deployment authority.
    async fn put_deployment_authority(
        &self,
        record: DeploymentAuthorityRecord,
        expected_version: Option<u64>,
    ) -> Result<DeploymentAuthorityRecord, AuthorizationStateError>;
}

/// Narrow runtime evidence repository port used by materialization.
#[async_trait]
pub trait EvidenceRepository: Send + Sync {
    /// Load current authority-level deployment evidence.
    async fn get_deployment_evidence(
        &self,
        deployment_id: &str,
    ) -> Result<Option<DeploymentRecord>, AuthorizationStateError>;

    /// Replace current authority-level deployment evidence.
    async fn put_deployment_evidence(
        &self,
        deployment: DeploymentRecord,
    ) -> Result<(), AuthorizationStateError>;

    /// Load one stable runtime instance.
    async fn get_runtime_instance(
        &self,
        instance_id: &str,
    ) -> Result<Option<RuntimeInstanceRecord>, AuthorizationStateError>;

    /// Create or update one runtime instance without changing its stable ownership.
    async fn put_runtime_instance(
        &self,
        instance: RuntimeInstanceRecord,
    ) -> Result<(), AuthorizationStateError>;

    /// Load one deployment-scoped durable device record.
    async fn get_device(
        &self,
        principal_id: &str,
        deployment_id: &str,
    ) -> Result<Option<DeviceRecord>, AuthorizationStateError>;

    /// Create or update one deployment-scoped durable device record.
    async fn put_device(&self, device: DeviceRecord) -> Result<(), AuthorizationStateError>;

    /// Load device delegation evidence at its complete identity.
    async fn get_device_delegation(
        &self,
        principal_id: &str,
        deployment_id: &str,
    ) -> Result<Option<DeviceDelegationRecord>, AuthorizationStateError>;

    /// Create or replace deployment-scoped device delegation evidence.
    async fn put_device_delegation(
        &self,
        delegation: DeviceDelegationRecord,
    ) -> Result<(), AuthorizationStateError>;

    /// Load the runtime selection owned by one session.
    async fn get_session_runtime_binding(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionRuntimeBinding>, AuthorizationStateError>;

    /// Create or replace a session runtime selection without owning selected evidence.
    async fn put_session_runtime_binding(
        &self,
        binding: SessionRuntimeBinding,
    ) -> Result<(), AuthorizationStateError>;

    /// Remove only a session runtime selection, preserving shared runtime evidence.
    async fn remove_session_runtime_binding(
        &self,
        session_id: &str,
    ) -> Result<(), AuthorizationStateError>;

    /// Load session-specific deployment, instance, device, and delegation evidence.
    async fn get_runtime_evidence(
        &self,
        session_id: &str,
    ) -> Result<Option<RuntimeEvidence>, AuthorizationStateError>;

    /// Load current dependency evidence for one typed authority target.
    async fn list_dependency_evidence(
        &self,
        target: &AuthorityTarget,
    ) -> Result<Vec<DependencyEvidence>, AuthorizationStateError>;

    /// Replace dependency evidence at an exact authority and participant scope.
    async fn replace_dependency_evidence(
        &self,
        scope: AuthorityEvidenceScope,
        evidence: Vec<DependencyEvidence>,
    ) -> Result<(), AuthorizationStateError>;

    /// Load current resource-binding evidence for one typed authority target.
    async fn list_resource_evidence(
        &self,
        target: &AuthorityTarget,
    ) -> Result<Vec<ResourceBindingEvidence>, AuthorizationStateError>;

    /// Replace resource evidence at an exact authority and participant scope.
    async fn replace_resource_evidence(
        &self,
        scope: AuthorityEvidenceScope,
        evidence: Vec<ResourceBindingEvidence>,
    ) -> Result<(), AuthorizationStateError>;
}

/// Coherent materialization, issuance snapshot, and transition-outbox repository port.
#[async_trait]
pub trait AuthorizationMaterializationRepository: Send + Sync {
    /// Load the current materialization by desired-authority identity.
    async fn get_materialized_authority(
        &self,
        kind: AuthorityKind,
        authority_id: &str,
    ) -> Result<Option<MaterializationReplacement>, AuthorizationStateError>;

    /// Recompute or invalidate one authority under a single coherent unit of work.
    async fn reconcile_authority(
        &self,
        target: &AuthorityTarget,
        now: i64,
    ) -> Result<AuthorityReconciliationOutcome, AuthorizationStateError>;

    /// Enumerate the union of desired and materialized authority targets.
    async fn list_reconciliation_targets(
        &self,
    ) -> Result<Vec<AuthorityTarget>, AuthorizationStateError>;

    /// Read all session-level issuance inputs under one coherent snapshot.
    async fn load_issuance_snapshot(
        &self,
        session_id: &str,
    ) -> Result<IssuanceSnapshot, AuthorizationStateError>;

    /// List pending deterministic transition outbox records in creation order.
    async fn list_transition_outbox(
        &self,
        limit: usize,
    ) -> Result<Vec<AuthorizationTransitionOutboxRecord>, AuthorizationStateError>;

    /// Acknowledge successful delivery of one deterministic transition.
    async fn acknowledge_transition(&self, event_id: &str) -> Result<(), AuthorizationStateError>;
}

#[derive(Debug, Default)]
struct MemoryState {
    principals: BTreeMap<String, PrincipalRecord>,
    provider_identities: BTreeMap<(String, String), ProviderIdentityLink>,
    sessions: BTreeMap<String, SessionRecord>,
    session_key_ids: BTreeMap<String, String>,
    participant_bindings: BTreeMap<(String, String), ParticipantBindingRecord>,
    identity_authorities: BTreeMap<(String, String), IdentityAuthorityRecord>,
    deployment_authorities: BTreeMap<(String, String), DeploymentAuthorityRecord>,
    deployments: BTreeMap<String, DeploymentRecord>,
    runtime_instances: BTreeMap<String, RuntimeInstanceRecord>,
    session_runtime_bindings: BTreeMap<String, SessionRuntimeBinding>,
    devices: BTreeMap<(String, String), DeviceRecord>,
    device_delegations: BTreeMap<(String, String), DeviceDelegationRecord>,
    dependency_evidence:
        BTreeMap<AuthorityTarget, (AuthorityEvidenceScope, Vec<DependencyEvidence>)>,
    resource_evidence:
        BTreeMap<AuthorityTarget, (AuthorityEvidenceScope, Vec<ResourceBindingEvidence>)>,
    materializations: BTreeMap<(AuthorityKind, String), MaterializationReplacement>,
    transition_outbox: BTreeMap<String, AuthorizationTransitionOutboxRecord>,
}

/// Constraint-faithful in-memory authorization repositories for pure tests and examples.
#[derive(Clone, Debug, Default)]
pub struct InMemoryAuthorizationStore {
    state: Arc<Mutex<MemoryState>>,
}

impl InMemoryAuthorizationStore {
    fn state(&self) -> Result<std::sync::MutexGuard<'_, MemoryState>, AuthorizationStateError> {
        self.state
            .lock()
            .map_err(|_| AuthorizationStateError::Storage("in-memory lock poisoned".to_owned()))
    }
}

#[async_trait]
impl PrincipalRepository for InMemoryAuthorizationStore {
    async fn get_principal(
        &self,
        id: &str,
    ) -> Result<Option<PrincipalRecord>, AuthorizationStateError> {
        Ok(self.state()?.principals.get(id).cloned())
    }

    async fn create_principal(
        &self,
        record: PrincipalRecord,
    ) -> Result<PrincipalRecord, AuthorizationStateError> {
        validate_principal(&record)?;
        let mut state = self.state()?;
        if state.principals.contains_key(&record.principal_id) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        state
            .principals
            .insert(record.principal_id.clone(), record.clone());
        Ok(record)
    }

    async fn update_principal_authorization_state(
        &self,
        id: &str,
        expected_version: u64,
        change: PrincipalAuthorizationChange,
    ) -> Result<PrincipalRecord, AuthorizationStateError> {
        require_protocol_timestamp("changedAt", change.changed_at)?;
        let mut state = self.state()?;
        let record = state
            .principals
            .get_mut(id)
            .ok_or(AuthorizationStateError::PrincipalMissing)?;
        if record.version != expected_version {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if record.state == change.state {
            return Ok(record.clone());
        }
        if record.state == PrincipalState::Revoked {
            return Err(AuthorizationStateError::StorageConflict);
        }
        record.state = change.state;
        record.updated_at = change.changed_at;
        record.version = next_version("version", record.version)?;
        record.disabled_at =
            (change.state == PrincipalState::Disabled).then_some(change.changed_at);
        record.revoked_at = (change.state == PrincipalState::Revoked).then_some(change.changed_at);
        Ok(record.clone())
    }
}

#[async_trait]
impl ProviderIdentityRepository for InMemoryAuthorizationStore {
    async fn get_provider_identity(
        &self,
        provider: &str,
        subject: &str,
    ) -> Result<Option<ProviderIdentityLink>, AuthorizationStateError> {
        Ok(self
            .state()?
            .provider_identities
            .get(&(provider.to_owned(), subject.to_owned()))
            .cloned())
    }

    async fn link_provider_identity(
        &self,
        link: ProviderIdentityLink,
    ) -> Result<(), AuthorizationStateError> {
        validate_provider_identity(&link)?;
        let mut state = self.state()?;
        let principal = state
            .principals
            .get(&link.principal_id)
            .ok_or(AuthorizationStateError::PrincipalMissing)?;
        if principal.kind != PrincipalKind::User {
            return Err(AuthorizationStateError::InvalidRecord(
                "provider identities may link only to user principals".to_owned(),
            ));
        }
        let key = (link.provider.clone(), link.provider_subject.clone());
        if state.provider_identities.contains_key(&key) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        state.provider_identities.insert(key, link);
        Ok(())
    }
}

#[async_trait]
impl ParticipantBindingRepository for InMemoryAuthorizationStore {
    async fn get_participant_binding(
        &self,
        participant_id: &str,
        artifact_digest: &str,
    ) -> Result<Option<ParticipantBindingRecord>, AuthorizationStateError> {
        Ok(self
            .state()?
            .participant_bindings
            .get(&(participant_id.to_owned(), artifact_digest.to_owned()))
            .cloned())
    }

    async fn put_participant_binding(
        &self,
        binding: ParticipantBindingRecord,
    ) -> Result<(), AuthorizationStateError> {
        require_protocol_timestamp("resolvedAt", binding.resolved_at)?;
        binding.resolve()?;
        self.state()?.participant_bindings.insert(
            (
                binding.participant_id.clone(),
                binding.artifact_digest.clone(),
            ),
            binding,
        );
        Ok(())
    }
}

#[async_trait]
impl SessionRepository for InMemoryAuthorizationStore {
    async fn get_session(
        &self,
        id: &str,
    ) -> Result<Option<SessionRecord>, AuthorizationStateError> {
        Ok(self.state()?.sessions.get(id).cloned())
    }

    async fn create_session(
        &self,
        record: SessionRecord,
    ) -> Result<SessionRecord, AuthorizationStateError> {
        validate_session(&record)?;
        let mut state = self.state()?;
        let principal = state
            .principals
            .get(&record.principal_id)
            .ok_or(AuthorizationStateError::PrincipalMissing)?;
        if principal.kind != record.principal_kind {
            return Err(AuthorizationStateError::InvalidRecord(
                "session principal kind does not match principal".to_owned(),
            ));
        }
        let binding_key = (
            record.participant_id.clone(),
            record.participant_artifact_digest.clone(),
        );
        let binding = state
            .participant_bindings
            .get(&binding_key)
            .ok_or(AuthorizationStateError::ParticipantMissing)?;
        if binding.participant_kind != record.participant_kind {
            return Err(AuthorizationStateError::InvalidRecord(
                "session participant kind does not match participant binding".to_owned(),
            ));
        }
        if binding.needs_digest != record.participant_needs_digest {
            return Err(AuthorizationStateError::NeedsDigestMismatch);
        }
        if state.sessions.contains_key(&record.session_id)
            || state.session_key_ids.contains_key(&record.session_key_id)
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        state
            .session_key_ids
            .insert(record.session_key_id.clone(), record.session_id.clone());
        state
            .sessions
            .insert(record.session_id.clone(), record.clone());
        Ok(record)
    }

    async fn touch_session(
        &self,
        id: &str,
        observed_at: i64,
    ) -> Result<(), AuthorizationStateError> {
        require_protocol_timestamp("observedAt", observed_at)?;
        let mut state = self.state()?;
        let record = state
            .sessions
            .get_mut(id)
            .ok_or(AuthorizationStateError::SessionMissing)?;
        if record.state != SessionState::Active {
            return match record.state {
                SessionState::Expired => Err(AuthorizationStateError::SessionExpired),
                SessionState::Revoked => Err(AuthorizationStateError::SessionRevoked),
                SessionState::Active => Ok(()),
            };
        }
        record.last_seen_at = record.last_seen_at.max(observed_at);
        Ok(())
    }

    async fn revoke_session(
        &self,
        id: &str,
        expected_version: u64,
        revoked_at: i64,
    ) -> Result<SessionRecord, AuthorizationStateError> {
        require_protocol_timestamp("revokedAt", revoked_at)?;
        let mut state = self.state()?;
        let record = state
            .sessions
            .get_mut(id)
            .ok_or(AuthorizationStateError::SessionMissing)?;
        if record.version != expected_version || record.state != SessionState::Active {
            return Err(AuthorizationStateError::StorageConflict);
        }
        record.state = SessionState::Revoked;
        record.revoked_at = Some(revoked_at);
        record.version = next_version("version", record.version)?;
        Ok(record.clone())
    }

    async fn expire_session(
        &self,
        id: &str,
        expected_version: u64,
        expired_at: i64,
    ) -> Result<SessionRecord, AuthorizationStateError> {
        require_protocol_timestamp("expiredAt", expired_at)?;
        let mut state = self.state()?;
        let record = state
            .sessions
            .get_mut(id)
            .ok_or(AuthorizationStateError::SessionMissing)?;
        if record.version != expected_version || record.state != SessionState::Active {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if record.expires_at.is_some_and(|expiry| expired_at < expiry) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        record.state = SessionState::Expired;
        record.version = next_version("version", record.version)?;
        Ok(record.clone())
    }

    async fn rebind_session(
        &self,
        id: &str,
        expected_version: u64,
        participant: &ParticipantBindingRecord,
    ) -> Result<SessionRecord, AuthorizationStateError> {
        participant.resolve()?;
        let mut state = self.state()?;
        if !state.participant_bindings.contains_key(&(
            participant.participant_id.clone(),
            participant.artifact_digest.clone(),
        )) {
            return Err(AuthorizationStateError::ParticipantMissing);
        }
        let record = state
            .sessions
            .get_mut(id)
            .ok_or(AuthorizationStateError::SessionMissing)?;
        validate_principal_participant(record.principal_kind, participant.participant_kind)?;
        if record.version != expected_version || record.state != SessionState::Active {
            return Err(AuthorizationStateError::StorageConflict);
        }
        record
            .participant_id
            .clone_from(&participant.participant_id);
        record.participant_kind = participant.participant_kind;
        record
            .participant_artifact_digest
            .clone_from(&participant.artifact_digest);
        record
            .participant_needs_digest
            .clone_from(&participant.needs_digest);
        record.version = next_version("version", record.version)?;
        Ok(record.clone())
    }

    async fn list_sessions_for_principal(
        &self,
        principal_id: &str,
    ) -> Result<Vec<SessionRecord>, AuthorizationStateError> {
        Ok(self
            .state()?
            .sessions
            .values()
            .filter(|record| record.principal_id == principal_id)
            .cloned()
            .collect())
    }
}

#[async_trait]
impl IdentityAuthorityRepository for InMemoryAuthorizationStore {
    async fn get_identity_authority(
        &self,
        principal_id: &str,
        participant_id: &str,
    ) -> Result<Option<IdentityAuthorityRecord>, AuthorizationStateError> {
        Ok(self
            .state()?
            .identity_authorities
            .get(&(principal_id.to_owned(), participant_id.to_owned()))
            .cloned())
    }

    async fn put_identity_authority(
        &self,
        mut record: IdentityAuthorityRecord,
        expected_version: Option<u64>,
    ) -> Result<IdentityAuthorityRecord, AuthorizationStateError> {
        validate_identity_authority(&mut record)?;
        let mut state = self.state()?;
        if state
            .principals
            .get(&record.principal_id)
            .is_none_or(|principal| principal.kind != PrincipalKind::User)
        {
            return Err(AuthorizationStateError::PrincipalMissing);
        }
        let binding = state
            .participant_bindings
            .get(&(
                record.participant_id.clone(),
                record.participant_artifact_digest.clone(),
            ))
            .ok_or(AuthorizationStateError::ParticipantMissing)?;
        if binding.needs_digest != record.accepted_needs_digest {
            return Err(AuthorizationStateError::NeedsDigestMismatch);
        }
        validate_principal_participant(PrincipalKind::User, binding.participant_kind)?;
        let key = (record.principal_id.clone(), record.participant_id.clone());
        versioned_put(
            &mut state.identity_authorities,
            key,
            record,
            expected_version,
        )
    }
}

#[async_trait]
impl DeploymentAuthorityRepository for InMemoryAuthorizationStore {
    async fn get_deployment_authority(
        &self,
        deployment_id: &str,
        participant_id: &str,
    ) -> Result<Option<DeploymentAuthorityRecord>, AuthorizationStateError> {
        Ok(self
            .state()?
            .deployment_authorities
            .get(&(deployment_id.to_owned(), participant_id.to_owned()))
            .cloned())
    }

    async fn put_deployment_authority(
        &self,
        mut record: DeploymentAuthorityRecord,
        expected_version: Option<u64>,
    ) -> Result<DeploymentAuthorityRecord, AuthorizationStateError> {
        validate_deployment_authority(&mut record)?;
        let mut state = self.state()?;
        let binding = state
            .participant_bindings
            .get(&(
                record.participant_id.clone(),
                record.participant_artifact_digest.clone(),
            ))
            .ok_or(AuthorizationStateError::ParticipantMissing)?;
        if binding.participant_kind != record.participant_kind {
            return Err(AuthorizationStateError::InvalidRecord(
                "deployment authority participant kind does not match binding".to_owned(),
            ));
        }
        if binding.needs_digest != record.accepted_needs_digest {
            return Err(AuthorizationStateError::NeedsDigestMismatch);
        }
        let key = (record.deployment_id.clone(), record.participant_id.clone());
        versioned_put(
            &mut state.deployment_authorities,
            key,
            record,
            expected_version,
        )
    }
}

#[async_trait]
impl EvidenceRepository for InMemoryAuthorizationStore {
    async fn get_deployment_evidence(
        &self,
        deployment_id: &str,
    ) -> Result<Option<DeploymentRecord>, AuthorizationStateError> {
        Ok(self.state()?.deployments.get(deployment_id).cloned())
    }

    async fn put_deployment_evidence(
        &self,
        deployment: DeploymentRecord,
    ) -> Result<(), AuthorizationStateError> {
        validate_deployment_evidence(&deployment)?;
        let mut state = self.state()?;
        if let Some(existing) = state.deployments.get(&deployment.deployment_id) {
            if existing.participant_id != deployment.participant_id
                || existing.participant_kind != deployment.participant_kind
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "deployment participant identity cannot change".to_owned(),
                ));
            }
        }
        state
            .deployments
            .insert(deployment.deployment_id.clone(), deployment);
        Ok(())
    }

    async fn get_runtime_evidence(
        &self,
        session_id: &str,
    ) -> Result<Option<RuntimeEvidence>, AuthorizationStateError> {
        let state = self.state()?;
        memory_runtime_evidence(&state, session_id)
    }

    async fn get_runtime_instance(
        &self,
        instance_id: &str,
    ) -> Result<Option<RuntimeInstanceRecord>, AuthorizationStateError> {
        Ok(self.state()?.runtime_instances.get(instance_id).cloned())
    }

    async fn put_runtime_instance(
        &self,
        instance: RuntimeInstanceRecord,
    ) -> Result<(), AuthorizationStateError> {
        validate_runtime_instance(&instance)?;
        let mut state = self.state()?;
        validate_runtime_instance_relationships(&state, &instance)?;
        if let Some(existing) = state.runtime_instances.get(&instance.instance_id) {
            if existing.deployment_id != instance.deployment_id
                || existing.principal_id != instance.principal_id
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "runtime instance identity cannot change".to_owned(),
                ));
            }
        }
        state
            .runtime_instances
            .insert(instance.instance_id.clone(), instance);
        Ok(())
    }

    async fn get_device(
        &self,
        principal_id: &str,
        deployment_id: &str,
    ) -> Result<Option<DeviceRecord>, AuthorizationStateError> {
        Ok(self
            .state()?
            .devices
            .get(&(principal_id.to_owned(), deployment_id.to_owned()))
            .cloned())
    }

    async fn put_device(&self, device: DeviceRecord) -> Result<(), AuthorizationStateError> {
        validate_device(&device)?;
        let mut state = self.state()?;
        validate_device_relationships(&state, &device)?;
        state.devices.insert(
            (device.principal_id.clone(), device.deployment_id.clone()),
            device,
        );
        Ok(())
    }

    async fn get_device_delegation(
        &self,
        principal_id: &str,
        deployment_id: &str,
    ) -> Result<Option<DeviceDelegationRecord>, AuthorizationStateError> {
        Ok(self
            .state()?
            .device_delegations
            .get(&(principal_id.to_owned(), deployment_id.to_owned()))
            .cloned())
    }

    async fn put_device_delegation(
        &self,
        delegation: DeviceDelegationRecord,
    ) -> Result<(), AuthorizationStateError> {
        validate_device_delegation(&delegation)?;
        let mut state = self.state()?;
        if !state.devices.contains_key(&(
            delegation.principal_id.clone(),
            delegation.deployment_id.clone(),
        )) {
            return Err(AuthorizationStateError::DeviceInactive);
        }
        state.device_delegations.insert(
            (
                delegation.principal_id.clone(),
                delegation.deployment_id.clone(),
            ),
            delegation,
        );
        Ok(())
    }

    async fn get_session_runtime_binding(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionRuntimeBinding>, AuthorizationStateError> {
        Ok(self
            .state()?
            .session_runtime_bindings
            .get(session_id)
            .cloned())
    }

    async fn put_session_runtime_binding(
        &self,
        binding: SessionRuntimeBinding,
    ) -> Result<(), AuthorizationStateError> {
        validate_session_runtime_binding(&binding)?;
        let mut state = self.state()?;
        validate_session_runtime_binding_relationships(&state, &binding)?;
        state
            .session_runtime_bindings
            .insert(binding.session_id.clone(), binding);
        Ok(())
    }

    async fn remove_session_runtime_binding(
        &self,
        session_id: &str,
    ) -> Result<(), AuthorizationStateError> {
        require_nonempty("sessionId", session_id)?;
        self.state()?.session_runtime_bindings.remove(session_id);
        Ok(())
    }

    async fn list_dependency_evidence(
        &self,
        target: &AuthorityTarget,
    ) -> Result<Vec<DependencyEvidence>, AuthorizationStateError> {
        Ok(self
            .state()?
            .dependency_evidence
            .get(target)
            .map(|(_, evidence)| evidence.clone())
            .unwrap_or_default())
    }

    async fn replace_dependency_evidence(
        &self,
        scope: AuthorityEvidenceScope,
        evidence: Vec<DependencyEvidence>,
    ) -> Result<(), AuthorizationStateError> {
        validate_dependency_evidence(&evidence)?;
        let mut state = self.state()?;
        validate_evidence_scope(&state, &scope)?;
        state
            .dependency_evidence
            .insert(scope.target.clone(), (scope, evidence));
        Ok(())
    }

    async fn list_resource_evidence(
        &self,
        target: &AuthorityTarget,
    ) -> Result<Vec<ResourceBindingEvidence>, AuthorizationStateError> {
        Ok(self
            .state()?
            .resource_evidence
            .get(target)
            .map(|(_, evidence)| evidence.clone())
            .unwrap_or_default())
    }

    async fn replace_resource_evidence(
        &self,
        scope: AuthorityEvidenceScope,
        evidence: Vec<ResourceBindingEvidence>,
    ) -> Result<(), AuthorizationStateError> {
        validate_resource_evidence(&evidence)?;
        let mut state = self.state()?;
        validate_evidence_scope(&state, &scope)?;
        state
            .resource_evidence
            .insert(scope.target.clone(), (scope, evidence));
        Ok(())
    }
}

#[async_trait]
impl AuthorizationMaterializationRepository for InMemoryAuthorizationStore {
    async fn get_materialized_authority(
        &self,
        kind: AuthorityKind,
        authority_id: &str,
    ) -> Result<Option<MaterializationReplacement>, AuthorizationStateError> {
        Ok(self
            .state()?
            .materializations
            .get(&(kind, authority_id.to_owned()))
            .cloned())
    }

    async fn reconcile_authority(
        &self,
        target: &AuthorityTarget,
        now: i64,
    ) -> Result<AuthorityReconciliationOutcome, AuthorizationStateError> {
        let mut state = self.state()?;
        let snapshot = memory_materialization_snapshot(&state, target)?;
        let token = snapshot.token.clone();
        let previous = snapshot.previous.clone();
        let mut replacement = materialize_authority(&snapshot, now);
        if replacement.as_ref().is_some_and(|current| {
            previous
                .as_ref()
                .is_some_and(|previous| materialization_semantics_equal(previous, current))
        }) {
            return Ok(AuthorityReconciliationOutcome {
                target: target.clone(),
                snapshot_token: token,
                materialization: previous,
                changed: false,
            });
        }
        if replacement.is_none() && previous.is_none() {
            return Ok(AuthorityReconciliationOutcome {
                target: target.clone(),
                snapshot_token: token,
                materialization: None,
                changed: false,
            });
        }
        if let Some(current) = replacement.as_mut() {
            current.authority.materialization_version = match previous.as_ref() {
                Some(previous) => next_version(
                    "materializationVersion",
                    previous.authority.materialization_version,
                )?,
                None => 1,
            };
            validate_materialization(current)?;
        }
        let transition = transition_for_change(previous.as_ref(), replacement.as_ref(), now)?;
        if let Some(current) = replacement.as_ref() {
            state
                .materializations
                .insert((target.kind, target.authority_id.clone()), current.clone());
        } else {
            state
                .materializations
                .remove(&(target.kind, target.authority_id.clone()));
        }
        if let Some(transition) = transition {
            state.transition_outbox.insert(
                transition.event_id.clone(),
                AuthorizationTransitionOutboxRecord {
                    event_id: transition.event_id.clone(),
                    created_at: transition.created_at,
                    transition,
                },
            );
        }
        Ok(AuthorityReconciliationOutcome {
            target: target.clone(),
            snapshot_token: token,
            materialization: replacement,
            changed: true,
        })
    }

    async fn list_reconciliation_targets(
        &self,
    ) -> Result<Vec<AuthorityTarget>, AuthorizationStateError> {
        let state = self.state()?;
        let mut targets = BTreeSet::new();
        targets.extend(
            state
                .identity_authorities
                .values()
                .cloned()
                .map(DesiredAuthorityRecord::Identity)
                .map(|authority| authority.target()),
        );
        targets.extend(
            state
                .deployment_authorities
                .values()
                .cloned()
                .map(DesiredAuthorityRecord::Deployment)
                .map(|authority| authority.target()),
        );
        targets.extend(
            state
                .materializations
                .keys()
                .map(|(kind, authority_id)| AuthorityTarget {
                    kind: *kind,
                    authority_id: authority_id.clone(),
                }),
        );
        Ok(targets.into_iter().collect())
    }

    async fn load_issuance_snapshot(
        &self,
        session_id: &str,
    ) -> Result<IssuanceSnapshot, AuthorizationStateError> {
        let state = self.state()?;
        memory_issuance_snapshot(&state, session_id)
    }

    async fn list_transition_outbox(
        &self,
        limit: usize,
    ) -> Result<Vec<AuthorizationTransitionOutboxRecord>, AuthorizationStateError> {
        let mut records = self
            .state()?
            .transition_outbox
            .values()
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            (left.created_at, &left.event_id).cmp(&(right.created_at, &right.event_id))
        });
        records.truncate(limit);
        Ok(records)
    }

    async fn acknowledge_transition(&self, event_id: &str) -> Result<(), AuthorizationStateError> {
        self.state()?.transition_outbox.remove(event_id);
        Ok(())
    }
}

fn memory_materialization_snapshot(
    state: &MemoryState,
    target: &AuthorityTarget,
) -> Result<AuthorityMaterializationSnapshot, AuthorizationStateError> {
    let authority = desired_authority_by_target(state, target);
    let subject = authority.as_ref().and_then(|authority| match authority {
        DesiredAuthorityRecord::Identity(record) => state
            .principals
            .get(&record.principal_id)
            .cloned()
            .map(AuthoritySubjectRecord::Identity),
        DesiredAuthorityRecord::Deployment(record) => state
            .deployments
            .get(&record.deployment_id)
            .cloned()
            .map(AuthoritySubjectRecord::Deployment),
    });
    let participant = authority.as_ref().and_then(|authority| {
        state
            .participant_bindings
            .get(&(
                authority.participant_id().to_owned(),
                authority.participant_artifact_digest().to_owned(),
            ))
            .cloned()
    });
    let dependency_entry = state.dependency_evidence.get(target).cloned();
    let resource_entry = state.resource_evidence.get(target).cloned();
    let dependencies = dependency_entry
        .as_ref()
        .map_or_else(Vec::new, |(_, evidence)| evidence.clone());
    let resources = resource_entry
        .as_ref()
        .map_or_else(Vec::new, |(_, evidence)| evidence.clone());
    let previous = state
        .materializations
        .get(&(target.kind, target.authority_id.clone()))
        .cloned();
    let token = snapshot_token(&(
        &authority,
        &subject,
        &participant,
        &dependency_entry,
        &resource_entry,
        &previous,
    ))?;
    Ok(AuthorityMaterializationSnapshot {
        target: target.clone(),
        token,
        authority,
        subject,
        participant,
        dependencies,
        dependency_scope: dependency_entry.map(|(scope, _)| scope),
        resources,
        resource_scope: resource_entry.map(|(scope, _)| scope),
        previous,
    })
}

fn memory_runtime_evidence(
    state: &MemoryState,
    session_id: &str,
) -> Result<Option<RuntimeEvidence>, AuthorizationStateError> {
    let Some(session) = state.sessions.get(session_id) else {
        return Err(AuthorizationStateError::SessionMissing);
    };
    if session.principal_kind == PrincipalKind::User {
        return Ok(Some(RuntimeEvidence::User));
    }
    let Some(binding) = state.session_runtime_bindings.get(session_id) else {
        return Ok(None);
    };
    let instance = state
        .runtime_instances
        .get(&binding.instance_id)
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(
                "session runtime binding references a missing instance".to_owned(),
            )
        })?;
    if instance.deployment_id != binding.deployment_id
        || instance.principal_id != session.principal_id
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "session runtime binding does not match instance identity".to_owned(),
        ));
    }
    let instance_active = instance.state == RuntimeInstanceState::Active;
    match session.principal_kind {
        PrincipalKind::Service => Ok(Some(RuntimeEvidence::Service(ServiceEvidence {
            deployment_id: binding.deployment_id.clone(),
            instance_id: binding.instance_id.clone(),
            instance_active,
        }))),
        PrincipalKind::Device => {
            let key = (session.principal_id.clone(), binding.deployment_id.clone());
            let device_active = state
                .devices
                .get(&key)
                .is_some_and(|device| device.state == DeviceState::Active);
            let delegation = state
                .device_delegations
                .get(&key)
                .map(|record| DelegationEvidence {
                    active: record.state == DeviceDelegationState::Active,
                    required: record.required,
                    expires_at: record.expires_at,
                });
            Ok(Some(RuntimeEvidence::Device(DeviceEvidence {
                deployment_id: binding.deployment_id.clone(),
                instance_id: binding.instance_id.clone(),
                device_active,
                instance_active,
                delegation,
            })))
        }
        PrincipalKind::User => Ok(Some(RuntimeEvidence::User)),
    }
}

fn memory_issuance_snapshot(
    state: &MemoryState,
    session_id: &str,
) -> Result<IssuanceSnapshot, AuthorizationStateError> {
    let session = state.sessions.get(session_id).cloned();
    let principal = session
        .as_ref()
        .and_then(|session| state.principals.get(&session.principal_id))
        .cloned();
    let participant = session.as_ref().and_then(|session| {
        state
            .participant_bindings
            .get(&(
                session.participant_id.clone(),
                session.participant_artifact_digest.clone(),
            ))
            .cloned()
    });
    let runtime = match principal.as_ref().map(|principal| principal.kind) {
        Some(PrincipalKind::User) => Some(RuntimeEvidence::User),
        Some(PrincipalKind::Service | PrincipalKind::Device) => {
            memory_runtime_evidence(state, session_id)?
        }
        None => None,
    };
    let authority = match (session.as_ref(), principal.as_ref(), runtime.as_ref()) {
        (Some(session), Some(principal), Some(RuntimeEvidence::User)) => state
            .identity_authorities
            .get(&(
                principal.principal_id.clone(),
                session.participant_id.clone(),
            ))
            .cloned()
            .map(DesiredAuthorityRecord::Identity),
        (Some(session), _, Some(RuntimeEvidence::Service(evidence))) => state
            .deployment_authorities
            .get(&(
                evidence.deployment_id.clone(),
                session.participant_id.clone(),
            ))
            .cloned()
            .map(DesiredAuthorityRecord::Deployment),
        (Some(session), _, Some(RuntimeEvidence::Device(evidence))) => state
            .deployment_authorities
            .get(&(
                evidence.deployment_id.clone(),
                session.participant_id.clone(),
            ))
            .cloned()
            .map(DesiredAuthorityRecord::Deployment),
        _ => None,
    };
    let deployment = match runtime.as_ref() {
        Some(RuntimeEvidence::Service(evidence)) => {
            state.deployments.get(&evidence.deployment_id).cloned()
        }
        Some(RuntimeEvidence::Device(evidence)) => {
            state.deployments.get(&evidence.deployment_id).cloned()
        }
        Some(RuntimeEvidence::User) | None => None,
    };
    let materialization = authority.as_ref().and_then(|authority| {
        let target = authority.target();
        state
            .materializations
            .get(&(target.kind, target.authority_id))
            .cloned()
    });
    Ok(IssuanceSnapshot {
        session,
        principal,
        participant,
        runtime,
        deployment,
        authority,
        materialization,
    })
}

fn validate_runtime_instance_relationships(
    state: &MemoryState,
    instance: &RuntimeInstanceRecord,
) -> Result<(), AuthorizationStateError> {
    let deployment = state
        .deployments
        .get(&instance.deployment_id)
        .ok_or(AuthorizationStateError::DeploymentInactive)?;
    let principal = state
        .principals
        .get(&instance.principal_id)
        .ok_or(AuthorizationStateError::PrincipalMissing)?;
    let kind_matches = matches!(
        (principal.kind, deployment.participant_kind),
        (
            PrincipalKind::Service,
            trellis_protocol::ParticipantKindV1::Service
        ) | (
            PrincipalKind::Device,
            trellis_protocol::ParticipantKindV1::Device
        )
    );
    if !kind_matches {
        return Err(AuthorizationStateError::InvalidRecord(
            "runtime instance principal kind does not match deployment".to_owned(),
        ));
    }
    Ok(())
}

fn validate_device_relationships(
    state: &MemoryState,
    device: &DeviceRecord,
) -> Result<(), AuthorizationStateError> {
    let deployment = state
        .deployments
        .get(&device.deployment_id)
        .ok_or(AuthorizationStateError::DeploymentInactive)?;
    let principal = state
        .principals
        .get(&device.principal_id)
        .ok_or(AuthorizationStateError::PrincipalMissing)?;
    if principal.kind != PrincipalKind::Device
        || deployment.participant_kind != trellis_protocol::ParticipantKindV1::Device
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "device evidence requires a device principal and deployment".to_owned(),
        ));
    }
    Ok(())
}

fn validate_session_runtime_binding_relationships(
    state: &MemoryState,
    binding: &SessionRuntimeBinding,
) -> Result<(), AuthorizationStateError> {
    let session = state
        .sessions
        .get(&binding.session_id)
        .ok_or(AuthorizationStateError::SessionMissing)?;
    if session.principal_kind == PrincipalKind::User {
        return Err(AuthorizationStateError::InvalidRecord(
            "user sessions cannot have runtime bindings".to_owned(),
        ));
    }
    let deployment = state
        .deployments
        .get(&binding.deployment_id)
        .ok_or(AuthorizationStateError::DeploymentInactive)?;
    if deployment.participant_id != session.participant_id
        || deployment.participant_kind != session.participant_kind
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "session runtime deployment does not match participant".to_owned(),
        ));
    }
    let instance = state
        .runtime_instances
        .get(&binding.instance_id)
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(
                "session runtime binding references a missing instance".to_owned(),
            )
        })?;
    if instance.deployment_id != binding.deployment_id
        || instance.principal_id != session.principal_id
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "session runtime binding does not match instance identity".to_owned(),
        ));
    }
    Ok(())
}

fn desired_authority_by_target(
    state: &MemoryState,
    target: &AuthorityTarget,
) -> Option<DesiredAuthorityRecord> {
    match target.kind {
        AuthorityKind::Identity => state
            .identity_authorities
            .values()
            .find(|record| record.authority_id == target.authority_id)
            .cloned()
            .map(DesiredAuthorityRecord::Identity),
        AuthorityKind::Deployment => state
            .deployment_authorities
            .values()
            .find(|record| record.authority_id == target.authority_id)
            .cloned()
            .map(DesiredAuthorityRecord::Deployment),
    }
}

fn validate_evidence_scope(
    state: &MemoryState,
    scope: &AuthorityEvidenceScope,
) -> Result<(), AuthorizationStateError> {
    let authority = desired_authority_by_target(state, &scope.target)
        .ok_or(AuthorizationStateError::AuthorityMissing)?;
    if authority.participant_id() != scope.participant_id
        || authority.participant_artifact_digest() != scope.participant_artifact_digest
    {
        return Err(AuthorizationStateError::ParticipantDigestMismatch);
    }
    if authority.accepted_needs_digest() != scope.participant_needs_digest {
        return Err(AuthorizationStateError::NeedsDigestMismatch);
    }
    Ok(())
}

fn snapshot_token<T: serde::Serialize>(
    value: &T,
) -> Result<AuthoritySnapshotToken, AuthorizationStateError> {
    let encoded = serde_json::to_vec(value).map_err(|error| {
        AuthorizationStateError::Storage(format!("cannot encode authority snapshot: {error}"))
    })?;
    Ok(AuthoritySnapshotToken(
        URL_SAFE_NO_PAD.encode(Sha256::digest(encoded)),
    ))
}

fn next_version(field: &str, current: u64) -> Result<u64, AuthorizationStateError> {
    let next = current
        .checked_add(1)
        .ok_or_else(|| AuthorizationStateError::InvalidRecord(format!("{field} overflow")))?;
    require_positive(field, next)?;
    Ok(next)
}

trait VersionedAuthority: Clone {
    fn authority_id(&self) -> &str;
    fn version(&self) -> u64;
    fn set_version(&mut self, version: u64);
    fn enforceability_equals(&self, other: &Self) -> bool;
}

impl VersionedAuthority for IdentityAuthorityRecord {
    fn authority_id(&self) -> &str {
        &self.authority_id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn set_version(&mut self, version: u64) {
        self.version = version;
    }

    fn enforceability_equals(&self, other: &Self) -> bool {
        identity_enforceability_equal(self, other)
    }
}

impl VersionedAuthority for DeploymentAuthorityRecord {
    fn authority_id(&self) -> &str {
        &self.authority_id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn set_version(&mut self, version: u64) {
        self.version = version;
    }

    fn enforceability_equals(&self, other: &Self) -> bool {
        deployment_enforceability_equal(self, other)
    }
}

fn versioned_put<K: Ord, V: VersionedAuthority>(
    records: &mut BTreeMap<K, V>,
    key: K,
    mut record: V,
    expected_version: Option<u64>,
) -> Result<V, AuthorizationStateError> {
    match (records.get(&key), expected_version) {
        (None, None) if record.version() == 1 => {}
        (Some(current), Some(expected))
            if current.version() == expected && current.authority_id() == record.authority_id() =>
        {
            if current.enforceability_equals(&record) {
                record.set_version(expected);
            } else if record.version()
                != expected.checked_add(1).ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord("authority version overflow".to_owned())
                })?
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
        }
        _ => return Err(AuthorizationStateError::StorageConflict),
    }
    records.insert(key, record.clone());
    Ok(record)
}

pub(super) fn identity_enforceability_equal(
    left: &IdentityAuthorityRecord,
    right: &IdentityAuthorityRecord,
) -> bool {
    left.principal_id == right.principal_id
        && left.participant_id == right.participant_id
        && left.participant_artifact_digest == right.participant_artifact_digest
        && left.accepted_needs_digest == right.accepted_needs_digest
        && left.desired_grant_set == right.desired_grant_set
        && left.desired_capabilities == right.desired_capabilities
        && left.state == right.state
        && left.expires_at == right.expires_at
}

pub(super) fn deployment_enforceability_equal(
    left: &DeploymentAuthorityRecord,
    right: &DeploymentAuthorityRecord,
) -> bool {
    left.deployment_id == right.deployment_id
        && left.participant_id == right.participant_id
        && left.participant_kind == right.participant_kind
        && left.participant_artifact_digest == right.participant_artifact_digest
        && left.accepted_needs_digest == right.accepted_needs_digest
        && left.desired_grant_set == right.desired_grant_set
        && left.desired_capabilities == right.desired_capabilities
        && left.state == right.state
        && left.expires_at == right.expires_at
}

pub(super) fn validate_principal(record: &PrincipalRecord) -> Result<(), AuthorizationStateError> {
    require_nonempty("principalId", &record.principal_id)?;
    require_positive("version", record.version)?;
    require_protocol_timestamp("createdAt", record.created_at)?;
    require_protocol_timestamp("updatedAt", record.updated_at)?;
    for (field, value) in [
        ("disabledAt", record.disabled_at),
        ("revokedAt", record.revoked_at),
    ] {
        if let Some(value) = value {
            require_protocol_timestamp(field, value)?;
        }
    }
    if record.updated_at < record.created_at {
        return Err(AuthorizationStateError::InvalidRecord(
            "principal updatedAt precedes createdAt".to_owned(),
        ));
    }
    if record.version != 1 {
        return Err(AuthorizationStateError::InvalidRecord(
            "new principal version must be one".to_owned(),
        ));
    }
    let timestamps_match = match record.state {
        PrincipalState::Active => record.disabled_at.is_none() && record.revoked_at.is_none(),
        PrincipalState::Disabled => record.disabled_at.is_some() && record.revoked_at.is_none(),
        PrincipalState::Revoked => record.disabled_at.is_none() && record.revoked_at.is_some(),
    };
    if !timestamps_match {
        return Err(AuthorizationStateError::InvalidRecord(
            "principal state and lifecycle timestamps do not match".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_session(record: &SessionRecord) -> Result<(), AuthorizationStateError> {
    require_nonempty("sessionId", &record.session_id)?;
    require_nonempty("principalId", &record.principal_id)?;
    require_nonempty("participantId", &record.participant_id)?;
    require_digest(
        "participantArtifactDigest",
        &record.participant_artifact_digest,
    )?;
    require_digest("participantNeedsDigest", &record.participant_needs_digest)?;
    require_digest("sessionKeyId", &record.session_key_id)?;
    require_positive("version", record.version)?;
    require_protocol_timestamp("createdAt", record.created_at)?;
    require_protocol_timestamp("lastSeenAt", record.last_seen_at)?;
    if let Some(expires_at) = record.expires_at {
        require_protocol_timestamp("expiresAt", expires_at)?;
    }
    if record.version != 1 || record.state != SessionState::Active || record.revoked_at.is_some() {
        return Err(AuthorizationStateError::InvalidRecord(
            "new session must be active at version one".to_owned(),
        ));
    }
    let derived = SessionRecord::from_new(super::NewSession {
        session_id: record.session_id.clone(),
        principal_id: record.principal_id.clone(),
        principal_kind: record.principal_kind,
        participant_id: record.participant_id.clone(),
        participant_kind: record.participant_kind,
        participant_artifact_digest: record.participant_artifact_digest.clone(),
        participant_needs_digest: record.participant_needs_digest.clone(),
        session_public_key: record.session_public_key.clone(),
        inbox_prefix: record.inbox_prefix.clone(),
        created_at: record.created_at,
        expires_at: record.expires_at,
    })?;
    if derived.session_key_id != record.session_key_id || record.last_seen_at < record.created_at {
        return Err(AuthorizationStateError::InvalidRecord(
            "session key ID or last-seen time is invalid".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_provider_identity(
    link: &ProviderIdentityLink,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("provider", &link.provider)?;
    require_nonempty("providerSubject", &link.provider_subject)?;
    require_nonempty("principalId", &link.principal_id)?;
    require_protocol_timestamp("linkedAt", link.linked_at)?;
    require_protocol_timestamp("lastSeenAt", link.last_seen_at)?;
    if link.last_seen_at < link.linked_at {
        return Err(AuthorizationStateError::InvalidRecord(
            "provider identity lastSeenAt precedes linkedAt".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_identity_authority(
    record: &mut IdentityAuthorityRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("authorityId", &record.authority_id)?;
    require_nonempty("principalId", &record.principal_id)?;
    require_nonempty("participantId", &record.participant_id)?;
    require_digest(
        "participantArtifactDigest",
        &record.participant_artifact_digest,
    )?;
    require_digest("acceptedNeedsDigest", &record.accepted_needs_digest)?;
    require_positive("version", record.version)?;
    validate_authority_timestamps(
        record.created_at,
        record.updated_at,
        record.expires_at,
        record.decision.as_ref(),
    )?;
    record.desired_capabilities = canonical_capabilities(record.desired_capabilities.clone())?;
    validate_decision(record.state, record.decision.is_some())
}

pub(super) fn validate_deployment_authority(
    record: &mut DeploymentAuthorityRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("authorityId", &record.authority_id)?;
    require_nonempty("deploymentId", &record.deployment_id)?;
    require_nonempty("participantId", &record.participant_id)?;
    require_digest(
        "participantArtifactDigest",
        &record.participant_artifact_digest,
    )?;
    require_digest("acceptedNeedsDigest", &record.accepted_needs_digest)?;
    require_positive("version", record.version)?;
    validate_authority_timestamps(
        record.created_at,
        record.updated_at,
        record.expires_at,
        record.decision.as_ref(),
    )?;
    if !matches!(
        record.participant_kind,
        trellis_protocol::ParticipantKindV1::Service | trellis_protocol::ParticipantKindV1::Device
    ) {
        return Err(AuthorizationStateError::InvalidRecord(
            "deployment authority requires a service or device participant".to_owned(),
        ));
    }
    record.desired_capabilities = canonical_capabilities(record.desired_capabilities.clone())?;
    validate_decision(record.state, record.decision.is_some())
}

fn validate_decision(
    state: AuthorityState,
    has_decision: bool,
) -> Result<(), AuthorizationStateError> {
    if (state == AuthorityState::Pending) == has_decision {
        return Err(AuthorizationStateError::InvalidRecord(
            "pending authority must not have a decision; decided authority must have one"
                .to_owned(),
        ));
    }
    Ok(())
}

fn validate_authority_timestamps(
    created_at: i64,
    updated_at: i64,
    expires_at: Option<i64>,
    decision: Option<&super::AuthorityDecision>,
) -> Result<(), AuthorizationStateError> {
    require_protocol_timestamp("createdAt", created_at)?;
    require_protocol_timestamp("updatedAt", updated_at)?;
    if updated_at < created_at {
        return Err(AuthorizationStateError::InvalidRecord(
            "authority updatedAt precedes createdAt".to_owned(),
        ));
    }
    if let Some(expires_at) = expires_at {
        require_protocol_timestamp("expiresAt", expires_at)?;
    }
    if let Some(decision) = decision {
        require_protocol_timestamp("decision.decidedAt", decision.decided_at)?;
        require_nonempty("decision.decidedBy", &decision.decided_by)?;
    }
    Ok(())
}

pub(super) fn validate_dependency_evidence(
    evidence: &[DependencyEvidence],
) -> Result<(), AuthorizationStateError> {
    let mut keys = BTreeMap::new();
    for item in evidence {
        require_nonempty("dependency.alias", &item.alias)?;
        require_nonempty("dependency.apiId", &item.api_id)?;
        require_digest("dependency.apiDigest", &item.api_digest)?;
        require_nonempty(
            "dependency.providerParticipantId",
            &item.provider_participant_id,
        )?;
        require_protocol_timestamp("dependency.observedAt", item.observed_at)?;
        if keys.insert((&item.alias, item.required), ()).is_some() {
            return Err(AuthorizationStateError::InvalidRecord(
                "duplicate dependency evidence".to_owned(),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_resource_evidence(
    evidence: &[ResourceBindingEvidence],
) -> Result<(), AuthorizationStateError> {
    let mut keys = BTreeMap::new();
    for item in evidence {
        require_nonempty("resource.resourceKind", &item.resource_kind)?;
        require_nonempty("resource.localName", &item.local_name)?;
        require_nonempty("resource.bindingId", &item.binding_id)?;
        require_nonempty("resource.ownerParticipantId", &item.owner_participant_id)?;
        require_nonempty("resource.providerIdentity", &item.provider_identity)?;
        require_protocol_timestamp("resource.materializedAt", item.materialized_at)?;
        if keys
            .insert((&item.resource_kind, &item.local_name), ())
            .is_some()
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "duplicate resource-binding evidence".to_owned(),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_runtime_instance(
    instance: &RuntimeInstanceRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("instanceId", &instance.instance_id)?;
    require_nonempty("deploymentId", &instance.deployment_id)?;
    require_nonempty("principalId", &instance.principal_id)
}

pub(super) fn validate_device(device: &DeviceRecord) -> Result<(), AuthorizationStateError> {
    require_nonempty("principalId", &device.principal_id)?;
    require_nonempty("deploymentId", &device.deployment_id)
}

pub(super) fn validate_device_delegation(
    delegation: &DeviceDelegationRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("principalId", &delegation.principal_id)?;
    require_nonempty("deploymentId", &delegation.deployment_id)?;
    if let Some(expires_at) = delegation.expires_at {
        require_protocol_timestamp("delegation.expiresAt", expires_at)?;
    }
    Ok(())
}

pub(super) fn validate_session_runtime_binding(
    binding: &SessionRuntimeBinding,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("sessionId", &binding.session_id)?;
    require_nonempty("deploymentId", &binding.deployment_id)?;
    require_nonempty("instanceId", &binding.instance_id)
}

pub(super) fn validate_deployment_evidence(
    deployment: &DeploymentRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("deploymentId", &deployment.deployment_id)?;
    require_nonempty("participantId", &deployment.participant_id)?;
    if !matches!(
        deployment.participant_kind,
        trellis_protocol::ParticipantKindV1::Service | trellis_protocol::ParticipantKindV1::Device
    ) {
        return Err(AuthorizationStateError::InvalidRecord(
            "deployment evidence requires a service or device participant".to_owned(),
        ));
    }
    if let Some(expires_at) = deployment.expires_at {
        require_protocol_timestamp("deployment.expiresAt", expires_at)?;
    }
    Ok(())
}

pub(super) fn validate_materialization(
    replacement: &MaterializationReplacement,
) -> Result<(), AuthorizationStateError> {
    let record = &replacement.authority;
    require_nonempty("materializationId", &record.materialization_id)?;
    require_nonempty("authorityId", &record.authority_id)?;
    require_positive("authorityVersion", record.authority_version)?;
    require_positive("materializationVersion", record.materialization_version)?;
    require_nonempty("subjectId", &record.subject_id)?;
    require_nonempty("participantId", &record.participant_id)?;
    require_digest(
        "participantArtifactDigest",
        &record.participant_artifact_digest,
    )?;
    require_digest("participantNeedsDigest", &record.participant_needs_digest)?;
    if let Some(reconciled_at) = record.reconciled_at {
        require_protocol_timestamp("reconciledAt", reconciled_at)?;
    }
    if let Some(expires_at) = record.expires_at {
        require_protocol_timestamp("expiresAt", expires_at)?;
    }
    validate_dependency_evidence(&replacement.dependencies)?;
    validate_resource_evidence(&replacement.resources)
}

pub(super) fn materialization_semantics_equal(
    left: &MaterializationReplacement,
    right: &MaterializationReplacement,
) -> bool {
    let left_record = &left.authority;
    let right_record = &right.authority;
    left_record.authority_version == right_record.authority_version
        && left_record.subject_id == right_record.subject_id
        && left_record.participant_id == right_record.participant_id
        && left_record.participant_kind == right_record.participant_kind
        && left_record.participant_artifact_digest == right_record.participant_artifact_digest
        && left_record.participant_needs_digest == right_record.participant_needs_digest
        && left_record.effective_grant_set == right_record.effective_grant_set
        && left_record.effective_capabilities == right_record.effective_capabilities
        && left_record.state == right_record.state
        && left_record.error == right_record.error
        && left_record.expires_at == right_record.expires_at
        && left.dependencies == right.dependencies
        && left.resources == right.resources
}
