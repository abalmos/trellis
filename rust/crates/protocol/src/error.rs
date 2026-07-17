use jsonptr::PointerBuf;

/// A contextual participant-resolution failure category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionErrorCodeV1 {
    /// A referenced API was not supplied.
    MissingApi,
    /// A supplied API did not match the pinned digest.
    ApiDigestMismatch,
    /// A selected API surface does not exist.
    MissingSurface,
    /// Cancellation was selected for a non-cancelable operation.
    InvalidCancelSelection,
    /// A selected operation signal does not exist.
    MissingOperationSignal,
    /// An implemented operation transfer mapping is invalid.
    InvalidImplementedTransfer,
    /// An implemented send-transfer operation lacks a mapping.
    MissingRequiredTransfer,
    /// A required transfer uses an optional store.
    OptionalStoreForRequiredTransfer,
    /// A schema pointer cannot be proven to resolve.
    UnresolvableSchemaPointer,
    /// A resolved schema pointer cannot produce the required value type.
    SchemaPointerTypeMismatch,
}

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

    /// Contextual participant resolution against exact API artifacts failed.
    #[error("participant '{participant}' resolution failed at '{path}' ({code:?}): {message}")]
    ParticipantResolution {
        /// Stable failure category.
        code: ResolutionErrorCodeV1,
        /// Participant being resolved.
        participant: String,
        /// Participant-local API alias, when applicable.
        alias: Option<String>,
        /// Canonical API identifier, when known.
        api: Option<String>,
        /// Exact authored RFC 6901 path.
        path: PointerBuf,
        /// Specific validation failure.
        message: String,
    },
}
