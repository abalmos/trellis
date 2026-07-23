use std::collections::BTreeMap;
use std::sync::Arc;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use ulid::Ulid;
use url::Url;

use super::account::{hash_password, normalize_username, verify_password};
use super::{
    AccountCreation, AccountFlowCreation, AccountFlowKind, AccountFlowRecord,
    AccountFlowRepository, AccountFlowState, AccountRepository, ActivationReviewCreation,
    ActivationReviewDecision, AuthSessionRepository, AuthorityDecision, AuthorityDecisionOutcome,
    AuthorityDecisionRecord, AuthorityKind, AuthorityProposalCreation, AuthorityProposalDecision,
    AuthorityProposalKind, AuthorityProposalRecord, AuthorityProposalRepository,
    AuthorityProposalState, AuthorityState, AuthorityTarget,
    AuthorizationMaterializationRepository, AuthorizationStateError, AuthorizationStateService,
    DeploymentAuthorityRepository, DeploymentRecord, DesiredAuthorityRecord,
    DeviceActivationReviewRecord, DeviceActivationReviewState, DeviceDelegationRecord,
    DeviceProvisioning, DeviceProvisioningSecretConsumption, DeviceProvisioningSecretRecord,
    DeviceRecord, DeviceState, EvidenceRepository, FirstAdminCompletion, IdempotencyResultRecord,
    IdempotentOutcome, IdentityAuthorityRecord, IdentityAuthorityRepository,
    IdentityLinkCompletion, LocalCredentialRecord, LocalLoginAttempt, NewSession,
    ParticipantBindingRecord, ParticipantBindingRepository, ParticipantBindingState,
    PasswordChange, PasswordResetCompletion, PostCommitActionRecord, PrincipalKind,
    PrincipalRecord, PrincipalRepository, PrincipalState, ProviderIdentityLink,
    ProvisionedIdentityKind, ProvisionedIdentityRecord, ProvisionedIdentityState,
    ProvisioningRepository, ProvisioningSecretState, RuntimeInstanceRecord, RuntimeInstanceState,
    ServiceIdentityProvisioning, SessionCreation, SessionRecord, SessionRepository,
    SessionRevocation, SessionRuntimeBinding, UserAccountMutation, UserProfileRecord,
};
use trellis_protocol::{
    canonicalize_json, compare_api_replacement_v1, parse_api_v1, parse_participant_v1,
    resolve_participant_v1, ApiArtifactV1, GrantSetV1, ParticipantKindV1,
};

/// Security settings for the Rust-owned authentication service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthServiceConfig {
    /// Minimum local-password length in Unicode scalar values.
    pub password_min_length: usize,
    /// Consecutive failures that trigger a local-account lock.
    pub maximum_login_failures: u32,
    /// Local-account lock duration in milliseconds.
    pub login_lock_duration_ms: u64,
    /// Lifetime of a one-time first-administrator flow in milliseconds.
    pub first_admin_flow_ttl_ms: u64,
    /// Default authenticated-session lifetime in milliseconds.
    pub session_ttl_ms: u64,
    /// Lifetime of a one-time device provisioning secret in milliseconds.
    pub device_provisioning_secret_ttl_ms: u64,
}

impl Default for AuthServiceConfig {
    fn default() -> Self {
        Self {
            password_min_length: 12,
            maximum_login_failures: 5,
            login_lock_duration_ms: 15 * 60_000,
            first_admin_flow_ttl_ms: 24 * 60 * 60_000,
            session_ttl_ms: 24 * 60 * 60_000,
            device_provisioning_secret_ttl_ms: 15 * 60_000,
        }
    }
}

/// Uniform result of local username/password authentication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocalAuthentication {
    /// Credentials and current account lifecycle are valid.
    Authenticated {
        /// Stable active user principal.
        principal: PrincipalRecord,
    },
    /// Credentials or current account lifecycle are not eligible.
    Denied,
}

/// Exact internal authority target installed by first-admin completion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirstAdminAuthorityTarget {
    /// Internal administration participant ID.
    pub participant_id: String,
    /// Exact internal participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact internal participant needs digest.
    pub participant_needs_digest: String,
}

/// One-time first-administrator startup result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirstAdminBootstrap {
    /// Built-in portal URL containing the one-time bearer secret, present only when first created.
    pub bootstrap_url: Option<String>,
    /// Digest of the secret stored in durable state and safe to log separately.
    pub flow_id_hash: String,
    /// Flow expiry in Unix milliseconds.
    pub expires_at: i64,
}

/// Input for atomic first-administrator account completion.
#[derive(Clone, Debug)]
pub struct FirstAdminRegistration {
    /// Raw one-time bearer token from the startup URL.
    pub token: String,
    /// Expected pending flow version.
    pub expected_flow_version: u64,
    /// Desired local username.
    pub username: String,
    /// New plaintext password, retained only for this call.
    pub password: String,
    /// User-facing profile name.
    pub display_name: String,
    /// Required-nullable profile email.
    pub email: Option<String>,
    /// Required-nullable profile image URL.
    pub image_url: Option<String>,
    /// Exact app or agent participant ID receiving admin authority.
    pub participant_id: String,
    /// Exact participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact accepted needs digest.
    pub participant_needs_digest: String,
    /// Exact grants required to invoke administrator surfaces.
    pub grant_set: GrantSetV1,
    /// Required-nullable authority expiry.
    pub authority_expires_at: Option<i64>,
    /// Completion time in Unix milliseconds.
    pub completed_at: i64,
    /// Durable proof claim; its result is replaced with the committed principal ID.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Input for atomic federated first-administrator account completion.
#[derive(Clone, Debug)]
pub struct FirstAdminFederatedRegistration {
    /// Raw one-time bearer token from the startup URL.
    pub token: String,
    /// Expected pending flow version.
    pub expected_flow_version: u64,
    /// Configured OIDC provider ID.
    pub provider: String,
    /// Verified immutable provider subject.
    pub provider_subject: String,
    /// Required-nullable user-facing profile name.
    pub display_name: Option<String>,
    /// Required-nullable verified profile email.
    pub email: Option<String>,
    /// Required-nullable profile image URL.
    pub image_url: Option<String>,
    /// Exact administration participant ID.
    pub participant_id: String,
    /// Exact administration participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact accepted administration participant needs digest.
    pub participant_needs_digest: String,
    /// Exact grants required to invoke administrator surfaces.
    pub grant_set: GrantSetV1,
    /// Required-nullable authority expiry.
    pub authority_expires_at: Option<i64>,
    /// Completion time in Unix milliseconds.
    pub completed_at: i64,
    /// Durable proof claim; its result is replaced with the committed principal ID.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Newly committed first-administrator account.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirstAdminAccount {
    /// Stable administrator principal.
    pub principal: PrincipalRecord,
    /// Current administrator profile.
    pub profile: UserProfileRecord,
}

/// Input shared by user, service, and device session creation.
#[derive(Clone, Debug)]
pub struct CreateSessionInput {
    /// Stable authenticated principal ID.
    pub principal_id: String,
    /// Authenticated principal class.
    pub principal_kind: PrincipalKind,
    /// Exact participant ID.
    pub participant_id: String,
    /// Exact participant class.
    pub participant_kind: ParticipantKindV1,
    /// Exact participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact participant needs digest.
    pub participant_needs_digest: String,
    /// Canonical Ed25519 session public key.
    pub session_public_key: String,
    /// Optional identity authority accepted during this user bind.
    pub desired_authority: Option<DesiredAuthorityRecord>,
    /// Required deployment ID for service and device sessions.
    pub deployment_id: Option<String>,
    /// Required runtime instance ID for service and device sessions.
    pub instance_id: Option<String>,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Durable proof claim; its result is replaced with the committed session ID.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Service-owned input for an immutable authority proposal.
#[derive(Clone, Debug)]
pub struct CreateAuthorityProposalInput {
    /// Typed authority class.
    pub authority_kind: AuthorityKind,
    /// Stable desired-authority ID being proposed.
    pub authority_id: String,
    /// Deployment owning this lineage; absent for identity authority.
    pub deployment_id: Option<String>,
    /// Proposal intent.
    pub proposal_kind: AuthorityProposalKind,
    /// Exact participant ID.
    pub participant_id: String,
    /// Exact participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact participant needs digest.
    pub participant_needs_digest: String,
    /// Proposed exact grants.
    pub grant_set: GrantSetV1,
    /// Proposed canonical platform capabilities.
    pub capabilities: Vec<String>,
    /// Authority version against which this semantic proposal was derived.
    pub base_authority_version: Option<u64>,
    /// Immutable proposal metadata.
    pub payload: serde_json::Value,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Required-nullable proposal expiry.
    pub expires_at: Option<i64>,
    /// Durable proof claim; its result is replaced with the proposal ID.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Exact participant and API presentation used to plan deployment authority.
#[derive(Clone, Debug)]
pub struct PresentDeploymentAuthorityInput {
    /// Stable deployment receiving the participant binding.
    pub deployment_id: String,
    /// Full `trellis.participant.v1` artifact.
    pub participant_artifact: Value,
    /// Every exact API artifact referenced by the participant.
    pub referenced_api_artifacts: Vec<Value>,
    /// Proposal creation time in Unix milliseconds.
    pub created_at: i64,
    /// Required-nullable administrative proposal expiry.
    pub expires_at: Option<i64>,
    /// Durable request identity.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Service-owned input for one terminal proposal decision.
#[derive(Clone, Debug)]
pub struct DecideAuthorityProposalInput {
    /// Stable proposal ID.
    pub proposal_id: String,
    /// Expected pending proposal version.
    pub expected_version: u64,
    /// Accepted or rejected outcome.
    pub outcome: AuthorityDecisionOutcome,
    /// Stable deciding principal or operator.
    pub decided_by: String,
    /// Required-nullable safe reason.
    pub reason: Option<String>,
    /// Exact desired authority for acceptance; absent for rejection.
    pub desired_authority: Option<DesiredAuthorityRecord>,
    /// Decision time in Unix milliseconds.
    pub decided_at: i64,
    /// Durable proof claim; its result is replaced with the terminal decision.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Client-keyed service identity provisioning input.
#[derive(Clone, Debug)]
pub struct ProvisionServiceIdentityInput {
    /// Existing deployment ID.
    pub deployment_id: String,
    /// Caller-selected stable instance ID, or a generated ID.
    pub instance_id: Option<String>,
    /// Canonical client-generated Ed25519 identity public key.
    pub identity_public_key: String,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Durable proof claim; its result is replaced with committed identities.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Administrator device provisioning input.
#[derive(Clone, Debug)]
pub struct ProvisionDeviceInput {
    /// Existing deployment ID.
    pub deployment_id: String,
    /// Caller-selected stable instance ID, or a generated ID.
    pub instance_id: Option<String>,
    /// Optional client-generated identity installed during provisioning.
    pub identity_public_key: Option<String>,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Durable proof claim; its result excludes the one-time raw secret.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// One-time result returned only when device provisioning is first applied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionedDevice {
    /// Stable device principal ID.
    pub principal_id: String,
    /// Stable runtime instance ID.
    pub instance_id: String,
    /// Raw one-time provisioning secret, never stored by Trellis.
    pub provisioning_secret: Option<String>,
    /// Secret expiry in Unix milliseconds.
    pub expires_at: i64,
}

/// Device identity enrollment input using a one-time provisioning secret.
#[derive(Clone, Debug)]
pub struct EnrollDeviceIdentityInput {
    /// Raw one-time provisioning secret.
    pub provisioning_secret: String,
    /// Expected pending secret version.
    pub expected_version: u64,
    /// Stable provisioned device principal ID.
    pub principal_id: String,
    /// Stable deployment ID.
    pub deployment_id: String,
    /// Stable runtime instance ID.
    pub instance_id: String,
    /// Canonical device-generated Ed25519 identity public key.
    pub identity_public_key: String,
    /// Enrollment time in Unix milliseconds.
    pub consumed_at: i64,
    /// Durable proof claim; its result is replaced with identity metadata.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Device activation-review request input.
#[derive(Clone, Debug)]
pub struct CreateActivationReviewInput {
    /// Stable device principal ID.
    pub principal_id: String,
    /// Stable deployment ID.
    pub deployment_id: String,
    /// Stable runtime instance ID.
    pub instance_id: String,
    /// Canonical activation-request digest.
    pub request_digest: String,
    /// Immutable request metadata.
    pub payload: serde_json::Value,
    /// Request time in Unix milliseconds.
    pub requested_at: i64,
    /// Durable proof claim; its result is replaced with the review ID.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Administrator activation-review decision input.
#[derive(Clone, Debug)]
pub struct DecideActivationReviewInput {
    /// Stable review ID.
    pub review_id: String,
    /// Expected pending review version.
    pub expected_version: u64,
    /// Approved or rejected terminal state.
    pub state: DeviceActivationReviewState,
    /// Stable deciding principal or operator.
    pub decided_by: String,
    /// Required-nullable safe reason.
    pub reason: Option<String>,
    /// Optional approved delegation.
    pub delegation: Option<DeviceDelegationRecord>,
    /// Decision time in Unix milliseconds.
    pub decided_at: i64,
    /// Durable proof claim; its result is replaced with the review outcome.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Administrator input for a credential-less user account.
#[derive(Clone, Debug)]
pub struct CreateUserInput {
    /// Required-nullable display name.
    pub name: Option<String>,
    /// Required-nullable email address.
    pub email: Option<String>,
    /// Required-nullable image URL.
    pub image: Option<String>,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Durable request proof and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Browser input for atomic local-identity registration.
#[derive(Clone, Debug)]
pub struct CreateLocalUserInput {
    /// Canonicalizable local username.
    pub username: String,
    /// Plaintext password retained only for this call.
    pub password: String,
    /// Required-nullable display name.
    pub name: Option<String>,
    /// Required-nullable email address.
    pub email: Option<String>,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Durable request proof and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// OIDC-authenticated user registration input.
#[derive(Clone, Debug)]
pub struct CreateFederatedUserInput {
    /// Stable provider ID.
    pub provider: String,
    /// Stable provider subject.
    pub provider_subject: String,
    /// Required-nullable display name.
    pub name: Option<String>,
    /// Required-nullable email address.
    pub email: Option<String>,
    /// Required-nullable image URL.
    pub image: Option<String>,
    /// Registration time in Unix milliseconds.
    pub created_at: i64,
    /// Durable callback claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Administrator input for an optimistic user-account replacement.
#[derive(Clone, Debug)]
pub struct UpdateUserInput {
    /// Stable user principal ID.
    pub principal_id: String,
    /// Expected principal and profile version.
    pub expected_version: u64,
    /// Required-nullable display name.
    pub name: Option<String>,
    /// Required-nullable email address.
    pub email: Option<String>,
    /// Required-nullable image URL.
    pub image: Option<String>,
    /// Requested active or disabled lifecycle state.
    pub state: PrincipalState,
    /// Update time in Unix milliseconds.
    pub updated_at: i64,
    /// Durable request proof and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// User principal joined with its non-authority profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserAccount {
    /// Durable user principal.
    pub principal: PrincipalRecord,
    /// Required user profile.
    pub profile: UserProfileRecord,
}

/// Service-owned single-use account-flow input.
#[derive(Clone, Debug)]
pub struct CreateAccountFlowInput {
    /// Password reset or identity-link purpose.
    pub kind: AccountFlowKind,
    /// Required-nullable target user principal.
    pub target_principal_id: Option<String>,
    /// Required-nullable target provider.
    pub target_provider_id: Option<String>,
    /// Required-nullable validated return location.
    pub return_location: Option<String>,
    /// Immutable flow metadata.
    pub payload: serde_json::Value,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Expiry time in Unix milliseconds.
    pub expires_at: i64,
    /// Durable request proof; its result excludes the raw token.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// One-time account-flow result returned only on first application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatedAccountFlow {
    /// Stable non-secret flow ID.
    pub flow_id: String,
    /// Raw one-time bearer token, never stored by Trellis.
    pub token: String,
    /// Flow expiry in Unix milliseconds.
    pub expires_at: i64,
}

/// Identity-link flow completion input.
#[derive(Clone, Debug)]
pub struct CompleteIdentityLinkInput {
    /// Raw one-time flow token.
    pub token: String,
    /// Expected pending flow version.
    pub expected_flow_version: u64,
    /// Exact provider identity link.
    pub identity: ProviderIdentityLink,
    /// Completion time in Unix milliseconds.
    pub completed_at: i64,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Single Rust-owned composition root for auth domain behavior.
#[derive(Clone, Debug)]
pub struct AuthService<R> {
    repository: R,
    authorization: AuthorizationStateService<R>,
    config: AuthServiceConfig,
    dummy_password_hash: Arc<str>,
}

impl<R> AuthService<R>
where
    R: Clone,
{
    /// Construct auth behavior over one coherent repository set.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationStateError::InvalidRecord`] for unsafe password
    /// or lockout settings, or a storage error if the uniform-failure hash
    /// cannot be generated.
    pub fn new(repository: R, config: AuthServiceConfig) -> Result<Self, AuthorizationStateError> {
        if config.maximum_login_failures == 0
            || config.login_lock_duration_ms == 0
            || config.first_admin_flow_ttl_ms == 0
            || config.session_ttl_ms == 0
            || config.device_provisioning_secret_ttl_ms == 0
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "local login lockout limits must be positive".to_owned(),
            ));
        }
        let (dummy_password_hash, _) = hash_password(
            "trellis uniform local authentication failure",
            Some(config.password_min_length),
        )?;
        Ok(Self {
            authorization: AuthorizationStateService::new(repository.clone()),
            repository,
            config,
            dummy_password_hash: dummy_password_hash.into(),
        })
    }

    /// Borrow the accepted authorization-state component.
    #[must_use]
    pub fn authorization(&self) -> &AuthorizationStateService<R> {
        &self.authorization
    }

    /// Borrow the coherent auth repository set.
    #[must_use]
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

impl<R> AuthService<R>
where
    R: AccountFlowRepository + AccountRepository + AuthorizationMaterializationRepository + Clone,
{
    /// Complete a first-administrator flow and reconcile its exact authority.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for malformed tokens, unsafe passwords,
    /// or inconsistent authority input, and a conflict if the flow was already
    /// consumed or an administrator became active concurrently.
    pub async fn complete_first_admin(
        &self,
        mut input: FirstAdminRegistration,
    ) -> Result<IdempotentOutcome<FirstAdminAccount>, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("completedAt", input.completed_at)?;
        let token = URL_SAFE_NO_PAD.decode(&input.token).map_err(|_| {
            AuthorizationStateError::InvalidRecord(
                "first-admin token is not canonical base64url".to_owned(),
            )
        })?;
        if token.len() != 32 || URL_SAFE_NO_PAD.encode(&token) != input.token {
            return Err(AuthorizationStateError::InvalidRecord(
                "first-admin token must canonically encode 32 bytes".to_owned(),
            ));
        }
        let token_hash = URL_SAFE_NO_PAD.encode(Sha256::digest(token));
        let username = normalize_username(&input.username)?;
        let (password_hash, hash_profile) =
            hash_password(&input.password, Some(self.config.password_min_length))?;
        let principal_id = format!("usr_{}", Ulid::new());
        let principal = PrincipalRecord {
            principal_id: principal_id.clone(),
            kind: PrincipalKind::User,
            state: PrincipalState::Active,
            created_at: input.completed_at,
            updated_at: input.completed_at,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        };
        let profile = UserProfileRecord {
            principal_id: principal_id.clone(),
            display_name: Some(input.display_name),
            email: input.email,
            image_url: input.image_url,
            created_at: input.completed_at,
            updated_at: input.completed_at,
            version: 1,
        };
        let credential = LocalCredentialRecord {
            principal_id: principal_id.clone(),
            normalized_username: username.clone(),
            password_hash,
            hash_profile,
            failed_attempts: 0,
            locked_until: None,
            password_changed_at: input.completed_at,
            updated_at: input.completed_at,
            version: 1,
        };
        let local_identity = ProviderIdentityLink {
            provider: "local".to_owned(),
            provider_subject: username,
            principal_id: principal_id.clone(),
            linked_at: input.completed_at,
            last_seen_at: input.completed_at,
        };
        let authority = IdentityAuthorityRecord {
            authority_id: format!("auth_{}", Ulid::new()),
            principal_id: principal_id.clone(),
            participant_id: input.participant_id,
            participant_artifact_digest: input.participant_artifact_digest,
            accepted_needs_digest: input.participant_needs_digest,
            desired_grant_set: input.grant_set,
            desired_capabilities: vec!["admin".to_owned()],
            state: AuthorityState::Accepted,
            version: 1,
            created_at: input.completed_at,
            updated_at: input.completed_at,
            expires_at: input.authority_expires_at,
            decision: Some(AuthorityDecision {
                decided_at: input.completed_at,
                decided_by: "system:first-admin".to_owned(),
                reason: None,
            }),
        };
        input.idempotency.result = json!({
            "principalId": principal_id,
            "authorityId": authority.authority_id,
        });
        let outcome = self
            .repository
            .complete_first_admin(FirstAdminCompletion {
                token_hash,
                expected_flow_version: input.expected_flow_version,
                principal: principal.clone(),
                profile: profile.clone(),
                credential: Some(credential),
                identity: local_identity,
                authority: authority.clone(),
                consumed_at: input.completed_at,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?;
        let authority_id = match &outcome {
            IdempotentOutcome::Applied(_) => authority.authority_id,
            IdempotentOutcome::Replayed(value) => value
                .get("authorityId")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    AuthorizationStateError::Storage(
                        "first-admin replay result has no authorityId".to_owned(),
                    )
                })?
                .to_owned(),
        };
        self.authorization
            .reconcile_authority(
                &AuthorityTarget {
                    kind: AuthorityKind::Identity,
                    authority_id,
                },
                input.completed_at,
            )
            .await?;
        Ok(match outcome {
            IdempotentOutcome::Applied(_) => {
                IdempotentOutcome::Applied(FirstAdminAccount { principal, profile })
            }
            IdempotentOutcome::Replayed(value) => IdempotentOutcome::Replayed(value),
        })
    }

    /// Complete a first-administrator flow with one verified federated identity.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for malformed tokens or identity data, and
    /// a conflict if the flow was consumed or an administrator became active.
    pub async fn complete_first_admin_federated(
        &self,
        mut input: FirstAdminFederatedRegistration,
    ) -> Result<IdempotentOutcome<FirstAdminAccount>, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("completedAt", input.completed_at)?;
        let token = URL_SAFE_NO_PAD.decode(&input.token).map_err(|_| {
            AuthorizationStateError::InvalidRecord(
                "first-admin token is not canonical base64url".to_owned(),
            )
        })?;
        if token.len() != 32 || URL_SAFE_NO_PAD.encode(&token) != input.token {
            return Err(AuthorizationStateError::InvalidRecord(
                "first-admin token must canonically encode 32 bytes".to_owned(),
            ));
        }
        let token_hash = URL_SAFE_NO_PAD.encode(Sha256::digest(token));
        let principal_id = format!("usr_{}", Ulid::new());
        let principal = PrincipalRecord {
            principal_id: principal_id.clone(),
            kind: PrincipalKind::User,
            state: PrincipalState::Active,
            created_at: input.completed_at,
            updated_at: input.completed_at,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        };
        let profile = UserProfileRecord {
            principal_id: principal_id.clone(),
            display_name: input.display_name,
            email: input.email,
            image_url: input.image_url,
            created_at: input.completed_at,
            updated_at: input.completed_at,
            version: 1,
        };
        let identity = ProviderIdentityLink {
            provider: input.provider,
            provider_subject: input.provider_subject,
            principal_id: principal_id.clone(),
            linked_at: input.completed_at,
            last_seen_at: input.completed_at,
        };
        let authority = IdentityAuthorityRecord {
            authority_id: format!("auth_{}", Ulid::new()),
            principal_id: principal_id.clone(),
            participant_id: input.participant_id,
            participant_artifact_digest: input.participant_artifact_digest,
            accepted_needs_digest: input.participant_needs_digest,
            desired_grant_set: input.grant_set,
            desired_capabilities: vec!["admin".to_owned()],
            state: AuthorityState::Accepted,
            version: 1,
            created_at: input.completed_at,
            updated_at: input.completed_at,
            expires_at: input.authority_expires_at,
            decision: Some(AuthorityDecision {
                decided_at: input.completed_at,
                decided_by: "system:first-admin".to_owned(),
                reason: Some("federated first-administrator bootstrap".to_owned()),
            }),
        };
        input.idempotency.result = json!({
            "principalId": principal_id,
            "authorityId": authority.authority_id,
        });
        let outcome = self
            .repository
            .complete_first_admin(FirstAdminCompletion {
                token_hash,
                expected_flow_version: input.expected_flow_version,
                principal: principal.clone(),
                profile: profile.clone(),
                credential: None,
                identity,
                authority: authority.clone(),
                consumed_at: input.completed_at,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?;
        let authority_id = match &outcome {
            IdempotentOutcome::Applied(_) => authority.authority_id,
            IdempotentOutcome::Replayed(value) => value
                .get("authorityId")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    AuthorizationStateError::Storage(
                        "first-admin replay result has no authorityId".to_owned(),
                    )
                })?
                .to_owned(),
        };
        self.authorization
            .reconcile_authority(
                &AuthorityTarget {
                    kind: AuthorityKind::Identity,
                    authority_id,
                },
                input.completed_at,
            )
            .await?;
        Ok(match outcome {
            IdempotentOutcome::Applied(_) => {
                IdempotentOutcome::Applied(FirstAdminAccount { principal, profile })
            }
            IdempotentOutcome::Replayed(value) => IdempotentOutcome::Replayed(value),
        })
    }
}

impl<R> AuthService<R>
where
    R: AuthSessionRepository
        + AuthorizationMaterializationRepository
        + DeploymentAuthorityRepository
        + super::EvidenceRepository
        + IdentityAuthorityRepository
        + SessionRepository
        + Clone,
{
    /// Create any principal session through the single aggregate path.
    ///
    /// This generates the session ID and inbox prefix, commits exact authority
    /// or runtime evidence, and reconciles the applicable authority. Replays
    /// retry reconciliation against the previously committed session.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for mismatched principal, participant,
    /// deployment, or instance inputs; repository conflicts remain fail-closed.
    pub async fn create_session(
        &self,
        mut input: CreateSessionInput,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("createdAt", input.created_at)?;
        let expires_at = u64::try_from(input.created_at)
            .ok()
            .and_then(|created| created.checked_add(self.config.session_ttl_ms))
            .filter(|expires| *expires <= super::MAX_PROTOCOL_INTEGER)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("session expiry overflow".to_owned())
            })? as i64;
        let session_id = format!("ses_{}", Ulid::new());
        let session = SessionRecord::from_new(NewSession {
            session_id: session_id.clone(),
            principal_id: input.principal_id,
            principal_kind: input.principal_kind,
            participant_id: input.participant_id,
            participant_kind: input.participant_kind,
            participant_artifact_digest: input.participant_artifact_digest,
            participant_needs_digest: input.participant_needs_digest,
            session_public_key: input.session_public_key,
            inbox_prefix: format!("_INBOX.{session_id}"),
            created_at: input.created_at,
            expires_at: Some(expires_at),
        })?;
        let runtime_binding = match (input.deployment_id, input.instance_id) {
            (None, None) => None,
            (Some(deployment_id), Some(instance_id)) => Some(SessionRuntimeBinding {
                session_id: session_id.clone(),
                deployment_id,
                instance_id,
            }),
            _ => {
                return Err(AuthorizationStateError::InvalidRecord(
                    "deploymentId and instanceId must be supplied together".to_owned(),
                ));
            }
        };
        input.idempotency.result = json!({ "sessionId": session_id });
        let outcome = self
            .repository
            .create_session(SessionCreation {
                session: session.clone(),
                desired_authority: input.desired_authority,
                runtime_binding,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?;
        let committed = match &outcome {
            IdempotentOutcome::Applied(session) => session.clone(),
            IdempotentOutcome::Replayed(value) => {
                let session_id = value
                    .get("sessionId")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| {
                        AuthorizationStateError::Storage(
                            "session replay result has no sessionId".to_owned(),
                        )
                    })?;
                self.repository
                    .get_session(session_id)
                    .await?
                    .ok_or(AuthorizationStateError::SessionMissing)?
            }
        };
        let target = match committed.principal_kind {
            PrincipalKind::User => self
                .repository
                .get_identity_authority(&committed.principal_id, &committed.participant_id)
                .await?
                .map(|authority| AuthorityTarget {
                    kind: AuthorityKind::Identity,
                    authority_id: authority.authority_id,
                }),
            PrincipalKind::Service | PrincipalKind::Device => {
                let binding = self
                    .repository
                    .get_session_runtime_binding(&committed.session_id)
                    .await?
                    .ok_or(AuthorizationStateError::AuthorityMissing)?;
                self.repository
                    .get_deployment_authority(&binding.deployment_id, &committed.participant_id)
                    .await?
                    .map(|authority| AuthorityTarget {
                        kind: AuthorityKind::Deployment,
                        authority_id: authority.authority_id,
                    })
            }
        }
        .ok_or(AuthorizationStateError::AuthorityMissing)?;
        self.authorization
            .reconcile_authority(&target, input.created_at)
            .await?;
        Ok(outcome)
    }

    /// Revoke a session and durably enqueue its event and kick intents.
    ///
    /// # Errors
    ///
    /// Returns a conflict when the expected active version changed, or an
    /// invalid-record error unless both event and kick actions are supplied.
    pub async fn revoke_session(
        &self,
        session_id: String,
        expected_version: u64,
        revoked_at: i64,
        mut idempotency: IdempotencyResultRecord,
        actions: Vec<PostCommitActionRecord>,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError> {
        idempotency.result = json!({ "sessionId": session_id, "state": "revoked" });
        self.repository
            .revoke_session(SessionRevocation {
                session_id,
                expected_version,
                revoked_at,
                idempotency,
                actions,
            })
            .await
    }
}

impl<R> AuthService<R>
where
    R: ParticipantBindingRepository
        + DeploymentAuthorityRepository
        + AuthorityProposalRepository
        + AuthorizationMaterializationRepository
        + EvidenceRepository
        + Clone,
{
    /// Parse, bind, classify, and create or reuse one deployment-authority proposal.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error when any artifact, digest, or reference is
    /// invalid, and a repository error when the exact binding or proposal cannot
    /// be committed.
    pub async fn present_deployment_authority(
        &self,
        input: PresentDeploymentAuthorityInput,
    ) -> Result<IdempotentOutcome<AuthorityProposalRecord>, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("createdAt", input.created_at)?;
        let participant = parse_participant_v1(&input.participant_artifact)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let mut apis = BTreeMap::<String, ApiArtifactV1>::new();
        let mut canonical_apis = BTreeMap::new();
        for value in input.referenced_api_artifacts {
            let api = parse_api_v1(&value)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let id = api.id().to_owned();
            if let Some(existing) = apis.get(&id) {
                if existing
                    .digest()
                    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
                    != api.digest().map_err(|error| {
                        AuthorizationStateError::InvalidRecord(error.to_string())
                    })?
                {
                    return Err(AuthorizationStateError::InvalidRecord(format!(
                        "conflicting API artifacts are presented for {id}"
                    )));
                }
                continue;
            }
            canonical_apis.insert(
                id.clone(),
                api.normalized_value()
                    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            );
            apis.insert(id, api);
        }
        let resolved = resolve_participant_v1(&participant, &apis)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let participant_digest = resolved.participant_digest().to_owned();
        let needs_digest = resolved
            .needs()
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let binding = ParticipantBindingRecord {
            participant_id: resolved.participant_id().to_owned(),
            participant_kind: resolved.participant_kind(),
            artifact_digest: participant_digest.clone(),
            needs_digest: needs_digest.clone(),
            participant_json: participant
                .canonical_json()
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            api_artifacts_json: canonicalize_json(
                &serde_json::to_value(&canonical_apis)
                    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            )
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            resolved_at: input.created_at,
            state: ParticipantBindingState::Resolved,
            error: None,
        };
        let current = self
            .repository
            .get_deployment_authority(&input.deployment_id, resolved.participant_id())
            .await?;
        let proposal_kind = if let Some(current) = &current {
            if current.participant_artifact_digest == participant_digest {
                AuthorityProposalKind::Update
            } else {
                let previous = self
                    .repository
                    .get_participant_binding(
                        resolved.participant_id(),
                        &current.participant_artifact_digest,
                    )
                    .await?
                    .ok_or(AuthorizationStateError::ParticipantMissing)?;
                if participant_api_update_is_compatible(&previous, &binding)? {
                    AuthorityProposalKind::Update
                } else {
                    AuthorityProposalKind::Migration
                }
            }
        } else {
            AuthorityProposalKind::Initial
        };
        self.repository.put_participant_binding(binding).await?;

        let proposal = resolved.proposal();
        let grant_set = GrantSetV1::new(
            proposal
                .required()
                .grant_set()
                .permissions()
                .iter()
                .chain(proposal.optional().grant_set().permissions())
                .cloned()
                .collect(),
        );
        let capabilities = proposal
            .required()
            .capabilities()
            .iter()
            .chain(proposal.optional().capabilities())
            .map(|capability| capability.name().to_owned())
            .collect();
        self.create_authority_proposal(CreateAuthorityProposalInput {
            authority_kind: AuthorityKind::Deployment,
            authority_id: super::service_domain::deployment_authority_id(
                &input.deployment_id,
                resolved.participant_id(),
            )?,
            deployment_id: Some(input.deployment_id.clone()),
            proposal_kind,
            participant_id: resolved.participant_id().to_owned(),
            participant_artifact_digest: participant_digest,
            participant_needs_digest: needs_digest,
            grant_set,
            capabilities,
            base_authority_version: current.as_ref().map(|authority| authority.version),
            payload: json!({
                "deploymentId": input.deployment_id,
                "subjectId": input.deployment_id,
                "baseAuthorityVersion": current.as_ref().map(|authority| authority.version),
                "reasons": [],
                "resolution": proposal.normalized_value().map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            }),
            created_at: input.created_at,
            expires_at: input.expires_at,
            idempotency: input.idempotency,
            actions: input.actions,
        })
        .await
    }
}

impl<R> AuthService<R>
where
    R: AuthorityProposalRepository
        + AuthorizationMaterializationRepository
        + EvidenceRepository
        + Clone,
{
    /// Create one immutable authority proposal with a service-owned digest.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for malformed or non-canonical proposal
    /// input and a repository conflict for duplicate immutable identities.
    pub async fn create_authority_proposal(
        &self,
        mut input: CreateAuthorityProposalInput,
    ) -> Result<IdempotentOutcome<AuthorityProposalRecord>, AuthorizationStateError> {
        let proposal_id = format!("apr_{}", Ulid::new());
        let capabilities = super::domain::canonical_capabilities(input.capabilities.clone())?;
        let proposal_digest = proposal_semantic_digest(&input, &capabilities)?;
        let proposal = AuthorityProposalRecord {
            proposal_id: proposal_id.clone(),
            authority_kind: input.authority_kind,
            authority_id: input.authority_id,
            deployment_id: input.deployment_id,
            proposal_kind: input.proposal_kind,
            participant_id: input.participant_id,
            participant_artifact_digest: input.participant_artifact_digest,
            participant_needs_digest: input.participant_needs_digest,
            proposed_grant_set: input.grant_set,
            proposed_capabilities: capabilities,
            proposal_digest,
            payload: input.payload,
            state: AuthorityProposalState::Pending,
            created_at: input.created_at,
            expires_at: input.expires_at,
            superseded_at: None,
            version: 1,
        };
        input.idempotency.result = json!({ "proposalId": proposal_id });
        self.repository
            .create_authority_proposal(AuthorityProposalCreation {
                proposal,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await
    }

    /// Commit one terminal proposal decision and reconcile accepted authority.
    ///
    /// A replay retries reconciliation from the durable authority identity in
    /// the result, closing an unknown outcome after a post-commit failure.
    ///
    /// # Errors
    ///
    /// Returns a conflict for stale or terminal proposals and an invalid-record
    /// error when accepted authority does not exactly match the proposal.
    pub async fn decide_authority_proposal(
        &self,
        mut input: DecideAuthorityProposalInput,
    ) -> Result<IdempotentOutcome<AuthorityProposalRecord>, AuthorizationStateError> {
        let proposal_id = input.proposal_id.clone();
        let decision_digest = protocol_digest(&json!({
            "proposalId": input.proposal_id,
            "outcome": input.outcome,
            "decidedBy": input.decided_by,
            "reason": input.reason,
            "decidedAt": input.decided_at,
        }))?;
        let target = input.desired_authority.as_ref().map(authority_target);
        let deployment = match input.desired_authority.as_ref() {
            Some(DesiredAuthorityRecord::Deployment(authority))
                if self
                    .repository
                    .get_deployment_evidence(&authority.deployment_id)
                    .await?
                    .is_none() =>
            {
                Some(DeploymentRecord {
                    deployment_id: authority.deployment_id.clone(),
                    participant_id: authority.participant_id.clone(),
                    participant_kind: authority.participant_kind,
                    active: true,
                    expires_at: authority.expires_at,
                })
            }
            _ => None,
        };
        input.idempotency.result = match &target {
            Some(target) => json!({
                "proposalId": input.proposal_id,
                "outcome": input.outcome,
                "authorityKind": target.kind,
                "authorityId": target.authority_id,
            }),
            None => json!({
                "proposalId": input.proposal_id,
                "outcome": input.outcome,
                "authorityKind": null,
                "authorityId": null,
            }),
        };
        let outcome = self
            .repository
            .decide_authority_proposal(AuthorityProposalDecision {
                proposal_id: input.proposal_id,
                expected_version: input.expected_version,
                decision: AuthorityDecisionRecord {
                    proposal_id,
                    outcome: input.outcome,
                    decided_by: input.decided_by,
                    reason: input.reason,
                    decided_at: input.decided_at,
                    decision_digest,
                },
                desired_authority: input.desired_authority,
                deployment,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?;
        let replay_target = match &outcome {
            IdempotentOutcome::Applied(_) => target,
            IdempotentOutcome::Replayed(value) => {
                match (
                    value
                        .get("authorityKind")
                        .and_then(serde_json::Value::as_str),
                    value.get("authorityId").and_then(serde_json::Value::as_str),
                ) {
                    (Some(kind), Some(authority_id)) => Some(AuthorityTarget {
                        kind: match kind {
                            "identity" => AuthorityKind::Identity,
                            "deployment" => AuthorityKind::Deployment,
                            _ => {
                                return Err(AuthorizationStateError::Storage(
                                    "proposal replay has invalid authorityKind".to_owned(),
                                ));
                            }
                        },
                        authority_id: authority_id.to_owned(),
                    }),
                    (None, None) => None,
                    _ => {
                        return Err(AuthorizationStateError::Storage(
                            "proposal replay has incomplete authority identity".to_owned(),
                        ));
                    }
                }
            }
        };
        if let Some(target) = replay_target {
            self.authorization
                .reconcile_authority(&target, input.decided_at)
                .await?;
        }
        Ok(outcome)
    }
}

impl<R> AuthService<R>
where
    R: ProvisioningRepository + Clone,
{
    /// Provision immutable service identity metadata around a client-generated key.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for malformed keys or timestamps and a
    /// conflict when deployment or stable identity relationships do not match.
    pub async fn provision_service_identity(
        &self,
        mut input: ProvisionServiceIdentityInput,
    ) -> Result<IdempotentOutcome<ProvisionedIdentityRecord>, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("createdAt", input.created_at)?;
        let identity_key_id = super::domain::validate_ed25519_public_key(
            "identityPublicKey",
            &input.identity_public_key,
        )?;
        let principal_id = format!("svc_{}", Ulid::new());
        let instance_id = input
            .instance_id
            .take()
            .unwrap_or_else(|| format!("ins_{}", Ulid::new()));
        let principal = PrincipalRecord {
            principal_id: principal_id.clone(),
            kind: PrincipalKind::Service,
            state: PrincipalState::Active,
            created_at: input.created_at,
            updated_at: input.created_at,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        };
        let instance = RuntimeInstanceRecord {
            instance_id: instance_id.clone(),
            deployment_id: input.deployment_id.clone(),
            principal_id: principal_id.clone(),
            state: RuntimeInstanceState::Active,
            created_at: input.created_at,
            updated_at: input.created_at,
            version: 1,
        };
        let identity = ProvisionedIdentityRecord {
            identity_key_id: identity_key_id.clone(),
            identity_public_key: input.identity_public_key,
            principal_id: principal_id.clone(),
            deployment_id: input.deployment_id.clone(),
            instance_id: instance_id.clone(),
            kind: ProvisionedIdentityKind::Service,
            state: ProvisionedIdentityState::Active,
            created_at: input.created_at,
            revoked_at: None,
        };
        input.idempotency.result = json!({
            "principalId": principal_id,
            "instanceId": instance_id,
            "identityKeyId": identity_key_id,
        });
        self.repository
            .provision_service_identity(ServiceIdentityProvisioning {
                principal,
                instance,
                identity,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await
    }

    /// Provision a disabled device and return its raw secret exactly once.
    ///
    /// The durable replay result deliberately excludes the raw secret. A caller
    /// that loses the first successful response must create a new provisioning
    /// record rather than recover secret material from Trellis.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for unsafe timestamps and a repository
    /// conflict when the deployment or generated identities cannot be committed.
    pub async fn provision_device(
        &self,
        mut input: ProvisionDeviceInput,
    ) -> Result<IdempotentOutcome<ProvisionedDevice>, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("createdAt", input.created_at)?;
        let expires_at = u64::try_from(input.created_at)
            .ok()
            .and_then(|created| created.checked_add(self.config.device_provisioning_secret_ttl_ms))
            .filter(|expires| *expires <= super::MAX_PROTOCOL_INTEGER)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(
                    "device provisioning expiry overflow".to_owned(),
                )
            })? as i64;
        let mut secret = [0_u8; 32];
        getrandom::fill(&mut secret).map_err(|error| {
            AuthorizationStateError::Storage(format!(
                "device provisioning secret generation failed: {error}"
            ))
        })?;
        let provisioning_secret = URL_SAFE_NO_PAD.encode(secret);
        let secret_hash = URL_SAFE_NO_PAD.encode(Sha256::digest(secret));
        let principal_id = format!("dev_{}", Ulid::new());
        let instance_id = input
            .instance_id
            .take()
            .unwrap_or_else(|| format!("ins_{}", Ulid::new()));
        let principal = PrincipalRecord {
            principal_id: principal_id.clone(),
            kind: PrincipalKind::Device,
            state: PrincipalState::Active,
            created_at: input.created_at,
            updated_at: input.created_at,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        };
        let instance = RuntimeInstanceRecord {
            instance_id: instance_id.clone(),
            deployment_id: input.deployment_id.clone(),
            principal_id: principal_id.clone(),
            state: RuntimeInstanceState::Active,
            created_at: input.created_at,
            updated_at: input.created_at,
            version: 1,
        };
        let device = DeviceRecord {
            principal_id: principal_id.clone(),
            deployment_id: input.deployment_id.clone(),
            state: DeviceState::Pending,
            created_at: input.created_at,
            updated_at: input.created_at,
            version: 1,
        };
        let returns_secret = input.identity_public_key.is_none();
        let identity = input
            .identity_public_key
            .map(|identity_public_key| {
                let identity_key_id = super::domain::validate_ed25519_public_key(
                    "identityPublicKey",
                    &identity_public_key,
                )?;
                Ok(ProvisionedIdentityRecord {
                    identity_key_id,
                    identity_public_key,
                    principal_id: principal_id.clone(),
                    deployment_id: input.deployment_id.clone(),
                    instance_id: instance_id.clone(),
                    kind: ProvisionedIdentityKind::Device,
                    state: ProvisionedIdentityState::Active,
                    created_at: input.created_at,
                    revoked_at: None,
                })
            })
            .transpose()?;
        let durable_secret = DeviceProvisioningSecretRecord {
            secret_id: format!("dps_{}", Ulid::new()),
            instance_id: instance_id.clone(),
            secret_hash,
            state: if identity.is_some() {
                ProvisioningSecretState::Consumed
            } else {
                ProvisioningSecretState::Pending
            },
            created_at: input.created_at,
            expires_at,
            consumed_at: identity.as_ref().map(|_| input.created_at),
            version: 1,
        };
        input.idempotency.result = json!({
            "principalId": principal_id,
            "instanceId": instance_id,
            "expiresAt": expires_at,
        });
        match self
            .repository
            .provision_device(DeviceProvisioning {
                principal,
                instance,
                device,
                identity,
                secret: durable_secret,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?
        {
            IdempotentOutcome::Applied(_) => Ok(IdempotentOutcome::Applied(ProvisionedDevice {
                principal_id,
                instance_id,
                provisioning_secret: returns_secret.then_some(provisioning_secret),
                expires_at,
            })),
            IdempotentOutcome::Replayed(value) => Ok(IdempotentOutcome::Replayed(value)),
        }
    }

    /// Consume one device secret and bind its client-generated identity key.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for malformed secret or key encodings,
    /// and a conflict for expired, consumed, or mismatched provisioning state.
    pub async fn enroll_device_identity(
        &self,
        mut input: EnrollDeviceIdentityInput,
    ) -> Result<IdempotentOutcome<ProvisionedIdentityRecord>, AuthorizationStateError> {
        let secret_hash = bearer_secret_digest(&input.provisioning_secret)?;
        let identity_key_id = super::domain::validate_ed25519_public_key(
            "identityPublicKey",
            &input.identity_public_key,
        )?;
        let identity = ProvisionedIdentityRecord {
            identity_key_id: identity_key_id.clone(),
            identity_public_key: input.identity_public_key,
            principal_id: input.principal_id,
            deployment_id: input.deployment_id,
            instance_id: input.instance_id,
            kind: ProvisionedIdentityKind::Device,
            state: ProvisionedIdentityState::Active,
            created_at: input.consumed_at,
            revoked_at: None,
        };
        input.idempotency.result = json!({ "identityKeyId": identity_key_id });
        match self
            .repository
            .consume_device_provisioning_secret(DeviceProvisioningSecretConsumption {
                secret_hash,
                expected_version: input.expected_version,
                identity: identity.clone(),
                consumed_at: input.consumed_at,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?
        {
            IdempotentOutcome::Applied(_) => Ok(IdempotentOutcome::Applied(identity)),
            IdempotentOutcome::Replayed(value) => Ok(IdempotentOutcome::Replayed(value)),
        }
    }

    /// Create one immutable pending device activation review.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for malformed request evidence and a
    /// conflict unless device, deployment, and runtime instance match exactly.
    pub async fn create_activation_review(
        &self,
        mut input: CreateActivationReviewInput,
    ) -> Result<IdempotentOutcome<DeviceActivationReviewRecord>, AuthorizationStateError> {
        let review_id = format!("dar_{}", Ulid::new());
        let review = DeviceActivationReviewRecord {
            review_id: review_id.clone(),
            principal_id: input.principal_id,
            deployment_id: input.deployment_id,
            instance_id: input.instance_id,
            request_digest: input.request_digest,
            payload: input.payload,
            state: DeviceActivationReviewState::Pending,
            requested_at: input.requested_at,
            decided_at: None,
            decided_by: None,
            reason: None,
            version: 1,
        };
        input.idempotency.result = json!({ "reviewId": review_id });
        self.repository
            .create_activation_review(ActivationReviewCreation {
                review,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await
    }

    /// Decide a device activation review and apply approved device state.
    ///
    /// # Errors
    ///
    /// Returns a conflict for stale reviews and an invalid-record error for
    /// unsupported states or inconsistent delegation evidence.
    pub async fn decide_activation_review(
        &self,
        mut input: DecideActivationReviewInput,
    ) -> Result<IdempotentOutcome<DeviceActivationReviewRecord>, AuthorizationStateError> {
        input.idempotency.result = json!({
            "reviewId": input.review_id,
            "state": input.state,
        });
        self.repository
            .decide_activation_review(ActivationReviewDecision {
                review_id: input.review_id,
                expected_version: input.expected_version,
                state: input.state,
                decided_at: input.decided_at,
                decided_by: input.decided_by,
                reason: input.reason,
                delegation: input.delegation,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await
    }
}

impl<R> AuthService<R>
where
    R: AccountRepository + Clone,
{
    /// Create one local user, profile, credential, and identity atomically.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for unsafe credentials or malformed
    /// profile data, and a conflict when the normalized username already exists.
    pub async fn create_local_user(
        &self,
        mut input: CreateLocalUserInput,
    ) -> Result<IdempotentOutcome<UserAccount>, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("createdAt", input.created_at)?;
        let username = normalize_username(&input.username)?;
        let (password_hash, hash_profile) =
            hash_password(&input.password, Some(self.config.password_min_length))?;
        let principal_id = format!("usr_{}", Ulid::new());
        let principal = PrincipalRecord {
            principal_id: principal_id.clone(),
            kind: PrincipalKind::User,
            state: PrincipalState::Active,
            created_at: input.created_at,
            updated_at: input.created_at,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        };
        let profile = UserProfileRecord {
            principal_id: principal_id.clone(),
            display_name: input.name,
            email: input.email,
            image_url: None,
            created_at: input.created_at,
            updated_at: input.created_at,
            version: 1,
        };
        input.idempotency.result = json!({ "principalId": principal_id });
        match self
            .repository
            .create_user_account(AccountCreation {
                principal: principal.clone(),
                profile: profile.clone(),
                credential: Some(LocalCredentialRecord {
                    principal_id: principal_id.clone(),
                    normalized_username: username.clone(),
                    password_hash,
                    hash_profile,
                    failed_attempts: 0,
                    locked_until: None,
                    password_changed_at: input.created_at,
                    updated_at: input.created_at,
                    version: 1,
                }),
                identity: Some(ProviderIdentityLink {
                    provider: "local".to_owned(),
                    provider_subject: username,
                    principal_id: principal_id.clone(),
                    linked_at: input.created_at,
                    last_seen_at: input.created_at,
                }),
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?
        {
            IdempotentOutcome::Applied(_) => Ok(IdempotentOutcome::Applied(UserAccount {
                principal,
                profile,
            })),
            IdempotentOutcome::Replayed(value) => Ok(IdempotentOutcome::Replayed(value)),
        }
    }

    /// Create one credential-less user account through the aggregate path.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for malformed profile data or timestamps
    /// and a conflict for duplicate durable request or principal identity.
    pub async fn create_user(
        &self,
        mut input: CreateUserInput,
    ) -> Result<IdempotentOutcome<UserAccount>, AuthorizationStateError> {
        let principal_id = format!("usr_{}", Ulid::new());
        let principal = PrincipalRecord {
            principal_id: principal_id.clone(),
            kind: PrincipalKind::User,
            state: PrincipalState::Active,
            created_at: input.created_at,
            updated_at: input.created_at,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        };
        let profile = UserProfileRecord {
            principal_id: principal_id.clone(),
            display_name: input.name,
            email: input.email,
            image_url: input.image,
            created_at: input.created_at,
            updated_at: input.created_at,
            version: 1,
        };
        input.idempotency.result = json!({ "principalId": principal_id });
        match self
            .repository
            .create_user_account(AccountCreation {
                principal: principal.clone(),
                profile: profile.clone(),
                credential: None,
                identity: None,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?
        {
            IdempotentOutcome::Applied(_) => Ok(IdempotentOutcome::Applied(UserAccount {
                principal,
                profile,
            })),
            IdempotentOutcome::Replayed(value) => Ok(IdempotentOutcome::Replayed(value)),
        }
    }

    /// Create one user and immutable federated provider link atomically.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for malformed profile or provider data,
    /// and a conflict when the provider identity is already assigned.
    pub async fn create_federated_user(
        &self,
        mut input: CreateFederatedUserInput,
    ) -> Result<IdempotentOutcome<UserAccount>, AuthorizationStateError> {
        let principal_id = format!("usr_{}", Ulid::new());
        let principal = PrincipalRecord {
            principal_id: principal_id.clone(),
            kind: PrincipalKind::User,
            state: PrincipalState::Active,
            created_at: input.created_at,
            updated_at: input.created_at,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        };
        let profile = UserProfileRecord {
            principal_id: principal_id.clone(),
            display_name: input.name,
            email: input.email,
            image_url: input.image,
            created_at: input.created_at,
            updated_at: input.created_at,
            version: 1,
        };
        let identity = ProviderIdentityLink {
            provider: input.provider,
            provider_subject: input.provider_subject,
            principal_id: principal_id.clone(),
            linked_at: input.created_at,
            last_seen_at: input.created_at,
        };
        input.idempotency.result = json!({ "principalId": principal_id });
        match self
            .repository
            .create_user_account(AccountCreation {
                principal: principal.clone(),
                profile: profile.clone(),
                credential: None,
                identity: Some(identity),
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?
        {
            IdempotentOutcome::Applied(_) => Ok(IdempotentOutcome::Applied(UserAccount {
                principal,
                profile,
            })),
            IdempotentOutcome::Replayed(value) => Ok(IdempotentOutcome::Replayed(value)),
        }
    }

    /// Load one user account by stable principal ID.
    ///
    /// # Errors
    ///
    /// Returns a repository error when the coherent principal/profile read fails.
    pub async fn user(
        &self,
        principal_id: &str,
    ) -> Result<Option<UserAccount>, AuthorizationStateError> {
        Ok(self
            .repository
            .get_user_account(principal_id)
            .await?
            .map(|(principal, profile)| UserAccount { principal, profile }))
    }

    /// List user accounts after an exclusive principal-ID cursor.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for an unsafe limit and a repository
    /// error when the coherent page cannot be read.
    pub async fn users(
        &self,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<Vec<UserAccount>, AuthorizationStateError> {
        Ok(self
            .repository
            .list_user_accounts(cursor, limit)
            .await?
            .into_iter()
            .map(|(principal, profile)| UserAccount { principal, profile })
            .collect())
    }

    /// Atomically replace a user lifecycle and profile.
    ///
    /// # Errors
    ///
    /// Returns a conflict when the expected version changed and an
    /// invalid-record error for revoked or malformed replacement input.
    pub async fn update_user(
        &self,
        mut input: UpdateUserInput,
    ) -> Result<IdempotentOutcome<UserAccount>, AuthorizationStateError> {
        if !matches!(
            input.state,
            PrincipalState::Active | PrincipalState::Disabled
        ) {
            return Err(AuthorizationStateError::InvalidRecord(
                "user update state must be active or disabled".to_owned(),
            ));
        }
        let (current_principal, current_profile) = self
            .repository
            .get_user_account(&input.principal_id)
            .await?
            .ok_or(AuthorizationStateError::PrincipalMissing)?;
        if current_principal.version != input.expected_version
            || current_profile.version != input.expected_version
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let version = input.expected_version.checked_add(1).ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("user version overflow".to_owned())
        })?;
        let principal = PrincipalRecord {
            state: input.state,
            updated_at: input.updated_at,
            version,
            disabled_at: (input.state == PrincipalState::Disabled).then_some(input.updated_at),
            ..current_principal
        };
        let profile = UserProfileRecord {
            display_name: input.name,
            email: input.email,
            image_url: input.image,
            updated_at: input.updated_at,
            version,
            ..current_profile
        };
        input.idempotency.result = json!({
            "principalId": input.principal_id,
            "version": version,
        });
        match self
            .repository
            .update_user_account(UserAccountMutation {
                principal: principal.clone(),
                profile: profile.clone(),
                expected_version: input.expected_version,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?
        {
            IdempotentOutcome::Applied(_) => Ok(IdempotentOutcome::Applied(UserAccount {
                principal,
                profile,
            })),
            IdempotentOutcome::Replayed(value) => Ok(IdempotentOutcome::Replayed(value)),
        }
    }
}

impl<R> AuthService<R>
where
    R: AccountFlowRepository + Clone,
{
    /// Create a password-reset or identity-link flow and return its token once.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for first-admin or malformed flow input,
    /// and a repository conflict when durable identities cannot be committed.
    pub async fn create_account_flow(
        &self,
        mut input: CreateAccountFlowInput,
    ) -> Result<IdempotentOutcome<CreatedAccountFlow>, AuthorizationStateError> {
        if input.kind == AccountFlowKind::FirstAdmin {
            return Err(AuthorizationStateError::InvalidRecord(
                "first-admin flows use ensure_first_admin_flow".to_owned(),
            ));
        }
        let mut secret = [0_u8; 32];
        getrandom::fill(&mut secret).map_err(|error| {
            AuthorizationStateError::Storage(format!(
                "account-flow secret generation failed: {error}"
            ))
        })?;
        let token = URL_SAFE_NO_PAD.encode(secret);
        let token_hash = URL_SAFE_NO_PAD.encode(Sha256::digest(secret));
        let flow_id = format!("afl_{}", Ulid::new());
        let flow = AccountFlowRecord {
            flow_id: flow_id.clone(),
            kind: input.kind,
            token_hash,
            target_principal_id: input.target_principal_id,
            target_provider_id: input.target_provider_id,
            return_location: input.return_location,
            payload: input.payload,
            state: AccountFlowState::Pending,
            created_at: input.created_at,
            expires_at: input.expires_at,
            consumed_at: None,
            version: 1,
        };
        input.idempotency.result = json!({
            "flowId": flow_id,
            "expiresAt": input.expires_at,
        });
        match self
            .repository
            .create_account_flow(AccountFlowCreation {
                flow,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?
        {
            IdempotentOutcome::Applied(_) => Ok(IdempotentOutcome::Applied(CreatedAccountFlow {
                flow_id,
                token,
                expires_at: input.expires_at,
            })),
            IdempotentOutcome::Replayed(value) => Ok(IdempotentOutcome::Replayed(value)),
        }
    }

    /// Consume an identity-link flow and attach the exact provider identity.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for malformed token or identity input and
    /// a conflict for expired, consumed, or mismatched flow state.
    pub async fn complete_identity_link(
        &self,
        mut input: CompleteIdentityLinkInput,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError> {
        let token_hash = bearer_secret_digest(&input.token)?;
        input.idempotency.result = json!({
            "principalId": input.identity.principal_id,
            "provider": input.identity.provider,
        });
        self.repository
            .complete_identity_link(IdentityLinkCompletion {
                token_hash,
                expected_flow_version: input.expected_flow_version,
                identity: input.identity,
                consumed_at: input.completed_at,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await
    }
}

fn protocol_digest(value: &serde_json::Value) -> Result<String, AuthorizationStateError> {
    trellis_protocol::digest_json(value).map_err(|error| {
        AuthorizationStateError::InvalidRecord(format!(
            "value cannot be canonically digested: {error}"
        ))
    })
}

fn proposal_semantic_digest(
    input: &CreateAuthorityProposalInput,
    capabilities: &[String],
) -> Result<String, AuthorizationStateError> {
    protocol_digest(&json!({
        "format": "trellis.authority-proposal-semantic.v1",
        "authorityKind": input.authority_kind,
        "authorityId": input.authority_id,
        "proposalKind": input.proposal_kind,
        "participantId": input.participant_id,
        "participantArtifactDigest": input.participant_artifact_digest,
        "participantNeedsDigest": input.participant_needs_digest,
        "grantSet": input.grant_set,
        "capabilities": capabilities,
        "baseAuthorityVersion": input.base_authority_version,
    }))
}

fn participant_api_update_is_compatible(
    previous: &ParticipantBindingRecord,
    candidate: &ParticipantBindingRecord,
) -> Result<bool, AuthorizationStateError> {
    let previous = binding_apis(previous)?;
    let candidate = binding_apis(candidate)?;
    for (id, candidate) in candidate {
        let Some(previous) = previous.get(&id) else {
            continue;
        };
        if previous
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
            != candidate
                .digest()
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
            && !compare_api_replacement_v1(previous, &candidate)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
                .compatible
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn binding_apis(
    binding: &ParticipantBindingRecord,
) -> Result<BTreeMap<String, ApiArtifactV1>, AuthorizationStateError> {
    let values: BTreeMap<String, Value> = serde_json::from_str(&binding.api_artifacts_json)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    values
        .into_iter()
        .map(|(id, value)| {
            let api = parse_api_v1(&value)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            if api.id() != id {
                return Err(AuthorizationStateError::InvalidRecord(format!(
                    "API artifact map key {id} does not match {}",
                    api.id()
                )));
            }
            Ok((id, api))
        })
        .collect()
}

fn bearer_secret_digest(value: &str) -> Result<String, AuthorizationStateError> {
    let secret = URL_SAFE_NO_PAD.decode(value).map_err(|_| {
        AuthorizationStateError::InvalidRecord("secret is not canonical base64url".to_owned())
    })?;
    if secret.len() != 32 || URL_SAFE_NO_PAD.encode(&secret) != value {
        return Err(AuthorizationStateError::InvalidRecord(
            "secret must canonically encode 32 bytes".to_owned(),
        ));
    }
    Ok(URL_SAFE_NO_PAD.encode(Sha256::digest(secret)))
}

fn authority_target(authority: &DesiredAuthorityRecord) -> AuthorityTarget {
    match authority {
        DesiredAuthorityRecord::Identity(authority) => AuthorityTarget {
            kind: AuthorityKind::Identity,
            authority_id: authority.authority_id.clone(),
        },
        DesiredAuthorityRecord::Deployment(authority) => AuthorityTarget {
            kind: AuthorityKind::Deployment,
            authority_id: authority.authority_id.clone(),
        },
    }
}

impl<R> AuthService<R>
where
    R: AccountRepository + Clone,
{
    /// Create or report one pending first-administrator flow when no active admin exists.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for an invalid base URL or timestamp,
    /// and a repository or entropy error when the flow cannot be committed.
    pub async fn ensure_first_admin_flow(
        &self,
        portal_base_url: &str,
        authority_target: &FirstAdminAuthorityTarget,
        now: i64,
    ) -> Result<Option<FirstAdminBootstrap>, AuthorizationStateError> {
        self.first_admin_flow(portal_base_url, authority_target, now, false)
            .await
    }

    /// Explicitly revoke an existing pending first-administrator flow and create a new one.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for an invalid base URL or timestamp,
    /// and a repository or entropy error when rotation cannot be committed.
    pub async fn rotate_first_admin_flow(
        &self,
        portal_base_url: &str,
        authority_target: &FirstAdminAuthorityTarget,
        now: i64,
    ) -> Result<Option<FirstAdminBootstrap>, AuthorizationStateError> {
        self.first_admin_flow(portal_base_url, authority_target, now, true)
            .await
    }

    async fn first_admin_flow(
        &self,
        portal_base_url: &str,
        authority_target: &FirstAdminAuthorityTarget,
        now: i64,
        rotate: bool,
    ) -> Result<Option<FirstAdminBootstrap>, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("now", now)?;
        let mut bootstrap_url = Url::parse(portal_base_url).map_err(|_| {
            AuthorizationStateError::InvalidRecord("portal base URL is invalid".to_owned())
        })?;
        let expires_at = u64::try_from(now)
            .ok()
            .and_then(|now| now.checked_add(self.config.first_admin_flow_ttl_ms))
            .filter(|expires| *expires <= super::MAX_PROTOCOL_INTEGER)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("first-admin expiry overflow".to_owned())
            })? as i64;
        let mut secret = [0_u8; 32];
        getrandom::fill(&mut secret).map_err(|error| {
            AuthorizationStateError::Storage(format!(
                "first-admin secret generation failed: {error}"
            ))
        })?;
        let token = URL_SAFE_NO_PAD.encode(secret);
        let token_hash = URL_SAFE_NO_PAD.encode(Sha256::digest(secret));
        let flow = AccountFlowRecord {
            flow_id: format!("afl_{}", Ulid::new()),
            kind: AccountFlowKind::FirstAdmin,
            token_hash: token_hash.clone(),
            target_principal_id: None,
            target_provider_id: None,
            return_location: None,
            payload: json!({
                "participantId": authority_target.participant_id,
                "participantArtifactDigest": authority_target.participant_artifact_digest,
                "participantNeedsDigest": authority_target.participant_needs_digest,
            }),
            state: AccountFlowState::Pending,
            created_at: now,
            expires_at,
            consumed_at: None,
            version: 1,
        };
        let stored = if let Some(stored) = self
            .repository
            .replace_first_admin_flow(flow, now, rotate)
            .await?
        {
            stored
        } else {
            return Ok(None);
        };
        let created = stored.token_hash == token_hash;
        let bootstrap_url = if created {
            bootstrap_url.set_path("/_trellis/portal/account/password");
            bootstrap_url.set_query(None);
            bootstrap_url
                .query_pairs_mut()
                .append_pair("flowId", &token);
            Some(bootstrap_url.into())
        } else {
            None
        };
        Ok(Some(FirstAdminBootstrap {
            bootstrap_url,
            flow_id_hash: stored.token_hash,
            expires_at: stored.expires_at,
        }))
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::{
        proposal_semantic_digest, AuthService, AuthServiceConfig, CreateAuthorityProposalInput,
        FirstAdminAuthorityTarget, LocalAuthentication,
    };
    use crate::platform::auth::account::hash_password;
    use crate::platform::auth::{
        AccountCreation, AccountFlowRepository, AccountFlowState, AccountRepository, AuthorityKind,
        AuthorityProposalKind, IdempotencyResultRecord, InMemoryAuthorizationStore,
        LocalCredentialRecord, PrincipalKind, PrincipalRecord, PrincipalState,
        ProviderIdentityLink, SqliteAuthorizationStore, UserProfileRecord,
    };
    use trellis_protocol::GrantSetV1;

    const NOW: i64 = 1_700_000_000_000;

    #[test]
    fn semantic_proposal_digest_ignores_record_metadata_and_tracks_base_version() {
        let mut input = proposal_input(None);
        let capabilities = vec!["read".to_owned()];
        let first = proposal_semantic_digest(&input, &capabilities).unwrap();
        input.created_at += 50;
        input.expires_at = Some(NOW + 9_000);
        input.idempotency.request_id = "another-request".to_owned();
        input.idempotency.request_digest = "another-digest".to_owned();
        assert_eq!(
            proposal_semantic_digest(&input, &capabilities).unwrap(),
            first
        );
        input.base_authority_version = Some(1);
        assert_ne!(
            proposal_semantic_digest(&input, &capabilities).unwrap(),
            first
        );
    }

    fn proposal_input(base_authority_version: Option<u64>) -> CreateAuthorityProposalInput {
        CreateAuthorityProposalInput {
            authority_kind: AuthorityKind::Deployment,
            authority_id: "dau_test".to_owned(),
            deployment_id: Some("dep_test".to_owned()),
            proposal_kind: AuthorityProposalKind::Initial,
            participant_id: "participant.test@v1".to_owned(),
            participant_artifact_digest: "artifact".to_owned(),
            participant_needs_digest: "needs".to_owned(),
            grant_set: GrantSetV1::new(Vec::new()),
            capabilities: vec!["read".to_owned()],
            base_authority_version,
            payload: serde_json::json!({ "presentation": "ignored" }),
            created_at: NOW,
            expires_at: Some(NOW + 1_000),
            idempotency: IdempotencyResultRecord {
                scope_key: "scope".to_owned(),
                purpose: "proposal".to_owned(),
                signer_id: "signer".to_owned(),
                request_id: "request".to_owned(),
                request_digest: "digest".to_owned(),
                result: serde_json::Value::Null,
                created_at: NOW,
                expires_at: NOW + 1_000,
            },
            actions: Vec::new(),
        }
    }

    #[tokio::test]
    async fn in_memory_local_authentication_is_uniform_and_locks() {
        exercise_local_authentication(InMemoryAuthorizationStore::default())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn sqlite_local_authentication_is_uniform_and_locks() {
        exercise_local_authentication(SqliteAuthorizationStore::open_in_memory().unwrap())
            .await
            .unwrap();
    }

    async fn exercise_local_authentication<R>(
        repository: R,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        R: AccountFlowRepository
            + AccountRepository
            + crate::platform::auth::PrincipalRepository
            + Clone,
    {
        let (password_hash, hash_profile) = hash_password("password1", Some(8))?;
        repository
            .create_user_account(AccountCreation {
                principal: PrincipalRecord {
                    principal_id: "usr_login".to_owned(),
                    kind: PrincipalKind::User,
                    state: PrincipalState::Active,
                    created_at: NOW,
                    updated_at: NOW,
                    version: 1,
                    disabled_at: None,
                    revoked_at: None,
                },
                profile: UserProfileRecord {
                    principal_id: "usr_login".to_owned(),
                    display_name: Some("Login User".to_owned()),
                    email: None,
                    image_url: None,
                    created_at: NOW,
                    updated_at: NOW,
                    version: 1,
                },
                credential: Some(LocalCredentialRecord {
                    principal_id: "usr_login".to_owned(),
                    normalized_username: "login".to_owned(),
                    password_hash,
                    hash_profile,
                    failed_attempts: 0,
                    locked_until: None,
                    password_changed_at: NOW,
                    updated_at: NOW,
                    version: 1,
                }),
                identity: Some(ProviderIdentityLink {
                    provider: "local".to_owned(),
                    provider_subject: "login".to_owned(),
                    principal_id: "usr_login".to_owned(),
                    linked_at: NOW,
                    last_seen_at: NOW,
                }),
                idempotency: IdempotencyResultRecord {
                    scope_key: "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE".to_owned(),
                    purpose: "account.create".to_owned(),
                    signer_id: "test".to_owned(),
                    request_id: "create-login-user".to_owned(),
                    request_digest: "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI".to_owned(),
                    result: serde_json::json!({ "principalId": "usr_login" }),
                    created_at: NOW,
                    expires_at: NOW + 1_000,
                },
                actions: Vec::new(),
            })
            .await?;
        let service = AuthService::new(
            repository.clone(),
            AuthServiceConfig {
                password_min_length: 8,
                maximum_login_failures: 2,
                login_lock_duration_ms: 100,
                ..AuthServiceConfig::default()
            },
        )?;
        let target = FirstAdminAuthorityTarget {
            participant_id: "trellis-platform-administration".to_owned(),
            participant_artifact_digest: "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE".to_owned(),
            participant_needs_digest: "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI".to_owned(),
        };

        assert!(service
            .ensure_first_admin_flow("not a URL", &target, NOW)
            .await
            .is_err());
        let first = service
            .ensure_first_admin_flow("https://auth.example/base", &target, NOW)
            .await?
            .ok_or("first-admin flow missing")?;
        let first_record = repository
            .get_account_flow_by_hash(&first.flow_id_hash)
            .await?
            .ok_or("first-admin record missing")?;
        assert_eq!(first_record.state, AccountFlowState::Pending);
        assert!(!first
            .bootstrap_url
            .as_deref()
            .ok_or("first-admin URL missing")?
            .contains(&first.flow_id_hash));
        let second = service
            .ensure_first_admin_flow("https://auth.example/base", &target, NOW + 1)
            .await?
            .ok_or("pending first-admin flow missing")?;
        assert_eq!(second.flow_id_hash, first.flow_id_hash);
        assert!(second.bootstrap_url.is_none());
        assert_eq!(
            repository
                .get_account_flow_by_hash(&first.flow_id_hash)
                .await?
                .ok_or("old first-admin record missing")?
                .state,
            AccountFlowState::Pending
        );
        let replacement = service
            .rotate_first_admin_flow("https://auth.example/base", &target, NOW + 2)
            .await?
            .ok_or("first-admin flow was not rotated")?;
        assert_ne!(replacement.flow_id_hash, first.flow_id_hash);
        assert!(replacement.bootstrap_url.is_some());
        assert_eq!(
            repository
                .get_account_flow_by_hash(&first.flow_id_hash)
                .await?
                .ok_or("rotated first-admin record missing")?
                .state,
            AccountFlowState::Revoked
        );
        let after_expiry = service
            .ensure_first_admin_flow(
                "https://auth.example/base",
                &target,
                replacement.expires_at + 1,
            )
            .await?
            .ok_or("expired first-admin flow was not replaced")?;
        assert_ne!(after_expiry.flow_id_hash, replacement.flow_id_hash);
        assert_eq!(
            repository
                .get_account_flow_by_hash(&replacement.flow_id_hash)
                .await?
                .ok_or("expired first-admin record missing")?
                .state,
            AccountFlowState::Expired
        );

        assert_eq!(
            service
                .authenticate_local("missing", "password1", NOW)
                .await?,
            LocalAuthentication::Denied
        );
        assert_eq!(
            service
                .authenticate_local("LOGIN", "wrong", NOW + 1)
                .await?,
            LocalAuthentication::Denied
        );
        assert_eq!(
            service
                .authenticate_local("login", "wrong", NOW + 2)
                .await?,
            LocalAuthentication::Denied
        );
        let locked = repository
            .get_local_credential("usr_login")
            .await?
            .ok_or("credential missing")?;
        assert_eq!(locked.failed_attempts, 2);
        assert_eq!(locked.locked_until, Some(NOW + 102));
        assert_eq!(
            service
                .authenticate_local("login", "password1", NOW + 3)
                .await?,
            LocalAuthentication::Denied
        );
        assert_eq!(
            repository
                .get_local_credential("usr_login")
                .await?
                .ok_or("credential missing")?
                .version,
            locked.version
        );
        assert!(matches!(
            service
                .authenticate_local("login", "password1", NOW + 102)
                .await?,
            LocalAuthentication::Authenticated { .. }
        ));
        let reset = repository
            .get_local_credential("usr_login")
            .await?
            .ok_or("credential missing")?;
        assert_eq!(reset.failed_attempts, 0);
        assert_eq!(reset.locked_until, None);
        Ok(())
    }
}

impl<R> AuthService<R>
where
    R: AccountRepository + PrincipalRepository + Clone,
{
    /// Authenticate a local account with uniform caller-visible denial.
    ///
    /// # Errors
    ///
    /// Returns a repository error when credential lockout state cannot be read
    /// or committed.
    pub async fn authenticate_local(
        &self,
        username: &str,
        password: &str,
        now: i64,
    ) -> Result<LocalAuthentication, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("now", now)?;
        let Ok(username) = normalize_username(username) else {
            let _ = verify_password(&self.dummy_password_hash, password);
            return Ok(LocalAuthentication::Denied);
        };
        let credential = self
            .repository
            .get_local_credential_by_username(&username)
            .await?;
        let verified = verify_password(
            credential
                .as_ref()
                .map_or(&self.dummy_password_hash, |value| {
                    value.password_hash.as_str()
                }),
            password,
        );
        let Some(credential) = credential else {
            return Ok(LocalAuthentication::Denied);
        };
        if credential.locked_until.is_some_and(|until| until > now) {
            return Ok(LocalAuthentication::Denied);
        }
        self.repository
            .record_local_login_attempt(LocalLoginAttempt {
                principal_id: credential.principal_id.clone(),
                expected_version: credential.version,
                succeeded: verified,
                attempted_at: now,
                maximum_failures: self.config.maximum_login_failures,
                lock_duration_ms: self.config.login_lock_duration_ms,
            })
            .await?;
        if !verified {
            return Ok(LocalAuthentication::Denied);
        }
        let Some(principal) = self
            .repository
            .get_principal(&credential.principal_id)
            .await?
        else {
            return Ok(LocalAuthentication::Denied);
        };
        let Some(_) = self
            .repository
            .get_user_profile(&credential.principal_id)
            .await?
        else {
            return Ok(LocalAuthentication::Denied);
        };
        if principal.kind != PrincipalKind::User || principal.state != PrincipalState::Active {
            return Ok(LocalAuthentication::Denied);
        }
        Ok(LocalAuthentication::Authenticated { principal })
    }
}

impl<R> AuthService<R>
where
    R: AccountFlowRepository + AccountRepository + Clone,
{
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn change_password(
        &self,
        principal_id: &str,
        current_session_id: &str,
        current_password: &str,
        new_password: &str,
        changed_at: i64,
        idempotency: IdempotencyResultRecord,
        actions: Vec<PostCommitActionRecord>,
    ) -> Result<IdempotentOutcome<usize>, AuthorizationStateError> {
        let credential = self
            .repository
            .get_local_credential(principal_id)
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("local credential not found".to_owned())
            })?;
        if !verify_password(&credential.password_hash, current_password) {
            return Err(AuthorizationStateError::InvalidRecord(
                "current password is invalid".to_owned(),
            ));
        }
        let (password_hash, hash_profile) =
            hash_password(new_password, Some(self.config.password_min_length))?;
        let replacement = LocalCredentialRecord {
            password_hash,
            hash_profile,
            failed_attempts: 0,
            locked_until: None,
            password_changed_at: changed_at,
            updated_at: changed_at,
            version: credential.version + 1,
            ..credential.clone()
        };
        self.repository
            .change_password(PasswordChange {
                principal_id: principal_id.to_owned(),
                current_session_id: current_session_id.to_owned(),
                credential: replacement,
                expected_version: credential.version,
                changed_at,
                idempotency,
                actions,
            })
            .await
    }

    /// Replace a local password and consume its durable account flow atomically.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for an unsafe password, or a repository
    /// conflict when the flow or credential changed concurrently.
    pub async fn complete_password_reset(
        &self,
        token: String,
        expected_flow_version: u64,
        password: &str,
        consumed_at: i64,
        mut idempotency: IdempotencyResultRecord,
        actions: Vec<PostCommitActionRecord>,
    ) -> Result<IdempotentOutcome<super::AccountFlowRecord>, AuthorizationStateError> {
        let token_hash = bearer_secret_digest(&token)?;
        let flow = self
            .repository
            .get_account_flow_by_hash(&token_hash)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let principal_id = flow
            .target_principal_id
            .as_deref()
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let current = self
            .repository
            .get_local_credential(principal_id)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let (password_hash, hash_profile) =
            hash_password(password, Some(self.config.password_min_length))?;
        let replacement = LocalCredentialRecord {
            principal_id: current.principal_id.clone(),
            normalized_username: current.normalized_username,
            password_hash,
            hash_profile,
            failed_attempts: 0,
            locked_until: None,
            password_changed_at: consumed_at,
            updated_at: consumed_at,
            version: current.version.checked_add(1).ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("credential version overflow".to_owned())
            })?,
        };
        idempotency.result = json!({ "principalId": principal_id, "completed": true });
        self.repository
            .complete_password_reset(PasswordResetCompletion {
                token_hash,
                expected_flow_version,
                replacement,
                consumed_at,
                idempotency,
                actions,
            })
            .await
    }
}
