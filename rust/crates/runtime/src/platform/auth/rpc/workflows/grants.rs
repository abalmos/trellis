use serde_json::{json, Value};
use trellis_runtime_apis::auth::types::{
    AuthDeploymentsApplyRequest, AuthGrantsGetRequest, AuthGrantsRevokeRequest,
    AuthGrantsSetRequest, AuthIssuersRevokeRequest, AuthParticipantsInstallRequest,
};

use super::super::{
    now_millis, require_admin, rpc_idempotency, AuthRpcProcessor, ValidatedRequest,
};
use crate::platform::auth::{
    AuthorizationStateError, GrantBinding, GrantBindingState, GrantOwnerKind,
    ParticipantBindingRecord,
};

pub(super) async fn dispatch(
    processor: &AuthRpcProcessor,
    subject: &str,
    payload: &[u8],
    caller: ValidatedRequest,
) -> Result<Value, AuthorizationStateError> {
    let input: Value = serde_json::from_slice(payload)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let repository = processor.service.repository();
    let now = now_millis()?;
    match subject {
        "rpc.v1.Auth.Issuers.Revoke" => {
            require_admin(&caller)?;
            let request: AuthIssuersRevokeRequest = serde_json::from_value(input.clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let idempotency = rpc_idempotency(
                "Auth.Issuers.Revoke",
                &caller.principal_id,
                &request.idempotency_key,
                &input,
                now,
            )?;
            repository
                .revoke_issuer(
                    request.key_id,
                    caller.principal_id,
                    request.reason,
                    idempotency,
                )
                .await
        }
        "rpc.v1.Auth.Grants.Get" => {
            let request: AuthGrantsGetRequest = serde_json::from_value(input)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let owner_kind = if request.owner_kind == "user" {
                GrantOwnerKind::User
            } else {
                GrantOwnerKind::Deployment
            };
            if owner_kind != GrantOwnerKind::User || request.owner_id != caller.principal_id {
                require_admin(&caller)?;
            }
            let binding = repository
                .get_grant_binding(owner_kind, request.owner_id, request.participant_id)
                .await?;
            Ok(json!({"binding": binding}))
        }
        "rpc.v1.Auth.Participants.Install" => {
            require_admin(&caller)?;
            let request: AuthParticipantsInstallRequest = serde_json::from_value(input.clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let expected = u64::try_from(request.expected_revision)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let idempotency = rpc_idempotency(
                "Auth.Participants.Install",
                &caller.principal_id,
                &request.idempotency_key,
                &input,
                now,
            )?;
            let participant = serde_json::to_value(request.participant_artifact)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let apis = request
                .api_artifacts
                .into_iter()
                .map(|api| {
                    serde_json::to_value(api)
                        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let binding = ParticipantBindingRecord::from_artifacts(&participant, &apis, now)?;
            repository
                .install_participant(binding, expected, idempotency)
                .await
        }
        "rpc.v1.Auth.Deployments.Apply" => {
            require_admin(&caller)?;
            let request: AuthDeploymentsApplyRequest = serde_json::from_value(input.clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let expected = u64::try_from(request.expected_revision)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let idempotency = rpc_idempotency(
                "Auth.Deployments.Apply",
                &caller.principal_id,
                &request.idempotency_key,
                &input,
                now,
            )?;
            let participant = serde_json::to_value(request.participant_artifact)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let apis = request
                .api_artifacts
                .into_iter()
                .map(|api| {
                    serde_json::to_value(api)
                        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let binding = ParticipantBindingRecord::from_artifacts(&participant, &apis, now)?;
            repository
                .apply_deployment(
                    request.deployment_id,
                    binding,
                    request.optional_capabilities.unwrap_or_default(),
                    expected,
                    idempotency,
                )
                .await
        }
        "rpc.v1.Auth.Grants.Set" => {
            require_admin(&caller)?;
            let request: AuthGrantsSetRequest = serde_json::from_value(input.clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let expected = u64::try_from(request.expected_revision)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let owner_kind = if request.owner_kind == "user" {
                GrantOwnerKind::User
            } else {
                GrantOwnerKind::Deployment
            };
            let idempotency = rpc_idempotency(
                "Auth.Grants.Set",
                &caller.principal_id,
                &request.idempotency_key,
                &input,
                now,
            )?;
            let grants = serde_json::from_value(input["grants"].clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let platform_privileges =
                serde_json::from_value(input["platformPrivileges"].clone())
                    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            repository
                .set_grant_binding(
                    GrantBinding {
                        owner_kind,
                        owner_id: request.owner_id,
                        participant_id: request.participant_id,
                        installed_revision: 1,
                        grants,
                        platform_privileges,
                        revision: 1,
                        state: GrantBindingState::Active,
                        expires_at: request.expires_at,
                        provenance: None,
                    },
                    expected,
                    idempotency,
                )
                .await
        }
        "rpc.v1.Auth.Grants.Revoke" => {
            require_admin(&caller)?;
            let request: AuthGrantsRevokeRequest = serde_json::from_value(input.clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let expected = u64::try_from(request.expected_revision)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let owner_kind = if request.owner_kind == "user" {
                GrantOwnerKind::User
            } else {
                GrantOwnerKind::Deployment
            };
            let idempotency = rpc_idempotency(
                "Auth.Grants.Revoke",
                &caller.principal_id,
                &request.idempotency_key,
                &input,
                now,
            )?;
            repository
                .revoke_grant_binding(
                    owner_kind,
                    request.owner_id,
                    request.participant_id,
                    expected,
                    idempotency,
                )
                .await
        }
        _ => Err(AuthorizationStateError::InvalidRecord(format!(
            "unknown grants operation: {subject}"
        ))),
    }
}
