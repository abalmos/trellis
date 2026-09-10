use serde_json::{json, Value};
use trellis_runtime_apis::types::{
    AuthDeploymentsApplyRequest, AuthGrantsGetRequest, AuthGrantsGetRequestOwnerKind,
    AuthGrantsListRequest, AuthGrantsListRequestOwnerId, AuthGrantsListRequestOwnerKind,
    AuthGrantsRevokeRequest, AuthGrantsRevokeRequestOwnerKind, AuthGrantsSetRequest,
    AuthGrantsSetRequestOwnerKind, AuthIssuersRevokeRequest, AuthParticipantsGetRequest,
    AuthParticipantsInstallRequest,
};

use super::super::{
    mutation_actor, now_millis, require_admin, rpc_idempotency, AuthRpcProcessor, ValidatedRequest,
};
use crate::platform::auth::domain::GrantBindingReplacement;
use crate::platform::auth::evidence::PackageEvidenceInput;
use crate::platform::auth::{
    AuthorizationStateError, GrantBindingState, GrantOwnerKind, ParticipantBindingRecord,
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
                &request.idempotency_key.0,
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
                AuthGrantsGetRequestOwnerKind::Unknown(_) => {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "unknown grant owner kind".to_owned(),
                    ))
                }
            };
            if owner_kind != GrantOwnerKind::User || request.owner_id.0 != caller.principal_id {
                require_admin(&caller)?;
            }
            let binding = repository
                .get_grant_binding(owner_kind, request.owner_id.0, request.participant_id.0)
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
                        .is_some_and(|id| id.0 != caller.principal_id)
                {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "not_authorized".to_owned(),
                    ));
                }
                request.owner_kind = Some(AuthGrantsListRequestOwnerKind::User);
                request.owner_id = Some(AuthGrantsListRequestOwnerId(caller.principal_id));
            }
            repository.list_grant_bindings(request).await
        }
        "rpc.v1.Auth.Participants.Get" => {
            let request: AuthParticipantsGetRequest = serde_json::from_value(input)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let revision = request
                .revision
                .map(|revision| u64::try_from(revision.0 .0))
                .transpose()
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            repository
                .get_installed_participant(request.participant_id.0, revision)
                .await
        }
        "rpc.v1.Auth.Participants.Install" => {
            require_admin(&caller)?;
            let request: AuthParticipantsInstallRequest = serde_json::from_value(input.clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let expected = request.expected_revision.0 .0;
            let idempotency = rpc_idempotency(
                "Auth.Participants.Install",
                &caller.principal_id,
                &request.idempotency_key.0,
                &input,
                now,
            )?;
            let evidence = PackageEvidenceInput::from_generated_wire(
                request.package_evidence,
                request.participant_path,
                request.package_digest,
            )?;
            let root_package = evidence.package_evidence.root_package.clone();
            let (binding, evidence_json) =
                ParticipantBindingRecord::from_package_evidence(&evidence, now)?;
            repository
                .install_participant(
                    mutation_actor(&caller),
                    binding,
                    root_package,
                    evidence_json,
                    request.platform_trust.unwrap_or(false),
                    expected,
                    idempotency,
                )
                .await
        }
        "rpc.v1.Auth.Deployments.Apply" => {
            require_admin(&caller)?;
            let request: AuthDeploymentsApplyRequest = serde_json::from_value(input.clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let expected = request.expected_revision.0 .0;
            let idempotency = rpc_idempotency(
                "Auth.Deployments.Apply",
                &caller.principal_id,
                &request.idempotency_key.0,
                &input,
                now,
            )?;
            let evidence = PackageEvidenceInput::from_generated_wire(
                request.package_evidence,
                request.participant_path,
                request.package_digest,
            )?;
            let root_package = evidence.package_evidence.root_package.clone();
            let (binding, evidence_json) =
                ParticipantBindingRecord::from_package_evidence(&evidence, now)?;
            let resources = crate::platform::auth::resources::provision_deployment_resources(
                &processor.client,
                &binding,
                &request.deployment_id.0,
                now,
            )
            .await?;
            repository
                .apply_deployment(
                    mutation_actor(&caller),
                    request.deployment_id.0,
                    (
                        binding,
                        root_package,
                        evidence_json,
                        request
                            .optional_capabilities
                            .unwrap_or_default()
                            .into_iter()
                            .map(|capability| capability.0)
                            .collect(),
                        resources,
                    ),
                    expected,
                    idempotency,
                )
                .await
        }
        "rpc.v1.Auth.Grants.Set" => {
            require_admin(&caller)?;
            let request: AuthGrantsSetRequest = serde_json::from_value(input.clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let expected = request.expected_revision.0 .0;
            let owner_kind = match request.owner_kind {
                AuthGrantsSetRequestOwnerKind::User => GrantOwnerKind::User,
                AuthGrantsSetRequestOwnerKind::Deployment => GrantOwnerKind::Deployment,
                AuthGrantsSetRequestOwnerKind::Unknown(_) => {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "unknown grant owner kind".to_owned(),
                    ))
                }
            };
            let idempotency = rpc_idempotency(
                "Auth.Grants.Set",
                &caller.principal_id,
                &request.idempotency_key.0,
                &input,
                now,
            )?;
            let grants = serde_json::from_value(input["grants"].clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let platform_privileges =
                serde_json::from_value(input["platformPrivileges"].clone())
                    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let installed_revision = u64::try_from(request.installed_revision.0 .0)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let expires_at = input
                .get("expiresAt")
                .filter(|value| !value.is_null())
                .and_then(Value::as_str)
                .map(str::parse::<i64>)
                .transpose()
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            repository
                .admin_set_grant_binding(
                    mutation_actor(&caller),
                    GrantBindingReplacement {
                        owner_kind,
                        owner_id: request.owner_id.0,
                        participant_id: request.participant_id.0,
                        installed_revision,
                        grants,
                        platform_privileges,
                        expected_revision: expected,
                        expected_current_installed_revision: None,
                        state: GrantBindingState::Active,
                        expires_at,
                        provenance: None,
                    },
                    idempotency,
                )
                .await
        }
        "rpc.v1.Auth.Grants.Revoke" => {
            let request: AuthGrantsRevokeRequest = serde_json::from_value(input.clone())
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let expected = request.expected_revision.0 .0;
            let owner_kind = match request.owner_kind {
                AuthGrantsRevokeRequestOwnerKind::User => GrantOwnerKind::User,
                AuthGrantsRevokeRequestOwnerKind::Deployment => GrantOwnerKind::Deployment,
                AuthGrantsRevokeRequestOwnerKind::Unknown(_) => {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "unknown grant owner kind".to_owned(),
                    ))
                }
            };
            if owner_kind != GrantOwnerKind::User || request.owner_id.0 != caller.principal_id {
                require_admin(&caller)?;
            }
            let idempotency = rpc_idempotency(
                "Auth.Grants.Revoke",
                &caller.principal_id,
                &request.idempotency_key.0,
                &input,
                now,
            )?;
            repository
                .revoke_grant_binding(
                    mutation_actor(&caller),
                    owner_kind,
                    request.owner_id.0,
                    request.participant_id.0,
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
