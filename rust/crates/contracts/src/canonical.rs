use serde_json::Value;

use crate::ContractsError;

/// Render JSON into the canonical Trellis form used for digests.
pub fn canonicalize_json(value: &Value) -> Result<String, ContractsError> {
    trellis_protocol::canonicalize_json(value).map_err(|error| match error {
        trellis_protocol::ProtocolError::Json(error) => ContractsError::Json(error),
        trellis_protocol::ProtocolError::NonCanonicalNumber(value) => {
            ContractsError::NonCanonicalNumber(value)
        }
        trellis_protocol::ProtocolError::InvalidIdentifier { .. } => {
            unreachable!("identifier validation is not part of canonical JSON rendering")
        }
        trellis_protocol::ProtocolError::InvalidPermission { .. } => {
            unreachable!("permission validation is not part of canonical JSON rendering")
        }
        trellis_protocol::ProtocolError::InvalidGrantSetFormat(_) => {
            unreachable!("grant-set validation is not part of canonical JSON rendering")
        }
        trellis_protocol::ProtocolError::ApiValidation { .. } => {
            unreachable!("API validation is not part of canonical JSON rendering")
        }
        trellis_protocol::ProtocolError::SchemaProfile { .. } => {
            unreachable!("schema-profile validation is not part of canonical JSON rendering")
        }
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
