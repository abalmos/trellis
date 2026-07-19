use std::collections::BTreeMap;
use std::collections::BTreeSet;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use trellis_protocol::{
    parse_api_v1, parse_participant_v1, resolve_participant_v1, ApiArtifactV1,
    AuthorizationAuthorityRefV1, AuthorizationParticipantV1, AuthorizationPrincipalV1, GrantSetV1,
    ParticipantKindV1, ResolvedParticipantV1,
};

/// Largest integer exactly representable by interoperable JSON security objects.
pub const MAX_PROTOCOL_INTEGER: u64 = 9_007_199_254_740_991;

/// Stable authorization principal class.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    /// A human account independent of any provider identity.
    User,
    /// A deployed service runtime.
    Service,
    /// A durable device identity.
    Device,
}

/// Durable principal authorization state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalState {
    /// The principal may receive current authority.
    Active,
    /// Administrative policy temporarily disables the principal.
    Disabled,
    /// The principal has been durably revoked.
    Revoked,
}

/// Rust-owned principal authorization record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalRecord {
    /// Stable authorization subject ID.
    pub principal_id: String,
    /// Principal class.
    pub kind: PrincipalKind,
    /// Current authorization state.
    pub state: PrincipalState,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last authorization-relevant update time in Unix milliseconds.
    pub updated_at: i64,
    /// Positive optimistic authorization version.
    pub version: u64,
    /// Time the principal was disabled, when applicable.
    pub disabled_at: Option<i64>,
    /// Time the principal was revoked, when applicable.
    pub revoked_at: Option<i64>,
}

/// Authorization-relevant principal state transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrincipalAuthorizationChange {
    /// New principal state.
    pub state: PrincipalState,
    /// Transition time in Unix milliseconds.
    pub changed_at: i64,
}

/// Link from an external provider identity to a stable user principal.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderIdentityLink {
    /// Identity provider key.
    pub provider: String,
    /// Provider-owned stable subject.
    pub provider_subject: String,
    /// Stable Trellis user principal ID.
    pub principal_id: String,
    /// Link creation time in Unix milliseconds.
    pub linked_at: i64,
    /// Last provider observation time in Unix milliseconds.
    pub last_seen_at: i64,
}

/// Durable authenticated-session state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// The session is eligible for authorization.
    Active,
    /// The session has passed its expiry bound.
    Expired,
    /// The session has been durably revoked.
    Revoked,
}

/// Input accepted when creating a session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewSession {
    /// Stable session ID.
    pub session_id: String,
    /// Stable principal ID.
    pub principal_id: String,
    /// Principal class, repeated to make mismatches fail closed.
    pub principal_kind: PrincipalKind,
    /// Stable participant ID.
    pub participant_id: String,
    /// Participant class.
    pub participant_kind: ParticipantKindV1,
    /// Exact participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact accepted-needs digest.
    pub participant_needs_digest: String,
    /// Canonical unpadded base64url Ed25519 public key.
    pub session_public_key: String,
    /// Authoritative reply inbox prefix.
    pub inbox_prefix: String,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Optional session expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
}

/// Persisted authenticated-session record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    /// Stable session ID.
    pub session_id: String,
    /// Stable principal ID.
    pub principal_id: String,
    /// Principal class.
    pub principal_kind: PrincipalKind,
    /// Stable participant ID.
    pub participant_id: String,
    /// Participant class.
    pub participant_kind: ParticipantKindV1,
    /// Exact participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact accepted-needs digest.
    pub participant_needs_digest: String,
    /// Canonical unpadded base64url Ed25519 public key.
    pub session_public_key: String,
    /// SHA-256 key ID derived from the raw public key.
    pub session_key_id: String,
    /// Authoritative reply inbox prefix.
    pub inbox_prefix: String,
    /// Session lifecycle state.
    pub state: SessionState,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last liveness observation in Unix milliseconds.
    pub last_seen_at: i64,
    /// Optional session expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
    /// Revocation time when revoked.
    pub revoked_at: Option<i64>,
    /// Positive optimistic authorization version.
    pub version: u64,
}

impl SessionRecord {
    /// Validate a new session and derive its canonical public-key ID.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationStateError::InvalidRecord`] when an identifier is
    /// empty, the participant/principal classes conflict, the expiry precedes
    /// creation, or the public key is not canonical unpadded base64url encoding
    /// of exactly 32 bytes.
    pub fn from_new(value: NewSession) -> Result<Self, AuthorizationStateError> {
        require_nonempty("sessionId", &value.session_id)?;
        require_nonempty("principalId", &value.principal_id)?;
        require_nonempty("participantId", &value.participant_id)?;
        require_digest(
            "participantArtifactDigest",
            &value.participant_artifact_digest,
        )?;
        require_digest("participantNeedsDigest", &value.participant_needs_digest)?;
        require_nonempty("inboxPrefix", &value.inbox_prefix)?;
        require_protocol_timestamp("createdAt", value.created_at)?;
        if let Some(expires_at) = value.expires_at {
            require_protocol_timestamp("expiresAt", expires_at)?;
        }
        validate_principal_participant(value.principal_kind, value.participant_kind)?;
        if value
            .expires_at
            .is_some_and(|expires| expires < value.created_at)
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "expiresAt precedes createdAt".to_owned(),
            ));
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(&value.session_public_key)
            .map_err(|_| {
                AuthorizationStateError::InvalidRecord(
                    "sessionPublicKey is not unpadded base64url".to_owned(),
                )
            })?;
        let raw: [u8; 32] = bytes.try_into().map_err(|_| {
            AuthorizationStateError::InvalidRecord(
                "sessionPublicKey must encode 32 bytes".to_owned(),
            )
        })?;
        if URL_SAFE_NO_PAD.encode(raw) != value.session_public_key {
            return Err(AuthorizationStateError::InvalidRecord(
                "sessionPublicKey is not canonical".to_owned(),
            ));
        }
        let verifying_key = VerifyingKey::from_bytes(&raw).map_err(|_| {
            AuthorizationStateError::InvalidRecord(
                "sessionPublicKey is not a valid Ed25519 public key".to_owned(),
            )
        })?;
        if verifying_key.is_weak() {
            return Err(AuthorizationStateError::InvalidRecord(
                "sessionPublicKey is a weak Ed25519 public key".to_owned(),
            ));
        }
        Ok(Self {
            session_id: value.session_id,
            principal_id: value.principal_id,
            principal_kind: value.principal_kind,
            participant_id: value.participant_id,
            participant_kind: value.participant_kind,
            participant_artifact_digest: value.participant_artifact_digest,
            participant_needs_digest: value.participant_needs_digest,
            session_public_key: value.session_public_key,
            session_key_id: URL_SAFE_NO_PAD.encode(Sha256::digest(raw)),
            inbox_prefix: value.inbox_prefix,
            state: SessionState::Active,
            created_at: value.created_at,
            last_seen_at: value.created_at,
            expires_at: value.expires_at,
            revoked_at: None,
            version: 1,
        })
    }
}

/// Exact participant artifact and API-artifact binding.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantBindingRecord {
    /// Stable participant ID.
    pub participant_id: String,
    /// Participant class.
    pub participant_kind: ParticipantKindV1,
    /// Exact participant artifact digest.
    pub artifact_digest: String,
    /// Exact resolved-needs digest.
    pub needs_digest: String,
    /// Canonical participant artifact JSON.
    pub participant_json: String,
    /// Canonical API artifacts keyed by canonical API ID.
    pub api_artifacts_json: String,
    /// Resolution time in Unix milliseconds.
    pub resolved_at: i64,
    /// Whether the binding is currently usable.
    pub state: ParticipantBindingState,
    /// Safe resolution error when unavailable.
    pub error: Option<String>,
}

impl ParticipantBindingRecord {
    /// Parse and verify the exact participant and API artifacts retained by this binding.
    ///
    /// # Errors
    ///
    /// Returns a typed digest mismatch when the canonical artifact or needs
    /// digest differs from the stored identity, or [`AuthorizationStateError::InvalidRecord`]
    /// when the retained JSON cannot be parsed and contextually resolved.
    pub fn resolve(&self) -> Result<ResolvedParticipantV1, AuthorizationStateError> {
        if self.state != ParticipantBindingState::Resolved {
            return Err(AuthorizationStateError::ParticipantMissing);
        }
        let participant_value = serde_json::from_str(&self.participant_json).map_err(|error| {
            AuthorizationStateError::InvalidRecord(format!(
                "participant artifact JSON is invalid: {error}"
            ))
        })?;
        let participant = parse_participant_v1(&participant_value).map_err(|error| {
            AuthorizationStateError::InvalidRecord(format!(
                "participant artifact is invalid: {error}"
            ))
        })?;
        if participant.id() != self.participant_id || participant.kind() != self.participant_kind {
            return Err(AuthorizationStateError::ParticipantDigestMismatch);
        }
        if participant
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
            != self.artifact_digest
        {
            return Err(AuthorizationStateError::ParticipantDigestMismatch);
        }
        let api_values: BTreeMap<String, serde_json::Value> =
            serde_json::from_str(&self.api_artifacts_json).map_err(|error| {
                AuthorizationStateError::InvalidRecord(format!(
                    "API artifact map JSON is invalid: {error}"
                ))
            })?;
        let apis = api_values
            .into_iter()
            .map(|(id, value)| {
                let api = parse_api_v1(&value).map_err(|error| {
                    AuthorizationStateError::InvalidRecord(format!(
                        "API artifact {id} is invalid: {error}"
                    ))
                })?;
                if api.id() != id {
                    return Err(AuthorizationStateError::InvalidRecord(format!(
                        "API artifact map key {id} does not match {}",
                        api.id()
                    )));
                }
                Ok((id, api))
            })
            .collect::<Result<BTreeMap<String, ApiArtifactV1>, AuthorizationStateError>>()?;
        let resolved = resolve_participant_v1(&participant, &apis).map_err(|error| {
            AuthorizationStateError::InvalidRecord(format!(
                "participant resolution failed: {error}"
            ))
        })?;
        let needs_digest = resolved
            .needs()
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        if needs_digest != self.needs_digest {
            return Err(AuthorizationStateError::NeedsDigestMismatch);
        }
        Ok(resolved)
    }
}

/// Exact participant binding state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantBindingState {
    /// The exact artifacts resolve and their digests match.
    Resolved,
    /// Resolution failed and the binding cannot issue authority.
    Invalid,
}

/// Desired authority lifecycle state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityState {
    /// Awaiting an authority decision.
    Pending,
    /// Accepted and enforceable when all runtime evidence is current.
    Accepted,
    /// Explicitly rejected.
    Rejected,
    /// Durably revoked.
    Revoked,
    /// Bound participant or needs evidence no longer matches.
    Stale,
}

/// Desired authority class.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityKind {
    /// Authority delegated by a user identity.
    Identity,
    /// Authority owned by a service or device deployment.
    Deployment,
}

/// Typed durable identity of one desired authority and its materialization.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityTarget {
    /// Desired authority class.
    pub kind: AuthorityKind,
    /// Stable desired authority ID within that class.
    pub authority_id: String,
}

impl AuthorityTarget {
    /// Construct and validate one typed authority target.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationStateError::InvalidRecord`] for an empty or
    /// non-canonical authority ID.
    pub fn new(
        kind: AuthorityKind,
        authority_id: impl Into<String>,
    ) -> Result<Self, AuthorizationStateError> {
        let authority_id = authority_id.into();
        require_nonempty("authorityId", &authority_id)?;
        Ok(Self { kind, authority_id })
    }
}

/// Decision metadata for desired authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityDecision {
    /// Decision time in Unix milliseconds.
    pub decided_at: i64,
    /// Stable principal or operator that made the decision.
    pub decided_by: String,
    /// Optional safe decision reason.
    pub reason: Option<String>,
}

/// User-owned desired authority bound to one exact participant needs object.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityAuthorityRecord {
    /// Stable authority record ID.
    pub authority_id: String,
    /// User principal receiving the delegated authority.
    pub principal_id: String,
    /// App or agent participant ID.
    pub participant_id: String,
    /// Exact participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact accepted-needs digest.
    pub accepted_needs_digest: String,
    /// Exact accepted machine permissions.
    pub desired_grant_set: GrantSetV1,
    /// Canonical platform capability keys.
    pub desired_capabilities: Vec<String>,
    /// Desired-authority lifecycle state.
    pub state: AuthorityState,
    /// Positive desired-authority version.
    pub version: u64,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last permission-bearing update in Unix milliseconds.
    pub updated_at: i64,
    /// Optional authority expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
    /// Current decision metadata.
    pub decision: Option<AuthorityDecision>,
}

/// Deployment-owned desired authority bound to one service or device deployment.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentAuthorityRecord {
    /// Stable authority record ID.
    pub authority_id: String,
    /// Authorized deployment ID.
    pub deployment_id: String,
    /// Service or device participant ID.
    pub participant_id: String,
    /// Participant class.
    pub participant_kind: ParticipantKindV1,
    /// Exact participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact accepted-needs digest.
    pub accepted_needs_digest: String,
    /// Exact accepted machine permissions.
    pub desired_grant_set: GrantSetV1,
    /// Canonical platform capability keys.
    pub desired_capabilities: Vec<String>,
    /// Desired-authority lifecycle state.
    pub state: AuthorityState,
    /// Positive desired-authority version.
    pub version: u64,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last permission-bearing update in Unix milliseconds.
    pub updated_at: i64,
    /// Optional authority expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
    /// Current decision metadata.
    pub decision: Option<AuthorityDecision>,
}

/// Typed desired authority without conflating identity and deployment records.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "record")]
pub enum DesiredAuthorityRecord {
    /// User-owned delegated authority.
    Identity(IdentityAuthorityRecord),
    /// Deployment-owned service or device authority.
    Deployment(DeploymentAuthorityRecord),
}

impl DesiredAuthorityRecord {
    /// Return this record's typed durable target.
    #[must_use]
    pub fn target(&self) -> AuthorityTarget {
        AuthorityTarget {
            kind: match self {
                Self::Identity(_) => AuthorityKind::Identity,
                Self::Deployment(_) => AuthorityKind::Deployment,
            },
            authority_id: self.authority_id().to_owned(),
        }
    }

    /// Return the authority-level principal or deployment subject ID.
    #[must_use]
    pub fn subject_id(&self) -> &str {
        match self {
            Self::Identity(value) => &value.principal_id,
            Self::Deployment(value) => &value.deployment_id,
        }
    }

    /// Return the stable participant ID.
    #[must_use]
    pub fn participant_id(&self) -> &str {
        match self {
            Self::Identity(value) => &value.participant_id,
            Self::Deployment(value) => &value.participant_id,
        }
    }

    /// Return the stable desired-authority ID.
    #[must_use]
    pub fn authority_id(&self) -> &str {
        match self {
            Self::Identity(value) => &value.authority_id,
            Self::Deployment(value) => &value.authority_id,
        }
    }

    /// Return the positive desired-authority version.
    #[must_use]
    pub fn version(&self) -> u64 {
        match self {
            Self::Identity(value) => value.version,
            Self::Deployment(value) => value.version,
        }
    }

    /// Return the desired-authority state.
    #[must_use]
    pub fn state(&self) -> AuthorityState {
        match self {
            Self::Identity(value) => value.state,
            Self::Deployment(value) => value.state,
        }
    }

    /// Return exact accepted grants.
    #[must_use]
    pub fn grant_set(&self) -> &GrantSetV1 {
        match self {
            Self::Identity(value) => &value.desired_grant_set,
            Self::Deployment(value) => &value.desired_grant_set,
        }
    }

    /// Return accepted canonical platform capabilities.
    #[must_use]
    pub fn capabilities(&self) -> &[String] {
        match self {
            Self::Identity(value) => &value.desired_capabilities,
            Self::Deployment(value) => &value.desired_capabilities,
        }
    }

    /// Return the exact participant artifact digest.
    #[must_use]
    pub fn participant_artifact_digest(&self) -> &str {
        match self {
            Self::Identity(value) => &value.participant_artifact_digest,
            Self::Deployment(value) => &value.participant_artifact_digest,
        }
    }

    /// Return the exact accepted-needs digest.
    #[must_use]
    pub fn accepted_needs_digest(&self) -> &str {
        match self {
            Self::Identity(value) => &value.accepted_needs_digest,
            Self::Deployment(value) => &value.accepted_needs_digest,
        }
    }

    /// Return the optional authority expiry.
    #[must_use]
    pub fn expires_at(&self) -> Option<i64> {
        match self {
            Self::Identity(value) => value.expires_at,
            Self::Deployment(value) => value.expires_at,
        }
    }
}

/// Exact authority and participant identity that scopes dependency and resource evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityEvidenceScope {
    /// Typed desired authority target.
    pub target: AuthorityTarget,
    /// Stable consuming participant ID.
    pub participant_id: String,
    /// Exact consuming participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact accepted participant-needs digest.
    pub participant_needs_digest: String,
}

/// Authority-level deployment state used during deployment materialization.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentRecord {
    /// Stable deployment ID.
    pub deployment_id: String,
    /// Stable deployed participant ID.
    pub participant_id: String,
    /// Service or device participant class.
    pub participant_kind: ParticipantKindV1,
    /// Whether the deployment can currently authorize sessions.
    pub active: bool,
    /// Optional deployment-level expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
}

/// Durable lifecycle state for one runtime instance.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeInstanceState {
    /// The instance can currently authorize sessions.
    Active,
    /// The instance was administratively disabled.
    Disabled,
    /// The instance was permanently revoked.
    Revoked,
    /// The instance evidence is no longer current.
    Stale,
}

/// Deployment- and principal-owned runtime instance evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInstanceRecord {
    /// Stable runtime instance ID.
    pub instance_id: String,
    /// Deployment that owns the instance.
    pub deployment_id: String,
    /// Service or device principal that owns the instance.
    pub principal_id: String,
    /// Current instance lifecycle state.
    pub state: RuntimeInstanceState,
}

/// Session selection of deployment-owned runtime evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRuntimeBinding {
    /// Session selecting the runtime evidence.
    pub session_id: String,
    /// Selected deployment ID.
    pub deployment_id: String,
    /// Selected runtime instance ID, required by current service and device policy.
    pub instance_id: String,
}

/// Durable lifecycle state for one device in one deployment.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceState {
    /// The device can currently authorize sessions.
    Active,
    /// The device was administratively disabled.
    Disabled,
    /// The device was permanently revoked.
    Revoked,
}

/// Deployment-scoped durable device lifecycle evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRecord {
    /// Stable device principal ID.
    pub principal_id: String,
    /// Deployment in which the device lifecycle applies.
    pub deployment_id: String,
    /// Current device lifecycle state.
    pub state: DeviceState,
}

/// Durable device-delegation lifecycle state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceDelegationState {
    /// Required delegation is current.
    Active,
    /// Required delegation has not been supplied.
    Missing,
    /// Delegation was explicitly revoked.
    Revoked,
}

/// Device- and deployment-scoped activation or delegation evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDelegationRecord {
    /// Stable device principal ID.
    pub principal_id: String,
    /// Deployment in which the delegation applies.
    pub deployment_id: String,
    /// Whether this device lifecycle requires delegation.
    pub required: bool,
    /// Current delegation lifecycle state.
    pub state: DeviceDelegationState,
    /// Optional delegation expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
}

/// Current dependency availability state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyState {
    /// A current provider satisfies the exact API evidence.
    Available,
    /// No current provider is available.
    Unavailable,
    /// The last provider evidence is stale.
    Stale,
}

/// Structured dependency evidence used during materialization.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyEvidence {
    /// Participant-local dependency alias.
    pub alias: String,
    /// Whether missing evidence invalidates all materialization.
    pub required: bool,
    /// Canonical API ID.
    pub api_id: String,
    /// Exact API artifact digest.
    pub api_digest: String,
    /// Provider participant ID.
    pub provider_participant_id: String,
    /// Provider deployment ID when deployment-backed.
    pub provider_deployment_id: Option<String>,
    /// Current provider instance ID when instance-backed.
    pub provider_instance_id: Option<String>,
    /// Current availability state.
    pub state: DependencyState,
    /// Observation time in Unix milliseconds.
    pub observed_at: i64,
}

/// Current resource-binding state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceBindingState {
    /// A current binding is usable.
    Available,
    /// No usable binding exists.
    Unavailable,
    /// The binding exists but is stale.
    Stale,
}

/// Structured materialized resource evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceBindingEvidence {
    /// Canonical participant resource family.
    pub resource_kind: String,
    /// Participant-local resource name.
    pub local_name: String,
    /// Stable binding ID.
    pub binding_id: String,
    /// Participant that owns the private resource.
    pub owner_participant_id: String,
    /// Provider or storage identity, not an inferred bucket name.
    pub provider_identity: String,
    /// Current binding state.
    pub state: ResourceBindingState,
    /// Materialization time in Unix milliseconds.
    pub materialized_at: i64,
    /// Safe binding error when unavailable.
    pub error: Option<String>,
}

/// Service deployment and instance authorization evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEvidence {
    /// Authorized deployment ID.
    pub deployment_id: String,
    /// Current runtime instance ID.
    pub instance_id: String,
    /// Whether the instance is active.
    pub instance_active: bool,
}

/// Device deployment, instance, and lifecycle authorization evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceEvidence {
    /// Authorized deployment ID.
    pub deployment_id: String,
    /// Current device instance ID.
    pub instance_id: String,
    /// Whether the durable device is active.
    pub device_active: bool,
    /// Whether an applicable runtime instance is active.
    pub instance_active: bool,
    /// Activation and delegation evidence when required.
    pub delegation: Option<DelegationEvidence>,
}

/// Device user-activation or delegation evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegationEvidence {
    /// Whether the activation/delegation remains active.
    pub active: bool,
    /// Whether this device lifecycle requires user delegation.
    pub required: bool,
    /// Optional delegation expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
}

/// Principal-specific runtime eligibility evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeEvidence {
    /// No deployment evidence is needed for a user app or agent.
    User,
    /// Service deployment and instance evidence.
    Service(ServiceEvidence),
    /// Device deployment, instance, and delegation evidence.
    Device(DeviceEvidence),
}

/// Effective materialization lifecycle state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterializationState {
    /// Effective authority is current and issuable.
    Available,
    /// Expected evidence is absent or inactive.
    Unavailable,
    /// Materialization failed because stored input is invalid.
    Error,
}

/// Distinct effective-authority projection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializedAuthorityRecord {
    /// Stable materialization record ID.
    pub materialization_id: String,
    /// Desired authority class.
    pub authority_kind: AuthorityKind,
    /// Desired authority record ID.
    pub authority_id: String,
    /// Desired authority version used by this projection.
    pub authority_version: u64,
    /// Positive effective-state generation.
    pub materialization_version: u64,
    /// Authority-level subject ID: principal for identity authority, deployment otherwise.
    pub subject_id: String,
    /// Stable participant ID.
    pub participant_id: String,
    /// Participant class.
    pub participant_kind: ParticipantKindV1,
    /// Exact participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact accepted-needs digest.
    pub participant_needs_digest: String,
    /// Current exact effective permissions.
    pub effective_grant_set: GrantSetV1,
    /// Current canonical platform capabilities.
    pub effective_capabilities: Vec<String>,
    /// Effective-state lifecycle.
    pub state: MaterializationState,
    /// Reconciliation time in Unix milliseconds when a write occurred.
    pub reconciled_at: Option<i64>,
    /// Stable safe failure category.
    pub error: Option<String>,
    /// Tightest authority or deployment-level expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
}

/// Meaningful authorization transition published through the Event Log boundary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationTransition {
    /// Deterministic logical event ID.
    pub event_id: String,
    /// Transition class.
    pub kind: AuthorizationTransitionKind,
    /// Desired authority class.
    pub authority_kind: AuthorityKind,
    /// Stable desired authority ID.
    pub authority_id: String,
    /// Current desired-authority version.
    pub authority_version: u64,
    /// Current materialization generation.
    pub materialization_version: u64,
    /// Current effective-state lifecycle.
    pub state: MaterializationState,
    /// Safe stable denial category when unavailable.
    pub error: Option<String>,
    /// Transition creation time in Unix milliseconds.
    pub created_at: i64,
}

/// Durable transition waiting for a future Event Log publisher.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationTransitionOutboxRecord {
    /// Deterministic logical event ID.
    pub event_id: String,
    /// Serialized transition payload.
    pub transition: AuthorizationTransition,
    /// Durable enqueue time in Unix milliseconds.
    pub created_at: i64,
}

/// Materialized-authority transition class.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationTransitionKind {
    /// Effective permissions or supporting evidence changed while available.
    MaterializedChanged,
    /// Effective authority became unavailable and stale grants were cleared.
    MaterializedUnavailable,
    /// Previously unavailable authority became available again.
    MaterializedRestored,
}

/// Complete trusted unsigned state required by later context issuance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuableAuthorizationState {
    /// Stable principal projected into the protocol context shape.
    pub principal: AuthorizationPrincipalV1,
    /// Stable session ID.
    pub session_id: String,
    /// Canonical session public key.
    pub session_public_key: String,
    /// Content-derived session key ID.
    pub session_key_id: String,
    /// Authoritative reply inbox prefix.
    pub inbox_prefix: String,
    /// Exact participant and needs evidence.
    pub participant: AuthorizationParticipantV1,
    /// Desired authority record and version.
    pub authority_ref: AuthorizationAuthorityRefV1,
    /// Deployment ID for service/device principals.
    pub deployment_id: Option<String>,
    /// Runtime instance ID when required.
    pub instance_id: Option<String>,
    /// Exact effective permissions.
    pub grant_set: GrantSetV1,
    /// Canonical platform capabilities.
    pub capabilities: Vec<String>,
    /// Session expiry bound in Unix milliseconds.
    pub session_expires_at: Option<i64>,
    /// Effective materialized authority and deployment expiry bound in Unix milliseconds.
    pub effective_authority_expires_at: Option<i64>,
    /// Device delegation expiry bound in Unix milliseconds.
    pub delegation_expires_at: Option<i64>,
    /// Effective materialization generation.
    pub materialization_version: u64,
}

/// Stable expected denial and storage-conflict categories.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AuthorizationStateError {
    /// A record violates a durable domain invariant.
    #[error("invalid authorization record: {0}")]
    InvalidRecord(String),
    /// The requested session does not exist.
    #[error("session is missing")]
    SessionMissing,
    /// The session has expired.
    #[error("session has expired")]
    SessionExpired,
    /// The session was revoked.
    #[error("session was revoked")]
    SessionRevoked,
    /// The principal does not exist.
    #[error("principal is missing")]
    PrincipalMissing,
    /// The principal is disabled or revoked.
    #[error("principal is inactive")]
    PrincipalInactive,
    /// The exact participant artifact is missing.
    #[error("participant binding is missing")]
    ParticipantMissing,
    /// Participant identity or artifact digest does not match.
    #[error("participant artifact digest does not match")]
    ParticipantDigestMismatch,
    /// Accepted-needs digest does not match.
    #[error("participant needs digest does not match")]
    NeedsDigestMismatch,
    /// No desired authority record applies.
    #[error("desired authority is missing")]
    AuthorityMissing,
    /// Desired authority is pending.
    #[error("desired authority is pending")]
    AuthorityPending,
    /// Desired authority was rejected.
    #[error("desired authority was rejected")]
    AuthorityRejected,
    /// Desired authority was revoked.
    #[error("desired authority was revoked")]
    AuthorityRevoked,
    /// Desired authority is stale.
    #[error("desired authority is stale")]
    AuthorityStale,
    /// Desired authority has expired.
    #[error("desired authority has expired")]
    AuthorityExpired,
    /// The deployment is inactive.
    #[error("deployment is inactive")]
    DeploymentInactive,
    /// The runtime instance is inactive.
    #[error("runtime instance is inactive")]
    InstanceInactive,
    /// The durable device is inactive.
    #[error("device is inactive")]
    DeviceInactive,
    /// Required activation/delegation evidence is missing.
    #[error("device activation is missing")]
    ActivationMissing,
    /// Device delegation has expired.
    #[error("device delegation has expired")]
    DelegationExpired,
    /// A required dependency is unavailable.
    #[error("required dependency {0} is unavailable")]
    RequiredDependencyUnavailable(String),
    /// A required resource is unavailable.
    #[error("required resource {0} is unavailable")]
    RequiredResourceUnavailable(String),
    /// Materialized authority is absent, unavailable, or no longer current.
    #[error("materialized authority is stale")]
    MaterializationStale,
    /// An optimistic version guard failed.
    #[error("authorization storage conflict")]
    StorageConflict,
    /// Persistent storage failed unexpectedly.
    #[error("authorization storage failed: {0}")]
    Storage(String),
}

impl AuthorizationStateError {
    /// Return whether this error is an expected fail-closed issuance denial.
    #[must_use]
    pub fn is_expected_denial(&self) -> bool {
        !matches!(
            self,
            Self::InvalidRecord(_) | Self::StorageConflict | Self::Storage(_)
        )
    }
}

pub(crate) fn canonical_capabilities(
    values: impl IntoIterator<Item = String>,
) -> Result<Vec<String>, AuthorizationStateError> {
    let mut output = BTreeSet::new();
    for value in values {
        require_nonempty("capability", &value)?;
        output.insert(value);
    }
    Ok(output.into_iter().collect())
}

pub(crate) fn require_nonempty(field: &str, value: &str) -> Result<(), AuthorizationStateError> {
    if value.is_empty() || value.trim() != value {
        return Err(AuthorizationStateError::InvalidRecord(format!(
            "{field} must be nonempty and trimmed"
        )));
    }
    Ok(())
}

pub(crate) fn require_positive(field: &str, value: u64) -> Result<(), AuthorizationStateError> {
    if value == 0 || value > MAX_PROTOCOL_INTEGER {
        return Err(AuthorizationStateError::InvalidRecord(format!(
            "{field} must be between 1 and {MAX_PROTOCOL_INTEGER}"
        )));
    }
    Ok(())
}

pub(crate) fn require_protocol_timestamp(
    field: &str,
    value: i64,
) -> Result<(), AuthorizationStateError> {
    if value < 0 || value as u64 > MAX_PROTOCOL_INTEGER {
        return Err(AuthorizationStateError::InvalidRecord(format!(
            "{field} must be between 0 and {MAX_PROTOCOL_INTEGER}"
        )));
    }
    Ok(())
}

pub(crate) fn require_digest(field: &str, value: &str) -> Result<(), AuthorizationStateError> {
    require_nonempty(field, value)?;
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| {
        AuthorizationStateError::InvalidRecord(format!(
            "{field} must be an unpadded base64url SHA-256 digest"
        ))
    })?;
    if bytes.len() != 32 || URL_SAFE_NO_PAD.encode(bytes) != value {
        return Err(AuthorizationStateError::InvalidRecord(format!(
            "{field} must be a canonical SHA-256 digest"
        )));
    }
    Ok(())
}

pub(crate) fn validate_principal_participant(
    principal: PrincipalKind,
    participant: ParticipantKindV1,
) -> Result<(), AuthorizationStateError> {
    let valid = matches!(
        (principal, participant),
        (
            PrincipalKind::User,
            ParticipantKindV1::App | ParticipantKindV1::Agent
        ) | (PrincipalKind::Service, ParticipantKindV1::Service)
            | (PrincipalKind::Device, ParticipantKindV1::Device)
    );
    if valid {
        Ok(())
    } else {
        Err(AuthorizationStateError::InvalidRecord(
            "principal and participant kinds are incompatible".to_owned(),
        ))
    }
}
