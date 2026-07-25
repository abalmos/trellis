//! WASM boundary for deterministic Trellis protocol proof operations.

#![deny(missing_docs)]

use serde::Deserialize;
use serde_json::{json, Value};
use trellis_protocol::{
    authorization_context_refresh_at_v1, parse_authorization_context_token_v1,
    parse_issuer_certificate_v1, parse_issuer_manifest_v1, session_proof_request_digest_v1,
    session_proof_signing_digest_v1, verify_authorization_context_v1, verify_issuer_manifest_v1,
    verify_session_proof_v1, AuthorizationTrustRootV1, AuthorizationVerificationPolicyV1,
    ProtocolError, SessionProofInputV1, SessionProofPolicyV1, SessionProofV1,
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
enum WireSessionProofInputV1 {
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
    ClientBootstrap {
        request_id: String,
        issued_at: i64,
        session_id: String,
        session_key_id: String,
        session_public_key: String,
        session_nkey: String,
        expected_participant_digest: RequiredNullable<String>,
        expected_needs_digest: RequiredNullable<String>,
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
    NatsConnect {
        request_id: String,
        issued_at: i64,
        session_id: String,
        session_key_id: String,
        session_public_key: String,
        session_nkey: String,
        participant_digest: String,
        nonce: String,
    },
    SessionSelfControl {
        request_id: String,
        issued_at: i64,
        session_id: String,
        session_key_id: String,
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
    NatsConnectContext {
        request_id: String,
        issued_at: i64,
        session_id: String,
        session_key_id: String,
        session_public_key: String,
        session_nkey: String,
        participant_digest: String,
        context_digest: String,
        nonce: String,
    },
}

impl TryFrom<WireSessionProofInputV1> for SessionProofInputV1 {
    type Error = ProtocolError;

    fn try_from(value: WireSessionProofInputV1) -> Result<Self, Self::Error> {
        match value {
            WireSessionProofInputV1::UserAuthRequest {
                request_id,
                issued_at,
                session_public_key,
                session_nkey,
                participant_id,
                participant_digest,
                redirect_target,
                request_digest,
            } => Self::user_auth_request(
                request_id,
                issued_at,
                session_public_key,
                session_nkey,
                participant_id,
                participant_digest,
                redirect_target,
                request_digest,
            ),
            WireSessionProofInputV1::ClientBootstrap {
                request_id,
                issued_at,
                session_id,
                session_key_id,
                session_public_key,
                session_nkey,
                expected_participant_digest,
                expected_needs_digest,
                request_digest,
            } => Self::client_bootstrap(
                request_id,
                issued_at,
                session_id,
                session_key_id,
                session_public_key,
                session_nkey,
                expected_participant_digest.0,
                expected_needs_digest.0,
                request_digest,
            ),
            WireSessionProofInputV1::ServiceBootstrap {
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
            } => Self::service_bootstrap(
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
            ),
            WireSessionProofInputV1::DeviceBootstrap {
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
            } => Self::device_bootstrap(
                request_id,
                issued_at,
                deployment_id,
                instance_id,
                device_identity_key_id,
                new_session_public_key,
                new_session_nkey,
                participant_id,
                participant_digest,
                challenge_digest.0,
                request_digest,
            ),
            WireSessionProofInputV1::NatsConnect {
                request_id,
                issued_at,
                session_id,
                session_key_id,
                session_public_key,
                session_nkey,
                participant_digest,
                nonce,
            } => Self::nats_connect(
                request_id,
                issued_at,
                session_id,
                session_key_id,
                session_public_key,
                session_nkey,
                participant_digest,
                nonce,
            ),
            WireSessionProofInputV1::SessionSelfControl {
                request_id,
                issued_at,
                session_id,
                session_key_id,
                request_digest,
            } => Self::session_self_control(
                request_id,
                issued_at,
                session_id,
                session_key_id,
                request_digest,
            ),
            WireSessionProofInputV1::AuthorizationContextRefresh {
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
            } => Self::authorization_context_refresh(
                request_id,
                issued_at,
                session_id,
                session_key_id,
                current_context_digest.0,
                expected_participant_digest.0,
                expected_needs_digest.0,
                known_root_key_id,
                minimum_manifest_generation,
                request_digest,
            ),
            WireSessionProofInputV1::NatsConnectContext {
                request_id,
                issued_at,
                session_id,
                session_key_id,
                session_public_key,
                session_nkey,
                participant_digest,
                context_digest,
                nonce,
            } => Self::nats_connect_context(
                request_id,
                issued_at,
                session_id,
                session_key_id,
                session_public_key,
                session_nkey,
                participant_digest,
                context_digest,
                nonce,
            ),
        }
    }
}

fn parse_input(input_json: &str) -> Result<SessionProofInputV1, JsError> {
    serde_json::from_str::<WireSessionProofInputV1>(input_json)
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
    session_proof_request_digest_v1(&request).map_err(|error| JsError::new(&error.to_string()))
}

/// Return the canonical signing digest for a JSON-encoded purpose-specific input.
#[wasm_bindgen]
pub fn session_proof_signing_digest(input_json: &str) -> Result<String, JsError> {
    session_proof_signing_digest_v1(&parse_input(input_json)?)
        .map_err(|error| JsError::new(&error.to_string()))
}

/// Verify a JSON-encoded session proof and return its replay identity as JSON.
#[wasm_bindgen]
pub fn verify_session_proof(
    input_json: &str,
    proof_json: &str,
    signer_public_key: &str,
    now_ms: f64,
    maximum_age_ms: f64,
    maximum_future_skew_ms: f64,
) -> Result<String, JsError> {
    let proof: SessionProofV1 =
        serde_json::from_str(proof_json).map_err(|error| JsError::new(&error.to_string()))?;
    let now_ms = safe_integer(now_ms, "nowMs")?;
    let policy = SessionProofPolicyV1::new(
        safe_integer(maximum_age_ms, "maximumAgeMs")?,
        safe_integer(maximum_future_skew_ms, "maximumFutureSkewMs")?,
    )
    .map_err(|error| JsError::new(&error.to_string()))?;
    let verified = verify_session_proof_v1(
        &parse_input(input_json)?,
        &proof,
        signer_public_key,
        now_ms,
        policy,
    )
    .map_err(|error| JsError::new(&error.to_string()))?;
    let replay = verified.replay_key();
    serde_json::to_string(&json!({
        "purpose": replay.purpose().to_string(),
        "signerKeyId": replay.signer_key_id(),
        "requestId": replay.request_id(),
        "transcriptDigest": replay.transcript_digest(),
    }))
    .map_err(|error| JsError::new(&error.to_string()))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WireAuthorizationVerificationPolicyV1 {
    now_unix_seconds: f64,
    allowed_clock_skew_seconds: u32,
    maximum_context_lifetime_seconds: u32,
    maximum_context_bytes: usize,
    maximum_permissions: usize,
    maximum_capabilities: usize,
    minimum_manifest_generation: f64,
    refresh_lead_seconds: u32,
    refresh_jitter_seconds: u32,
}

fn authorization_verification_policy(
    policy_json: &str,
) -> Result<
    (
        WireAuthorizationVerificationPolicyV1,
        AuthorizationVerificationPolicyV1,
    ),
    JsError,
> {
    let wire: WireAuthorizationVerificationPolicyV1 =
        serde_json::from_str(policy_json).map_err(|error| JsError::new(&error.to_string()))?;
    let policy = AuthorizationVerificationPolicyV1::new(
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
    Ok((wire, policy))
}

/// Verify a complete authorization token and trust chain, returning its compact projection.
#[wasm_bindgen]
pub fn verify_authorization_context_token(
    root_json: &str,
    manifest_json: &str,
    certificate_json: &str,
    context_token: &str,
    policy_json: &str,
) -> Result<String, JsError> {
    let (wire_policy, policy) = authorization_verification_policy(policy_json)?;
    let root_value: Value =
        serde_json::from_str(root_json).map_err(|error| JsError::new(&error.to_string()))?;
    let root = AuthorizationTrustRootV1::parse(&root_value)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let manifest_value: Value =
        serde_json::from_str(manifest_json).map_err(|error| JsError::new(&error.to_string()))?;
    let manifest = parse_issuer_manifest_v1(&manifest_value)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let certificate_value: Value =
        serde_json::from_str(certificate_json).map_err(|error| JsError::new(&error.to_string()))?;
    let certificate = parse_issuer_certificate_v1(&certificate_value)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let verified_manifest = verify_issuer_manifest_v1(&root, &manifest, &policy)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let context = parse_authorization_context_token_v1(context_token, &policy)
        .map_err(|error| JsError::new(&error.to_string()))?;
    let verified =
        verify_authorization_context_v1(&root, &verified_manifest, &certificate, &context, &policy)
            .map_err(|error| JsError::new(&error.to_string()))?;
    let context_digest = verified.context_digest();
    let signed = verified.signed_context();
    let refresh_at = authorization_context_refresh_at_v1(
        context_digest,
        signed.unsigned.issued_at,
        signed.unsigned.not_before,
        signed.unsigned.expires_at,
        wire_policy.refresh_lead_seconds,
        wire_policy.refresh_jitter_seconds,
    )
    .map_err(|error| JsError::new(&error.to_string()))?;
    serde_json::to_string(&json!({
        "authority": root.authority(),
        "rootKeyId": root.key_id(),
        "rootDigest": root.digest().map_err(|error| JsError::new(&error.to_string()))?,
        "manifestDigest": verified_manifest.digest().map_err(|error| JsError::new(&error.to_string()))?,
        "contextDigest": context_digest,
        "context": signed,
        "manifestGeneration": verified_manifest.generation(),
        "refreshAt": refresh_at,
    }))
    .map_err(|error| JsError::new(&error.to_string()))
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
            "clientBootstrap" => &[
                "sessionId",
                "sessionKeyId",
                "sessionNkey",
                "expectedParticipantDigest",
                "expectedNeedsDigest",
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
            "natsConnect" => &[
                "sessionId",
                "sessionKeyId",
                "sessionNkey",
                "participantDigest",
                "nonce",
            ],
            "sessionSelfControl" => &["sessionId", "sessionKeyId"],
            "authorizationContextRefresh" => &[
                "sessionId",
                "sessionKeyId",
                "currentContextDigest",
                "expectedParticipantDigest",
                "expectedNeedsDigest",
                "knownRootKeyId",
                "minimumManifestGeneration",
            ],
            "natsConnectContext" => &[
                "sessionId",
                "sessionKeyId",
                "sessionNkey",
                "participantDigest",
                "contextDigest",
                "nonce",
            ],
            purpose => panic!("unknown vector purpose {purpose}"),
        };
        for name in fields {
            input.insert((*name).to_owned(), value[*name].clone());
        }
        if matches!(
            field(case, "purpose"),
            "clientBootstrap" | "natsConnect" | "natsConnectContext"
        ) {
            input.insert(
                "sessionPublicKey".to_owned(),
                fixture["identityPublicKey"].clone(),
            );
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
            let replay: Value = serde_json::from_str(
                &verify_session_proof(
                    &input_json,
                    &serde_json::to_string(&source["proof"]).expect("encode proof"),
                    field(case, "signerPublicKey"),
                    source["issuedAt"].as_f64().expect("issuedAt"),
                    30_000.0,
                    30_000.0,
                )
                .expect("verify proof"),
            )
            .expect("parse replay");
            assert_eq!(replay["transcriptDigest"], case["transcriptDigest"]);
        }
    }
}
