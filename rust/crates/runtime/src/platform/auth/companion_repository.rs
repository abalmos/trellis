use async_trait::async_trait;
use serde_json::{json, Value};
use trellis_protocol::ParticipantKindV1;

use super::domain::{
    canonical_capabilities, require_digest, require_nonempty, require_positive,
    require_protocol_timestamp, validate_ed25519_public_key, validate_principal_participant,
};
use super::repository::{
    deployment_enforceability_equal, identity_enforceability_equal, validate_deployment_authority,
    validate_device, validate_device_delegation, validate_identity_authority, validate_principal,
    validate_provider_identity, validate_runtime_instance, validate_session,
    validate_session_runtime_binding, validate_session_runtime_binding_relationships,
};
use super::{
    AccountFlowKind, AccountFlowRecord, AccountFlowState, AuthorityDecisionOutcome,
    AuthorityDecisionRecord, AuthorityKind, AuthorityProposalRecord, AuthorityProposalState,
    AuthorityState, AuthorizationStateError, DeploymentProfileRecord, DeploymentProfileState,
    DeploymentRecord, DesiredAuthorityRecord, DeviceActivationReviewRecord,
    DeviceActivationReviewState, DeviceDelegationRecord, DeviceProvisioningSecretRecord,
    DeviceRecord, DeviceState, IdempotencyResultRecord, IdentityAuthorityRecord,
    InMemoryAuthorizationStore, LocalCredentialRecord, LoginPortalRecord, LoginSettingsRecord,
    PortalRouteRecord, PostCommitActionKind, PostCommitActionRecord, PrincipalKind,
    PrincipalRecord, PrincipalState, ProviderIdentityLink, ProvisionedIdentityKind,
    ProvisionedIdentityRecord, ProvisionedIdentityState, ProvisioningSecretState,
    RuntimeInstanceRecord, RuntimeInstanceState, SessionRecord, SessionRuntimeBinding,
    SessionState, UserProfileRecord, MAX_PROTOCOL_INTEGER,
};

/// Atomic deployment-profile creation.
#[derive(Clone, Debug)]
pub struct DeploymentProfileCreation {
    /// Principal installed for the deployment.
    pub principal: PrincipalRecord,
    /// Product-facing deployment metadata.
    pub profile: DeploymentProfileRecord,
    /// Durable replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic deployment-profile lifecycle mutation.
#[derive(Clone, Debug)]
pub struct DeploymentProfileMutation {
    /// Complete replacement profile.
    pub profile: DeploymentProfileRecord,
    /// Expected current profile version.
    pub expected_version: u64,
    /// Durable replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Product-facing deployment administration repository.
#[async_trait]
pub trait DeploymentProfileRepository: Send + Sync {
    /// Create a deployment principal and profile atomically.
    async fn create_deployment_profile(
        &self,
        command: DeploymentProfileCreation,
    ) -> Result<IdempotentOutcome<DeploymentProfileRecord>, AuthorizationStateError>;

    /// Load one deployment profile.
    async fn get_deployment_profile(
        &self,
        deployment_id: &str,
    ) -> Result<Option<DeploymentProfileRecord>, AuthorizationStateError>;

    /// List deployment profiles in stable ID order.
    async fn list_deployment_profiles(
        &self,
    ) -> Result<Vec<DeploymentProfileRecord>, AuthorizationStateError>;

    /// Replace profile lifecycle state and matching runtime evidence atomically.
    async fn put_deployment_profile(
        &self,
        command: DeploymentProfileMutation,
    ) -> Result<IdempotentOutcome<DeploymentProfileRecord>, AuthorizationStateError>;
}

#[async_trait]
impl DeploymentProfileRepository for InMemoryAuthorizationStore {
    async fn create_deployment_profile(
        &self,
        command: DeploymentProfileCreation,
    ) -> Result<IdempotentOutcome<DeploymentProfileRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_principal(&command.principal)?;
        validate_deployment_profile(&command.profile)?;
        if command.principal.principal_id != command.profile.deployment_id
            || command.principal.kind != command.profile.kind
            || command.principal.version != 1
            || command.profile.version != 1
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let mut staged = state.clone();
        if staged
            .principals
            .insert(command.principal.principal_id.clone(), command.principal)
            .is_some()
            || staged
                .deployment_profiles
                .insert(
                    command.profile.deployment_id.clone(),
                    command.profile.clone(),
                )
                .is_some()
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if let Some(participant_id) = &command.profile.participant_id {
            staged.deployments.insert(
                command.profile.deployment_id.clone(),
                super::DeploymentRecord {
                    deployment_id: command.profile.deployment_id.clone(),
                    participant_id: participant_id.clone(),
                    participant_kind: match command.profile.kind {
                        PrincipalKind::Service => ParticipantKindV1::Service,
                        PrincipalKind::Device => ParticipantKindV1::Device,
                        PrincipalKind::User => unreachable!(),
                    },
                    active: true,
                    expires_at: command.profile.expires_at,
                },
            );
        }
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.profile))
    }

    async fn get_deployment_profile(
        &self,
        deployment_id: &str,
    ) -> Result<Option<DeploymentProfileRecord>, AuthorizationStateError> {
        Ok(self
            .state()?
            .deployment_profiles
            .get(deployment_id)
            .cloned())
    }

    async fn list_deployment_profiles(
        &self,
    ) -> Result<Vec<DeploymentProfileRecord>, AuthorizationStateError> {
        Ok(self
            .state()?
            .deployment_profiles
            .values()
            .cloned()
            .collect())
    }

    async fn put_deployment_profile(
        &self,
        command: DeploymentProfileMutation,
    ) -> Result<IdempotentOutcome<DeploymentProfileRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_deployment_profile(&command.profile)?;
        let current = state
            .deployment_profiles
            .get(&command.profile.deployment_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        if current.version != command.expected_version
            || command.profile.version != next_version(command.expected_version)?
            || current.created_at != command.profile.created_at
            || current.kind != command.profile.kind
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let mut staged = state.clone();
        staged.deployment_profiles.insert(
            command.profile.deployment_id.clone(),
            command.profile.clone(),
        );
        if let Some(principal) = staged.principals.get_mut(&command.profile.deployment_id) {
            principal.state = match command.profile.state {
                DeploymentProfileState::Active => PrincipalState::Active,
                DeploymentProfileState::Disabled => PrincipalState::Disabled,
                DeploymentProfileState::Removed => PrincipalState::Revoked,
            };
            principal.updated_at = command.profile.updated_at;
            principal.disabled_at =
                (principal.state == PrincipalState::Disabled).then_some(command.profile.updated_at);
            principal.revoked_at =
                (principal.state == PrincipalState::Revoked).then_some(command.profile.updated_at);
            principal.version = command.profile.version;
        }
        if let Some(deployment) = staged.deployments.get_mut(&command.profile.deployment_id) {
            if command.profile.participant_id.as_deref() != Some(&deployment.participant_id) {
                return Err(AuthorizationStateError::StorageConflict);
            }
            deployment.active = command.profile.state == DeploymentProfileState::Active;
            deployment.expires_at = command.profile.expires_at;
        } else if let Some(participant_id) = &command.profile.participant_id {
            staged.deployments.insert(
                command.profile.deployment_id.clone(),
                super::DeploymentRecord {
                    deployment_id: command.profile.deployment_id.clone(),
                    participant_id: participant_id.clone(),
                    participant_kind: match command.profile.kind {
                        PrincipalKind::Service => ParticipantKindV1::Service,
                        PrincipalKind::Device => ParticipantKindV1::Device,
                        PrincipalKind::User => unreachable!(),
                    },
                    active: command.profile.state == DeploymentProfileState::Active,
                    expires_at: command.profile.expires_at,
                },
            );
        }
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.profile))
    }
}

/// Result of an idempotent aggregate transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdempotentOutcome<T> {
    /// The mutation was applied and committed.
    Applied(T),
    /// The matching request had already committed this durable JSON result.
    Replayed(Value),
}

/// One optimistic local-login state transition.
#[derive(Clone, Debug)]
pub struct LocalLoginAttempt {
    /// Target local-account principal.
    pub principal_id: String,
    /// Credential version observed during password verification.
    pub expected_version: u64,
    /// Whether password verification succeeded.
    pub succeeded: bool,
    /// Attempt time in Unix milliseconds.
    pub attempted_at: i64,
    /// Consecutive failures that trigger a lock.
    pub maximum_failures: u32,
    /// Lock duration in milliseconds.
    pub lock_duration_ms: u64,
}

/// Atomic password-reset flow completion.
#[derive(Clone, Debug)]
pub struct PasswordResetCompletion {
    /// Bearer-token digest selecting the flow.
    pub token_hash: String,
    /// Expected pending flow version.
    pub expected_flow_version: u64,
    /// Complete replacement credential with version `current + 1`.
    pub replacement: LocalCredentialRecord,
    /// Completion time in Unix milliseconds.
    pub consumed_at: i64,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic provider-identity-link flow completion.
#[derive(Clone, Debug)]
pub struct IdentityLinkCompletion {
    /// Bearer-token digest selecting the flow.
    pub token_hash: String,
    /// Expected pending flow version.
    pub expected_flow_version: u64,
    /// Exact provider identity to attach.
    pub identity: ProviderIdentityLink,
    /// Completion time in Unix milliseconds.
    pub consumed_at: i64,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic first-administrator flow completion.
#[derive(Clone, Debug)]
pub struct FirstAdminCompletion {
    /// Bearer-token digest selecting the flow.
    pub token_hash: String,
    /// Expected pending flow version.
    pub expected_flow_version: u64,
    /// New administrator principal.
    pub principal: PrincipalRecord,
    /// New administrator profile.
    pub profile: UserProfileRecord,
    /// Optional new administrator local credential; federated completion has none.
    pub credential: Option<LocalCredentialRecord>,
    /// New administrator authentication identity.
    pub identity: ProviderIdentityLink,
    /// Accepted administrator authority.
    pub authority: IdentityAuthorityRecord,
    /// Completion time in Unix milliseconds.
    pub consumed_at: i64,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic terminal authority-proposal decision.
#[derive(Clone, Debug)]
pub struct AuthorityProposalDecision {
    /// Proposal ID to decide.
    pub proposal_id: String,
    /// Expected pending proposal version.
    pub expected_version: u64,
    /// Caller-observed authority version for optimistic acceptance; outer `None` skips the check.
    pub expected_base_authority_version: Option<Option<u64>>,
    /// Immutable terminal decision.
    pub decision: AuthorityDecisionRecord,
    /// Accepted desired authority, or `None` for rejection.
    pub desired_authority: Option<DesiredAuthorityRecord>,
    /// Deployment evidence installed atomically with initial deployment authority.
    pub deployment: Option<DeploymentRecord>,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic device provisioning-secret consumption.
#[derive(Clone, Debug)]
pub struct DeviceProvisioningSecretConsumption {
    /// Secret digest selecting the pending secret.
    pub secret_hash: String,
    /// Expected pending secret version.
    pub expected_version: u64,
    /// Immutable device identity created by consumption.
    pub identity: ProvisionedIdentityRecord,
    /// Consumption time in Unix milliseconds.
    pub consumed_at: i64,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic terminal activation-review decision.
#[derive(Clone, Debug)]
pub struct ActivationReviewDecision {
    /// Review ID to decide.
    pub review_id: String,
    /// Expected pending review version.
    pub expected_version: u64,
    /// Approved or rejected terminal state.
    pub state: DeviceActivationReviewState,
    /// Decision time in Unix milliseconds.
    pub decided_at: i64,
    /// Stable deciding administrator.
    pub decided_by: String,
    /// Optional operator reason.
    pub reason: Option<String>,
    /// Optional approved delegation replacement; absent on rejection.
    pub delegation: Option<DeviceDelegationRecord>,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic service identity provisioning.
#[derive(Clone, Debug)]
pub struct ServiceIdentityProvisioning {
    /// New service principal.
    pub principal: PrincipalRecord,
    /// New deployment-owned runtime instance.
    pub instance: RuntimeInstanceRecord,
    /// Immutable service identity metadata.
    pub identity: ProvisionedIdentityRecord,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic device and one-time secret provisioning.
#[derive(Clone, Debug)]
pub struct DeviceProvisioning {
    /// New device principal.
    pub principal: PrincipalRecord,
    /// New deployment-owned runtime instance.
    pub instance: RuntimeInstanceRecord,
    /// New deployment-scoped device record.
    pub device: DeviceRecord,
    /// Optional identity installed immediately instead of returning a secret.
    pub identity: Option<ProvisionedIdentityRecord>,
    /// Hashed one-time provisioning secret.
    pub secret: DeviceProvisioningSecretRecord,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic service/device instance lifecycle mutation.
#[derive(Clone, Debug)]
pub struct ProvisionedInstanceMutation {
    /// Replacement runtime instance.
    pub instance: RuntimeInstanceRecord,
    /// Optional replacement device lifecycle record.
    pub device: Option<DeviceRecord>,
    /// Optional immutable identity whose state follows the lifecycle.
    pub identity: Option<ProvisionedIdentityRecord>,
    /// Expected API-visible lifecycle version.
    pub expected_version: u64,
    /// Durable replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic external-identity unlink command.
#[derive(Clone, Debug)]
pub struct ProviderIdentityUnlink {
    /// Provider key.
    pub provider: String,
    /// Provider-owned stable subject.
    pub provider_subject: String,
    /// Principal that must own the link.
    pub principal_id: String,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic authenticated password change and sibling-session revocation.
#[derive(Clone, Debug)]
pub struct PasswordChange {
    /// User principal owning the credential.
    pub principal_id: String,
    /// Session that remains active after the password change.
    pub current_session_id: String,
    /// Replacement Argon2id credential.
    pub credential: LocalCredentialRecord,
    /// Expected credential version.
    pub expected_version: u64,
    /// Password-change time in Unix milliseconds.
    pub changed_at: i64,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic device delegation revocation.
#[derive(Clone, Debug)]
pub struct DeviceDelegationMutation {
    /// Replacement device lifecycle record.
    pub device: DeviceRecord,
    /// Replacement delegation record.
    pub delegation: DeviceDelegationRecord,
    /// Expected device version.
    pub expected_version: u64,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic authenticated-session creation.
#[derive(Clone, Debug)]
pub struct SessionCreation {
    /// New validated session record.
    pub session: SessionRecord,
    /// Optional exact identity authority for a user browser bind.
    pub desired_authority: Option<DesiredAuthorityRecord>,
    /// Required exact runtime binding for service and device sessions.
    pub runtime_binding: Option<SessionRuntimeBinding>,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic authenticated-session revocation.
#[derive(Clone, Debug)]
pub struct SessionRevocation {
    /// Stable session ID to revoke.
    pub session_id: String,
    /// Expected active session version.
    pub expected_version: u64,
    /// Revocation time in Unix milliseconds.
    pub revoked_at: i64,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic event and connection-kick actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic admission of one retry-safe client bootstrap.
#[derive(Clone, Debug)]
pub struct ClientBootstrapAdmission {
    /// Session authenticated by the bootstrap proof.
    pub session_id: String,
    /// Observation time used to coalesce the session touch.
    pub observed_at: i64,
    /// Durable proof claim and stable admission identity; credentials are never replayed.
    pub idempotency: IdempotencyResultRecord,
}

/// Atomic user-account creation with optional local-login records.
#[derive(Clone, Debug)]
pub struct AccountCreation {
    /// New user principal.
    pub principal: PrincipalRecord,
    /// New user profile.
    pub profile: UserProfileRecord,
    /// Optional local credential; requires a matching local `identity`.
    pub credential: Option<LocalCredentialRecord>,
    /// Optional local or federated provider identity.
    pub identity: Option<ProviderIdentityLink>,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic user principal and profile replacement.
#[derive(Clone, Debug)]
pub struct UserAccountMutation {
    /// Complete replacement principal.
    pub principal: PrincipalRecord,
    /// Complete replacement profile.
    pub profile: UserProfileRecord,
    /// Expected current version shared by the principal and profile.
    pub expected_version: u64,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic account-flow creation.
#[derive(Clone, Debug)]
pub struct AccountFlowCreation {
    /// New pending account flow.
    pub flow: AccountFlowRecord,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic authority-proposal creation.
#[derive(Clone, Debug)]
pub struct AuthorityProposalCreation {
    /// New immutable pending proposal.
    pub proposal: AuthorityProposalRecord,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic activation-review creation.
#[derive(Clone, Debug)]
pub struct ActivationReviewCreation {
    /// New pending activation review.
    pub review: DeviceActivationReviewRecord,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic login-portal and settings mutation.
#[derive(Clone, Debug)]
pub struct LoginPortalMutation {
    /// Portal record to create or replace.
    pub portal: LoginPortalRecord,
    /// Settings record to create or replace with the portal.
    pub settings: LoginSettingsRecord,
    /// Expected current version, or `None` for creation.
    pub expected_version: Option<u64>,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic portal-route mutation.
#[derive(Clone, Debug)]
pub struct PortalRouteMutation {
    /// Route record to create or replace.
    pub route: PortalRouteRecord,
    /// Expected current version, or `None` for creation.
    pub expected_version: Option<u64>,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Atomic portal-route removal.
#[derive(Clone, Debug)]
pub struct PortalRouteRemoval {
    /// Stable route ID to remove.
    pub route_id: String,
    /// Expected current route version.
    pub expected_version: u64,
    /// Durable proof claim and replay result.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

/// Authoritative aggregate repository for session creation and revocation.
#[async_trait]
pub trait AuthSessionRepository: Send + Sync {
    /// Create a session with any desired authority or runtime binding atomically.
    async fn create_session(
        &self,
        command: SessionCreation,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError>;

    /// Revoke one active session with proof result and actions atomically.
    async fn revoke_session(
        &self,
        command: SessionRevocation,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError>;

    /// Atomically admit one client bootstrap and record its replayable result.
    async fn admit_client_bootstrap(
        &self,
        command: ClientBootstrapAdmission,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError>;
}

#[async_trait]
impl AuthSessionRepository for InMemoryAuthorizationStore {
    async fn create_session(
        &self,
        command: SessionCreation,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        let mut staged = state.clone();
        insert_memory_session(&mut staged, command.session.clone())?;
        match command.session.principal_kind {
            PrincipalKind::User => {
                if command.runtime_binding.is_some() {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "user sessions cannot have runtime bindings".to_owned(),
                    ));
                }
                if let Some(desired) = command.desired_authority {
                    validate_session_desired_authority(&command.session, &desired)?;
                    put_memory_desired_authority(&mut staged, desired)?;
                }
            }
            PrincipalKind::Service | PrincipalKind::Device => {
                if command.desired_authority.is_some() {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "deployed sessions cannot put user desired authority".to_owned(),
                    ));
                }
                let binding = command.runtime_binding.ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord(
                        "deployed sessions require a runtime binding".to_owned(),
                    )
                })?;
                if binding.session_id != command.session.session_id {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "runtime binding does not identify the created session".to_owned(),
                    ));
                }
                validate_session_runtime_binding(&binding)?;
                validate_session_runtime_binding_relationships(&staged, &binding)?;
                staged
                    .session_runtime_bindings
                    .insert(binding.session_id.clone(), binding);
            }
        }
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.session))
    }

    async fn revoke_session(
        &self,
        command: SessionRevocation,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        require_protocol_timestamp("revokedAt", command.revoked_at)?;
        validate_session_revocation_actions(&command.actions)?;
        let mut staged = state.clone();
        let session = staged
            .sessions
            .get_mut(&command.session_id)
            .ok_or(AuthorizationStateError::SessionMissing)?;
        if session.version != command.expected_version
            || session.state != SessionState::Active
            || command.revoked_at < session.created_at
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        session.state = SessionState::Revoked;
        session.revoked_at = Some(command.revoked_at);
        session.version = next_version(session.version)?;
        let result = session.clone();
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(result))
    }

    async fn admit_client_bootstrap(
        &self,
        command: ClientBootstrapAdmission,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        require_protocol_timestamp("observedAt", command.observed_at)?;
        let mut staged = state.clone();
        let session = staged
            .sessions
            .get_mut(&command.session_id)
            .ok_or(AuthorizationStateError::SessionMissing)?;
        if session.state != SessionState::Active
            || session.principal_kind != PrincipalKind::User
            || command.observed_at < session.created_at
            || session
                .expires_at
                .is_some_and(|expires_at| command.observed_at >= expires_at)
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        session.last_seen_at = session.last_seen_at.max(command.observed_at);
        let result = session.clone();
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, Vec::new())?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(result))
    }
}

/// Aggregate repository for user-account companion records.
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Atomically create a user principal and profile, optionally with local-login records.
    async fn create_user_account(
        &self,
        command: AccountCreation,
    ) -> Result<IdempotentOutcome<UserProfileRecord>, AuthorizationStateError>;

    /// Load one user principal and its required profile.
    async fn get_user_account(
        &self,
        principal_id: &str,
    ) -> Result<Option<(PrincipalRecord, UserProfileRecord)>, AuthorizationStateError>;

    /// List user accounts by principal ID after an optional exclusive cursor.
    async fn list_user_accounts(
        &self,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(PrincipalRecord, UserProfileRecord)>, AuthorizationStateError>;

    /// Atomically replace one user principal and profile using optimistic versioning.
    async fn update_user_account(
        &self,
        command: UserAccountMutation,
    ) -> Result<IdempotentOutcome<(PrincipalRecord, UserProfileRecord)>, AuthorizationStateError>;

    /// Load one user profile by principal ID.
    async fn get_user_profile(
        &self,
        principal_id: &str,
    ) -> Result<Option<UserProfileRecord>, AuthorizationStateError>;

    /// Load one local credential by principal ID.
    async fn get_local_credential(
        &self,
        principal_id: &str,
    ) -> Result<Option<LocalCredentialRecord>, AuthorizationStateError>;

    /// Atomically replace a password and revoke sibling sessions.
    async fn change_password(
        &self,
        command: PasswordChange,
    ) -> Result<IdempotentOutcome<usize>, AuthorizationStateError>;

    /// Load one local credential by canonical username.
    async fn get_local_credential_by_username(
        &self,
        normalized_username: &str,
    ) -> Result<Option<LocalCredentialRecord>, AuthorizationStateError>;

    /// Atomically record one successful or failed local login attempt.
    async fn record_local_login_attempt(
        &self,
        attempt: LocalLoginAttempt,
    ) -> Result<LocalCredentialRecord, AuthorizationStateError>;

    /// Return whether an active, accepted, unexpired administrator exists.
    async fn has_active_administrator(&self, now: i64) -> Result<bool, AuthorizationStateError>;

    /// Create the startup first-admin flow, or return the existing unexpired pending flow.
    async fn replace_first_admin_flow(
        &self,
        flow: AccountFlowRecord,
        now: i64,
        rotate: bool,
    ) -> Result<Option<AccountFlowRecord>, AuthorizationStateError>;
}

/// Aggregate repository for login portals, settings, and deterministic routes.
#[async_trait]
pub trait LoginPortalRepository: Send + Sync {
    /// List login portals in stable ID order.
    async fn list_login_portals(&self) -> Result<Vec<LoginPortalRecord>, AuthorizationStateError>;

    /// Load one portal and its required settings.
    async fn get_login_portal(
        &self,
        portal_id: &str,
    ) -> Result<Option<(LoginPortalRecord, LoginSettingsRecord)>, AuthorizationStateError>;

    /// Create or compare-and-swap a portal and settings atomically.
    async fn put_login_portal(
        &self,
        command: LoginPortalMutation,
    ) -> Result<IdempotentOutcome<(LoginPortalRecord, LoginSettingsRecord)>, AuthorizationStateError>;

    /// Create or compare-and-swap one portal route.
    async fn put_portal_route(
        &self,
        command: PortalRouteMutation,
    ) -> Result<IdempotentOutcome<PortalRouteRecord>, AuthorizationStateError>;

    /// Remove one route with an optimistic version guard.
    async fn remove_portal_route(
        &self,
        command: PortalRouteRemoval,
    ) -> Result<IdempotentOutcome<PortalRouteRecord>, AuthorizationStateError>;

    /// List routes in deterministic selection order.
    async fn list_portal_routes(&self) -> Result<Vec<PortalRouteRecord>, AuthorizationStateError>;
}

/// Repository for hashed, expiring, single-use account flows.
#[async_trait]
pub trait AccountFlowRepository: Send + Sync {
    /// Create one pending flow with a globally unique token hash.
    async fn create_account_flow(
        &self,
        command: AccountFlowCreation,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError>;

    /// Load a flow by its bearer-token digest.
    async fn get_account_flow_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<AccountFlowRecord>, AuthorizationStateError>;

    /// Complete a password reset atomically with proof result and actions.
    async fn complete_password_reset(
        &self,
        command: PasswordResetCompletion,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError>;

    /// Complete an identity link atomically with proof result and actions.
    async fn complete_identity_link(
        &self,
        command: IdentityLinkCompletion,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError>;

    /// Complete first-administrator creation atomically.
    async fn complete_first_admin(
        &self,
        command: FirstAdminCompletion,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError>;
}

/// Repository for immutable authority proposals and terminal decisions.
#[async_trait]
pub trait AuthorityProposalRepository: Send + Sync {
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

/// Repository for provisioned identities, one-time secrets, and activation reviews.
#[async_trait]
pub trait ProvisioningRepository: Send + Sync {
    /// List provisioned identities in stable key order.
    async fn list_provisioned_identities(
        &self,
    ) -> Result<Vec<ProvisionedIdentityRecord>, AuthorizationStateError>;

    /// Load provisioned identity metadata by key ID.
    async fn get_provisioned_identity(
        &self,
        identity_key_id: &str,
    ) -> Result<Option<ProvisionedIdentityRecord>, AuthorizationStateError>;

    /// Consume a device secret and create its immutable identity atomically.
    async fn consume_device_provisioning_secret(
        &self,
        command: DeviceProvisioningSecretConsumption,
    ) -> Result<IdempotentOutcome<DeviceProvisioningSecretRecord>, AuthorizationStateError>;

    /// Create a pending activation review after exact device relationship checks.
    async fn create_activation_review(
        &self,
        command: ActivationReviewCreation,
    ) -> Result<IdempotentOutcome<DeviceActivationReviewRecord>, AuthorizationStateError>;

    /// Load one activation review by stable ID.
    async fn get_activation_review(
        &self,
        review_id: &str,
    ) -> Result<Option<DeviceActivationReviewRecord>, AuthorizationStateError>;

    /// List device activation reviews in stable ID order.
    async fn list_activation_reviews(
        &self,
    ) -> Result<Vec<DeviceActivationReviewRecord>, AuthorizationStateError>;

    /// Decide a review and approved device state atomically.
    async fn decide_activation_review(
        &self,
        command: ActivationReviewDecision,
    ) -> Result<IdempotentOutcome<DeviceActivationReviewRecord>, AuthorizationStateError>;

    /// Provision a service principal, instance, identity, proof result, and actions atomically.
    async fn provision_service_identity(
        &self,
        command: ServiceIdentityProvisioning,
    ) -> Result<IdempotentOutcome<ProvisionedIdentityRecord>, AuthorizationStateError>;

    /// Provision a device principal, instance, record, secret, proof result, and actions atomically.
    async fn provision_device(
        &self,
        command: DeviceProvisioning,
    ) -> Result<IdempotentOutcome<DeviceProvisioningSecretRecord>, AuthorizationStateError>;

    /// Mutate one provisioned service/device lifecycle atomically.
    async fn mutate_provisioned_instance(
        &self,
        command: ProvisionedInstanceMutation,
    ) -> Result<IdempotentOutcome<RuntimeInstanceRecord>, AuthorizationStateError>;

    /// Atomically replace device delegation state and durable side effects.
    async fn mutate_device_delegation(
        &self,
        command: DeviceDelegationMutation,
    ) -> Result<IdempotentOutcome<DeviceRecord>, AuthorizationStateError>;
}

/// Repository for durable state-changing request results.
#[async_trait]
pub trait IdempotencyRepository: Send + Sync {
    /// Load one durable proof result by its authenticated request scope.
    async fn get_idempotency_result(
        &self,
        purpose: &str,
        signer_id: &str,
        request_id: &str,
    ) -> Result<Option<IdempotencyResultRecord>, AuthorizationStateError>;

    /// Atomically record or replay a completed deterministic operation.
    async fn record_idempotency_result(
        &self,
        record: IdempotencyResultRecord,
    ) -> Result<IdempotentOutcome<Value>, AuthorizationStateError>;
}

/// Retry-safe repository for ordinary post-commit actions.
#[async_trait]
pub trait PostCommitActionRepository: Send + Sync {
    /// List dispatchable actions by next-attempt time and action ID.
    async fn list_ready_post_commit_actions(
        &self,
        now: i64,
        limit: usize,
    ) -> Result<Vec<PostCommitActionRecord>, AuthorizationStateError>;

    /// Claim one ready action until the supplied protocol timestamp.
    async fn claim_post_commit_action(
        &self,
        action_id: &str,
        now: i64,
        claimed_until: i64,
    ) -> Result<Option<PostCommitActionRecord>, AuthorizationStateError>;

    /// Record a failed claimed attempt and schedule its retry.
    async fn fail_post_commit_action(
        &self,
        action_id: &str,
        expected_claimed_until: i64,
        next_attempt_at: i64,
        error: String,
    ) -> Result<PostCommitActionRecord, AuthorizationStateError>;

    /// Acknowledge and remove one action only for its current claim.
    async fn acknowledge_post_commit_action(
        &self,
        action_id: &str,
        expected_claimed_until: i64,
    ) -> Result<(), AuthorizationStateError>;
}

#[async_trait]
impl AccountRepository for InMemoryAuthorizationStore {
    async fn create_user_account(
        &self,
        command: AccountCreation,
    ) -> Result<IdempotentOutcome<UserProfileRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        let mut staged = state.clone();
        insert_memory_user_account(
            &mut staged,
            command.principal,
            command.profile.clone(),
            command.credential,
            command.identity,
        )?;
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.profile))
    }

    async fn get_user_account(
        &self,
        principal_id: &str,
    ) -> Result<Option<(PrincipalRecord, UserProfileRecord)>, AuthorizationStateError> {
        let state = self.state()?;
        Ok(state
            .principals
            .get(principal_id)
            .filter(|principal| principal.kind == PrincipalKind::User)
            .zip(state.user_profiles.get(principal_id))
            .map(|(principal, profile)| (principal.clone(), profile.clone())))
    }

    async fn list_user_accounts(
        &self,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(PrincipalRecord, UserProfileRecord)>, AuthorizationStateError> {
        validate_account_list(cursor, limit)?;
        let state = self.state()?;
        Ok(state
            .principals
            .iter()
            .filter(|(principal_id, principal)| {
                principal.kind == PrincipalKind::User
                    && cursor.is_none_or(|cursor| principal_id.as_str() > cursor)
            })
            .filter_map(|(principal_id, principal)| {
                state
                    .user_profiles
                    .get(principal_id)
                    .map(|profile| (principal.clone(), profile.clone()))
            })
            .take(limit)
            .collect())
    }

    async fn update_user_account(
        &self,
        command: UserAccountMutation,
    ) -> Result<IdempotentOutcome<(PrincipalRecord, UserProfileRecord)>, AuthorizationStateError>
    {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        let mut staged = state.clone();
        let current_principal = staged
            .principals
            .get(&command.principal.principal_id)
            .ok_or(AuthorizationStateError::PrincipalMissing)?;
        let current_profile = staged
            .user_profiles
            .get(&command.principal.principal_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let (principal, profile) = user_account_replacement(
            current_principal,
            current_profile,
            command.principal,
            command.profile,
            command.expected_version,
        )?;
        staged
            .principals
            .insert(principal.principal_id.clone(), principal.clone());
        staged
            .user_profiles
            .insert(profile.principal_id.clone(), profile.clone());
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied((principal, profile)))
    }

    async fn get_user_profile(
        &self,
        principal_id: &str,
    ) -> Result<Option<UserProfileRecord>, AuthorizationStateError> {
        Ok(self.state()?.user_profiles.get(principal_id).cloned())
    }

    async fn get_local_credential(
        &self,
        principal_id: &str,
    ) -> Result<Option<LocalCredentialRecord>, AuthorizationStateError> {
        Ok(self.state()?.local_credentials.get(principal_id).cloned())
    }

    async fn change_password(
        &self,
        mut command: PasswordChange,
    ) -> Result<IdempotentOutcome<usize>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        let mut staged = state.clone();
        let current = staged
            .local_credentials
            .get(&command.principal_id)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("local credential not found".to_owned())
            })?;
        validate_local_credential(&command.credential)?;
        if current.version != command.expected_version
            || command.credential.principal_id != command.principal_id
            || command.credential.version != command.expected_version + 1
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        staged
            .local_credentials
            .insert(command.principal_id.clone(), command.credential);
        let mut revoked = 0;
        for session in staged.sessions.values_mut() {
            if session.principal_id == command.principal_id
                && session.session_id != command.current_session_id
                && session.state == SessionState::Active
            {
                session.state = SessionState::Revoked;
                session.revoked_at = Some(command.changed_at);
                session.last_seen_at = command.changed_at;
                session.version = next_version(session.version)?;
                revoked += 1;
            }
        }
        command.idempotency.result = json!({
            "changedAt": command.changed_at,
            "revokedSessionCount": revoked,
        });
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(revoked))
    }

    async fn get_local_credential_by_username(
        &self,
        normalized_username: &str,
    ) -> Result<Option<LocalCredentialRecord>, AuthorizationStateError> {
        let state = self.state()?;
        Ok(state
            .local_usernames
            .get(normalized_username)
            .and_then(|principal_id| state.local_credentials.get(principal_id))
            .cloned())
    }

    async fn record_local_login_attempt(
        &self,
        attempt: LocalLoginAttempt,
    ) -> Result<LocalCredentialRecord, AuthorizationStateError> {
        let mut state = self.state()?;
        let current = state
            .local_credentials
            .get(&attempt.principal_id)
            .cloned()
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let next = local_login_attempt_result(&current, &attempt)?;
        state
            .local_credentials
            .insert(attempt.principal_id, next.clone());
        Ok(next)
    }

    async fn has_active_administrator(&self, now: i64) -> Result<bool, AuthorizationStateError> {
        require_protocol_timestamp("now", now)?;
        let state = self.state()?;
        Ok(has_active_administrator(&state, now))
    }

    async fn replace_first_admin_flow(
        &self,
        flow: AccountFlowRecord,
        now: i64,
        rotate: bool,
    ) -> Result<Option<AccountFlowRecord>, AuthorizationStateError> {
        validate_account_flow(&flow)?;
        require_protocol_timestamp("now", now)?;
        if flow.kind != AccountFlowKind::FirstAdmin
            || flow.created_at > now
            || flow.expires_at <= now
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "replacement first-admin flow must be pending and unexpired".to_owned(),
            ));
        }
        let mut state = self.state()?;
        if has_active_administrator(&state, now) {
            return Ok(None);
        }
        if state.account_flows.contains_key(&flow.flow_id)
            || state.account_flow_hashes.contains_key(&flow.token_hash)
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if let Some(existing) = state.account_flows.values().find(|existing| {
            existing.kind == AccountFlowKind::FirstAdmin
                && existing.state == AccountFlowState::Pending
                && existing.expires_at > now
        }) {
            if !rotate {
                return Ok(Some(existing.clone()));
            }
        }
        let mut staged = state.clone();
        if rotate {
            for existing in staged.account_flows.values_mut().filter(|existing| {
                existing.kind == AccountFlowKind::FirstAdmin
                    && existing.state == AccountFlowState::Pending
                    && existing.expires_at > now
            }) {
                existing.state = AccountFlowState::Revoked;
                existing.version = next_version(existing.version)?;
            }
        }
        for existing in staged.account_flows.values_mut().filter(|existing| {
            existing.kind == AccountFlowKind::FirstAdmin
                && existing.state == AccountFlowState::Pending
                && existing.expires_at <= now
        }) {
            existing.state = AccountFlowState::Expired;
            existing.version = next_version(existing.version)?;
        }
        staged
            .account_flow_hashes
            .insert(flow.token_hash.clone(), flow.flow_id.clone());
        staged
            .account_flows
            .insert(flow.flow_id.clone(), flow.clone());
        *state = staged;
        Ok(Some(flow))
    }
}

#[async_trait]
impl LoginPortalRepository for InMemoryAuthorizationStore {
    async fn list_login_portals(&self) -> Result<Vec<LoginPortalRecord>, AuthorizationStateError> {
        Ok(self.state()?.login_portals.values().cloned().collect())
    }

    async fn get_login_portal(
        &self,
        portal_id: &str,
    ) -> Result<Option<(LoginPortalRecord, LoginSettingsRecord)>, AuthorizationStateError> {
        let state = self.state()?;
        Ok(state
            .login_portals
            .get(portal_id)
            .cloned()
            .zip(state.login_settings.get(portal_id).cloned()))
    }

    async fn put_login_portal(
        &self,
        command: LoginPortalMutation,
    ) -> Result<IdempotentOutcome<(LoginPortalRecord, LoginSettingsRecord)>, AuthorizationStateError>
    {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_login_portal(&command.portal, &command.settings)?;
        let mut staged = state.clone();
        match (
            staged.login_portals.get(&command.portal.portal_id),
            command.expected_version,
        ) {
            (None, None) if command.portal.version == 1 && command.settings.version == 1 => {}
            (Some(current), Some(expected))
                if current.version == expected
                    && staged
                        .login_settings
                        .get(&command.portal.portal_id)
                        .is_some_and(|v| v.version == expected)
                    && current.builtin == command.portal.builtin
                    && current.created_at == command.portal.created_at
                    && command.portal.version == next_version(expected)?
                    && command.settings.version == command.portal.version => {}
            _ => return Err(AuthorizationStateError::StorageConflict),
        }
        staged
            .login_portals
            .insert(command.portal.portal_id.clone(), command.portal.clone());
        staged
            .login_settings
            .insert(command.settings.portal_id.clone(), command.settings.clone());
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied((
            command.portal,
            command.settings,
        )))
    }

    async fn put_portal_route(
        &self,
        command: PortalRouteMutation,
    ) -> Result<IdempotentOutcome<PortalRouteRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_portal_route(&command.route)?;
        let mut staged = state.clone();
        if !staged.login_portals.contains_key(&command.route.portal_id)
            || command
                .route
                .deployment_id
                .as_ref()
                .is_some_and(|id| !staged.deployments.contains_key(id))
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "portal route relationships do not exist".to_owned(),
            ));
        }
        match (
            staged.portal_routes.get(&command.route.route_id),
            command.expected_version,
        ) {
            (None, None) if command.route.version == 1 => {}
            (Some(current), Some(expected))
                if current.version == expected
                    && current.created_at == command.route.created_at
                    && command.route.version == next_version(expected)? => {}
            _ => return Err(AuthorizationStateError::StorageConflict),
        }
        staged
            .portal_routes
            .insert(command.route.route_id.clone(), command.route.clone());
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.route))
    }

    async fn remove_portal_route(
        &self,
        command: PortalRouteRemoval,
    ) -> Result<IdempotentOutcome<PortalRouteRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        require_nonempty("routeId", &command.route_id)?;
        require_positive("expectedVersion", command.expected_version)?;
        let mut staged = state.clone();
        let current = staged
            .portal_routes
            .get(&command.route_id)
            .cloned()
            .ok_or(AuthorizationStateError::StorageConflict)?;
        if current.version != command.expected_version {
            return Err(AuthorizationStateError::StorageConflict);
        }
        staged.portal_routes.remove(&command.route_id);
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(current))
    }

    async fn list_portal_routes(&self) -> Result<Vec<PortalRouteRecord>, AuthorizationStateError> {
        let mut routes = self
            .state()?
            .portal_routes
            .values()
            .cloned()
            .collect::<Vec<_>>();
        routes.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.route_id.cmp(&right.route_id))
        });
        Ok(routes)
    }
}

#[async_trait]
impl AccountFlowRepository for InMemoryAuthorizationStore {
    async fn create_account_flow(
        &self,
        command: AccountFlowCreation,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_account_flow(&command.flow)?;
        if command.flow.kind == AccountFlowKind::FirstAdmin {
            return Err(AuthorizationStateError::InvalidRecord(
                "first-admin flows must use replace_first_admin_flow".to_owned(),
            ));
        }
        let mut staged = state.clone();
        if staged.account_flows.contains_key(&command.flow.flow_id)
            || staged
                .account_flow_hashes
                .contains_key(&command.flow.token_hash)
            || command
                .flow
                .target_principal_id
                .as_ref()
                .is_some_and(|id| !staged.principals.contains_key(id))
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        staged.account_flow_hashes.insert(
            command.flow.token_hash.clone(),
            command.flow.flow_id.clone(),
        );
        staged
            .account_flows
            .insert(command.flow.flow_id.clone(), command.flow.clone());
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.flow))
    }

    async fn get_account_flow_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<AccountFlowRecord>, AuthorizationStateError> {
        let state = self.state()?;
        Ok(state
            .account_flow_hashes
            .get(token_hash)
            .and_then(|id| state.account_flows.get(id))
            .cloned())
    }

    async fn complete_password_reset(
        &self,
        command: PasswordResetCompletion,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        let mut staged = state.clone();
        let flow = pending_account_flow(
            &staged,
            &command.token_hash,
            command.expected_flow_version,
            AccountFlowKind::PasswordReset,
            command.consumed_at,
        )?;
        let principal_id = flow.target_principal_id.clone().ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(
                "password-reset flow has no target principal".to_owned(),
            )
        })?;
        let current = staged
            .local_credentials
            .get(&principal_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        validate_replacement_credential(current, &command.replacement, &principal_id)?;
        validate_session_revocation_actions(&command.actions)?;
        staged
            .local_credentials
            .insert(principal_id, command.replacement);
        for session in staged.sessions.values_mut().filter(|session| {
            session.principal_id == flow.target_principal_id.as_deref().unwrap_or_default()
                && session.state == SessionState::Active
        }) {
            session.state = SessionState::Revoked;
            session.revoked_at = Some(command.consumed_at);
            session.version = next_version(session.version)?;
        }
        let completed = consume_memory_flow(&mut staged, &flow.flow_id, command.consumed_at)?;
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(completed))
    }

    async fn complete_identity_link(
        &self,
        command: IdentityLinkCompletion,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_provider_identity(&command.identity)?;
        let mut staged = state.clone();
        let flow = pending_account_flow(
            &staged,
            &command.token_hash,
            command.expected_flow_version,
            AccountFlowKind::IdentityLink,
            command.consumed_at,
        )?;
        if flow.target_principal_id.as_deref() != Some(command.identity.principal_id.as_str())
            || flow.target_provider_id.as_deref() != Some(command.identity.provider.as_str())
            || staged
                .principals
                .get(&command.identity.principal_id)
                .is_none_or(|principal| principal.kind != PrincipalKind::User)
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "identity-link flow target does not match the supplied identity".to_owned(),
            ));
        }
        let key = (
            command.identity.provider.clone(),
            command.identity.provider_subject.clone(),
        );
        if staged.provider_identities.contains_key(&key) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        staged.provider_identities.insert(key, command.identity);
        let completed = consume_memory_flow(&mut staged, &flow.flow_id, command.consumed_at)?;
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(completed))
    }

    async fn complete_first_admin(
        &self,
        command: FirstAdminCompletion,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_new_user_account(
            &command.principal,
            &command.profile,
            command.credential.as_ref(),
            Some(&command.identity),
        )?;
        validate_first_admin_authority(
            &command.authority,
            &command.principal,
            command.consumed_at,
        )?;
        let mut staged = state.clone();
        let flow = pending_account_flow(
            &staged,
            &command.token_hash,
            command.expected_flow_version,
            AccountFlowKind::FirstAdmin,
            command.consumed_at,
        )?;
        if flow.target_principal_id.is_some()
            || flow.target_provider_id.is_some()
            || has_active_administrator(&staged, command.consumed_at)
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        insert_memory_user_account(
            &mut staged,
            command.principal.clone(),
            command.profile,
            command.credential,
            Some(command.identity),
        )?;
        put_memory_desired_authority(
            &mut staged,
            DesiredAuthorityRecord::Identity(command.authority),
        )?;
        let completed = consume_memory_flow(&mut staged, &flow.flow_id, command.consumed_at)?;
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(completed))
    }
}

#[async_trait]
impl AuthorityProposalRepository for InMemoryAuthorizationStore {
    async fn list_authority_proposals(
        &self,
    ) -> Result<
        Vec<(AuthorityProposalRecord, Option<AuthorityDecisionRecord>)>,
        AuthorizationStateError,
    > {
        let state = self.state()?;
        Ok(state
            .authority_proposals
            .values()
            .cloned()
            .map(|proposal| {
                let decision = state
                    .authority_decisions
                    .get(&proposal.proposal_id)
                    .cloned();
                (proposal, decision)
            })
            .collect())
    }

    async fn create_authority_proposal(
        &self,
        mut command: AuthorityProposalCreation,
    ) -> Result<IdempotentOutcome<AuthorityProposalRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        validate_authority_proposal(&command.proposal)?;
        let mut staged = state.clone();
        for existing in staged.authority_proposals.values_mut().filter(|existing| {
            existing.authority_kind == command.proposal.authority_kind
                && existing.authority_id == command.proposal.authority_id
                && existing.state == AuthorityProposalState::Pending
                && existing
                    .expires_at
                    .is_some_and(|expires_at| expires_at <= command.proposal.created_at)
        }) {
            existing.state = AuthorityProposalState::Expired;
            existing.version = next_version(existing.version)?;
        }
        if let Some(result) = memory_idempotency_replay(&staged, &command.idempotency)? {
            *state = staged;
            return Ok(IdempotentOutcome::Replayed(result));
        }
        if let Some(proposal_id) = staged
            .authority_proposals
            .values()
            .find(|existing| {
                existing.state == AuthorityProposalState::Pending
                    && existing.authority_kind == command.proposal.authority_kind
                    && existing.authority_id == command.proposal.authority_id
                    && existing.proposal_digest == command.proposal.proposal_digest
            })
            .map(|existing| existing.proposal_id.clone())
        {
            command.idempotency.result = serde_json::json!({ "proposalId": proposal_id });
            let result = command.idempotency.result.clone();
            memory_commit_idempotency_and_actions(&mut staged, command.idempotency, Vec::new())?;
            *state = staged;
            return Ok(IdempotentOutcome::Replayed(result));
        }
        let binding = staged.participant_bindings.get(&(
            command.proposal.participant_id.clone(),
            command.proposal.participant_artifact_digest.clone(),
        ));
        if staged
            .authority_proposals
            .contains_key(&command.proposal.proposal_id)
            || binding
                .is_none_or(|value| value.needs_digest != command.proposal.participant_needs_digest)
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        for existing in staged.authority_proposals.values_mut().filter(|existing| {
            existing.authority_kind == command.proposal.authority_kind
                && existing.authority_id == command.proposal.authority_id
                && existing.state == AuthorityProposalState::Pending
        }) {
            existing.state = AuthorityProposalState::Superseded;
            existing.superseded_at = Some(command.proposal.created_at);
            existing.version = next_version(existing.version)?;
        }
        staged.authority_proposals.insert(
            command.proposal.proposal_id.clone(),
            command.proposal.clone(),
        );
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.proposal))
    }

    async fn get_authority_proposal(
        &self,
        proposal_id: &str,
    ) -> Result<
        Option<(AuthorityProposalRecord, Option<AuthorityDecisionRecord>)>,
        AuthorizationStateError,
    > {
        let state = self.state()?;
        Ok(state
            .authority_proposals
            .get(proposal_id)
            .cloned()
            .map(|proposal| {
                let decision = state.authority_decisions.get(proposal_id).cloned();
                (proposal, decision)
            }))
    }

    async fn decide_authority_proposal(
        &self,
        command: AuthorityProposalDecision,
    ) -> Result<IdempotentOutcome<AuthorityProposalRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_authority_decision(&command.proposal_id, &command.decision)?;
        let mut staged = state.clone();
        if staged
            .authority_decisions
            .contains_key(&command.proposal_id)
            || staged
                .authority_decision_digests
                .contains_key(&command.decision.decision_digest)
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let current = staged
            .authority_proposals
            .get(&command.proposal_id)
            .cloned()
            .ok_or(AuthorizationStateError::StorageConflict)?;
        if current.version != command.expected_version
            || current.state != AuthorityProposalState::Pending
            || current
                .expires_at
                .is_some_and(|expires| command.decision.decided_at >= expires)
            || command.decision.decided_at < current.created_at
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if command.decision.outcome == AuthorityDecisionOutcome::Accepted {
            if let Some(expected_base_authority_version) = command.expected_base_authority_version {
                if expected_base_authority_version != proposal_base_authority_version(&current)? {
                    return Err(AuthorizationStateError::StorageConflict);
                }
            }
            let current_authority_version = match current.authority_kind {
                AuthorityKind::Identity => staged
                    .identity_authorities
                    .values()
                    .find(|authority| authority.authority_id == current.authority_id)
                    .map(|authority| authority.version),
                AuthorityKind::Deployment => staged
                    .deployment_authorities
                    .values()
                    .find(|authority| authority.authority_id == current.authority_id)
                    .map(|authority| authority.version),
            };
            if proposal_base_authority_version(&current)? != current_authority_version {
                return Err(AuthorizationStateError::StorageConflict);
            }
        }
        validate_proposal_desired_authority(
            &current,
            command.decision.outcome,
            command.desired_authority.as_ref(),
        )?;
        if let Some(deployment) = command.deployment {
            super::repository::validate_deployment_evidence(&deployment)?;
            if staged
                .deployments
                .get(&deployment.deployment_id)
                .is_some_and(|current| {
                    current.participant_id != deployment.participant_id
                        || current.participant_kind != deployment.participant_kind
                })
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            staged
                .deployments
                .insert(deployment.deployment_id.clone(), deployment);
        }
        if let Some(desired) = command.desired_authority {
            put_memory_desired_authority(&mut staged, desired)?;
        }
        for proposal in staged.authority_proposals.values_mut().filter(|proposal| {
            proposal.proposal_id != command.proposal_id
                && proposal.authority_kind == current.authority_kind
                && proposal.authority_id == current.authority_id
                && proposal.state == AuthorityProposalState::Pending
        }) {
            proposal.state = AuthorityProposalState::Superseded;
            proposal.superseded_at = Some(command.decision.decided_at);
            proposal.version = next_version(proposal.version)?;
        }
        let proposal = staged
            .authority_proposals
            .get_mut(&command.proposal_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        proposal.state = match command.decision.outcome {
            AuthorityDecisionOutcome::Accepted => AuthorityProposalState::Accepted,
            AuthorityDecisionOutcome::Rejected => AuthorityProposalState::Rejected,
        };
        proposal.version = next_version(proposal.version)?;
        let result = proposal.clone();
        staged.authority_decision_digests.insert(
            command.decision.decision_digest.clone(),
            command.proposal_id.clone(),
        );
        staged
            .authority_decisions
            .insert(command.proposal_id, command.decision);
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(result))
    }
}

#[async_trait]
impl ProvisioningRepository for InMemoryAuthorizationStore {
    async fn list_provisioned_identities(
        &self,
    ) -> Result<Vec<ProvisionedIdentityRecord>, AuthorizationStateError> {
        Ok(self
            .state()?
            .provisioned_identities
            .values()
            .cloned()
            .collect())
    }

    async fn get_provisioned_identity(
        &self,
        identity_key_id: &str,
    ) -> Result<Option<ProvisionedIdentityRecord>, AuthorizationStateError> {
        Ok(self
            .state()?
            .provisioned_identities
            .get(identity_key_id)
            .cloned())
    }

    async fn consume_device_provisioning_secret(
        &self,
        command: DeviceProvisioningSecretConsumption,
    ) -> Result<IdempotentOutcome<DeviceProvisioningSecretRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_provisioned_identity(&command.identity)?;
        let mut staged = state.clone();
        let secret_id = staged
            .provisioning_secret_hashes
            .get(&command.secret_hash)
            .cloned()
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let secret = staged
            .provisioning_secrets
            .get(&secret_id)
            .cloned()
            .ok_or(AuthorizationStateError::StorageConflict)?;
        if secret.version != command.expected_version
            || secret.state != ProvisioningSecretState::Pending
            || command.consumed_at < secret.created_at
            || command.consumed_at >= secret.expires_at
            || command.identity.instance_id != secret.instance_id
            || command.identity.kind != ProvisionedIdentityKind::Device
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        validate_identity_relationships(&staged, &command.identity)?;
        insert_memory_provisioned_identity(&mut staged, command.identity)?;
        let secret = staged
            .provisioning_secrets
            .get_mut(&secret_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        secret.state = ProvisioningSecretState::Consumed;
        secret.consumed_at = Some(command.consumed_at);
        secret.version = next_version(secret.version)?;
        let result = secret.clone();
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(result))
    }

    async fn create_activation_review(
        &self,
        command: ActivationReviewCreation,
    ) -> Result<IdempotentOutcome<DeviceActivationReviewRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_activation_review(&command.review)?;
        let mut staged = state.clone();
        let exact_device = staged.devices.get(&(
            command.review.principal_id.clone(),
            command.review.deployment_id.clone(),
        ));
        let exact_instance = staged.runtime_instances.get(&command.review.instance_id);
        if exact_device.is_none_or(|device| device.state == DeviceState::Revoked)
            || exact_instance.is_none_or(|instance| {
                instance.deployment_id != command.review.deployment_id
                    || instance.principal_id != command.review.principal_id
            })
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "activation review relationships do not match exactly".to_owned(),
            ));
        }
        if staged
            .activation_reviews
            .contains_key(&command.review.review_id)
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        staged
            .activation_reviews
            .insert(command.review.review_id.clone(), command.review.clone());
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.review))
    }

    async fn get_activation_review(
        &self,
        review_id: &str,
    ) -> Result<Option<DeviceActivationReviewRecord>, AuthorizationStateError> {
        Ok(self.state()?.activation_reviews.get(review_id).cloned())
    }

    async fn list_activation_reviews(
        &self,
    ) -> Result<Vec<DeviceActivationReviewRecord>, AuthorizationStateError> {
        Ok(self.state()?.activation_reviews.values().cloned().collect())
    }

    async fn decide_activation_review(
        &self,
        command: ActivationReviewDecision,
    ) -> Result<IdempotentOutcome<DeviceActivationReviewRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_activation_decision(command.state, command.decided_at, &command.decided_by)?;
        validate_activation_decision_changes(&command)?;
        let mut staged = state.clone();
        let current = staged
            .activation_reviews
            .get(&command.review_id)
            .cloned()
            .ok_or(AuthorizationStateError::StorageConflict)?;
        if current.version != command.expected_version
            || current.state != DeviceActivationReviewState::Pending
            || command.decided_at < current.requested_at
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if command.state == DeviceActivationReviewState::Approved {
            let device = staged
                .devices
                .get_mut(&(current.principal_id.clone(), current.deployment_id.clone()))
                .ok_or(AuthorizationStateError::StorageConflict)?;
            device.state = DeviceState::Active;
            device.updated_at = command.decided_at;
            device.version = next_version(device.version)?;
        }
        if let Some(delegation) = command.delegation {
            if delegation.principal_id != current.principal_id
                || delegation.deployment_id != current.deployment_id
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "activation decision delegation does not match review".to_owned(),
                ));
            }
            staged.device_delegations.insert(
                (
                    delegation.principal_id.clone(),
                    delegation.deployment_id.clone(),
                ),
                delegation,
            );
        }
        let review = staged
            .activation_reviews
            .get_mut(&command.review_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        review.state = command.state;
        review.decided_at = Some(command.decided_at);
        review.decided_by = Some(command.decided_by);
        review.reason = command.reason;
        review.version = next_version(review.version)?;
        let result = review.clone();
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(result))
    }

    async fn provision_service_identity(
        &self,
        command: ServiceIdentityProvisioning,
    ) -> Result<IdempotentOutcome<ProvisionedIdentityRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_provisioning_aggregate(
            &command.principal,
            &command.instance,
            ProvisionedIdentityKind::Service,
        )?;
        validate_provisioned_identity(&command.identity)?;
        let mut staged = state.clone();
        validate_new_runtime_relationships(
            &staged,
            &command.principal,
            &command.instance,
            ProvisionedIdentityKind::Service,
        )?;
        if command.identity.principal_id != command.principal.principal_id
            || command.identity.deployment_id != command.instance.deployment_id
            || command.identity.instance_id != command.instance.instance_id
            || command.identity.kind != ProvisionedIdentityKind::Service
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "service identity aggregate does not match exactly".to_owned(),
            ));
        }
        insert_memory_principal_and_instance(&mut staged, command.principal, command.instance)?;
        insert_memory_provisioned_identity(&mut staged, command.identity.clone())?;
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.identity))
    }

    async fn provision_device(
        &self,
        command: DeviceProvisioning,
    ) -> Result<IdempotentOutcome<DeviceProvisioningSecretRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_provisioning_aggregate(
            &command.principal,
            &command.instance,
            ProvisionedIdentityKind::Device,
        )?;
        validate_provisioning_secret(&command.secret)?;
        validate_device(&command.device)?;
        if let Some(identity) = &command.identity {
            validate_provisioned_identity(identity)?;
            if identity.principal_id != command.principal.principal_id
                || identity.deployment_id != command.instance.deployment_id
                || identity.instance_id != command.instance.instance_id
                || identity.kind != ProvisionedIdentityKind::Device
                || command.secret.state != ProvisioningSecretState::Consumed
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "immediate device identity does not match provisioning".to_owned(),
                ));
            }
        }
        let mut staged = state.clone();
        validate_new_runtime_relationships(
            &staged,
            &command.principal,
            &command.instance,
            ProvisionedIdentityKind::Device,
        )?;
        if command.device.principal_id != command.principal.principal_id
            || command.device.deployment_id != command.instance.deployment_id
            || command.secret.instance_id != command.instance.instance_id
            || command.device.state != DeviceState::Pending
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "device provisioning aggregate does not match exactly".to_owned(),
            ));
        }
        insert_memory_principal_and_instance(&mut staged, command.principal, command.instance)?;
        let device_key = (
            command.device.principal_id.clone(),
            command.device.deployment_id.clone(),
        );
        if staged.devices.insert(device_key, command.device).is_some()
            || staged
                .provisioning_secrets
                .contains_key(&command.secret.secret_id)
            || staged
                .provisioning_secret_hashes
                .contains_key(&command.secret.secret_hash)
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        staged.provisioning_secret_hashes.insert(
            command.secret.secret_hash.clone(),
            command.secret.secret_id.clone(),
        );
        staged
            .provisioning_secrets
            .insert(command.secret.secret_id.clone(), command.secret.clone());
        if let Some(identity) = command.identity {
            insert_memory_provisioned_identity(&mut staged, identity)?;
        }
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.secret))
    }

    async fn mutate_provisioned_instance(
        &self,
        command: ProvisionedInstanceMutation,
    ) -> Result<IdempotentOutcome<RuntimeInstanceRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_runtime_instance(&command.instance)?;
        let current = state
            .runtime_instances
            .get(&command.instance.instance_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let visible_version = command
            .device
            .as_ref()
            .and_then(|device| {
                state
                    .devices
                    .get(&(device.principal_id.clone(), device.deployment_id.clone()))
                    .map(|current| current.version)
            })
            .unwrap_or(current.version);
        if visible_version != command.expected_version
            || current.created_at != command.instance.created_at
            || current.deployment_id != command.instance.deployment_id
            || current.principal_id != command.instance.principal_id
            || command.instance.version != next_version(current.version)?
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let mut staged = state.clone();
        staged.runtime_instances.insert(
            command.instance.instance_id.clone(),
            command.instance.clone(),
        );
        let principal = staged
            .principals
            .get_mut(&command.instance.principal_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        principal.state = match command.instance.state {
            RuntimeInstanceState::Active => PrincipalState::Active,
            RuntimeInstanceState::Disabled | RuntimeInstanceState::Stale => {
                PrincipalState::Disabled
            }
            RuntimeInstanceState::Revoked => PrincipalState::Revoked,
        };
        principal.updated_at = command.instance.updated_at;
        principal.version = next_version(principal.version)?;
        principal.disabled_at =
            (principal.state == PrincipalState::Disabled).then_some(command.instance.updated_at);
        principal.revoked_at =
            (principal.state == PrincipalState::Revoked).then_some(command.instance.updated_at);
        if let Some(device) = command.device {
            validate_device(&device)?;
            if device.version != next_version(command.expected_version)? {
                return Err(AuthorizationStateError::StorageConflict);
            }
            staged.devices.insert(
                (device.principal_id.clone(), device.deployment_id.clone()),
                device,
            );
        }
        if let Some(identity) = command.identity {
            let current = staged
                .provisioned_identities
                .get(&identity.identity_key_id)
                .ok_or(AuthorizationStateError::StorageConflict)?;
            if current.identity_public_key != identity.identity_public_key
                || current.principal_id != identity.principal_id
                || current.deployment_id != identity.deployment_id
                || current.instance_id != identity.instance_id
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            staged
                .provisioned_identities
                .insert(identity.identity_key_id.clone(), identity);
        }
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.instance))
    }

    async fn mutate_device_delegation(
        &self,
        command: DeviceDelegationMutation,
    ) -> Result<IdempotentOutcome<DeviceRecord>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(result) = memory_idempotency_replay(&state, &command.idempotency)? {
            return Ok(IdempotentOutcome::Replayed(result));
        }
        validate_device(&command.device)?;
        validate_device_delegation(&command.delegation)?;
        let key = (
            command.device.principal_id.clone(),
            command.device.deployment_id.clone(),
        );
        let current = state
            .devices
            .get(&key)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let current_delegation = state
            .device_delegations
            .get(&key)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        if current.version != command.expected_version
            || command.device.version != next_version(current.version)?
            || command.device.created_at != current.created_at
            || command.delegation.principal_id != current_delegation.principal_id
            || command.delegation.deployment_id != current_delegation.deployment_id
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let mut staged = state.clone();
        staged.devices.insert(key.clone(), command.device.clone());
        staged.device_delegations.insert(key, command.delegation);
        memory_commit_idempotency_and_actions(&mut staged, command.idempotency, command.actions)?;
        *state = staged;
        Ok(IdempotentOutcome::Applied(command.device))
    }
}

#[async_trait]
impl IdempotencyRepository for InMemoryAuthorizationStore {
    async fn get_idempotency_result(
        &self,
        purpose: &str,
        signer_id: &str,
        request_id: &str,
    ) -> Result<Option<IdempotencyResultRecord>, AuthorizationStateError> {
        let state = self.state()?;
        let key = (
            purpose.to_owned(),
            signer_id.to_owned(),
            request_id.to_owned(),
        );
        Ok(state
            .idempotency_requests
            .get(&key)
            .and_then(|scope| state.idempotency_results.get(scope))
            .cloned())
    }

    async fn record_idempotency_result(
        &self,
        record: IdempotencyResultRecord,
    ) -> Result<IdempotentOutcome<Value>, AuthorizationStateError> {
        let mut state = self.state()?;
        if let Some(value) = memory_idempotency_replay(&state, &record)? {
            return Ok(IdempotentOutcome::Replayed(value));
        }
        let value = record.result.clone();
        memory_commit_idempotency_and_actions(&mut state, record, Vec::new())?;
        Ok(IdempotentOutcome::Applied(value))
    }
}

#[async_trait]
impl PostCommitActionRepository for InMemoryAuthorizationStore {
    async fn list_ready_post_commit_actions(
        &self,
        now: i64,
        limit: usize,
    ) -> Result<Vec<PostCommitActionRecord>, AuthorizationStateError> {
        require_protocol_timestamp("now", now)?;
        let mut actions = self
            .state()?
            .post_commit_actions
            .values()
            .filter(|action| {
                action.next_attempt_at <= now
                    && action.claimed_until.is_none_or(|until| until <= now)
            })
            .cloned()
            .collect::<Vec<_>>();
        actions.sort_by(|left, right| {
            left.next_attempt_at
                .cmp(&right.next_attempt_at)
                .then_with(|| left.action_id.cmp(&right.action_id))
        });
        actions.truncate(limit);
        Ok(actions)
    }

    async fn claim_post_commit_action(
        &self,
        action_id: &str,
        now: i64,
        claimed_until: i64,
    ) -> Result<Option<PostCommitActionRecord>, AuthorizationStateError> {
        require_protocol_timestamp("now", now)?;
        require_protocol_timestamp("claimedUntil", claimed_until)?;
        if claimed_until <= now {
            return Err(AuthorizationStateError::InvalidRecord(
                "claimedUntil must follow now".to_owned(),
            ));
        }
        let mut state = self.state()?;
        let Some(action) = state.post_commit_actions.get_mut(action_id) else {
            return Ok(None);
        };
        if action.claimed_until == Some(claimed_until) {
            return Ok(Some(action.clone()));
        }
        if action.next_attempt_at > now || action.claimed_until.is_some_and(|until| until > now) {
            return Ok(None);
        }
        if action.claimed_until.is_some() {
            action.attempts = action.attempts.checked_add(1).ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("attempts overflow".to_owned())
            })?;
        }
        action.claimed_until = Some(claimed_until);
        Ok(Some(action.clone()))
    }

    async fn fail_post_commit_action(
        &self,
        action_id: &str,
        expected_claimed_until: i64,
        next_attempt_at: i64,
        error: String,
    ) -> Result<PostCommitActionRecord, AuthorizationStateError> {
        require_protocol_timestamp("nextAttemptAt", next_attempt_at)?;
        require_nonempty("error", &error)?;
        let mut state = self.state()?;
        let action = state
            .post_commit_actions
            .get_mut(action_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        if action.claimed_until.is_none()
            && action.next_attempt_at == next_attempt_at
            && action.last_error.as_deref() == Some(error.as_str())
        {
            return Ok(action.clone());
        }
        if action.claimed_until != Some(expected_claimed_until) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        action.attempts = action.attempts.checked_add(1).ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("attempts overflow".to_owned())
        })?;
        action.next_attempt_at = next_attempt_at;
        action.claimed_until = None;
        action.last_error = Some(error);
        Ok(action.clone())
    }

    async fn acknowledge_post_commit_action(
        &self,
        action_id: &str,
        expected_claimed_until: i64,
    ) -> Result<(), AuthorizationStateError> {
        let mut state = self.state()?;
        let Some(action) = state.post_commit_actions.get(action_id) else {
            return Ok(());
        };
        if action.claimed_until != Some(expected_claimed_until) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        state.post_commit_actions.remove(action_id);
        Ok(())
    }
}

fn insert_memory_session(
    state: &mut super::repository::MemoryState,
    session: SessionRecord,
) -> Result<(), AuthorizationStateError> {
    validate_session(&session)?;
    let principal = state
        .principals
        .get(&session.principal_id)
        .ok_or(AuthorizationStateError::PrincipalMissing)?;
    if principal.kind != session.principal_kind {
        return Err(AuthorizationStateError::InvalidRecord(
            "session principal kind does not match principal".to_owned(),
        ));
    }
    let participant = state
        .participant_bindings
        .get(&(
            session.participant_id.clone(),
            session.participant_artifact_digest.clone(),
        ))
        .ok_or(AuthorizationStateError::ParticipantMissing)?;
    if participant.participant_kind != session.participant_kind {
        return Err(AuthorizationStateError::InvalidRecord(
            "session participant kind does not match participant binding".to_owned(),
        ));
    }
    if participant.needs_digest != session.participant_needs_digest {
        return Err(AuthorizationStateError::NeedsDigestMismatch);
    }
    if state.sessions.contains_key(&session.session_id)
        || state.session_key_ids.contains_key(&session.session_key_id)
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    state
        .session_key_ids
        .insert(session.session_key_id.clone(), session.session_id.clone());
    state.sessions.insert(session.session_id.clone(), session);
    Ok(())
}

pub(super) fn validate_session_desired_authority(
    session: &SessionRecord,
    desired: &DesiredAuthorityRecord,
) -> Result<(), AuthorizationStateError> {
    let DesiredAuthorityRecord::Identity(authority) = desired else {
        return Err(AuthorizationStateError::InvalidRecord(
            "user session desired authority must be identity kind".to_owned(),
        ));
    };
    if session.principal_kind != PrincipalKind::User
        || authority.principal_id != session.principal_id
        || authority.participant_id != session.participant_id
        || authority.participant_artifact_digest != session.participant_artifact_digest
        || authority.accepted_needs_digest != session.participant_needs_digest
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "session desired authority does not match the session exactly".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_session_revocation_actions(
    actions: &[PostCommitActionRecord],
) -> Result<(), AuthorizationStateError> {
    if !actions
        .iter()
        .any(|action| action.kind == PostCommitActionKind::Event)
        || !actions
            .iter()
            .any(|action| action.kind == PostCommitActionKind::Kick)
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "session revocation requires deterministic event and kick actions".to_owned(),
        ));
    }
    Ok(())
}

fn has_active_administrator(state: &super::repository::MemoryState, now: i64) -> bool {
    state.identity_authorities.values().any(|authority| {
        authority.state == AuthorityState::Accepted
            && authority
                .expires_at
                .is_none_or(|expires_at| expires_at > now)
            && authority
                .desired_capabilities
                .iter()
                .any(|value| value == "admin")
            && state
                .principals
                .get(&authority.principal_id)
                .is_some_and(|principal| {
                    principal.kind == PrincipalKind::User
                        && principal.state == PrincipalState::Active
                })
    })
}

pub(super) fn memory_idempotency_replay(
    state: &super::repository::MemoryState,
    input: &IdempotencyResultRecord,
) -> Result<Option<Value>, AuthorizationStateError> {
    validate_idempotency_result(input)?;
    let request = (
        input.purpose.clone(),
        input.signer_id.clone(),
        input.request_id.clone(),
    );
    let Some(scope) = state.idempotency_requests.get(&request) else {
        if state.idempotency_results.contains_key(&input.scope_key) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        return Ok(None);
    };
    let existing = state.idempotency_results.get(scope).ok_or_else(|| {
        AuthorizationStateError::Storage("idempotency index is inconsistent".to_owned())
    })?;
    if existing.request_digest != input.request_digest {
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(Some(existing.result.clone()))
}

pub(super) fn memory_commit_idempotency_and_actions(
    state: &mut super::repository::MemoryState,
    idempotency: IdempotencyResultRecord,
    actions: Vec<PostCommitActionRecord>,
) -> Result<(), AuthorizationStateError> {
    if memory_idempotency_replay(state, &idempotency)?.is_some() {
        return Err(AuthorizationStateError::StorageConflict);
    }
    for (index, action) in actions.iter().enumerate() {
        validate_post_commit_action(action)?;
        if actions[..index].iter().any(|existing| {
            existing.action_id == action.action_id
                && !post_commit_action_identity_equal(existing, action)
        }) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if state
            .post_commit_actions
            .get(&action.action_id)
            .is_some_and(|existing| !post_commit_action_identity_equal(existing, action))
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
    }
    let request = (
        idempotency.purpose.clone(),
        idempotency.signer_id.clone(),
        idempotency.request_id.clone(),
    );
    state
        .idempotency_requests
        .insert(request, idempotency.scope_key.clone());
    state
        .idempotency_results
        .insert(idempotency.scope_key.clone(), idempotency);
    for action in actions {
        state
            .post_commit_actions
            .entry(action.action_id.clone())
            .or_insert(action);
    }
    Ok(())
}

fn pending_account_flow(
    state: &super::repository::MemoryState,
    token_hash: &str,
    expected_version: u64,
    kind: AccountFlowKind,
    consumed_at: i64,
) -> Result<AccountFlowRecord, AuthorizationStateError> {
    require_digest("tokenHash", token_hash)?;
    require_protocol_timestamp("consumedAt", consumed_at)?;
    let flow = state
        .account_flow_hashes
        .get(token_hash)
        .and_then(|id| state.account_flows.get(id))
        .cloned()
        .ok_or(AuthorizationStateError::StorageConflict)?;
    if flow.kind != kind
        || flow.version != expected_version
        || flow.state != AccountFlowState::Pending
        || consumed_at < flow.created_at
        || consumed_at >= flow.expires_at
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(flow)
}

fn consume_memory_flow(
    state: &mut super::repository::MemoryState,
    flow_id: &str,
    consumed_at: i64,
) -> Result<AccountFlowRecord, AuthorizationStateError> {
    let flow = state
        .account_flows
        .get_mut(flow_id)
        .ok_or(AuthorizationStateError::StorageConflict)?;
    flow.state = AccountFlowState::Consumed;
    flow.consumed_at = Some(consumed_at);
    flow.version = next_version(flow.version)?;
    Ok(flow.clone())
}

pub(super) fn validate_replacement_credential(
    current: &LocalCredentialRecord,
    replacement: &LocalCredentialRecord,
    principal_id: &str,
) -> Result<(), AuthorizationStateError> {
    validate_local_credential(replacement)?;
    if replacement.principal_id != principal_id
        || current.principal_id != principal_id
        || replacement.normalized_username != current.normalized_username
        || replacement.version != next_version(current.version)?
        || replacement.updated_at < current.updated_at
        || replacement.password_changed_at < current.password_changed_at
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(())
}

pub(super) fn local_login_attempt_result(
    current: &LocalCredentialRecord,
    attempt: &LocalLoginAttempt,
) -> Result<LocalCredentialRecord, AuthorizationStateError> {
    require_nonempty("principalId", &attempt.principal_id)?;
    require_protocol_timestamp("attemptedAt", attempt.attempted_at)?;
    require_positive("maximumFailures", u64::from(attempt.maximum_failures))?;
    require_positive("lockDurationMs", attempt.lock_duration_ms)?;
    if current.principal_id != attempt.principal_id || current.version != attempt.expected_version {
        return Err(AuthorizationStateError::StorageConflict);
    }
    if attempt.succeeded && current.failed_attempts == 0 && current.locked_until.is_none() {
        return Ok(current.clone());
    }
    let mut next = current.clone();
    next.version = next_version(current.version)?;
    next.updated_at = attempt.attempted_at;
    if attempt.succeeded {
        next.failed_attempts = 0;
        next.locked_until = None;
    } else {
        next.failed_attempts = current.failed_attempts.checked_add(1).ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("failedAttempts overflow".to_owned())
        })?;
        if next.failed_attempts >= attempt.maximum_failures {
            let locked_until = u64::try_from(attempt.attempted_at)
                .ok()
                .and_then(|at| at.checked_add(attempt.lock_duration_ms))
                .filter(|until| *until <= MAX_PROTOCOL_INTEGER)
                .ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord("lockedUntil overflow".to_owned())
                })?;
            next.locked_until = Some(locked_until as i64);
        }
    }
    validate_local_credential(&next)?;
    Ok(next)
}

fn insert_memory_user_account(
    state: &mut super::repository::MemoryState,
    principal: PrincipalRecord,
    profile: UserProfileRecord,
    credential: Option<LocalCredentialRecord>,
    identity: Option<ProviderIdentityLink>,
) -> Result<(), AuthorizationStateError> {
    validate_new_user_account(&principal, &profile, credential.as_ref(), identity.as_ref())?;
    if state.principals.contains_key(&principal.principal_id)
        || credential.as_ref().is_some_and(|credential| {
            state
                .local_usernames
                .contains_key(&credential.normalized_username)
        })
        || identity.as_ref().is_some_and(|identity| {
            state
                .provider_identities
                .contains_key(&(identity.provider.clone(), identity.provider_subject.clone()))
        })
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    state
        .user_profiles
        .insert(profile.principal_id.clone(), profile);
    if let Some(credential) = credential {
        state.local_usernames.insert(
            credential.normalized_username.clone(),
            principal.principal_id.clone(),
        );
        state
            .local_credentials
            .insert(credential.principal_id.clone(), credential);
    }
    if let Some(identity) = identity {
        state.provider_identities.insert(
            (identity.provider.clone(), identity.provider_subject.clone()),
            identity,
        );
    }
    state
        .principals
        .insert(principal.principal_id.clone(), principal);
    Ok(())
}

pub(super) fn validate_first_admin_authority(
    authority: &IdentityAuthorityRecord,
    principal: &PrincipalRecord,
    completed_at: i64,
) -> Result<(), AuthorizationStateError> {
    let mut validated = authority.clone();
    validate_identity_authority(&mut validated)?;
    if validated != *authority
        || authority.principal_id != principal.principal_id
        || authority.state != AuthorityState::Accepted
        || !authority
            .desired_capabilities
            .iter()
            .any(|value| value == "admin")
        || authority
            .expires_at
            .is_some_and(|expires_at| expires_at <= completed_at)
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "first-admin authority must be accepted, exact, active, and administrative".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_proposal_desired_authority(
    proposal: &AuthorityProposalRecord,
    outcome: AuthorityDecisionOutcome,
    desired: Option<&DesiredAuthorityRecord>,
) -> Result<(), AuthorizationStateError> {
    match (outcome, desired) {
        (AuthorityDecisionOutcome::Rejected, None) => return Ok(()),
        (AuthorityDecisionOutcome::Accepted, Some(desired)) => {
            let (
                kind,
                authority_id,
                participant_id,
                artifact_digest,
                needs_digest,
                grants,
                capabilities,
                state,
            ) = match desired {
                DesiredAuthorityRecord::Identity(record) => (
                    AuthorityKind::Identity,
                    &record.authority_id,
                    &record.participant_id,
                    &record.participant_artifact_digest,
                    &record.accepted_needs_digest,
                    &record.desired_grant_set,
                    &record.desired_capabilities,
                    record.state,
                ),
                DesiredAuthorityRecord::Deployment(record) => (
                    AuthorityKind::Deployment,
                    &record.authority_id,
                    &record.participant_id,
                    &record.participant_artifact_digest,
                    &record.accepted_needs_digest,
                    &record.desired_grant_set,
                    &record.desired_capabilities,
                    record.state,
                ),
            };
            if proposal.authority_kind == kind
                && proposal.authority_id == *authority_id
                && proposal.participant_id == *participant_id
                && proposal.participant_artifact_digest == *artifact_digest
                && proposal.participant_needs_digest == *needs_digest
                && proposal.proposed_grant_set == *grants
                && proposal.proposed_capabilities == *capabilities
                && state == AuthorityState::Accepted
            {
                return Ok(());
            }
        }
        _ => {}
    }
    Err(AuthorizationStateError::InvalidRecord(
        "proposal outcome and desired authority do not match exactly".to_owned(),
    ))
}

fn put_memory_desired_authority(
    state: &mut super::repository::MemoryState,
    desired: DesiredAuthorityRecord,
) -> Result<(), AuthorizationStateError> {
    match desired {
        DesiredAuthorityRecord::Identity(mut record) => {
            validate_identity_authority(&mut record)?;
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
            match state.identity_authorities.get(&key) {
                None if record.version == 1 => {}
                Some(current) if current.authority_id == record.authority_id => {
                    if identity_enforceability_equal(current, &record) {
                        record.version = current.version;
                    } else if record.version != next_version(current.version)? {
                        return Err(AuthorizationStateError::StorageConflict);
                    }
                }
                _ => return Err(AuthorizationStateError::StorageConflict),
            }
            state.identity_authorities.insert(key, record);
        }
        DesiredAuthorityRecord::Deployment(mut record) => {
            validate_deployment_authority(&mut record)?;
            if state
                .deployments
                .get(&record.deployment_id)
                .is_none_or(|deployment| {
                    deployment.participant_id != record.participant_id
                        || deployment.participant_kind != record.participant_kind
                })
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "deployment authority target does not match its deployment".to_owned(),
                ));
            }
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
            match state.deployment_authorities.get(&key) {
                None if record.version == 1 => {}
                Some(current) if current.authority_id == record.authority_id => {
                    if deployment_enforceability_equal(current, &record) {
                        record.version = current.version;
                    } else if record.version != next_version(current.version)? {
                        return Err(AuthorizationStateError::StorageConflict);
                    }
                }
                _ => return Err(AuthorizationStateError::StorageConflict),
            }
            state.deployment_authorities.insert(key, record);
        }
    }
    Ok(())
}

fn insert_memory_provisioned_identity(
    state: &mut super::repository::MemoryState,
    identity: ProvisionedIdentityRecord,
) -> Result<(), AuthorizationStateError> {
    if state
        .provisioned_identities
        .contains_key(&identity.identity_key_id)
        || state
            .provisioned_public_keys
            .contains_key(&identity.identity_public_key)
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    state.provisioned_public_keys.insert(
        identity.identity_public_key.clone(),
        identity.identity_key_id.clone(),
    );
    state
        .provisioned_identities
        .insert(identity.identity_key_id.clone(), identity);
    Ok(())
}

pub(super) fn validate_provisioning_aggregate(
    principal: &PrincipalRecord,
    instance: &RuntimeInstanceRecord,
    kind: ProvisionedIdentityKind,
) -> Result<(), AuthorizationStateError> {
    validate_principal(principal)?;
    validate_runtime_instance(instance)?;
    let principal_kind = match kind {
        ProvisionedIdentityKind::Service => PrincipalKind::Service,
        ProvisionedIdentityKind::Device => PrincipalKind::Device,
    };
    if principal.kind != principal_kind
        || principal.state != PrincipalState::Active
        || instance.principal_id != principal.principal_id
        || instance.state != RuntimeInstanceState::Active
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "provisioned principal and instance do not match exactly".to_owned(),
        ));
    }
    Ok(())
}

fn validate_new_runtime_relationships(
    state: &super::repository::MemoryState,
    principal: &PrincipalRecord,
    instance: &RuntimeInstanceRecord,
    kind: ProvisionedIdentityKind,
) -> Result<(), AuthorizationStateError> {
    let participant_kind = match kind {
        ProvisionedIdentityKind::Service => ParticipantKindV1::Service,
        ProvisionedIdentityKind::Device => ParticipantKindV1::Device,
    };
    if state
        .deployments
        .get(&instance.deployment_id)
        .is_none_or(|deployment| deployment.participant_kind != participant_kind)
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "provisioned instance deployment kind does not match".to_owned(),
        ));
    }
    if state.principals.contains_key(&principal.principal_id)
        || state.runtime_instances.contains_key(&instance.instance_id)
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(())
}

fn insert_memory_principal_and_instance(
    state: &mut super::repository::MemoryState,
    principal: PrincipalRecord,
    instance: RuntimeInstanceRecord,
) -> Result<(), AuthorizationStateError> {
    if state
        .principals
        .insert(principal.principal_id.clone(), principal)
        .is_some()
        || state
            .runtime_instances
            .insert(instance.instance_id.clone(), instance)
            .is_some()
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(())
}

pub(super) fn validate_activation_decision_changes(
    command: &ActivationReviewDecision,
) -> Result<(), AuthorizationStateError> {
    match command.state {
        DeviceActivationReviewState::Approved => {}
        DeviceActivationReviewState::Rejected if command.delegation.is_none() => return Ok(()),
        _ => {
            return Err(AuthorizationStateError::InvalidRecord(
                "activation rejection forbids delegation changes".to_owned(),
            ))
        }
    }
    if let Some(delegation) = &command.delegation {
        validate_device_delegation(delegation)?;
    }
    Ok(())
}

pub(super) fn validate_new_user_account(
    principal: &PrincipalRecord,
    profile: &UserProfileRecord,
    credential: Option<&LocalCredentialRecord>,
    identity: Option<&ProviderIdentityLink>,
) -> Result<(), AuthorizationStateError> {
    validate_principal(principal)?;
    if principal.kind != PrincipalKind::User || profile.principal_id != principal.principal_id {
        return Err(AuthorizationStateError::InvalidRecord(
            "account records do not identify one user principal".to_owned(),
        ));
    }
    if let Some(display_name) = &profile.display_name {
        require_nonempty("displayName", display_name)?;
    }
    require_protocol_timestamp("profile.createdAt", profile.created_at)?;
    require_protocol_timestamp("profile.updatedAt", profile.updated_at)?;
    if profile.version != 1 || profile.updated_at < profile.created_at {
        return Err(AuthorizationStateError::InvalidRecord(
            "new account profile version or timestamps are invalid".to_owned(),
        ));
    }
    match (credential, identity) {
        (None, None) => {}
        (Some(credential), Some(identity)) => {
            validate_provider_identity(identity)?;
            validate_local_credential(credential)?;
            if credential.principal_id != principal.principal_id
                || identity.principal_id != principal.principal_id
                || identity.provider != "local"
                || identity.provider_subject != credential.normalized_username
                || credential.version != 1
                || credential.updated_at < credential.password_changed_at
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "local credential and identity do not match the user account".to_owned(),
                ));
            }
        }
        (None, Some(identity)) => {
            validate_provider_identity(identity)?;
            if identity.principal_id != principal.principal_id || identity.provider == "local" {
                return Err(AuthorizationStateError::InvalidRecord(
                    "federated identity does not match the user account".to_owned(),
                ));
            }
        }
        (Some(_), None) => {
            return Err(AuthorizationStateError::InvalidRecord(
                "local credential requires a matching local identity".to_owned(),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_account_list(
    cursor: Option<&str>,
    limit: usize,
) -> Result<(), AuthorizationStateError> {
    if let Some(cursor) = cursor {
        require_nonempty("cursor", cursor)?;
    }
    if limit > 100 {
        return Err(AuthorizationStateError::InvalidRecord(
            "user account list limit exceeds 100".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn user_account_replacement(
    current_principal: &PrincipalRecord,
    current_profile: &UserProfileRecord,
    mut principal: PrincipalRecord,
    profile: UserProfileRecord,
    expected_version: u64,
) -> Result<(PrincipalRecord, UserProfileRecord), AuthorizationStateError> {
    require_positive("expectedVersion", expected_version)?;
    if current_principal.version != expected_version
        || current_profile.version != expected_version
        || current_principal.version != current_profile.version
        || current_principal.state == PrincipalState::Revoked
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    let replacement_version = next_version(expected_version)?;
    if current_principal.kind != PrincipalKind::User
        || principal.kind != PrincipalKind::User
        || principal.principal_id != current_principal.principal_id
        || profile.principal_id != current_principal.principal_id
        || current_profile.principal_id != current_principal.principal_id
        || principal.created_at != current_principal.created_at
        || profile.created_at != current_profile.created_at
        || principal.version != replacement_version
        || profile.version != replacement_version
        || principal.updated_at != profile.updated_at
        || principal.updated_at < current_principal.updated_at
        || profile.updated_at < current_profile.updated_at
        || principal.revoked_at.is_some()
        || !matches!(
            principal.state,
            PrincipalState::Active | PrincipalState::Disabled
        )
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "user account replacement identity, version, state, or timestamps are invalid"
                .to_owned(),
        ));
    }
    require_protocol_timestamp("principal.updatedAt", principal.updated_at)?;
    require_protocol_timestamp("profile.updatedAt", profile.updated_at)?;
    if let Some(display_name) = &profile.display_name {
        require_nonempty("displayName", display_name)?;
    }
    principal.disabled_at = match principal.state {
        PrincipalState::Active => None,
        PrincipalState::Disabled if current_principal.state == PrincipalState::Disabled => {
            current_principal.disabled_at
        }
        PrincipalState::Disabled => Some(principal.updated_at),
        PrincipalState::Revoked => None,
    };
    Ok((principal, profile))
}

pub(super) fn validate_local_credential(
    credential: &LocalCredentialRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("principalId", &credential.principal_id)?;
    require_nonempty("normalizedUsername", &credential.normalized_username)?;
    require_nonempty("passwordHash", &credential.password_hash)?;
    require_positive("hashProfile", u64::from(credential.hash_profile))?;
    require_positive("version", credential.version)?;
    require_protocol_timestamp("passwordChangedAt", credential.password_changed_at)?;
    require_protocol_timestamp("credential.updatedAt", credential.updated_at)?;
    if let Some(locked_until) = credential.locked_until {
        require_protocol_timestamp("lockedUntil", locked_until)?;
    }
    if credential.updated_at < credential.password_changed_at {
        return Err(AuthorizationStateError::InvalidRecord(
            "credential updatedAt precedes passwordChangedAt".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_login_portal(
    portal: &LoginPortalRecord,
    settings: &LoginSettingsRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("portalId", &portal.portal_id)?;
    require_nonempty("displayName", &portal.display_name)?;
    if let Some(entry_url) = &portal.entry_url {
        let parsed = url::Url::parse(entry_url).map_err(|_| {
            AuthorizationStateError::InvalidRecord("portal entryUrl is invalid".to_owned())
        })?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(AuthorizationStateError::InvalidRecord(
                "portal entryUrl must use HTTP or HTTPS".to_owned(),
            ));
        }
    }
    require_positive("portal.version", portal.version)?;
    require_positive("settings.version", settings.version)?;
    require_protocol_timestamp("portal.createdAt", portal.created_at)?;
    require_protocol_timestamp("portal.updatedAt", portal.updated_at)?;
    require_protocol_timestamp("settings.updatedAt", settings.updated_at)?;
    if settings.portal_id != portal.portal_id
        || portal.updated_at < portal.created_at
        || settings.updated_at < portal.created_at
        || settings
            .default_provider_id
            .as_ref()
            .is_some_and(|id| !portal.provider_ids.contains(id))
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "login portal settings do not match the portal".to_owned(),
        ));
    }
    for (index, provider) in portal.provider_ids.iter().enumerate() {
        require_nonempty("providerId", provider)?;
        if portal.provider_ids[..index].contains(provider) {
            return Err(AuthorizationStateError::InvalidRecord(
                "providerIds must be unique".to_owned(),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_deployment_profile(
    profile: &DeploymentProfileRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("deploymentId", &profile.deployment_id)?;
    require_nonempty("displayName", &profile.display_name)?;
    if !matches!(profile.kind, PrincipalKind::Service | PrincipalKind::Device) {
        return Err(AuthorizationStateError::InvalidRecord(
            "deployment profile kind must be service or device".to_owned(),
        ));
    }
    if profile
        .participant_id
        .as_ref()
        .is_some_and(|value| value.is_empty())
        || profile
            .portal_id
            .as_ref()
            .is_some_and(|value| value.is_empty())
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "deployment profile optional IDs cannot be empty".to_owned(),
        ));
    }
    require_protocol_timestamp("createdAt", profile.created_at)?;
    require_protocol_timestamp("updatedAt", profile.updated_at)?;
    if let Some(expires_at) = profile.expires_at {
        require_protocol_timestamp("expiresAt", expires_at)?;
    }
    require_positive("version", profile.version)?;
    Ok(())
}

pub(super) fn validate_portal_route(
    route: &PortalRouteRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("routeId", &route.route_id)?;
    require_nonempty("portalId", &route.portal_id)?;
    require_positive("version", route.version)?;
    require_protocol_timestamp("createdAt", route.created_at)?;
    require_protocol_timestamp("updatedAt", route.updated_at)?;
    if route.priority.unsigned_abs() > MAX_PROTOCOL_INTEGER {
        return Err(AuthorizationStateError::InvalidRecord(
            "priority exceeds protocol integer range".to_owned(),
        ));
    }
    for (field, value) in [
        ("participantId", route.participant_id.as_deref()),
        ("origin", route.origin.as_deref()),
        ("deploymentId", route.deployment_id.as_deref()),
    ] {
        if let Some(value) = value {
            require_nonempty(field, value)?;
        }
    }
    if route.updated_at < route.created_at
        || (route.participant_id.is_none()
            && route.origin.is_none()
            && route.deployment_id.is_none())
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "portal route requires a selector".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_account_flow(
    flow: &AccountFlowRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("flowId", &flow.flow_id)?;
    require_digest("tokenHash", &flow.token_hash)?;
    require_protocol_timestamp("createdAt", flow.created_at)?;
    require_protocol_timestamp("expiresAt", flow.expires_at)?;
    for (field, value) in [
        ("targetPrincipalId", flow.target_principal_id.as_deref()),
        ("targetProviderId", flow.target_provider_id.as_deref()),
        ("returnLocation", flow.return_location.as_deref()),
    ] {
        if let Some(value) = value {
            require_nonempty(field, value)?;
        }
    }
    if flow.state != AccountFlowState::Pending
        || flow.consumed_at.is_some()
        || flow.version != 1
        || flow.expires_at <= flow.created_at
        || !matches!(
            (
                flow.kind,
                flow.target_principal_id.is_some(),
                flow.target_provider_id.is_some()
            ),
            (AccountFlowKind::FirstAdmin, false, false)
                | (AccountFlowKind::PasswordReset, true, false)
                | (AccountFlowKind::IdentityLink, true, true)
        )
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "new account flow must be pending, typed, unconsumed, version one, and unexpired"
                .to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_authority_proposal(
    proposal: &AuthorityProposalRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("proposalId", &proposal.proposal_id)?;
    require_nonempty("authorityId", &proposal.authority_id)?;
    require_nonempty("participantId", &proposal.participant_id)?;
    require_digest(
        "participantArtifactDigest",
        &proposal.participant_artifact_digest,
    )?;
    require_digest("participantNeedsDigest", &proposal.participant_needs_digest)?;
    require_digest("proposalDigest", &proposal.proposal_digest)?;
    match proposal.authority_kind {
        AuthorityKind::Deployment => {
            let deployment_id = proposal.deployment_id.as_deref().ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(
                    "deployment proposal is missing deploymentId".to_owned(),
                )
            })?;
            if proposal.authority_id
                != super::service_domain::deployment_authority_id(
                    deployment_id,
                    &proposal.participant_id,
                )?
                || proposal.payload.get("deploymentId").and_then(Value::as_str)
                    != Some(deployment_id)
                || proposal
                    .payload
                    .get("subjectId")
                    .and_then(Value::as_str)
                    .is_some_and(|subject_id| subject_id != deployment_id)
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "deployment proposal lineage is inconsistent".to_owned(),
                ));
            }
        }
        AuthorityKind::Identity if proposal.deployment_id.is_some() => {
            return Err(AuthorizationStateError::InvalidRecord(
                "identity proposal cannot name a deployment".to_owned(),
            ));
        }
        AuthorityKind::Identity => {}
    }
    require_protocol_timestamp("createdAt", proposal.created_at)?;
    if let Some(expires_at) = proposal.expires_at {
        require_protocol_timestamp("expiresAt", expires_at)?;
    }
    if proposal.state != AuthorityProposalState::Pending
        || proposal.superseded_at.is_some()
        || proposal.version != 1
        || proposal
            .expires_at
            .is_some_and(|expires| expires <= proposal.created_at)
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "new authority proposal lifecycle is invalid".to_owned(),
        ));
    }
    if canonical_capabilities(proposal.proposed_capabilities.clone())?
        != proposal.proposed_capabilities
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "proposedCapabilities must be canonical".to_owned(),
        ));
    }
    proposal_base_authority_version(proposal)?;
    Ok(())
}

pub(super) fn proposal_base_authority_version(
    proposal: &AuthorityProposalRecord,
) -> Result<Option<u64>, AuthorizationStateError> {
    match proposal.payload.get("baseAuthorityVersion") {
        Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => value.as_u64().map(Some).ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(
                "baseAuthorityVersion must be a protocol integer or null".to_owned(),
            )
        }),
        _ => Err(AuthorizationStateError::InvalidRecord(
            "proposal payload is missing baseAuthorityVersion".to_owned(),
        )),
    }
}

pub(super) fn validate_authority_decision(
    proposal_id: &str,
    decision: &AuthorityDecisionRecord,
) -> Result<(), AuthorizationStateError> {
    if decision.proposal_id != proposal_id {
        return Err(AuthorizationStateError::InvalidRecord(
            "decision proposalId does not match".to_owned(),
        ));
    }
    require_nonempty("decidedBy", &decision.decided_by)?;
    require_protocol_timestamp("decidedAt", decision.decided_at)?;
    require_digest("decisionDigest", &decision.decision_digest)
}

pub(super) fn validate_provisioned_identity(
    identity: &ProvisionedIdentityRecord,
) -> Result<(), AuthorizationStateError> {
    require_digest("identityKeyId", &identity.identity_key_id)?;
    let derived_key_id =
        validate_ed25519_public_key("identityPublicKey", &identity.identity_public_key)?;
    require_nonempty("principalId", &identity.principal_id)?;
    require_nonempty("deploymentId", &identity.deployment_id)?;
    require_nonempty("instanceId", &identity.instance_id)?;
    require_protocol_timestamp("createdAt", identity.created_at)?;
    if identity.identity_key_id != derived_key_id
        || identity.state != ProvisionedIdentityState::Active
        || identity.revoked_at.is_some()
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "new provisioned identity key ID or lifecycle is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn validate_identity_relationships(
    state: &super::repository::MemoryState,
    identity: &ProvisionedIdentityRecord,
) -> Result<(), AuthorizationStateError> {
    let principal = state.principals.get(&identity.principal_id);
    let deployment = state.deployments.get(&identity.deployment_id);
    let instance = state.runtime_instances.get(&identity.instance_id);
    let kind_matches = matches!(
        (
            identity.kind,
            principal.map(|p| p.kind),
            deployment.map(|d| d.participant_kind)
        ),
        (
            ProvisionedIdentityKind::Service,
            Some(PrincipalKind::Service),
            Some(ParticipantKindV1::Service)
        ) | (
            ProvisionedIdentityKind::Device,
            Some(PrincipalKind::Device),
            Some(ParticipantKindV1::Device)
        )
    );
    if !kind_matches
        || instance.is_none_or(|value| {
            value.deployment_id != identity.deployment_id
                || value.principal_id != identity.principal_id
        })
        || (identity.kind == ProvisionedIdentityKind::Device
            && !state.devices.contains_key(&(
                identity.principal_id.clone(),
                identity.deployment_id.clone(),
            )))
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "provisioned identity relationships do not match exactly".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_provisioning_secret(
    secret: &DeviceProvisioningSecretRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("secretId", &secret.secret_id)?;
    require_nonempty("instanceId", &secret.instance_id)?;
    require_digest("secretHash", &secret.secret_hash)?;
    require_protocol_timestamp("createdAt", secret.created_at)?;
    require_protocol_timestamp("expiresAt", secret.expires_at)?;
    if !matches!(
        (secret.state, secret.consumed_at),
        (ProvisioningSecretState::Pending, None) | (ProvisioningSecretState::Consumed, Some(_))
    ) || secret.version != 1
        || secret.expires_at < secret.created_at
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "new provisioning secret lifecycle is invalid".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_activation_review(
    review: &DeviceActivationReviewRecord,
) -> Result<(), AuthorizationStateError> {
    require_nonempty("reviewId", &review.review_id)?;
    require_nonempty("principalId", &review.principal_id)?;
    require_nonempty("deploymentId", &review.deployment_id)?;
    require_nonempty("instanceId", &review.instance_id)?;
    require_digest("requestDigest", &review.request_digest)?;
    require_protocol_timestamp("requestedAt", review.requested_at)?;
    if review.state != DeviceActivationReviewState::Pending
        || review.decided_at.is_some()
        || review.decided_by.is_some()
        || review.reason.is_some()
        || review.version != 1
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "new activation review lifecycle is invalid".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_activation_decision(
    state: DeviceActivationReviewState,
    decided_at: i64,
    decided_by: &str,
) -> Result<(), AuthorizationStateError> {
    if !matches!(
        state,
        DeviceActivationReviewState::Approved | DeviceActivationReviewState::Rejected
    ) {
        return Err(AuthorizationStateError::InvalidRecord(
            "activation decision must approve or reject".to_owned(),
        ));
    }
    require_protocol_timestamp("decidedAt", decided_at)?;
    require_nonempty("decidedBy", decided_by)
}

pub(super) fn validate_idempotency_result(
    result: &IdempotencyResultRecord,
) -> Result<(), AuthorizationStateError> {
    require_digest("scopeKey", &result.scope_key)?;
    require_nonempty("purpose", &result.purpose)?;
    require_nonempty("signerId", &result.signer_id)?;
    require_nonempty("requestId", &result.request_id)?;
    require_digest("requestDigest", &result.request_digest)?;
    require_protocol_timestamp("createdAt", result.created_at)?;
    require_protocol_timestamp("expiresAt", result.expires_at)?;
    if result.expires_at <= result.created_at {
        return Err(AuthorizationStateError::InvalidRecord(
            "idempotency expiry precedes creation".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_post_commit_action(
    action: &PostCommitActionRecord,
) -> Result<(), AuthorizationStateError> {
    require_digest("actionId", &action.action_id)?;
    require_protocol_timestamp("createdAt", action.created_at)?;
    require_protocol_timestamp("nextAttemptAt", action.next_attempt_at)?;
    if let Some(claimed_until) = action.claimed_until {
        require_protocol_timestamp("claimedUntil", claimed_until)?;
    }
    if u64::from(action.attempts) > MAX_PROTOCOL_INTEGER {
        return Err(AuthorizationStateError::InvalidRecord(
            "attempts exceeds protocol integer range".to_owned(),
        ));
    }
    if action.attempts != 0
        || action.claimed_until.is_some()
        || action.last_error.is_some()
        || action.next_attempt_at < action.created_at
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "new post-commit action must be unclaimed, unattempted, and scheduled after creation"
                .to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn post_commit_action_identity_equal(
    left: &PostCommitActionRecord,
    right: &PostCommitActionRecord,
) -> bool {
    left.action_id == right.action_id
        && left.kind == right.kind
        && left.payload == right.payload
        && left.created_at == right.created_at
}

pub(super) fn next_version(version: u64) -> Result<u64, AuthorizationStateError> {
    let next = version
        .checked_add(1)
        .ok_or_else(|| AuthorizationStateError::InvalidRecord("version overflow".to_owned()))?;
    require_positive("version", next)?;
    Ok(next)
}
