use serde_json::Value;

use crate::ContractsError;

/// Render JSON into the canonical Trellis form used for digests.
pub fn canonicalize_json(value: &Value) -> Result<String, ContractsError> {
    trellis_protocol::canonicalize_json(value).map_err(|error| match error {
        trellis_protocol::ProtocolError::NonCanonicalNumber(value) => {
            ContractsError::NonCanonicalNumber(value)
        }
        _ => unreachable!("canonical JSON has no other failure mode"),
    })
}

/// Compute a base64url-encoded SHA-256 digest for text.
pub fn sha256_base64url(text: &str) -> String {
    trellis_protocol::sha256_base64url(text)
}

/// Canonicalize JSON and return its Trellis digest.
pub fn digest_json(value: &Value) -> Result<String, ContractsError> {
    canonicalize_json(value).map(|canonical| trellis_protocol::sha256_base64url(&canonical))
}
