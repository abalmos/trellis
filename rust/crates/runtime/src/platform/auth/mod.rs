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
//! permissions remain [`trellis_protocol::GrantSet`] values; platform
//! capabilities never expand those permissions.
//!
//! Context signing, public auth/bootstrap routes, and transport admission are
//! implemented inside this module and composed by the platform runtime.

mod account;
mod application;
mod authority;
mod builtin_semantics;
mod builtins;
pub(crate) mod context;
mod domain;
mod ephemeral;
mod evidence;
mod grant_repository;
mod http;

mod issuance;

pub(crate) const DEVICE_ACTIVATION_REVIEW_TTL_MS: i64 = 15 * 60_000;
pub(super) use builtins::{
    auth_runtime_participant_binding, cli_participant_binding, console_participant_binding,
    events_runtime_participant_binding, health_runtime_participant_binding,
    jobs_runtime_participant_binding, portal_participant_binding,
};
pub(crate) use ephemeral::{
    validate_connection_kick_response, AuthConnectionPresence, AuthEphemeralRepository,
    ConsentApproval, ConsentRequest, NatsAuthEphemeralRepository,
};
pub(crate) use grant_repository::GrantRepository;
pub(super) use http::{
    discover_oidc_providers, router as auth_http_router, AuthHttpOptions, NatsBootstrapIssuer,
};
mod model;
pub(crate) use model::{auth_event_subject, connection_event_action};

pub(crate) mod policy;
mod portal_reconciliation;
pub(crate) mod resources;
pub(crate) mod rpc;
mod sqlite;
mod transport;
pub(crate) mod verifier;

pub(super) use transport::{compile_transport_permissions, TransportPermissions};

pub(crate) async fn resolve_api_bindings<R>(
    repository: &R,
    participant: &ParticipantBindingRecord,
    provider_deployment_id: Option<&str>,
) -> Result<
    std::collections::BTreeMap<String, trellis_rs::client::AuthorizationApiBinding>,
    AuthorizationStateError,
>
where
    R: DeploymentRepository + GrantRepository + Send + Sync,
{
    api_bindings(repository, participant, provider_deployment_id, true).await
}

async fn current_api_bindings<R>(
    repository: &R,
    participant: &ParticipantBindingRecord,
    provider_deployment_id: Option<&str>,
) -> Result<
    std::collections::BTreeMap<String, trellis_rs::client::AuthorizationApiBinding>,
    AuthorizationStateError,
>
where
    R: DeploymentRepository + GrantRepository + Send + Sync,
{
    api_bindings(repository, participant, provider_deployment_id, false).await
}

async fn api_bindings<R>(
    repository: &R,
    participant: &ParticipantBindingRecord,
    provider_deployment_id: Option<&str>,
    select_providers: bool,
) -> Result<
    std::collections::BTreeMap<String, trellis_rs::client::AuthorizationApiBinding>,
    AuthorizationStateError,
>
where
    R: DeploymentRepository + GrantRepository + Send + Sync,
{
    let consumer_graph = trellis_idl::compile_evidence(
        repository
            .get_installed_package_evidence(&participant.evidence_digest)
            .await?
            .ok_or(AuthorizationStateError::ParticipantMissing)?,
    )
    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let consumer = consumer_graph
        .root_package()
        .participants()
        .values()
        .find(|candidate| candidate.identity().as_str() == participant.participant_id)
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(
                "installed participant is absent from its package evidence".to_owned(),
            )
        })?;
    let deployments = repository.list_deployment_profiles().await?;
    let mut bindings = std::collections::BTreeMap::new();
    let mut updates = Vec::new();
    let consumer_binding_scope = provider_deployment_id.unwrap_or(&participant.participant_id);
    for api_id in participant.projection.implemented_apis.keys() {
        bindings.insert(
            api_id.clone(),
            trellis_rs::client::AuthorizationApiBinding {
                provider_deployment_id: provider_deployment_id
                    .ok_or(AuthorizationStateError::NotAuthorized)?
                    .to_owned(),
            },
        );
    }
    let mut selections = consumer.uses().clone();
    if participant
        .projection
        .resources
        .values()
        .any(|resource| resource.kind == trellis_protocol::ParticipantResourceKind::EventConsumer)
    {
        let matching_apis = consumer_graph
            .packages()
            .values()
            .flat_map(|package| package.apis().keys())
            .filter(|api_id| {
                api_id.as_str() == trellis_runtime_apis::apis::trellis_events_v1::API_ID
            })
            .cloned()
            .collect::<Vec<_>>();
        let [api_id] = matching_apis.as_slice() else {
            return Err(AuthorizationStateError::InvalidRecord(format!(
                "event Consumer participant requires exactly one trellis.events@v1 definition, found {}",
                matching_apis.len()
            )));
        };
        let api_id = api_id.clone();
        selections
            .entry(api_id.clone())
            .or_insert_with(|| trellis_idl::InteractionSelection {
                api: api_id,
                actions: [
                    "Consumers.ReportDelivery",
                    "Consumers.Query",
                    "Consumers.Inspect",
                    "DeadLetters.Query",
                    "DeadLetters.Inspect",
                    "DeadLetters.Replay",
                    "DeadLetters.Dismiss",
                ]
                .into_iter()
                .map(|name| trellis_idl::ActionSelection {
                    action: trellis_idl::ActionId {
                        kind: trellis_idl::ActionKind::Rpc,
                        name: name.to_owned(),
                    },
                    direction: trellis_idl::InteractionDirection::Call,
                })
                .collect(),
                optional_capabilities: std::collections::BTreeSet::new(),
            });
    }
    if selections.is_empty() {
        return Ok(bindings);
    }
    if !select_providers {
        for api in selections.keys() {
            let api_id = api.as_str();
            if participant.projection.implemented_apis.contains_key(api_id) {
                continue;
            }
            let provider_deployment_id = repository
                .get_api_binding(consumer_binding_scope, api_id)
                .await?
                .ok_or(AuthorizationStateError::NotAuthorized)?;
            bindings.insert(
                api_id.to_owned(),
                trellis_rs::client::AuthorizationApiBinding {
                    provider_deployment_id,
                },
            );
        }
        return Ok(bindings);
    }
    let provider_candidates = futures_util::future::join_all(
        deployments
            .iter()
            .filter(|deployment| deployment.state == DeploymentProfileState::Active)
            .filter_map(|deployment| {
                deployment.participant_id.as_ref().map(|participant_id| {
                    (deployment.deployment_id.clone(), participant_id.clone())
                })
            })
            .map(|(deployment_id, participant_id)| async move {
                let Some((_, provider)) = repository
                    .get_installed_participant_record(participant_id, None)
                    .await?
                else {
                    return Ok(None);
                };
                let provider_graph = trellis_idl::compile_evidence(
                    repository
                        .get_installed_package_evidence(&provider.evidence_digest)
                        .await?
                        .ok_or(AuthorizationStateError::ParticipantMissing)?,
                )
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
                Ok::<_, AuthorizationStateError>(Some((deployment_id, provider, provider_graph)))
            }),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    for (api, selection) in &selections {
        let api_id = api.as_str();
        if participant.projection.implemented_apis.contains_key(api_id) {
            continue;
        }
        let current = repository
            .get_api_binding(consumer_binding_scope, api_id)
            .await?;
        let mut compatible = Vec::new();
        for (deployment_id, provider, provider_graph) in &provider_candidates {
            if provider.projection.implemented_apis.contains_key(api_id)
                && trellis_idl::compare_selected(&consumer_graph, selection, provider_graph)
                    .compatible
            {
                compatible.push(deployment_id.clone());
            }
        }
        compatible.sort_unstable();
        tracing::info!(
            event = "trellis.auth.api_binding.resolve",
            participant_id = %participant.participant_id,
            binding_scope = %consumer_binding_scope,
            api_id,
            current = ?current,
            compatible = ?compatible,
            "resolving authorization API provider binding"
        );
        let provider_deployment_id = if let Some(current) = current
            .as_ref()
            .filter(|current| compatible.contains(current))
        {
            current.clone()
        } else {
            compatible
                .first()
                .cloned()
                .ok_or(AuthorizationStateError::NotAuthorized)?
        };
        if current.as_deref() != Some(provider_deployment_id.as_str()) {
            updates.push((api_id.to_owned(), provider_deployment_id.clone()));
        }
        bindings.insert(
            api_id.to_owned(),
            trellis_rs::client::AuthorizationApiBinding {
                provider_deployment_id,
            },
        );
    }
    for (api_id, provider_deployment_id) in updates {
        repository
            .put_api_binding(consumer_binding_scope, &api_id, &provider_deployment_id)
            .await?;
    }
    Ok(bindings)
}

#[cfg(test)]
mod tests;

pub(crate) use application::repository::IdempotentOutcome;
pub(crate) use application::repository::{
    AccountCreation, AccountFlowCreation, AccountRepository, ActivationReviewClaim,
    ActivationReviewCreation, ActivationReviewDecision, DeploymentProfileCreation,
    DeploymentProfileMutation, DeploymentRepository, DeviceDelegationMutation, DeviceProvisioning,
    DeviceProvisioningSecretConsumption, FirstAdminCompletion, IdentityLinkCompletion,
    LocalLoginAttempt, LoginPortalMutation, OutboxRepository, PasswordChange,
    PasswordResetCompletion, PortalRepository, PortalRouteMutation, PortalRouteRemoval,
    ProviderIdentityUnlink, ProvisionedInstanceMutation, ProvisioningRepository,
    ServiceIdentityProvisioning, SessionCreation, SessionRepository, SessionRevocation,
    UserAccountMutation,
};
pub(crate) use application::validation::validate_login_portal;
pub(crate) use application::{
    AuthService, AuthServiceConfig, ChangePasswordInput, ClaimActivationReviewInput,
    CompleteIdentityLinkInput, CompletePasswordResetInput, CreateAccountFlowInput,
    CreateActivationReviewInput, CreateFederatedUserInput, CreateLocalUserInput,
    CreateSessionInput, CreateUserInput, DecideActivationReviewInput, EnrollDeviceIdentityInput,
    FirstAdminAuthorityTarget, FirstAdminBinding, FirstAdminFederatedRegistration,
    FirstAdminRegistration, LocalAuthentication, ProvisionDeviceInput,
    ProvisionServiceIdentityInput, UpdateUserInput, UserAccount,
};
pub(crate) use authority::validate_principal;
pub(crate) use authority::{AuthorityEvidenceRepository, ContextRepository};
pub(crate) use authority::{IssuanceConnection, IssuanceCredential};
pub(crate) use context::{
    AuthorizationContextBundle, AuthorizationContextIssueRequest, AuthorizationContextService,
    AuthorizationRegistryBinding,
};
pub(crate) use domain::MutationActor;
pub(crate) use domain::{
    validate_ed25519_public_key, verify_detached_ed25519_proof, GrantBindingReplacement,
};
pub use domain::{
    ApprovalMode, ApprovedCapability, ApprovedResource, AuthorizationResourceKind,
    AuthorizationStateError, DelegationCeiling, DeploymentRecord, DeviceDelegationRecord,
    DeviceDelegationState, DeviceRecord, DeviceState, GrantBinding, GrantBindingState,
    GrantOwnerKind, IssuableAuthorizationState, NewSession, ParticipantBindingRecord,
    ParticipantBindingState, PortalGrantProvenance, PrincipalKind, PrincipalRecord, PrincipalState,
    ProviderIdentityLink, ResourceBindingEvidence, ResourceBindingState, ResourceCommitment,
    ResourceProviderIdentity, RuntimeInstanceRecord, RuntimeInstanceState, SessionRecord,
    SessionRuntimeBinding, SessionState, MAX_PROTOCOL_INTEGER,
};
pub(crate) use model::PortalPolicySnapshot;
pub(crate) use model::{activation_review_event, activation_review_event_action_id};
pub use model::{
    AccountFlowKind, AccountFlowRecord, AccountFlowState, CapabilityGroupRecord,
    DeploymentProfileRecord, DeploymentProfileState, DeviceActivationReviewRecord,
    DeviceActivationReviewState, DeviceProvisioningSecretRecord, DeviceReviewMode,
    IdempotencyResultRecord, LocalCredentialRecord, LoginPortalRecord, LoginSettingsRecord,
    PortalGrantBindingRecord, PortalGrantOverrideRecord, PortalRoleMapping, PortalRouteRecord,
    PostCommitActionKind, PostCommitActionRecord, ProvisionedIdentityKind,
    ProvisionedIdentityRecord, ProvisionedIdentityState, ProvisioningSecretState,
    UserProfileRecord,
};
pub(crate) use policy::{
    participant_resource_commitments, portal_policy_snapshot, resolve_portal_authority_selection,
    ProviderLoginAttributes,
};
pub(crate) use portal_reconciliation::{
    portal_policy_reconciliation, PortalPolicyReconciliationHandle,
};
pub use sqlite::SqliteAuthorizationStore;
