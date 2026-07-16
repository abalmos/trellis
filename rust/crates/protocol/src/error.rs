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

    /// A `trellis.api.v1` member failed structural or semantic validation.
    #[error("invalid API artifact at '{path}': {message}")]
    ApiValidation {
        /// JSON Pointer-like path to the invalid member.
        path: String,
        /// Specific validation failure.
        message: String,
    },

    /// A `trellis.participant.v1` member failed structural or semantic validation.
    #[error("invalid participant artifact at '{path}': {message}")]
    ParticipantValidation {
        /// JSON Pointer-like path to the invalid member.
        path: String,
        /// Specific validation failure.
        message: String,
    },

    /// An embedded protocol schema violates the Trellis Draft 2020-12 profile.
    #[error("invalid schema '{schema}' at '{path}': {message}")]
    SchemaProfile {
        /// Name of the embedded schema.
        schema: String,
        /// Path within the embedded schema.
        path: String,
        /// Specific profile or JSON Schema failure.
        message: String,
    },
}
