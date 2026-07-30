use async_nats::header::HeaderMap;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use bytes::Bytes;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use futures_util::StreamExt;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use trellis_protocol::{
    parse_api_v1, parse_session_proof_v1, session_proof_request_digest_v1, verify_session_proof_v1,
    ApiArtifactV1, ApiSurfaceKindV1, AuthorizationPrincipalKindV1, GrantSetV1, ParticipantKindV1,
    PermissionActionV1, PermissionAtomV1, PermissionTargetV1, ProtocolError, SessionProofInputV1,
    SessionProofPolicyV1,
};
use trellis_rs::auth::{
    AuthEventPublisher, AuthEventValidationStatus, AuthEventsValidateRequest,
    AuthEventsValidateResponse, AuthRequestsValidateRequest, AuthRequestsValidateResponse,
};
use ulid::Ulid;

use super::auth::{
    ensure_authority_dependencies, ensure_deployment_resources, AccountFlowKind, AccountRepository,
    AuthConnectionPresence, AuthEphemeralRepository, AuthService, AuthorityDecision,
    AuthorityDecisionOutcome, AuthorityDecisionRecord, AuthorityKind, AuthorityProposalKind,
    AuthorityProposalRecord, AuthorityProposalRepository, AuthorityProposalState, AuthorityState,
    AuthorityTarget, AuthorizationMaterializationRepository, AuthorizationStateError,
    CreateAccountFlowInput, CreateAuthorityProposalInput, CreateUserInput,
    DecideActivationReviewInput, DecideAuthorityProposalInput, DeploymentAuthorityRecord,
    DeploymentAuthorityRepository, DeploymentProfileCreation, DeploymentProfileMutation,
    DeploymentProfileRecord, DeploymentProfileRepository, DeploymentProfileState,
    DesiredAuthorityRecord, DeviceActivationReviewRecord, DeviceActivationReviewState,
    DeviceDelegationMutation, DeviceDelegationRecord, DeviceDelegationState, EvidenceRepository,
    IdempotencyRepository, IdempotencyResultRecord, IdempotentOutcome, IdentityAuthorityRecord,
    IdentityAuthorityRepository, IssuableAuthorizationState, LoginPortalMutation,
    LoginPortalRecord, LoginPortalRepository, LoginSettingsRecord, NatsAuthEphemeralRepository,
    ParticipantBindingRecord, ParticipantBindingRepository, PortalRouteMutation, PortalRouteRecord,
    PortalRouteRemoval, PostCommitActionKind, PostCommitActionRecord, PrincipalKind,
    PrincipalRepository, PrincipalState, ProviderIdentityRepository, ProviderIdentityUnlink,
    ProvisionDeviceInput, ProvisionServiceIdentityInput, ProvisionedIdentityKind,
    ProvisionedIdentityRecord, ProvisionedIdentityState, ProvisionedInstanceMutation,
    ProvisioningRepository, RuntimeInstanceState, SessionRecord, SessionRepository,
    SqliteAuthorizationStore, UpdateUserInput, UserAccount,
};
use crate::shutdown::StopHandle;
use crate::supervisor::RuntimeError;

const REQUEST_MAXIMUM_AGE_SECONDS: i64 = 60;
const REQUEST_MAXIMUM_FUTURE_SKEW_SECONDS: i64 = 5;
const MAX_CONCURRENT_REQUESTS: usize = 64;

pub(crate) struct AuthRpcRuntime {
    subscriber: async_nats::Subscriber,
    processor: AuthRpcProcessor,
}

#[derive(Clone)]
struct AuthRpcProcessor {
    client: async_nats::Client,
    system_client: async_nats::Client,
    service: AuthService<SqliteAuthorizationStore>,
    ephemeral: NatsAuthEphemeralRepository,
    public_origin: String,
    native_nats_servers: Vec<String>,
    websocket_nats_servers: Vec<String>,
    request_replays: Arc<Mutex<BTreeMap<(String, String), i64>>>,
}

struct ValidatedRequest {
    session: SessionRecord,
    authorization: IssuableAuthorizationState,
}

impl AuthRpcRuntime {
    pub(crate) async fn start(
        client: async_nats::Client,
        system_client: async_nats::Client,
        service: AuthService<SqliteAuthorizationStore>,
        ephemeral: NatsAuthEphemeralRepository,
        public_origin: String,
        native_nats_servers: Vec<String>,
        websocket_nats_servers: Vec<String>,
    ) -> Result<Self, AuthorizationStateError> {
        let subscriber = client
            .queue_subscribe("rpc.v1.Auth.>", "trellis-auth-rpc".to_owned())
            .await
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        Ok(Self {
            subscriber,
            processor: AuthRpcProcessor {
                client,
                system_client,
                service,
                ephemeral,
                public_origin,
                native_nats_servers,
                websocket_nats_servers,
                request_replays: Arc::new(Mutex::new(BTreeMap::new())),
            },
        })
    }

    pub(crate) async fn run(mut self, stop: StopHandle) -> Result<(), RuntimeError> {
        let mut requests = tokio::task::JoinSet::new();
        loop {
            tokio::select! {
                () = stop.stopped() => break,
                result = requests.join_next(), if !requests.is_empty() => {
                    if let Some(result) = result {
                        result.map_err(|error| RuntimeError::Platform(error.to_string()))??;
                    }
                }
                message = self.subscriber.next(), if requests.len() < MAX_CONCURRENT_REQUESTS => {
                    let Some(message) = message else {
                        return Err(RuntimeError::Platform("Auth RPC subscription closed".to_owned()));
                    };
                    let processor = self.processor.clone();
                    requests.spawn(async move { processor.process(message).await });
                }
            }
        }
        requests.abort_all();
        Ok(())
    }
}

impl AuthRpcProcessor {
    async fn deployment_authority_plan(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let deployment_id = required_string(&input, "deploymentId")?;
        let participant_artifact = input
            .get("participantArtifact")
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("participantArtifact is required".to_owned())
            })?
            .clone();
        let referenced_api_artifacts = input
            .get("referencedApiArtifacts")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(
                    "referencedApiArtifacts is required".to_owned(),
                )
            })?
            .clone();
        let now = now_millis()?;
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        let outcome = self
            .service
            .present_deployment_authority(super::auth::PresentDeploymentAuthorityInput {
                deployment_id: deployment_id.to_owned(),
                participant_artifact,
                referenced_api_artifacts,
                created_at: now,
                expires_at: input.get("expiresAt").and_then(Value::as_i64),
                idempotency: rpc_idempotency(
                    "Auth.DeploymentAuthority.Plan",
                    &caller.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let proposal = match outcome {
            IdempotentOutcome::Applied(proposal) => proposal,
            IdempotentOutcome::Replayed(value) => {
                let proposal_id = value
                    .get("proposalId")
                    .and_then(Value::as_str)
                    .ok_or(AuthorizationStateError::StorageConflict)?;
                self.service
                    .repository()
                    .get_authority_proposal(proposal_id)
                    .await?
                    .map(|value| value.0)
                    .ok_or(AuthorizationStateError::StorageConflict)?
            }
        };
        Ok(json!({ "proposal": proposal_value(proposal, None) }))
    }

    async fn process(&self, message: async_nats::Message) -> Result<(), RuntimeError> {
        tracing::debug!(subject = %message.subject, reply = ?message.reply, "processing Auth RPC request");
        let Some(reply) = message.reply.clone() else {
            return Ok(());
        };
        let subject = message.subject.as_str();
        let dispatch_started = Instant::now();
        let result = self.dispatch(subject, &message).await;
        let dispatch_elapsed = dispatch_started.elapsed();
        if dispatch_elapsed >= Duration::from_secs(1) {
            tracing::warn!(
                subject,
                duration_ms = dispatch_elapsed.as_millis(),
                "Auth RPC dispatch exceeded one second"
            );
        }
        tracing::debug!(
            subject,
            success = result.is_ok(),
            "finished Auth RPC dispatch"
        );
        let (headers, payload) = match result {
            Ok(value) => (HeaderMap::new(), serde_json::to_vec(&value)),
            Err(error) => {
                tracing::warn!(subject, %error, "Auth RPC request failed");
                let mut headers = HeaderMap::new();
                headers.insert("status", "error");
                let error = public_rpc_error(subject, &error);
                (headers, serde_json::to_vec(&error))
            }
        };
        let payload = payload.map_err(|error| RuntimeError::Platform(error.to_string()))?;
        let publish_started = Instant::now();
        self.client
            .publish_with_headers(reply, headers, Bytes::from(payload))
            .await
            .map_err(|error| RuntimeError::Platform(error.to_string()))?;
        let publish_elapsed = publish_started.elapsed();
        if publish_elapsed >= Duration::from_secs(1) {
            tracing::warn!(
                subject,
                duration_ms = publish_elapsed.as_millis(),
                "Auth RPC reply publish exceeded one second"
            );
        }
        tracing::debug!(subject, "published Auth RPC response");
        Ok(())
    }

    async fn dispatch(
        &self,
        subject: &str,
        message: &async_nats::Message,
    ) -> Result<Value, AuthorizationStateError> {
        if subject == "rpc.v1.Auth.Requests.Validate" {
            let request: AuthRequestsValidateRequest = serde_json::from_slice(&message.payload)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let validated = self.validate_request(&request).await?;
            return serde_json::to_value(AuthRequestsValidateResponse {
                allowed: true,
                inbox_prefix: validated.session.inbox_prefix,
                caller: caller_value(&validated.authorization)?,
            })
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()));
        }
        if subject == "rpc.v1.Auth.Events.Validate" {
            let request: AuthEventsValidateRequest = serde_json::from_slice(&message.payload)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            return serde_json::to_value(self.validate_event(&request).await?)
                .map_err(|error| AuthorizationStateError::Storage(error.to_string()));
        }

        let request = request_from_headers(subject, message)?;
        let validated = self.validate_request(&request).await?;
        match subject {
            "rpc.v1.Auth.Sessions.Me" => self.sessions_me(validated).await,
            "rpc.v1.Auth.Sessions.List" => self.sessions_list(&message.payload).await,
            "rpc.v1.Auth.Sessions.Revoke" => {
                self.sessions_revoke(&message.payload, Some(&validated))
                    .await
            }
            "rpc.v1.Auth.Sessions.Logout" => {
                self.sessions_logout(&message.payload, &validated).await
            }
            "rpc.v1.Auth.Connections.List" => {
                self.connections_list(&message.payload, validated).await
            }
            "rpc.v1.Auth.Connections.Kick" => self.connections_kick(&message.payload).await,
            "rpc.v1.Auth.Portals.List" => self.portals_list(&message.payload).await,
            "rpc.v1.Auth.Portals.Get" => self.portals_get(&message.payload).await,
            "rpc.v1.Auth.Portals.Put" => self.portals_put(&message.payload, &validated).await,
            "rpc.v1.Auth.Portals.Remove" => self.portals_remove(&message.payload, &validated).await,
            "rpc.v1.Auth.Portals.LoginSettings.Get" => {
                self.portal_settings_get(&message.payload).await
            }
            "rpc.v1.Auth.Portals.LoginSettings.Update" => {
                self.portal_settings_update(&message.payload, &validated)
                    .await
            }
            "rpc.v1.Auth.Portals.Routes.Put" => {
                self.portal_route_put(&message.payload, &validated).await
            }
            "rpc.v1.Auth.Portals.Routes.Remove" => {
                self.portal_route_remove(&message.payload, &validated).await
            }
            "rpc.v1.Auth.Capabilities.List" => self.capabilities_list(&message.payload).await,
            "rpc.v1.Auth.Users.Create" => self.users_create(&message.payload, &validated).await,
            "rpc.v1.Auth.Users.Get" => self.users_get(&message.payload).await,
            "rpc.v1.Auth.Users.Resolve" => self.users_resolve(&message.payload).await,
            "rpc.v1.Auth.Users.List" => self.users_list(&message.payload).await,
            "rpc.v1.Auth.Users.Update" => self.users_update(&message.payload, &validated).await,
            "rpc.v1.Auth.Users.PasswordReset.Create" => {
                self.password_reset_create(&message.payload, &validated)
                    .await
            }
            "rpc.v1.Auth.Users.Password.Change" => {
                self.password_change(&message.payload, &validated).await
            }
            "rpc.v1.Auth.Users.IdentityLink.Create" => {
                self.identity_link_create(&message.payload, &validated)
                    .await
            }
            "rpc.v1.Auth.UserIdentities.List" => {
                self.user_identities_list(&message.payload, &validated)
                    .await
            }
            "rpc.v1.Auth.UserIdentities.Unlink" => {
                self.user_identities_unlink(&message.payload, &validated)
                    .await
            }
            "rpc.v1.Auth.Deployments.Create" => {
                self.deployments_create(&message.payload, &validated).await
            }
            "rpc.v1.Auth.Deployments.List" => self.deployments_list(&message.payload).await,
            "rpc.v1.Auth.Deployments.Enable" => {
                self.deployments_set_state(
                    &message.payload,
                    &validated,
                    DeploymentProfileState::Active,
                )
                .await
            }
            "rpc.v1.Auth.Deployments.Disable" => {
                self.deployments_set_state(
                    &message.payload,
                    &validated,
                    DeploymentProfileState::Disabled,
                )
                .await
            }
            "rpc.v1.Auth.Deployments.Remove" => {
                self.deployments_set_state(
                    &message.payload,
                    &validated,
                    DeploymentProfileState::Removed,
                )
                .await
            }
            "rpc.v1.Auth.ServiceInstances.Provision" => {
                self.service_instances_provision(&message.payload, &validated)
                    .await
            }
            "rpc.v1.Auth.ServiceInstances.List" => {
                self.service_instances_list(&message.payload).await
            }
            "rpc.v1.Auth.ServiceInstances.Enable" => {
                self.provisioned_instance_set_state(
                    &message.payload,
                    &validated,
                    ProvisionedIdentityKind::Service,
                    RuntimeInstanceState::Active,
                )
                .await
            }
            "rpc.v1.Auth.ServiceInstances.Disable" => {
                self.provisioned_instance_set_state(
                    &message.payload,
                    &validated,
                    ProvisionedIdentityKind::Service,
                    RuntimeInstanceState::Disabled,
                )
                .await
            }
            "rpc.v1.Auth.ServiceInstances.Remove" => {
                self.provisioned_instance_set_state(
                    &message.payload,
                    &validated,
                    ProvisionedIdentityKind::Service,
                    RuntimeInstanceState::Revoked,
                )
                .await
            }
            "rpc.v1.Auth.Devices.Provision" => {
                self.devices_provision(&message.payload, &validated).await
            }
            "rpc.v1.Auth.Devices.List" => self.devices_list(&message.payload).await,
            "rpc.v1.Auth.Devices.ConnectInfo.Get" => {
                self.devices_connect_info(&message.payload).await
            }
            "rpc.v1.Auth.DeviceUserAuthorities.List" => {
                self.device_user_authorities_list(&message.payload).await
            }
            "rpc.v1.Auth.DeviceUserAuthorities.Revoke" => {
                self.device_user_authorities_revoke(&message.payload, &validated)
                    .await
            }
            "rpc.v1.Auth.Devices.Enable" => {
                self.provisioned_instance_set_state(
                    &message.payload,
                    &validated,
                    ProvisionedIdentityKind::Device,
                    RuntimeInstanceState::Active,
                )
                .await
            }
            "rpc.v1.Auth.Devices.Disable" => {
                self.provisioned_instance_set_state(
                    &message.payload,
                    &validated,
                    ProvisionedIdentityKind::Device,
                    RuntimeInstanceState::Disabled,
                )
                .await
            }
            "rpc.v1.Auth.Devices.Remove" => {
                self.provisioned_instance_set_state(
                    &message.payload,
                    &validated,
                    ProvisionedIdentityKind::Device,
                    RuntimeInstanceState::Revoked,
                )
                .await
            }
            "rpc.v1.Auth.DeviceUserAuthorities.Reviews.List" => {
                self.activation_reviews_list(&message.payload).await
            }
            "rpc.v1.Auth.DeviceUserAuthorities.Reviews.Decide" => {
                self.activation_reviews_decide(&message.payload, &validated)
                    .await
            }
            "rpc.v1.Auth.DeploymentAuthority.List" => {
                self.deployment_authority_list(&message.payload).await
            }
            "rpc.v1.Auth.IdentityAuthority.List" => {
                self.identity_authority_list(&message.payload).await
            }
            "rpc.v1.Auth.IdentityAuthority.Get" => {
                self.identity_authority_get(&message.payload).await
            }
            "rpc.v1.Auth.IdentityAuthority.Revoke" => {
                self.identity_authority_revoke(&message.payload, &validated)
                    .await
            }
            "rpc.v1.Auth.DeploymentAuthority.Get" => {
                self.deployment_authority_get(&message.payload).await
            }
            "rpc.v1.Auth.DeploymentAuthority.Plan" => {
                self.deployment_authority_plan(&message.payload, &validated)
                    .await
            }
            "rpc.v1.Auth.DeploymentAuthority.Plans.List" => {
                self.authority_plans_list(&message.payload).await
            }
            "rpc.v1.Auth.DeploymentAuthority.Plans.Get" => {
                self.authority_plans_get(&message.payload).await
            }
            "rpc.v1.Auth.DeploymentAuthority.AcceptUpdate"
            | "rpc.v1.Auth.DeploymentAuthority.AcceptMigration" => {
                self.authority_accept(&message.payload, &validated).await
            }
            "rpc.v1.Auth.DeploymentAuthority.Reject" => {
                self.authority_reject(&message.payload, &validated).await
            }
            "rpc.v1.Auth.DeploymentAuthority.Reconcile" => {
                self.deployment_authority_reconcile(&message.payload, &validated)
                    .await
            }
            _ => Err(AuthorizationStateError::InvalidRecord(format!(
                "Auth RPC is not implemented by Rust: {subject}"
            ))),
        }
    }

    async fn validate_request(
        &self,
        request: &AuthRequestsValidateRequest,
    ) -> Result<ValidatedRequest, AuthorizationStateError> {
        let now = now_seconds()?;
        if request.iat < now.saturating_sub(REQUEST_MAXIMUM_AGE_SECONDS)
            || request.iat > now.saturating_add(REQUEST_MAXIMUM_FUTURE_SKEW_SECONDS)
            || request.request_id.is_empty()
            || request.subject.is_empty()
        {
            return denied("request proof is outside the accepted window");
        }
        let session = self
            .service
            .repository()
            .get_session_by_public_key(&request.session_key)
            .await?
            .ok_or(AuthorizationStateError::SessionMissing)?;
        tracing::debug!(session_id = %session.session_id, "loaded Auth RPC session");
        verify_request_proof(request)?;
        let now_ms = now
            .checked_mul(1_000)
            .ok_or_else(|| AuthorizationStateError::Storage("time overflow".to_owned()))?;
        let authorization = self
            .service
            .authorization()
            .resolve_issuable_state(&session.session_id, now_ms)
            .await?;
        tracing::debug!(session_id = %session.session_id, "resolved Auth RPC authority");
        let binding = self
            .service
            .repository()
            .get_participant_binding(
                &authorization.participant.id,
                &authorization.participant.artifact_digest,
            )
            .await?
            .ok_or(AuthorizationStateError::ParticipantMissing)?;
        let Some(capability_permissions) =
            request_capability_permissions(&request.capabilities, &authorization, &binding)?
        else {
            tracing::warn!(
                requested_capabilities = ?request.capabilities,
                granted_capabilities = ?authorization.capabilities,
                grants = ?authorization.grant_set,
                participant_id = %authorization.participant.id,
                "request capability evidence is not granted"
            );
            return denied("request capability is not granted by the active authority");
        };
        let allowed = matches!(
            request.subject.as_str(),
            "rpc.v1.Auth.Requests.Validate" | "rpc.v1.Auth.Events.Validate"
        ) || request_subject_allowed(
            &request.subject,
            &capability_permissions,
            &authorization,
            &binding,
        )?;
        if !allowed {
            return denied("request is not granted by the active authority");
        }
        {
            let mut replays = self.request_replays.lock().map_err(|_| {
                AuthorizationStateError::Storage("request replay lock poisoned".to_owned())
            })?;
            replays.retain(|_, expires_at| *expires_at > now);
            if replays
                .insert(
                    (request.session_key.clone(), request.request_id.clone()),
                    request
                        .iat
                        .saturating_add(REQUEST_MAXIMUM_AGE_SECONDS)
                        .saturating_add(1),
                )
                .is_some()
            {
                return denied("request proof was already used");
            }
        }
        Ok(ValidatedRequest {
            session,
            authorization,
        })
    }

    async fn validate_event(
        &self,
        request: &AuthEventsValidateRequest,
    ) -> Result<AuthEventsValidateResponse, AuthorizationStateError> {
        let Some(session) = self
            .service
            .repository()
            .get_session_by_public_key(&request.session_key)
            .await?
        else {
            return Ok(event_denied(AuthEventValidationStatus::MissingSession));
        };
        if verify_event_proof(request).is_err() {
            return Ok(event_denied(AuthEventValidationStatus::InvalidSignature));
        }
        let event_time = time::OffsetDateTime::parse(
            &request.event_time,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| AuthorizationStateError::InvalidRecord("invalid event time".to_owned()))?;
        let event_time_ms =
            i64::try_from(event_time.unix_timestamp_nanos() / 1_000_000).map_err(|_| {
                AuthorizationStateError::InvalidRecord("event time overflow".to_owned())
            })?;
        if event_time_ms < session.created_at
            || session
                .expires_at
                .is_some_and(|expiry| event_time_ms >= expiry)
            || session
                .revoked_at
                .is_some_and(|revoked_at| event_time_ms >= revoked_at)
        {
            return Ok(event_denied(
                AuthEventValidationStatus::OutsideSessionWindow,
            ));
        }
        let authorization = match self
            .service
            .authorization()
            .resolve_retained_event_state(&session.session_id, event_time_ms)
            .await
        {
            Ok(authorization) => authorization,
            Err(_) => return Ok(event_denied(AuthEventValidationStatus::SubjectDenied)),
        };
        let binding = self
            .service
            .repository()
            .get_participant_binding(
                &session.participant_id,
                &session.participant_artifact_digest,
            )
            .await?
            .filter(|binding| binding.needs_digest == session.participant_needs_digest)
            .ok_or(AuthorizationStateError::ParticipantMissing)?;
        if !event_subject_allowed(&request.subject, &authorization, &binding)? {
            return Ok(event_denied(AuthEventValidationStatus::SubjectDenied));
        }
        let caller = caller_value(&authorization)?;
        let publisher = AuthEventPublisher {
            kind: serde_json::to_value(session.principal_kind)
                .ok()
                .and_then(|value| value.as_str().map(str::to_owned))
                .unwrap_or_else(|| "unknown".to_owned()),
            deployment_id: authorization.deployment_id,
            instance_id: authorization.instance_id,
            contract_id: Some(session.participant_id.clone()),
            contract_digest: Some(session.participant_artifact_digest.clone()),
            session_status: serde_json::to_value(session.state)
                .ok()
                .and_then(|value| value.as_str().map(str::to_owned))
                .unwrap_or_else(|| "unknown".to_owned()),
        };
        Ok(AuthEventsValidateResponse {
            allowed: true,
            status: AuthEventValidationStatus::Verified,
            caller: Some(caller),
            publisher: Some(publisher),
        })
    }

    async fn deployments_create(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let kind = match required_string(&input, "kind")? {
            "service" => PrincipalKind::Service,
            "device" => PrincipalKind::Device,
            _ => {
                return Err(AuthorizationStateError::InvalidRecord(
                    "deployment kind is invalid".to_owned(),
                ));
            }
        };
        let now = now_millis()?;
        let deployment_id = format!("dep_{}", Ulid::new());
        let profile = DeploymentProfileRecord {
            deployment_id: deployment_id.clone(),
            kind,
            display_name: required_string(&input, "displayName")?.to_owned(),
            participant_id: nullable_string(&input, "participantId")?,
            portal_id: nullable_string(&input, "portalId")?,
            requires_device_delegation: required_bool(&input, "requiresDeviceDelegation")?,
            expires_at: input.get("expiresAt").and_then(Value::as_i64),
            state: DeploymentProfileState::Active,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.service
            .repository()
            .create_deployment_profile(DeploymentProfileCreation {
                principal: super::auth::PrincipalRecord {
                    principal_id: deployment_id.clone(),
                    kind,
                    state: PrincipalState::Active,
                    created_at: now,
                    updated_at: now,
                    version: 1,
                    disabled_at: None,
                    revoked_at: None,
                },
                profile: profile.clone(),
                idempotency: rpc_idempotency(
                    "Auth.Deployments.Create",
                    &caller.session.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        Ok(json!({ "deployment": self.deployment_value(profile).await? }))
    }

    async fn deployments_list(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let kind = input.get("kind").and_then(Value::as_str);
        let state = input.get("state").and_then(Value::as_str);
        let mut entries = Vec::new();
        for profile in self.service.repository().list_deployment_profiles().await? {
            if kind.is_some_and(|value| enum_string(profile.kind) != value)
                || state.is_some_and(|value| enum_string(profile.state) != value)
            {
                continue;
            }
            entries.push(self.deployment_value(profile).await?);
        }
        Ok(paginate_values(entries, &input))
    }

    async fn deployments_set_state(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
        state: DeploymentProfileState,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let deployment_id = required_string(&input, "deploymentId")?;
        let expected_version = required_u64(&input, "expectedVersion")?;
        let mut profile = self
            .service
            .repository()
            .get_deployment_profile(deployment_id)
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("deployment not found".to_owned())
            })?;
        if profile.version != expected_version {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let now = now_millis()?;
        profile.state = state;
        profile.updated_at = now;
        profile.version = profile
            .version
            .checked_add(1)
            .ok_or_else(|| AuthorizationStateError::Storage("version overflow".to_owned()))?;
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        let actions = (state != DeploymentProfileState::Active)
            .then(|| PostCommitActionRecord {
                action_id: format!(
                    "act_{}",
                    digest_parts(&[deployment_id, idempotency_key, "kick",])
                ),
                kind: PostCommitActionKind::Kick,
                payload: json!({ "deploymentId": deployment_id }),
                created_at: now,
                attempts: 0,
                next_attempt_at: now,
                claimed_until: None,
                last_error: None,
            })
            .into_iter()
            .collect();
        self.service
            .repository()
            .put_deployment_profile(DeploymentProfileMutation {
                profile: profile.clone(),
                expected_version,
                idempotency: rpc_idempotency(
                    "Auth.Deployments.State",
                    &caller.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions,
            })
            .await?;
        Ok(json!({
            "deployment": self.deployment_value(profile.clone()).await?,
            "mutation": {
                "resourceId": deployment_id,
                "state": state,
                "version": profile.version,
                "changed": true,
            }
        }))
    }

    async fn deployment_value(
        &self,
        profile: DeploymentProfileRecord,
    ) -> Result<Value, AuthorizationStateError> {
        let principal = self
            .service
            .repository()
            .get_principal(&profile.deployment_id)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        Ok(json!({
            "deploymentId": profile.deployment_id,
            "kind": profile.kind,
            "displayName": profile.display_name,
            "state": profile.state,
            "participantId": profile.participant_id,
            "expiresAt": profile.expires_at,
            "requiresDeviceDelegation": profile.requires_device_delegation,
            "portalId": profile.portal_id,
            "createdAt": profile.created_at,
            "updatedAt": profile.updated_at,
            "disabledAt": principal.disabled_at,
            "revokedAt": principal.revoked_at,
            "version": profile.version,
        }))
    }

    async fn bind_deployment_participant(
        &self,
        deployment_id: &str,
        requested_participant_id: Option<String>,
        expected_kind: PrincipalKind,
        input: &Value,
        caller: &ValidatedRequest,
        now: i64,
    ) -> Result<DeploymentProfileRecord, AuthorizationStateError> {
        let mut profile = self
            .service
            .repository()
            .get_deployment_profile(deployment_id)
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("deployment not found".to_owned())
            })?;
        if profile.kind != expected_kind || profile.state != DeploymentProfileState::Active {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if profile
            .participant_id
            .as_ref()
            .zip(requested_participant_id.as_ref())
            .is_some_and(|(current, requested)| current != requested)
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if profile.participant_id.is_none() {
            profile.participant_id = requested_participant_id;
            if profile.participant_id.is_none() {
                return Err(AuthorizationStateError::InvalidRecord(
                    "participantId is required before provisioning".to_owned(),
                ));
            }
            let expected_version = profile.version;
            profile.version = profile
                .version
                .checked_add(1)
                .ok_or_else(|| AuthorizationStateError::Storage("version overflow".to_owned()))?;
            profile.updated_at = now;
            self.service
                .repository()
                .put_deployment_profile(DeploymentProfileMutation {
                    profile: profile.clone(),
                    expected_version,
                    idempotency: rpc_idempotency(
                        "Auth.Deployments.BindParticipant",
                        &caller.session.principal_id,
                        required_string(input, "idempotencyKey")?,
                        input,
                        now,
                    )?,
                    actions: Vec::new(),
                })
                .await?;
        }
        Ok(profile)
    }

    async fn service_instances_provision(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let deployment_id = required_string(&input, "deploymentId")?;
        let now = now_millis()?;
        let profile = self
            .bind_deployment_participant(
                deployment_id,
                nullable_string(&input, "participantId")?,
                PrincipalKind::Service,
                &input,
                caller,
                now,
            )
            .await?;
        let outcome = self
            .service
            .provision_service_identity(ProvisionServiceIdentityInput {
                deployment_id: deployment_id.to_owned(),
                instance_id: nullable_string(&input, "instanceId")?,
                identity_public_key: required_string(&input, "identityPublicKey")?.to_owned(),
                created_at: now,
                idempotency: rpc_idempotency(
                    "Auth.ServiceInstances.Provision",
                    &caller.session.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let identity = match outcome {
            IdempotentOutcome::Applied(identity) => identity,
            IdempotentOutcome::Replayed(value) => {
                let identity_key_id = value
                    .get("identityKeyId")
                    .and_then(Value::as_str)
                    .ok_or(AuthorizationStateError::StorageConflict)?;
                self.service
                    .repository()
                    .get_provisioned_identity(identity_key_id)
                    .await?
                    .ok_or(AuthorizationStateError::StorageConflict)?
            }
        };
        let instance = self
            .service
            .repository()
            .get_runtime_instance(&identity.instance_id)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        Ok(json!({
            "instance": service_instance_value(instance, identity, &profile),
        }))
    }

    async fn service_instances_list(
        &self,
        payload: &[u8],
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let identities = self
            .service
            .repository()
            .list_provisioned_identities()
            .await?;
        let mut entries = Vec::new();
        for identity in identities
            .into_iter()
            .filter(|identity| identity.kind == ProvisionedIdentityKind::Service)
        {
            if input
                .get("deploymentId")
                .and_then(Value::as_str)
                .is_some_and(|value| identity.deployment_id != value)
            {
                continue;
            }
            let Some(instance) = self
                .service
                .repository()
                .get_runtime_instance(&identity.instance_id)
                .await?
            else {
                continue;
            };
            if input
                .get("state")
                .and_then(Value::as_str)
                .is_some_and(|value| enum_string(instance.state) != value)
            {
                continue;
            }
            let profile = self
                .service
                .repository()
                .get_deployment_profile(&identity.deployment_id)
                .await?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            entries.push(service_instance_value(instance, identity, &profile));
        }
        Ok(paginate_values(entries, &input))
    }

    async fn devices_provision(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let deployment_id = required_string(&input, "deploymentId")?;
        let now = now_millis()?;
        let profile = self
            .bind_deployment_participant(
                deployment_id,
                nullable_string(&input, "participantId")?,
                PrincipalKind::Device,
                &input,
                caller,
                now,
            )
            .await?;
        let outcome = self
            .service
            .provision_device(ProvisionDeviceInput {
                deployment_id: deployment_id.to_owned(),
                instance_id: nullable_string(&input, "instanceId")?,
                identity_public_key: nullable_string(&input, "identityPublicKey")?,
                created_at: now,
                idempotency: rpc_idempotency(
                    "Auth.Devices.Provision",
                    &caller.session.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let (principal_id, instance_id, provisioning_secret) = match outcome {
            IdempotentOutcome::Applied(device) => (
                device.principal_id,
                device.instance_id,
                device.provisioning_secret,
            ),
            IdempotentOutcome::Replayed(value) => (
                required_string(&value, "principalId")?.to_owned(),
                required_string(&value, "instanceId")?.to_owned(),
                None,
            ),
        };
        Ok(json!({
            "device": self.device_value(&principal_id, &instance_id, &profile).await?,
            "provisioningSecret": provisioning_secret,
        }))
    }

    async fn devices_connect_info(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let deployment_id = required_string(&input, "deploymentId")?;
        let instance_id = required_string(&input, "instanceId")?;
        let identity_key_id = required_string(&input, "deviceIdentityKeyId")?;
        let identity = self
            .service
            .repository()
            .get_provisioned_identity(identity_key_id)
            .await?
            .filter(|identity| {
                identity.kind == ProvisionedIdentityKind::Device
                    && identity.state == ProvisionedIdentityState::Active
                    && identity.deployment_id == deployment_id
                    && identity.instance_id == instance_id
            })
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("device identity is not active".to_owned())
            })?;
        self.service
            .repository()
            .get_device(&identity.principal_id, deployment_id)
            .await?
            .filter(|device| device.state == super::auth::DeviceState::Active)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("device is not active".to_owned())
            })?;
        let participant_id = required_string(&input, "participantId")?;
        let participant_digest = required_string(&input, "participantDigest")?;
        let mut proof_request = input.clone();
        proof_request
            .as_object_mut()
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("request must be an object".to_owned())
            })?
            .insert("proof".to_owned(), Value::Null);
        let request_digest = session_proof_request_digest_v1(&proof_request)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let proof_input = SessionProofInputV1::device_bootstrap(
            required_string(&input, "requestId")?,
            required_i64(&input, "issuedAt")?,
            deployment_id,
            instance_id,
            identity_key_id,
            required_string(&input, "newSessionPublicKey")?,
            required_string(&input, "newSessionNkey")?,
            participant_id,
            participant_digest,
            Some(required_string(&input, "challengeDigest")?.to_owned()),
            &request_digest,
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let now = now_millis()?;
        verify_session_proof_v1(
            &proof_input,
            &parse_session_proof_v1(input.get("proof").ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("proof is required".to_owned())
            })?)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            &identity.identity_public_key,
            now,
            SessionProofPolicyV1::default(),
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let delegation_required = self
            .service
            .repository()
            .get_deployment_profile(deployment_id)
            .await?
            .is_some_and(|profile| profile.requires_device_delegation);
        if delegation_required {
            self.service
                .repository()
                .get_device_delegation(&identity.principal_id, deployment_id)
                .await?
                .filter(|delegation| {
                    delegation.state == DeviceDelegationState::Active
                        && delegation
                            .expires_at
                            .is_none_or(|expires_at| expires_at > now)
                })
                .ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord(
                        "device delegation is not active".to_owned(),
                    )
                })?;
        }
        Ok(json!({
            "deploymentId": deployment_id,
            "instanceId": instance_id,
            "participantId": participant_id,
            "endpoints": {
                "native": self.native_nats_servers,
                "websocket": self.websocket_nats_servers,
                "authMode": "session_nkey",
                "authorityMode": "server_issued",
                "maximumClockSkewMs": 5_000,
            },
        }))
    }

    async fn devices_list(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let instances = self.service.repository().list_runtime_instances().await?;
        let mut entries = Vec::new();
        for device in self.service.repository().list_devices().await? {
            if input
                .get("deploymentId")
                .and_then(Value::as_str)
                .is_some_and(|value| device.deployment_id != value)
            {
                continue;
            }
            let Some(instance) = instances
                .iter()
                .find(|instance| instance.principal_id == device.principal_id)
            else {
                continue;
            };
            let profile = self
                .service
                .repository()
                .get_deployment_profile(&device.deployment_id)
                .await?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            let value = self
                .device_value(&device.principal_id, &instance.instance_id, &profile)
                .await?;
            if input
                .get("state")
                .and_then(Value::as_str)
                .is_some_and(|state| value.get("state").and_then(Value::as_str) != Some(state))
            {
                continue;
            }
            entries.push(value);
        }
        Ok(paginate_values(entries, &input))
    }

    async fn provisioned_instance_set_state(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
        kind: ProvisionedIdentityKind,
        target: RuntimeInstanceState,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let instance_id = required_string(&input, "instanceId")?;
        let expected_version = required_u64(&input, "expectedVersion")?;
        let mut instance = self
            .service
            .repository()
            .get_runtime_instance(instance_id)
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("instance not found".to_owned())
            })?;
        let mut identity = self
            .service
            .repository()
            .list_provisioned_identities()
            .await?
            .into_iter()
            .find(|identity| identity.instance_id == instance_id && identity.kind == kind);
        if kind == ProvisionedIdentityKind::Service && identity.is_none() {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let now = now_millis()?;
        let device = if kind == ProvisionedIdentityKind::Device {
            let mut device = self
                .service
                .repository()
                .get_device(&instance.principal_id, &instance.deployment_id)
                .await?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            if device.version != expected_version
                || (device.state == super::auth::DeviceState::Pending
                    && target == RuntimeInstanceState::Active)
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            device.state = match target {
                RuntimeInstanceState::Active => super::auth::DeviceState::Active,
                RuntimeInstanceState::Disabled | RuntimeInstanceState::Stale => {
                    super::auth::DeviceState::Disabled
                }
                RuntimeInstanceState::Revoked => super::auth::DeviceState::Revoked,
            };
            device.updated_at = now;
            device.version += 1;
            Some(device)
        } else {
            if instance.version != expected_version {
                return Err(AuthorizationStateError::StorageConflict);
            }
            None
        };
        instance.state = target;
        instance.updated_at = now;
        instance.version += 1;
        if let Some(identity) = identity.as_mut() {
            identity.state = match target {
                RuntimeInstanceState::Active => ProvisionedIdentityState::Active,
                RuntimeInstanceState::Disabled | RuntimeInstanceState::Stale => {
                    ProvisionedIdentityState::Active
                }
                RuntimeInstanceState::Revoked => ProvisionedIdentityState::Revoked,
            };
            identity.revoked_at = (target == RuntimeInstanceState::Revoked).then_some(now);
        }
        let action_kind = match kind {
            ProvisionedIdentityKind::Service => "ServiceInstances",
            ProvisionedIdentityKind::Device => "Devices",
        };
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        self.service
            .repository()
            .mutate_provisioned_instance(ProvisionedInstanceMutation {
                instance: instance.clone(),
                device: device.clone(),
                identity,
                expected_version,
                idempotency: rpc_idempotency(
                    &format!("Auth.{action_kind}.State"),
                    &caller.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: (target != RuntimeInstanceState::Active)
                    .then(|| PostCommitActionRecord {
                        action_id: format!(
                            "act_{}",
                            digest_parts(&[instance_id, idempotency_key, "kick"])
                        ),
                        kind: PostCommitActionKind::Kick,
                        payload: json!({ "principalId": instance.principal_id }),
                        created_at: now,
                        attempts: 0,
                        next_attempt_at: now,
                        claimed_until: None,
                        last_error: None,
                    })
                    .into_iter()
                    .collect(),
            })
            .await?;
        let profile = self
            .service
            .repository()
            .get_deployment_profile(&instance.deployment_id)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let mutation = json!({
            "resourceId": instance_id,
            "state": target,
            "version": device.as_ref().map_or(instance.version, |device| device.version),
            "changed": true,
        });
        match kind {
            ProvisionedIdentityKind::Service => {
                let identity = self
                    .service
                    .repository()
                    .list_provisioned_identities()
                    .await?
                    .into_iter()
                    .find(|identity| identity.instance_id == instance_id)
                    .ok_or(AuthorizationStateError::StorageConflict)?;
                Ok(json!({
                    "instance": service_instance_value(instance, identity, &profile),
                    "mutation": mutation,
                }))
            }
            ProvisionedIdentityKind::Device => Ok(json!({
                "device": self.device_value(
                    &instance.principal_id,
                    instance_id,
                    &profile,
                ).await?,
                "mutation": mutation,
            })),
        }
    }

    async fn device_value(
        &self,
        principal_id: &str,
        instance_id: &str,
        profile: &DeploymentProfileRecord,
    ) -> Result<Value, AuthorizationStateError> {
        let device = self
            .service
            .repository()
            .get_device(principal_id, &profile.deployment_id)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let identity = self
            .service
            .repository()
            .list_provisioned_identities()
            .await?
            .into_iter()
            .find(|identity| {
                identity.principal_id == principal_id && identity.instance_id == instance_id
            });
        let delegation = self
            .service
            .repository()
            .get_device_delegation(principal_id, &profile.deployment_id)
            .await?;
        let state = match device.state {
            super::auth::DeviceState::Pending => "pending",
            super::auth::DeviceState::Active => "active",
            super::auth::DeviceState::Disabled => "disabled",
            super::auth::DeviceState::Revoked => "revoked",
        };
        Ok(json!({
            "instanceId": instance_id,
            "deploymentId": profile.deployment_id,
            "principalId": principal_id,
            "identityPublicKey": identity.as_ref().map(|value| value.identity_public_key.clone()),
            "identityKeyId": identity.as_ref().map(|value| value.identity_key_id.clone()),
            "participantId": profile.participant_id,
            "state": state,
            "administrativeApproval": match device.state {
                super::auth::DeviceState::Pending => "pending",
                super::auth::DeviceState::Active => "approved",
                super::auth::DeviceState::Disabled => "approved",
                super::auth::DeviceState::Revoked => "revoked",
            },
            "delegationRequired": profile.requires_device_delegation,
            "delegationState": delegation.as_ref().map_or(
                if profile.requires_device_delegation { "missing" } else { "active" },
                |value| match value.state {
                    super::auth::DeviceDelegationState::Active => "active",
                    super::auth::DeviceDelegationState::Missing => "missing",
                    super::auth::DeviceDelegationState::Revoked => "revoked",
                },
            ),
            "delegationExpiresAt": delegation.and_then(|value| value.expires_at),
            "createdAt": device.created_at,
            "updatedAt": device.updated_at,
            "version": device.version,
        }))
    }

    async fn device_user_authorities_list(
        &self,
        payload: &[u8],
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let deployment_id = input.get("deploymentId").and_then(Value::as_str);
        let principal_id = input.get("principalId").and_then(Value::as_str);
        let identities = self
            .service
            .repository()
            .list_provisioned_identities()
            .await?;
        let mut entries = Vec::new();
        for device in self.service.repository().list_devices().await? {
            if deployment_id.is_some_and(|value| device.deployment_id != value)
                || principal_id.is_some_and(|value| device.principal_id != value)
            {
                continue;
            }
            let profile = self
                .service
                .repository()
                .get_deployment_profile(&device.deployment_id)
                .await?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            let identity = identities.iter().find(|identity| {
                identity.kind == ProvisionedIdentityKind::Device
                    && identity.principal_id == device.principal_id
                    && identity.deployment_id == device.deployment_id
            });
            let instance_id = identity
                .map(|identity| identity.instance_id.as_str())
                .ok_or(AuthorizationStateError::StorageConflict)?;
            entries.push(json!({
                "device": self.device_value(&device.principal_id, instance_id, &profile).await?,
                "authority": null,
            }));
        }
        Ok(paginate_values(entries, &input))
    }

    async fn device_user_authorities_revoke(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let deployment_id = required_string(&input, "deploymentId")?;
        let principal_id = required_string(&input, "devicePrincipalId")?;
        let mut device = self
            .service
            .repository()
            .get_device(principal_id, deployment_id)
            .await?
            .ok_or_else(|| AuthorizationStateError::InvalidRecord("device not found".to_owned()))?;
        let mut delegation = self
            .service
            .repository()
            .get_device_delegation(principal_id, deployment_id)
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("device delegation not found".to_owned())
            })?;
        let expected_version = device.version;
        let now = now_millis()?;
        device.updated_at = now;
        device.version += 1;
        delegation.state = DeviceDelegationState::Revoked;
        let identity = self
            .service
            .repository()
            .list_provisioned_identities()
            .await?
            .into_iter()
            .find(|identity| {
                identity.kind == ProvisionedIdentityKind::Device
                    && identity.principal_id == principal_id
                    && identity.deployment_id == deployment_id
            })
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        let action = |kind, suffix: &str, payload| PostCommitActionRecord {
            action_id: format!(
                "act_{}",
                digest_parts(&["Auth.DeviceUserAuthorities.Revoke", idempotency_key, suffix])
            ),
            kind,
            payload,
            created_at: now,
            attempts: 0,
            next_attempt_at: now,
            claimed_until: None,
            last_error: None,
        };
        self.service
            .repository()
            .mutate_device_delegation(DeviceDelegationMutation {
                device: device.clone(),
                delegation: delegation.clone(),
                expected_version,
                idempotency: rpc_idempotency(
                    "Auth.DeviceUserAuthorities.Revoke",
                    &caller.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: vec![
                    action(
                        PostCommitActionKind::Event,
                        "event",
                        json!({
                            "eventType": "Auth.DeviceUserAuthorities.Resolved",
                            "eventSubject": format!(
                                "events.v1.Auth.DeviceUserAuthorities.Resolved.{deployment_id}"
                            ),
                            "eventId": format!(
                                "evt_{}",
                                digest_parts(&[principal_id, deployment_id, idempotency_key])
                            ),
                            "occurredAt": now,
                            "deploymentId": deployment_id,
                            "instanceId": identity.instance_id.clone(),
                            "state": "revoked",
                        }),
                    ),
                    action(
                        PostCommitActionKind::Kick,
                        "kick",
                        json!({ "principalId": principal_id }),
                    ),
                ],
            })
            .await?;
        let profile = self
            .service
            .repository()
            .get_deployment_profile(deployment_id)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let kicked_session_count = self
            .service
            .repository()
            .list_sessions()
            .await?
            .into_iter()
            .filter(|session| {
                session.principal_id == principal_id
                    && session.state == super::auth::SessionState::Active
            })
            .count();
        Ok(json!({
            "device": self.device_value(&device.principal_id, &identity.instance_id, &profile).await?,
            "kickedSessionCount": kicked_session_count,
        }))
    }

    async fn activation_reviews_list(
        &self,
        payload: &[u8],
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let entries = self
            .service
            .repository()
            .list_activation_reviews()
            .await?
            .into_iter()
            .filter(|review| {
                input
                    .get("deploymentId")
                    .and_then(Value::as_str)
                    .is_none_or(|value| review.deployment_id == value)
                    && input
                        .get("state")
                        .and_then(Value::as_str)
                        .is_none_or(|value| enum_string(review.state) == value)
            })
            .map(activation_review_value)
            .collect();
        Ok(paginate_values(entries, &input))
    }

    async fn activation_reviews_decide(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let review_id = required_string(&input, "reviewId")?;
        let review = self
            .service
            .repository()
            .get_activation_review(review_id)
            .await?
            .ok_or_else(|| AuthorizationStateError::InvalidRecord("review not found".to_owned()))?;
        let state = match required_string(&input, "decision")? {
            "approve" => DeviceActivationReviewState::Approved,
            "reject" => DeviceActivationReviewState::Rejected,
            _ => {
                return Err(AuthorizationStateError::InvalidRecord(
                    "activation decision is invalid".to_owned(),
                ));
            }
        };
        let profile = self
            .service
            .repository()
            .get_deployment_profile(&review.deployment_id)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let now = now_millis()?;
        let outcome = self
            .service
            .decide_activation_review(DecideActivationReviewInput {
                review_id: review_id.to_owned(),
                expected_version: required_u64(&input, "expectedVersion")?,
                state,
                decided_by: caller.session.principal_id.clone(),
                reason: nullable_string(&input, "reason")?,
                delegation: (state == DeviceActivationReviewState::Approved
                    && profile.requires_device_delegation)
                    .then(|| DeviceDelegationRecord {
                        principal_id: review.principal_id.clone(),
                        deployment_id: review.deployment_id.clone(),
                        required: true,
                        state: DeviceDelegationState::Missing,
                        expires_at: None,
                    }),
                decided_at: now,
                idempotency: rpc_idempotency(
                    "Auth.DeviceUserAuthorities.Reviews.Decide",
                    &caller.session.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                actions: vec![PostCommitActionRecord {
                    action_id: format!(
                        "act_{}",
                        digest_parts(&[
                            review_id,
                            required_string(&input, "idempotencyKey")?,
                            "event",
                        ])
                    ),
                    kind: PostCommitActionKind::Event,
                    payload: json!({
                        "eventType": "Auth.DeviceUserAuthorities.Resolved",
                        "eventSubject": format!(
                            "events.v1.Auth.DeviceUserAuthorities.Resolved.{}",
                            review.deployment_id,
                        ),
                        "eventId": format!("evt_{}", digest_parts(&[review_id, "resolved"])),
                        "occurredAt": now,
                        "deploymentId": review.deployment_id,
                        "instanceId": review.instance_id,
                        "state": state,
                    }),
                    created_at: now,
                    attempts: 0,
                    next_attempt_at: now,
                    claimed_until: None,
                    last_error: None,
                }],
            })
            .await?;
        let review = match outcome {
            IdempotentOutcome::Applied(review) => review,
            IdempotentOutcome::Replayed(value) => {
                let review_id = required_string(&value, "reviewId")?;
                self.service
                    .repository()
                    .get_activation_review(review_id)
                    .await?
                    .ok_or(AuthorizationStateError::StorageConflict)?
            }
        };
        Ok(json!({ "review": activation_review_value(review) }))
    }

    async fn identity_authority_list(
        &self,
        payload: &[u8],
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let mut entries = self
            .service
            .repository()
            .list_identity_authorities()
            .await?;
        entries.retain(|authority| {
            input
                .get("principalId")
                .and_then(Value::as_str)
                .is_none_or(|value| authority.principal_id == value)
                && input
                    .get("participantId")
                    .and_then(Value::as_str)
                    .is_none_or(|value| authority.participant_id == value)
                && input
                    .get("state")
                    .and_then(Value::as_str)
                    .is_none_or(|value| enum_string(authority.state) == value)
        });
        Ok(paginate_values(
            entries.into_iter().map(identity_authority_value).collect(),
            &input,
        ))
    }

    async fn identity_authority_get(
        &self,
        payload: &[u8],
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let authority_id = required_string(&input, "authorityId")?;
        self.service
            .repository()
            .list_identity_authorities()
            .await?
            .into_iter()
            .find(|authority| authority.authority_id == authority_id)
            .map(|authority| json!({ "authority": identity_authority_value(authority) }))
            .ok_or_else(|| AuthorizationStateError::InvalidRecord("authority not found".to_owned()))
    }

    async fn identity_authority_revoke(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let authority_id = required_string(&input, "authorityId")?;
        let authority = self
            .service
            .repository()
            .list_identity_authorities()
            .await?
            .into_iter()
            .find(|authority| authority.authority_id == authority_id)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("authority not found".to_owned())
            })?;
        if input
            .get("expectedVersion")
            .and_then(Value::as_u64)
            .is_some_and(|expected| expected != authority.version)
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        let now = now_millis()?;
        let proposal_outcome = self
            .service
            .create_authority_proposal(CreateAuthorityProposalInput {
                authority_kind: super::auth::AuthorityKind::Identity,
                authority_id: authority.authority_id.clone(),
                deployment_id: None,
                proposal_kind: AuthorityProposalKind::Update,
                participant_id: authority.participant_id.clone(),
                participant_artifact_digest: authority.participant_artifact_digest.clone(),
                participant_needs_digest: authority.accepted_needs_digest.clone(),
                grant_set: authority.desired_grant_set.clone(),
                capabilities: authority.desired_capabilities.clone(),
                base_authority_version: Some(authority.version),
                payload: json!({
                    "principalId": authority.principal_id,
                    "baseAuthorityVersion": authority.version,
                    "reason": input.get("reason"),
                }),
                created_at: now,
                expires_at: None,
                idempotency: rpc_idempotency(
                    "Auth.IdentityAuthority.Revoke.Proposal",
                    &caller.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let proposal = match proposal_outcome {
            IdempotentOutcome::Applied(proposal) => proposal,
            IdempotentOutcome::Replayed(value) => self
                .service
                .repository()
                .get_authority_proposal(required_string(&value, "proposalId")?)
                .await?
                .map(|value| value.0)
                .ok_or(AuthorizationStateError::StorageConflict)?,
        };
        let reason = nullable_string(&input, "reason")?;
        let revoked = IdentityAuthorityRecord {
            state: AuthorityState::Revoked,
            version: authority.version + 1,
            updated_at: now,
            decision: Some(AuthorityDecision {
                decided_at: now,
                decided_by: caller.session.principal_id.clone(),
                reason: reason.clone(),
            }),
            ..authority
        };
        self.service
            .decide_authority_proposal(DecideAuthorityProposalInput {
                proposal_id: proposal.proposal_id,
                expected_version: proposal.version,
                expected_base_authority_version: None,
                outcome: AuthorityDecisionOutcome::Accepted,
                decided_by: caller.session.principal_id.clone(),
                reason,
                desired_authority: Some(DesiredAuthorityRecord::Identity(revoked)),
                decided_at: now,
                idempotency: rpc_idempotency(
                    "Auth.IdentityAuthority.Revoke.Decision",
                    &caller.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let authority = self
            .service
            .repository()
            .list_identity_authorities()
            .await?
            .into_iter()
            .find(|authority| authority.authority_id == authority_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        Ok(json!({ "authority": identity_authority_value(authority) }))
    }

    async fn deployment_authority_list(
        &self,
        payload: &[u8],
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let mut entries = self
            .service
            .repository()
            .list_deployment_authorities()
            .await?;
        entries.retain(|authority| {
            input
                .get("deploymentId")
                .and_then(Value::as_str)
                .is_none_or(|value| authority.deployment_id == value)
                && input
                    .get("participantId")
                    .and_then(Value::as_str)
                    .is_none_or(|value| authority.participant_id == value)
                && input
                    .get("state")
                    .and_then(Value::as_str)
                    .is_none_or(|value| enum_string(authority.state) == value)
        });
        Ok(paginate_values(
            entries.into_iter().map(authority_value).collect(),
            &input,
        ))
    }

    async fn deployment_authority_get(
        &self,
        payload: &[u8],
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let authority_id = required_string(&input, "authorityId")?;
        let authority = self
            .service
            .repository()
            .list_deployment_authorities()
            .await?
            .into_iter()
            .find(|authority| authority.authority_id == authority_id)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("authority not found".to_owned())
            })?;
        let mut value = authority_value(authority);
        value["materialization"] = serde_json::to_value(
            self.service
                .repository()
                .get_materialized_authority(AuthorityKind::Deployment, authority_id)
                .await?
                .map(|replacement| replacement.authority),
        )
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        Ok(json!({ "authority": value }))
    }

    async fn deployment_authority_reconcile(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let authority_id = required_string(&input, "authorityId")?;
        let authority = self
            .service
            .repository()
            .list_deployment_authorities()
            .await?
            .into_iter()
            .find(|authority| authority.authority_id == authority_id)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("authority not found".to_owned())
            })?;
        if input
            .get("expectedVersion")
            .and_then(Value::as_u64)
            .is_some_and(|expected| expected != authority.version)
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let now = now_millis()?;
        let target = AuthorityTarget::new(
            super::auth::AuthorityKind::Deployment,
            authority_id.to_owned(),
        )?;
        let binding = self
            .service
            .repository()
            .get_participant_binding(
                &authority.participant_id,
                &authority.participant_artifact_digest,
            )
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("participant binding not found".to_owned())
            })?;
        let scope = super::auth::AuthorityEvidenceScope {
            target: target.clone(),
            participant_id: binding.participant_id.clone(),
            participant_artifact_digest: binding.artifact_digest.clone(),
            participant_needs_digest: binding.needs_digest.clone(),
        };
        ensure_deployment_resources(
            &self.client,
            self.service.repository(),
            scope.clone(),
            &binding,
            &authority.deployment_id,
            now,
        )
        .await?;
        ensure_authority_dependencies(self.service.repository(), scope, &binding, now).await?;
        self.service
            .authorization()
            .reconcile_authority(&target, now)
            .await?;
        let authority = self
            .service
            .repository()
            .list_deployment_authorities()
            .await?
            .into_iter()
            .find(|authority| authority.authority_id == authority_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let result = json!({ "authority": authority_value(authority) });
        let mut idempotency = rpc_idempotency(
            "Auth.DeploymentAuthority.Reconcile",
            &caller.session.principal_id,
            required_string(&input, "idempotencyKey")?,
            &input,
            now,
        )?;
        idempotency.result = result.clone();
        match self
            .service
            .repository()
            .record_idempotency_result(idempotency)
            .await?
        {
            IdempotentOutcome::Applied(value) | IdempotentOutcome::Replayed(value) => Ok(value),
        }
    }

    async fn authority_plans_list(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let deployment_id = input.get("deploymentId").and_then(Value::as_str);
        let state = input.get("state").and_then(Value::as_str);
        let now = now_millis()?;
        let entries = self
            .service
            .repository()
            .list_authority_proposals()
            .await?
            .into_iter()
            .map(|(proposal, decision)| (effective_proposal(proposal, now), decision))
            .filter(|(proposal, _)| authority_plan_matches(proposal, deployment_id, state))
            .map(|(proposal, decision)| proposal_value(proposal, decision))
            .collect();
        Ok(paginate_values(entries, &input))
    }

    async fn authority_plans_get(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let proposal_id = required_string(&input, "proposalId")?;
        let now = now_millis()?;
        self.service
            .repository()
            .get_authority_proposal(proposal_id)
            .await?
            .map(|(proposal, decision)| {
                json!({ "proposal": proposal_value(effective_proposal(proposal, now), decision) })
            })
            .ok_or_else(|| AuthorizationStateError::InvalidRecord("proposal not found".to_owned()))
    }

    async fn authority_accept(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let proposal_id = required_string(&input, "proposalId")?;
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        let (proposal, _) = self
            .service
            .repository()
            .get_authority_proposal(proposal_id)
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("proposal not found".to_owned())
            })?;
        if proposal.authority_kind != super::auth::AuthorityKind::Deployment {
            return Err(AuthorizationStateError::InvalidRecord(
                "proposal is not deployment authority".to_owned(),
            ));
        }
        let current = self
            .service
            .repository()
            .list_deployment_authorities()
            .await?
            .into_iter()
            .find(|authority| authority.authority_id == proposal.authority_id);
        let deployment_id = proposal.deployment_id.clone().ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("proposal deploymentId is missing".to_owned())
        })?;
        if proposal.authority_id
            != super::auth::deployment_authority_id(&deployment_id, &proposal.participant_id)?
        {
            tracing::warn!(
                proposal_id = %proposal.proposal_id,
                authority_id = %proposal.authority_id,
                deployment_id = %deployment_id,
                participant_id = %proposal.participant_id,
                "authority proposal identity conflict"
            );
            return Err(AuthorizationStateError::StorageConflict);
        }
        let binding = self
            .service
            .repository()
            .get_participant_binding(
                &proposal.participant_id,
                &proposal.participant_artifact_digest,
            )
            .await?
            .filter(|binding| binding.needs_digest == proposal.participant_needs_digest)
            .ok_or(AuthorizationStateError::ParticipantMissing)?;
        let participant_kind = binding.participant_kind;
        let now = now_millis()?;
        let reason = nullable_string(&input, "reason")?;
        let desired = DesiredAuthorityRecord::Deployment(DeploymentAuthorityRecord {
            authority_id: proposal.authority_id.clone(),
            deployment_id: deployment_id.clone(),
            participant_id: proposal.participant_id.clone(),
            participant_kind,
            participant_artifact_digest: proposal.participant_artifact_digest.clone(),
            accepted_needs_digest: proposal.participant_needs_digest.clone(),
            desired_grant_set: proposal.proposed_grant_set.clone(),
            desired_capabilities: proposal.proposed_capabilities.clone(),
            state: AuthorityState::Accepted,
            version: current
                .as_ref()
                .map_or(1, |authority| authority.version + 1),
            created_at: current
                .as_ref()
                .map_or(now, |authority| authority.created_at),
            updated_at: now,
            expires_at: current.as_ref().and_then(|authority| authority.expires_at),
            decision: Some(AuthorityDecision {
                decided_at: now,
                decided_by: caller.session.principal_id.clone(),
                reason: reason.clone(),
            }),
        });
        self.service
            .decide_authority_proposal(DecideAuthorityProposalInput {
                proposal_id: proposal_id.to_owned(),
                expected_version: proposal.version,
                expected_base_authority_version: Some(
                    match input.get("expectedBaseAuthorityVersion") {
                        Some(Value::Null) => None,
                        Some(value) => Some(value.as_u64().ok_or_else(|| {
                            AuthorizationStateError::InvalidRecord(
                            "expectedBaseAuthorityVersion must be a non-negative integer or null"
                                .to_owned(),
                        )
                        })?),
                        None => {
                            return Err(AuthorizationStateError::InvalidRecord(
                                "expectedBaseAuthorityVersion is required".to_owned(),
                            ));
                        }
                    },
                ),
                outcome: AuthorityDecisionOutcome::Accepted,
                decided_by: caller.session.principal_id.clone(),
                reason,
                desired_authority: Some(desired),
                decided_at: now,
                idempotency: rpc_idempotency(
                    "Auth.DeploymentAuthority.Accept",
                    &caller.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let target = AuthorityTarget::new(AuthorityKind::Deployment, &proposal.authority_id)?;
        ensure_deployment_resources(
            &self.client,
            self.service.repository(),
            super::auth::AuthorityEvidenceScope {
                target: target.clone(),
                participant_id: binding.participant_id.clone(),
                participant_artifact_digest: binding.artifact_digest.clone(),
                participant_needs_digest: binding.needs_digest.clone(),
            },
            &binding,
            &deployment_id,
            now,
        )
        .await?;
        ensure_authority_dependencies(
            self.service.repository(),
            super::auth::AuthorityEvidenceScope {
                target: target.clone(),
                participant_id: binding.participant_id.clone(),
                participant_artifact_digest: binding.artifact_digest.clone(),
                participant_needs_digest: binding.needs_digest.clone(),
            },
            &binding,
            now,
        )
        .await?;
        self.service
            .authorization()
            .reconcile_authority(&target, now)
            .await?;
        let (proposal, decision) = self
            .service
            .repository()
            .get_authority_proposal(proposal_id)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        let authority = self
            .service
            .repository()
            .list_deployment_authorities()
            .await?
            .into_iter()
            .find(|authority| authority.authority_id == proposal.authority_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        Ok(json!({
            "proposal": proposal_value(proposal, decision),
            "authority": authority_value(authority),
        }))
    }

    async fn authority_reject(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let proposal_id = required_string(&input, "proposalId")?;
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        let (proposal, _) = self
            .service
            .repository()
            .get_authority_proposal(proposal_id)
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("proposal not found".to_owned())
            })?;
        let now = now_millis()?;
        self.service
            .decide_authority_proposal(DecideAuthorityProposalInput {
                proposal_id: proposal_id.to_owned(),
                expected_version: proposal.version,
                expected_base_authority_version: None,
                outcome: AuthorityDecisionOutcome::Rejected,
                decided_by: caller.session.principal_id.clone(),
                reason: nullable_string(&input, "reason")?,
                desired_authority: None,
                decided_at: now,
                idempotency: rpc_idempotency(
                    "Auth.DeploymentAuthority.Reject",
                    &caller.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let (proposal, decision) = self
            .service
            .repository()
            .get_authority_proposal(proposal_id)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        Ok(json!({ "proposal": proposal_value(proposal, decision) }))
    }

    async fn sessions_me(
        &self,
        validated: ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let user = if validated.session.principal_kind == PrincipalKind::User {
            let (principal, profile) = self
                .service
                .repository()
                .get_user_account(&validated.session.principal_id)
                .await?
                .ok_or(AuthorizationStateError::PrincipalMissing)?;
            let mut value = user_value(UserAccount { principal, profile });
            value["capabilities"] = json!(effective_capabilities(&validated.authorization));
            Some(value)
        } else {
            None
        };
        let binding = self
            .service
            .repository()
            .get_session_runtime_binding(&validated.session.session_id)
            .await?;
        Ok(json!({
            "session": validated.session,
            "user": user,
            "deploymentId": binding.as_ref().map(|binding| &binding.deployment_id),
            "instanceId": binding.as_ref().map(|binding| &binding.instance_id),
        }))
    }

    async fn connections_list(
        &self,
        payload: &[u8],
        _validated: ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let session_id = input.get("sessionId").and_then(Value::as_str);
        let mut entries = self.ephemeral.list_connection_presence(session_id).await?;
        entries.sort_by(|left, right| left.connection_id.cmp(&right.connection_id));
        let limit = input
            .get("limit")
            .and_then(Value::as_i64)
            .unwrap_or(100)
            .clamp(1, 500) as usize;
        let offset = input
            .get("cursor")
            .and_then(Value::as_str)
            .and_then(|cursor| cursor.parse::<usize>().ok())
            .unwrap_or(0);
        let next_cursor = (offset + limit < entries.len()).then(|| (offset + limit).to_string());
        let entries = entries
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(connection_value)
            .collect::<Vec<_>>();
        Ok(json!({ "entries": entries, "nextCursor": next_cursor }))
    }

    async fn portals_list(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let mut entries = Vec::new();
        for portal in self.service.repository().list_login_portals().await? {
            if portal.removed {
                continue;
            }
            let (_, settings) = self
                .service
                .repository()
                .get_login_portal(&portal.portal_id)
                .await?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            entries.push(portal_value(portal, settings));
        }
        Ok(paginate_values(entries, &input))
    }

    async fn portals_get(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let portal_id = required_string(&input, "portalId")?;
        let (portal, settings) = self
            .service
            .repository()
            .get_login_portal(portal_id)
            .await?
            .ok_or_else(|| AuthorizationStateError::InvalidRecord("portal not found".to_owned()))?;
        if portal.removed {
            return Err(AuthorizationStateError::InvalidRecord(
                "portal not found".to_owned(),
            ));
        }
        let routes = self
            .service
            .repository()
            .list_portal_routes()
            .await?
            .into_iter()
            .filter(|route| route.portal_id == portal_id)
            .collect::<Vec<_>>();
        Ok(json!({ "portal": portal_value(portal, settings), "routes": routes }))
    }

    async fn portals_put(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let portal_id = required_string(&input, "portalId")?;
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        let current = self
            .service
            .repository()
            .get_login_portal(portal_id)
            .await?;
        let expected_version = input.get("expectedVersion").and_then(Value::as_u64);
        if current.as_ref().map(|value| value.0.version) != expected_version {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let now = now_millis()?;
        let settings_value = input.get("loginSettings").ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("loginSettings is required".to_owned())
        })?;
        let provider_ids =
            optional_string_array(settings_value, "providers")?.unwrap_or_else(|| {
                current
                    .as_ref()
                    .map_or_else(Vec::new, |value| value.0.provider_ids.clone())
            });
        let version = expected_version.map_or(1, |version| version + 1);
        let portal = LoginPortalRecord {
            portal_id: portal_id.to_owned(),
            display_name: required_string(&input, "displayName")?.to_owned(),
            entry_url: nullable_string(&input, "entryUrl")?,
            builtin: current.as_ref().is_some_and(|value| value.0.builtin),
            disabled: required_bool(&input, "disabled")?,
            removed: false,
            local_registration_enabled: required_bool(settings_value, "localRegistration")?,
            provider_ids: provider_ids.clone(),
            created_at: current.as_ref().map_or(now, |value| value.0.created_at),
            updated_at: now,
            version,
        };
        let settings =
            login_settings_from_value(portal_id, settings_value, provider_ids, now, version)?;
        self.service
            .repository()
            .put_login_portal(LoginPortalMutation {
                portal,
                settings,
                expected_version,
                idempotency: rpc_idempotency(
                    "Auth.Portals.Put",
                    &caller.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let (portal, settings) = self
            .service
            .repository()
            .get_login_portal(portal_id)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        Ok(json!({ "portal": portal_value(portal, settings) }))
    }

    async fn portals_remove(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let portal_id = required_string(&input, "portalId")?;
        let (mut portal, settings) = self
            .service
            .repository()
            .get_login_portal(portal_id)
            .await?
            .ok_or_else(|| AuthorizationStateError::InvalidRecord("portal not found".to_owned()))?;
        if portal.builtin {
            return Err(AuthorizationStateError::InvalidRecord(
                "the built-in portal cannot be removed".to_owned(),
            ));
        }
        let expected_version = required_u64(&input, "expectedVersion")?;
        if portal.version != expected_version {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let now = now_millis()?;
        portal.removed = true;
        portal.updated_at = now;
        portal.version += 1;
        self.service
            .repository()
            .put_login_portal(LoginPortalMutation {
                portal,
                expected_version: Some(expected_version),
                settings,
                idempotency: rpc_idempotency(
                    "Auth.Portals.Remove",
                    &caller.session.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        Ok(json!({ "removed": true }))
    }

    async fn capabilities_list(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let source_api = input.get("sourceApi").and_then(Value::as_str);
        let artifact: Value = serde_json::from_str(include_str!("../../trellis.api.json"))
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        let entries = if source_api.is_none_or(|value| value == "trellis.auth@v1") {
            artifact
                .get("capabilities")
                .and_then(Value::as_object)
                .into_iter()
                .flat_map(|capabilities| capabilities.iter())
                .map(|(capability, definition)| {
                    json!({
                        "capability": capability,
                        "displayName": capability,
                        "description": format!("trellis.auth@v1 {capability} capability"),
                        "allows": definition.get("allows").cloned().unwrap_or_else(|| json!([])),
                        "sourceApi": "trellis.auth@v1",
                    })
                })
                .collect()
        } else {
            Vec::new()
        };
        Ok(paginate_values(entries, &input))
    }

    async fn portal_settings_get(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let portal_id = required_string(&input, "portalId")?;
        let (portal, settings) = self
            .service
            .repository()
            .get_login_portal(portal_id)
            .await?
            .ok_or_else(|| AuthorizationStateError::InvalidRecord("portal not found".to_owned()))?;
        Ok(json!({
            "portalId": portal_id,
            "settings": login_settings_value(&portal, &settings),
            "version": settings.version,
        }))
    }

    async fn portal_settings_update(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let portal_id = required_string(&input, "portalId")?;
        let expected_version = required_u64(&input, "expectedVersion")?;
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        let (mut portal, current_settings) = self
            .service
            .repository()
            .get_login_portal(portal_id)
            .await?
            .ok_or_else(|| AuthorizationStateError::InvalidRecord("portal not found".to_owned()))?;
        if portal.version != expected_version || current_settings.version != expected_version {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let settings_value = input.get("settings").ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("settings is required".to_owned())
        })?;
        let provider_ids = optional_string_array(settings_value, "providers")?
            .unwrap_or_else(|| portal.provider_ids.clone());
        let now = now_millis()?;
        let version = expected_version + 1;
        portal.provider_ids = provider_ids.clone();
        portal.local_registration_enabled = required_bool(settings_value, "localRegistration")?;
        portal.updated_at = now;
        portal.version = version;
        let settings =
            login_settings_from_value(portal_id, settings_value, provider_ids, now, version)?;
        self.service
            .repository()
            .put_login_portal(LoginPortalMutation {
                portal,
                settings,
                expected_version: Some(expected_version),
                idempotency: rpc_idempotency(
                    "Auth.Portals.LoginSettings.Update",
                    &caller.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let (portal, settings) = self
            .service
            .repository()
            .get_login_portal(portal_id)
            .await?
            .ok_or(AuthorizationStateError::StorageConflict)?;
        Ok(json!({
            "portalId": portal_id,
            "settings": login_settings_value(&portal, &settings),
            "version": settings.version,
        }))
    }

    async fn portal_route_put(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let expected_version = input.get("expectedVersion").and_then(Value::as_u64);
        let route_id = input
            .get("routeId")
            .and_then(Value::as_str)
            .map_or_else(|| format!("ptr_{}", Ulid::new()), str::to_owned);
        let current = self
            .service
            .repository()
            .list_portal_routes()
            .await?
            .into_iter()
            .find(|route| route.route_id == route_id);
        if current.as_ref().map(|route| route.version) != expected_version {
            return Err(AuthorizationStateError::StorageConflict);
        }
        let now = now_millis()?;
        let route = PortalRouteRecord {
            route_id: route_id.clone(),
            portal_id: required_string(&input, "portalId")?.to_owned(),
            participant_id: nullable_string(&input, "participantId")?,
            origin: nullable_string(&input, "origin")?,
            deployment_id: nullable_string(&input, "deploymentId")?,
            priority: required_i64(&input, "priority")?,
            created_at: current.as_ref().map_or(now, |route| route.created_at),
            updated_at: now,
            version: expected_version.map_or(1, |version| version + 1),
        };
        self.service
            .repository()
            .put_portal_route(PortalRouteMutation {
                route: route.clone(),
                expected_version,
                idempotency: rpc_idempotency(
                    "Auth.Portals.Routes.Put",
                    &caller.session.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        Ok(json!({ "route": route }))
    }

    async fn portal_route_remove(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let route_id = required_string(&input, "routeId")?;
        let now = now_millis()?;
        self.service
            .repository()
            .remove_portal_route(PortalRouteRemoval {
                route_id: route_id.to_owned(),
                expected_version: required_u64(&input, "expectedVersion")?,
                idempotency: rpc_idempotency(
                    "Auth.Portals.Routes.Remove",
                    &caller.session.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        Ok(json!({ "routeId": route_id, "removed": true }))
    }

    async fn sessions_list(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let mut entries = self.service.repository().list_sessions().await?;
        entries.retain(|session| {
            input
                .get("principalId")
                .and_then(Value::as_str)
                .is_none_or(|value| session.principal_id == value)
                && input
                    .get("participantId")
                    .and_then(Value::as_str)
                    .is_none_or(|value| session.participant_id == value)
                && input
                    .get("state")
                    .and_then(Value::as_str)
                    .is_none_or(|value| {
                        serde_json::to_value(session.state)
                            .ok()
                            .and_then(|state| state.as_str().map(str::to_owned))
                            .as_deref()
                            == Some(value)
                    })
        });
        if let Some(deployment_id) = input.get("deploymentId").and_then(Value::as_str) {
            let mut deployed = Vec::new();
            for session in entries {
                if self
                    .service
                    .repository()
                    .get_session_runtime_binding(&session.session_id)
                    .await?
                    .is_some_and(|binding| binding.deployment_id == deployment_id)
                {
                    deployed.push(session);
                }
            }
            entries = deployed;
        }
        entries.sort_by(|left, right| left.session_id.cmp(&right.session_id));
        Ok(paginate_sessions(entries, &input))
    }

    async fn sessions_logout(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let mut input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let session_id = required_string(&input, "sessionId")?;
        if session_id != caller.session.session_id {
            return Err(AuthorizationStateError::InvalidRecord(
                "logout proof does not belong to the caller session".to_owned(),
            ));
        }
        let mut proof_request = input.clone();
        proof_request
            .as_object_mut()
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("request must be an object".to_owned())
            })?
            .insert("proof".to_owned(), Value::Null);
        let request_digest = session_proof_request_digest_v1(&proof_request)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let proof_input = SessionProofInputV1::session_self_control(
            required_string(&input, "requestId")?,
            required_i64(&input, "issuedAt")?,
            session_id,
            &caller.session.session_key_id,
            request_digest,
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        verify_session_proof_v1(
            &proof_input,
            &parse_session_proof_v1(input.get("proof").ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("proof is required".to_owned())
            })?)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            &caller.session.session_public_key,
            now_millis()?,
            SessionProofPolicyV1::default(),
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let request_id = required_string(&input, "requestId")?.to_owned();
        let object = input.as_object_mut().ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("request must be an object".to_owned())
        })?;
        object.insert("idempotencyKey".to_owned(), Value::String(request_id));
        object.insert("reason".to_owned(), Value::Null);
        self.sessions_revoke(
            &serde_json::to_vec(&input)
                .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?,
            Some(caller),
        )
        .await
    }

    async fn sessions_revoke(
        &self,
        payload: &[u8],
        caller: Option<&ValidatedRequest>,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let session_id = required_string(&input, "sessionId")?;
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        let session = self
            .service
            .repository()
            .get_session(session_id)
            .await?
            .ok_or(AuthorizationStateError::SessionMissing)?;
        let expected_version = input
            .get("expectedVersion")
            .and_then(Value::as_i64)
            .map(u64::try_from)
            .transpose()
            .map_err(|_| AuthorizationStateError::StorageConflict)?
            .unwrap_or(session.version);
        let now = now_seconds()?
            .checked_mul(1_000)
            .ok_or_else(|| AuthorizationStateError::Storage("time overflow".to_owned()))?;
        let request_digest = trellis_protocol::digest_json(&input)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let action = |kind, suffix: &str, payload| PostCommitActionRecord {
            action_id: digest_parts(&[session_id, idempotency_key, suffix]),
            kind,
            payload,
            created_at: now,
            attempts: 0,
            next_attempt_at: now,
            claimed_until: None,
            last_error: None,
        };
        let outcome = self
            .service
            .revoke_session(
                session_id.to_owned(),
                expected_version,
                now,
                IdempotencyResultRecord {
                    scope_key: digest_parts(&["Auth.Sessions.Revoke", session_id]),
                    purpose: "Auth.Sessions.Revoke".to_owned(),
                    signer_id: caller
                        .map(|caller| caller.session.principal_id.clone())
                        .unwrap_or_else(|| "rpc".to_owned()),
                    request_id: idempotency_key.to_owned(),
                    request_digest,
                    result: Value::Null,
                    created_at: now,
                    expires_at: now.saturating_add(86_400_000),
                },
                vec![
                    action(
                        PostCommitActionKind::Event,
                        "event",
                        json!({
                            "eventType": "Auth.Sessions.Revoked",
                            "eventId": format!(
                                "evt_{}",
                                digest_parts(&[session_id, idempotency_key])
                            ),
                            "occurredAt": now,
                            "sessionId": session_id,
                            "principalId": session.principal_id.clone(),
                            "participantId": session.participant_id.clone(),
                            "reason": input.get("reason"),
                            "revokedBy": caller.map(|caller| &caller.session.principal_id),
                        }),
                    ),
                    action(
                        PostCommitActionKind::Kick,
                        "kick",
                        json!({ "sessionId": session_id }),
                    ),
                ],
            )
            .await?;
        let session = match outcome {
            IdempotentOutcome::Applied(session) => session,
            IdempotentOutcome::Replayed(_) => self
                .service
                .repository()
                .get_session(session_id)
                .await?
                .ok_or(AuthorizationStateError::SessionMissing)?,
        };
        let kicked_connections = self.kick_session_connections(session_id).await;
        Ok(json!({
            "session": session,
            "kickedConnections": kicked_connections,
        }))
    }

    async fn connections_kick(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let connection_id = required_string(&input, "connectionId")?;
        let connection = self
            .ephemeral
            .list_connection_presence(None)
            .await?
            .into_iter()
            .find(|connection| connection.connection_id == connection_id)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("connection not found".to_owned())
            })?;
        self.kick_connection(&connection).await?;
        Ok(json!({ "connectionId": connection_id, "kicked": true }))
    }

    async fn kick_session_connections(&self, session_id: &str) -> usize {
        let Ok(connections) = self
            .ephemeral
            .list_connection_presence(Some(session_id))
            .await
        else {
            return 0;
        };
        let mut kicked = 0;
        for connection in connections {
            if self.kick_connection(&connection).await.is_ok() {
                kicked += 1;
            }
        }
        kicked
    }

    async fn kick_connection(
        &self,
        connection: &AuthConnectionPresence,
    ) -> Result<(), AuthorizationStateError> {
        let client_id = connection
            .client_id
            .parse::<u64>()
            .map_err(|_| AuthorizationStateError::InvalidRecord("invalid client id".to_owned()))?;
        self.system_client
            .request(
                format!("$SYS.REQ.SERVER.{}.KICK", connection.server_id),
                Bytes::from(
                    serde_json::to_vec(&json!({ "cid": client_id }))
                        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?,
                ),
            )
            .await
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        Ok(())
    }

    async fn users_create(
        &self,
        payload: &[u8],
        validated: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        let now = now_millis()?;
        let outcome = self
            .service
            .create_user(CreateUserInput {
                name: nullable_string(&input, "name")?,
                email: nullable_string(&input, "email")?,
                image: nullable_string(&input, "image")?,
                created_at: now,
                idempotency: rpc_idempotency(
                    "Auth.Users.Create",
                    &validated.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let account = match outcome {
            IdempotentOutcome::Applied(account) => account,
            IdempotentOutcome::Replayed(result) => {
                let principal_id = required_string(&result, "principalId")?;
                self.service
                    .user(principal_id)
                    .await?
                    .ok_or(AuthorizationStateError::PrincipalMissing)?
            }
        };
        Ok(json!({ "user": user_value(account) }))
    }

    async fn password_change(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let now = now_millis()?;
        let action = PostCommitActionRecord {
            action_id: format!(
                "act_{}",
                digest_parts(&[
                    "Auth.Users.Password.Change",
                    &caller.session.principal_id,
                    required_string(&input, "idempotencyKey")?,
                ])
            ),
            kind: PostCommitActionKind::Kick,
            payload: json!({
                "principalId": caller.session.principal_id,
                "exceptSessionId": caller.session.session_id,
            }),
            created_at: now,
            attempts: 0,
            next_attempt_at: now,
            claimed_until: None,
            last_error: None,
        };
        match self
            .service
            .change_password(
                &caller.session.principal_id,
                &caller.session.session_id,
                required_string(&input, "currentPassword")?,
                required_string(&input, "newPassword")?,
                now,
                rpc_idempotency(
                    "Auth.Users.Password.Change",
                    &caller.session.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                vec![action],
            )
            .await?
        {
            IdempotentOutcome::Applied(revoked) => Ok(json!({
                "changedAt": now,
                "revokedSessionCount": revoked,
            })),
            IdempotentOutcome::Replayed(value) => Ok(value),
        }
    }

    async fn password_reset_create(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let principal_id = required_string(&input, "userId")?;
        self.service
            .user(principal_id)
            .await?
            .ok_or_else(|| AuthorizationStateError::InvalidRecord("user not found".to_owned()))?;
        self.create_rpc_account_flow(
            &input,
            caller,
            AccountFlowKind::PasswordReset,
            principal_id,
            Vec::new(),
            "Auth.Users.PasswordReset.Create",
        )
        .await
    }

    async fn identity_link_create(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let allowed_providers = required_string_array(&input, "allowedProviders")?;
        self.create_rpc_account_flow(
            &input,
            caller,
            AccountFlowKind::IdentityLink,
            &caller.session.principal_id,
            allowed_providers,
            "Auth.Users.IdentityLink.Create",
        )
        .await
    }

    async fn create_rpc_account_flow(
        &self,
        input: &Value,
        caller: &ValidatedRequest,
        kind: AccountFlowKind,
        principal_id: &str,
        allowed_providers: Vec<String>,
        purpose: &str,
    ) -> Result<Value, AuthorizationStateError> {
        let now = now_millis()?;
        let return_target = nullable_string(input, "returnTarget")?;
        let outcome = self
            .service
            .create_account_flow(CreateAccountFlowInput {
                kind,
                target_principal_id: Some(principal_id.to_owned()),
                target_provider_id: None,
                return_location: return_target.clone(),
                payload: json!({ "allowedProviders": allowed_providers }),
                created_at: now,
                expires_at: now.saturating_add(15 * 60_000),
                idempotency: rpc_idempotency(
                    purpose,
                    &caller.session.principal_id,
                    required_string(input, "idempotencyKey")?,
                    input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let flow = match outcome {
            IdempotentOutcome::Applied(flow) => flow,
            IdempotentOutcome::Replayed(_) => {
                return Err(AuthorizationStateError::StorageConflict);
            }
        };
        let kind = match kind {
            AccountFlowKind::PasswordReset => "password_reset",
            AccountFlowKind::IdentityLink => "identity_link",
            _ => {
                return Err(AuthorizationStateError::InvalidRecord(
                    "unsupported RPC account flow".to_owned(),
                ));
            }
        };
        Ok(json!({
            "flow": {
                "flowId": flow.flow_id,
                "kind": kind,
                "targetPrincipalId": principal_id,
                "allowedProviders": allowed_providers,
                "returnTarget": return_target,
                "createdAt": now,
                "expiresAt": flow.expires_at,
                "consumedAt": null,
                "version": 1,
                "completionUrl": format!(
                    "{}/auth/account-flow/{}",
                    self.public_origin.trim_end_matches('/'),
                    flow.token
                ),
            }
        }))
    }

    async fn user_identities_list(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let provider = input.get("providerId").and_then(Value::as_str);
        let entries = self
            .service
            .repository()
            .list_provider_identities(&caller.session.principal_id)
            .await?
            .into_iter()
            .filter(|identity| provider.is_none_or(|value| identity.provider == value))
            .map(|identity| {
                json!({
                    "providerId": identity.provider,
                    "subject": identity.provider_subject,
                    "principalId": identity.principal_id,
                    "username": null,
                    "observedName": null,
                    "observedEmail": null,
                    "createdAt": identity.linked_at,
                    "lastSeenAt": identity.last_seen_at,
                })
            })
            .collect();
        Ok(paginate_values(entries, &input))
    }

    async fn user_identities_unlink(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let now = now_millis()?;
        let mut idempotency = rpc_idempotency(
            "Auth.UserIdentities.Unlink",
            &caller.session.principal_id,
            required_string(&input, "idempotencyKey")?,
            &input,
            now,
        )?;
        idempotency.result = json!({ "unlinked": true });
        let outcome = self
            .service
            .repository()
            .unlink_provider_identity(ProviderIdentityUnlink {
                provider: required_string(&input, "providerId")?.to_owned(),
                provider_subject: required_string(&input, "subject")?.to_owned(),
                principal_id: caller.session.principal_id.clone(),
                idempotency,
                actions: Vec::new(),
            })
            .await?;
        match outcome {
            IdempotentOutcome::Applied(unlinked) => Ok(json!({ "unlinked": unlinked })),
            IdempotentOutcome::Replayed(value) => Ok(value),
        }
    }

    async fn users_get(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let account = self
            .service
            .user(required_string(&input, "userId")?)
            .await?
            .ok_or(AuthorizationStateError::PrincipalMissing)?;
        Ok(json!({ "user": user_value(account) }))
    }

    async fn users_resolve(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let selector = input.get("selector").ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("selector is required".to_owned())
        })?;
        let principal_id = match selector.get("kind").and_then(Value::as_str) {
            Some("user") => required_string(selector, "userId")?.to_owned(),
            Some("provider") => self
                .service
                .repository()
                .get_provider_identity(
                    required_string(selector, "providerId")?,
                    required_string(selector, "providerSubject")?,
                )
                .await?
                .map(|identity| identity.principal_id)
                .ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord("user not found".to_owned())
                })?,
            _ => {
                return Err(AuthorizationStateError::InvalidRecord(
                    "selector kind is invalid".to_owned(),
                ));
            }
        };
        let account =
            self.service.user(&principal_id).await?.ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("user not found".to_owned())
            })?;
        Ok(json!({ "user": user_value(account) }))
    }

    async fn users_list(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let limit = input
            .get("limit")
            .and_then(Value::as_u64)
            .unwrap_or(100)
            .clamp(1, 100) as usize;
        let state = input.get("state").and_then(Value::as_str);
        let mut accounts = self
            .service
            .users(input.get("cursor").and_then(Value::as_str), limit + 1)
            .await?;
        if let Some(state) = state {
            accounts.retain(|account| principal_state(&account.principal) == state);
        }
        let next_cursor =
            (accounts.len() > limit).then(|| accounts[limit - 1].principal.principal_id.clone());
        accounts.truncate(limit);
        Ok(json!({
            "entries": accounts.into_iter().map(user_value).collect::<Vec<_>>(),
            "nextCursor": next_cursor,
        }))
    }

    async fn users_update(
        &self,
        payload: &[u8],
        validated: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let principal_id = required_string(&input, "userId")?;
        let idempotency_key = required_string(&input, "idempotencyKey")?;
        let expected_version = required_u64(&input, "expectedVersion")?;
        let state = match required_string(&input, "state")? {
            "active" => PrincipalState::Active,
            "disabled" => PrincipalState::Disabled,
            _ => {
                return Err(AuthorizationStateError::InvalidRecord(
                    "user state must be active or disabled".to_owned(),
                ));
            }
        };
        let now = now_millis()?;
        let outcome = self
            .service
            .update_user(UpdateUserInput {
                principal_id: principal_id.to_owned(),
                expected_version,
                name: nullable_string(&input, "name")?,
                email: nullable_string(&input, "email")?,
                image: nullable_string(&input, "image")?,
                state,
                updated_at: now,
                idempotency: rpc_idempotency(
                    "Auth.Users.Update",
                    &validated.session.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        let account = match outcome {
            IdempotentOutcome::Applied(account) => account,
            IdempotentOutcome::Replayed(_) => self
                .service
                .user(principal_id)
                .await?
                .ok_or(AuthorizationStateError::PrincipalMissing)?,
        };
        Ok(json!({ "user": user_value(account) }))
    }
}

fn public_rpc_error(subject: &str, error: &AuthorizationStateError) -> Value {
    if subject == "rpc.v1.Auth.Requests.Validate"
        && matches!(error, AuthorizationStateError::SessionMissing)
    {
        return json!({
            "id": format!("err_{}", Ulid::new()),
            "type": "AuthError",
            "message": "The authenticated session was not found.",
            "reason": "session_not_found",
        });
    }
    let (error_type, code, message) = match error {
        AuthorizationStateError::InvalidRecord(_) => {
            ("AuthError", "invalid_request", "The request is invalid.")
        }
        AuthorizationStateError::StorageConflict => (
            "AuthError",
            "conflict",
            "The request conflicts with current authentication state.",
        ),
        AuthorizationStateError::PrincipalMissing
        | AuthorizationStateError::SessionMissing
        | AuthorizationStateError::AuthorityMissing => (
            "AuthError",
            "not_found",
            "The requested authentication record was not found.",
        ),
        error if error.is_expected_denial() => (
            "AuthError",
            "not_authorized",
            "The request is not authorized.",
        ),
        _ => (
            "UnexpectedError",
            "internal_error",
            "The request could not be completed.",
        ),
    };
    if error_type == "AuthError" {
        json!({
            "id": format!("err_{}", Ulid::new()),
            "type": error_type,
            "message": message,
            "reason": code,
        })
    } else {
        json!({
            "id": format!("err_{}", Ulid::new()),
            "type": error_type,
            "message": message,
            "context": { "code": code },
        })
    }
}

fn request_capability_permissions(
    required: &[String],
    authorization: &IssuableAuthorizationState,
    binding: &super::auth::ParticipantBindingRecord,
) -> Result<Option<Vec<PermissionAtomV1>>, AuthorizationStateError> {
    let apis: BTreeMap<String, Value> = serde_json::from_str(&binding.api_artifacts_json)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let mut required_permissions = Vec::new();
    for capability in required {
        let definitions = if let Some((api_id, name)) = capability.rsplit_once("::") {
            apis.get(api_id)
                .or_else(|| {
                    apis.iter()
                        .find(|(id, _)| {
                            id.split_once('@').is_some_and(|(owner, _)| owner == api_id)
                        })
                        .map(|(_, api)| api)
                })
                .and_then(|api| api.get("capabilities"))
                .and_then(|capabilities| {
                    capabilities
                        .get(capability)
                        .or_else(|| capabilities.get(name))
                })
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            apis.values()
                .filter_map(|api| api.get("capabilities"))
                .filter_map(|capabilities| {
                    capabilities.get(capability).or_else(|| {
                        capabilities
                            .as_object()?
                            .iter()
                            .find_map(|(name, definition)| {
                                name.rsplit_once("::")
                                    .is_some_and(|(_, name)| name == capability)
                                    .then_some(definition)
                            })
                    })
                })
                .collect::<Vec<_>>()
        };
        let [definition] = definitions.as_slice() else {
            return Ok(None);
        };
        if !authorization
            .capabilities
            .iter()
            .any(|granted| capability_evidence_matches(granted, capability))
        {
            return Ok(None);
        }
        let Some(allows) = definition.get("allows").and_then(Value::as_array) else {
            return Ok(None);
        };
        for permission in allows {
            let permission: PermissionAtomV1 = serde_json::from_value(permission.clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            if !authorization.grant_set.permissions().contains(&permission) {
                return Ok(None);
            }
            required_permissions.push(permission);
        }
    }
    Ok(Some(
        GrantSetV1::new(required_permissions).permissions().to_vec(),
    ))
}

fn capability_evidence_matches(granted: &str, required: &str) -> bool {
    if granted == required {
        return true;
    }
    let (granted_owner, granted_name) = granted
        .rsplit_once("::")
        .map_or((None, granted), |(owner, name)| (Some(owner), name));
    let (required_owner, required_name) = required
        .rsplit_once("::")
        .map_or((None, required), |(owner, name)| (Some(owner), name));
    if granted_name != required_name {
        return false;
    }
    match required_owner {
        None => true,
        Some(required_owner) => granted_owner.is_some_and(|granted_owner| {
            granted_owner
                .split_once('@')
                .map_or(granted_owner, |(id, _)| id)
                == required_owner
                    .split_once('@')
                    .map_or(required_owner, |(id, _)| id)
        }),
    }
}

fn request_subject_allowed(
    subject: &str,
    capability_permissions: &[PermissionAtomV1],
    authorization: &IssuableAuthorizationState,
    binding: &ParticipantBindingRecord,
) -> Result<bool, AuthorizationStateError> {
    let apis = binding_apis(binding)?;
    if let Some(direction) = transfer_subject_direction(subject) {
        return transfer_subject_allowed(direction, &apis, authorization);
    }
    let mut candidates = Vec::new();
    let mut control_candidates = Vec::new();
    let mut control_operations = 0;
    for (api_id, api) in &apis {
        let subjects = api
            .derived_subjects()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        for (name, expected) in &subjects.rpc {
            if expected == subject {
                candidates.push(resolved_permission(
                    PermissionTargetV1::api_surface(api_id, ApiSurfaceKindV1::Rpc, name),
                    PermissionActionV1::Call,
                )?);
            }
        }
        for (name, expected) in &subjects.operations {
            if expected == subject {
                candidates.push(resolved_permission(
                    PermissionTargetV1::api_surface(api_id, ApiSurfaceKindV1::Operation, name),
                    PermissionActionV1::Invoke,
                )?);
            } else if subject == format!("{expected}.control") {
                control_operations += 1;
                if capability_permissions.is_empty() {
                    control_candidates.push(resolved_permission(
                        PermissionTargetV1::api_surface(api_id, ApiSurfaceKindV1::Operation, name),
                        PermissionActionV1::Observe,
                    )?);
                } else {
                    control_candidates.extend(capability_permissions.iter().filter_map(
                        |permission| match permission.target() {
                            PermissionTargetV1::ApiSurface {
                                api,
                                surface: ApiSurfaceKindV1::Operation,
                                name: operation,
                            } if api == api_id
                                && operation == name
                                && matches!(
                                    permission.action(),
                                    PermissionActionV1::Observe
                                        | PermissionActionV1::Cancel
                                        | PermissionActionV1::Control
                                ) =>
                            {
                                Some(permission.clone())
                            }
                            PermissionTargetV1::OperationSignal { api, operation, .. }
                                if api == api_id && operation == name =>
                            {
                                Some(permission.clone())
                            }
                            _ => None,
                        },
                    ));
                }
            }
        }
        for (name, expected) in &subjects.feeds {
            if expected == subject {
                candidates.push(resolved_permission(
                    PermissionTargetV1::api_surface(api_id, ApiSurfaceKindV1::Feed, name),
                    PermissionActionV1::Subscribe,
                )?);
            }
        }
    }
    let state_action = match subject {
        "rpc.v1.State.Get" | "rpc.v1.State.List" => Some(PermissionActionV1::Read),
        "rpc.v1.State.Put" => Some(PermissionActionV1::Write),
        "rpc.v1.State.Delete" => Some(PermissionActionV1::Delete),
        _ => None,
    };
    if let Some(action) = state_action {
        candidates.extend(
            capability_permissions
                .iter()
                .filter(|permission| {
                    permission.action() == action
                        && matches!(
                            permission.target(),
                            PermissionTargetV1::ApiSurface {
                                surface: ApiSurfaceKindV1::State,
                                ..
                            }
                        )
                })
                .cloned(),
        );
    }
    if control_operations > 0 {
        if control_operations != 1 || control_candidates.is_empty() {
            return Ok(false);
        }
        return Ok(control_candidates
            .iter()
            .all(|permission| authorization.grant_set.permissions().contains(permission)));
    }
    let candidates = GrantSetV1::new(candidates);
    Ok(matches!(candidates.permissions(), [permission]
        if authorization.grant_set.permissions().contains(permission)))
}

fn transfer_subject_direction(subject: &str) -> Option<&str> {
    let parts = subject.split('.').collect::<Vec<_>>();
    match parts.as_slice() {
        ["transfer", "v1", direction @ ("upload" | "download"), session, transfer]
            if !session.is_empty() && !transfer.is_empty() =>
        {
            Some(direction)
        }
        _ => None,
    }
}

fn transfer_subject_allowed(
    direction: &str,
    apis: &BTreeMap<String, ApiArtifactV1>,
    authorization: &IssuableAuthorizationState,
) -> Result<bool, AuthorizationStateError> {
    for permission in authorization.grant_set.permissions() {
        let Some((api_id, surface, name)) = permission.target().as_api_surface() else {
            continue;
        };
        let expected = match direction {
            "upload"
                if surface == ApiSurfaceKindV1::Operation
                    && permission.action() == PermissionActionV1::Invoke =>
            {
                "send"
            }
            "download"
                if surface == ApiSurfaceKindV1::Rpc
                    && permission.action() == PermissionActionV1::Call =>
            {
                "receive"
            }
            _ => continue,
        };
        let Some(api) = apis.get(api_id) else {
            continue;
        };
        let value = api
            .normalized_value()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let section = match surface {
            ApiSurfaceKindV1::Operation => "operations",
            ApiSurfaceKindV1::Rpc => "rpc",
            _ => continue,
        };
        if value[section][name]["transfer"]["direction"].as_str() == Some(expected) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn event_subject_allowed(
    subject: &str,
    authorization: &IssuableAuthorizationState,
    binding: &ParticipantBindingRecord,
) -> Result<bool, AuthorizationStateError> {
    let mut candidates = Vec::new();
    for (api_id, api) in binding_apis(binding)? {
        let subjects = api
            .derived_subjects()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        for (name, expected) in subjects.events {
            if nats_subject_matches(&expected.wildcard, subject) {
                candidates.push(resolved_permission(
                    PermissionTargetV1::api_surface(&api_id, ApiSurfaceKindV1::Event, &name),
                    PermissionActionV1::Publish,
                )?);
            }
        }
    }
    let candidates = GrantSetV1::new(candidates);
    Ok(matches!(candidates.permissions(), [permission]
        if authorization.grant_set.permissions().contains(permission)))
}

fn binding_apis(
    binding: &ParticipantBindingRecord,
) -> Result<BTreeMap<String, ApiArtifactV1>, AuthorizationStateError> {
    let values: BTreeMap<String, Value> = serde_json::from_str(&binding.api_artifacts_json)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    values
        .into_iter()
        .map(|(api_id, value)| {
            let api = parse_api_v1(&value)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            if api.id() != api_id {
                return Err(AuthorizationStateError::InvalidRecord(
                    "API artifact map key does not match artifact ID".to_owned(),
                ));
            }
            Ok((api_id, api))
        })
        .collect()
}

fn resolved_permission(
    target: Result<PermissionTargetV1, ProtocolError>,
    action: PermissionActionV1,
) -> Result<PermissionAtomV1, AuthorizationStateError> {
    let target =
        target.map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    PermissionAtomV1::new(target, action)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))
}

fn request_from_headers(
    subject: &str,
    message: &async_nats::Message,
) -> Result<AuthRequestsValidateRequest, AuthorizationStateError> {
    let headers = message.headers.as_ref().ok_or_else(|| {
        AuthorizationStateError::InvalidRecord("request headers missing".to_owned())
    })?;
    let value = |name: &str| {
        headers
            .get(name)
            .map(|value| value.as_str().to_owned())
            .ok_or_else(|| AuthorizationStateError::InvalidRecord(format!("{name} header missing")))
    };
    Ok(AuthRequestsValidateRequest {
        session_key: value("session-key")?,
        proof: value("proof")?,
        subject: subject.to_owned(),
        payload_hash: URL_SAFE_NO_PAD.encode(Sha256::digest(&message.payload)),
        iat: value("iat")?
            .parse()
            .map_err(|_| AuthorizationStateError::InvalidRecord("invalid iat header".to_owned()))?,
        request_id: value("request-id")?,
        capabilities: Vec::new(),
    })
}

fn verify_request_proof(
    request: &AuthRequestsValidateRequest,
) -> Result<(), AuthorizationStateError> {
    let public_key: [u8; 32] = URL_SAFE_NO_PAD
        .decode(&request.session_key)
        .ok()
        .and_then(|value| value.try_into().ok())
        .ok_or_else(|| AuthorizationStateError::InvalidRecord("invalid session key".to_owned()))?;
    let signature: [u8; 64] = URL_SAFE_NO_PAD
        .decode(&request.proof)
        .ok()
        .and_then(|value| value.try_into().ok())
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("invalid request proof".to_owned())
        })?;
    let payload_hash = URL_SAFE_NO_PAD
        .decode(&request.payload_hash)
        .map_err(|_| AuthorizationStateError::InvalidRecord("invalid payload hash".to_owned()))?;
    if payload_hash.len() != 32 {
        return denied("payload hash must encode 32 bytes");
    }
    let mut input = Vec::new();
    for value in [
        request.session_key.as_bytes(),
        request.subject.as_bytes(),
        payload_hash.as_slice(),
        request.iat.to_string().as_bytes(),
        request.request_id.as_bytes(),
    ] {
        input.extend_from_slice(&(value.len() as u32).to_be_bytes());
        input.extend_from_slice(value);
    }
    VerifyingKey::from_bytes(&public_key)
        .map_err(|_| AuthorizationStateError::InvalidRecord("invalid session key".to_owned()))?
        .verify(&Sha256::digest(input), &Signature::from_bytes(&signature))
        .map_err(|_| AuthorizationStateError::InvalidRecord("invalid request proof".to_owned()))
}

fn verify_event_proof(request: &AuthEventsValidateRequest) -> Result<(), AuthorizationStateError> {
    let public_key: [u8; 32] = URL_SAFE_NO_PAD
        .decode(&request.session_key)
        .ok()
        .and_then(|value| value.try_into().ok())
        .ok_or_else(|| AuthorizationStateError::InvalidRecord("invalid session key".to_owned()))?;
    let signature: [u8; 64] = URL_SAFE_NO_PAD
        .decode(&request.proof)
        .ok()
        .and_then(|value| value.try_into().ok())
        .ok_or_else(|| AuthorizationStateError::InvalidRecord("invalid event proof".to_owned()))?;
    let payload_hash = URL_SAFE_NO_PAD
        .decode(&request.payload_hash)
        .map_err(|_| AuthorizationStateError::InvalidRecord("invalid payload hash".to_owned()))?;
    if payload_hash.len() != 32 {
        return denied("payload hash must encode 32 bytes");
    }
    let mut input = Vec::new();
    for value in [
        b"trellis-event-proof-v1".as_slice(),
        request.session_key.as_bytes(),
        request.subject.as_bytes(),
        payload_hash.as_slice(),
        request.event_id.as_bytes(),
        request.event_time.as_bytes(),
    ] {
        input.extend_from_slice(&(value.len() as u32).to_be_bytes());
        input.extend_from_slice(value);
    }
    VerifyingKey::from_bytes(&public_key)
        .map_err(|_| AuthorizationStateError::InvalidRecord("invalid session key".to_owned()))?
        .verify(&Sha256::digest(input), &Signature::from_bytes(&signature))
        .map_err(|_| AuthorizationStateError::InvalidRecord("invalid event proof".to_owned()))
}

fn nats_subject_matches(pattern: &str, subject: &str) -> bool {
    let mut pattern = pattern.split('.');
    let mut subject = subject.split('.');
    loop {
        match (pattern.next(), subject.next()) {
            (Some(">"), _) => return true,
            (Some("*"), Some(_)) => {}
            (Some(expected), Some(actual)) if expected == actual => {}
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn event_denied(status: AuthEventValidationStatus) -> AuthEventsValidateResponse {
    AuthEventsValidateResponse {
        allowed: false,
        status,
        caller: None,
        publisher: None,
    }
}

fn caller_value(
    authorization: &IssuableAuthorizationState,
) -> Result<Value, AuthorizationStateError> {
    let capabilities = authorization.capabilities.clone();
    Ok(match authorization.principal.kind {
        AuthorizationPrincipalKindV1::User => {
            let participant_kind = match authorization.participant.kind {
                ParticipantKindV1::Agent => "agent",
                _ => "app",
            };
            let last_auth = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
            json!({
                "type": "user",
                "participantKind": participant_kind,
                "userId": authorization.principal.id,
                "identity": {
                    "identityId": authorization.principal.id,
                    "provider": "trellis",
                    "subject": authorization.principal.id,
                },
                "active": true,
                "name": authorization.principal.id,
                "email": "",
                "capabilities": capabilities,
                "lastAuth": last_auth,
            })
        }
        AuthorizationPrincipalKindV1::Service => json!({
            "type": "service",
            "id": authorization.principal.id,
            "name": authorization.participant.id,
            "active": true,
            "capabilities": capabilities,
        }),
        AuthorizationPrincipalKindV1::Device => json!({
            "type": "device",
            "deviceId": authorization.principal.id,
            "deviceType": authorization.participant.id,
            "runtimePublicKey": authorization.session_public_key,
            "deploymentId": authorization.deployment_id,
            "active": true,
            "capabilities": capabilities,
        }),
    })
}

fn effective_capabilities(authorization: &IssuableAuthorizationState) -> Vec<String> {
    if authorization
        .grant_set
        .permissions()
        .iter()
        .any(|permission| match permission.target() {
            PermissionTargetV1::ApiSurface { api, name, .. } => {
                api == "trellis.auth@v1" && name == "Auth.Users.List"
            }
            _ => false,
        })
    {
        vec!["admin".to_owned()]
    } else {
        Vec::new()
    }
}

fn connection_value(connection: AuthConnectionPresence) -> Value {
    json!({
        "connectionId": connection.connection_id,
        "sessionId": connection.session_id,
        "serverId": connection.server_id,
        "clientId": connection.client_id,
        "userNkey": connection.user_nkey,
        "remoteAddress": connection.remote_address,
        "connectedAt": connection.connected_at,
        "lastSeenAt": connection.last_seen_at,
    })
}

fn authority_value(authority: DeploymentAuthorityRecord) -> Value {
    json!({
        "authorityId": authority.authority_id,
        "participantId": authority.participant_id,
        "participantArtifactDigest": authority.participant_artifact_digest,
        "acceptedNeedsDigest": authority.accepted_needs_digest,
        "desiredGrantSet": authority.desired_grant_set,
        "desiredCapabilities": authority.desired_capabilities,
        "state": authority.state,
        "version": authority.version,
        "createdAt": authority.created_at,
        "updatedAt": authority.updated_at,
        "expiresAt": authority.expires_at,
        "decision": authority.decision,
        "materialization": null,
        "kind": "deployment",
        "deploymentId": authority.deployment_id,
        "participantKind": authority.participant_kind,
    })
}

fn identity_authority_value(authority: IdentityAuthorityRecord) -> Value {
    json!({
        "authorityId": authority.authority_id,
        "participantId": authority.participant_id,
        "participantArtifactDigest": authority.participant_artifact_digest,
        "acceptedNeedsDigest": authority.accepted_needs_digest,
        "desiredGrantSet": authority.desired_grant_set,
        "desiredCapabilities": authority.desired_capabilities,
        "state": authority.state,
        "version": authority.version,
        "createdAt": authority.created_at,
        "updatedAt": authority.updated_at,
        "expiresAt": authority.expires_at,
        "decision": authority.decision,
        "materialization": null,
        "kind": "identity",
        "principalId": authority.principal_id,
    })
}

fn service_instance_value(
    instance: super::auth::RuntimeInstanceRecord,
    identity: ProvisionedIdentityRecord,
    profile: &DeploymentProfileRecord,
) -> Value {
    json!({
        "instanceId": instance.instance_id,
        "deploymentId": instance.deployment_id,
        "principalId": instance.principal_id,
        "identityPublicKey": identity.identity_public_key,
        "identityKeyId": identity.identity_key_id,
        "participantId": profile.participant_id,
        "state": instance.state,
        "createdAt": instance.created_at,
        "updatedAt": instance.updated_at,
        "version": instance.version,
    })
}

fn activation_review_value(review: DeviceActivationReviewRecord) -> Value {
    json!({
        "reviewId": review.review_id,
        "deploymentId": review.deployment_id,
        "instanceId": review.instance_id,
        "devicePrincipalId": review.principal_id,
        "state": review.state,
        "confirmationCode": review.payload.get("confirmationCode"),
        "requestedAt": review.requested_at,
        "expiresAt": review.payload.get("expiresAt"),
        "decidedAt": review.decided_at,
        "decidedBy": review.decided_by,
        "reason": review.reason,
        "version": review.version,
    })
}

fn proposal_value(
    proposal: AuthorityProposalRecord,
    decision: Option<AuthorityDecisionRecord>,
) -> Value {
    let subject_id = proposal
        .deployment_id
        .clone()
        .or_else(|| {
            proposal
                .payload
                .get("subjectId")
                .or_else(|| proposal.payload.get("deploymentId"))
                .and_then(Value::as_str)
                .map(String::from)
        })
        .unwrap_or_else(|| proposal.authority_id.clone());
    let reasons = proposal
        .payload
        .get("reasons")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let base_authority_version = proposal.payload.get("baseAuthorityVersion").cloned();
    json!({
        "proposalId": proposal.proposal_id,
        "authorityKind": proposal.authority_kind,
        "subjectId": subject_id,
        "participantId": proposal.participant_id,
        "participantArtifactDigest": proposal.participant_artifact_digest,
        "participantNeedsDigest": proposal.participant_needs_digest,
        "proposedGrantSet": proposal.proposed_grant_set,
        "proposedCapabilities": proposal.proposed_capabilities,
        "classification": proposal.proposal_kind,
        "state": proposal.state,
        "reasons": reasons,
        "createdAt": proposal.created_at,
        "expiresAt": proposal.expires_at,
        "decisionAt": decision.as_ref().map(|value| value.decided_at),
        "decisionBy": decision.as_ref().map(|value| value.decided_by.clone()),
        "decisionReason": decision.and_then(|value| value.reason),
        "baseAuthorityVersion": base_authority_version,
    })
}

fn effective_proposal(mut proposal: AuthorityProposalRecord, now: i64) -> AuthorityProposalRecord {
    if proposal.state == AuthorityProposalState::Pending
        && proposal
            .expires_at
            .is_some_and(|expires_at| expires_at <= now)
    {
        proposal.state = AuthorityProposalState::Expired;
    }
    proposal
}

fn authority_plan_matches(
    proposal: &AuthorityProposalRecord,
    deployment_id: Option<&str>,
    state: Option<&str>,
) -> bool {
    proposal.authority_kind == AuthorityKind::Deployment
        && deployment_id.is_none_or(|id| proposal.deployment_id.as_deref() == Some(id))
        && state.is_none_or(|value| enum_string(proposal.state) == value)
}

fn enum_string(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn paginate_values(entries: Vec<Value>, input: &Value) -> Value {
    let limit = input
        .get("limit")
        .and_then(Value::as_i64)
        .unwrap_or(100)
        .clamp(1, 500) as usize;
    let offset = input
        .get("cursor")
        .and_then(Value::as_str)
        .and_then(|cursor| cursor.parse::<usize>().ok())
        .unwrap_or(0);
    let next_cursor = (offset + limit < entries.len()).then(|| (offset + limit).to_string());
    json!({
        "entries": entries.into_iter().skip(offset).take(limit).collect::<Vec<_>>(),
        "nextCursor": next_cursor,
    })
}

fn paginate_sessions(entries: Vec<SessionRecord>, input: &Value) -> Value {
    let limit = input
        .get("limit")
        .and_then(Value::as_i64)
        .unwrap_or(100)
        .clamp(1, 500) as usize;
    let offset = input
        .get("cursor")
        .and_then(Value::as_str)
        .and_then(|cursor| cursor.parse::<usize>().ok())
        .unwrap_or(0);
    let next_cursor = (offset + limit < entries.len()).then(|| (offset + limit).to_string());
    json!({
        "entries": entries.into_iter().skip(offset).take(limit).collect::<Vec<_>>(),
        "nextCursor": next_cursor,
    })
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, AuthorizationStateError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AuthorizationStateError::InvalidRecord(format!("{key} is required")))
}

fn nullable_string(value: &Value, key: &str) -> Result<Option<String>, AuthorizationStateError> {
    match value.get(key) {
        Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        _ => Err(AuthorizationStateError::InvalidRecord(format!(
            "{key} must be a string or null"
        ))),
    }
}

fn required_u64(value: &Value, key: &str) -> Result<u64, AuthorizationStateError> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| AuthorizationStateError::InvalidRecord(format!("{key} is required")))
}

fn required_i64(value: &Value, key: &str) -> Result<i64, AuthorizationStateError> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| AuthorizationStateError::InvalidRecord(format!("{key} is required")))
}

fn required_bool(value: &Value, key: &str) -> Result<bool, AuthorizationStateError> {
    value
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| AuthorizationStateError::InvalidRecord(format!("{key} is required")))
}

fn optional_string_array(
    value: &Value,
    key: &str,
) -> Result<Option<Vec<String>>, AuthorizationStateError> {
    match value.get(key) {
        Some(Value::Null) => Ok(None),
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value.as_str().map(str::to_owned).ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord(format!("{key} must contain strings"))
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        _ => Err(AuthorizationStateError::InvalidRecord(format!(
            "{key} must be an array or null"
        ))),
    }
}

fn required_string_array(value: &Value, key: &str) -> Result<Vec<String>, AuthorizationStateError> {
    optional_string_array(value, key)?
        .ok_or_else(|| AuthorizationStateError::InvalidRecord(format!("{key} is required")))
}

fn login_settings_from_value(
    portal_id: &str,
    value: &Value,
    provider_ids: Vec<String>,
    now: i64,
    version: u64,
) -> Result<LoginSettingsRecord, AuthorizationStateError> {
    Ok(LoginSettingsRecord {
        portal_id: portal_id.to_owned(),
        default_provider_id: (provider_ids.len() == 1).then(|| provider_ids[0].clone()),
        local_login_enabled: required_bool(value, "localLogin")?,
        federated_registration_enabled: required_bool(value, "federatedRegistration")?,
        provider_selection_enabled: provider_ids.len() > 1,
        updated_at: now,
        version,
    })
}

fn login_settings_value(portal: &LoginPortalRecord, settings: &LoginSettingsRecord) -> Value {
    json!({
        "providers": portal.provider_ids,
        "localLogin": settings.local_login_enabled,
        "localRegistration": portal.local_registration_enabled,
        "federatedRegistration": settings.federated_registration_enabled,
    })
}

fn portal_value(portal: LoginPortalRecord, settings: LoginSettingsRecord) -> Value {
    json!({
        "portalId": portal.portal_id,
        "displayName": portal.display_name,
        "entryUrl": portal.entry_url,
        "builtIn": portal.builtin,
        "disabled": portal.disabled,
        "loginSettings": login_settings_value(&portal, &settings),
        "createdAt": portal.created_at,
        "updatedAt": portal.updated_at,
        "version": portal.version,
    })
}

fn rpc_idempotency(
    purpose: &str,
    signer_id: &str,
    request_id: &str,
    input: &Value,
    now: i64,
) -> Result<IdempotencyResultRecord, AuthorizationStateError> {
    Ok(IdempotencyResultRecord {
        scope_key: digest_parts(&[purpose, signer_id, request_id]),
        purpose: purpose.to_owned(),
        signer_id: signer_id.to_owned(),
        request_id: request_id.to_owned(),
        request_digest: trellis_protocol::digest_json(input)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        result: Value::Null,
        created_at: now,
        expires_at: now.saturating_add(86_400_000),
    })
}

fn principal_state(principal: &super::auth::PrincipalRecord) -> &'static str {
    match principal.state {
        PrincipalState::Active => "active",
        PrincipalState::Disabled => "disabled",
        PrincipalState::Revoked => "revoked",
    }
}

fn user_value(account: UserAccount) -> Value {
    json!({
        "userId": account.principal.principal_id,
        "principalId": account.profile.principal_id,
        "state": principal_state(&account.principal),
        "name": account.profile.display_name,
        "email": account.profile.email,
        "image": account.profile.image_url,
        "createdAt": account.principal.created_at,
        "updatedAt": account.profile.updated_at,
        "disabledAt": account.principal.disabled_at,
        "revokedAt": account.principal.revoked_at,
        "version": account.principal.version,
    })
}

fn digest_parts(parts: &[&str]) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update((part.len() as u32).to_be_bytes());
        hash.update(part.as_bytes());
    }
    URL_SAFE_NO_PAD.encode(hash.finalize())
}

fn now_seconds() -> Result<i64, AuthorizationStateError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?
        .as_secs()
        .try_into()
        .map_err(|_| AuthorizationStateError::Storage("current time overflow".to_owned()))
}

fn now_millis() -> Result<i64, AuthorizationStateError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?
        .as_millis()
        .try_into()
        .map_err(|_| AuthorizationStateError::Storage("current time overflow".to_owned()))
}

fn denied<T>(message: impl Into<String>) -> Result<T, AuthorizationStateError> {
    Err(AuthorizationStateError::InvalidRecord(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_rs::client::SessionAuth;

    #[test]
    fn initial_authority_plan_filters_without_an_accepted_authority() {
        let proposal = AuthorityProposalRecord {
            proposal_id: "apr_01".to_owned(),
            authority_kind: AuthorityKind::Deployment,
            authority_id: "dau_test".to_owned(),
            deployment_id: Some("dep_test".to_owned()),
            proposal_kind: AuthorityProposalKind::Initial,
            participant_id: "participant.test@v1".to_owned(),
            participant_artifact_digest: "A".repeat(43),
            participant_needs_digest: "B".repeat(43),
            proposed_grant_set: GrantSetV1::new(Vec::new()),
            proposed_capabilities: Vec::new(),
            proposal_digest: "C".repeat(43),
            payload: json!({
                "deploymentId": "dep_test",
                "subjectId": "dep_test",
                "baseAuthorityVersion": null,
            }),
            state: AuthorityProposalState::Pending,
            created_at: 100,
            expires_at: Some(200),
            superseded_at: None,
            version: 1,
        };
        assert!(authority_plan_matches(
            &proposal,
            Some("dep_test"),
            Some("pending")
        ));
        assert!(!authority_plan_matches(&proposal, Some("dep_other"), None));
        let expired = effective_proposal(proposal.clone(), 200);
        assert_eq!(expired.state, AuthorityProposalState::Expired);
        assert!(authority_plan_matches(
            &expired,
            Some("dep_test"),
            Some("expired")
        ));
        assert_eq!(proposal_value(proposal, None)["subjectId"], "dep_test");
    }

    fn issuable_state(
        binding: &ParticipantBindingRecord,
        grant_set: GrantSetV1,
        capabilities: Vec<String>,
    ) -> IssuableAuthorizationState {
        IssuableAuthorizationState {
            principal: trellis_protocol::AuthorizationPrincipalV1 {
                kind: AuthorizationPrincipalKindV1::Service,
                id: "svc-1".to_owned(),
            },
            session_id: "ses-1".to_owned(),
            session_public_key: "session-key".to_owned(),
            session_key_id: "session-key-id".to_owned(),
            inbox_prefix: "_INBOX.ses-1".to_owned(),
            participant: trellis_protocol::AuthorizationParticipantV1 {
                kind: binding.participant_kind,
                id: binding.participant_id.clone(),
                artifact_digest: binding.artifact_digest.clone(),
                needs_digest: binding.needs_digest.clone(),
            },
            authority_ref: trellis_protocol::AuthorizationAuthorityRefV1 {
                kind: trellis_protocol::AuthorizationAuthorityKindV1::Deployment,
                id: "authority-1".to_owned(),
                version: 1,
            },
            deployment_id: Some("deployment-1".to_owned()),
            instance_id: Some("instance-1".to_owned()),
            grant_set,
            resource_bindings: Vec::new(),
            capabilities,
            session_expires_at: Some(10_000),
            effective_authority_expires_at: Some(10_000),
            delegation_expires_at: None,
            materialization_version: 1,
        }
    }

    #[test]
    fn every_auth_rpc_subject_has_an_explicit_handler() {
        let artifact: Value = serde_json::from_str(include_str!("../../trellis.api.json")).unwrap();
        let source = include_str!("auth_rpc.rs");
        let rpc = artifact.get("rpc").and_then(Value::as_object).unwrap();
        for (name, descriptor) in rpc {
            let version = descriptor.get("version").and_then(Value::as_str).unwrap();
            let subject = format!("\"rpc.{version}.{name}\"");
            assert!(
                source.contains(&subject),
                "missing Auth RPC handler for {name}"
            );
        }
    }

    #[test]
    fn authored_event_ownership_is_explicit_required_authority() {
        let binding = super::super::auth::auth_runtime_participant_binding(1).unwrap();
        let grants = binding
            .resolve()
            .unwrap()
            .proposal()
            .required()
            .grant_set()
            .clone();
        for event in [
            "Auth.Sessions.Revoked",
            "Auth.DeviceUserAuthorities.Resolved",
        ] {
            assert!(grants.permissions().contains(
                &PermissionAtomV1::new(
                    PermissionTargetV1::api_surface(
                        "trellis.auth@v1",
                        ApiSurfaceKindV1::Event,
                        event,
                    )
                    .unwrap(),
                    PermissionActionV1::Publish,
                )
                .unwrap()
            ));
        }
    }

    #[test]
    fn request_subject_resolution_requires_one_exact_permission_atom() {
        let binding = super::super::auth::auth_runtime_participant_binding(1).unwrap();
        let rpc = resolved_permission(
            PermissionTargetV1::api_surface(
                "trellis.auth@v1",
                ApiSurfaceKindV1::Rpc,
                "Auth.Sessions.Me",
            ),
            PermissionActionV1::Call,
        )
        .unwrap();
        let authorization = issuable_state(&binding, GrantSetV1::new(vec![rpc]), Vec::new());
        assert!(
            request_subject_allowed("rpc.v1.Auth.Sessions.Me", &[], &authorization, &binding,)
                .unwrap()
        );
        assert!(!request_subject_allowed("$JS.API.INFO", &[], &authorization, &binding).unwrap());
        assert!(!request_subject_allowed(
            "rpc.v1.Auth.Users.Create",
            &[],
            &authorization,
            &binding,
        )
        .unwrap());

        let mut ambiguous = binding.clone();
        let mut apis: BTreeMap<String, Value> =
            serde_json::from_str(&ambiguous.api_artifacts_json).unwrap();
        let mut duplicate = apis["trellis.auth@v1"].clone();
        duplicate["id"] = json!("other.auth@v1");
        duplicate["capabilities"] = json!({});
        duplicate["consent"] = json!({});
        apis.insert("other.auth@v1".to_owned(), duplicate);
        ambiguous.api_artifacts_json = serde_json::to_string(&apis).unwrap();
        assert!(!request_subject_allowed(
            "rpc.v1.Auth.Sessions.Me",
            &[],
            &authorization,
            &ambiguous,
        )
        .unwrap());
    }

    #[test]
    fn operation_control_and_event_publish_remain_exact() {
        let binding = super::super::auth::auth_runtime_participant_binding(1).unwrap();
        let observe = resolved_permission(
            PermissionTargetV1::api_surface(
                "trellis.auth@v1",
                ApiSurfaceKindV1::Operation,
                "Auth.DeviceUserAuthorities.Resolve",
            ),
            PermissionActionV1::Observe,
        )
        .unwrap();
        let authorization =
            issuable_state(&binding, GrantSetV1::new(vec![observe.clone()]), Vec::new());
        assert!(request_subject_allowed(
            "operations.v1.Auth.DeviceUserAuthorities.Resolve.control",
            &[observe],
            &authorization,
            &binding,
        )
        .unwrap());
        assert!(request_subject_allowed(
            "operations.v1.Auth.DeviceUserAuthorities.Resolve.control",
            &[],
            &authorization,
            &binding,
        )
        .unwrap());
        assert!(!request_subject_allowed(
            "operations.v1.Auth.DeviceUserAuthorities.Resolve",
            &[],
            &authorization,
            &binding,
        )
        .unwrap());

        let publish = resolved_permission(
            PermissionTargetV1::api_surface(
                "trellis.auth@v1",
                ApiSurfaceKindV1::Event,
                "Auth.DeviceUserAuthorities.Resolved",
            ),
            PermissionActionV1::Publish,
        )
        .unwrap();
        let authorization = issuable_state(&binding, GrantSetV1::new(vec![publish]), Vec::new());
        assert!(event_subject_allowed(
            "events.v1.Auth.DeviceUserAuthorities.Resolved.dep-1",
            &authorization,
            &binding,
        )
        .unwrap());
        let authorization = issuable_state(&binding, GrantSetV1::new(Vec::new()), Vec::new());
        assert!(!event_subject_allowed(
            "events.v1.Auth.DeviceUserAuthorities.Resolved.dep-1",
            &authorization,
            &binding,
        )
        .unwrap());
    }

    #[test]
    fn capability_requires_every_mapped_atom_and_platform_capability() {
        let mut binding = super::super::auth::auth_runtime_participant_binding(1).unwrap();
        let first = resolved_permission(
            PermissionTargetV1::api_surface(
                "trellis.auth@v1",
                ApiSurfaceKindV1::Rpc,
                "Auth.Sessions.Me",
            ),
            PermissionActionV1::Call,
        )
        .unwrap();
        let second = resolved_permission(
            PermissionTargetV1::api_surface(
                "trellis.auth@v1",
                ApiSurfaceKindV1::Rpc,
                "Auth.Sessions.List",
            ),
            PermissionActionV1::Call,
        )
        .unwrap();
        let mut apis: BTreeMap<String, Value> =
            serde_json::from_str(&binding.api_artifacts_json).unwrap();
        apis.get_mut("trellis.auth@v1").unwrap()["capabilities"]["test.multi"] = json!({
            "allows": [first, second]
        });
        binding.api_artifacts_json = serde_json::to_string(&apis).unwrap();
        let authorization = issuable_state(
            &binding,
            GrantSetV1::new(vec![first.clone()]),
            vec!["test.multi".to_owned()],
        );
        assert!(request_capability_permissions(
            &["test.multi".to_owned()],
            &authorization,
            &binding,
        )
        .unwrap()
        .is_none());
        let authorization =
            issuable_state(&binding, GrantSetV1::new(vec![first, second]), Vec::new());
        assert!(request_capability_permissions(
            &["test.multi".to_owned()],
            &authorization,
            &binding,
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn transitional_request_proof_rejects_payload_tampering() {
        let auth = SessionAuth::from_seed_base64url("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
            .expect("fixed seed");
        let payload = br#"{"limit":10}"#;
        let subject = "rpc.v1.Auth.Connections.List";
        let iat = 1_700_000_000;
        let request_id = "req_fixed";
        let proof = auth.create_proof(subject, payload, iat, request_id);
        let mut request = AuthRequestsValidateRequest {
            session_key: auth.session_key,
            proof,
            subject: subject.to_owned(),
            payload_hash: URL_SAFE_NO_PAD.encode(Sha256::digest(payload)),
            iat,
            request_id: request_id.to_owned(),
            capabilities: Vec::new(),
        };

        verify_request_proof(&request).expect("valid proof");
        request.payload_hash = URL_SAFE_NO_PAD.encode(Sha256::digest(b"tampered"));
        assert!(verify_request_proof(&request).is_err());
    }

    #[test]
    fn transitional_event_proof_rejects_subject_tampering() {
        let auth = SessionAuth::from_seed_base64url("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
            .expect("fixed seed");
        let payload = br#"{"deviceId":"dev_1"}"#;
        let subject = "events.v1.Example.Changed";
        let event_id = "evt_fixed";
        let event_time = "2026-07-20T00:00:00Z";
        let proof = auth.create_event_proof(subject, payload, event_id, event_time);
        let mut request = AuthEventsValidateRequest {
            session_key: auth.session_key,
            proof,
            subject: subject.to_owned(),
            payload_hash: URL_SAFE_NO_PAD.encode(Sha256::digest(payload)),
            event_id: event_id.to_owned(),
            event_time: event_time.to_owned(),
        };

        verify_event_proof(&request).expect("valid event proof");
        request.subject = "events.v1.Example.Deleted".to_owned();
        assert!(verify_event_proof(&request).is_err());
    }

    #[test]
    fn public_rpc_errors_never_serialize_internal_causes() {
        let secret = "postgres://admin:secret@internal/auth";
        let payload = public_rpc_error(
            "rpc.v1.Auth.Users.List",
            &AuthorizationStateError::Storage(secret.to_owned()),
        );
        let encoded = serde_json::to_string(&payload).unwrap();
        assert!(!encoded.contains(secret));
        assert_eq!(payload["type"], "UnexpectedError");
        assert_eq!(payload["context"]["code"], "internal_error");
        let invalid = public_rpc_error(
            "rpc.v1.Auth.DeploymentAuthority.Plan",
            &AuthorizationStateError::InvalidRecord(secret.to_owned()),
        );
        assert_eq!(invalid["type"], "AuthError");
        assert_eq!(invalid["reason"], "invalid_request");
        assert!(!serde_json::to_string(&invalid).unwrap().contains(secret));
    }
}
