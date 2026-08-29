use std::collections::BTreeMap;
use std::ops::Index;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// The canonical format identifier for a native Trellis API source.
pub const API_FORMAT_V1: &str = "trellis.api.v1";

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
    #[serde(skip)]
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
#[serde(transparent)]
pub struct ContractErrorRef {
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
    /// Operations whose cancel controls are selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel: Option<Vec<String>>,
    /// Operations whose signal controls are selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control: Option<Vec<String>>,
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
    #[serde(default)]
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
    #[serde(default)]
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
    #[serde(default)]
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
    #[serde(default)]
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

/// Private in-memory state shared by the native API and participant builders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthoringState {
    /// Exact owned API identity.
    pub api_id: String,
    /// Independently authored API release version.
    pub api_version: String,
    /// Stable participant identity when producing participant artifacts.
    pub participant_id: Option<String>,
    #[doc = concat!("The `", stringify!(display_name), "` contract value.")]
    pub display_name: String,
    #[doc = concat!("The `", stringify!(description), "` contract value.")]
    pub description: String,
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub docs: Option<ContractDocs>,
    #[doc = concat!("The `", stringify!(kind), "` contract value.")]
    pub kind: ContractKind,
    #[doc = concat!("The `", stringify!(capabilities), "` contract value.")]
    pub capabilities: ContractCapabilities,
    #[doc = concat!("The `", stringify!(schemas), "` contract value.")]
    pub schemas: BTreeMap<String, Value>,
    #[doc = concat!("The `", stringify!(exports), "` contract value.")]
    pub exports: ContractExports,
    #[doc = concat!("The `", stringify!(uses), "` contract value.")]
    pub uses: ContractUses,
    #[doc = concat!("The `", stringify!(state), "` contract value.")]
    pub state: BTreeMap<String, ContractStateStore>,
    #[doc = concat!("The `", stringify!(rpc), "` contract value.")]
    pub rpc: BTreeMap<String, ContractRpcMethod>,
    #[doc = concat!("The `", stringify!(operations), "` contract value.")]
    pub operations: BTreeMap<String, ContractOperation>,
    #[doc = concat!("The `", stringify!(events), "` contract value.")]
    pub events: BTreeMap<String, ContractEvent>,
    #[doc = concat!("The `", stringify!(feeds), "` contract value.")]
    pub feeds: BTreeMap<String, ContractFeed>,
    #[doc = concat!("The `", stringify!(errors), "` contract value.")]
    pub errors: BTreeMap<String, ContractErrorDecl>,
    #[doc = concat!("The `", stringify!(jobs), "` contract value.")]
    pub jobs: BTreeMap<String, ContractJobQueueResource>,
    #[doc = concat!("The `", stringify!(event_consumers), "` contract value.")]
    pub event_consumers: BTreeMap<String, ContractEventConsumerGroup>,
    #[doc = concat!("The `", stringify!(resources), "` contract value.")]
    pub resources: ContractResources,
}

/// Exact API-only fields used by SDK renderers.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiRenderModel {
    /// API identity.
    pub id: String,
    #[serde(rename = "displayName")]
    /// Human-facing API name.
    pub display_name: String,
    /// Human-facing API description.
    pub description: String,
    /// Programmer-facing API documentation.
    pub docs: Option<ContractDocs>,
    #[serde(default)]
    /// Named wire schemas.
    pub schemas: BTreeMap<String, Value>,
    #[serde(default)]
    /// Public schema exports.
    pub exports: ContractExports,
    #[serde(default)]
    /// State declarations.
    pub state: BTreeMap<String, ContractStateStore>,
    #[serde(default)]
    /// RPC declarations.
    pub rpc: BTreeMap<String, ContractRpcMethod>,
    #[serde(default)]
    /// Operation declarations.
    pub operations: BTreeMap<String, ApiOperationRenderModel>,
    #[serde(default)]
    /// Event declarations.
    pub events: BTreeMap<String, ContractEvent>,
    #[serde(default)]
    /// Feed declarations.
    pub feeds: BTreeMap<String, ContractFeed>,
    #[serde(default)]
    /// Declared wire errors.
    pub errors: BTreeMap<String, ContractErrorDecl>,
}

/// One native API operation used by SDK renderers.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiOperationRenderModel {
    /// Surface version.
    pub version: String,
    /// Input schema.
    pub input: ContractSchemaRef,
    /// Optional cumulative update schema.
    pub update: Option<ContractSchemaRef>,
    /// Optional progress schema.
    pub progress: Option<ContractSchemaRef>,
    /// Optional output schema.
    pub output: Option<ContractSchemaRef>,
    /// Declared error names.
    #[serde(default)]
    pub errors: Vec<ContractErrorRef>,
    /// Optional API-level transfer direction.
    pub transfer: Option<ApiTransferRenderModel>,
    /// Whether callers may cancel the operation.
    pub cancel: Option<bool>,
    /// Named operation signals.
    #[serde(default)]
    pub signals: BTreeMap<String, ContractOperationSignal>,
    /// Programmer-facing documentation.
    pub docs: Option<ContractDocs>,
}

/// Direction-only transfer declaration carried by a native API.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiTransferRenderModel {
    /// Transfer direction (`send` or `receive`).
    pub direction: String,
}

/// Exact participant-only fields used by facade renderers.
#[derive(Debug, Clone, Deserialize)]
pub struct ParticipantRenderModel {
    /// Participant identity.
    pub id: String,
    /// Participant kind.
    pub kind: ContractKind,
    /// Participant-owned schemas used by State, jobs, and resources.
    #[serde(default)]
    pub schemas: BTreeMap<String, Value>,
    /// Participant API selections.
    #[serde(default)]
    pub uses: ParticipantUsesRenderModel,
    /// Participant-owned state metadata.
    #[serde(default)]
    pub state: BTreeMap<String, ContractStateStore>,
    /// Participant-owned job queues.
    #[serde(rename = "jobQueues", default)]
    pub jobs: BTreeMap<String, ContractJobQueueResource>,
    /// Participant-owned durable event consumers.
    #[serde(rename = "eventConsumers", default)]
    pub event_consumers: BTreeMap<String, ContractEventConsumerGroup>,
    /// Participant-owned resource requirements.
    #[serde(default)]
    pub resources: ContractResources,
}

/// Native participant dependency selections used by facade renderers.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ParticipantUsesRenderModel {
    #[serde(default)]
    required: BTreeMap<String, ParticipantUseRenderModel>,
    #[serde(default)]
    optional: BTreeMap<String, ParticipantUseRenderModel>,
}

/// One exact native API dependency selection.
#[derive(Debug, Clone, Deserialize)]
pub struct ParticipantUseRenderModel {
    /// Referenced API identity.
    pub api: String,
    /// Exact referenced API digest.
    #[serde(rename = "apiDigest")]
    pub api_digest: String,
    /// Selected RPC calls.
    pub rpc: Option<ContractUseRpc>,
    /// Selected operation actions.
    pub operations: Option<ParticipantOperationUseRenderModel>,
    /// Selected event actions.
    pub events: Option<ContractUsePubSub>,
    /// Selected feed subscriptions.
    pub feeds: Option<ContractUseFeed>,
}

/// Native operation selections for one used API.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ParticipantOperationUseRenderModel {
    /// Operations the participant may invoke.
    pub invoke: Option<Vec<String>>,
    /// Operations the participant may observe.
    pub observe: Option<Vec<String>>,
    /// Operations the participant may cancel.
    pub cancel: Option<Vec<String>>,
    /// Signals selected per operation.
    pub control: Option<BTreeMap<String, Vec<String>>>,
}

impl ParticipantUsesRenderModel {
    /// Return a dependency alias, searching required before optional aliases.
    pub fn get(&self, alias: &str) -> Option<&ParticipantUseRenderModel> {
        self.required
            .get(alias)
            .or_else(|| self.optional.get(alias))
    }

    /// Iterate over required aliases first, followed by optional aliases.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ParticipantUseRenderModel)> {
        self.required.iter().chain(
            self.optional
                .iter()
                .filter(|(alias, _)| !self.required.contains_key(*alias)),
        )
    }
}

/// A strict native participant artifact and its participant-only rendering fields.
#[derive(Debug, Clone)]
pub struct LoadedParticipant {
    /// Source path.
    pub path: PathBuf,
    /// Parsed protocol-owned participant artifact.
    pub participant: trellis_protocol::ParticipantArtifact,
    /// Participant-only rendering fields.
    pub render_model: ParticipantRenderModel,
    /// Normalized participant JSON.
    pub value: Value,
    /// Canonical participant JSON.
    pub canonical: String,
    /// Semantic participant digest.
    pub digest: String,
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

/// A native API together with its canonical forms and derived routing subjects.
#[derive(Debug, Clone)]
pub struct LoadedApi {
    #[doc = concat!("The `", stringify!(path), "` contract value.")]
    pub path: PathBuf,
    #[doc = concat!("The `", stringify!(value), "` contract value.")]
    pub value: Value,
    /// Parsed protocol-owned native API artifact.
    pub api: trellis_protocol::ApiArtifact,
    /// Exact deserialization of the API-only rendering fields.
    pub render_model: ApiRenderModel,
    /// Derived subjects keyed by API surface.
    pub subjects: trellis_protocol::DerivedApiSubjects,
    #[doc = concat!("The `", stringify!(canonical), "` contract value.")]
    pub canonical: String,
    #[doc = concat!("The `", stringify!(digest), "` contract value.")]
    pub digest: String,
}
