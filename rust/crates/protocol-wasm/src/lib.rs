//! WASM boundary for deterministic Trellis protocol proof operations.

#![deny(missing_docs)]

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{json, Value};
use trellis_protocol::{
    parse_api, parse_authorization_context, parse_participant,
    resolve_participant as resolve_participant_protocol,
    session_proof_request_digest as session_proof_request_digest_protocol,
    session_proof_signing_digest as session_proof_signing_digest_protocol,
    verify_authorization_context as verify_authorization_context_protocol,
    verify_authorization_event as verify_authorization_event_protocol,
    verify_authorization_request as verify_authorization_request_protocol,
    verify_session_proof as verify_session_proof_protocol, AuthorizationContextPurpose,
    AuthorizationContextRefreshSessionProofInput, AuthorizationEventProof,
    AuthorizationEventPublisher, AuthorizationEventVerificationInput, AuthorizationIssuerKey,
    AuthorizationRequestProof, AuthorizationRequestVerificationInput,
    AuthorizationVerificationPolicy, NativeBootstrapSessionProofInput, PermissionAtom,
    ProtocolError, SessionProof, SessionProofInput, SessionProofPolicy,
    UserAuthBindSessionProofInput, UserAuthRequestSessionProofInput, VerifiedAuthorizationContext,
};
use wasm_bindgen::prelude::*;

const MAXIMUM_SAFE_JSON_INTEGER: f64 = 9_007_199_254_740_991.0;

#[derive(Deserialize)]
#[serde(transparent)]
struct RequiredNullable<T>(Option<T>);

#[derive(Deserialize)]
#[serde(
    deny_unknown_fields,
    tag = "purpose",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum WireSessionProofInput {
    UserAuthBind {
        request_id: String,
        issued_at: i64,
        flow_id: String,
        session_public_key: String,
        request_digest: String,
    },
    UserAuthRequest {
        request_id: String,
        issued_at: i64,
        session_public_key: String,
        session_nkey: String,
        participant_id: String,
        participant_digest: String,
        redirect_target: String,
        request_digest: String,
    },
    ServiceBootstrap {
        origin: String,
        unsigned_request: Value,
    },
    DeviceBootstrap {
        origin: String,
        unsigned_request: Value,
    },
    DeviceEnrollment {
        origin: String,
        unsigned_request: Value,
    },
    AuthorizationContextRefresh {
        origin: String,
        session_public_key: String,
        unsigned_request: Value,
    },
}

impl TryFrom<WireSessionProofInput> for SessionProofInput {
    type Error = ProtocolError;

    fn try_from(value: WireSessionProofInput) -> Result<Self, Self::Error> {
        match value {
            WireSessionProofInput::UserAuthBind {
                request_id,
                issued_at,
                flow_id,
                session_public_key,
                request_digest,
            } => Self::user_auth_bind(UserAuthBindSessionProofInput {
                request_id,
                issued_at,
                flow_id,
                session_public_key,
                request_digest,
            }),
            WireSessionProofInput::UserAuthRequest {
                request_id,
                issued_at,
                session_public_key,
                session_nkey,
                participant_id,
                participant_digest,
                redirect_target,
                request_digest,
            } => Self::user_auth_request(UserAuthRequestSessionProofInput {
                request_id,
                issued_at,
                session_public_key,
                session_nkey,
                participant_id,
                participant_digest,
                redirect_target,
                request_digest,
            }),
            WireSessionProofInput::ServiceBootstrap {
                origin,
                unsigned_request,
            } => Self::service_bootstrap(NativeBootstrapSessionProofInput {
                origin,
                unsigned_request,
            }),
            WireSessionProofInput::DeviceBootstrap {
                origin,
                unsigned_request,
            } => Self::device_bootstrap(NativeBootstrapSessionProofInput {
                origin,
                unsigned_request,
            }),
            WireSessionProofInput::DeviceEnrollment {
                origin,
                unsigned_request,
            } => Self::device_enrollment(NativeBootstrapSessionProofInput {
                origin,
                unsigned_request,
            }),
            WireSessionProofInput::AuthorizationContextRefresh {
                origin,
                session_public_key,
                unsigned_request,
            } => {
                Self::authorization_context_refresh(AuthorizationContextRefreshSessionProofInput {
                    origin,
                    session_public_key,
                    unsigned_request,
                })
            }
        }
    }
}

fn parse_input(input_json: &str) -> Result<SessionProofInput, JsError> {
    serde_json::from_str::<WireSessionProofInput>(input_json)
        .map_err(|error| JsError::new(&error.to_string()))?
        .try_into()
        .map_err(|error: ProtocolError| JsError::new(&error.to_string()))
}

fn safe_integer(value: f64, name: &str) -> Result<i64, JsError> {
    if value.is_finite() && value.fract() == 0.0 && value.abs() <= MAXIMUM_SAFE_JSON_INTEGER {
        Ok(value as i64)
    } else {
        Err(JsError::new(&format!(
            "{name} must be an interoperable JSON safe integer"
        )))
    }
}

/// Return the canonical request digest for a JSON-encoded proof-bearing request.
#[wasm_bindgen]
pub fn session_proof_request_digest(request_json: &str) -> Result<String, JsError> {
    let request: Value =
        serde_json::from_str(request_json).map_err(|error| JsError::new(&error.to_string()))?;
    session_proof_request_digest_protocol(&request)
        .map_err(|error| JsError::new(&error.to_string()))
}

/// Return the canonical signing digest for a JSON-encoded purpose-specific input.
#[wasm_bindgen]
pub fn session_proof_signing_digest(input_json: &str) -> Result<String, JsError> {
    session_proof_signing_digest_protocol(&parse_input(input_json)?)
        .map_err(|error| JsError::new(&error.to_string()))
}

/// Parse and normalize one strict JSON-encoded session-proof envelope.
#[wasm_bindgen]
pub fn parse_session_proof(proof_json: &str) -> Result<String, JsError> {
    let value: Value =
        serde_json::from_str(proof_json).map_err(|error| JsError::new(&error.to_string()))?;
    let proof = trellis_protocol::parse_session_proof(&value)
        .map_err(|error| JsError::new(&error.to_string()))?;
    serde_json::to_string(&proof).map_err(|error| JsError::new(&error.to_string()))
}

/// Verify a JSON-encoded session proof.
#[wasm_bindgen]
pub fn verify_session_proof(
    input_json: &str,
    proof_json: &str,
    signer_public_key: &str,
    now_ms: f64,
    maximum_age_ms: f64,
    maximum_future_skew_ms: f64,
) -> Result<(), JsError> {
    let proof: SessionProof =
        serde_json::from_str(proof_json).map_err(|error| JsError::new(&error.to_string()))?;
    let now_ms = safe_integer(now_ms, "nowMs")?;
    let policy = SessionProofPolicy::new(
        safe_integer(maximum_age_ms, "maximumAgeMs")?,
        safe_integer(maximum_future_skew_ms, "maximumFutureSkewMs")?,
    )
    .map_err(|error| JsError::new(&error.to_string()))?;
    verify_session_proof_protocol(
        &parse_input(input_json)?,
        &proof,
        signer_public_key,
        now_ms,
        policy,
    )
    .map_err(|error| JsError::new(&error.to_string()))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WireAuthorizationVerificationPolicy {
    now_unix_seconds: f64,
    allowed_clock_skew_seconds: u32,
    maximum_context_lifetime_seconds: u32,
    maximum_context_bytes: usize,
    maximum_permissions: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WireAuthorizationRequest {
    subject: String,
    reply: RequiredNullable<String>,
    iat: i64,
    request_id: String,
    proof: String,
    required_permissions: Vec<PermissionAtom>,
    policy: WireAuthorizationVerificationPolicy,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WireAuthorizationEvent {
    subject: String,
    event_id: String,
    event_time: String,
    proof: String,
    required_permissions: Vec<PermissionAtom>,
    #[serde(default)]
    revoked_at: Option<i64>,
    policy: WireAuthorizationVerificationPolicy,
}

fn authorization_verification_policy(
    policy_json: &str,
) -> Result<AuthorizationVerificationPolicy, JsError> {
    let wire: WireAuthorizationVerificationPolicy =
        serde_json::from_str(policy_json).map_err(|error| JsError::new(&error.to_string()))?;
    let policy = authorization_verification_policy_from_wire(&wire)?;
    Ok(policy)
}

fn authorization_verification_policy_from_wire(
    wire: &WireAuthorizationVerificationPolicy,
) -> Result<AuthorizationVerificationPolicy, JsError> {
    let policy = AuthorizationVerificationPolicy::new(
        safe_integer(wire.now_unix_seconds, "nowUnixSeconds")?,
        wire.allowed_clock_skew_seconds,
        wire.maximum_context_lifetime_seconds,
        wire.maximum_context_bytes,
        wire.maximum_permissions,
    )
    .map_err(|error| JsError::new(&error.to_string()))?;
    Ok(policy)
}

/// Verify a live context with its authenticated online issuer entry.
#[wasm_bindgen]
pub fn verify_authorization_context(
    issuer_json: &str,
    context_json: &str,
    policy_json: &str,
) -> Result<String, JsError> {
    create_authorization_context_handle(issuer_json, context_json, policy_json, false)?.projection()
}

/// Resolve one participant against its exact native API artifacts.
#[wasm_bindgen]
pub fn resolve_participant(participant_json: &str, apis_json: &str) -> Result<String, JsError> {
    let participant_value: Value =
        serde_json::from_str(participant_json).map_err(|error| JsError::new(&error.to_string()))?;
    let participant =
        parse_participant(&participant_value).map_err(|error| JsError::new(&error.to_string()))?;
    let api_values: BTreeMap<String, Value> =
        serde_json::from_str(apis_json).map_err(|error| JsError::new(&error.to_string()))?;
    let mut apis = BTreeMap::new();
    for (id, value) in api_values {
        let api = parse_api(&value).map_err(|error| JsError::new(&error.to_string()))?;
        if api.id() != id {
            return Err(JsError::new(&format!(
                "API map key '{id}' does not match artifact id '{}'",
                api.id()
            )));
        }
        apis.insert(id, api);
    }
    let resolved = resolve_participant_protocol(&participant, &apis)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let api_artifacts = apis
        .iter()
        .map(|(id, api)| {
            Ok((
                id.clone(),
                api.normalized_value()
                    .map_err(|error| JsError::new(&error.to_string()))?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, JsError>>()?;
    let api_digests = apis
        .iter()
        .map(|(id, api)| {
            Ok((
                id.clone(),
                api.digest()
                    .map_err(|error| JsError::new(&error.to_string()))?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, JsError>>()?;
    serde_json::to_string(&json!({
        "apiArtifacts": api_artifacts,
        "apiDigests": api_digests,
        "participant": participant.normalized_value()
            .map_err(|error| JsError::new(&error.to_string()))?,
        "participantDigest": resolved.participant_digest(),
        "participantNeeds": resolved.needs(),
        "participantNeedsDigest": resolved.needs().digest()
            .map_err(|error| JsError::new(&error.to_string()))?,
        "requiredGrants": resolved.proposal().required().grant_set(),
        "optionalGrants": resolved.proposal().optional().grant_set(),
        "authorityProposal": resolved.proposal(),
    }))
    .map_err(|error| JsError::new(&error.to_string()))
}

/// Opaque Rust-owned authorization context retained for repeated proof verification.
#[wasm_bindgen]
pub struct VerifiedAuthorizationContextHandle {
    context: VerifiedAuthorizationContext,
    projection: String,
}

/// Verify an online-issued context once and retain it inside WASM.
#[wasm_bindgen]
pub fn create_authorization_context_handle(
    issuer_json: &str,
    context_json: &str,
    policy_json: &str,
    historical: bool,
) -> Result<VerifiedAuthorizationContextHandle, JsError> {
    let policy = authorization_verification_policy(policy_json)?;
    let issuer: AuthorizationIssuerKey =
        serde_json::from_str(issuer_json).map_err(|error| JsError::new(&error.to_string()))?;
    let context_value: Value =
        serde_json::from_str(context_json).map_err(|error| JsError::new(&error.to_string()))?;
    let signed_context = parse_authorization_context(&context_value)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let purpose = if historical {
        AuthorizationContextPurpose::HistoricalEvent
    } else {
        AuthorizationContextPurpose::Live
    };
    let context = verify_authorization_context_protocol(&issuer, &signed_context, &policy, purpose)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let projection = serde_json::to_string(&json!({
        "issuer": issuer,
        "contextDigest": context.context_digest(),
        "context": context.signed_context(),
    }))
    .map_err(|error| JsError::new(&error.to_string()))?;
    Ok(VerifiedAuthorizationContextHandle {
        context,
        projection,
    })
}

#[wasm_bindgen]
impl VerifiedAuthorizationContextHandle {
    /// Return the verified context projection used by the TypeScript cache.
    pub fn projection(&self) -> Result<String, JsError> {
        Ok(self.projection.clone())
    }

    /// Require the retained context to be eligible at the supplied current time.
    pub fn assert_current(&self, policy_json: &str) -> Result<(), JsError> {
        let policy = authorization_verification_policy(policy_json)?;
        self.context
            .assert_current(&policy)
            .map_err(|error| JsError::new(&error.to_string()))
    }
}

fn verified_context_projection(
    context: &VerifiedAuthorizationContext,
) -> Result<Value, ProtocolError> {
    Ok(json!({
        "ownerKind": context.signed_context().unsigned.owner_kind,
        "ownerId": context.signed_context().unsigned.owner_id,
        "grantRevision": context.signed_context().unsigned.grant_revision,
        "principalId": context.principal_id(),
        "principalKind": context.principal_kind(),
        "participantId": context.participant_id(),
        "identityKeyId": context.signed_context().unsigned.identity_key_id,
        "loginSessionId": context.signed_context().unsigned.login_session_id,
        "deploymentId": context.deployment_id(),
        "instanceId": context.instance_id(),
        "issuerKeyId": context.signed_context().unsigned.issuer_key_id,
        "connectionId": context.connection_id(),
        "sessionKey": context.signed_context().unsigned.session_key,
        "inboxPrefix": context.inbox_prefix(),
        "issuedAt": context.issued_at(),
        "notBefore": context.not_before(),
        "expiresAt": context.expires_at(),
        "grants": context.grant_set(),
        "grantDigest": context.grant_set().digest()?,
        "platformPrivileges": context.platform_privileges(),
        "extensions": context.signed_context().unsigned.extensions,
        "contextDigest": context.context_digest(),
    }))
}

fn protocol_error_result(error: &ProtocolError) -> String {
    let (code, path) = match error {
        ProtocolError::Authorization { code, path, .. } => (format!("{code:?}"), path.to_string()),
        _ => ("InvalidInput".to_owned(), String::new()),
    };
    json_result(json!({
        "ok": false,
        "error": {
            "code": code,
            "path": path,
        },
    }))
}

fn input_error_result(path: &str) -> String {
    json_result(json!({
        "ok": false,
        "error": {
            "code": "InvalidInput",
            "path": path,
        },
    }))
}

fn json_result(value: Value) -> String {
    serde_json::to_string(&value).unwrap_or_else(|_| {
        r#"{"ok":false,"error":{"code":"SerializationError","path":""}}"#.to_owned()
    })
}

fn verified_result(mut projection: Value) -> String {
    projection["ok"] = Value::Bool(true);
    json_result(projection)
}

fn request_result(
    context: &VerifiedAuthorizationContext,
    input: WireAuthorizationRequest,
    payload: &[u8],
) -> String {
    let policy = match authorization_verification_policy_from_wire(&input.policy) {
        Ok(policy) => policy,
        Err(_) => return input_error_result("/policy"),
    };
    let proof = match AuthorizationRequestProof::parse(input.proof) {
        Ok(proof) => proof,
        Err(error) => return protocol_error_result(&error),
    };
    let verified =
        match verify_authorization_request_protocol(AuthorizationRequestVerificationInput {
            context,
            subject: &input.subject,
            reply_subject: input.reply.0.as_deref(),
            raw_payload: payload,
            iat: input.iat,
            request_id: &input.request_id,
            proof: &proof,
            policy: &policy,
            required_permissions: &input.required_permissions,
        }) {
            Ok(verified) => verified,
            Err(error) => return protocol_error_result(&error),
        };
    let projection = match verified_context_projection(verified.context()) {
        Ok(projection) => projection,
        Err(error) => return protocol_error_result(&error),
    };
    verified_result(projection)
}

fn event_publisher_projection(publisher: &AuthorizationEventPublisher) -> Value {
    json!({
        "kind": publisher.kind,
        "deploymentId": publisher.deployment_id,
        "instanceId": publisher.instance_id,
        "participantId": publisher.participant_id,
        "connectionId": publisher.connection_id,
        "loginSessionId": publisher.login_session_id,
    })
}

fn event_result(
    context: &VerifiedAuthorizationContext,
    input: WireAuthorizationEvent,
    payload: &[u8],
) -> String {
    let policy = match authorization_verification_policy_from_wire(&input.policy) {
        Ok(policy) => policy,
        Err(_) => return input_error_result("/policy"),
    };
    let proof = match AuthorizationEventProof::parse(input.proof) {
        Ok(proof) => proof,
        Err(error) => return protocol_error_result(&error),
    };
    let verified = match verify_authorization_event_protocol(AuthorizationEventVerificationInput {
        context,
        subject: &input.subject,
        raw_payload: payload,
        event_id: &input.event_id,
        event_time: &input.event_time,
        proof: &proof,
        policy: &policy,
        required_permissions: &input.required_permissions,
        revoked_at: input.revoked_at,
    }) {
        Ok(verified) => verified,
        Err(error) => return protocol_error_result(&error),
    };
    let mut projection = match verified_context_projection(verified.context()) {
        Ok(projection) => projection,
        Err(error) => return protocol_error_result(&error),
    };
    projection["publisher"] = event_publisher_projection(verified.publisher());
    verified_result(projection)
}

/// Verify one context-bound authorization request proof from a JSON argument.
///
/// The result is always a JSON object. Successful results have `ok: true` and
/// contain verified caller/context metadata; rejected inputs have `ok: false`
/// and a stable authorization error code and path.
#[wasm_bindgen]
pub fn verify_authorization_request(
    context: &VerifiedAuthorizationContextHandle,
    request_json: &str,
    payload: &[u8],
) -> String {
    let input: WireAuthorizationRequest = match serde_json::from_str(request_json) {
        Ok(input) => input,
        Err(_) => return input_error_result(""),
    };
    request_result(&context.context, input, payload)
}

/// Verify one context-bound authorization event proof from a JSON argument.
///
/// The result is always a JSON object. Successful results have `ok: true` and
/// contain verified publisher/context metadata; rejected inputs have `ok: false`
/// and a stable authorization error code and path. Event context chains are
/// checked at their signed historical boundary before the strict event-time
/// window is evaluated.
#[wasm_bindgen]
pub fn verify_authorization_event(
    context: &VerifiedAuthorizationContextHandle,
    event_json: &str,
    payload: &[u8],
) -> String {
    let input: WireAuthorizationEvent = match serde_json::from_str(event_json) {
        Ok(input) => input,
        Err(_) => return input_error_result(""),
    };
    event_result(&context.context, input, payload)
}
