use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use trellis_protocol::{
    parse_api, parse_participant, resolve_participant, ApiArtifact, GrantSet, ParticipantKind,
    PlatformPrivilege, ResolvedParticipant,
};

/// Largest integer exactly representable by interoperable JSON security objects.
pub const MAX_PROTOCOL_INTEGER: u64 = 9_007_199_254_740_991;

pub use trellis_protocol::GrantOwnerKind;

#[derive(Clone, Debug)]
pub(crate) struct MutationActor {
    pub context_digest: String,
    pub principal_id: String,
    pub participant_id: String,
    pub owner_kind: GrantOwnerKind,
    pub owner_id: String,
    pub grant_revision: u64,
    pub login_session_id: Option<String>,
    pub session_public_key: String,
}

/// Lifecycle of the retained current binding, including revocation tombstones.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GrantBindingState {
    /// The current grants may be issued while owner and credential remain eligible.
    Active,
    /// Authority is revoked; the row and its revision remain reserved.
    Revoked,
}

/// Verified portal/provider policy association for automatic user grants.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PortalGrantProvenance {
    /// Registered trusted portal responsible for the verified login.
    pub portal_id: String,
    /// Verified identity provider, never a participant-supplied claim.
    pub provider_id: String,
    /// Verified provider roles used by the explicit grant transaction.
    pub roles: Vec<String>,
    /// Exact server policy used for that transaction.
    pub effective_policy_digest: String,
}

/// The sole current authority for `(ownerKind, ownerId, participantId)`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GrantBinding {
    /// Existing owner record class.
    pub owner_kind: GrantOwnerKind,
    /// Existing deployment or user ID; no separately allocated authority ID.
    pub owner_id: String,
    /// Stable installed participant ID.
    pub participant_id: String,
    /// Server-owned definitions used to interpret these grants.
    pub installed_revision: u64,
    /// Expanded, canonical action and resource permissions.
    pub grants: GrantSet,
    /// Explicit administrative meta-authority, never participant capability names.
    pub platform_privileges: Vec<PlatformPrivilege>,
    /// Positive safe-integer revision covering all authorization-relevant fields.
    pub revision: u64,
    /// Current active state or retained revocation tombstone.
    pub state: GrantBindingState,
    /// Optional absolute grant expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
    /// Verified automatic-policy provenance; manual consent clears it.
    pub provenance: Option<PortalGrantProvenance>,
    /// Original binding creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last semantic replacement time in Unix milliseconds.
    pub updated_at: i64,
}

/// Exact replacement intent; SQL owns the stored revision and timestamps.
#[derive(Clone, Debug)]
pub(crate) struct GrantBindingReplacement {
    pub owner_kind: GrantOwnerKind,
    pub owner_id: String,
    pub participant_id: String,
    pub installed_revision: u64,
    pub grants: GrantSet,
    pub platform_privileges: Vec<PlatformPrivilege>,
    pub state: GrantBindingState,
    pub expires_at: Option<i64>,
    pub provenance: Option<PortalGrantProvenance>,
    pub expected_revision: u64,
    pub expected_current_installed_revision: Option<u64>,
}

impl GrantBinding {
    /// Validate and canonicalize a binding before transactional storage or signing.
    pub fn validate(&mut self) -> Result<(), AuthorizationStateError> {
        require_nonempty("ownerId", &self.owner_id)?;
        require_nonempty("participantId", &self.participant_id)?;
        require_positive("installedRevision", self.installed_revision)?;
        require_positive("revision", self.revision)?;
        require_protocol_timestamp("createdAt", self.created_at)?;
        require_protocol_timestamp("updatedAt", self.updated_at)?;
        if self.updated_at < self.created_at {
            return Err(AuthorizationStateError::InvalidRecord(
                "updatedAt precedes createdAt".to_owned(),
            ));
        }
        if let Some(expires_at) = self.expires_at {
            require_protocol_timestamp("expiresAt", expires_at)?;
        }
        self.platform_privileges.sort_unstable();
        self.platform_privileges.dedup();
        if self.state == GrantBindingState::Revoked
            && (!self.grants.permissions().is_empty() || !self.platform_privileges.is_empty())
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "revoked grant bindings must contain no authority".to_owned(),
            ));
        }
        if let Some(provenance) = &mut self.provenance {
            if self.owner_kind != GrantOwnerKind::User {
                return Err(AuthorizationStateError::InvalidRecord(
                    "portal provenance requires a user-owned binding".to_owned(),
                ));
            }
            require_nonempty("portalId", &provenance.portal_id)?;
            require_nonempty("providerId", &provenance.provider_id)?;
            require_digest("effectivePolicyDigest", &provenance.effective_policy_digest)?;
            for role in &provenance.roles {
                require_nonempty("role", role)?;
            }
            provenance.roles.sort();
            provenance.roles.dedup();
        }
        Ok(())
    }
}

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

/// Input accepted when creating a user installation login.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewSession {
    /// Stable session ID.
    pub session_id: String,
    /// Stable principal ID.
    pub principal_id: String,
    /// Stable participant ID.
    pub participant_id: String,
    /// Participant class.
    pub participant_kind: ParticipantKind,
    /// Canonical unpadded base64url Ed25519 public key.
    pub session_public_key: String,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Optional session expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
}

/// Persisted user installation login, independent of transport connections.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    /// Stable session ID.
    pub session_id: String,
    /// Stable principal ID.
    pub principal_id: String,
    /// Stable participant ID.
    pub participant_id: String,
    /// Participant class.
    pub participant_kind: ParticipantKind,
    /// Canonical unpadded base64url Ed25519 public key.
    pub session_public_key: String,
    /// SHA-256 key ID derived from the raw public key.
    pub session_key_id: String,
    /// Session lifecycle state.
    pub state: SessionState,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last successful interactive authentication time in Unix milliseconds.
    pub last_authenticated_at: i64,
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
        let session_id = value.session_id.parse::<ulid::Ulid>().map_err(|_| {
            AuthorizationStateError::InvalidRecord("sessionId must be a ULID".to_owned())
        })?;
        if session_id.to_string() != value.session_id {
            return Err(AuthorizationStateError::InvalidRecord(
                "sessionId must be canonical".to_owned(),
            ));
        }
        require_nonempty("principalId", &value.principal_id)?;
        require_nonempty("participantId", &value.participant_id)?;
        require_protocol_timestamp("createdAt", value.created_at)?;
        if let Some(expires_at) = value.expires_at {
            require_protocol_timestamp("expiresAt", expires_at)?;
        }
        validate_principal_participant(PrincipalKind::User, value.participant_kind)?;
        if value
            .expires_at
            .is_some_and(|expires| expires < value.created_at)
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "expiresAt precedes createdAt".to_owned(),
            ));
        }
        let session_key_id =
            validate_ed25519_public_key("sessionPublicKey", &value.session_public_key)?;
        Ok(Self {
            session_id: value.session_id,
            principal_id: value.principal_id,
            participant_id: value.participant_id,
            participant_kind: value.participant_kind,
            session_public_key: value.session_public_key,
            session_key_id,
            state: SessionState::Active,
            created_at: value.created_at,
            last_authenticated_at: value.created_at,
            expires_at: value.expires_at,
            revoked_at: None,
            version: 1,
        })
    }
}

pub(crate) fn validate_ed25519_public_key(
    field: &str,
    value: &str,
) -> Result<String, AuthorizationStateError> {
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| {
        AuthorizationStateError::InvalidRecord(format!("{field} is not unpadded base64url"))
    })?;
    let raw: [u8; 32] = bytes.try_into().map_err(|_| {
        AuthorizationStateError::InvalidRecord(format!("{field} must encode 32 bytes"))
    })?;
    if URL_SAFE_NO_PAD.encode(raw) != value {
        return Err(AuthorizationStateError::InvalidRecord(format!(
            "{field} is not canonical"
        )));
    }
    let verifying_key = VerifyingKey::from_bytes(&raw).map_err(|_| {
        AuthorizationStateError::InvalidRecord(format!("{field} is not a valid Ed25519 public key"))
    })?;
    if verifying_key.is_weak() {
        return Err(AuthorizationStateError::InvalidRecord(format!(
            "{field} is a weak Ed25519 public key"
        )));
    }
    Ok(URL_SAFE_NO_PAD.encode(Sha256::digest(raw)))
}

/// Exact participant artifact and API-artifact binding.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantBindingRecord {
    /// Stable participant ID.
    pub participant_id: String,
    /// Participant class.
    pub participant_kind: ParticipantKind,
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
    /// Validate canonical participant/API input for server-owned installation.
    pub fn from_artifacts(
        participant: &serde_json::Value,
        api_artifacts: &[serde_json::Value],
        now: i64,
    ) -> Result<Self, AuthorizationStateError> {
        require_protocol_timestamp("installedAt", now)?;
        let participant = parse_participant(participant)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let mut apis = BTreeMap::<String, ApiArtifact>::new();
        let mut canonical_apis = BTreeMap::new();
        for value in api_artifacts {
            let api = parse_api(value)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let canonical = api
                .normalized_value()
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            if canonical_apis
                .get(api.id())
                .is_some_and(|previous| previous != &canonical)
            {
                return Err(AuthorizationStateError::InvalidRecord(format!(
                    "conflicting API artifacts for {}",
                    api.id(),
                )));
            }
            canonical_apis.insert(api.id().to_owned(), canonical);
            apis.insert(api.id().to_owned(), api);
        }
        let resolved = resolve_participant(&participant, &apis)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        Ok(Self {
            participant_id: participant.id().to_owned(),
            participant_kind: participant.kind(),
            artifact_digest: resolved.participant_digest().to_owned(),
            needs_digest: resolved
                .needs()
                .digest()
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            participant_json: participant
                .canonical_json()
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            api_artifacts_json: trellis_protocol::canonicalize_json(
                &serde_json::to_value(canonical_apis)
                    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            )
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            resolved_at: now,
            state: ParticipantBindingState::Resolved,
            error: None,
        })
    }

    /// Parse and verify the exact participant and API artifacts retained by this binding.
    ///
    /// # Errors
    ///
    /// Returns a typed digest mismatch when the canonical artifact or needs
    /// digest differs from the stored identity, or [`AuthorizationStateError::InvalidRecord`]
    /// when the retained JSON cannot be parsed and contextually resolved.
    pub fn resolve(&self) -> Result<ResolvedParticipant, AuthorizationStateError> {
        if self.state != ParticipantBindingState::Resolved {
            return Err(AuthorizationStateError::ParticipantMissing);
        }
        let participant_value = serde_json::from_str(&self.participant_json).map_err(|error| {
            AuthorizationStateError::InvalidRecord(format!(
                "participant artifact JSON is invalid: {error}"
            ))
        })?;
        let participant = parse_participant(&participant_value).map_err(|error| {
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
                let api = parse_api(&value).map_err(|error| {
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
            .collect::<Result<BTreeMap<String, ApiArtifact>, AuthorizationStateError>>()?;
        let resolved = resolve_participant(&participant, &apis).map_err(|error| {
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

/// Authority-level deployment state used during deployment materialization.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentRecord {
    /// Stable deployment ID.
    pub deployment_id: String,
    /// Stable deployed participant ID.
    pub participant_id: String,
    /// Service or device participant class.
    pub participant_kind: ParticipantKind,
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
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last lifecycle update time in Unix milliseconds.
    pub updated_at: i64,
    /// Optimistic lifecycle version.
    pub version: u64,
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
    /// The device is awaiting administrative activation approval.
    Pending,
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
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last lifecycle update time in Unix milliseconds.
    pub updated_at: i64,
    /// Optimistic lifecycle version.
    pub version: u64,
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
    /// Exact typed physical provider identity.
    pub provider_identity: ResourceProviderIdentity,
    /// Current binding state.
    pub state: ResourceBindingState,
    /// Materialization time in Unix milliseconds.
    pub materialized_at: i64,
    /// Safe binding error when unavailable.
    pub error: Option<String>,
}

/// Exact physical transport identity for one participant resource binding.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ResourceProviderIdentity {
    /// NATS KV bucket backing a participant KV resource.
    Kv {
        /// Exact NATS KV bucket name.
        bucket: String,
    },
    /// NATS object-store bucket backing a participant store resource.
    Store {
        /// Exact NATS object-store bucket name.
        bucket: String,
    },
    /// NATS KV bucket backing participant-local state.
    State {
        /// Exact NATS KV bucket used by State.
        bucket: String,
    },
    /// Jobs namespace, work stream, and exact queue subject prefixes.
    JobQueue {
        /// Exact Jobs namespace.
        namespace: String,
        /// Exact JetStream work-stream name.
        work_stream: String,
        /// Exact queue submission subject prefix.
        publish_prefix: String,
        /// Optional exact job-update subject prefix.
        updates_prefix: Option<String>,
        /// Exact worker delivery subject.
        work_subject: String,
        /// Exact durable worker consumer name.
        consumer: String,
    },
    /// Exact JetStream stream and durable consumer identity.
    EventConsumer {
        /// Exact JetStream source stream.
        stream: String,
        /// Exact durable consumer name.
        consumer: String,
        /// Exact event subjects selected by the durable consumer.
        filter_subjects: Vec<String>,
    },
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

/// Current, eligible authority and exact installed resource interpretation for one connection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuableAuthorizationState {
    /// Stable authenticated principal.
    pub principal_id: String,
    /// Authenticated principal class.
    pub principal_kind: trellis_protocol::AuthorizationPrincipalKind,
    /// Stable logical connection ID.
    pub connection_id: String,
    /// User installation login; absent for native callers.
    pub login_session_id: Option<String>,
    /// Provisioned identity; absent for user callers.
    pub identity_key_id: Option<String>,
    /// Canonical session public key.
    pub session_public_key: String,
    /// Content-derived session key ID.
    pub session_key_id: String,
    /// Authoritative reply inbox prefix.
    pub inbox_prefix: String,
    /// Exact immutable installed definitions, used only by the server.
    pub participant: ParticipantBindingRecord,
    /// Current participant-scoped grant record.
    pub binding: GrantBinding,
    /// Deployment ID for service/device principals.
    pub deployment_id: Option<String>,
    /// Runtime instance ID when required.
    pub instance_id: Option<String>,
    /// Exact effective permissions.
    pub grant_set: GrantSet,
    /// Exact available physical resource bindings supporting the grant set.
    pub resource_bindings: Vec<ResourceBindingEvidence>,
    /// Tightest credential, grant, deployment, and delegation expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
}

impl IssuableAuthorizationState {
    /// Compare the signed projection to current eligibility; issuer and signature checks are separate.
    pub(crate) fn matches_context(
        &self,
        context: &trellis_protocol::UnsignedAuthorizationContext,
        installed_revision: u64,
    ) -> bool {
        context.principal_id == self.principal_id
            && context.principal_kind == self.principal_kind
            && context.participant_id == self.participant.participant_id
            && context.owner_kind == self.binding.owner_kind
            && context.owner_id == self.binding.owner_id
            && context.grant_revision == self.binding.revision
            && installed_revision == self.binding.installed_revision
            && context.connection_id == self.connection_id
            && context.login_session_id == self.login_session_id
            && context.identity_key_id == self.identity_key_id
            && context.deployment_id == self.deployment_id
            && context.instance_id == self.instance_id
            && context.session_key == self.session_public_key
            && context.inbox_prefix == self.inbox_prefix
            && context.grants == self.grant_set
            && context.platform_privileges == self.binding.platform_privileges
            && self
                .expires_at
                .is_none_or(|expiry| context.expires_at <= expiry.div_euclid(1_000))
    }
}

/// Authorization-state denial, conflict, and storage categories.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AuthorizationStateError {
    /// A record violates a durable domain invariant.
    #[error("invalid authorization record: {0}")]
    InvalidRecord(String),
    /// Current verified authority does not permit the requested action or scope.
    #[error("not authorized")]
    NotAuthorized,
    /// The operation requires a different authenticated principal kind.
    #[error("wrong principal kind")]
    WrongPrincipalKind,
    /// The requested public object does not exist.
    #[error("requested object is missing")]
    NotFound,
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
    /// The requested provider identity does not exist for the principal.
    #[error("identity is missing")]
    IdentityMissing,
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
    /// Current authority cannot provide the configured minimum context lifetime.
    #[error("authorization context lifetime is unavailable")]
    ContextLifetimeUnavailable,
    /// A context could not be committed after bounded coherent-snapshot retries.
    #[error("authorization context snapshot changed")]
    ContextSnapshotChanged,
    /// Trusted-portal policy inputs changed after selection.
    #[error("portal policy changed")]
    PortalPolicyChanged,
    /// An optimistic version guard failed.
    #[error("authorization storage conflict")]
    StorageConflict,
    /// The requested issuer public key has never been installed.
    #[error("issuer key is missing")]
    IssuerMissing,
    /// A replacement signer must become current before this key can be revoked.
    #[error("current signing issuer must be replaced before revocation")]
    CurrentIssuerConflict,
    /// An explicit grant or installed-participant revision no longer matches.
    #[error("revision_conflict: expected {expected}, current {current}")]
    RevisionConflict {
        /// Revision provided by the caller, with zero meaning absent.
        expected: u64,
        /// Current retained revision, with zero meaning absent.
        current: u64,
    },
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
            Self::InvalidRecord(_)
                | Self::PortalPolicyChanged
                | Self::StorageConflict
                | Self::CurrentIssuerConflict
                | Self::RevisionConflict { .. }
                | Self::Storage(_)
        )
    }
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
    participant: ParticipantKind,
) -> Result<(), AuthorizationStateError> {
    let valid = matches!(
        (principal, participant),
        (
            PrincipalKind::User,
            ParticipantKind::App | ParticipantKind::Agent
        ) | (PrincipalKind::Service, ParticipantKind::Service)
            | (PrincipalKind::Device, ParticipantKind::Device)
    );
    if valid {
        Ok(())
    } else {
        Err(AuthorizationStateError::InvalidRecord(
            "principal and participant kinds are incompatible".to_owned(),
        ))
    }
}
