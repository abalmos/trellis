/// Errors produced while validating or canonicalizing Trellis protocol values.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    /// JSON encoding or decoding failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// A JSON number cannot be represented canonically.
    #[error("non-canonical JSON number: {0}")]
    NonCanonicalNumber(String),

    /// A protocol identifier is empty or contains forbidden characters.
    #[error("invalid {field}: {reason}")]
    InvalidIdentifier {
        /// The semantic field that failed validation.
        field: &'static str,
        /// The validation failure.
        reason: &'static str,
    },

    /// An action is not valid for its target kind.
    #[error("action '{action}' is not valid for {target}")]
    InvalidPermission {
        /// The invalid wire action.
        action: String,
        /// The target kind receiving the action.
        target: &'static str,
    },

    /// A grant set uses an unsupported wire format.
    #[error("unsupported grant-set format '{0}'")]
    InvalidGrantSetFormat(String),
}
