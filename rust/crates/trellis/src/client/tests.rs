use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::client::proof::{base64url_encode, build_event_proof_input, build_proof_input, sha256};
use crate::client::{verify_event_proof, verify_proof, SessionAuth};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthProofFixture {
    name: String,
    seed: String,
    session_key: String,
    oauth_init: DomainSigFixture,
    flow_bind: DomainSigFixture,
    rpc_proof: RpcProofFixture,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DomainSigFixture {
    redirect_to: Option<String>,
    flow_id: Option<String>,
    sig: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RpcProofFixture {
    subject: String,
    payload: String,
    iat: i64,
    request_id: String,
    payload_hash_base64url: String,
    proof_input_hex: String,
    proof_digest_base64url: String,
    proof: String,
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut out, "{:02x}", byte).unwrap();
    }
    out
}

#[test]
fn auth_proof_matches_shared_conformance_vectors() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../conformance/auth-proof/vectors.json");
    let fixtures: Vec<AuthProofFixture> =
        serde_json::from_str(&fs::read_to_string(fixture_path).unwrap()).unwrap();

    assert!(fixtures.len() >= 2);

    for fixture in fixtures {
        assert!(fixture.name.starts_with("proof-layout-current"));
        let auth = SessionAuth::from_seed_base64url(&fixture.seed).unwrap();
        assert_eq!(auth.session_key, fixture.session_key);

        assert_eq!(
            auth.sign_sha256_domain(
                "oauth-init",
                &format!(
                    "{}:null",
                    fixture.oauth_init.redirect_to.as_deref().unwrap()
                )
            ),
            fixture.oauth_init.sig
        );
        assert_eq!(
            auth.sign_sha256_domain("bind-flow", fixture.flow_bind.flow_id.as_deref().unwrap()),
            fixture.flow_bind.sig
        );
        let payload_hash = sha256(fixture.rpc_proof.payload.as_bytes());
        assert_eq!(
            base64url_encode(&payload_hash),
            fixture.rpc_proof.payload_hash_base64url
        );

        let proof_input = build_proof_input(
            &fixture.session_key,
            &fixture.rpc_proof.subject,
            &payload_hash,
            fixture.rpc_proof.iat,
            &fixture.rpc_proof.request_id,
        );
        assert_eq!(
            bytes_to_hex(&proof_input),
            fixture.rpc_proof.proof_input_hex
        );

        let proof_digest = sha256(&proof_input);
        assert_eq!(
            base64url_encode(&proof_digest),
            fixture.rpc_proof.proof_digest_base64url
        );

        assert_eq!(
            auth.create_proof(
                &fixture.rpc_proof.subject,
                fixture.rpc_proof.payload.as_bytes(),
                fixture.rpc_proof.iat,
                &fixture.rpc_proof.request_id,
            ),
            fixture.rpc_proof.proof
        );

        assert!(verify_proof(
            &fixture.session_key,
            &fixture.rpc_proof.subject,
            fixture.rpc_proof.payload.as_bytes(),
            fixture.rpc_proof.iat,
            &fixture.rpc_proof.request_id,
            &fixture.rpc_proof.proof,
        )
        .unwrap());
    }
}

#[test]
fn event_proof_matches_typescript_byte_encoding() {
    let auth = SessionAuth::from_seed_base64url("AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8")
        .expect("session auth");
    let subject = "events.v1.Thing.Changed.one";
    let payload = br#"{"value":"one"}"#;
    let event_id = "evt_123";
    let event_time = "2026-04-26T00:00:00.000Z";
    let payload_hash = sha256(payload);

    assert_eq!(
        auth.session_key,
        "A6EHv_POEL4dcN0Y50vAmWfk1jCbpQ1fHdyGZBJVMbg"
    );
    assert_eq!(
        base64url_encode(&payload_hash),
        "8MGs1k0aqs-bFg6Kt_g8bnO40DhjibIusT1A6ABYYic"
    );

    let proof_input = build_event_proof_input(
        &auth.session_key,
        subject,
        &payload_hash,
        event_id,
        event_time,
    );
    assert_eq!(
        base64url_encode(&proof_input),
        "AAAAFnRyZWxsaXMtZXZlbnQtcHJvb2YtdjEAAAArQTZFSHZfUE9FTDRkY04wWTUwdkFtV2ZrMWpDYnBRMWZIZHlHWkJKVk1iZwAAABtldmVudHMudjEuVGhpbmcuQ2hhbmdlZC5vbmUAAAAg8MGs1k0aqs-bFg6Kt_g8bnO40DhjibIusT1A6ABYYicAAAAHZXZ0XzEyMwAAABgyMDI2LTA0LTI2VDAwOjAwOjAwLjAwMFo"
    );
    assert_eq!(
        base64url_encode(&sha256(&proof_input)),
        "VwB4GO64q3oXEFe1BIs7t0vV8ytzkJ64bLCieL5q_LY"
    );

    let proof = auth.create_event_proof(subject, payload, event_id, event_time);
    assert_eq!(
        proof,
        "9YyUCXtu8zoxXhF4Bs5nxJvMw_vusLXlR2jOfjdTtmIbm1TOWbI4aqsN_yCXR7UlXw0iDyTj1unjv0RXle1gCw"
    );
    assert!(verify_event_proof(
        &auth.session_key,
        subject,
        payload,
        event_id,
        event_time,
        &proof,
    )
    .expect("event proof verifies"));
    assert!(!verify_event_proof(
        &auth.session_key,
        subject,
        payload,
        "evt_other",
        event_time,
        &proof,
    )
    .expect("changed event id rejects"));
}
