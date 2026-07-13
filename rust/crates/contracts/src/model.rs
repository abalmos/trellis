use std::collections::BTreeMap;
use std::ops::Index;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// The canonical format identifier for a Trellis contract manifest.
pub const CONTRACT_FORMAT_V1: &str = "trellis.contract.v1";

/// The canonical format identifier for a Trellis catalog.
pub const CATALOG_FORMAT_V1: &str = "trellis.catalog.v1";

/// The supported kinds of Trellis contracts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContractKind {
    /// A service provider.
    Service,
    /// An interactive application.
    App,
    /// An activated device.
    Device,
    /// A user-authenticated background agent.
    Agent,
}

/// A named serializable error definition declared by a contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractErrorDecl {
    #[serde(rename = "type")]
    #[doc = concat!("The `", stringify!(error_type), "` contract value.")]
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(schema), "` contract value.")]
    pub schema: Option<ContractSchemaRef>,
}

/// A reference to one named top-level contract schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractSchemaRef {
    #[doc = concat!("The `", stringify!(schema), "` contract value.")]
    pub schema: String,
}

/// A reference to a named contract error declaration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractErrorRef {
    #[serde(rename = "type")]
    #[doc = concat!("The `", stringify!(error_type), "` contract value.")]
    pub error_type: String,
}

/// Programmer-facing Markdown documentation attached to a contract surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractDocs {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(summary), "` contract value.")]
    pub summary: Option<String>,
    #[doc = concat!("The `", stringify!(markdown), "` contract value.")]
    pub markdown: String,
}

/// Capability requirements for invoking an RPC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RpcCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(call), "` contract value.")]
    pub call: Option<Vec<String>>,
}

/// Human-facing metadata for one contract-declared capability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractCapabilityMetadata {
    #[serde(rename = "displayName")]
    #[doc = concat!("The `", stringify!(display_name), "` contract value.")]
    pub display_name: String,
    #[doc = concat!("The `", stringify!(description), "` contract value.")]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(consequence), "` contract value.")]
    pub consequence: Option<String>,
}

/// Contract-declared capability metadata, keyed by capability name.
pub type ContractCapabilities = BTreeMap<String, ContractCapabilityMetadata>;

/// Capability requirements for publishing or subscribing to a surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PubSubCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(publish), "` contract value.")]
    pub publish: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(subscribe), "` contract value.")]
    pub subscribe: Option<Vec<String>>,
}

/// Capability requirements for subscribing to a feed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FeedCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(subscribe), "` contract value.")]
    pub subscribe: Option<Vec<String>>,
}

/// RPC selections from a `uses` dependency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ContractUseRpc {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(call), "` contract value.")]
    pub call: Option<Vec<String>>,
}

/// Event selections from a `uses` dependency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ContractUsePubSub {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(publish), "` contract value.")]
    pub publish: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(subscribe), "` contract value.")]
    pub subscribe: Option<Vec<String>>,
}

/// Feed selections from a `uses` dependency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ContractUseFeed {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(subscribe), "` contract value.")]
    pub subscribe: Option<Vec<String>>,
}

/// Operation selections from a `uses` dependency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ContractUseOperation {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(call), "` contract value.")]
    pub call: Option<Vec<String>>,
}

/// One cross-contract dependency declared by a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractUseRef {
    #[doc = concat!("The `", stringify!(contract), "` contract value.")]
    pub contract: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(rpc), "` contract value.")]
    pub rpc: Option<ContractUseRpc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(operations), "` contract value.")]
    pub operations: Option<ContractUseOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(events), "` contract value.")]
    pub events: Option<ContractUsePubSub>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(feeds), "` contract value.")]
    pub feeds: Option<ContractUseFeed>,
}

/// Contract dependency declarations split by whether they are required at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContractUses {
    required: BTreeMap<String, ContractUseRef>,
    optional: BTreeMap<String, ContractUseRef>,
}

#[derive(Debug, Deserialize)]
struct ContractUsesGroupedWire {
    #[serde(default)]
    required: BTreeMap<String, ContractUseRef>,
    #[serde(default)]
    optional: BTreeMap<String, ContractUseRef>,
}

impl<'de> Deserialize<'de> for ContractUses {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let grouped = ContractUsesGroupedWire::deserialize(deserializer)?;
        Ok(Self {
            required: grouped.required,
            optional: grouped.optional,
        })
    }
}

impl Serialize for ContractUses {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct GroupedUses<'a> {
            #[serde(skip_serializing_if = "BTreeMap::is_empty")]
            required: &'a BTreeMap<String, ContractUseRef>,
            #[serde(skip_serializing_if = "BTreeMap::is_empty")]
            optional: &'a BTreeMap<String, ContractUseRef>,
        }

        GroupedUses {
            required: &self.required,
            optional: &self.optional,
        }
        .serialize(serializer)
    }
}

/// Supported Trellis-managed state store shapes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContractStateKind {
    /// One value per participant scope.
    Value,
    /// Multiple keyed values per participant scope.
    Map,
}

/// One Trellis-managed state store declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractStateStore {
    #[doc = concat!("The `", stringify!(kind), "` contract value.")]
    pub kind: ContractStateKind,
    #[doc = concat!("The `", stringify!(schema), "` contract value.")]
    pub schema: ContractSchemaRef,
    #[serde(rename = "stateVersion", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(state_version), "` contract value.")]
    pub state_version: Option<String>,
    #[serde(
        rename = "acceptedVersions",
        skip_serializing_if = "BTreeMap::is_empty",
        default
    )]
    #[doc = concat!("The `", stringify!(accepted_versions), "` contract value.")]
    pub accepted_versions: BTreeMap<String, ContractSchemaRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
}

/// Capability requirements for invoking and observing an operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct OperationCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(call), "` contract value.")]
    pub call: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(observe), "` contract value.")]
    pub observe: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(cancel), "` contract value.")]
    pub cancel: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(control), "` contract value.")]
    pub control: Option<Vec<String>>,
}

/// Transfer direction for operation-backed file uploads.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContractOperationTransferDirection {
    /// The caller uploads content to the provider.
    Send,
}

/// File-transfer configuration for an operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractOperationTransfer {
    #[doc = concat!("The `", stringify!(direction), "` contract value.")]
    pub direction: ContractOperationTransferDirection,
    #[doc = concat!("The `", stringify!(store), "` contract value.")]
    pub store: String,
    #[doc = concat!("The `", stringify!(key), "` contract value.")]
    pub key: String,
    #[serde(rename = "contentType", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(content_type), "` contract value.")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(metadata), "` contract value.")]
    pub metadata: Option<String>,
    #[serde(rename = "expiresInMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(expires_in_ms), "` contract value.")]
    pub expires_in_ms: Option<i64>,
    #[serde(rename = "maxBytes", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_bytes), "` contract value.")]
    pub max_bytes: Option<i64>,
}

/// One owned operation declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractOperation {
    #[doc = concat!("The `", stringify!(version), "` contract value.")]
    pub version: String,
    #[doc = concat!("The `", stringify!(subject), "` contract value.")]
    pub subject: String,
    #[doc = concat!("The `", stringify!(input), "` contract value.")]
    pub input: ContractSchemaRef,
    /// Optional schema for live-only cumulative updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(update), "` contract value.")]
    pub update: Option<ContractSchemaRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(progress), "` contract value.")]
    pub progress: Option<ContractSchemaRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(output), "` contract value.")]
    pub output: Option<ContractSchemaRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(errors), "` contract value.")]
    pub errors: Option<Vec<ContractErrorRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(transfer), "` contract value.")]
    pub transfer: Option<ContractOperationTransfer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(capabilities), "` contract value.")]
    pub capabilities: Option<OperationCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(cancel), "` contract value.")]
    pub cancel: Option<bool>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(signals), "` contract value.")]
    pub signals: BTreeMap<String, ContractOperationSignal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
}

/// One named signal declaration for a running operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractOperationSignal {
    #[doc = concat!("The `", stringify!(input), "` contract value.")]
    pub input: ContractSchemaRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
}

/// Transfer direction for RPC-backed receive grants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContractRpcTransferDirection {
    /// The caller downloads content from the provider.
    Receive,
}

/// One RPC transfer grant declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractRpcTransfer {
    #[doc = concat!("The `", stringify!(direction), "` contract value.")]
    pub direction: ContractRpcTransferDirection,
}

/// One owned RPC declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractRpcMethod {
    #[doc = concat!("The `", stringify!(version), "` contract value.")]
    pub version: String,
    #[doc = concat!("The `", stringify!(subject), "` contract value.")]
    pub subject: String,
    #[doc = concat!("The `", stringify!(input), "` contract value.")]
    pub input: ContractSchemaRef,
    #[doc = concat!("The `", stringify!(output), "` contract value.")]
    pub output: ContractSchemaRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(capabilities), "` contract value.")]
    pub capabilities: Option<RpcCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(errors), "` contract value.")]
    pub errors: Option<Vec<ContractErrorRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(transfer), "` contract value.")]
    pub transfer: Option<ContractRpcTransfer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(internal), "` contract value.")]
    pub internal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
}

/// One owned event declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractEvent {
    #[doc = concat!("The `", stringify!(version), "` contract value.")]
    pub version: String,
    #[doc = concat!("The `", stringify!(subject), "` contract value.")]
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(params), "` contract value.")]
    pub params: Option<Vec<String>>,
    #[doc = concat!("The `", stringify!(event), "` contract value.")]
    pub event: ContractSchemaRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(capabilities), "` contract value.")]
    pub capabilities: Option<PubSubCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
}

/// One owned feed declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractFeed {
    #[doc = concat!("The `", stringify!(version), "` contract value.")]
    pub version: String,
    #[doc = concat!("The `", stringify!(subject), "` contract value.")]
    pub subject: String,
    #[doc = concat!("The `", stringify!(input), "` contract value.")]
    pub input: ContractSchemaRef,
    #[doc = concat!("The `", stringify!(event), "` contract value.")]
    pub event: ContractSchemaRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(capabilities), "` contract value.")]
    pub capabilities: Option<FeedCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
}

/// Replay policy for a durable event consumer group.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContractEventConsumerReplay {
    #[default]
    /// Deliver only events published after consumer creation.
    New,
    /// Replay all retained events before live delivery.
    All,
}

/// Ordering policy for a durable event consumer group.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContractEventConsumerOrdering {
    #[default]
    /// Deliver one event at a time in stream order.
    Strict,
    /// Permit concurrent delivery.
    Parallel,
}

/// One durable event consumer group declared by a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractEventConsumerGroup {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(uses), "` contract value.")]
    pub uses: BTreeMap<String, Vec<String>>,
    #[serde(rename = "self", default, skip_serializing_if = "Vec::is_empty")]
    #[doc = concat!("The `", stringify!(self_events), "` contract value.")]
    pub self_events: Vec<String>,
    #[serde(default)]
    #[doc = concat!("The `", stringify!(replay), "` contract value.")]
    pub replay: ContractEventConsumerReplay,
    #[serde(default)]
    #[doc = concat!("The `", stringify!(ordering), "` contract value.")]
    pub ordering: ContractEventConsumerOrdering,
    #[serde(rename = "ackWaitMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(ack_wait_ms), "` contract value.")]
    pub ack_wait_ms: Option<i64>,
    #[serde(rename = "maxDeliver", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_deliver), "` contract value.")]
    pub max_deliver: Option<i64>,
    #[serde(rename = "backoffMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(backoff_ms), "` contract value.")]
    pub backoff_ms: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
}

/// One logical KV resource declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractKvResource {
    #[doc = concat!("The `", stringify!(purpose), "` contract value.")]
    pub purpose: String,
    #[doc = concat!("The `", stringify!(schema), "` contract value.")]
    pub schema: ContractSchemaRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(required), "` contract value.")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(history), "` contract value.")]
    pub history: Option<i64>,
    #[serde(rename = "ttlMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(ttl_ms), "` contract value.")]
    pub ttl_ms: Option<i64>,
    #[serde(rename = "maxValueBytes", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_value_bytes), "` contract value.")]
    pub max_value_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
}

/// One logical store resource declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractStoreResource {
    #[doc = concat!("The `", stringify!(purpose), "` contract value.")]
    pub purpose: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(required), "` contract value.")]
    pub required: Option<bool>,
    #[serde(rename = "ttlMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(ttl_ms), "` contract value.")]
    pub ttl_ms: Option<i64>,
    #[serde(rename = "maxObjectBytes", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_object_bytes), "` contract value.")]
    pub max_object_bytes: Option<i64>,
    #[serde(rename = "maxTotalBytes", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_total_bytes), "` contract value.")]
    pub max_total_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
}

/// Stale active-key policy for keyed jobs queues.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum JobKeyConcurrencyStalePolicy {
    /// Fail jobs that hold a stale active-key lease.
    FailStale,
    /// Keep later jobs blocked until the stale lease is resolved.
    Block,
}

/// Per-key active job policy for one logical jobs queue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobKeyConcurrencyDescriptor {
    #[doc = concat!("The `", stringify!(key), "` contract value.")]
    pub key: Vec<String>,
    #[serde(rename = "maxActive", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_active), "` contract value.")]
    pub max_active: Option<i64>,
    #[serde(
        rename = "heartbeatIntervalMs",
        skip_serializing_if = "Option::is_none"
    )]
    #[doc = concat!("The `", stringify!(heartbeat_interval_ms), "` contract value.")]
    pub heartbeat_interval_ms: Option<i64>,
    #[serde(rename = "heartbeatTtlMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(heartbeat_ttl_ms), "` contract value.")]
    pub heartbeat_ttl_ms: Option<i64>,
    #[serde(rename = "stalePolicy", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(stale_policy), "` contract value.")]
    pub stale_policy: Option<JobKeyConcurrencyStalePolicy>,
}

/// Queue-depth policy for one keyed jobs queue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobQueueDepthDescriptor {
    #[serde(rename = "maxQueuedPerKey", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_queued_per_key), "` contract value.")]
    pub max_queued_per_key: Option<i64>,
    #[serde(rename = "whenFull", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(when_full), "` contract value.")]
    pub when_full: Option<JobQueueWhenFullPolicy>,
}

/// Admission behavior when a keyed queue is full for a derived key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum JobQueueWhenFullPolicy {
    /// Reject the incoming job.
    Reject,
    /// Merge the incoming request with an existing queued job.
    Coalesce,
    /// Replace the oldest queued job for the key.
    ReplaceOldest,
}

/// One logical jobs queue declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractJobQueueResource {
    #[doc = concat!("The `", stringify!(payload), "` contract value.")]
    pub payload: ContractSchemaRef,
    /// Optional schema for live-only cumulative updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(update), "` contract value.")]
    pub update: Option<ContractSchemaRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(result), "` contract value.")]
    pub result: Option<ContractSchemaRef>,
    #[serde(rename = "maxDeliver", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_deliver), "` contract value.")]
    pub max_deliver: Option<i64>,
    #[serde(rename = "backoffMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(backoff_ms), "` contract value.")]
    pub backoff_ms: Option<Vec<i64>>,
    #[serde(rename = "ackWaitMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(ack_wait_ms), "` contract value.")]
    pub ack_wait_ms: Option<i64>,
    #[serde(rename = "defaultDeadlineMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(default_deadline_ms), "` contract value.")]
    pub default_deadline_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(progress), "` contract value.")]
    pub progress: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(logs), "` contract value.")]
    pub logs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(dlq), "` contract value.")]
    pub dlq: Option<bool>,
    #[serde(rename = "keyConcurrency", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(key_concurrency), "` contract value.")]
    pub key_concurrency: Option<JobKeyConcurrencyDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(queue), "` contract value.")]
    pub queue: Option<JobQueueDepthDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
}

/// Resource declarations in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ContractResources {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(kv), "` contract value.")]
    pub kv: BTreeMap<String, ContractKvResource>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(store), "` contract value.")]
    pub store: BTreeMap<String, ContractStoreResource>,
}

/// Explicit public schema exports for generated SDK consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ContractExports {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[doc = concat!("The `", stringify!(schemas), "` contract value.")]
    pub schemas: Vec<String>,
}

/// The canonical Trellis contract manifest model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractManifest {
    #[doc = concat!("The `", stringify!(format), "` contract value.")]
    pub format: String,
    #[doc = concat!("The `", stringify!(id), "` contract value.")]
    pub id: String,
    #[serde(rename = "displayName")]
    #[doc = concat!("The `", stringify!(display_name), "` contract value.")]
    pub display_name: String,
    #[doc = concat!("The `", stringify!(description), "` contract value.")]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
    #[doc = concat!("The `", stringify!(kind), "` contract value.")]
    pub kind: ContractKind,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(capabilities), "` contract value.")]
    pub capabilities: ContractCapabilities,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(schemas), "` contract value.")]
    pub schemas: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "ContractExports::is_empty")]
    #[doc = concat!("The `", stringify!(exports), "` contract value.")]
    pub exports: ContractExports,
    #[serde(default, skip_serializing_if = "ContractUses::is_empty")]
    #[doc = concat!("The `", stringify!(uses), "` contract value.")]
    pub uses: ContractUses,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(state), "` contract value.")]
    pub state: BTreeMap<String, ContractStateStore>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(rpc), "` contract value.")]
    pub rpc: BTreeMap<String, ContractRpcMethod>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(operations), "` contract value.")]
    pub operations: BTreeMap<String, ContractOperation>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(events), "` contract value.")]
    pub events: BTreeMap<String, ContractEvent>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(feeds), "` contract value.")]
    pub feeds: BTreeMap<String, ContractFeed>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(errors), "` contract value.")]
    pub errors: BTreeMap<String, ContractErrorDecl>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(jobs), "` contract value.")]
    pub jobs: BTreeMap<String, ContractJobQueueResource>,
    #[serde(
        rename = "eventConsumers",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    #[doc = concat!("The `", stringify!(event_consumers), "` contract value.")]
    pub event_consumers: BTreeMap<String, ContractEventConsumerGroup>,
    #[serde(default, skip_serializing_if = "ContractResources::is_empty")]
    #[doc = concat!("The `", stringify!(resources), "` contract value.")]
    pub resources: ContractResources,
}

impl ContractResources {
    fn is_empty(&self) -> bool {
        self.kv.is_empty() && self.store.is_empty()
    }
}

impl ContractExports {
    fn is_empty(&self) -> bool {
        self.schemas.is_empty()
    }
}

impl ContractUses {
    /// Return whether no required or optional dependency aliases are declared.
    pub fn is_empty(&self) -> bool {
        self.required.is_empty() && self.optional.is_empty()
    }

    /// Return required dependency aliases.
    pub fn required(&self) -> &BTreeMap<String, ContractUseRef> {
        &self.required
    }

    /// Return mutable required dependency aliases.
    pub fn required_mut(&mut self) -> &mut BTreeMap<String, ContractUseRef> {
        &mut self.required
    }

    /// Return optional dependency aliases.
    pub fn optional(&self) -> &BTreeMap<String, ContractUseRef> {
        &self.optional
    }

    /// Return mutable optional dependency aliases.
    pub fn optional_mut(&mut self) -> &mut BTreeMap<String, ContractUseRef> {
        &mut self.optional
    }

    /// Return a dependency alias, searching required aliases before optional aliases.
    pub fn get(&self, alias: &str) -> Option<&ContractUseRef> {
        self.required
            .get(alias)
            .or_else(|| self.optional.get(alias))
    }

    /// Return whether a dependency alias is declared in either group.
    pub fn contains_key(&self, alias: &str) -> bool {
        self.get(alias).is_some()
    }

    /// Iterate over required aliases first, followed by optional aliases.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ContractUseRef)> {
        self.required.iter().chain(
            self.optional
                .iter()
                .filter(|(alias, _)| !self.required.contains_key(*alias)),
        )
    }
}

impl Index<&str> for ContractUses {
    type Output = ContractUseRef;

    fn index(&self, index: &str) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("no contract use alias '{index}'"))
    }
}

/// The deployment-wide active contract catalog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Catalog {
    #[doc = concat!("The `", stringify!(format), "` contract value.")]
    pub format: String,
    #[doc = concat!("The `", stringify!(contracts), "` contract value.")]
    pub contracts: Vec<CatalogEntry>,
}

/// One active contract entry in a catalog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogEntry {
    #[doc = concat!("The `", stringify!(id), "` contract value.")]
    pub id: String,
    #[doc = concat!("The `", stringify!(digest), "` contract value.")]
    pub digest: String,
    #[serde(rename = "displayName")]
    #[doc = concat!("The `", stringify!(display_name), "` contract value.")]
    pub display_name: String,
    #[doc = concat!("The `", stringify!(description), "` contract value.")]
    pub description: String,
}

/// A manifest together with its parsed, canonicalized, and digested forms.
#[derive(Debug, Clone)]
pub struct LoadedManifest {
    #[doc = concat!("The `", stringify!(path), "` contract value.")]
    pub path: PathBuf,
    #[doc = concat!("The `", stringify!(value), "` contract value.")]
    pub value: Value,
    #[doc = concat!("The `", stringify!(manifest), "` contract value.")]
    pub manifest: ContractManifest,
    #[doc = concat!("The `", stringify!(canonical), "` contract value.")]
    pub canonical: String,
    #[doc = concat!("The `", stringify!(digest), "` contract value.")]
    pub digest: String,
}

/// The result of packing multiple manifests into one catalog.
#[derive(Debug, Clone)]
pub struct CatalogPack {
    #[doc = concat!("The `", stringify!(catalog), "` contract value.")]
    pub catalog: Catalog,
    #[doc = concat!("The `", stringify!(contracts), "` contract value.")]
    pub contracts: Vec<LoadedManifest>,
}
