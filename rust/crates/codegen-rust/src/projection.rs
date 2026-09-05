use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The supported kinds of Trellis contracts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ParticipantKind {
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

/// RPC selections from a `uses` dependency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct UsedRpc {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(call), "` contract value.")]
    pub(crate) call: Option<Vec<String>>,
}

/// Event selections from a `uses` dependency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct UsedPubSub {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(publish), "` contract value.")]
    pub(crate) publish: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(subscribe), "` contract value.")]
    pub(crate) subscribe: Option<Vec<String>>,
}

/// Feed selections from a `uses` dependency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct UsedFeed {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(subscribe), "` contract value.")]
    pub(crate) subscribe: Option<Vec<String>>,
}

/// Supported Trellis-managed state store shapes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum StateKind {
    /// One value per participant scope.
    Value,
    /// Multiple keyed values per participant scope.
    Map,
}

/// One Trellis-managed state store declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StateDefinition {
    #[doc = concat!("The `", stringify!(kind), "` contract value.")]
    pub(crate) kind: StateKind,
    #[doc = concat!("The `", stringify!(schema), "` contract value.")]
    pub(crate) schema: SchemaReference,
    #[serde(rename = "stateVersion", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(state_version), "` contract value.")]
    pub(crate) state_version: Option<String>,
    #[serde(
        rename = "acceptedVersions",
        skip_serializing_if = "BTreeMap::is_empty",
        default
    )]
    #[doc = concat!("The `", stringify!(accepted_versions), "` contract value.")]
    pub(crate) accepted_versions: BTreeMap<String, SchemaReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub(crate) docs: Option<Documentation>,
}

/// One named signal declaration for a running operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperationDefinitionSignal {
    #[doc = concat!("The `", stringify!(input), "` contract value.")]
    pub(crate) input: SchemaReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub(crate) docs: Option<Documentation>,
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

/// Replay policy for a durable event consumer group.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum EventDefinitionConsumerReplay {
    #[default]
    /// Deliver only events published after consumer creation.
    New,
    /// Replay all retained events before live delivery.
    All,
}

/// Ordering policy for a durable event consumer group.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum EventDefinitionConsumerOrdering {
    #[default]
    /// Deliver one event at a time in stream order.
    Strict,
    /// Permit concurrent delivery.
    Parallel,
}

/// One durable event consumer group declared by a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EventDefinitionConsumerGroup {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(uses), "` contract value.")]
    pub(crate) uses: BTreeMap<String, Vec<String>>,
    #[serde(rename = "self", default, skip_serializing_if = "Vec::is_empty")]
    #[doc = concat!("The `", stringify!(self_events), "` contract value.")]
    pub(crate) self_events: Vec<String>,
    #[serde(default)]
    #[doc = concat!("The `", stringify!(replay), "` contract value.")]
    pub(crate) replay: EventDefinitionConsumerReplay,
    #[serde(default)]
    #[doc = concat!("The `", stringify!(ordering), "` contract value.")]
    pub(crate) ordering: EventDefinitionConsumerOrdering,
    #[serde(rename = "ackWaitMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(ack_wait_ms), "` contract value.")]
    pub(crate) ack_wait_ms: Option<i64>,
    #[serde(rename = "maxDeliver", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_deliver), "` contract value.")]
    pub(crate) max_deliver: Option<i64>,
    #[serde(rename = "backoffMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(backoff_ms), "` contract value.")]
    pub(crate) backoff_ms: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub(crate) docs: Option<Documentation>,
}

/// One logical KV resource declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct KvDefinition {
    #[doc = concat!("The `", stringify!(purpose), "` contract value.")]
    pub(crate) purpose: String,
    #[doc = concat!("The `", stringify!(schema), "` contract value.")]
    pub(crate) schema: SchemaReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(required), "` contract value.")]
    pub(crate) required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(history), "` contract value.")]
    pub(crate) history: Option<i64>,
    #[serde(rename = "ttlMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(ttl_ms), "` contract value.")]
    pub(crate) ttl_ms: Option<i64>,
    #[serde(rename = "maxValueBytes", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_value_bytes), "` contract value.")]
    pub(crate) max_value_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub(crate) docs: Option<Documentation>,
}

/// One logical store resource declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StoreDefinition {
    #[doc = concat!("The `", stringify!(purpose), "` contract value.")]
    pub(crate) purpose: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(required), "` contract value.")]
    pub(crate) required: Option<bool>,
    #[serde(rename = "ttlMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(ttl_ms), "` contract value.")]
    pub(crate) ttl_ms: Option<i64>,
    #[serde(rename = "maxObjectBytes", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_object_bytes), "` contract value.")]
    pub(crate) max_object_bytes: Option<i64>,
    #[serde(rename = "maxTotalBytes", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_total_bytes), "` contract value.")]
    pub(crate) max_total_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub(crate) docs: Option<Documentation>,
}

/// Stale active-key policy for keyed jobs queues.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum JobKeyConcurrencyStalePolicy {
    /// Fail jobs that hold a stale active-key lease.
    FailStale,
    /// Keep later jobs blocked until the stale lease is resolved.
    Block,
}

/// Per-key active job policy for one logical jobs queue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct JobKeyConcurrencyDescriptor {
    #[doc = concat!("The `", stringify!(key), "` contract value.")]
    pub(crate) key: Vec<String>,
    #[serde(rename = "maxActive", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_active), "` contract value.")]
    pub(crate) max_active: Option<i64>,
    #[serde(
        rename = "heartbeatIntervalMs",
        skip_serializing_if = "Option::is_none"
    )]
    #[doc = concat!("The `", stringify!(heartbeat_interval_ms), "` contract value.")]
    pub(crate) heartbeat_interval_ms: Option<i64>,
    #[serde(rename = "heartbeatTtlMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(heartbeat_ttl_ms), "` contract value.")]
    pub(crate) heartbeat_ttl_ms: Option<i64>,
    #[serde(rename = "stalePolicy", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(stale_policy), "` contract value.")]
    pub(crate) stale_policy: Option<JobKeyConcurrencyStalePolicy>,
}

/// Queue-depth policy for one keyed jobs queue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct JobQueueDepthDescriptor {
    #[serde(rename = "maxQueuedPerKey", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_queued_per_key), "` contract value.")]
    pub(crate) max_queued_per_key: Option<i64>,
    #[serde(rename = "whenFull", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(when_full), "` contract value.")]
    pub(crate) when_full: Option<JobQueueWhenFullPolicy>,
}

/// Admission behavior when a keyed queue is full for a derived key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum JobQueueWhenFullPolicy {
    /// Reject the incoming job.
    Reject,
    /// Merge the incoming request with an existing queued job.
    Coalesce,
    /// Replace the oldest queued job for the key.
    ReplaceOldest,
}

/// One logical jobs queue declaration in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct JobDefinition {
    #[doc = concat!("The `", stringify!(payload), "` contract value.")]
    pub(crate) payload: SchemaReference,
    /// Optional schema for live-only cumulative updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(update), "` contract value.")]
    pub(crate) update: Option<SchemaReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(result), "` contract value.")]
    pub(crate) result: Option<SchemaReference>,
    #[serde(rename = "maxDeliver", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_deliver), "` contract value.")]
    pub(crate) max_deliver: Option<i64>,
    #[serde(rename = "backoffMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(backoff_ms), "` contract value.")]
    pub(crate) backoff_ms: Option<Vec<i64>>,
    #[serde(rename = "ackWaitMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(ack_wait_ms), "` contract value.")]
    pub(crate) ack_wait_ms: Option<i64>,
    #[serde(rename = "defaultDeadlineMs", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(default_deadline_ms), "` contract value.")]
    pub(crate) default_deadline_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(progress), "` contract value.")]
    pub(crate) progress: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(logs), "` contract value.")]
    pub(crate) logs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(dlq), "` contract value.")]
    pub(crate) dlq: Option<bool>,
    #[serde(rename = "keyConcurrency", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(key_concurrency), "` contract value.")]
    pub(crate) key_concurrency: Option<JobKeyConcurrencyDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(queue), "` contract value.")]
    pub(crate) queue: Option<JobQueueDepthDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(docs), "` contract value.")]
    pub(crate) docs: Option<Documentation>,
}

/// Resource declarations in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct ResourceDefinitions {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(kv), "` contract value.")]
    pub(crate) kv: BTreeMap<String, KvDefinition>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[doc = concat!("The `", stringify!(store), "` contract value.")]
    pub(crate) store: BTreeMap<String, StoreDefinition>,
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
    #[serde(rename = "displayName")]
    /// Human-facing API name.
    pub(crate) display_name: String,
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

/// Direction-only transfer declaration carried by a native API.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TransferProjection {}

/// Exact participant-only fields used by facade renderers.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ParticipantProjection {
    /// Participant identity.
    pub(crate) id: String,
    /// Participant kind.
    pub(crate) kind: ParticipantKind,
    /// Participant-owned schemas used by State, jobs, and resources.
    #[serde(default)]
    pub(crate) schemas: BTreeMap<String, Value>,
    /// Participant API selections.
    #[serde(default)]
    pub(crate) uses: ParticipantUses,
    /// Participant-owned state metadata.
    #[serde(default)]
    pub(crate) state: BTreeMap<String, StateDefinition>,
    /// Participant-owned job queues.
    #[serde(rename = "jobQueues", default)]
    pub(crate) jobs: BTreeMap<String, JobDefinition>,
    /// Participant-owned durable event consumers.
    #[serde(rename = "eventConsumers", default)]
    pub(crate) event_consumers: BTreeMap<String, EventDefinitionConsumerGroup>,
    /// Participant-owned resource requirements.
    #[serde(default)]
    pub(crate) resources: ResourceDefinitions,
}

/// Native participant dependency selections used by facade renderers.
#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct ParticipantUses {
    #[serde(default)]
    required: BTreeMap<String, ParticipantUse>,
    #[serde(default)]
    optional: BTreeMap<String, ParticipantUse>,
}

/// One exact native API dependency selection.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ParticipantUse {
    /// Referenced API identity.
    pub(crate) api: String,
    /// Selected RPC calls.
    pub(crate) rpc: Option<UsedRpc>,
    /// Selected operation actions.
    pub(crate) operations: Option<ParticipantOperationUse>,
    /// Selected event actions.
    pub(crate) events: Option<UsedPubSub>,
    /// Selected feed subscriptions.
    pub(crate) feeds: Option<UsedFeed>,
}

/// Native operation selections for one used API.
#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct ParticipantOperationUse {
    /// Operations the participant may invoke.
    pub(crate) invoke: Option<Vec<String>>,
}

impl ParticipantUses {
    /// Return a dependency alias, searching required before optional aliases.
    pub(crate) fn get(&self, alias: &str) -> Option<&ParticipantUse> {
        self.required
            .get(alias)
            .or_else(|| self.optional.get(alias))
    }

    /// Iterate over required aliases first, followed by optional aliases.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&String, &ParticipantUse)> {
        self.required.iter().chain(
            self.optional
                .iter()
                .filter(|(alias, _)| !self.required.contains_key(*alias)),
        )
    }
}

/// A strict native participant artifact and its participant-only rendering fields.
#[derive(Debug, Clone)]
pub(crate) struct ParticipantInput {
    /// Parsed protocol-owned participant artifact.
    pub(crate) participant: trellis_protocol::ParticipantArtifact,
    /// Participant-only rendering fields.
    pub(crate) render_model: ParticipantProjection,
    /// Canonical participant JSON.
    pub(crate) canonical: String,
    /// Semantic participant digest.
    pub(crate) digest: String,
}
/// A native API together with its canonical forms and derived routing subjects.
#[derive(Debug, Clone)]
pub(crate) struct ApiInput {
    #[doc = concat!("The `", stringify!(path), "` contract value.")]
    pub(crate) path: PathBuf,
    #[doc = concat!("The `", stringify!(value), "` contract value.")]
    pub(crate) value: Value,
    /// Parsed protocol-owned native API artifact.
    pub(crate) api: trellis_protocol::ApiArtifact,
    /// Exact deserialization of the API-only rendering fields.
    pub(crate) render_model: ApiProjection,
    /// Derived subjects keyed by API surface.
    pub(crate) subjects: trellis_protocol::DerivedApiSubjects,
    #[doc = concat!("The `", stringify!(canonical), "` contract value.")]
    pub(crate) canonical: String,
    #[doc = concat!("The `", stringify!(digest), "` contract value.")]
    pub(crate) digest: String,
}
