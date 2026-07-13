//! Shared request and response types for `trellis.core@v1`.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
/// Generated schema type `TrellisBindingsGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetRequest {
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
    /// The `digest` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesEventConsumersValueOrdering`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisBindingsGetResponseBindingResourcesEventConsumersValueOrdering {
    /// The `strict` wire value.
    #[serde(rename = "strict")]
    Strict,
    /// The `parallel` wire value.
    #[serde(rename = "parallel")]
    Parallel,
}
impl TrellisBindingsGetResponseBindingResourcesEventConsumersValueOrdering {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Parallel => "parallel",
        }
    }
}
impl AsRef<str> for TrellisBindingsGetResponseBindingResourcesEventConsumersValueOrdering {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisBindingsGetResponseBindingResourcesEventConsumersValueOrdering {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisBindingsGetResponseBindingResourcesEventConsumersValueOrdering {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisBindingsGetResponseBindingResourcesEventConsumersValueOrdering> for &str {
    fn eq(
        &self,
        other: &TrellisBindingsGetResponseBindingResourcesEventConsumersValueOrdering,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesEventConsumersValueReplay`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisBindingsGetResponseBindingResourcesEventConsumersValueReplay {
    /// The `new` wire value.
    #[serde(rename = "new")]
    New,
    /// The `all` wire value.
    #[serde(rename = "all")]
    All,
}
impl TrellisBindingsGetResponseBindingResourcesEventConsumersValueReplay {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::All => "all",
        }
    }
}
impl AsRef<str> for TrellisBindingsGetResponseBindingResourcesEventConsumersValueReplay {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisBindingsGetResponseBindingResourcesEventConsumersValueReplay {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisBindingsGetResponseBindingResourcesEventConsumersValueReplay {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisBindingsGetResponseBindingResourcesEventConsumersValueReplay> for &str {
    fn eq(
        &self,
        other: &TrellisBindingsGetResponseBindingResourcesEventConsumersValueReplay,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesEventConsumersValue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBindingResourcesEventConsumersValue {
    /// The `ackWaitMs` wire field.
    #[serde(rename = "ackWaitMs")]
    pub ack_wait_ms: i64,
    /// The `backoffMs` wire field.
    #[serde(rename = "backoffMs")]
    pub backoff_ms: Vec<i64>,
    /// The `consumerName` wire field.
    #[serde(rename = "consumerName")]
    pub consumer_name: String,
    /// The `filterSubjects` wire field.
    #[serde(rename = "filterSubjects")]
    pub filter_subjects: Vec<String>,
    /// The `maxDeliver` wire field.
    #[serde(rename = "maxDeliver")]
    pub max_deliver: i64,
    /// The `ordering` wire field.
    pub ordering: TrellisBindingsGetResponseBindingResourcesEventConsumersValueOrdering,
    /// The `replay` wire field.
    pub replay: TrellisBindingsGetResponseBindingResourcesEventConsumersValueReplay,
    /// The `stream` wire field.
    pub stream: String,
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrencyStalePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrencyStalePolicy {
    /// The `fail-stale` wire value.
    #[serde(rename = "fail-stale")]
    FailStale,
    /// The `block` wire value.
    #[serde(rename = "block")]
    Block,
}
impl TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrencyStalePolicy {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::FailStale => "fail-stale",
            Self::Block => "block",
        }
    }
}
impl AsRef<str>
    for TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrencyStalePolicy
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrencyStalePolicy
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrencyStalePolicy
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrencyStalePolicy>
    for &str
{
    fn eq(
        &self,
        other: &TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrencyStalePolicy,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrency {
    /// The `heartbeatIntervalMs` wire field.
    #[serde(rename = "heartbeatIntervalMs")]
    pub heartbeat_interval_ms: i64,
    /// The `heartbeatTtlMs` wire field.
    #[serde(rename = "heartbeatTtlMs")]
    pub heartbeat_ttl_ms: i64,
    /// The `key` wire field.
    pub key: Vec<String>,
    /// The `maxActive` wire field.
    #[serde(rename = "maxActive")]
    pub max_active: i64,
    /// The `stalePolicy` wire field.
    #[serde(rename = "stalePolicy")]
    pub stale_policy:
        TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrencyStalePolicy,
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesJobsQueuesValuePayload`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBindingResourcesJobsQueuesValuePayload {
    /// The `schema` wire field.
    pub schema: String,
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueueWhenFull`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueueWhenFull {
    /// The `reject` wire value.
    #[serde(rename = "reject")]
    Reject,
    /// The `coalesce` wire value.
    #[serde(rename = "coalesce")]
    Coalesce,
    /// The `replace-oldest` wire value.
    #[serde(rename = "replace-oldest")]
    ReplaceOldest,
}
impl TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueueWhenFull {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::Coalesce => "coalesce",
            Self::ReplaceOldest => "replace-oldest",
        }
    }
}
impl AsRef<str> for TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueueWhenFull {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueueWhenFull {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueueWhenFull {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueueWhenFull> for &str {
    fn eq(
        &self,
        other: &TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueueWhenFull,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueue {
    /// The `maxQueuedPerKey` wire field.
    #[serde(rename = "maxQueuedPerKey")]
    pub max_queued_per_key: i64,
    /// The `whenFull` wire field.
    #[serde(rename = "whenFull")]
    pub when_full: TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueueWhenFull,
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesJobsQueuesValueResult`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBindingResourcesJobsQueuesValueResult {
    /// The `schema` wire field.
    pub schema: String,
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesJobsQueuesValueUpdate`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBindingResourcesJobsQueuesValueUpdate {
    /// The `schema` wire field.
    pub schema: String,
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesJobsQueuesValue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBindingResourcesJobsQueuesValue {
    /// The `ackWaitMs` wire field.
    #[serde(rename = "ackWaitMs")]
    pub ack_wait_ms: i64,
    /// The `backoffMs` wire field.
    #[serde(rename = "backoffMs")]
    pub backoff_ms: Vec<i64>,
    /// The `consumerName` wire field.
    #[serde(rename = "consumerName")]
    pub consumer_name: String,
    /// The `defaultDeadlineMs` wire field.
    #[serde(rename = "defaultDeadlineMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_deadline_ms: Option<i64>,
    /// The `dlq` wire field.
    pub dlq: bool,
    /// The `keyConcurrency` wire field.
    #[serde(rename = "keyConcurrency")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_concurrency:
        Option<TrellisBindingsGetResponseBindingResourcesJobsQueuesValueKeyConcurrency>,
    /// The `logs` wire field.
    pub logs: bool,
    /// The `maxDeliver` wire field.
    #[serde(rename = "maxDeliver")]
    pub max_deliver: i64,
    /// The `payload` wire field.
    pub payload: TrellisBindingsGetResponseBindingResourcesJobsQueuesValuePayload,
    /// The `progress` wire field.
    pub progress: bool,
    /// The `publishPrefix` wire field.
    #[serde(rename = "publishPrefix")]
    pub publish_prefix: String,
    /// The `queue` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<TrellisBindingsGetResponseBindingResourcesJobsQueuesValueQueue>,
    /// The `queueType` wire field.
    #[serde(rename = "queueType")]
    pub queue_type: String,
    /// The `result` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<TrellisBindingsGetResponseBindingResourcesJobsQueuesValueResult>,
    /// The `update` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<TrellisBindingsGetResponseBindingResourcesJobsQueuesValueUpdate>,
    /// The `updatesPrefix` wire field.
    #[serde(rename = "updatesPrefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updates_prefix: Option<String>,
    /// The `workSubject` wire field.
    #[serde(rename = "workSubject")]
    pub work_subject: String,
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesJobs`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBindingResourcesJobs {
    /// The `namespace` wire field.
    pub namespace: String,
    /// The `queues` wire field.
    pub queues: BTreeMap<String, TrellisBindingsGetResponseBindingResourcesJobsQueuesValue>,
    /// The `serviceName` wire field.
    #[serde(rename = "serviceName")]
    pub service_name: String,
    /// The `workStream` wire field.
    #[serde(rename = "workStream")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_stream: Option<String>,
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesKvValue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBindingResourcesKvValue {
    /// The `bucket` wire field.
    pub bucket: String,
    /// The `history` wire field.
    pub history: i64,
    /// The `maxValueBytes` wire field.
    #[serde(rename = "maxValueBytes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value_bytes: Option<i64>,
    /// The `ttlMs` wire field.
    #[serde(rename = "ttlMs")]
    pub ttl_ms: i64,
}
/// Generated schema type `TrellisBindingsGetResponseBindingResourcesStoreValue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBindingResourcesStoreValue {
    /// The `maxObjectBytes` wire field.
    #[serde(rename = "maxObjectBytes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_object_bytes: Option<i64>,
    /// The `maxTotalBytes` wire field.
    #[serde(rename = "maxTotalBytes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_total_bytes: Option<i64>,
    /// The `name` wire field.
    pub name: String,
    /// The `ttlMs` wire field.
    #[serde(rename = "ttlMs")]
    pub ttl_ms: i64,
}
/// Generated schema type `TrellisBindingsGetResponseBindingResources`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBindingResources {
    /// The `eventConsumers` wire field.
    #[serde(rename = "eventConsumers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_consumers:
        Option<BTreeMap<String, TrellisBindingsGetResponseBindingResourcesEventConsumersValue>>,
    /// The `jobs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jobs: Option<TrellisBindingsGetResponseBindingResourcesJobs>,
    /// The `kv` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kv: Option<BTreeMap<String, TrellisBindingsGetResponseBindingResourcesKvValue>>,
    /// The `store` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<BTreeMap<String, TrellisBindingsGetResponseBindingResourcesStoreValue>>,
}
/// Generated schema type `TrellisBindingsGetResponseBinding`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseBinding {
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `digest` wire field.
    pub digest: String,
    /// The `resources` wire field.
    pub resources: TrellisBindingsGetResponseBindingResources,
}
/// Generated schema type `TrellisBindingsGetResponseEventConsumersValueOrdering`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisBindingsGetResponseEventConsumersValueOrdering {
    /// The `strict` wire value.
    #[serde(rename = "strict")]
    Strict,
    /// The `parallel` wire value.
    #[serde(rename = "parallel")]
    Parallel,
}
impl TrellisBindingsGetResponseEventConsumersValueOrdering {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Parallel => "parallel",
        }
    }
}
impl AsRef<str> for TrellisBindingsGetResponseEventConsumersValueOrdering {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisBindingsGetResponseEventConsumersValueOrdering {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisBindingsGetResponseEventConsumersValueOrdering {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisBindingsGetResponseEventConsumersValueOrdering> for &str {
    fn eq(&self, other: &TrellisBindingsGetResponseEventConsumersValueOrdering) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisBindingsGetResponseEventConsumersValueReplay`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisBindingsGetResponseEventConsumersValueReplay {
    /// The `new` wire value.
    #[serde(rename = "new")]
    New,
    /// The `all` wire value.
    #[serde(rename = "all")]
    All,
}
impl TrellisBindingsGetResponseEventConsumersValueReplay {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::All => "all",
        }
    }
}
impl AsRef<str> for TrellisBindingsGetResponseEventConsumersValueReplay {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisBindingsGetResponseEventConsumersValueReplay {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisBindingsGetResponseEventConsumersValueReplay {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisBindingsGetResponseEventConsumersValueReplay> for &str {
    fn eq(&self, other: &TrellisBindingsGetResponseEventConsumersValueReplay) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisBindingsGetResponseEventConsumersValue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponseEventConsumersValue {
    /// The `ackWaitMs` wire field.
    #[serde(rename = "ackWaitMs")]
    pub ack_wait_ms: i64,
    /// The `backoffMs` wire field.
    #[serde(rename = "backoffMs")]
    pub backoff_ms: Vec<i64>,
    /// The `consumerName` wire field.
    #[serde(rename = "consumerName")]
    pub consumer_name: String,
    /// The `filterSubjects` wire field.
    #[serde(rename = "filterSubjects")]
    pub filter_subjects: Vec<String>,
    /// The `maxDeliver` wire field.
    #[serde(rename = "maxDeliver")]
    pub max_deliver: i64,
    /// The `ordering` wire field.
    pub ordering: TrellisBindingsGetResponseEventConsumersValueOrdering,
    /// The `replay` wire field.
    pub replay: TrellisBindingsGetResponseEventConsumersValueReplay,
    /// The `stream` wire field.
    pub stream: String,
}
/// Generated schema type `TrellisBindingsGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisBindingsGetResponse {
    /// The `binding` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding: Option<TrellisBindingsGetResponseBinding>,
    /// The `eventConsumers` wire field.
    #[serde(rename = "eventConsumers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_consumers: Option<BTreeMap<String, TrellisBindingsGetResponseEventConsumersValue>>,
}
/// Generated schema type `TrellisCatalogResponseCatalogContractsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisCatalogResponseCatalogContractsItem {
    /// The `description` wire field.
    pub description: String,
    /// The `digest` wire field.
    pub digest: String,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `id` wire field.
    pub id: String,
}
/// Generated schema type `TrellisCatalogResponseCatalogFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisCatalogResponseCatalogFormat {
    /// The `trellis.catalog.v1` wire value.
    #[serde(rename = "trellis.catalog.v1")]
    TrellisCatalogV1,
}
impl TrellisCatalogResponseCatalogFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisCatalogV1 => "trellis.catalog.v1",
        }
    }
}
impl AsRef<str> for TrellisCatalogResponseCatalogFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisCatalogResponseCatalogFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisCatalogResponseCatalogFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisCatalogResponseCatalogFormat> for &str {
    fn eq(&self, other: &TrellisCatalogResponseCatalogFormat) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisCatalogResponseCatalogIssuesItemActionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisCatalogResponseCatalogIssuesItemActionsItemAction {
    /// The `keep-current` wire value.
    #[serde(rename = "keep-current")]
    KeepCurrent,
    /// The `force-replace` wire value.
    #[serde(rename = "force-replace")]
    ForceReplace,
}
impl TrellisCatalogResponseCatalogIssuesItemActionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::KeepCurrent => "keep-current",
            Self::ForceReplace => "force-replace",
        }
    }
}
impl AsRef<str> for TrellisCatalogResponseCatalogIssuesItemActionsItemAction {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisCatalogResponseCatalogIssuesItemActionsItemAction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisCatalogResponseCatalogIssuesItemActionsItemAction {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisCatalogResponseCatalogIssuesItemActionsItemAction> for &str {
    fn eq(&self, other: &TrellisCatalogResponseCatalogIssuesItemActionsItemAction) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisCatalogResponseCatalogIssuesItemActionsItemRisk`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisCatalogResponseCatalogIssuesItemActionsItemRisk {
    /// The `recommended` wire value.
    #[serde(rename = "recommended")]
    Recommended,
    /// The `dangerous` wire value.
    #[serde(rename = "dangerous")]
    Dangerous,
}
impl TrellisCatalogResponseCatalogIssuesItemActionsItemRisk {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Recommended => "recommended",
            Self::Dangerous => "dangerous",
        }
    }
}
impl AsRef<str> for TrellisCatalogResponseCatalogIssuesItemActionsItemRisk {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisCatalogResponseCatalogIssuesItemActionsItemRisk {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisCatalogResponseCatalogIssuesItemActionsItemRisk {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisCatalogResponseCatalogIssuesItemActionsItemRisk> for &str {
    fn eq(&self, other: &TrellisCatalogResponseCatalogIssuesItemActionsItemRisk) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisCatalogResponseCatalogIssuesItemActionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisCatalogResponseCatalogIssuesItemActionsItem {
    /// The `action` wire field.
    pub action: TrellisCatalogResponseCatalogIssuesItemActionsItemAction,
    /// The `deploymentIds` wire field.
    #[serde(rename = "deploymentIds")]
    pub deployment_ids: Vec<String>,
    /// The `description` wire field.
    pub description: String,
    /// The `digests` wire field.
    pub digests: Vec<String>,
    /// The `label` wire field.
    pub label: String,
    /// The `risk` wire field.
    pub risk: TrellisCatalogResponseCatalogIssuesItemActionsItemRisk,
}
/// Generated schema type `TrellisCatalogResponseCatalogIssuesItemKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisCatalogResponseCatalogIssuesItemKind {
    /// The `missing-active-contract` wire value.
    #[serde(rename = "missing-active-contract")]
    MissingActiveContract,
    /// The `invalid-active-contract` wire value.
    #[serde(rename = "invalid-active-contract")]
    InvalidActiveContract,
    /// The `incompatible-active-contract` wire value.
    #[serde(rename = "incompatible-active-contract")]
    IncompatibleActiveContract,
    /// The `invalid-active-contract-uses` wire value.
    #[serde(rename = "invalid-active-contract-uses")]
    InvalidActiveContractUses,
}
impl TrellisCatalogResponseCatalogIssuesItemKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MissingActiveContract => "missing-active-contract",
            Self::InvalidActiveContract => "invalid-active-contract",
            Self::IncompatibleActiveContract => "incompatible-active-contract",
            Self::InvalidActiveContractUses => "invalid-active-contract-uses",
        }
    }
}
impl AsRef<str> for TrellisCatalogResponseCatalogIssuesItemKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisCatalogResponseCatalogIssuesItemKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisCatalogResponseCatalogIssuesItemKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisCatalogResponseCatalogIssuesItemKind> for &str {
    fn eq(&self, other: &TrellisCatalogResponseCatalogIssuesItemKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisCatalogResponseCatalogIssuesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisCatalogResponseCatalogIssuesItem {
    /// The `actions` wire field.
    pub actions: Vec<TrellisCatalogResponseCatalogIssuesItemActionsItem>,
    /// The `conflictingDeploymentIds` wire field.
    #[serde(rename = "conflictingDeploymentIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicting_deployment_ids: Option<Vec<String>>,
    /// The `conflictingDigest` wire field.
    #[serde(rename = "conflictingDigest")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicting_digest: Option<String>,
    /// The `conflictingDigests` wire field.
    #[serde(rename = "conflictingDigests")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicting_digests: Option<Vec<String>>,
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
    /// The `deploymentIds` wire field.
    #[serde(rename = "deploymentIds")]
    pub deployment_ids: Vec<String>,
    /// The `digest` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    /// The `effectiveDeploymentIds` wire field.
    #[serde(rename = "effectiveDeploymentIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_deployment_ids: Option<Vec<String>>,
    /// The `effectiveDigests` wire field.
    #[serde(rename = "effectiveDigests")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_digests: Option<Vec<String>>,
    /// The `issueId` wire field.
    #[serde(rename = "issueId")]
    pub issue_id: String,
    /// The `kind` wire field.
    pub kind: TrellisCatalogResponseCatalogIssuesItemKind,
    /// The `message` wire field.
    pub message: String,
}
/// Generated schema type `TrellisCatalogResponseCatalog`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisCatalogResponseCatalog {
    /// The `contracts` wire field.
    pub contracts: Vec<TrellisCatalogResponseCatalogContractsItem>,
    /// The `format` wire field.
    pub format: TrellisCatalogResponseCatalogFormat,
    /// The `issues` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<Vec<TrellisCatalogResponseCatalogIssuesItem>>,
}
/// Generated schema type `TrellisCatalogResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisCatalogResponse {
    /// The `catalog` wire field.
    pub catalog: TrellisCatalogResponseCatalog,
}
/// Generated schema type `TrellisContractGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetRequest {
    /// The `digest` wire field.
    pub digest: String,
}
/// Generated schema type `TrellisContractGetResponseContractDocs`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractDocs {
    /// The `markdown` wire field.
    pub markdown: String,
    /// The `summary` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `TrellisContractGetResponseContractExports`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractExports {
    /// The `schemas` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<Vec<String>>,
}
/// Generated schema type `TrellisContractGetResponseContractFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisContractGetResponseContractFormat {
    /// The `trellis.contract.v1` wire value.
    #[serde(rename = "trellis.contract.v1")]
    TrellisContractV1,
}
impl TrellisContractGetResponseContractFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisContractV1 => "trellis.contract.v1",
        }
    }
}
impl AsRef<str> for TrellisContractGetResponseContractFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisContractGetResponseContractFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisContractGetResponseContractFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisContractGetResponseContractFormat> for &str {
    fn eq(&self, other: &TrellisContractGetResponseContractFormat) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisContractGetResponseContractJobsValueDocs`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractJobsValueDocs {
    /// The `markdown` wire field.
    pub markdown: String,
    /// The `summary` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `TrellisContractGetResponseContractJobsValueKeyConcurrencyStalePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisContractGetResponseContractJobsValueKeyConcurrencyStalePolicy {
    /// The `fail-stale` wire value.
    #[serde(rename = "fail-stale")]
    FailStale,
    /// The `block` wire value.
    #[serde(rename = "block")]
    Block,
}
impl TrellisContractGetResponseContractJobsValueKeyConcurrencyStalePolicy {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::FailStale => "fail-stale",
            Self::Block => "block",
        }
    }
}
impl AsRef<str> for TrellisContractGetResponseContractJobsValueKeyConcurrencyStalePolicy {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisContractGetResponseContractJobsValueKeyConcurrencyStalePolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisContractGetResponseContractJobsValueKeyConcurrencyStalePolicy {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisContractGetResponseContractJobsValueKeyConcurrencyStalePolicy> for &str {
    fn eq(
        &self,
        other: &TrellisContractGetResponseContractJobsValueKeyConcurrencyStalePolicy,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisContractGetResponseContractJobsValueKeyConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractJobsValueKeyConcurrency {
    /// The `heartbeatIntervalMs` wire field.
    #[serde(rename = "heartbeatIntervalMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_interval_ms: Option<i64>,
    /// The `heartbeatTtlMs` wire field.
    #[serde(rename = "heartbeatTtlMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ttl_ms: Option<i64>,
    /// The `key` wire field.
    pub key: Vec<String>,
    /// The `maxActive` wire field.
    #[serde(rename = "maxActive")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_active: Option<i64>,
    /// The `stalePolicy` wire field.
    #[serde(rename = "stalePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_policy: Option<TrellisContractGetResponseContractJobsValueKeyConcurrencyStalePolicy>,
}
/// Generated schema type `TrellisContractGetResponseContractJobsValuePayload`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractJobsValuePayload {
    /// The `schema` wire field.
    pub schema: String,
}
/// Generated schema type `TrellisContractGetResponseContractJobsValueQueueWhenFull`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisContractGetResponseContractJobsValueQueueWhenFull {
    /// The `reject` wire value.
    #[serde(rename = "reject")]
    Reject,
    /// The `coalesce` wire value.
    #[serde(rename = "coalesce")]
    Coalesce,
    /// The `replace-oldest` wire value.
    #[serde(rename = "replace-oldest")]
    ReplaceOldest,
}
impl TrellisContractGetResponseContractJobsValueQueueWhenFull {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::Coalesce => "coalesce",
            Self::ReplaceOldest => "replace-oldest",
        }
    }
}
impl AsRef<str> for TrellisContractGetResponseContractJobsValueQueueWhenFull {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisContractGetResponseContractJobsValueQueueWhenFull {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisContractGetResponseContractJobsValueQueueWhenFull {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisContractGetResponseContractJobsValueQueueWhenFull> for &str {
    fn eq(&self, other: &TrellisContractGetResponseContractJobsValueQueueWhenFull) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisContractGetResponseContractJobsValueQueue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractJobsValueQueue {
    /// The `maxQueuedPerKey` wire field.
    #[serde(rename = "maxQueuedPerKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_queued_per_key: Option<i64>,
    /// The `whenFull` wire field.
    #[serde(rename = "whenFull")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when_full: Option<TrellisContractGetResponseContractJobsValueQueueWhenFull>,
}
/// Generated schema type `TrellisContractGetResponseContractJobsValueResult`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractJobsValueResult {
    /// The `schema` wire field.
    pub schema: String,
}
/// Generated schema type `TrellisContractGetResponseContractJobsValueUpdate`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractJobsValueUpdate {
    /// The `schema` wire field.
    pub schema: String,
}
/// Generated schema type `TrellisContractGetResponseContractJobsValue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractJobsValue {
    /// The `ackWaitMs` wire field.
    #[serde(rename = "ackWaitMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack_wait_ms: Option<i64>,
    /// The `backoffMs` wire field.
    #[serde(rename = "backoffMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_ms: Option<Vec<i64>>,
    /// The `defaultDeadlineMs` wire field.
    #[serde(rename = "defaultDeadlineMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_deadline_ms: Option<i64>,
    /// The `dlq` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dlq: Option<bool>,
    /// The `docs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<TrellisContractGetResponseContractJobsValueDocs>,
    /// The `keyConcurrency` wire field.
    #[serde(rename = "keyConcurrency")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_concurrency: Option<TrellisContractGetResponseContractJobsValueKeyConcurrency>,
    /// The `logs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<bool>,
    /// The `maxDeliver` wire field.
    #[serde(rename = "maxDeliver")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_deliver: Option<i64>,
    /// The `payload` wire field.
    pub payload: TrellisContractGetResponseContractJobsValuePayload,
    /// The `progress` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<bool>,
    /// The `queue` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<TrellisContractGetResponseContractJobsValueQueue>,
    /// The `result` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<TrellisContractGetResponseContractJobsValueResult>,
    /// The `update` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<TrellisContractGetResponseContractJobsValueUpdate>,
}
/// Generated schema type `TrellisContractGetResponseContractKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisContractGetResponseContractKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `app` wire value.
    #[serde(rename = "app")]
    App,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
    /// The `agent` wire value.
    #[serde(rename = "agent")]
    Agent,
}
impl TrellisContractGetResponseContractKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::App => "app",
            Self::Device => "device",
            Self::Agent => "agent",
        }
    }
}
impl AsRef<str> for TrellisContractGetResponseContractKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisContractGetResponseContractKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisContractGetResponseContractKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisContractGetResponseContractKind> for &str {
    fn eq(&self, other: &TrellisContractGetResponseContractKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisContractGetResponseContractResourcesKvValueDocs`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractResourcesKvValueDocs {
    /// The `markdown` wire field.
    pub markdown: String,
    /// The `summary` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `TrellisContractGetResponseContractResourcesKvValueSchema`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractResourcesKvValueSchema {
    /// The `schema` wire field.
    pub schema: String,
}
/// Generated schema type `TrellisContractGetResponseContractResourcesKvValue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractResourcesKvValue {
    /// The `docs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<TrellisContractGetResponseContractResourcesKvValueDocs>,
    /// The `history` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<i64>,
    /// The `maxValueBytes` wire field.
    #[serde(rename = "maxValueBytes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value_bytes: Option<i64>,
    /// The `purpose` wire field.
    pub purpose: String,
    /// The `required` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// The `schema` wire field.
    pub schema: TrellisContractGetResponseContractResourcesKvValueSchema,
    /// The `ttlMs` wire field.
    #[serde(rename = "ttlMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_ms: Option<i64>,
}
/// Generated schema type `TrellisContractGetResponseContractResourcesStoreValueDocs`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractResourcesStoreValueDocs {
    /// The `markdown` wire field.
    pub markdown: String,
    /// The `summary` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `TrellisContractGetResponseContractResourcesStoreValue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractResourcesStoreValue {
    /// The `docs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<TrellisContractGetResponseContractResourcesStoreValueDocs>,
    /// The `maxObjectBytes` wire field.
    #[serde(rename = "maxObjectBytes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_object_bytes: Option<i64>,
    /// The `maxTotalBytes` wire field.
    #[serde(rename = "maxTotalBytes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_total_bytes: Option<i64>,
    /// The `purpose` wire field.
    pub purpose: String,
    /// The `required` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// The `ttlMs` wire field.
    #[serde(rename = "ttlMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_ms: Option<i64>,
}
/// Generated schema type `TrellisContractGetResponseContractResources`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractResources {
    /// The `kv` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kv: Option<BTreeMap<String, TrellisContractGetResponseContractResourcesKvValue>>,
    /// The `store` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<BTreeMap<String, TrellisContractGetResponseContractResourcesStoreValue>>,
}
/// Generated schema type `TrellisContractGetResponseContractStateValueAcceptedVersionsValue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractStateValueAcceptedVersionsValue {
    /// The `schema` wire field.
    pub schema: String,
}
/// Generated schema type `TrellisContractGetResponseContractStateValueDocs`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractStateValueDocs {
    /// The `markdown` wire field.
    pub markdown: String,
    /// The `summary` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `TrellisContractGetResponseContractStateValueKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisContractGetResponseContractStateValueKind {
    /// The `value` wire value.
    #[serde(rename = "value")]
    Value,
    /// The `map` wire value.
    #[serde(rename = "map")]
    Map,
}
impl TrellisContractGetResponseContractStateValueKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Map => "map",
        }
    }
}
impl AsRef<str> for TrellisContractGetResponseContractStateValueKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisContractGetResponseContractStateValueKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisContractGetResponseContractStateValueKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisContractGetResponseContractStateValueKind> for &str {
    fn eq(&self, other: &TrellisContractGetResponseContractStateValueKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisContractGetResponseContractStateValueSchema`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractStateValueSchema {
    /// The `schema` wire field.
    pub schema: String,
}
/// Generated schema type `TrellisContractGetResponseContractStateValue`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContractStateValue {
    /// The `acceptedVersions` wire field.
    #[serde(rename = "acceptedVersions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_versions:
        Option<BTreeMap<String, TrellisContractGetResponseContractStateValueAcceptedVersionsValue>>,
    /// The `docs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<TrellisContractGetResponseContractStateValueDocs>,
    /// The `kind` wire field.
    pub kind: TrellisContractGetResponseContractStateValueKind,
    /// The `schema` wire field.
    pub schema: TrellisContractGetResponseContractStateValueSchema,
    /// The `stateVersion` wire field.
    #[serde(rename = "stateVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_version: Option<String>,
}
/// Generated schema type `TrellisContractGetResponseContract`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponseContract {
    /// The `description` wire field.
    pub description: String,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `docs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<TrellisContractGetResponseContractDocs>,
    /// The `errors` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<BTreeMap<String, BTreeMap<String, Value>>>,
    /// The `events` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<BTreeMap<String, BTreeMap<String, Value>>>,
    /// The `exports` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exports: Option<TrellisContractGetResponseContractExports>,
    /// The `format` wire field.
    pub format: TrellisContractGetResponseContractFormat,
    /// The `id` wire field.
    pub id: String,
    /// The `jobs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jobs: Option<BTreeMap<String, TrellisContractGetResponseContractJobsValue>>,
    /// The `kind` wire field.
    pub kind: TrellisContractGetResponseContractKind,
    /// The `operations` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<BTreeMap<String, BTreeMap<String, Value>>>,
    /// The `resources` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<TrellisContractGetResponseContractResources>,
    /// The `rpc` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc: Option<BTreeMap<String, BTreeMap<String, Value>>>,
    /// The `schemas` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<BTreeMap<String, Value>>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<BTreeMap<String, TrellisContractGetResponseContractStateValue>>,
    /// The `uses` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<BTreeMap<String, BTreeMap<String, Value>>>,
}
/// Generated schema type `TrellisContractGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisContractGetResponse {
    /// The `contract` wire field.
    pub contract: TrellisContractGetResponseContract,
}
/// Generated schema type `TrellisSurfaceStatusRequestAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisSurfaceStatusRequestAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
}
impl TrellisSurfaceStatusRequestAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Observe => "observe",
        }
    }
}
impl AsRef<str> for TrellisSurfaceStatusRequestAction {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisSurfaceStatusRequestAction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisSurfaceStatusRequestAction {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisSurfaceStatusRequestAction> for &str {
    fn eq(&self, other: &TrellisSurfaceStatusRequestAction) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisSurfaceStatusRequestKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisSurfaceStatusRequestKind {
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
}
impl TrellisSurfaceStatusRequestKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
        }
    }
}
impl AsRef<str> for TrellisSurfaceStatusRequestKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisSurfaceStatusRequestKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisSurfaceStatusRequestKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisSurfaceStatusRequestKind> for &str {
    fn eq(&self, other: &TrellisSurfaceStatusRequestKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisSurfaceStatusRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisSurfaceStatusRequest {
    /// The `action` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<TrellisSurfaceStatusRequestAction>,
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `kind` wire field.
    pub kind: TrellisSurfaceStatusRequestKind,
    /// The `surface` wire field.
    pub surface: String,
}
/// Generated schema type `TrellisSurfaceStatusResponseStatusAvailableRuntime`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisSurfaceStatusResponseStatusAvailableRuntime {
    /// The `live` wire value.
    #[serde(rename = "live")]
    Live,
    /// The `no_live_implementer` wire value.
    #[serde(rename = "no_live_implementer")]
    NoLiveImplementer,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
}
impl TrellisSurfaceStatusResponseStatusAvailableRuntime {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::NoLiveImplementer => "no_live_implementer",
            Self::Disabled => "disabled",
        }
    }
}
impl AsRef<str> for TrellisSurfaceStatusResponseStatusAvailableRuntime {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisSurfaceStatusResponseStatusAvailableRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisSurfaceStatusResponseStatusAvailableRuntime {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisSurfaceStatusResponseStatusAvailableRuntime> for &str {
    fn eq(&self, other: &TrellisSurfaceStatusResponseStatusAvailableRuntime) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisSurfaceStatusResponseStatusUnavailableReason`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisSurfaceStatusResponseStatusUnavailableReason {
    /// The `authority_unavailable` wire value.
    #[serde(rename = "authority_unavailable")]
    AuthorityUnavailable,
}
impl TrellisSurfaceStatusResponseStatusUnavailableReason {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::AuthorityUnavailable => "authority_unavailable",
        }
    }
}
impl AsRef<str> for TrellisSurfaceStatusResponseStatusUnavailableReason {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisSurfaceStatusResponseStatusUnavailableReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisSurfaceStatusResponseStatusUnavailableReason {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisSurfaceStatusResponseStatusUnavailableReason> for &str {
    fn eq(&self, other: &TrellisSurfaceStatusResponseStatusUnavailableReason) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisSurfaceStatusResponseStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state")]
pub enum TrellisSurfaceStatusResponseStatus {
    /// The `available` variant.
    #[serde(rename = "available")]
    Available {
        /// The `liveImplementer` wire field.
        #[serde(rename = "liveImplementer")]
        live_implementer: bool,
        /// The `runtime` wire field.
        runtime: TrellisSurfaceStatusResponseStatusAvailableRuntime,
    },
    /// The `unavailable` variant.
    #[serde(rename = "unavailable")]
    Unavailable {
        /// The `reason` wire field.
        reason: TrellisSurfaceStatusResponseStatusUnavailableReason,
    },
    /// The `unauthorized` variant.
    #[serde(rename = "unauthorized")]
    Unauthorized {
        /// The `missingCapabilities` wire field.
        #[serde(rename = "missingCapabilities")]
        missing_capabilities: Vec<String>,
    },
    /// The `unknown_contract` variant.
    #[serde(rename = "unknown_contract")]
    UnknownContract {
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
    },
    /// The `unknown_surface` variant.
    #[serde(rename = "unknown_surface")]
    UnknownSurface {
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `kind` wire field.
        kind: String,
        /// The `surface` wire field.
        surface: String,
    },
}
/// Generated schema type `TrellisSurfaceStatusResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisSurfaceStatusResponse {
    /// The `status` wire field.
    pub status: TrellisSurfaceStatusResponseStatus,
}
