use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use ed25519_dalek::{Signature, Verifier as _, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::client::TrellisClientError;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[doc = concat!("Trellis API operation `", stringify!(sha256), "`.")]
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.into()
}

#[doc = concat!("Trellis API operation `", stringify!(base64url_encode), "`.")]
pub fn base64url_encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

#[doc = concat!("Trellis API operation `", stringify!(base64url_decode), "`.")]
pub fn base64url_decode(value: &str) -> Result<Vec<u8>, base64::DecodeError> {
    URL_SAFE_NO_PAD.decode(value)
}

#[doc = concat!("Trellis API operation `", stringify!(now_iat_seconds), "`.")]
pub fn now_iat_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[doc = concat!("Trellis API operation `", stringify!(new_request_id), "`.")]
pub fn new_request_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("req_{nanos:x}_{sequence:x}")
}

/// Verify a context-bound v2 event proof against the raw published values.
#[doc = concat!("Trellis API operation `", stringify!(verify_event_proof_v2), "`.")]
pub fn verify_event_proof_v2(
    public_session_key: &str,
    context_digest: &str,
    subject: &str,
    payload: &[u8],
    event_id: &str,
    event_time: &str,
    proof_base64url: &str,
) -> Result<bool, TrellisClientError> {
    let context_digest = decode_context_digest(context_digest)?;
    let input = trellis_protocol::build_authorization_event_proof_input_v2(
        &context_digest,
        subject,
        payload,
        event_id,
        event_time,
    )
    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    Ok(verify_v2_signature(
        public_session_key,
        input.digest(),
        proof_base64url,
    ))
}

fn decode_context_digest(context_digest: &str) -> Result<[u8; 32], TrellisClientError> {
    let decoded = base64url_decode(context_digest)?;
    decoded.try_into().map_err(|_| {
        TrellisClientError::Bootstrap("authorization context digest must encode 32 bytes".into())
    })
}

fn verify_v2_signature(public_session_key: &str, digest: &[u8; 32], proof_base64url: &str) -> bool {
    let Ok(public_key_bytes) = base64url_decode(public_session_key) else {
        return false;
    };
    let Ok(public_key) = <[u8; 32]>::try_from(public_key_bytes.as_slice()) else {
        return false;
    };
    let Ok(signature_bytes) = base64url_decode(proof_base64url) else {
        return false;
    };
    let Ok(signature) = <[u8; 64]>::try_from(signature_bytes.as_slice()) else {
        return false;
    };
    let Ok(public_key) = VerifyingKey::from_bytes(&public_key) else {
        return false;
    };
    public_key
        .verify(digest, &Signature::from_bytes(&signature))
        .is_ok()
}
