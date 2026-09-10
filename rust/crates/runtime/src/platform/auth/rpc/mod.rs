//! Exact Auth RPC routing and workflow dispatch.

mod error;
mod router;
mod workflows;

use error::public_rpc_error;

use async_nats::header::HeaderMap;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use bytes::Bytes;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use trellis_protocol::AuthorizationPrincipalKind;
use trellis_rs::service::Router;
use ulid::Ulid;

use super::context::AuthorizationContextRepository;
use super::{
    activation_review_event_action_id, validate_connection_kick_response, AccountFlowKind,
    AccountRepository, AuthConnectionPresence, AuthEphemeralRepository, AuthService,
    AuthorityEvidenceRepository, AuthorizationStateError, CapabilityGroupRecord,
    ChangePasswordInput, CreateAccountFlowInput, CreateUserInput, DecideActivationReviewInput,
    DeploymentProfileCreation, DeploymentProfileMutation, DeploymentProfileRecord,
    DeploymentProfileState, DeploymentRepository, DeviceActivationReviewRecord,
    DeviceActivationReviewState, DeviceDelegationMutation, DeviceDelegationRecord,
    DeviceDelegationState, DeviceReviewMode, GrantOwnerKind, IdempotencyResultRecord,
    IdempotentOutcome, LoginPortalMutation, LoginPortalRecord, LoginSettingsRecord,
    NatsAuthEphemeralRepository, PortalGrantOverrideRecord, PortalPolicyReconciliationHandle,
    PortalRepository, PortalRoleMapping, PortalRouteMutation, PortalRouteRecord,
    PortalRouteRemoval, PostCommitActionKind, PostCommitActionRecord, PrincipalKind,
    PrincipalState, ProviderIdentityUnlink, ProvisionDeviceInput, ProvisionServiceIdentityInput,
    ProvisionedIdentityKind, ProvisionedIdentityRecord, ProvisionedIdentityState,
    ProvisionedInstanceMutation, ProvisioningRepository, RuntimeInstanceState, SessionRecord,
    SessionRepository, SessionState, SqliteAuthorizationStore, UpdateUserInput, UserAccount,
};
use crate::shutdown::StopHandle;
use crate::supervisor::RuntimeError;

const MAX_CONCURRENT_REQUESTS: usize = 64;

pub(crate) struct AuthRpcRuntime {
    subscriber: async_nats::Subscriber,
    processor: AuthRpcProcessor,
}

#[derive(Clone)]
pub(crate) struct AuthRpcProcessor {
    pub(crate) client: async_nats::Client,
    pub(crate) system_client: async_nats::Client,
    pub(crate) service: AuthService<SqliteAuthorizationStore>,
    pub(crate) ephemeral: NatsAuthEphemeralRepository,
    pub(crate) public_origin: String,
    pub(crate) verifier: crate::platform::auth::verifier::RuntimeAuthVerifier,
    pub(crate) routes: Arc<Router>,
    pub(crate) portal_reconciliation: PortalPolicyReconciliationHandle,
}

struct ValidatedRequest {
    principal_id: String,
    principal_kind: PrincipalKind,
    context: trellis_protocol::VerifiedAuthorizationContext,
    session_public_key: String,
    platform_privileges: Vec<trellis_protocol::PlatformPrivilege>,
}

fn mutation_actor(caller: &ValidatedRequest) -> super::MutationActor {
    super::MutationActor {
        context_digest: caller.context.context_digest().to_owned(),
        principal_id: caller.principal_id.clone(),
        participant_id: caller.context.participant_id().to_owned(),
        owner_kind: caller.context.owner_kind(),
        owner_id: caller.context.owner_id().to_owned(),
        grant_revision: caller.context.grant_revision(),
        login_session_id: caller.context.login_session_id().map(str::to_owned),
        session_public_key: caller.session_public_key.clone(),
    }
}

impl AuthRpcRuntime {
    pub(crate) async fn start(
        processor: AuthRpcProcessor,
    ) -> Result<Self, AuthorizationStateError> {
        let subscriber = processor
            .client
            .queue_subscribe("rpc.v1.Auth.>", "trellis-auth-rpc".to_owned())
            .await
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        Ok(Self {
            subscriber,
            processor,
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
        router::dispatch(self, subject, message).await
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
        let review_mode = match (kind, nullable_string(&input, "reviewMode")?.as_deref()) {
            (PrincipalKind::Device, Some("none")) => Some(DeviceReviewMode::None),
            (PrincipalKind::Device, Some("required")) => Some(DeviceReviewMode::Required),
            (PrincipalKind::Service, None) => None,
            (PrincipalKind::Device, _) => {
                return Err(AuthorizationStateError::InvalidRecord(
                    "device deployment reviewMode must be none or required".to_owned(),
                ));
            }
            (PrincipalKind::Service, _) => {
                return Err(AuthorizationStateError::InvalidRecord(
                    "service deployment reviewMode must be null".to_owned(),
                ));
            }
            (PrincipalKind::User, _) => unreachable!("deployment kind excludes users"),
        };
        let now = now_millis()?;
        let deployment_id = format!("dep_{}", Ulid::new());
        let profile = DeploymentProfileRecord {
            deployment_id: deployment_id.clone(),
            kind,
            display_name: required_string(&input, "displayName")?.to_owned(),
            participant_id: nullable_string(&input, "participantId")?,
            portal_id: nullable_string(&input, "portalId")?,
            review_mode,
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
                principal: super::PrincipalRecord {
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
                    &caller.principal_id,
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
                || state.is_some_and(|value| deployment_state_wire(profile.state) != value)
            {
                continue;
            }
            entries.push(self.deployment_value(profile).await?);
        }
        Ok(paginate_values(entries, &input))
    }

    async fn deployments_get(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let deployment_id = required_string(&input, "deploymentId")?;
        let profile = self
            .service
            .repository()
            .get_deployment_profile(deployment_id)
            .await?
            .ok_or(AuthorizationStateError::NotFound)?;
        let participant_id = profile.participant_id.clone().ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(
                "deployment has no installed participant".to_owned(),
            )
        })?;
        let binding = self
            .service
            .repository()
            .get_grant_binding(
                GrantOwnerKind::Deployment,
                deployment_id.to_owned(),
                participant_id.clone(),
            )
            .await?;
        if binding.is_some()
            && !(caller.context.owner_kind() == GrantOwnerKind::Deployment
                && caller.context.owner_id() == deployment_id)
        {
            require_admin(caller)?;
        }
        let resources = if let Some(binding) = &binding {
            self.service
                .repository()
                .get_resource_bindings(
                    GrantOwnerKind::Deployment,
                    deployment_id.to_owned(),
                    participant_id,
                    binding.installed_revision,
                )
                .await?
        } else {
            Vec::new()
        };
        Ok(json!({
            "deployment": self.deployment_value(profile).await?,
            "binding": binding,
            "resources": resources,
        }))
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
                predecessor_action_id: None,
                action_id: digest_parts(&[deployment_id, idempotency_key, "kick"]),
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
                    &caller.principal_id,
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
                "state": deployment_state_wire(state),
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
        let mut value = json!({
            "deploymentId": profile.deployment_id,
            "kind": profile.kind,
            "displayName": profile.display_name,
            "state": deployment_state_wire(profile.state),
            "participantId": profile.participant_id,
            "expiresAt": profile.expires_at,
            "reviewMode": profile.review_mode,
            "requiresDeviceDelegation": profile.requires_device_delegation,
            "portalId": profile.portal_id,
            "createdAt": profile.created_at,
            "updatedAt": profile.updated_at,
            "disabledAt": principal.disabled_at,
            "revokedAt": principal.revoked_at,
            "version": profile.version,
            "disabled": profile.state != DeploymentProfileState::Active,
        });
        if profile.kind == PrincipalKind::Service {
            value["namespaces"] = json!([]);
        }
        Ok(value)
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
                        &caller.principal_id,
                        input
                            .get("idempotencyKey")
                            .and_then(Value::as_str)
                            .unwrap_or(deployment_id),
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
                    &caller.principal_id,
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
                    &caller.principal_id,
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
                || (device.state == crate::platform::auth::DeviceState::Pending
                    && target == RuntimeInstanceState::Active)
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            device.state = match target {
                RuntimeInstanceState::Active => crate::platform::auth::DeviceState::Active,
                RuntimeInstanceState::Disabled | RuntimeInstanceState::Stale => {
                    crate::platform::auth::DeviceState::Disabled
                }
                RuntimeInstanceState::Revoked => crate::platform::auth::DeviceState::Revoked,
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
                    &caller.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: (target != RuntimeInstanceState::Active)
                    .then(|| PostCommitActionRecord {
                        predecessor_action_id: None,
                        action_id: digest_parts(&[instance_id, idempotency_key, "kick"]),
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
            crate::platform::auth::DeviceState::Pending => "pending",
            crate::platform::auth::DeviceState::Active => "active",
            crate::platform::auth::DeviceState::Disabled => "disabled",
            crate::platform::auth::DeviceState::Revoked => "revoked",
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
                super::DeviceState::Pending => "pending",
                super::DeviceState::Active => "approved",
                super::DeviceState::Disabled => "approved",
                super::DeviceState::Revoked => "revoked",
            },
            "delegationRequired": profile.requires_device_delegation,
            "delegationState": delegation.as_ref().map_or(
                if profile.requires_device_delegation { "missing" } else { "active" },
                |value| match value.state {
                    super::DeviceDelegationState::Active => "active",
                    super::DeviceDelegationState::Missing => "missing",
                    super::DeviceDelegationState::Revoked => "revoked",
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
            predecessor_action_id: None,
            action_id: digest_parts(&[
                "Auth.DeviceUserAuthorities.Revoke",
                idempotency_key,
                suffix,
            ]),
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
                    &caller.principal_id,
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
                    && session.state == crate::platform::auth::SessionState::Active
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
        self.service
            .expire_due_activation_reviews(now_millis()?)
            .await?;
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
        let now = now_millis()?;
        self.service.expire_due_activation_reviews(now).await?;
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
        if profile.review_mode != Some(DeviceReviewMode::Required) {
            return Err(AuthorizationStateError::InvalidRecord(
                "deployment does not require administrative device review".to_owned(),
            ));
        }
        let event_suffix = if state == DeviceActivationReviewState::Approved {
            "approved"
        } else {
            "resolved"
        };
        let event_payload = if state == DeviceActivationReviewState::Approved {
            json!({
                "eventType": "Auth.DeviceUserAuthorities.Approved",
                "eventSubject": format!(
                    "events.v1.Auth.DeviceUserAuthorities.Approved.{}",
                    review.deployment_id,
                ),
                "eventId": format!("evt_{}", digest_parts(&[review_id, event_suffix])),
                "occurredAt": now,
                "deploymentId": review.deployment_id,
                "instanceId": review.instance_id,
                "approvedBy": caller.principal_id,
                "approvedAt": now,
            })
        } else {
            json!({
                "eventType": "Auth.DeviceUserAuthorities.Resolved",
                "eventSubject": format!(
                    "events.v1.Auth.DeviceUserAuthorities.Resolved.{}",
                    review.deployment_id,
                ),
                "eventId": format!("evt_{}", digest_parts(&[review_id, event_suffix])),
                "occurredAt": now,
                "deploymentId": review.deployment_id,
                "instanceId": review.instance_id,
                "state": "rejected",
            })
        };
        let mut actions = vec![PostCommitActionRecord {
            predecessor_action_id: Some(activation_review_event_action_id(
                review_id,
                if review.activated_by_user_principal_id.is_some() {
                    "requested"
                } else {
                    "review-requested"
                },
            )?),
            action_id: activation_review_event_action_id(review_id, event_suffix)?,
            kind: PostCommitActionKind::Event,
            payload: event_payload,
            created_at: now,
            attempts: 0,
            next_attempt_at: now,
            claimed_until: None,
            last_error: None,
        }];
        let activation_ready = state == DeviceActivationReviewState::Approved
            && (!profile.requires_device_delegation
                || review.activated_by_user_principal_id.is_some());
        if activation_ready {
            actions.push(PostCommitActionRecord {
                predecessor_action_id: Some(activation_review_event_action_id(
                    review_id,
                    event_suffix,
                )?),
                action_id: activation_review_event_action_id(review_id, "resolved")?,
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
                    "state": "active",
                }),
                created_at: now,
                attempts: 0,
                next_attempt_at: now,
                claimed_until: None,
                last_error: None,
            });
        }
        let outcome = self
            .service
            .decide_activation_review(DecideActivationReviewInput {
                review_id: review_id.to_owned(),
                expected_version: required_u64(&input, "expectedVersion")?,
                state,
                decided_by: caller.principal_id.clone(),
                reason: nullable_string(&input, "reason")?,
                delegation: (state == DeviceActivationReviewState::Approved
                    && profile.requires_device_delegation
                    && review.activated_by_user_principal_id.is_some())
                .then(|| DeviceDelegationRecord {
                    principal_id: review.principal_id.clone(),
                    deployment_id: review.deployment_id.clone(),
                    required: true,
                    state: DeviceDelegationState::Active,
                    expires_at: None,
                }),
                activate_device: activation_ready,
                decided_at: now,
                idempotency: rpc_idempotency(
                    "Auth.DeviceUserAuthorities.Reviews.Decide",
                    &caller.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                actions,
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

    async fn capability_groups_list(
        &self,
        payload: &[u8],
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let entries = self
            .service
            .repository()
            .list_capability_groups()
            .await?
            .into_iter()
            .map(|group| serde_json::to_value(group).expect("group serializes"))
            .collect();
        Ok(offset_page(entries, &input))
    }

    async fn capability_groups_get(
        &self,
        payload: &[u8],
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let group = self
            .service
            .repository()
            .get_capability_group(required_string(&input, "groupKey")?)
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("capability group not found".to_owned())
            })?;
        Ok(json!({ "group": group }))
    }

    async fn capability_groups_put(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let key = required_string(&input, "groupKey")?;
        let now = now_millis()?;
        let current = self.service.repository().get_capability_group(key).await?;
        let expected_version = input.get("expectedVersion").and_then(Value::as_u64);
        let mut capabilities = optional_string_array(&input, "capabilities")?.unwrap_or_default();
        capabilities.sort();
        capabilities.dedup();
        let mut included_groups =
            optional_string_array(&input, "includedGroups")?.unwrap_or_default();
        included_groups.sort();
        included_groups.dedup();
        for included_group in &included_groups {
            if included_group == key
                || self
                    .service
                    .repository()
                    .get_capability_group(included_group)
                    .await?
                    .is_none()
            {
                return Err(AuthorizationStateError::InvalidRecord(format!(
                    "unknown included capability group '{included_group}'"
                )));
            }
        }
        let semantic_changed = current.as_ref().is_none_or(|group| {
            group.capabilities != capabilities || group.included_groups != included_groups
        });
        let group = CapabilityGroupRecord {
            group_key: key.to_owned(),
            display_name: required_string(&input, "displayName")?.to_owned(),
            description: required_string(&input, "description")?.to_owned(),
            capabilities,
            included_groups,
            created_at: current.as_ref().map_or(now, |group| group.created_at),
            updated_at: now,
            version: expected_version.map_or(1, |version| version + 1),
        };
        let outcome = self
            .service
            .repository()
            .put_capability_group(
                group,
                expected_version,
                rpc_idempotency(
                    "Auth.CapabilityGroups.Put",
                    &caller.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
            )
            .await?;
        if semantic_changed {
            self.portal_reconciliation.notify_all();
        }
        let group = match outcome {
            IdempotentOutcome::Applied(group) => serde_json::to_value(group),
            IdempotentOutcome::Replayed(group) => Ok(group),
        }
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        Ok(json!({ "group": group }))
    }

    async fn capability_groups_delete(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let key = required_string(&input, "groupKey")?;
        let now = now_millis()?;
        let outcome = self
            .service
            .repository()
            .delete_capability_group(
                key,
                required_u64(&input, "expectedVersion")?,
                rpc_idempotency(
                    "Auth.CapabilityGroups.Delete",
                    &caller.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
            )
            .await?;
        self.portal_reconciliation.notify_all();
        let success = match outcome {
            IdempotentOutcome::Applied(success) => success,
            IdempotentOutcome::Replayed(success) => success.as_bool().ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("invalid delete replay".to_owned())
            })?,
        };
        Ok(json!({ "success": success }))
    }

    async fn portal_grant_overrides_list(
        &self,
        payload: &[u8],
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let entries = self
            .service
            .repository()
            .list_portal_grant_overrides(
                input.get("portalId").and_then(Value::as_str),
                input.get("participantId").and_then(Value::as_str),
            )
            .await?
            .into_iter()
            .map(|entry| serde_json::to_value(entry).expect("policy serializes"))
            .collect();
        Ok(offset_page(entries, &input))
    }

    async fn portal_grant_overrides_put(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        require_admin(caller)?;
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let portal_id = required_string(&input, "portalId")?;
        let participant_id = required_string(&input, "participantId")?;
        let expected_version = input.get("expectedVersion").and_then(Value::as_u64);
        let current = self
            .service
            .repository()
            .get_portal_grant_override(portal_id, participant_id)
            .await?;
        let mut direct_capabilities =
            optional_string_array(&input, "directCapabilities")?.unwrap_or_default();
        direct_capabilities.sort();
        direct_capabilities.dedup();
        let mut capability_group_keys =
            optional_string_array(&input, "capabilityGroupKeys")?.unwrap_or_default();
        capability_group_keys.sort();
        capability_group_keys.dedup();
        let mut role_mappings = input
            .get("roleMappings")
            .cloned()
            .map(serde_json::from_value::<Vec<PortalRoleMapping>>)
            .transpose()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
            .unwrap_or_default();
        for mapping in &mut role_mappings {
            mapping.direct_capabilities.sort();
            mapping.direct_capabilities.dedup();
            mapping.capability_group_keys.sort();
            mapping.capability_group_keys.dedup();
        }
        sort_and_validate_role_mappings(&mut role_mappings)?;
        let now = now_millis()?;
        let policy = PortalGrantOverrideRecord {
            portal_id: portal_id.to_owned(),
            participant_id: participant_id.to_owned(),
            direct_capabilities,
            capability_group_keys,
            role_mappings,
            created_at: current.as_ref().map_or(now, |policy| policy.created_at),
            updated_at: now,
            version: expected_version.map_or(1, |version| version + 1),
        };
        let outcome = self
            .service
            .repository()
            .put_portal_grant_override(
                policy,
                expected_version,
                rpc_idempotency(
                    "Auth.Portals.GrantOverrides.Put",
                    &caller.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
            )
            .await?;
        self.portal_reconciliation.notify_portal(portal_id).await;
        let policy = match outcome {
            IdempotentOutcome::Applied(policy) => serde_json::to_value(policy),
            IdempotentOutcome::Replayed(policy) => Ok(policy),
        }
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        Ok(json!({ "policy": policy }))
    }

    async fn portal_grant_overrides_remove(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        require_admin(caller)?;
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let now = now_millis()?;
        let outcome = self
            .service
            .repository()
            .remove_portal_grant_override(
                required_string(&input, "portalId")?,
                required_string(&input, "participantId")?,
                required_u64(&input, "expectedVersion")?,
                rpc_idempotency(
                    "Auth.Portals.GrantOverrides.Remove",
                    &caller.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
            )
            .await?;
        self.portal_reconciliation
            .notify_portal(required_string(&input, "portalId")?)
            .await;
        let removed: Option<super::PortalGrantOverrideRecord> = match outcome {
            IdempotentOutcome::Applied(policy) => policy,
            IdempotentOutcome::Replayed(policy) => serde_json::from_value(policy)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        };
        Ok(removed.map_or_else(|| json!({}), |removed| json!({ "removed": removed })))
    }

    async fn sessions_me(
        &self,
        validated: ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let now = now_millis()?;
        let issued = self
            .service
            .repository()
            .get_context_by_digest(validated.context.context_digest())
            .await?
            .ok_or(AuthorizationStateError::NotAuthorized)?;
        if issued.state != super::context::AuthorizationContextState::Active {
            return Err(AuthorizationStateError::NotAuthorized);
        }
        let installed = self
            .service
            .repository()
            .get_installed_participant(issued.participant_id, Some(issued.installed_revision))
            .await?;
        let session = match validated.context.login_session_id() {
            Some(id) => {
                let session = self
                    .service
                    .repository()
                    .get_session(id)
                    .await?
                    .ok_or(AuthorizationStateError::SessionMissing)?;
                if session.principal_id != validated.principal_id
                    || session.participant_id != validated.context.participant_id()
                    || session.session_public_key != validated.session_public_key
                {
                    return Err(AuthorizationStateError::NotAuthorized);
                }
                if session.state == SessionState::Revoked {
                    return Err(AuthorizationStateError::SessionRevoked);
                }
                if session.state == SessionState::Expired
                    || session.expires_at.is_some_and(|expires| expires <= now)
                {
                    return Err(AuthorizationStateError::SessionExpired);
                }
                Some(session)
            }
            None => None,
        };
        let user = if validated.principal_kind == PrincipalKind::User {
            let (principal, profile) = self
                .service
                .repository()
                .get_user_account(&validated.principal_id)
                .await?
                .ok_or(AuthorizationStateError::PrincipalMissing)?;
            if principal.state != PrincipalState::Active {
                return Err(AuthorizationStateError::PrincipalInactive);
            }
            Some(user_value(UserAccount { principal, profile }))
        } else {
            None
        };
        let mut connection = json!({
            "connectionId": validated.context.connection_id(),
            "sessionKey": validated.session_public_key,
            "inboxPrefix": validated.context.inbox_prefix(),
            "participantId": validated.context.participant_id(),
            "participantKind": installed["participant"]["participantKind"],
            "principalId": validated.principal_id,
            "principalKind": validated.principal_kind,
            "grants": validated.context.grant_set(),
            "platformPrivileges": validated.context.platform_privileges(),
        });
        for (field, value) in [
            ("loginSessionId", validated.context.login_session_id()),
            ("identityKeyId", validated.context.identity_key_id()),
            ("deploymentId", validated.context.deployment_id()),
            ("instanceId", validated.context.instance_id()),
        ] {
            if let Some(value) = value {
                connection[field] = json!(value);
            }
        }
        Ok(json!({
            "session": session,
            "user": user,
            "connection": connection,
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
                    &caller.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        self.portal_reconciliation.notify_portal(portal_id).await;
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
                    &caller.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        self.portal_reconciliation.notify_portal(portal_id).await;
        Ok(json!({ "removed": true }))
    }

    async fn capabilities_list(&self, payload: &[u8]) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let source_api = input.get("sourceApi").and_then(Value::as_str);
        let entries = if source_api
            .is_none_or(|value| value == trellis_runtime_apis::apis::trellis_auth_v1::API_ID)
        {
            let (_, participant) = self
                .service
                .repository()
                .get_installed_participant_record(
                    super::builtins::AUTH_RUNTIME_PARTICIPANT_ID.to_owned(),
                    None,
                )
                .await?
                .ok_or(AuthorizationStateError::ParticipantMissing)?;
            participant
                .projection
                .implemented_apis
                .get(trellis_runtime_apis::apis::trellis_auth_v1::API_ID)
                .ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord(
                        "installed Auth API projection is absent".to_owned(),
                    )
                })?
                .capabilities
                .iter()
                .map(|(capability, definition)| {
                    json!({
                        "capability": capability,
                        "displayName": definition.display_name,
                        "description": definition.description,
                        "allows": definition.allows,
                        "sourceApi": trellis_runtime_apis::apis::trellis_auth_v1::API_ID,
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
            "portal": {
                "portalId": portal.portal_id,
                "displayName": portal.display_name,
                "entryUrl": portal.entry_url,
                "builtIn": portal.builtin,
                "disabled": portal.disabled,
                "createdAt": millis_rfc3339(portal.created_at)?,
                "updatedAt": millis_rfc3339(portal.updated_at)?,
            },
            "settings": {
                "portalId": portal_id,
                "localRegistrationEnabled": portal.local_registration_enabled,
                "federatedRegistrationEnabled": settings.federated_registration_enabled,
                "allowedFederatedProviders": portal.provider_ids,
                "selfRegisteredAccountActive": true,
                "updatedAt": millis_rfc3339(settings.updated_at)?,
            },
            "defaultCapabilities": [],
            "defaultCapabilityGroups": [],
            "federatedProviders": [],
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
                    &caller.principal_id,
                    idempotency_key,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        self.portal_reconciliation.notify_portal(portal_id).await;
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
        let routes = self.service.repository().list_portal_routes().await?;
        let current = routes
            .iter()
            .find(|route| route.route_id == route_id)
            .cloned();
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
        if routes.iter().any(|existing| {
            existing.route_id != route.route_id
                && existing.participant_id == route.participant_id
                && existing.origin == route.origin
                && existing.deployment_id == route.deployment_id
                && existing.priority == route.priority
        }) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        self.service
            .repository()
            .put_portal_route(PortalRouteMutation {
                route: route.clone(),
                expected_version,
                idempotency: rpc_idempotency(
                    "Auth.Portals.Routes.Put",
                    &caller.principal_id,
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
                    &caller.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        Ok(json!({ "routeId": route_id, "removed": true }))
    }

    async fn sessions_list(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let request: trellis_runtime_apis::types::AuthSessionsListRequest =
            serde_json::from_slice(payload)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        if request
            .limit
            .as_ref()
            .is_some_and(|limit| !(1..=100).contains(&limit.0 .0))
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "invalid page limit".into(),
            ));
        }
        let mut input = serde_json::to_value(request)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        if !caller_is_admin(caller) {
            if nullable_string(&input, "principalId")?
                .is_some_and(|principal_id| principal_id != caller.principal_id)
            {
                return Err(AuthorizationStateError::NotAuthorized);
            }
            input["principalId"] = json!(caller.principal_id);
        }
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
        entries.sort_by(|left, right| left.session_id.cmp(&right.session_id));
        Ok(paginate_sessions(entries, &input))
    }

    async fn sessions_logout(
        &self,
        payload: &[u8],
        caller: &ValidatedRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let input: Value = serde_json::from_slice(payload)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        if !input.is_object() {
            return Err(AuthorizationStateError::InvalidRecord(
                "logout request must be an object".to_owned(),
            ));
        }
        let login_session_id = caller
            .context
            .login_session_id()
            .ok_or(AuthorizationStateError::WrongPrincipalKind)?;
        let input = json!({
            "sessionId": login_session_id,
            "expectedVersion": null,
            "idempotencyKey": "logout",
            "reason": null,
        });
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
        let now = now_millis()?;
        let request_digest = trellis_protocol::digest_json(&input)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let action = |kind, suffix: &str, payload| PostCommitActionRecord {
            predecessor_action_id: None,
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
                        .map(|caller| caller.principal_id.clone())
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
                            "revokedBy": caller.map(|caller| &caller.principal_id),
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
        let response = self
            .system_client
            .request(
                format!("$SYS.REQ.SERVER.{}.KICK", connection.server_id),
                Bytes::from(
                    serde_json::to_vec(&json!({ "cid": client_id }))
                        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?,
                ),
            )
            .await
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        validate_connection_kick_response(&response.payload)
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
                    &validated.principal_id,
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
        let login_session_id = caller
            .context
            .login_session_id()
            .ok_or(AuthorizationStateError::WrongPrincipalKind)?;
        let now = now_millis()?;
        let action = PostCommitActionRecord {
            predecessor_action_id: None,
            action_id: digest_parts(&[
                "Auth.Users.Password.Change",
                &caller.principal_id,
                required_string(&input, "idempotencyKey")?,
            ]),
            kind: PostCommitActionKind::Kick,
            payload: json!({
                "principalId": caller.principal_id,
                "exceptSessionId": login_session_id,
            }),
            created_at: now,
            attempts: 0,
            next_attempt_at: now,
            claimed_until: None,
            last_error: None,
        };
        match self
            .service
            .change_password(ChangePasswordInput {
                principal_id: caller.principal_id.clone(),
                current_session_id: login_session_id.to_owned(),
                current_password: required_string(&input, "currentPassword")?.to_owned(),
                new_password: required_string(&input, "newPassword")?.to_owned(),
                changed_at: now,
                idempotency: rpc_idempotency(
                    "Auth.Users.Password.Change",
                    &caller.principal_id,
                    required_string(&input, "idempotencyKey")?,
                    &input,
                    now,
                )?,
                actions: vec![action],
            })
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
            &caller.principal_id,
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
        let admin_target = if kind == AccountFlowKind::PasswordReset {
            self.require_admin_for_admin_target(caller, principal_id)
                .await?
        } else {
            false
        };
        let return_target = nullable_string(input, "returnTarget")?;
        let outcome = self
            .service
            .create_account_flow(CreateAccountFlowInput {
                kind,
                target_principal_id: Some(principal_id.to_owned()),
                target_provider_id: None,
                return_location: return_target.clone(),
                payload: json!({
                    "allowedProviders": allowed_providers,
                    "adminTarget": admin_target,
                    "requestedByPrincipalId": caller.principal_id,
                }),
                created_at: now,
                expires_at: now.saturating_add(15 * 60_000),
                idempotency: rpc_idempotency(
                    purpose,
                    &caller.principal_id,
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
            .list_provider_identities(&caller.principal_id)
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
            &caller.principal_id,
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
                principal_id: caller.principal_id.clone(),
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
                actor: mutation_actor(validated),
                principal_id: principal_id.to_owned(),
                expected_version,
                name: nullable_string(&input, "name")?,
                email: nullable_string(&input, "email")?,
                image: nullable_string(&input, "image")?,
                state,
                updated_at: now,
                idempotency: rpc_idempotency(
                    "Auth.Users.Update",
                    &validated.principal_id,
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

    async fn require_admin_for_admin_target(
        &self,
        caller: &ValidatedRequest,
        principal_id: &str,
    ) -> Result<bool, AuthorizationStateError> {
        let admin_target = self
            .service
            .repository()
            .user_is_admin(principal_id.to_owned())
            .await?;
        if admin_target && !caller_is_admin(caller) {
            require_admin(caller)?;
        }
        Ok(admin_target)
    }
}

fn connection_value(connection: AuthConnectionPresence) -> Value {
    json!({
        "connectionId": connection.connection_id,
        "runtimeConnectionId": connection.runtime_connection_id,
        "loginSessionId": connection.login_session_id,
        "contextDigest": connection.context_digest,
        "principalId": connection.principal_id,
        "participantId": connection.participant_id,
        "deploymentId": connection.deployment_id,
        "instanceId": connection.instance_id,
        "serverId": connection.server_id,
        "clientId": connection.client_id,
        "userNkey": connection.user_nkey,
        "remoteAddress": connection.remote_address,
        "connectedAt": connection.connected_at,
        "lastSeenAt": connection.last_seen_at,
    })
}

fn service_instance_value(
    instance: super::RuntimeInstanceRecord,
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
        "activatedByUserPrincipalId": review.activated_by_user_principal_id,
        "state": review.state,
        "confirmationCode": review.payload.get("confirmationCode"),
        "requestedAt": review.requested_at,
        "expiresAt": review.expires_at,
        "decidedAt": review.decided_at,
        "decidedBy": review.decided_by,
        "reason": review.reason,
        "version": review.version,
    })
}

fn enum_string(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn deployment_state_wire(state: DeploymentProfileState) -> &'static str {
    match state {
        DeploymentProfileState::Active => "active",
        DeploymentProfileState::Disabled => "disabled",
        DeploymentProfileState::Removed => "revoked",
    }
}

fn paginate_values(entries: Vec<Value>, input: &Value) -> Value {
    let limit = input
        .get("limit")
        .and_then(|value| {
            value
                .as_i64()
                .or_else(|| value.as_str()?.parse::<i64>().ok())
        })
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

fn offset_page(entries: Vec<Value>, input: &Value) -> Value {
    let limit = input
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(100)
        .min(500) as usize;
    let offset = input.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;
    let count = entries.len();
    let mut page = json!({
        "entries": entries.into_iter().skip(offset).take(limit).collect::<Vec<_>>(),
        "count": count,
        "offset": offset,
        "limit": limit,
    });
    if offset + limit < count {
        page["nextOffset"] = json!(offset + limit);
    }
    page
}

fn paginate_sessions(entries: Vec<SessionRecord>, input: &Value) -> Value {
    let limit = input
        .get("limit")
        .and_then(|value| {
            value
                .as_i64()
                .or_else(|| value.as_str()?.parse::<i64>().ok())
        })
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

fn require_admin(caller: &ValidatedRequest) -> Result<(), AuthorizationStateError> {
    if caller_is_admin(caller) {
        Ok(())
    } else {
        Err(AuthorizationStateError::NotAuthorized)
    }
}

fn caller_is_admin(caller: &ValidatedRequest) -> bool {
    caller
        .platform_privileges
        .contains(&trellis_protocol::PlatformPrivilege::Admin)
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, AuthorizationStateError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AuthorizationStateError::InvalidRecord(format!("{key} is required")))
}

fn millis_rfc3339(value: i64) -> Result<String, AuthorizationStateError> {
    time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(value) * 1_000_000)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))
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

fn sort_and_validate_role_mappings(
    role_mappings: &mut [PortalRoleMapping],
) -> Result<(), AuthorizationStateError> {
    role_mappings.sort_by(|left, right| {
        (&left.provider_id, &left.role).cmp(&(&right.provider_id, &right.role))
    });
    if role_mappings
        .windows(2)
        .any(|pair| pair[0].provider_id == pair[1].provider_id && pair[0].role == pair[1].role)
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "roleMappings contains a duplicate providerId and role".to_owned(),
        ));
    }
    Ok(())
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

/// Bind an Auth mutation's exact input to its caller, purpose, and retry key.
pub(in crate::platform::auth) fn rpc_idempotency(
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

fn principal_state(principal: &super::PrincipalRecord) -> &'static str {
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

fn now_millis() -> Result<i64, AuthorizationStateError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?
        .as_millis()
        .try_into()
        .map_err(|_| AuthorizationStateError::Storage("current time overflow".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_page_omits_exhausted_next_offset() {
        let page = offset_page(vec![json!({ "id": 1 })], &json!({ "limit": 1 }));
        assert_eq!(page.get("nextOffset"), None);
    }

    #[test]
    fn duplicate_portal_role_mapping_is_rejected() {
        let mut mappings = vec![
            PortalRoleMapping {
                provider_id: "oidc".to_owned(),
                role: "operator".to_owned(),
                direct_capabilities: vec!["example::read".to_owned()],
                capability_group_keys: Vec::new(),
            },
            PortalRoleMapping {
                provider_id: "oidc".to_owned(),
                role: "operator".to_owned(),
                direct_capabilities: vec!["example::write".to_owned()],
                capability_group_keys: Vec::new(),
            },
        ];
        assert!(matches!(
            sort_and_validate_role_mappings(&mut mappings),
            Err(AuthorizationStateError::InvalidRecord(_))
        ));
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
            "rpc.v1.Auth.Grants.Set",
            &AuthorizationStateError::InvalidRecord(secret.to_owned()),
        );
        assert_eq!(invalid["type"], "AuthError");
        assert_eq!(invalid["reason"], "invalid_request");
        assert!(!serde_json::to_string(&invalid).unwrap().contains(secret));
    }
}
