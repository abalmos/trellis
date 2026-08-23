use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::future::BoxFuture;
use serde_json::{json, Value};
use trellis_rs::client::SessionAuth;
use trellis_rs::sdk::auth::operations::AuthDeviceUserAuthoritiesResolveOperation;
use trellis_rs::sdk::auth::types::{
    AuthDeviceUserAuthoritiesResolveInput, AuthDeviceUserAuthoritiesResolveOutput,
    AuthDeviceUserAuthoritiesResolveProgress,
};
use trellis_rs::service::{
    AcceptedOperation, OperationRefData, OperationSnapshot, OperationState, RequestContext, Router,
    ServerError, ServiceOperationProvider,
};

use super::auth::{
    AuthService, AuthorityEvidenceRepository, ClaimActivationReviewInput,
    DecideActivationReviewInput, DeploymentRepository, DeviceActivationReviewRecord,
    DeviceActivationReviewState, DeviceDelegationRecord, DeviceDelegationState, DeviceReviewMode,
    IdempotencyResultRecord, PostCommitActionRecord, ProvisioningRepository,
    SqliteAuthorizationStore,
};
use crate::shutdown::StopHandle;
use crate::supervisor::RuntimeError;

const OPERATION: &str = "Auth.DeviceUserAuthorities.Resolve";

pub(crate) struct AuthOperationRuntime {
    client: async_nats::Client,
    router: Router,
    verifier: super::auth::verifier::RuntimeAuthVerifier,
}

impl AuthOperationRuntime {
    pub(crate) fn new(
        client: async_nats::Client,
        _auth: SessionAuth,
        service: AuthService<SqliteAuthorizationStore>,
        verifier: super::auth::verifier::RuntimeAuthVerifier,
    ) -> Self {
        let mut router = Router::new();
        router.register_operation_provider::<AuthDeviceUserAuthoritiesResolveOperation, _>(
            AuthResolveProvider { service },
        );
        Self {
            client,
            router,
            verifier,
        }
    }

    pub(crate) async fn run(self, stop: StopHandle) -> Result<(), RuntimeError> {
        tokio::select! {
            result = trellis_rs::service::internal::run_builtin_authenticated_router(
                self.client,
                "trellis.auth@v1",
                &[
                    "operations.v1.Auth.DeviceUserAuthorities.Resolve",
                    "operations.v1.Auth.DeviceUserAuthorities.Resolve.>",
                ],
                self.router,
                self.verifier,
            ) => result.map_err(|error| RuntimeError::Platform(error.to_string())),
            () = stop.stopped() => Ok(()),
        }
    }
}

struct AuthResolveProvider {
    service: AuthService<SqliteAuthorizationStore>,
}

impl ServiceOperationProvider<AuthDeviceUserAuthoritiesResolveOperation> for AuthResolveProvider {
    fn start(
        &self,
        context: RequestContext,
        input: AuthDeviceUserAuthoritiesResolveInput,
    ) -> BoxFuture<
        'static,
        Result<
            AcceptedOperation<
                AuthDeviceUserAuthoritiesResolveProgress,
                AuthDeviceUserAuthoritiesResolveOutput,
            >,
            ServerError,
        >,
    > {
        let service = self.service.clone();
        Box::pin(async move {
            let caller = caller_principal_id(&context)?;
            claim_activation(&service, caller, &input).await?;
            approve_unreviewed_activation(&service, caller, &input).await?;
            let snapshot = resolve_snapshot(&service, &context, &input.flow_id).await?;
            Ok(AcceptedOperation {
                kind: "accepted".to_owned(),
                operation_ref: OperationRefData {
                    id: input.flow_id,
                    service: "trellis.auth@v1".to_owned(),
                    operation: OPERATION.to_owned(),
                },
                snapshot,
                transfer: None,
            })
        })
    }

    fn get(
        &self,
        context: RequestContext,
        operation_id: String,
    ) -> BoxFuture<
        'static,
        Result<
            OperationSnapshot<
                AuthDeviceUserAuthoritiesResolveProgress,
                AuthDeviceUserAuthoritiesResolveOutput,
            >,
            ServerError,
        >,
    > {
        let service = self.service.clone();
        Box::pin(async move { resolve_by_id(&service, &context, operation_id).await })
    }

    fn wait(
        &self,
        context: RequestContext,
        operation_id: String,
    ) -> BoxFuture<
        'static,
        Result<
            OperationSnapshot<
                AuthDeviceUserAuthoritiesResolveProgress,
                AuthDeviceUserAuthoritiesResolveOutput,
            >,
            ServerError,
        >,
    > {
        let service = self.service.clone();
        Box::pin(async move { wait_for_resolution(&service, &context, operation_id).await })
    }
}

async fn claim_activation(
    service: &AuthService<SqliteAuthorizationStore>,
    caller: &str,
    input: &AuthDeviceUserAuthoritiesResolveInput,
) -> Result<(), ServerError> {
    let now = now_ms()?;
    service
        .expire_due_activation_reviews(now)
        .await
        .map_err(server_error)?;
    let review = service
        .repository()
        .get_activation_review(&input.flow_id)
        .await
        .map_err(server_error)?
        .ok_or_else(|| ServerError::Nats("activation review not found".to_owned()))?;
    let expected_confirmation_code = review
        .payload
        .get("confirmationCode")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ServerError::Nats("activation review is missing confirmation evidence".to_owned())
        })?;
    if !expected_confirmation_code.eq_ignore_ascii_case(&input.confirmation_code) {
        return Err(ServerError::Nats(
            "activation confirmation code is invalid".to_owned(),
        ));
    }
    if let Some(activated_by) = review.activated_by_user_principal_id.as_deref() {
        return if activated_by == caller {
            Ok(())
        } else {
            Err(ServerError::Nats(
                "activation review belongs to another user".to_owned(),
            ))
        };
    }
    if !matches!(
        review.state,
        DeviceActivationReviewState::Pending | DeviceActivationReviewState::Approved
    ) {
        return Ok(());
    }
    let profile = service
        .repository()
        .get_deployment_profile(&review.deployment_id)
        .await
        .map_err(server_error)?
        .ok_or_else(|| ServerError::Nats("activation deployment not found".to_owned()))?;
    let delegation = (review.state == DeviceActivationReviewState::Approved
        && profile.requires_device_delegation)
        .then(|| DeviceDelegationRecord {
            principal_id: review.principal_id.clone(),
            deployment_id: review.deployment_id.clone(),
            required: true,
            state: DeviceDelegationState::Active,
            expires_at: None,
        });
    let mut requested = requested_event(&review, caller, now)?;
    let requested_predecessor = match (review.state, profile.review_mode) {
        (DeviceActivationReviewState::Approved, _) => Some("approved"),
        (DeviceActivationReviewState::Pending, Some(DeviceReviewMode::Required)) => {
            Some("review-requested")
        }
        (DeviceActivationReviewState::Pending, Some(DeviceReviewMode::None)) => None,
        _ => {
            return Err(ServerError::Nats(
                "device activation review policy is invalid".to_owned(),
            ));
        }
    };
    if let Some(event) = requested_predecessor {
        requested.predecessor_action_id = Some(
            crate::platform::auth::activation_review_event_action_id(&review.review_id, event)
                .map_err(server_error)?,
        );
    }
    let mut actions = vec![requested];
    if delegation.is_some() {
        actions.push(resolved_event(&review, now, "active")?);
    }
    service
        .claim_activation_review(ClaimActivationReviewInput {
            review_id: review.review_id.clone(),
            expected_version: review.version,
            activated_by_user_principal_id: caller.to_owned(),
            now,
            delegation,
            idempotency: IdempotencyResultRecord {
                scope_key: resolve_scope_key(
                    "device.user-authority.resolve.claim",
                    caller,
                    &review.review_id,
                )?,
                purpose: "device.user-authority.resolve.claim".to_owned(),
                signer_id: caller.to_owned(),
                request_id: review.review_id.clone(),
                request_digest: review.request_digest.clone(),
                result: Value::Null,
                created_at: now,
                expires_at: now
                    .checked_add(86_400_000)
                    .ok_or_else(|| ServerError::Nats("idempotency expiry overflow".to_owned()))?,
            },
            actions,
        })
        .await
        .map_err(server_error)?;
    Ok(())
}

async fn approve_unreviewed_activation(
    service: &AuthService<SqliteAuthorizationStore>,
    caller: &str,
    input: &AuthDeviceUserAuthoritiesResolveInput,
) -> Result<(), ServerError> {
    let review = service
        .repository()
        .get_activation_review(&input.flow_id)
        .await
        .map_err(server_error)?
        .ok_or_else(|| ServerError::Nats("activation review not found".to_owned()))?;
    if review.state != DeviceActivationReviewState::Pending {
        return Ok(());
    }
    let profile = service
        .repository()
        .get_deployment_profile(&review.deployment_id)
        .await
        .map_err(server_error)?
        .ok_or_else(|| ServerError::Nats("activation deployment not found".to_owned()))?;
    if profile.review_mode == Some(DeviceReviewMode::Required) {
        return Ok(());
    }
    if profile.review_mode != Some(DeviceReviewMode::None) {
        return Err(ServerError::Nats(
            "device activation review policy is invalid".to_owned(),
        ));
    }
    let now = now_ms()?;
    let mut approved = approved_event(&review, caller, now)?;
    approved.predecessor_action_id = Some(
        crate::platform::auth::activation_review_event_action_id(&review.review_id, "requested")
            .map_err(server_error)?,
    );
    let mut resolved = resolved_event(&review, now, "active")?;
    resolved.predecessor_action_id = Some(
        crate::platform::auth::activation_review_event_action_id(&review.review_id, "approved")
            .map_err(server_error)?,
    );
    service
        .decide_activation_review(DecideActivationReviewInput {
            review_id: review.review_id.clone(),
            expected_version: review.version,
            state: DeviceActivationReviewState::Approved,
            decided_at: now,
            decided_by: caller.to_owned(),
            reason: None,
            delegation: profile
                .requires_device_delegation
                .then(|| DeviceDelegationRecord {
                    principal_id: review.principal_id.clone(),
                    deployment_id: review.deployment_id.clone(),
                    required: true,
                    state: DeviceDelegationState::Active,
                    expires_at: None,
                }),
            activate_device: true,
            idempotency: IdempotencyResultRecord {
                scope_key: resolve_scope_key(
                    "device.user-authority.resolve.approve",
                    caller,
                    &review.review_id,
                )?,
                purpose: "device.user-authority.resolve.approve".to_owned(),
                signer_id: caller.to_owned(),
                request_id: review.review_id.clone(),
                request_digest: review.request_digest.clone(),
                result: Value::Null,
                created_at: now,
                expires_at: now
                    .checked_add(86_400_000)
                    .ok_or_else(|| ServerError::Nats("idempotency expiry overflow".to_owned()))?,
            },
            actions: vec![approved, resolved],
        })
        .await
        .map_err(server_error)?;
    Ok(())
}

async fn resolve_by_id(
    service: &AuthService<SqliteAuthorizationStore>,
    context: &RequestContext,
    operation_id: String,
) -> Result<
    OperationSnapshot<
        AuthDeviceUserAuthoritiesResolveProgress,
        AuthDeviceUserAuthoritiesResolveOutput,
    >,
    ServerError,
> {
    service
        .expire_due_activation_reviews(now_ms()?)
        .await
        .map_err(server_error)?;
    resolve_snapshot(service, context, &operation_id).await
}

async fn wait_for_resolution(
    service: &AuthService<SqliteAuthorizationStore>,
    context: &RequestContext,
    operation_id: String,
) -> Result<
    OperationSnapshot<
        AuthDeviceUserAuthoritiesResolveProgress,
        AuthDeviceUserAuthoritiesResolveOutput,
    >,
    ServerError,
> {
    loop {
        let snapshot = resolve_by_id(service, context, operation_id.clone()).await?;
        if snapshot.state != OperationState::Running {
            return Ok(snapshot);
        }
        let revision = snapshot.revision;
        let waiter = service.activation_review_waiter(&operation_id).await;
        let snapshot = resolve_by_id(service, context, operation_id.clone()).await?;
        if snapshot.state != OperationState::Running {
            return Ok(snapshot);
        }
        if snapshot.revision != revision {
            continue;
        }
        let review = service
            .repository()
            .get_activation_review(&operation_id)
            .await
            .map_err(server_error)?
            .ok_or_else(|| ServerError::Nats("activation review not found".to_owned()))?;
        let remaining_ms = review.expires_at.saturating_sub(now_ms()?);
        tokio::select! {
            () = waiter.wait() => {}
            () = tokio::time::sleep(std::time::Duration::from_millis(
                u64::try_from(remaining_ms).unwrap_or(0),
            )) => {
                service
                    .expire_due_activation_reviews(now_ms()?)
                    .await
                    .map_err(server_error)?;
            }
        }
    }
}

async fn resolve_snapshot(
    service: &AuthService<SqliteAuthorizationStore>,
    context: &RequestContext,
    flow_id: &str,
) -> Result<
    OperationSnapshot<
        AuthDeviceUserAuthoritiesResolveProgress,
        AuthDeviceUserAuthoritiesResolveOutput,
    >,
    ServerError,
> {
    let review = service
        .repository()
        .get_activation_review(flow_id)
        .await
        .map_err(server_error)?
        .ok_or_else(|| ServerError::Nats("activation review not found".to_owned()))?;
    let caller = caller_principal_id(context)?;
    if review
        .activated_by_user_principal_id
        .as_deref()
        .is_some_and(|activated_by| activated_by != caller)
    {
        return Err(ServerError::Nats(
            "activation review belongs to another user".to_owned(),
        ));
    }
    match review.state {
        DeviceActivationReviewState::Pending => Ok(snapshot(
            &review,
            OperationState::Running,
            Some(serde_json::from_value(
                json!({"state": "review_pending", "retryAfterMs": 1_000}),
            )?),
            None,
        )),
        DeviceActivationReviewState::Approved => {
            let profile = service
                .repository()
                .get_deployment_profile(&review.deployment_id)
                .await
                .map_err(server_error)?
                .ok_or_else(|| ServerError::Nats("activation deployment not found".to_owned()))?;
            let device = service
                .repository()
                .get_device(&review.principal_id, &review.deployment_id)
                .await
                .map_err(server_error)?
                .ok_or_else(|| ServerError::Nats("activation device not found".to_owned()))?;
            let delegation = service
                .repository()
                .get_device_delegation(&review.principal_id, &review.deployment_id)
                .await
                .map_err(server_error)?;
            if profile.requires_device_delegation && review.activated_by_user_principal_id.is_none()
            {
                return Ok(snapshot(
                    &review,
                    OperationState::Running,
                    Some(serde_json::from_value(
                        json!({"state": "review_pending", "retryAfterMs": 1_000}),
                    )?),
                    None,
                ));
            }
            if profile.requires_device_delegation
                && delegation
                    .as_ref()
                    .is_none_or(|delegation| delegation.state != DeviceDelegationState::Active)
            {
                return Err(ServerError::Nats(
                    "approved activation is missing its required delegation".to_owned(),
                ));
            }
            let participant_id = service
                .repository()
                .get_deployment_evidence(&review.deployment_id)
                .await
                .map_err(server_error)?
                .map(|deployment| deployment.participant_id);
            let output = serde_json::from_value(json!({
                "device": {
                    "instanceId": review.instance_id,
                    "deploymentId": review.deployment_id,
                    "principalId": review.principal_id,
                    "identityPublicKey": null,
                    "identityKeyId": null,
                    "participantId": participant_id,
                    "state": "active",
                    "administrativeApproval": "approved",
                    "delegationRequired": profile.requires_device_delegation,
                    "delegationState": "active",
                    "delegationExpiresAt": delegation.and_then(|delegation| delegation.expires_at),
                    "createdAt": device.created_at,
                    "updatedAt": device.updated_at,
                    "version": device.version,
                },
                "review": {
                    "reviewId": review.review_id,
                    "deploymentId": review.deployment_id,
                    "instanceId": review.instance_id,
                    "devicePrincipalId": review.principal_id,
                    "activatedByUserPrincipalId": review.activated_by_user_principal_id,
                    "state": "approved",
                    "requestedAt": review.requested_at,
                    "expiresAt": review.expires_at,
                    "decidedAt": review.decided_at,
                    "decidedBy": review.decided_by,
                    "reason": review.reason,
                    "version": review.version,
                },
                "authority": null,
            }))?;
            Ok(snapshot(
                &review,
                OperationState::Completed,
                None,
                Some(output),
            ))
        }
        DeviceActivationReviewState::Rejected => {
            let device = service
                .repository()
                .get_device(&review.principal_id, &review.deployment_id)
                .await
                .map_err(server_error)?
                .ok_or_else(|| ServerError::Nats("activation device not found".to_owned()))?;
            let profile = service
                .repository()
                .get_deployment_profile(&review.deployment_id)
                .await
                .map_err(server_error)?
                .ok_or_else(|| ServerError::Nats("activation deployment not found".to_owned()))?;
            let output = serde_json::from_value(json!({
                "device": {
                    "instanceId": review.instance_id,
                    "deploymentId": review.deployment_id,
                    "principalId": review.principal_id,
                    "identityPublicKey": null,
                    "identityKeyId": null,
                    "participantId": profile.participant_id,
                    "state": device.state,
                    "administrativeApproval": "rejected",
                    "delegationRequired": profile.requires_device_delegation,
                    "delegationState": "missing",
                    "delegationExpiresAt": null,
                    "createdAt": device.created_at,
                    "updatedAt": device.updated_at,
                    "version": device.version,
                },
                "review": {
                    "reviewId": review.review_id,
                    "deploymentId": review.deployment_id,
                    "instanceId": review.instance_id,
                    "devicePrincipalId": review.principal_id,
                    "activatedByUserPrincipalId": review.activated_by_user_principal_id,
                    "state": "rejected",
                    "requestedAt": review.requested_at,
                    "expiresAt": review.expires_at,
                    "decidedAt": review.decided_at,
                    "decidedBy": review.decided_by,
                    "reason": review.reason,
                    "version": review.version,
                },
                "authority": null,
            }))?;
            Ok(snapshot(
                &review,
                OperationState::Completed,
                None,
                Some(output),
            ))
        }
        DeviceActivationReviewState::Expired => {
            let device = service
                .repository()
                .get_device(&review.principal_id, &review.deployment_id)
                .await
                .map_err(server_error)?
                .ok_or_else(|| ServerError::Nats("activation device not found".to_owned()))?;
            let profile = service
                .repository()
                .get_deployment_profile(&review.deployment_id)
                .await
                .map_err(server_error)?
                .ok_or_else(|| ServerError::Nats("activation deployment not found".to_owned()))?;
            let output = serde_json::from_value(json!({
                "device": {
                    "instanceId": review.instance_id,
                    "deploymentId": review.deployment_id,
                    "principalId": review.principal_id,
                    "identityPublicKey": null,
                    "identityKeyId": null,
                    "participantId": profile.participant_id,
                    "state": device.state,
                    "administrativeApproval": if review.decided_at.is_some() { "approved" } else { "pending" },
                    "delegationRequired": profile.requires_device_delegation,
                    "delegationState": "missing",
                    "delegationExpiresAt": null,
                    "createdAt": device.created_at,
                    "updatedAt": device.updated_at,
                    "version": device.version,
                },
                "review": {
                    "reviewId": review.review_id,
                    "deploymentId": review.deployment_id,
                    "instanceId": review.instance_id,
                    "devicePrincipalId": review.principal_id,
                    "activatedByUserPrincipalId": review.activated_by_user_principal_id,
                    "state": "expired",
                    "requestedAt": review.requested_at,
                    "expiresAt": review.expires_at,
                    "decidedAt": review.decided_at,
                    "decidedBy": review.decided_by,
                    "reason": review.reason,
                    "version": review.version,
                },
                "authority": null,
            }))?;
            Ok(snapshot(
                &review,
                OperationState::Completed,
                None,
                Some(output),
            ))
        }
    }
}

#[allow(clippy::result_large_err)]
fn resolve_scope_key(purpose: &str, caller: &str, review_id: &str) -> Result<String, ServerError> {
    trellis_protocol::digest_json(&json!({
        "purpose": purpose,
        "signerId": caller,
        "requestId": review_id,
    }))
    .map_err(|error| ServerError::Nats(error.to_string()))
}

fn snapshot(
    review: &DeviceActivationReviewRecord,
    state: OperationState,
    progress: Option<AuthDeviceUserAuthoritiesResolveProgress>,
    output: Option<AuthDeviceUserAuthoritiesResolveOutput>,
) -> OperationSnapshot<
    AuthDeviceUserAuthoritiesResolveProgress,
    AuthDeviceUserAuthoritiesResolveOutput,
> {
    OperationSnapshot {
        id: Some(review.review_id.clone()),
        service: Some("trellis.auth@v1".to_owned()),
        operation: Some(OPERATION.to_owned()),
        revision: review.version,
        state,
        created_at: None,
        updated_at: None,
        completed_at: None,
        progress,
        transfer: None,
        output,
        error: None,
    }
}

#[allow(clippy::result_large_err)]
fn caller_principal_id(context: &RequestContext) -> Result<&str, ServerError> {
    let caller = context
        .caller
        .as_ref()
        .ok_or_else(|| ServerError::Nats("authenticated user principal is missing".to_owned()))?;
    if caller.principal.kind != trellis_protocol::AuthorizationPrincipalKindV1::User {
        return Err(ServerError::Nats(
            "device activation requires a user principal".to_owned(),
        ));
    }
    if caller.principal.id.is_empty() {
        return Err(ServerError::Nats(
            "authenticated user principal is missing".to_owned(),
        ));
    }
    Ok(&caller.principal.id)
}

#[allow(clippy::result_large_err)]
fn requested_event(
    review: &DeviceActivationReviewRecord,
    caller: &str,
    now: i64,
) -> Result<PostCommitActionRecord, ServerError> {
    activation_event(
        review,
        "requested",
        "Auth.DeviceUserAuthorities.Requested",
        now,
        json!({
            "userPrincipalId": caller,
            "requestedAt": now,
        }),
    )
}

#[allow(clippy::result_large_err)]
fn approved_event(
    review: &DeviceActivationReviewRecord,
    caller: &str,
    now: i64,
) -> Result<PostCommitActionRecord, ServerError> {
    activation_event(
        review,
        "approved",
        "Auth.DeviceUserAuthorities.Approved",
        now,
        json!({
            "approvedBy": caller,
            "approvedAt": now,
        }),
    )
}

#[allow(clippy::result_large_err)]
fn resolved_event(
    review: &DeviceActivationReviewRecord,
    now: i64,
    state: &str,
) -> Result<PostCommitActionRecord, ServerError> {
    activation_event(
        review,
        "resolved",
        "Auth.DeviceUserAuthorities.Resolved",
        now,
        json!({ "state": state }),
    )
}

#[allow(clippy::result_large_err)]
fn activation_event(
    review: &DeviceActivationReviewRecord,
    suffix: &str,
    event_type: &str,
    now: i64,
    fields: Value,
) -> Result<PostCommitActionRecord, ServerError> {
    crate::platform::auth::activation_review_event(review, suffix, event_type, now, fields)
        .map_err(server_error)
}

#[allow(clippy::result_large_err)]
fn now_ms() -> Result<i64, ServerError> {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(server_error)?
            .as_millis(),
    )
    .map_err(|_| ServerError::Nats("current time exceeds i64 milliseconds".to_owned()))
}

fn server_error(error: impl std::fmt::Display) -> ServerError {
    tracing::warn!(%error, "auth operation failed");
    ServerError::Nats("auth_operation_failed".to_owned())
}

#[cfg(test)]
mod tests {
    use super::server_error;

    #[test]
    fn operation_errors_never_expose_internal_causes() {
        let secret = "postgres://admin:secret@internal/auth";
        let encoded = format!("{:?}", server_error(secret));
        assert!(!encoded.contains(secret));
        assert!(encoded.contains("auth_operation_failed"));
    }
}
