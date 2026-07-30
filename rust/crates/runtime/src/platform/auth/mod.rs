//! Rust-owned authorization state and deterministic authority materialization.
//!
//! This module is the platform-internal ownership boundary for principals,
//! provider identities, sessions, exact participant bindings, desired identity
//! and deployment authority, runtime evidence, materialized authority, and the
//! unsigned state consumed by later authorization-context issuance.
//!
//! Desired authority records an accepted decision. Materialized authority is a
//! separate fail-closed projection of that decision against authority-scoped
//! participant, dependency, resource, and deployment evidence. Session,
//! instance, and activation eligibility is checked only during issuance. Exact
//! permissions remain [`trellis_protocol::GrantSetV1`] values; platform
//! capabilities never expand those permissions.
//!
//! Context signing and public auth/bootstrap routes are intentionally outside
//! this module.

mod account;
mod auth_service;
mod companion_repository;
pub(crate) mod context;
mod domain;
mod ephemeral;
mod http;
pub(crate) use ephemeral::{
    AuthConnectionPresence, AuthEphemeralRepository, ConnectReplayRecord,
    NatsAuthEphemeralRepository,
};
pub(super) use http::{
    discover_oidc_providers, router as auth_http_router, AuthHttpOptions, NatsBootstrapIssuer,
};

pub(super) fn administration_participant_binding(
    resolved_at: i64,
) -> Result<ParticipantBindingRecord, AuthorizationStateError> {
    let api_value: serde_json::Value =
        serde_json::from_str(include_str!("../../../trellis.api.json"))
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let api = trellis_protocol::parse_api_v1(&api_value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let value: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../trellis/artifacts/trellis.admin.participant.json"
    ))
    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    trellis_protocol::lint_participant_v1_authoring(&value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let participant = trellis_protocol::parse_participant_v1(&value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let api_id = api.id().to_owned();
    let resolved = trellis_protocol::resolve_participant_v1(
        &participant,
        &std::collections::BTreeMap::from([(api_id.clone(), api)]),
    )
    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    Ok(ParticipantBindingRecord {
        participant_id: resolved.participant_id().to_owned(),
        participant_kind: resolved.participant_kind(),
        artifact_digest: resolved.participant_digest().to_owned(),
        needs_digest: resolved
            .needs()
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        participant_json: participant
            .canonical_json()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        api_artifacts_json: trellis_protocol::canonicalize_json(&serde_json::json!({
            api_id: api_value,
        }))
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        resolved_at,
        state: ParticipantBindingState::Resolved,
        error: None,
    })
}

pub(super) fn auth_runtime_participant_binding(
    resolved_at: i64,
) -> Result<ParticipantBindingRecord, AuthorizationStateError> {
    let api_value: serde_json::Value =
        serde_json::from_str(include_str!("../../../trellis.api.json"))
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    trellis_protocol::lint_api_v1_authoring(&api_value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let api = trellis_protocol::parse_api_v1(&api_value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let mut participant_value: serde_json::Value =
        serde_json::from_str(include_str!("../../../trellis.participant.json"))
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let mut api_values = std::collections::BTreeMap::from([(api.id().to_owned(), api_value)]);
    for (alias, manifest) in [
        ("core", trellis_rs::sdk::core::contract::contract_manifest()),
        (
            "eventlog",
            trellis_rs::sdk::eventlog::contract::contract_manifest(),
        ),
        (
            "health",
            trellis_rs::sdk::health::contract::contract_manifest(),
        ),
        ("jobs", trellis_rs::sdk::jobs::contract::contract_manifest()),
        (
            "state",
            trellis_rs::sdk::state::contract::contract_manifest(),
        ),
    ] {
        let api = trellis_rs::contracts::compile_protocol_artifacts(
            &serde_json::to_value(manifest)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            &std::collections::BTreeMap::new(),
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let value = api.api;
        let parsed = trellis_protocol::parse_api_v1(&value)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        participant_value["implements"][alias] = serde_json::json!({
            "api": parsed.id(),
            "apiDigest": parsed.digest().map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        });
        api_values.insert(parsed.id().to_owned(), value);
    }
    trellis_protocol::lint_participant_v1_authoring(&participant_value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let participant = trellis_protocol::parse_participant_v1(&participant_value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let mut apis = std::collections::BTreeMap::new();
    for value in api_values.values() {
        let api = trellis_protocol::parse_api_v1(value)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        apis.insert(api.id().to_owned(), api);
    }
    let resolved = trellis_protocol::resolve_participant_v1(&participant, &apis)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    Ok(ParticipantBindingRecord {
        participant_id: resolved.participant_id().to_owned(),
        participant_kind: resolved.participant_kind(),
        artifact_digest: resolved.participant_digest().to_owned(),
        needs_digest: resolved
            .needs()
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        participant_json: participant
            .canonical_json()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        api_artifacts_json: trellis_protocol::canonicalize_json(
            &serde_json::to_value(api_values)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        resolved_at,
        state: ParticipantBindingState::Resolved,
        error: None,
    })
}
mod materializer;
mod reconciliation;
mod repository;
mod resources;
mod service;
mod service_domain;
mod sqlite;
mod transport;
pub(super) use resources::{ensure_authority_dependencies, ensure_deployment_resources};
pub(super) use transport::{compile_transport_permissions, TransportPermissions};

#[cfg(test)]
mod artifact_tests;
#[cfg(test)]
mod tests;

pub use auth_service::{
    AuthService, AuthServiceConfig, CompleteIdentityLinkInput, CreateAccountFlowInput,
    CreateActivationReviewInput, CreateAuthorityProposalInput, CreateFederatedUserInput,
    CreateLocalUserInput, CreateSessionInput, CreateUserInput, CreatedAccountFlow,
    DecideActivationReviewInput, DecideAuthorityProposalInput, EnrollDeviceIdentityInput,
    FirstAdminAccount, FirstAdminAuthorityTarget, FirstAdminBootstrap,
    FirstAdminFederatedRegistration, FirstAdminRegistration, LocalAuthentication,
    PresentDeploymentAuthorityInput, ProvisionDeviceInput, ProvisionServiceIdentityInput,
    ProvisionedDevice, UpdateUserInput, UserAccount,
};
pub use companion_repository::{
    AccountCreation, AccountFlowCreation, AccountFlowRepository, AccountRepository,
    ActivationReviewCreation, ActivationReviewDecision, AuthSessionRepository,
    AuthorityProposalCreation, AuthorityProposalDecision, AuthorityProposalRepository,
    ClientBootstrapAdmission, DeploymentProfileCreation, DeploymentProfileMutation,
    DeploymentProfileRepository, DeviceDelegationMutation, DeviceProvisioning,
    DeviceProvisioningSecretConsumption, FirstAdminCompletion, IdempotencyRepository,
    IdempotentOutcome, IdentityLinkCompletion, LocalLoginAttempt, LoginPortalMutation,
    LoginPortalRepository, PasswordChange, PasswordResetCompletion, PortalRouteMutation,
    PortalRouteRemoval, PostCommitActionRepository, ProviderIdentityUnlink,
    ProvisionedInstanceMutation, ProvisioningRepository, ServiceIdentityProvisioning,
    SessionCreation, SessionRevocation, UserAccountMutation,
};
pub(crate) use context::*;
pub use domain::{
    AuthorityDecision, AuthorityEvidenceScope, AuthorityKind, AuthorityState, AuthorityTarget,
    AuthorizationStateError, AuthorizationTransition, AuthorizationTransitionKind,
    AuthorizationTransitionOutboxRecord, DelegationEvidence, DependencyEvidence, DependencyState,
    DeploymentAuthorityRecord, DeploymentRecord, DesiredAuthorityRecord, DeviceDelegationRecord,
    DeviceDelegationState, DeviceEvidence, DeviceRecord, DeviceState, IdentityAuthorityRecord,
    IssuableAuthorizationState, MaterializationState, MaterializedAuthorityRecord, NewSession,
    ParticipantBindingRecord, ParticipantBindingState, PrincipalAuthorizationChange, PrincipalKind,
    PrincipalRecord, PrincipalState, ProviderIdentityLink, ResourceBindingEvidence,
    ResourceBindingState, ResourceProviderIdentity, RuntimeEvidence, RuntimeInstanceRecord,
    RuntimeInstanceState, ServiceEvidence, SessionRecord, SessionRuntimeBinding, SessionState,
    MAX_PROTOCOL_INTEGER,
};
pub(crate) use reconciliation::authorization_reconciliation_channel;
pub use reconciliation::{AuthorizationReconciliationHandle, ReconciliationCause};
pub use repository::{
    ActiveProviderEvidence, AuthorityMaterializationSnapshot, AuthorityReconciliationOutcome,
    AuthoritySnapshotToken, AuthoritySubjectRecord, AuthorizationMaterializationRepository,
    DeploymentAuthorityRepository, EvidenceRepository, IdentityAuthorityRepository,
    InMemoryAuthorizationStore, IssuanceSnapshot, MaterializationReplacement,
    ParticipantBindingRepository, PrincipalRepository, ProviderIdentityRepository,
    SessionRepository,
};
pub use service::AuthorizationStateService;
pub(crate) use service_domain::deployment_authority_id;
pub use service_domain::{
    AccountFlowKind, AccountFlowRecord, AccountFlowState, AuthorityDecisionOutcome,
    AuthorityDecisionRecord, AuthorityProposalKind, AuthorityProposalRecord,
    AuthorityProposalState, DeploymentProfileRecord, DeploymentProfileState,
    DeviceActivationReviewRecord, DeviceActivationReviewState, DeviceProvisioningSecretRecord,
    IdempotencyResultRecord, LocalCredentialRecord, LoginPortalRecord, LoginSettingsRecord,
    PortalRouteRecord, PostCommitActionKind, PostCommitActionRecord, ProvisionedIdentityKind,
    ProvisionedIdentityRecord, ProvisionedIdentityState, ProvisioningSecretState,
    UserProfileRecord,
};
pub use sqlite::SqliteAuthorizationStore;
