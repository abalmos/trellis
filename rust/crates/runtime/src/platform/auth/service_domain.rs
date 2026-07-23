use serde::{Deserialize, Serialize};
use serde_json::Value;
use trellis_protocol::GrantSetV1;

use super::domain::PrincipalKind;
use super::{AuthorityKind, AuthorizationStateError};

/// Derive the stable deployment-authority lineage for one deployment and participant.
pub(crate) fn deployment_authority_id(
    deployment_id: &str,
    participant_id: &str,
) -> Result<String, AuthorizationStateError> {
    if deployment_id.is_empty() || participant_id.is_empty() {
        return Err(AuthorizationStateError::InvalidRecord(
            "deployment and participant IDs must be non-empty".to_owned(),
        ));
    }
    Ok(format!(
        "dau_v1_{}:{deployment_id}{participant_id}",
        deployment_id.len()
    ))
}

/// Non-authority profile data for one user principal.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileRecord {
    /// Stable user principal ID.
    pub principal_id: String,
    /// Required-nullable user-selected display name.
    pub display_name: Option<String>,
    /// Required-nullable observed email address.
    pub email: Option<String>,
    /// Required-nullable profile image URL.
    pub image_url: Option<String>,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last update time in Unix milliseconds.
    pub updated_at: i64,
    /// Optimistic record version.
    pub version: u64,
}

/// Argon2id credential state for one local user account.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalCredentialRecord {
    /// Stable user principal ID.
    pub principal_id: String,
    /// Canonical case-folded login name.
    pub normalized_username: String,
    /// Encoded Argon2id password hash.
    pub password_hash: String,
    /// Version of the bounded password-hash profile.
    pub hash_profile: u32,
    /// Consecutive failed authentication attempts.
    pub failed_attempts: u32,
    /// Required-nullable lock expiry in Unix milliseconds.
    pub locked_until: Option<i64>,
    /// Last password change in Unix milliseconds.
    pub password_changed_at: i64,
    /// Last record update in Unix milliseconds.
    pub updated_at: i64,
    /// Optimistic record version.
    pub version: u64,
}

/// Login portal presentation and provider policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginPortalRecord {
    /// Stable portal ID.
    pub portal_id: String,
    /// Operator-facing portal name.
    pub display_name: String,
    /// Required-nullable external portal entry URL; the built-in portal uses `None`.
    pub entry_url: Option<String>,
    /// Whether this is the non-removable built-in portal.
    pub builtin: bool,
    /// Whether new flows may select this portal.
    pub disabled: bool,
    /// Whether the portal was administratively removed.
    pub removed: bool,
    /// Whether local registration is permitted.
    pub local_registration_enabled: bool,
    /// Ordered configured provider IDs.
    pub provider_ids: Vec<String>,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last update time in Unix milliseconds.
    pub updated_at: i64,
    /// Optimistic record version.
    pub version: u64,
}

/// Login behavior settings attached to one portal.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginSettingsRecord {
    /// Owning portal ID.
    pub portal_id: String,
    /// Required-nullable provider selected without a chooser.
    pub default_provider_id: Option<String>,
    /// Whether existing local credentials may authenticate.
    pub local_login_enabled: bool,
    /// Whether unknown federated identities may register accounts.
    pub federated_registration_enabled: bool,
    /// Whether users may select among configured providers.
    pub provider_selection_enabled: bool,
    /// Last update time in Unix milliseconds.
    pub updated_at: i64,
    /// Optimistic record version.
    pub version: u64,
}

/// Administrative lifecycle state for a deployment profile.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeploymentProfileState {
    /// The deployment may be provisioned and connected.
    Active,
    /// New and existing connections are disabled.
    Disabled,
    /// The deployment has been administratively removed.
    Removed,
}

/// Product-facing deployment metadata independent of runtime evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentProfileRecord {
    /// Stable deployment and principal ID.
    pub deployment_id: String,
    /// Service or device deployment class.
    pub kind: PrincipalKind,
    /// Operator-facing name.
    pub display_name: String,
    /// Participant selected before or during provisioning.
    pub participant_id: Option<String>,
    /// Login portal used for device activation.
    pub portal_id: Option<String>,
    /// Whether device sessions require user delegation.
    pub requires_device_delegation: bool,
    /// Optional deployment expiry in Unix milliseconds.
    pub expires_at: Option<i64>,
    /// Administrative lifecycle state.
    pub state: DeploymentProfileState,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last update time in Unix milliseconds.
    pub updated_at: i64,
    /// Optimistic record version.
    pub version: u64,
}

/// Deterministic route from auth-flow evidence to a login portal.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalRouteRecord {
    /// Stable route ID.
    pub route_id: String,
    /// Selected portal ID.
    pub portal_id: String,
    /// Required-nullable participant selector.
    pub participant_id: Option<String>,
    /// Required-nullable exact origin selector.
    pub origin: Option<String>,
    /// Required-nullable deployment selector.
    pub deployment_id: Option<String>,
    /// Higher values take precedence.
    pub priority: i64,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Last update time in Unix milliseconds.
    pub updated_at: i64,
    /// Optimistic record version.
    pub version: u64,
}

/// Purpose of a durable single-use account flow.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountFlowKind {
    /// Bootstrap the first active administrator.
    FirstAdmin,
    /// Link another provider identity to an existing account.
    IdentityLink,
    /// Replace a local password after proof of account control.
    PasswordReset,
}

/// Lifecycle state of a durable account flow.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountFlowState {
    /// The flow can be consumed once.
    Pending,
    /// The flow completed successfully.
    Consumed,
    /// The flow passed its expiry without completion.
    Expired,
    /// The flow was administratively revoked.
    Revoked,
}

/// Hashed, expiring, single-use account workflow.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountFlowRecord {
    /// Stable flow ID.
    pub flow_id: String,
    /// Flow purpose.
    pub kind: AccountFlowKind,
    /// SHA-256 digest of the bearer secret.
    pub token_hash: String,
    /// Required-nullable target principal.
    pub target_principal_id: Option<String>,
    /// Required-nullable target provider.
    pub target_provider_id: Option<String>,
    /// Required-nullable validated return location.
    pub return_location: Option<String>,
    /// Purpose-specific immutable payload.
    pub payload: Value,
    /// Current lifecycle state.
    pub state: AccountFlowState,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Expiry time in Unix milliseconds.
    pub expires_at: i64,
    /// Required-nullable successful consumption time.
    pub consumed_at: Option<i64>,
    /// Optimistic record version.
    pub version: u64,
}

/// Semantic class of an authority proposal.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityProposalKind {
    /// First authority for a participant target.
    Initial,
    /// Additive or restrictive compatible update.
    Update,
    /// Explicitly accepted breaking migration.
    Migration,
}

/// Historical state of an immutable authority proposal.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityProposalState {
    /// Awaiting a decision.
    Pending,
    /// Accepted into current desired authority.
    Accepted,
    /// Explicitly rejected.
    Rejected,
    /// Replaced by a newer proposal for the same target.
    Superseded,
    /// Expired without a decision.
    Expired,
}

/// Immutable authority proposal plus mutable historical state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityProposalRecord {
    /// Stable proposal ID.
    pub proposal_id: String,
    /// Typed authority namespace.
    pub authority_kind: AuthorityKind,
    /// Stable authority ID.
    pub authority_id: String,
    /// Deployment owning this lineage; absent for identity authority.
    pub deployment_id: Option<String>,
    /// Proposal semantic class.
    pub proposal_kind: AuthorityProposalKind,
    /// Exact participant ID.
    pub participant_id: String,
    /// Exact participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact participant needs digest.
    pub participant_needs_digest: String,
    /// Proposed exact grants.
    pub proposed_grant_set: GrantSetV1,
    /// Proposed user-facing capability labels.
    pub proposed_capabilities: Vec<String>,
    /// Digest of the immutable proposal payload.
    pub proposal_digest: String,
    /// Immutable plan and consent payload.
    pub payload: Value,
    /// Historical proposal state.
    pub state: AuthorityProposalState,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Required-nullable proposal expiry.
    pub expires_at: Option<i64>,
    /// Required-nullable supersession time.
    pub superseded_at: Option<i64>,
    /// Optimistic state version.
    pub version: u64,
}

/// Terminal authority decision outcome.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityDecisionOutcome {
    /// Accept the proposal into desired authority.
    Accepted,
    /// Reject the proposal without changing desired authority.
    Rejected,
}

/// Immutable decision attached to one authority proposal.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityDecisionRecord {
    /// Decided proposal ID.
    pub proposal_id: String,
    /// Terminal outcome.
    pub outcome: AuthorityDecisionOutcome,
    /// Stable deciding principal or system identity.
    pub decided_by: String,
    /// Required-nullable operator reason.
    pub reason: Option<String>,
    /// Decision time in Unix milliseconds.
    pub decided_at: i64,
    /// Digest of the immutable decision payload.
    pub decision_digest: String,
}

/// Kind of provisioned workload identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisionedIdentityKind {
    /// Service instance identity.
    Service,
    /// Device instance identity.
    Device,
}

/// Lifecycle state of a provisioned workload identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisionedIdentityState {
    /// Identity may authenticate.
    Active,
    /// Identity is permanently revoked.
    Revoked,
}

/// Public metadata for an immutable provisioned identity key.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionedIdentityRecord {
    /// Digest-derived identity key ID.
    pub identity_key_id: String,
    /// Canonical public identity key.
    pub identity_public_key: String,
    /// Stable workload principal ID.
    pub principal_id: String,
    /// Immutable deployment assignment.
    pub deployment_id: String,
    /// Immutable instance assignment.
    pub instance_id: String,
    /// Workload identity kind.
    pub kind: ProvisionedIdentityKind,
    /// Current lifecycle state.
    pub state: ProvisionedIdentityState,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Required-nullable permanent revocation time.
    pub revoked_at: Option<i64>,
}

/// Lifecycle state of a one-time device provisioning secret.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisioningSecretState {
    /// Secret can be consumed once.
    Pending,
    /// Secret was consumed successfully.
    Consumed,
    /// Secret passed its expiry.
    Expired,
    /// Secret was administratively revoked.
    Revoked,
}

/// Hashed one-time device provisioning secret.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceProvisioningSecretRecord {
    /// Stable secret record ID.
    pub secret_id: String,
    /// Target runtime instance ID.
    pub instance_id: String,
    /// SHA-256 digest of the raw secret.
    pub secret_hash: String,
    /// Current lifecycle state.
    pub state: ProvisioningSecretState,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Expiry time in Unix milliseconds.
    pub expires_at: i64,
    /// Required-nullable successful consumption time.
    pub consumed_at: Option<i64>,
    /// Optimistic record version.
    pub version: u64,
}

/// Lifecycle state of an administrative device activation review.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceActivationReviewState {
    /// Awaiting an administrator decision.
    Pending,
    /// Approved by an administrator.
    Approved,
    /// Rejected by an administrator.
    Rejected,
    /// Cancelled by the requester.
    Cancelled,
    /// Expired without a decision.
    Expired,
}

/// Administrative review distinct from user delegation state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceActivationReviewRecord {
    /// Stable review ID.
    pub review_id: String,
    /// Device principal under review.
    pub principal_id: String,
    /// Deployment under review.
    pub deployment_id: String,
    /// Runtime instance under review.
    pub instance_id: String,
    /// Digest of the original activation request.
    pub request_digest: String,
    /// Immutable request payload.
    pub payload: Value,
    /// Current review state.
    pub state: DeviceActivationReviewState,
    /// Request time in Unix milliseconds.
    pub requested_at: i64,
    /// Required-nullable decision time.
    pub decided_at: Option<i64>,
    /// Required-nullable deciding administrator.
    pub decided_by: Option<String>,
    /// Required-nullable decision reason.
    pub reason: Option<String>,
    /// Optimistic record version.
    pub version: u64,
}

/// Durable result for one authenticated state-changing request ID.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdempotencyResultRecord {
    /// Digest of purpose, signer, and request ID.
    pub scope_key: String,
    /// Exact proof purpose.
    pub purpose: String,
    /// Authenticated principal or key ID.
    pub signer_id: String,
    /// Caller-generated request ID.
    pub request_id: String,
    /// Digest of the exact canonical request.
    pub request_digest: String,
    /// Replayable successful response.
    pub result: Value,
    /// Commit time in Unix milliseconds.
    pub created_at: i64,
    /// Retention expiry in Unix milliseconds.
    pub expires_at: i64,
}

/// Kind of post-commit side effect.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PostCommitActionKind {
    /// Publish one canonical auth event.
    Event,
    /// Kick exact active NATS connections.
    Kick,
}

/// Durable post-commit event or connection-kick intent.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostCommitActionRecord {
    /// Deterministic action ID.
    pub action_id: String,
    /// Side-effect kind.
    pub kind: PostCommitActionKind,
    /// Canonical adapter payload.
    pub payload: Value,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Number of failed or abandoned claims.
    pub attempts: u32,
    /// Earliest next dispatch time.
    pub next_attempt_at: i64,
    /// Required-nullable active dispatch claim expiry.
    pub claimed_until: Option<i64>,
    /// Required-nullable most recent error.
    pub last_error: Option<String>,
}
