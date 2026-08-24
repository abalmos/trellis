use std::collections::BTreeMap;

use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};

use super::application::repository::{
    AuthorityProposalCreation, AuthorityProposalDecision, IdempotentOutcome,
};
use super::domain::{
    canonical_capabilities, require_digest, require_nonempty, require_positive,
    require_protocol_timestamp,
};
use super::{
    AuthorityDecisionRecord, AuthorityEvidenceScope, AuthorityKind, AuthorityProposalRecord,
    AuthorityState, AuthorityTarget, AuthorizationStateError, DependencyEvidence,
    DeploymentAuthorityRecord, DeploymentRecord, DesiredAuthorityRecord, DeviceDelegationRecord,
    DeviceRecord, IdentityAuthorityRecord, MaterializedAuthorityRecord, ParticipantBindingRecord,
    PrincipalRecord, PrincipalState, ProviderIdentityLink, ResourceBindingEvidence,
    ResourceProviderIdentity, RuntimeEvidence, RuntimeInstanceRecord, SessionRecord,
    SessionRuntimeBinding, SessionState,
};

/// Atomic materialized-authority replacement and its exact supporting evidence.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MaterializationReplacement {
    /// New materialized-authority header and effective permissions.
    pub authority: MaterializedAuthorityRecord,
    /// Exact dependency evidence used by the result.
    pub dependencies: Vec<DependencyEvidence>,
    /// Exact resource-binding evidence used by the result.
    pub resources: Vec<ResourceBindingEvidence>,
}

/// Opaque digest of every authority-level input read by one coherent snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthoritySnapshotToken(pub String);

/// Opaque digest of every authorization-relevant input to context issuance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IssuanceSnapshotToken(pub String);

/// Principal or deployment record that owns one authority-level materialization.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "record")]
pub(crate) enum AuthoritySubjectRecord {
    /// Identity authority is owned by one user principal.
    Identity(PrincipalRecord),
    /// Deployment authority is owned by one service or device deployment.
    Deployment(DeploymentRecord),
}

/// Complete authority-level input read under one repository unit of work.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorityMaterializationSnapshot {
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
pub(crate) struct IssuanceSnapshot {
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

/// Compute the semantic token used to guard an authorization-context commit.
///
/// Session liveness and participant resolution timestamps are deliberately
/// excluded because they do not change authorization semantics.
pub(super) fn issuance_snapshot_token(
    snapshot: &IssuanceSnapshot,
) -> Result<IssuanceSnapshotToken, AuthorizationStateError> {
    let mut session = snapshot.session.clone();
    if let Some(session) = &mut session {
        session.last_seen_at = 0;
    }
    let mut participant = snapshot.participant.clone();
    if let Some(participant) = &mut participant {
        participant.resolved_at = 0;
    }
    let value = serde_json::json!({
        "session": session,
        "principal": snapshot.principal,
        "participant": participant,
        "runtime": snapshot.runtime,
        "deployment": snapshot.deployment,
        "authority": snapshot.authority,
        "materialization": snapshot.materialization,
    });
    let canonical = trellis_protocol::canonicalize_json(&value).map_err(|error| {
        AuthorizationStateError::Storage(format!("cannot encode issuance snapshot: {error}"))
    })?;
    Ok(IssuanceSnapshotToken(
        URL_SAFE_NO_PAD.encode(Sha256::digest(canonical.as_bytes())),
    ))
}

/// Result of one coherent authority reconciliation transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorityReconciliationOutcome {
    /// Typed reconciled target.
    pub target: AuthorityTarget,
    /// Complete input revision used by the replacement.
    pub snapshot_token: AuthoritySnapshotToken,
    /// Current projection, or `None` when an orphan was removed.
    pub materialization: Option<MaterializationReplacement>,
    /// Whether the transaction changed durable semantics.
    pub changed: bool,
}

/// Persistence contract for participant bindings and desired authority proposals.
#[async_trait]
pub(crate) trait AuthorityRepository: Send + Sync {
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

    /// List current identity authorities in stable key order.
    async fn list_identity_authorities(
        &self,
    ) -> Result<Vec<IdentityAuthorityRecord>, AuthorizationStateError>;

    /// Load current identity authority for one principal and participant.
    async fn get_identity_authority(
        &self,
        principal_id: &str,
        participant_id: &str,
    ) -> Result<Option<IdentityAuthorityRecord>, AuthorizationStateError>;

    /// List current deployment authorities in stable key order.
    async fn list_deployment_authorities(
        &self,
    ) -> Result<Vec<DeploymentAuthorityRecord>, AuthorizationStateError>;

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

    /// List authority proposals with their optional terminal decisions.
    async fn list_authority_proposals(
        &self,
    ) -> Result<
        Vec<(AuthorityProposalRecord, Option<AuthorityDecisionRecord>)>,
        AuthorizationStateError,
    >;

    /// Create one immutable pending proposal.
    async fn create_authority_proposal(
        &self,
        command: AuthorityProposalCreation,
    ) -> Result<IdempotentOutcome<AuthorityProposalRecord>, AuthorizationStateError>;

    /// Load a proposal and its optional terminal decision.
    async fn get_authority_proposal(
        &self,
        proposal_id: &str,
    ) -> Result<
        Option<(AuthorityProposalRecord, Option<AuthorityDecisionRecord>)>,
        AuthorizationStateError,
    >;

    /// Decide a proposal, desired authority, proof result, and actions atomically.
    async fn decide_authority_proposal(
        &self,
        command: AuthorityProposalDecision,
    ) -> Result<IdempotentOutcome<AuthorityProposalRecord>, AuthorizationStateError>;
}

/// Persistence contract for authority-scoped dependency and resource evidence.
#[async_trait]
pub(crate) trait AuthorityEvidenceRepository: Send + Sync {
    /// List active provider authority, instance, and artifact evidence in one coherent snapshot.
    async fn list_active_provider_evidence(
        &self,
        now: i64,
    ) -> Result<Vec<ActiveProviderEvidence>, AuthorizationStateError>;

    /// Replace dependency evidence at an exact authority and participant scope.
    async fn replace_dependency_evidence(
        &self,
        scope: AuthorityEvidenceScope,
        evidence: Vec<DependencyEvidence>,
    ) -> Result<(), AuthorizationStateError>;

    /// Replace resource evidence at an exact authority and participant scope.
    async fn replace_resource_evidence(
        &self,
        scope: AuthorityEvidenceScope,
        evidence: Vec<ResourceBindingEvidence>,
    ) -> Result<(), AuthorizationStateError>;

    /// List runtime instances in stable ID order.
    async fn list_runtime_instances(
        &self,
    ) -> Result<Vec<RuntimeInstanceRecord>, AuthorizationStateError>;

    /// List devices in stable deployment/principal order.
    async fn list_devices(&self) -> Result<Vec<DeviceRecord>, AuthorizationStateError>;

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

    /// Load device delegation evidence at its complete identity.
    async fn get_device_delegation(
        &self,
        principal_id: &str,
        deployment_id: &str,
    ) -> Result<Option<DeviceDelegationRecord>, AuthorizationStateError>;

    /// Load the runtime selection owned by one session.
    async fn get_session_runtime_binding(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionRuntimeBinding>, AuthorizationStateError>;
}

/// Exact active provider evidence used during dependency resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveProviderEvidence {
    /// Accepted deployment authority.
    pub authority: DeploymentAuthorityRecord,
    /// Stable active runtime instance selected for the deployment.
    pub instance: RuntimeInstanceRecord,
    /// Exact participant artifact bound to the authority.
    pub binding: ParticipantBindingRecord,
}

/// Persistence contract for authorization-context issuance snapshots and transitions.
#[async_trait]
pub(crate) trait ContextRepository: Send + Sync {
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

pub(crate) fn validate_principal(record: &PrincipalRecord) -> Result<(), AuthorizationStateError> {
    validate_persisted_principal(record)?;
    if record.version != 1 {
        return Err(AuthorizationStateError::InvalidRecord(
            "new principal version must be one".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_persisted_principal(
    record: &PrincipalRecord,
) -> Result<(), AuthorizationStateError> {
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
    validate_persisted_session(record)?;
    if record.version != 1 || record.state != SessionState::Active || record.revoked_at.is_some() {
        return Err(AuthorizationStateError::InvalidRecord(
            "new session must be active at version one".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_persisted_session(
    record: &SessionRecord,
) -> Result<(), AuthorizationStateError> {
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
    if derived.session_key_id != record.session_key_id
        || record.last_seen_at < record.created_at
        || record
            .revoked_at
            .is_some_and(|revoked_at| revoked_at < record.created_at)
        || !matches!(
            (record.state, record.revoked_at.is_some()),
            (SessionState::Active | SessionState::Expired, false) | (SessionState::Revoked, true)
        )
    {
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
        trellis_protocol::ParticipantKind::Service | trellis_protocol::ParticipantKind::Device
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

pub(crate) fn validate_resource_evidence(
    evidence: &[ResourceBindingEvidence],
) -> Result<(), AuthorizationStateError> {
    let mut keys = BTreeMap::new();
    for item in evidence {
        require_nonempty("resource.resourceKind", &item.resource_kind)?;
        require_nonempty("resource.localName", &item.local_name)?;
        require_nonempty("resource.bindingId", &item.binding_id)?;
        require_nonempty("resource.ownerParticipantId", &item.owner_participant_id)?;
        match (&item.resource_kind[..], &item.provider_identity) {
            ("kv", ResourceProviderIdentity::Kv { bucket })
            | ("store", ResourceProviderIdentity::Store { bucket })
            | ("state", ResourceProviderIdentity::State { bucket }) => {
                validate_physical_name("resource.providerIdentity.bucket", bucket, false)?;
            }
            (
                "jobQueue",
                ResourceProviderIdentity::JobQueue {
                    namespace,
                    work_stream,
                    publish_prefix,
                    updates_prefix,
                    work_subject,
                    consumer,
                },
            ) => {
                validate_physical_name("resource.providerIdentity.namespace", namespace, false)?;
                validate_physical_name("resource.providerIdentity.workStream", work_stream, false)?;
                validate_physical_name(
                    "resource.providerIdentity.publishPrefix",
                    publish_prefix,
                    true,
                )?;
                if let Some(prefix) = updates_prefix {
                    validate_physical_name(
                        "resource.providerIdentity.updatesPrefix",
                        prefix,
                        true,
                    )?;
                }
                validate_physical_name(
                    "resource.providerIdentity.workSubject",
                    work_subject,
                    true,
                )?;
                validate_physical_name("resource.providerIdentity.consumer", consumer, false)?;
            }
            (
                "eventConsumer",
                ResourceProviderIdentity::EventConsumer {
                    stream,
                    consumer,
                    filter_subjects,
                },
            ) => {
                validate_physical_name("resource.providerIdentity.stream", stream, false)?;
                validate_physical_name("resource.providerIdentity.consumer", consumer, false)?;
                if filter_subjects.is_empty() {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "event consumer requires filter subjects".to_owned(),
                    ));
                }
                for subject in filter_subjects {
                    validate_physical_name(
                        "resource.providerIdentity.filterSubjects",
                        subject,
                        true,
                    )?;
                }
            }
            _ => {
                return Err(AuthorizationStateError::InvalidRecord(
                    "resource kind does not match provider identity".to_owned(),
                ));
            }
        }
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

fn validate_physical_name(
    field: &str,
    value: &str,
    allow_tokens: bool,
) -> Result<(), AuthorizationStateError> {
    require_nonempty(field, value)?;
    if value.contains('*')
        || value.contains('>')
        || value.chars().any(char::is_whitespace)
        || (!allow_tokens && value.contains('.'))
        || (allow_tokens
            && (value.starts_with('.') || value.ends_with('.') || value.contains("..")))
    {
        return Err(AuthorizationStateError::InvalidRecord(format!(
            "{field} is not a safe physical NATS identity"
        )));
    }
    Ok(())
}

pub(crate) fn validate_runtime_instance(
    instance: &RuntimeInstanceRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("instanceId", &instance.instance_id)?;
    require_nonempty("deploymentId", &instance.deployment_id)?;
    require_nonempty("principalId", &instance.principal_id)?;
    require_protocol_timestamp("createdAt", instance.created_at)?;
    require_protocol_timestamp("updatedAt", instance.updated_at)?;
    require_positive("version", instance.version)
}

pub(super) fn validate_device(device: &DeviceRecord) -> Result<(), AuthorizationStateError> {
    require_nonempty("principalId", &device.principal_id)?;
    require_nonempty("deploymentId", &device.deployment_id)?;
    require_protocol_timestamp("createdAt", device.created_at)?;
    require_protocol_timestamp("updatedAt", device.updated_at)?;
    require_positive("version", device.version)
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

pub(crate) fn validate_deployment_evidence(
    deployment: &DeploymentRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("deploymentId", &deployment.deployment_id)?;
    require_nonempty("participantId", &deployment.participant_id)?;
    if !matches!(
        deployment.participant_kind,
        trellis_protocol::ParticipantKind::Service | trellis_protocol::ParticipantKind::Device
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
