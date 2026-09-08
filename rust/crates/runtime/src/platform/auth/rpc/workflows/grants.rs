use serde_json::{json, Value};
use trellis_runtime_apis::auth::types::{
    AuthDeploymentsApplyRequest, AuthGrantsGetRequest, AuthGrantsGetRequestOwnerKind,
    AuthGrantsListRequest, AuthGrantsListRequestOwnerKind, AuthGrantsRevokeRequest,
    AuthGrantsRevokeRequestOwnerKind, AuthGrantsSetRequest, AuthGrantsSetRequestOwnerKind,
    AuthIssuersRevokeRequest, AuthParticipantsGetRequest, AuthParticipantsInstallRequest,
};

use super::super::{
    now_millis, require_admin, rpc_idempotency, AuthRpcProcessor, ValidatedRequest,
};
use crate::platform::auth::domain::GrantBindingReplacement;
use crate::platform::auth::{
    AuthorizationStateError, GrantBindingState, GrantOwnerKind, MutationActor,
    ParticipantBindingRecord,
};

fn mutation_actor(caller: &ValidatedRequest) -> MutationActor {
    MutationActor {
        principal_id: caller.principal_id.clone(),
        participant_id: caller.context.participant_id().to_owned(),
        owner_kind: caller.context.owner_kind(),
        owner_id: caller.context.owner_id().to_owned(),
        grant_revision: caller.context.grant_revision(),
        login_session_id: caller.context.login_session_id().map(str::to_owned),
        session_public_key: caller.session_public_key.clone(),
    }
}

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
                    mutation_actor(&caller),
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
            let owner_kind = match request.owner_kind {
                AuthGrantsGetRequestOwnerKind::User => GrantOwnerKind::User,
                AuthGrantsGetRequestOwnerKind::Deployment => GrantOwnerKind::Deployment,
            };
            if owner_kind != GrantOwnerKind::User || request.owner_id != caller.principal_id {
                require_admin(&caller)?;
            }
            let binding = repository
                .get_grant_binding(owner_kind, request.owner_id, request.participant_id)
                .await?;
            Ok(json!({"binding": binding}))
        }
        "rpc.v1.Auth.Grants.List" => {
            let mut request: AuthGrantsListRequest = serde_json::from_value(input)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            if !caller
                .platform_privileges
                .contains(&trellis_protocol::PlatformPrivilege::Admin)
            {
                if request
                    .owner_kind
                    .as_ref()
                    .is_some_and(|kind| *kind != AuthGrantsListRequestOwnerKind::User)
                    || request
                        .owner_id
                        .as_ref()
                        .is_some_and(|id| id != &caller.principal_id)
                {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "not_authorized".to_owned(),
                    ));
                }
                request.owner_kind = Some(AuthGrantsListRequestOwnerKind::User);
                request.owner_id = Some(caller.principal_id);
            }
            repository.list_grant_bindings(request).await
        }
        "rpc.v1.Auth.Participants.Get" => {
            let request: AuthParticipantsGetRequest = serde_json::from_value(input)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let revision = request
                .revision
                .map(u64::try_from)
                .transpose()
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            repository
                .get_installed_participant(request.participant_id, revision)
                .await
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
            crate::platform::auth::builtins::validate_binding_namespace(&binding)?;
            repository
                .install_participant(mutation_actor(&caller), binding, expected, idempotency)
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
            crate::platform::auth::builtins::validate_binding_namespace(&binding)?;
            let resources = crate::platform::auth::resources::provision_deployment_resources(
                &processor.client,
                &binding,
                &request.deployment_id,
                now,
            )
            .await?;
            repository
                .apply_deployment(
                    mutation_actor(&caller),
                    request.deployment_id,
                    binding,
                    request.optional_capabilities.unwrap_or_default(),
                    resources,
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
            let owner_kind = match request.owner_kind {
                AuthGrantsSetRequestOwnerKind::User => GrantOwnerKind::User,
                AuthGrantsSetRequestOwnerKind::Deployment => GrantOwnerKind::Deployment,
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
            let installed_revision = u64::try_from(request.installed_revision)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            repository
                .admin_set_grant_binding(
                    mutation_actor(&caller),
                    GrantBindingReplacement {
                        owner_kind,
                        owner_id: request.owner_id,
                        participant_id: request.participant_id,
                        installed_revision,
                        grants,
                        platform_privileges,
                        expected_revision: expected,
                        state: GrantBindingState::Active,
                        expires_at: request.expires_at,
                        provenance: None,
                    },
                    idempotency,
                )
                .await
        }
        "rpc.v1.Auth.Grants.Revoke" => {
            let request: AuthGrantsRevokeRequest = serde_json::from_value(input.clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let expected = u64::try_from(request.expected_revision)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let owner_kind = match request.owner_kind {
                AuthGrantsRevokeRequestOwnerKind::User => GrantOwnerKind::User,
                AuthGrantsRevokeRequestOwnerKind::Deployment => GrantOwnerKind::Deployment,
            };
            if owner_kind != GrantOwnerKind::User || request.owner_id != caller.principal_id {
                require_admin(&caller)?;
            }
            let idempotency = rpc_idempotency(
                "Auth.Grants.Revoke",
                &caller.principal_id,
                &request.idempotency_key,
                &input,
                now,
            )?;
            repository
                .revoke_grant_binding(
                    mutation_actor(&caller),
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
