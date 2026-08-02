use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use trellis_rs::client::SessionAuth;
use trellis_rs::sdk::auth::operations::AuthDeviceUserAuthoritiesResolveOperation;
use trellis_rs::sdk::auth::types::{
    AuthDeviceUserAuthoritiesResolveInput, AuthDeviceUserAuthoritiesResolveOutput,
    AuthDeviceUserAuthoritiesResolveProgress,
};
use trellis_rs::service::{
    AcceptedOperation, OperationRefData, OperationSnapshot, OperationState, RequestContext, Router,
    ServerError,
};

use super::auth::{
    AuthService, AuthorityEvidenceRepository, DecideActivationReviewInput,
    DeviceActivationReviewRecord, DeviceActivationReviewState, DeviceDelegationMutation,
    DeviceDelegationRecord, DeviceDelegationState, IdempotencyResultRecord, PostCommitActionKind,
    PostCommitActionRecord, ProvisioningRepository, SqliteAuthorizationStore,
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
        let start_service = service.clone();
        let get_service = service.clone();
        let wait_service = service;
        router.register_operation::<AuthDeviceUserAuthoritiesResolveOperation, _, _, _, _, _, _, _, _>(
            move |context, input| {
                let service = start_service.clone();
                async move {
                    approve_pending_review(&service, &context, &input).await?;
                    let snapshot = resolve_snapshot(&service, &context, &input).await?;
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
                }
            },
            move |context, operation_id| {
                let service = get_service.clone();
                async move { resolve_by_id(&service, &context, operation_id).await }
            },
            move |context, operation_id| {
                let service = wait_service.clone();
                async move {
                    loop {
                        let snapshot = resolve_by_id(&service, &context, operation_id.clone()).await?;
                        if snapshot.state != OperationState::Running {
                            return Ok(snapshot);
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                    }
                }
            },
            |_context, operation_id| async move {
                Err(ServerError::InvalidOperationControlAction {
                    subject: operation_id,
                    action: "cancel".to_owned(),
                })
            },
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

async fn approve_pending_review(
    service: &AuthService<SqliteAuthorizationStore>,
    context: &RequestContext,
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
    let caller = caller_principal_id(context)?;
    let now = now_ms()?;
    service
        .decide_activation_review(DecideActivationReviewInput {
            review_id: review.review_id.clone(),
            expected_version: review.version,
            state: DeviceActivationReviewState::Approved,
            decided_at: now,
            decided_by: caller.to_owned(),
            reason: None,
            delegation: Some(DeviceDelegationRecord {
                principal_id: review.principal_id.clone(),
                deployment_id: review.deployment_id.clone(),
                required: true,
                state: DeviceDelegationState::Active,
                expires_at: None,
            }),
            idempotency: IdempotencyResultRecord {
                scope_key: format!(
                    "device.user-authority.resolve:{caller}:{}",
                    review.review_id
                ),
                purpose: "device.user-authority.resolve".to_owned(),
                signer_id: caller.to_owned(),
                request_id: review.review_id.clone(),
                request_digest: review.request_digest.clone(),
                result: Value::Null,
                created_at: now,
                expires_at: now
                    .checked_add(86_400_000)
                    .ok_or_else(|| ServerError::Nats("idempotency expiry overflow".to_owned()))?,
            },
            actions: vec![resolved_event(&review, now)],
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
    resolve_snapshot(
        service,
        context,
        &AuthDeviceUserAuthoritiesResolveInput {
            flow_id: operation_id,
        },
    )
    .await
}

async fn resolve_snapshot(
    service: &AuthService<SqliteAuthorizationStore>,
    context: &RequestContext,
    input: &AuthDeviceUserAuthoritiesResolveInput,
) -> Result<
    OperationSnapshot<
        AuthDeviceUserAuthoritiesResolveProgress,
        AuthDeviceUserAuthoritiesResolveOutput,
    >,
    ServerError,
> {
    let review = service
        .repository()
        .get_activation_review(&input.flow_id)
        .await
        .map_err(server_error)?
        .ok_or_else(|| ServerError::Nats("activation review not found".to_owned()))?;
    let caller = caller_principal_id(context)?;
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
            let mut device = service
                .repository()
                .get_device(&review.principal_id, &review.deployment_id)
                .await
                .map_err(server_error)?
                .ok_or_else(|| ServerError::Nats("activation device not found".to_owned()))?;
            let mut delegation = service
                .repository()
                .get_device_delegation(&review.principal_id, &review.deployment_id)
                .await
                .map_err(server_error)?
                .ok_or_else(|| ServerError::Nats("device delegation not found".to_owned()))?;
            if delegation.state != DeviceDelegationState::Active {
                let now = now_ms()?;
                let expected_version = device.version;
                device.version += 1;
                device.updated_at = now;
                delegation.state = DeviceDelegationState::Active;
                service
                    .repository()
                    .mutate_device_delegation(DeviceDelegationMutation {
                        device: device.clone(),
                        delegation: delegation.clone(),
                        expected_version,
                        idempotency: IdempotencyResultRecord {
                            scope_key: format!(
                                "device.user-authority.resolve:{caller}:{}",
                                review.review_id
                            ),
                            purpose: "device.user-authority.resolve".to_owned(),
                            signer_id: caller.to_owned(),
                            request_id: review.review_id.clone(),
                            request_digest: review.request_digest.clone(),
                            result: json!({"reviewId": review.review_id}),
                            created_at: now,
                            expires_at: now.checked_add(86_400_000).ok_or_else(|| {
                                ServerError::Nats("idempotency expiry overflow".to_owned())
                            })?,
                        },
                        actions: vec![resolved_event(&review, now)],
                    })
                    .await
                    .map_err(server_error)?;
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
                    "delegationRequired": true,
                    "delegationState": "active",
                    "delegationExpiresAt": delegation.expires_at,
                    "createdAt": device.created_at,
                    "updatedAt": device.updated_at,
                    "version": device.version,
                },
                "review": {
                    "reviewId": review.review_id,
                    "deploymentId": review.deployment_id,
                    "instanceId": review.instance_id,
                    "devicePrincipalId": review.principal_id,
                    "state": "approved",
                    "confirmationCode": review.request_digest.chars().take(8).collect::<String>(),
                    "requestedAt": review.requested_at,
                    "expiresAt": review.requested_at + 900_000,
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
        DeviceActivationReviewState::Rejected
        | DeviceActivationReviewState::Cancelled
        | DeviceActivationReviewState::Expired => Err(ServerError::Nats(
            "activation review is no longer approvable".to_owned(),
        )),
    }
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

fn resolved_event(review: &DeviceActivationReviewRecord, now: i64) -> PostCommitActionRecord {
    PostCommitActionRecord {
        action_id: format!("act_{}_resolved", review.review_id),
        kind: PostCommitActionKind::Event,
        payload: json!({
            "eventType": "Auth.DeviceUserAuthorities.Resolved",
            "eventSubject": format!(
                "events.v1.Auth.DeviceUserAuthorities.Resolved.{}",
                review.deployment_id,
            ),
            "eventId": format!("evt_{}_resolved", review.review_id),
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
    }
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
