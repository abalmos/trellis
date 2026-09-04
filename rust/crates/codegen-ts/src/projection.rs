use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A named serializable error definition declared by a contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ErrorDefinition {
    #[serde(skip)]
    #[doc = concat!("The `", stringify!(error_type), "` contract value.")]
    pub(crate) error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(schema), "` contract value.")]
    pub(crate) schema: Option<SchemaReference>,
}

/// A reference to one named top-level contract schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SchemaReference {
    #[doc = concat!("The `", stringify!(schema), "` contract value.")]
    pub(crate) schema: String,
}

/// A reference to a named contract error declaration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub(crate) struct ErrorReference {
    #[doc = concat!("The `", stringify!(error_type), "` contract value.")]
    pub(crate) error_type: String,
}

/// Programmer-facing Markdown documentation attached to a contract surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Documentation {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(summary), "` contract value.")]
    pub(crate) summary: Option<String>,
    #[doc = concat!("The `", stringify!(markdown), "` contract value.")]
    pub(crate) markdown: String,
}

/// Capability requirements for invoking an RPC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct RpcCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(call), "` contract value.")]
    pub(crate) call: Option<Vec<String>>,
}

/// Capability requirements for publishing or subscribing to a surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct PubSubCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(publish), "` contract value.")]
    pub(crate) publish: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(subscribe), "` contract value.")]
    pub(crate) subscribe: Option<Vec<String>>,
}

/// Capability requirements for subscribing to a feed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct FeedCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(subscribe), "` contract value.")]
    pub(crate) subscribe: Option<Vec<String>>,
}

/// Transfer direction for RPC-backed receive grants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RpcTransferDirection {
    /// The caller downloads content from the provider.
    Receive,
}

/// One RPC transfer grant declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RpcTransfer {
    #[doc = concat!("The `", stringify!(direction), "` contract value.")]
    pub(crate) direction: RpcTransferDirection,
}

/// One owned RPC declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RpcDefinition {
    #[doc = concat!("The `", stringify!(version), "` contract value.")]
    pub(crate) version: String,
    #[doc = concat!("The `", stringify!(subject), "` contract value.")]
    #[serde(default)]
    pub(crate) subject: String,
    #[doc = concat!("The `", stringify!(input), "` contract value.")]
    pub(crate) input: SchemaReference,
    #[doc = concat!("The `", stringify!(output), "` contract value.")]
    pub(crate) output: SchemaReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(capabilities), "` contract value.")]
    pub(crate) capabilities: Option<RpcCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(errors), "` contract value.")]
    pub(crate) errors: Option<Vec<ErrorReference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(transfer), "` contract value.")]
    pub(crate) transfer: Option<RpcTransfer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(internal), "` contract value.")]
    pub(crate) internal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub(crate) docs: Option<Documentation>,
}

/// One owned event declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EventDefinition {
    #[doc = concat!("The `", stringify!(version), "` contract value.")]
    pub(crate) version: String,
    #[doc = concat!("The `", stringify!(subject), "` contract value.")]
    #[serde(default)]
    pub(crate) subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(params), "` contract value.")]
    pub(crate) params: Option<Vec<String>>,
    #[doc = concat!("The `", stringify!(event), "` contract value.")]
    pub(crate) event: SchemaReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(capabilities), "` contract value.")]
    pub(crate) capabilities: Option<PubSubCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub(crate) docs: Option<Documentation>,
}

/// One owned feed declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FeedDefinition {
    #[doc = concat!("The `", stringify!(version), "` contract value.")]
    pub(crate) version: String,
    #[doc = concat!("The `", stringify!(subject), "` contract value.")]
    #[serde(default)]
    pub(crate) subject: String,
    #[doc = concat!("The `", stringify!(input), "` contract value.")]
    pub(crate) input: SchemaReference,
    #[doc = concat!("The `", stringify!(event), "` contract value.")]
    pub(crate) event: SchemaReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(capabilities), "` contract value.")]
    pub(crate) capabilities: Option<FeedCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub(crate) docs: Option<Documentation>,
}

/// Explicit public schema exports for generated SDK consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct Exports {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[doc = concat!("The `", stringify!(schemas), "` contract value.")]
    pub(crate) schemas: Vec<String>,
}
/// Exact API-only fields used by SDK renderers.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ApiProjection {
    /// API identity.
    pub(crate) id: String,
    #[serde(default)]
    /// Named wire schemas.
    pub(crate) schemas: BTreeMap<String, Value>,
    #[serde(default)]
    /// Public schema exports.
    pub(crate) exports: Exports,
    #[serde(default)]
    /// RPC declarations.
    pub(crate) rpc: BTreeMap<String, RpcDefinition>,
    #[serde(default)]
    /// Operation declarations.
    pub(crate) operations: BTreeMap<String, OperationProjection>,
    #[serde(default)]
    /// Event declarations.
    pub(crate) events: BTreeMap<String, EventDefinition>,
    #[serde(default)]
    /// Feed declarations.
    pub(crate) feeds: BTreeMap<String, FeedDefinition>,
    #[serde(default)]
    /// Declared wire errors.
    pub(crate) errors: BTreeMap<String, ErrorDefinition>,
}

/// One native API operation used by SDK renderers.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OperationProjection {
    /// Surface version.
    pub(crate) version: String,
    /// Input schema.
    pub(crate) input: SchemaReference,
    /// Optional cumulative update schema.
    pub(crate) update: Option<SchemaReference>,
    /// Optional progress schema.
    pub(crate) progress: Option<SchemaReference>,
    /// Optional output schema.
    pub(crate) output: Option<SchemaReference>,
    /// Declared error names.
    #[serde(default)]
    pub(crate) errors: Vec<ErrorReference>,
    /// Optional API-level transfer direction.
    pub(crate) transfer: Option<TransferProjection>,
    /// Whether callers may cancel the operation.
    pub(crate) cancel: Option<bool>,
    /// Named operation signals.
    #[serde(default)]
    pub(crate) signals: BTreeMap<String, OperationDefinitionSignal>,
}

/// One named signal declaration for a running operation.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OperationDefinitionSignal {
    /// Signal input schema.
    pub(crate) input: SchemaReference,
}

/// Direction-only transfer declaration carried by a native API.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TransferProjection {
    /// Transfer direction (`send` or `receive`).
    pub(crate) direction: String,
}

/// A native API together with its canonical forms and derived routing subjects.
#[derive(Debug, Clone)]
pub(crate) struct ApiInput {
    #[doc = concat!("The `", stringify!(value), "` contract value.")]
    pub(crate) value: Value,
    /// Exact deserialization of the API-only rendering fields.
    pub(crate) render_model: ApiProjection,
    /// Derived subjects keyed by API surface.
    pub(crate) subjects: trellis_protocol::DerivedApiSubjects,
    #[doc = concat!("The `", stringify!(digest), "` contract value.")]
    pub(crate) digest: String,
}
