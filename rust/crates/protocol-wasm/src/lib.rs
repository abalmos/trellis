//! WASM boundary for deterministic Trellis protocol proof operations.

#![deny(missing_docs)]

#[cfg(feature = "resolver")]
use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{json, Value};
#[cfg(feature = "resolver")]
use trellis_protocol::{
    parse_api, parse_participant, resolve_participant as resolve_participant_protocol,
};
use trellis_protocol::{
    parse_authorization_context, parse_issuer_manifest,
    session_proof_request_digest as session_proof_request_digest_protocol,
    session_proof_signing_digest as session_proof_signing_digest_protocol,
    verify_authorization_context as verify_authorization_context_protocol,
    verify_authorization_event as verify_authorization_event_protocol,
    verify_authorization_request as verify_authorization_request_protocol, verify_issuer_manifest,
    verify_session_proof as verify_session_proof_protocol,
    AuthorizationContextRefreshSessionProofInput, AuthorizationEventProof,
    AuthorizationEventPublisher, AuthorizationEventVerificationInput, AuthorizationRequestProof,
    AuthorizationRequestVerificationInput, AuthorizationTrustRoot, AuthorizationVerificationPolicy,
    DeviceBootstrapSessionProofInput, PermissionAtom, ProtocolError,
    ServiceBootstrapSessionProofInput, SessionProof, SessionProofInput, SessionProofPolicy,
    UserAuthRequestSessionProofInput, VerifiedAuthorizationContext,
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
        request_id: String,
        issued_at: i64,
        deployment_id: String,
        instance_id: String,
        provisioned_identity_key_id: String,
        new_session_public_key: String,
        new_session_nkey: String,
        participant_id: String,
        participant_digest: String,
        request_digest: String,
    },
    DeviceBootstrap {
        request_id: String,
        issued_at: i64,
        deployment_id: String,
        instance_id: String,
        device_identity_key_id: String,
        new_session_public_key: String,
        new_session_nkey: String,
        participant_id: String,
        participant_digest: String,
        challenge_digest: RequiredNullable<String>,
        request_digest: String,
    },
    AuthorizationContextRefresh {
        request_id: String,
        issued_at: i64,
        session_id: String,
        session_key_id: String,
        current_context_digest: RequiredNullable<String>,
        expected_participant_digest: RequiredNullable<String>,
        expected_needs_digest: RequiredNullable<String>,
        known_root_key_id: String,
        minimum_manifest_generation: i64,
        request_digest: String,
    },
}

impl TryFrom<WireSessionProofInput> for SessionProofInput {
    type Error = ProtocolError;

    fn try_from(value: WireSessionProofInput) -> Result<Self, Self::Error> {
        match value {
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
                request_id,
                issued_at,
                deployment_id,
                instance_id,
                provisioned_identity_key_id,
                new_session_public_key,
                new_session_nkey,
                participant_id,
                participant_digest,
                request_digest,
            } => Self::service_bootstrap(ServiceBootstrapSessionProofInput {
                request_id,
                issued_at,
                deployment_id,
                instance_id,
                provisioned_identity_key_id,
                new_session_public_key,
                new_session_nkey,
                participant_id,
                participant_digest,
                request_digest,
            }),
            WireSessionProofInput::DeviceBootstrap {
                request_id,
                issued_at,
                deployment_id,
                instance_id,
                device_identity_key_id,
                new_session_public_key,
                new_session_nkey,
                participant_id,
                participant_digest,
                challenge_digest,
                request_digest,
            } => Self::device_bootstrap(DeviceBootstrapSessionProofInput {
                request_id,
                issued_at,
                deployment_id,
                instance_id,
                device_identity_key_id,
                new_session_public_key,
                new_session_nkey,
                participant_id,
                participant_digest,
                challenge_digest: challenge_digest.0,
                request_digest,
            }),
            WireSessionProofInput::AuthorizationContextRefresh {
                request_id,
                issued_at,
                session_id,
                session_key_id,
                current_context_digest,
                expected_participant_digest,
                expected_needs_digest,
                known_root_key_id,
                minimum_manifest_generation,
                request_digest,
            } => {
                Self::authorization_context_refresh(AuthorizationContextRefreshSessionProofInput {
                    request_id,
                    issued_at,
                    session_id,
                    session_key_id,
                    current_context_digest: current_context_digest.0,
                    expected_participant_digest: expected_participant_digest.0,
                    expected_needs_digest: expected_needs_digest.0,
                    known_root_key_id,
                    minimum_manifest_generation,
                    request_digest,
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
    maximum_capabilities: usize,
    minimum_manifest_generation: f64,
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
    required_capabilities: Vec<String>,
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
    required_capabilities: Vec<String>,
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
        wire.maximum_capabilities,
        u64::try_from(safe_integer(
            wire.minimum_manifest_generation,
            "minimumManifestGeneration",
        )?)
        .map_err(|_| JsError::new("minimumManifestGeneration must be positive"))?,
    )
    .map_err(|error| JsError::new(&error.to_string()))?;
    Ok(policy)
}

/// Verify a root, issuer manifest, and signed authorization context JSON value.
#[wasm_bindgen]
pub fn verify_authorization_context(
    root_json: &str,
    manifest_json: &str,
    context_json: &str,
    policy_json: &str,
) -> Result<String, JsError> {
    let policy = authorization_verification_policy(policy_json)?;
    let root_value: Value =
        serde_json::from_str(root_json).map_err(|error| JsError::new(&error.to_string()))?;
    let root = AuthorizationTrustRoot::parse(&root_value)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let manifest_value: Value =
        serde_json::from_str(manifest_json).map_err(|error| JsError::new(&error.to_string()))?;
    let manifest =
        parse_issuer_manifest(&manifest_value).map_err(|error| JsError::new(&error.to_string()))?;
    let verified_manifest = verify_issuer_manifest(&root, &manifest, &policy)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let context_value: Value =
        serde_json::from_str(context_json).map_err(|error| JsError::new(&error.to_string()))?;
    let context = parse_authorization_context(&context_value)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let verified =
        verify_authorization_context_protocol(&root, &verified_manifest, &context, &policy)
            .map_err(|error| JsError::new(&error.to_string()))?;
    let context_digest = verified.context_digest();
    let signed = verified.signed_context();
    serde_json::to_string(&json!({
        "authority": root.authority(),
        "rootKeyId": root.key_id(),
        "rootDigest": root.digest().map_err(|error| JsError::new(&error.to_string()))?,
        "manifestDigest": verified_manifest.digest().map_err(|error| JsError::new(&error.to_string()))?,
        "contextDigest": context_digest,
        "context": signed,
        "manifestGeneration": verified_manifest.generation(),
    }))
    .map_err(|error| JsError::new(&error.to_string()))
}

/// Verify a root-signed issuer manifest and return its verified projection.
#[wasm_bindgen]
pub fn verify_authorization_manifest(
    root_json: &str,
    manifest_json: &str,
    policy_json: &str,
) -> Result<String, JsError> {
    let policy = authorization_verification_policy(policy_json)?;
    let root_value: Value =
        serde_json::from_str(root_json).map_err(|error| JsError::new(&error.to_string()))?;
    let root = AuthorizationTrustRoot::parse(&root_value)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let manifest_value: Value =
        serde_json::from_str(manifest_json).map_err(|error| JsError::new(&error.to_string()))?;
    let manifest =
        parse_issuer_manifest(&manifest_value).map_err(|error| JsError::new(&error.to_string()))?;
    let verified = verify_issuer_manifest(&root, &manifest, &policy)
        .map_err(|error| JsError::new(&error.to_string()))?;
    serde_json::to_string(&json!({
        "authority": verified.authority(),
        "rootKeyId": verified.root_key_id(),
        "generation": verified.generation(),
        "digest": verified.digest().map_err(|error| JsError::new(&error.to_string()))?,
        "issuerKeyIds": verified
            .manifest()
            .unsigned
            .issuers
            .iter()
            .map(|entry| entry.key_id.clone())
            .collect::<Vec<_>>(),
    }))
    .map_err(|error| JsError::new(&error.to_string()))
}

/// Resolve one participant against its exact native API artifacts.
#[cfg(feature = "resolver")]
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

/// Verify a trust chain once and retain its verified context inside WASM.
#[wasm_bindgen]
pub fn create_authorization_context_handle(
    root_json: &str,
    manifest_json: &str,
    context_json: &str,
    policy_json: &str,
    historical: bool,
) -> Result<VerifiedAuthorizationContextHandle, JsError> {
    let policy = authorization_verification_policy(policy_json)?;
    let root_value: Value =
        serde_json::from_str(root_json).map_err(|error| JsError::new(&error.to_string()))?;
    let root = AuthorizationTrustRoot::parse(&root_value)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let manifest_value: Value =
        serde_json::from_str(manifest_json).map_err(|error| JsError::new(&error.to_string()))?;
    let manifest =
        parse_issuer_manifest(&manifest_value).map_err(|error| JsError::new(&error.to_string()))?;
    let context_value: Value =
        serde_json::from_str(context_json).map_err(|error| JsError::new(&error.to_string()))?;
    let signed_context = parse_authorization_context(&context_value)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let verification_policy = if historical {
        let mut policy = policy;
        policy.now_unix_seconds = signed_context.unsigned.expires_at;
        policy
    } else {
        policy
    };
    let verified_manifest = verify_issuer_manifest(&root, &manifest, &verification_policy)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let context = verify_authorization_context_protocol(
        &root,
        &verified_manifest,
        &signed_context,
        &verification_policy,
    )
    .map_err(|error| JsError::new(&error.to_string()))?;
    let projection = serde_json::to_string(&json!({
        "authority": root.authority(),
        "rootKeyId": root.key_id(),
        "rootDigest": root.digest().map_err(|error| JsError::new(&error.to_string()))?,
        "manifestDigest": verified_manifest.digest().map_err(|error| JsError::new(&error.to_string()))?,
        "contextDigest": context.context_digest(),
        "context": context.signed_context(),
        "manifestGeneration": verified_manifest.generation(),
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
        let now = i128::from(policy.now_unix_seconds);
        let skew = i128::from(policy.allowed_clock_skew_seconds);
        if now + skew < i128::from(self.context.not_before()) {
            return Err(JsError::new("authorization context is not yet valid"));
        }
        if now - skew > i128::from(self.context.expires_at()) {
            return Err(JsError::new("authorization context has expired"));
        }
        Ok(())
    }
}

fn verified_context_projection(
    context: &VerifiedAuthorizationContext,
) -> Result<Value, ProtocolError> {
    Ok(json!({
        "authority": context.authority(),
        "authorityRef": context.authority_ref(),
        "principal": context.principal(),
        "participant": context.participant(),
        "deploymentId": context.deployment_id(),
        "instanceId": context.instance_id(),
        "issuerKeyId": context.signed_context().unsigned.issuer_key_id,
        "issuerManifestGeneration": context.signed_context().unsigned.issuer_manifest_generation,
        "sessionId": context.session_id(),
        "sessionKey": context.signed_context().unsigned.session_key,
        "inboxPrefix": context.inbox_prefix(),
        "issuedAt": context.issued_at(),
        "notBefore": context.not_before(),
        "expiresAt": context.expires_at(),
        "grantSet": context.grant_set(),
        "grantDigest": context.grant_set().digest()?,
        "capabilities": context.capabilities(),
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
            required_capabilities: &input.required_capabilities,
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
        "participantDigest": publisher.participant_digest,
        "sessionId": publisher.session_id,
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
        required_capabilities: &input.required_capabilities,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn field<'a>(value: &'a Value, name: &str) -> &'a str {
        value[name]
            .as_str()
            .unwrap_or_else(|| panic!("missing vector field {name}"))
    }

    fn wire_input(fixture: &Value, case: &Value) -> Value {
        let value = case.get("request").unwrap_or(&case["input"]);
        let common = json!({
            "purpose": field(case, "purpose"),
            "requestId": field(value, "requestId"),
            "issuedAt": value["issuedAt"],
        });
        let mut input = common.as_object().expect("common input object").clone();
        let fields: &[&str] = match field(case, "purpose") {
            "userAuthRequest" => &[
                "sessionPublicKey",
                "sessionNkey",
                "participantId",
                "participantDigest",
                "redirectTarget",
            ],
            "serviceBootstrap" => &[
                "deploymentId",
                "instanceId",
                "provisionedIdentityKeyId",
                "newSessionPublicKey",
                "newSessionNkey",
                "participantId",
                "participantDigest",
            ],
            "deviceBootstrap" => &[
                "deploymentId",
                "instanceId",
                "deviceIdentityKeyId",
                "newSessionPublicKey",
                "newSessionNkey",
                "participantId",
                "participantDigest",
                "challengeDigest",
            ],
            "authorizationContextRefresh" => &[
                "sessionId",
                "sessionKeyId",
                "currentContextDigest",
                "expectedParticipantDigest",
                "expectedNeedsDigest",
                "knownRootKeyId",
                "minimumManifestGeneration",
            ],
            purpose => panic!("unknown vector purpose {purpose}"),
        };
        for name in fields {
            input.insert((*name).to_owned(), value[*name].clone());
        }
        if field(case, "purpose") == "authorizationContextRefresh" {
            input.insert("sessionKeyId".to_owned(), fixture["identityKeyId"].clone());
        }
        if !case["requestDigest"].is_null() {
            input.insert("requestDigest".to_owned(), case["requestDigest"].clone());
        }
        Value::Object(input)
    }

    #[test]
    fn shared_vectors_match_wasm_boundary() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../conformance/session-proof/vectors.json"
        ))
        .expect("parse vectors");
        for case in fixture["cases"].as_array().expect("vector cases") {
            let input = wire_input(&fixture, case);
            let input_json = serde_json::to_string(&input).expect("encode input");
            assert_eq!(
                session_proof_signing_digest(&input_json).expect("signing digest"),
                field(case, "transcriptDigest")
            );

            let source = case.get("request").unwrap_or(&case["input"]);
            if let Some(expected) = case["requestDigest"].as_str() {
                assert_eq!(
                    session_proof_request_digest(
                        &serde_json::to_string(source).expect("encode request")
                    )
                    .expect("request digest"),
                    expected
                );
            }
            verify_session_proof(
                &input_json,
                &serde_json::to_string(&source["proof"]).expect("encode proof"),
                field(case, "signerPublicKey"),
                source["issuedAt"].as_f64().expect("issuedAt"),
                30_000.0,
                30_000.0,
            )
            .expect("verify proof");
        }
    }

    fn authorization_fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../../../conformance/authorization-context/vectors.json"
        ))
        .expect("parse authorization vectors")
    }

    fn context_input(fixture: &Value, policy: Value) -> Value {
        let chain = &fixture["completeChain"];
        json!({
            "root": serde_json::from_str::<Value>(chain["rootCanonicalJson"].as_str().unwrap()).unwrap(),
            "manifest": serde_json::from_str::<Value>(chain["manifestCanonicalJson"].as_str().unwrap()).unwrap(),
            "context": serde_json::from_str::<Value>(chain["contextCanonicalJson"].as_str().unwrap()).unwrap(),
            "policy": policy,
        })
    }

    fn payload(value: &str) -> Value {
        Value::Array(value.bytes().map(Value::from).collect())
    }

    fn error_code_and_path(result: &str) -> (String, String) {
        let value: Value = serde_json::from_str(result).expect("parse verification result");
        assert_eq!(value["ok"], false);
        (
            value["error"]["code"].as_str().unwrap().to_owned(),
            value["error"]["path"].as_str().unwrap().to_owned(),
        )
    }

    fn verify_request_input(input_json: &str) -> String {
        let mut input: Value = serde_json::from_str(input_json).unwrap();
        let payload = serde_json::from_value::<Vec<u8>>(input["payload"].take()).unwrap();
        let handle = create_authorization_context_handle(
            &serde_json::to_string(&input["root"]).unwrap(),
            &serde_json::to_string(&input["manifest"]).unwrap(),
            &serde_json::to_string(&input["context"]).unwrap(),
            &serde_json::to_string(&input["policy"]).unwrap(),
            false,
        )
        .unwrap();
        for field in ["root", "manifest", "context", "payload"] {
            input.as_object_mut().unwrap().remove(field);
        }
        verify_authorization_request(&handle, &serde_json::to_string(&input).unwrap(), &payload)
    }

    fn verify_event_input(input_json: &str) -> String {
        let mut input: Value = serde_json::from_str(input_json).unwrap();
        let payload = serde_json::from_value::<Vec<u8>>(input["payload"].take()).unwrap();
        let handle = create_authorization_context_handle(
            &serde_json::to_string(&input["root"]).unwrap(),
            &serde_json::to_string(&input["manifest"]).unwrap(),
            &serde_json::to_string(&input["context"]).unwrap(),
            &serde_json::to_string(&input["policy"]).unwrap(),
            true,
        )
        .unwrap();
        for field in ["root", "manifest", "context", "payload"] {
            input.as_object_mut().unwrap().remove(field);
        }
        verify_authorization_event(&handle, &serde_json::to_string(&input).unwrap(), &payload)
    }

    #[test]
    fn local_authorization_uses_conformance_chain() {
        let fixture = authorization_fixture();
        let chain = &fixture["completeChain"];
        let defaults = &fixture["defaults"];
        let policy = defaults["policy"].clone();
        let permission = defaults["permission"].clone();
        let mut denied_permission = permission.clone();
        denied_permission["target"]["name"] = Value::from("Documents.Other");
        let required_capabilities = json!(["platform.read"]);

        let request = |subject: &str, reply: Option<&str>, body: &str, permission| {
            let mut input = context_input(&fixture, policy.clone());
            input["subject"] = Value::from(subject);
            input["reply"] = reply.map_or(Value::Null, Value::from);
            input["payload"] = payload(body);
            input["iat"] = defaults["request"]["iat"].clone();
            input["requestId"] = defaults["request"]["requestId"].clone();
            input["proof"] = chain["requestProof"].clone();
            input["requiredPermissions"] = json!([permission]);
            input["requiredCapabilities"] = required_capabilities.clone();
            input
        };

        let valid = verify_request_input(
            &serde_json::to_string(&request(
                defaults["request"]["subject"].as_str().unwrap(),
                defaults["request"]["reply"].as_str(),
                defaults["request"]["payload"].as_str().unwrap(),
                permission.clone(),
            ))
            .unwrap(),
        );
        let valid: Value = serde_json::from_str(&valid).unwrap();
        assert_eq!(valid["ok"], true, "{valid}");
        assert_eq!(valid["contextDigest"], chain["contextDigest"]);

        for (subject, reply, body) in [
            (
                "rpc.v1.Documents.Other",
                Some("_INBOX.test.reply"),
                "{\"id\":\"doc-1\"}",
            ),
            (
                "rpc.v1.Documents.Get",
                Some("_INBOX.test.changed"),
                "{\"id\":\"doc-1\"}",
            ),
            ("rpc.v1.Documents.Get", Some("_INBOX.test.reply"), "changed"),
        ] {
            let result = verify_request_input(
                &serde_json::to_string(&request(subject, reply, body, permission.clone())).unwrap(),
            );
            assert_eq!(
                error_code_and_path(&result),
                ("InvalidRequestProof".to_owned(), "/proof".to_owned())
            );
        }

        let outside_reply = verify_request_input(
            &serde_json::to_string(&request(
                "rpc.v1.Documents.Get",
                Some("OTHER.reply"),
                "{\"id\":\"doc-1\"}",
                permission.clone(),
            ))
            .unwrap(),
        );
        assert_eq!(
            error_code_and_path(&outside_reply),
            ("ReplySubjectMismatch".to_owned(), "/reply".to_owned())
        );

        let missing_permission = verify_request_input(
            &serde_json::to_string(&request(
                defaults["request"]["subject"].as_str().unwrap(),
                defaults["request"]["reply"].as_str(),
                defaults["request"]["payload"].as_str().unwrap(),
                denied_permission,
            ))
            .unwrap(),
        );
        assert_eq!(
            error_code_and_path(&missing_permission),
            (
                "PermissionDenied".to_owned(),
                "/grantSet/permissions".to_owned()
            )
        );

        let mut event_policy = defaults["policy"].clone();
        event_policy["nowUnixSeconds"] = Value::from(1_400);
        let mut event = context_input(&fixture, event_policy);
        event["subject"] = defaults["event"]["subject"].clone();
        event["payload"] = payload(defaults["event"]["payload"].as_str().unwrap());
        event["eventId"] = defaults["event"]["eventId"].clone();
        event["eventTime"] = defaults["event"]["eventTime"].clone();
        event["proof"] = chain["eventProof"].clone();
        event["requiredPermissions"] = json!([permission]);
        event["requiredCapabilities"] = json!([]);

        let valid_event = verify_event_input(&serde_json::to_string(&event).unwrap());
        let valid_event: Value = serde_json::from_str(&valid_event).unwrap();
        assert_eq!(valid_event["ok"], true);
        assert_eq!(valid_event["publisher"]["sessionId"], "ses_test");

        let mut revoked = event.clone();
        revoked["revokedAt"] = Value::from(1_150);
        let revoked = verify_event_input(&serde_json::to_string(&revoked).unwrap());
        assert_eq!(
            error_code_and_path(&revoked),
            ("EventRevoked".to_owned(), "/event-time".to_owned())
        );

        let mut historical_window = event;
        historical_window["eventTime"] = Value::from("1970-01-01T00:16:40Z");
        let historical_window =
            verify_event_input(&serde_json::to_string(&historical_window).unwrap());
        assert_eq!(
            error_code_and_path(&historical_window),
            ("ContextNotYetValid".to_owned(), "/event-time".to_owned())
        );
    }
}

#[cfg(test)]
mod phase_a_tests {
    use super::*;

    #[test]
    fn direct_context_and_manifest_wasm_boundaries_verify() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../conformance/authorization-context/vectors.json"
        ))
        .unwrap();
        let chain = &fixture["completeChain"];
        let policy = fixture["defaults"]["policy"].clone();
        let context = verify_authorization_context(
            chain["rootCanonicalJson"].as_str().unwrap(),
            chain["manifestCanonicalJson"].as_str().unwrap(),
            chain["contextCanonicalJson"].as_str().unwrap(),
            &serde_json::to_string(&policy).unwrap(),
        )
        .unwrap();
        let context: Value = serde_json::from_str(&context).unwrap();
        assert_eq!(context["contextDigest"], chain["contextDigest"]);
        assert_eq!(context["context"]["issuerManifestGeneration"], 7);

        let manifest = verify_authorization_manifest(
            chain["rootCanonicalJson"].as_str().unwrap(),
            chain["manifestCanonicalJson"].as_str().unwrap(),
            &serde_json::to_string(&policy).unwrap(),
        )
        .unwrap();
        let manifest: Value = serde_json::from_str(&manifest).unwrap();
        assert_eq!(manifest["digest"], chain["manifestDigest"]);
        assert_eq!(manifest["issuerKeyIds"][0], chain["issuerKeyId"]);
    }
}
