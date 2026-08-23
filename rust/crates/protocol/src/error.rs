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
    /// A required event consumer depends on an optional API use.
    RequiredConsumerUsesOptionalApi,
    /// A schema pointer cannot be proven to resolve.
    UnresolvableSchemaPointer,
    /// A resolved schema pointer cannot produce the required value type.
    SchemaPointerTypeMismatch,
}

/// Stable failure categories for signed authorization protocol values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationErrorCodeV1 {
    /// A value has the wrong protocol format or strict object shape.
    InvalidFormat,
    /// An integer cannot be represented exactly by interoperable JSON implementations.
    UnsafeJsonInteger,
    /// A binary value is not canonical unpadded base64url.
    InvalidEncoding,
    /// An Ed25519 public key is malformed.
    InvalidPublicKey,
    /// A declared key id does not match its public key.
    InvalidKeyId,
    /// A signature is malformed or cryptographically invalid.
    InvalidSignature,
    /// An object belongs to another authority.
    WrongAuthority,
    /// A critical extension is not understood.
    UnknownCriticalExtension,
    /// A set-like array is duplicated or out of canonical order.
    NonCanonicalSet,
    /// An authored validity interval is inconsistent.
    InvalidValidityWindow,
    /// A manifest generation is older than the accepted minimum.
    ManifestRollback,
    /// The context was issued against a different manifest generation.
    ManifestGenerationMismatch,
    /// The issuer manifest is not yet valid.
    ManifestNotYetValid,
    /// The issuer manifest has expired.
    ManifestExpired,
    /// The context issuer is absent from the manifest.
    IssuerNotListed,
    /// The authorization context is not yet valid.
    ContextNotYetValid,
    /// The authorization context has expired.
    ContextExpired,
    /// The context lifetime exceeds explicit policy.
    ContextLifetimeExceeded,
    /// The context validity exceeds its manifest.
    ContextOutlivesManifest,
    /// The session verification key is malformed.
    InvalidSessionKey,
    /// One or more exact permission atoms are absent.
    PermissionDenied,
    /// One or more platform capability keys are absent.
    CapabilityDenied,
    /// The canonical signed context exceeds explicit policy.
    ContextTooLarge,
    /// The request issue time is outside the accepted skew.
    ProofIatOutOfRange,
    /// The context-bound request signature is invalid.
    InvalidRequestProof,
    /// The request reply subject is outside the caller inbox prefix.
    ReplySubjectMismatch,
    /// The event time is not canonical RFC 3339 UTC.
    InvalidEventTime,
    /// The context-bound event signature is invalid.
    InvalidEventProof,
    /// The event time is at or after the context revocation time.
    EventRevoked,
}

/// Stable failure categories for bootstrap and authorization-context refresh proofs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionProofErrorCodeV1 {
    /// A value has the wrong protocol format or strict object shape.
    InvalidFormat,
    /// An integer cannot be represented exactly by interoperable JSON implementations.
    UnsafeJsonInteger,
    /// A binary value is not canonical unpadded base64url.
    InvalidEncoding,
    /// An Ed25519 public key is malformed.
    InvalidPublicKey,
    /// A declared key id does not match its public key.
    InvalidKeyId,
    /// A NATS User NKey is malformed or does not encode the session public key.
    InvalidNatsKey,
    /// A signature is malformed or cryptographically invalid.
    InvalidSignature,
    /// The proof issue time is outside the accepted policy window.
    ProofIatOutOfRange,
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
        path: Box<PointerBuf>,
        /// Specific validation failure.
        message: String,
    },

    /// A signed authorization object or proof failed validation.
    #[error("authorization validation failed at '{path}' ({code:?}): {message}")]
    Authorization {
        /// Stable failure category.
        code: AuthorizationErrorCodeV1,
        /// Exact authored RFC 6901 path.
        path: Box<PointerBuf>,
        /// Safe diagnostic that omits secrets and signed payloads.
        message: String,
    },

    /// A bootstrap or authorization-context refresh proof failed validation.
    #[error("session proof validation failed at '{path}' ({code:?}): {message}")]
    SessionProof {
        /// Stable failure category.
        code: SessionProofErrorCodeV1,
        /// Exact authored RFC 6901 path.
        path: Box<PointerBuf>,
        /// Safe diagnostic that omits secrets and signed payloads.
        message: String,
    },
}
