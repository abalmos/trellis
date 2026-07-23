use std::path::PathBuf;

/// Errors returned while loading, validating, or packing Trellis contracts.
#[derive(thiserror::Error, Debug)]
pub enum ContractsError {
    /// A protocol-owned API or participant artifact is invalid.
    #[error("protocol artifact error: {0}")]
    Protocol(Box<trellis_protocol::ProtocolError>),

    /// Contract I/O failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Contract JSON encoding or decoding failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// A JSON number cannot be represented canonically.
    #[error("non-canonical number in manifest: {0}")]
    NonCanonicalNumber(String),

    /// A contract schema could not be compiled.
    #[error("failed to compile {kind} schema: {message}")]
    SchemaCompile {
        /// The schema category.
        kind: &'static str,
        /// The compiler diagnostic.
        message: String,
    },

    /// Contract data failed schema validation.
    #[error("invalid {kind}:\n{details}")]
    SchemaValidation {
        /// The validated data category.
        kind: &'static str,
        /// The validation diagnostics.
        details: String,
    },

    /// Two manifests define the same schema identifier differently.
    #[error("schema $id '{schema_id}' differs across manifests (found in {path})")]
    DuplicateSchemaId {
        /// The duplicated schema identifier.
        schema_id: String,
        /// The path containing the conflicting schema.
        path: PathBuf,
    },

    /// Two manifests use one contract ID with different digests.
    #[error("contract id '{id}' appears multiple times with different digests ({existing_digest} vs {new_digest})")]
    DuplicateContractId {
        /// The duplicated contract ID.
        id: String,
        /// The previously loaded digest.
        existing_digest: String,
        /// The conflicting digest.
        new_digest: String,
    },

    /// Two contracts own the same transport subject.
    #[error("subject '{subject}' is declared by both '{first_contract}' and '{second_contract}'")]
    SubjectCollision {
        /// The duplicated subject.
        subject: String,
        /// The first owning contract.
        first_contract: String,
        /// The second owning contract.
        second_contract: String,
    },

    /// One dependency alias references different contracts.
    #[error("contract use '{alias}' references both '{existing_contract}' and '{new_contract}'")]
    ContractUseConflict {
        /// The conflicting alias.
        alias: String,
        /// The previously referenced contract.
        existing_contract: String,
        /// The conflicting referenced contract.
        new_contract: String,
    },

    /// A surface references a capability absent from the contract declaration.
    #[error("{context} references undeclared local capability '{capability}'")]
    UndeclaredCapability {
        /// The surface containing the reference.
        context: String,
        /// The missing capability.
        capability: String,
    },

    /// A local capability incorrectly includes the contract namespace.
    #[error(
        "local capability '{capability}' must not start with contract namespace prefix '{prefix}'"
    )]
    InvalidLocalCapability {
        /// The invalid capability.
        capability: String,
        /// The reserved contract prefix.
        prefix: String,
    },
}

impl From<trellis_protocol::ProtocolError> for ContractsError {
    fn from(error: trellis_protocol::ProtocolError) -> Self {
        Self::Protocol(Box::new(error))
    }
}
