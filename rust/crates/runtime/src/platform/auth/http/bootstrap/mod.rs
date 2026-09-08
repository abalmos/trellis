use super::super::{AuthorizationContextBundle, AuthorizationContextIssueRequest};
use super::*;
use crate::platform::auth::{
    IssuanceConnection, IssuanceCredential, ProvisionedIdentityKind, ProvisionedIdentityState,
};

mod enroll;
pub(super) use enroll::device_enroll;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NativeBootstrapRequest {
    identity_key_id: String,
    session_key: String,
    connection_id: String,
    request_id: String,
    #[serde(rename = "iat")]
    _iat: i64,
    name: Option<String>,
    proof: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BootstrapResponse {
    server_now: i64,
    authorization_context: AuthorizationContextBundle,
    routing: BootstrapRouting,
    runtime: BootstrapRuntime,
    transports: BootstrapTransports,
    authorization: BootstrapAuthorization,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapRouting {
    bootstrap_jwt: String,
    bootstrap_jwt_expires_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapRuntime {
    connection_id: String,
    login_session_id: Option<String>,
    participant_id: String,
    inbox_prefix: String,
}

#[derive(Serialize)]
struct BootstrapTransports {
    #[serde(skip_serializing_if = "Option::is_none")]
    native: Option<BootstrapTransport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    websocket: Option<BootstrapTransport>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapTransport {
    nats_servers: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapAuthorization {
    participant_id: String,
    participant_artifact_digest: String,
    resource_runtime: ServiceResourceBindings,
}

pub(super) async fn service_bootstrap<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Json(raw): Json<Value>,
) -> Result<Json<BootstrapResponse>, HttpError>
where
    R: ContextRepository + ProvisioningRepository + Clone + Send + Sync + 'static,
    E: AuthEphemeralRepository + Clone,
{
    bootstrap(&state, raw, ProvisionedIdentityKind::Service)
        .await
        .map(Json)
}

pub(super) async fn device_bootstrap<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Json(raw): Json<Value>,
) -> Result<Json<BootstrapResponse>, HttpError>
where
    R: ContextRepository + ProvisioningRepository + Clone + Send + Sync + 'static,
    E: AuthEphemeralRepository + Clone,
{
    bootstrap(&state, raw, ProvisionedIdentityKind::Device)
        .await
        .map(Json)
}

async fn bootstrap<R, E>(
    state: &AuthHttpState<R, E>,
    raw: Value,
    expected_kind: ProvisionedIdentityKind,
) -> Result<BootstrapResponse, HttpError>
where
    R: ContextRepository + ProvisioningRepository + Clone + Send + Sync + 'static,
    E: AuthEphemeralRepository + Clone,
{
    let request: NativeBootstrapRequest = serde_json::from_value(raw.clone())
        .map_err(|_| HttpError::bad_request("invalid_native_bootstrap"))?;
    if request
        .name
        .as_ref()
        .is_some_and(|name| name.chars().count() > 128)
    {
        return Err(HttpError::bad_request("invalid_native_bootstrap"));
    }
    let mut unsigned_request = raw.clone();
    unsigned_request
        .as_object_mut()
        .ok_or_else(|| HttpError::bad_request("invalid_native_bootstrap"))?
        .remove("proof");
    let proof_input = match expected_kind {
        ProvisionedIdentityKind::Service => {
            SessionProofInput::service_bootstrap(NativeBootstrapSessionProofInput {
                origin: state.public_origin.clone(),
                unsigned_request,
            })
        }
        ProvisionedIdentityKind::Device => {
            SessionProofInput::device_bootstrap(NativeBootstrapSessionProofInput {
                origin: state.public_origin.clone(),
                unsigned_request,
            })
        }
    }
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let identity = state
        .service
        .repository()
        .get_provisioned_identity(&request.identity_key_id)
        .await?
        .ok_or_else(|| HttpError::unauthorized("identity_not_found"))?;
    if identity.kind != expected_kind || identity.state != ProvisionedIdentityState::Active {
        return Err(HttpError::unauthorized("identity_inactive"));
    }
    let proof = parse_session_proof(&request.proof)
        .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    verify_session_proof(
        &proof_input,
        &proof,
        &identity.identity_public_key,
        now_ms()?,
        state.proof_policy,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;

    let now = now_ms()?;
    let connection = IssuanceConnection {
        credential: IssuanceCredential::Native(request.identity_key_id),
        connection_id: request.connection_id,
        session_public_key: request.session_key.clone(),
    };
    issue_bootstrap(
        state,
        connection,
        request.request_id,
        proof_request_digest(&raw)
            .map_err(|_| HttpError::bad_request("invalid_native_bootstrap"))?,
        now,
    )
    .await
}

pub(super) async fn issue_bootstrap<R, E>(
    state: &AuthHttpState<R, E>,
    connection: IssuanceConnection,
    request_id: String,
    request_digest: String,
    now: i64,
) -> Result<BootstrapResponse, HttpError>
where
    R: ContextRepository + ProvisioningRepository + Clone + Send + Sync + 'static,
    E: AuthEphemeralRepository + Clone,
{
    let session_key = connection.session_public_key.clone();
    let (authorization_context, issuance) = state
        .authorization_contexts
        .issue_with_state(
            AuthorizationContextIssueRequest {
                connection,
                request_id,
                request_digest,
            },
            now / 1_000,
        )
        .await
        .map_err(map_issuance_error)?;
    let bootstrap_jwt_expires_at = authorization_context
        .context
        .get("expiresAt")
        .and_then(Value::as_i64)
        .ok_or_else(|| HttpError::internal("issued_context_invalid"))?;
    let route = state.issuer.deny_all_user_jwt(
        &session_public_key_to_user_nkey(&session_key)?,
        bootstrap_jwt_expires_at,
        now / 1_000,
    )?;
    let transports = BootstrapTransports {
        native: (!state.native_nats_servers.is_empty()).then(|| BootstrapTransport {
            nats_servers: state.native_nats_servers.clone(),
        }),
        websocket: (!state.websocket_nats_servers.is_empty()).then(|| BootstrapTransport {
            nats_servers: state.websocket_nats_servers.clone(),
        }),
    };
    if transports.native.is_none() && transports.websocket.is_none() {
        return Err(HttpError::internal("transport_unavailable"));
    }
    Ok(BootstrapResponse {
        server_now: now,
        authorization_context,
        routing: BootstrapRouting {
            bootstrap_jwt: route.jwt,
            bootstrap_jwt_expires_at: route.expires_at,
        },
        runtime: BootstrapRuntime {
            connection_id: issuance.connection_id,
            login_session_id: issuance.login_session_id,
            participant_id: issuance.participant.participant_id.clone(),
            inbox_prefix: issuance.inbox_prefix,
        },
        transports,
        authorization: BootstrapAuthorization {
            participant_id: issuance.participant.participant_id.clone(),
            participant_artifact_digest: issuance.participant.artifact_digest.clone(),
            resource_runtime: project_service_resource_bindings(
                &issuance.participant.participant_json,
                &issuance.resource_bindings,
                &issuance.participant.participant_id,
            )?,
        },
    })
}
