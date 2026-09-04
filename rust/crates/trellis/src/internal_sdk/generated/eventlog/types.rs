//! Shared request and response types for `trellis.eventlog@v1`.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
/// Generated schema type `EventLogConsumersInspectRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogConsumersInspectRequest {
    /// The `consumerName` wire field.
    #[serde(rename = "consumerName")]
    pub consumer_name: String,
    /// The `stream` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
}
/// Generated schema type `EventLogConsumersQueryRequestStatusItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventLogConsumersQueryRequestStatusItem {
    /// The `current` wire value.
    #[serde(rename = "current")]
    Current,
    /// The `processing` wire value.
    #[serde(rename = "processing")]
    Processing,
    /// The `behind` wire value.
    #[serde(rename = "behind")]
    Behind,
    /// The `saturated` wire value.
    #[serde(rename = "saturated")]
    Saturated,
    /// The `inactive` wire value.
    #[serde(rename = "inactive")]
    Inactive,
    /// The `failing` wire value.
    #[serde(rename = "failing")]
    Failing,
    /// The `missing` wire value.
    #[serde(rename = "missing")]
    Missing,
    /// The `orphaned` wire value.
    #[serde(rename = "orphaned")]
    Orphaned,
}
impl EventLogConsumersQueryRequestStatusItem {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Processing => "processing",
            Self::Behind => "behind",
            Self::Saturated => "saturated",
            Self::Inactive => "inactive",
            Self::Failing => "failing",
            Self::Missing => "missing",
            Self::Orphaned => "orphaned",
        }
    }
}
impl AsRef<str> for EventLogConsumersQueryRequestStatusItem {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for EventLogConsumersQueryRequestStatusItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for EventLogConsumersQueryRequestStatusItem {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<EventLogConsumersQueryRequestStatusItem> for &str {
    fn eq(&self, other: &EventLogConsumersQueryRequestStatusItem) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `EventLogConsumersQueryRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogConsumersQueryRequest {
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// The `ownerContractId` wire field.
    #[serde(rename = "ownerContractId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_contract_id: Option<String>,
    /// The `status` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<EventLogConsumersQueryRequestStatusItem>>,
    /// The `subject` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}
/// Generated schema type `EventLogConsumersQueryResponseConsumersItemManagedBy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventLogConsumersQueryResponseConsumersItemManagedBy {
    /// The `authority` wire value.
    #[serde(rename = "authority")]
    Authority,
    /// The `platform` wire value.
    #[serde(rename = "platform")]
    Platform,
    /// The `external` wire value.
    #[serde(rename = "external")]
    External,
}
impl EventLogConsumersQueryResponseConsumersItemManagedBy {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Authority => "authority",
            Self::Platform => "platform",
            Self::External => "external",
        }
    }
}
impl AsRef<str> for EventLogConsumersQueryResponseConsumersItemManagedBy {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for EventLogConsumersQueryResponseConsumersItemManagedBy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for EventLogConsumersQueryResponseConsumersItemManagedBy {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<EventLogConsumersQueryResponseConsumersItemManagedBy> for &str {
    fn eq(&self, other: &EventLogConsumersQueryResponseConsumersItemManagedBy) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `EventLogConsumersQueryResponseConsumersItemStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventLogConsumersQueryResponseConsumersItemStatus {
    /// The `current` wire value.
    #[serde(rename = "current")]
    Current,
    /// The `processing` wire value.
    #[serde(rename = "processing")]
    Processing,
    /// The `behind` wire value.
    #[serde(rename = "behind")]
    Behind,
    /// The `saturated` wire value.
    #[serde(rename = "saturated")]
    Saturated,
    /// The `inactive` wire value.
    #[serde(rename = "inactive")]
    Inactive,
    /// The `failing` wire value.
    #[serde(rename = "failing")]
    Failing,
    /// The `missing` wire value.
    #[serde(rename = "missing")]
    Missing,
    /// The `orphaned` wire value.
    #[serde(rename = "orphaned")]
    Orphaned,
}
impl EventLogConsumersQueryResponseConsumersItemStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Processing => "processing",
            Self::Behind => "behind",
            Self::Saturated => "saturated",
            Self::Inactive => "inactive",
            Self::Failing => "failing",
            Self::Missing => "missing",
            Self::Orphaned => "orphaned",
        }
    }
}
impl AsRef<str> for EventLogConsumersQueryResponseConsumersItemStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for EventLogConsumersQueryResponseConsumersItemStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for EventLogConsumersQueryResponseConsumersItemStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<EventLogConsumersQueryResponseConsumersItemStatus> for &str {
    fn eq(&self, other: &EventLogConsumersQueryResponseConsumersItemStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `EventLogConsumersQueryResponseConsumersItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogConsumersQueryResponseConsumersItem {
    /// The `ackPending` wire field.
    #[serde(rename = "ackPending")]
    pub ack_pending: i64,
    /// The `ackWaitMs` wire field.
    #[serde(rename = "ackWaitMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack_wait_ms: Option<i64>,
    /// The `consumerName` wire field.
    #[serde(rename = "consumerName")]
    pub consumer_name: String,
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
    /// The `filterSubjects` wire field.
    #[serde(rename = "filterSubjects")]
    pub filter_subjects: Vec<String>,
    /// The `group` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// The `managedBy` wire field.
    #[serde(rename = "managedBy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_by: Option<EventLogConsumersQueryResponseConsumersItemManagedBy>,
    /// The `maxDeliver` wire field.
    #[serde(rename = "maxDeliver")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_deliver: Option<i64>,
    /// The `oldestPendingAt` wire field.
    #[serde(rename = "oldestPendingAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_pending_at: Option<String>,
    /// The `oldestPendingEventId` wire field.
    #[serde(rename = "oldestPendingEventId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_pending_event_id: Option<String>,
    /// The `pending` wire field.
    pub pending: i64,
    /// The `redelivered` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redelivered: Option<i64>,
    /// The `status` wire field.
    pub status: EventLogConsumersQueryResponseConsumersItemStatus,
    /// The `stream` wire field.
    pub stream: String,
    /// The `waitingPulls` wire field.
    #[serde(rename = "waitingPulls")]
    pub waiting_pulls: i64,
}
/// Generated schema type `EventLogConsumersQueryResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogConsumersQueryResponse {
    /// The `consumers` wire field.
    pub consumers: Vec<EventLogConsumersQueryResponseConsumersItem>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    pub offset: i64,
    /// The `total` wire field.
    pub total: i64,
}
/// Generated schema type `EventLogInspectRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogInspectRequest {
    /// The `eventId` wire field.
    #[serde(rename = "eventId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The `streamSequence` wire field.
    #[serde(rename = "streamSequence")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_sequence: Option<i64>,
}
/// Generated schema type `EventLogMetricsRequestWindow`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventLogMetricsRequestWindow {
    /// The `15m` wire value.
    #[serde(rename = "15m")]
    V15m,
    /// The `1h` wire value.
    #[serde(rename = "1h")]
    V1h,
    /// The `6h` wire value.
    #[serde(rename = "6h")]
    V6h,
    /// The `24h` wire value.
    #[serde(rename = "24h")]
    V24h,
    /// The `7d` wire value.
    #[serde(rename = "7d")]
    V7d,
}
impl EventLogMetricsRequestWindow {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::V15m => "15m",
            Self::V1h => "1h",
            Self::V6h => "6h",
            Self::V24h => "24h",
            Self::V7d => "7d",
        }
    }
}
impl AsRef<str> for EventLogMetricsRequestWindow {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for EventLogMetricsRequestWindow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for EventLogMetricsRequestWindow {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<EventLogMetricsRequestWindow> for &str {
    fn eq(&self, other: &EventLogMetricsRequestWindow) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `EventLogMetricsRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsRequest {
    /// The `window` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<EventLogMetricsRequestWindow>,
}
/// Generated schema type `EventLogMetricsResponseBucketsItemByResolution`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseBucketsItemByResolution {
    /// The `malformed` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub malformed: Option<i64>,
    /// The `resolved` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<i64>,
    /// The `unresolved` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved: Option<i64>,
}
/// Generated schema type `EventLogMetricsResponseBucketsItemByVerificationStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseBucketsItemByVerificationStatus {
    /// The `auth-unavailable` wire field.
    #[serde(rename = "auth-unavailable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_unavailable: Option<i64>,
    /// The `invalid-signature` wire field.
    #[serde(rename = "invalid-signature")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_signature: Option<i64>,
    /// The `missing-proof` wire field.
    #[serde(rename = "missing-proof")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_proof: Option<i64>,
    /// The `missing-session` wire field.
    #[serde(rename = "missing-session")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_session: Option<i64>,
    /// The `outside-session-window` wire field.
    #[serde(rename = "outside-session-window")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outside_session_window: Option<i64>,
    /// The `subject-denied` wire field.
    #[serde(rename = "subject-denied")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_denied: Option<i64>,
    /// The `verified` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<i64>,
}
/// Generated schema type `EventLogMetricsResponseBucketsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseBucketsItem {
    /// The `byResolution` wire field.
    #[serde(rename = "byResolution")]
    pub by_resolution: EventLogMetricsResponseBucketsItemByResolution,
    /// The `byVerificationStatus` wire field.
    #[serde(rename = "byVerificationStatus")]
    pub by_verification_status: EventLogMetricsResponseBucketsItemByVerificationStatus,
    /// The `integrityExceptions` wire field.
    #[serde(rename = "integrityExceptions")]
    pub integrity_exceptions: i64,
    /// The `payloadSizeBytes` wire field.
    #[serde(rename = "payloadSizeBytes")]
    pub payload_size_bytes: i64,
    /// The `start` wire field.
    pub start: String,
    /// The `total` wire field.
    pub total: i64,
}
/// Generated schema type `EventLogMetricsResponseSummaryByResolution`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseSummaryByResolution {
    /// The `malformed` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub malformed: Option<i64>,
    /// The `resolved` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<i64>,
    /// The `unresolved` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved: Option<i64>,
}
/// Generated schema type `EventLogMetricsResponseSummaryByVerificationStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseSummaryByVerificationStatus {
    /// The `auth-unavailable` wire field.
    #[serde(rename = "auth-unavailable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_unavailable: Option<i64>,
    /// The `invalid-signature` wire field.
    #[serde(rename = "invalid-signature")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_signature: Option<i64>,
    /// The `missing-proof` wire field.
    #[serde(rename = "missing-proof")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_proof: Option<i64>,
    /// The `missing-session` wire field.
    #[serde(rename = "missing-session")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_session: Option<i64>,
    /// The `outside-session-window` wire field.
    #[serde(rename = "outside-session-window")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outside_session_window: Option<i64>,
    /// The `subject-denied` wire field.
    #[serde(rename = "subject-denied")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_denied: Option<i64>,
    /// The `verified` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<i64>,
}
/// Generated schema type `EventLogMetricsResponseSummaryEventTypesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseSummaryEventTypesItem {
    /// The `count` wire field.
    pub count: i64,
    /// The `ownerContractId` wire field.
    #[serde(rename = "ownerContractId")]
    pub owner_contract_id: String,
    /// The `ownerEventName` wire field.
    #[serde(rename = "ownerEventName")]
    pub owner_event_name: String,
}
/// Generated schema type `EventLogMetricsResponseSummary`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseSummary {
    /// The `byResolution` wire field.
    #[serde(rename = "byResolution")]
    pub by_resolution: EventLogMetricsResponseSummaryByResolution,
    /// The `byVerificationStatus` wire field.
    #[serde(rename = "byVerificationStatus")]
    pub by_verification_status: EventLogMetricsResponseSummaryByVerificationStatus,
    /// The `eventTypes` wire field.
    #[serde(rename = "eventTypes")]
    pub event_types: Vec<EventLogMetricsResponseSummaryEventTypesItem>,
    /// The `integrityExceptions` wire field.
    #[serde(rename = "integrityExceptions")]
    pub integrity_exceptions: i64,
    /// The `payloadSizeBytes` wire field.
    #[serde(rename = "payloadSizeBytes")]
    pub payload_size_bytes: i64,
    /// The `total` wire field.
    pub total: i64,
    /// The `uniqueSubjects` wire field.
    #[serde(rename = "uniqueSubjects")]
    pub unique_subjects: i64,
}
/// Generated schema type `EventLogMetricsResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponse {
    /// The `buckets` wire field.
    pub buckets: Vec<EventLogMetricsResponseBucketsItem>,
    /// The `summary` wire field.
    pub summary: EventLogMetricsResponseSummary,
}
/// Generated schema type `EventLogQueryRequestExcludeEventTypesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogQueryRequestExcludeEventTypesItem {
    /// The `ownerContractId` wire field.
    #[serde(rename = "ownerContractId")]
    pub owner_contract_id: String,
    /// The `ownerEventName` wire field.
    #[serde(rename = "ownerEventName")]
    pub owner_event_name: String,
}
/// Generated schema type `EventLogQueryRequestIncludeEventTypesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogQueryRequestIncludeEventTypesItem {
    /// The `ownerContractId` wire field.
    #[serde(rename = "ownerContractId")]
    pub owner_contract_id: String,
    /// The `ownerEventName` wire field.
    #[serde(rename = "ownerEventName")]
    pub owner_event_name: String,
}
/// Generated schema type `EventLogQueryRequestResolutionItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventLogQueryRequestResolutionItem {
    /// The `resolved` wire value.
    #[serde(rename = "resolved")]
    Resolved,
    /// The `unresolved` wire value.
    #[serde(rename = "unresolved")]
    Unresolved,
    /// The `malformed` wire value.
    #[serde(rename = "malformed")]
    Malformed,
}
impl EventLogQueryRequestResolutionItem {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Unresolved => "unresolved",
            Self::Malformed => "malformed",
        }
    }
}
impl AsRef<str> for EventLogQueryRequestResolutionItem {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for EventLogQueryRequestResolutionItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for EventLogQueryRequestResolutionItem {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<EventLogQueryRequestResolutionItem> for &str {
    fn eq(&self, other: &EventLogQueryRequestResolutionItem) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `EventLogQueryRequestVerificationStatusItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventLogQueryRequestVerificationStatusItem {
    /// The `verified` wire value.
    #[serde(rename = "verified")]
    Verified,
}
impl EventLogQueryRequestVerificationStatusItem {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
        }
    }
}
impl AsRef<str> for EventLogQueryRequestVerificationStatusItem {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for EventLogQueryRequestVerificationStatusItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for EventLogQueryRequestVerificationStatusItem {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<EventLogQueryRequestVerificationStatusItem> for &str {
    fn eq(&self, other: &EventLogQueryRequestVerificationStatusItem) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `EventLogQueryRequestWindow`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventLogQueryRequestWindow {
    /// The `15m` wire value.
    #[serde(rename = "15m")]
    V15m,
    /// The `1h` wire value.
    #[serde(rename = "1h")]
    V1h,
    /// The `6h` wire value.
    #[serde(rename = "6h")]
    V6h,
    /// The `24h` wire value.
    #[serde(rename = "24h")]
    V24h,
    /// The `7d` wire value.
    #[serde(rename = "7d")]
    V7d,
}
impl EventLogQueryRequestWindow {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::V15m => "15m",
            Self::V1h => "1h",
            Self::V6h => "6h",
            Self::V24h => "24h",
            Self::V7d => "7d",
        }
    }
}
impl AsRef<str> for EventLogQueryRequestWindow {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for EventLogQueryRequestWindow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for EventLogQueryRequestWindow {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<EventLogQueryRequestWindow> for &str {
    fn eq(&self, other: &EventLogQueryRequestWindow) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `EventLogQueryRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogQueryRequest {
    /// The `consumerDeploymentId` wire field.
    #[serde(rename = "consumerDeploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer_deployment_id: Option<String>,
    /// The `consumerName` wire field.
    #[serde(rename = "consumerName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer_name: Option<String>,
    /// The `excludeEventTypes` wire field.
    #[serde(rename = "excludeEventTypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_event_types: Option<Vec<EventLogQueryRequestExcludeEventTypesItem>>,
    /// The `includeEventTypes` wire field.
    #[serde(rename = "includeEventTypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_event_types: Option<Vec<EventLogQueryRequestIncludeEventTypesItem>>,
    /// The `integrityExceptionOnly` wire field.
    #[serde(rename = "integrityExceptionOnly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity_exception_only: Option<bool>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// The `ownerContractId` wire field.
    #[serde(rename = "ownerContractId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_contract_id: Option<String>,
    /// The `ownerEventName` wire field.
    #[serde(rename = "ownerEventName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_event_name: Option<String>,
    /// The `publisherDeploymentId` wire field.
    #[serde(rename = "publisherDeploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_deployment_id: Option<String>,
    /// The `publisherParticipantId` wire field.
    #[serde(rename = "publisherParticipantId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_participant_id: Option<String>,
    /// The `resolution` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Vec<EventLogQueryRequestResolutionItem>>,
    /// The `search` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    /// The `sort` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<BTreeMap<String, Value>>,
    /// The `subject` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The `verificationStatus` wire field.
    #[serde(rename = "verificationStatus")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_status: Option<Vec<EventLogQueryRequestVerificationStatusItem>>,
    /// The `window` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<EventLogQueryRequestWindow>,
}
/// Generated schema type `EventLogQueryResponseEventsItemPublisherKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventLogQueryResponseEventsItemPublisherKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
    /// The `user` wire value.
    #[serde(rename = "user")]
    User,
}
impl EventLogQueryResponseEventsItemPublisherKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
            Self::User => "user",
        }
    }
}
impl AsRef<str> for EventLogQueryResponseEventsItemPublisherKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for EventLogQueryResponseEventsItemPublisherKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for EventLogQueryResponseEventsItemPublisherKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<EventLogQueryResponseEventsItemPublisherKind> for &str {
    fn eq(&self, other: &EventLogQueryResponseEventsItemPublisherKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `EventLogQueryResponseEventsItemResolution`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventLogQueryResponseEventsItemResolution {
    /// The `resolved` wire value.
    #[serde(rename = "resolved")]
    Resolved,
    /// The `unresolved` wire value.
    #[serde(rename = "unresolved")]
    Unresolved,
    /// The `malformed` wire value.
    #[serde(rename = "malformed")]
    Malformed,
}
impl EventLogQueryResponseEventsItemResolution {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Unresolved => "unresolved",
            Self::Malformed => "malformed",
        }
    }
}
impl AsRef<str> for EventLogQueryResponseEventsItemResolution {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for EventLogQueryResponseEventsItemResolution {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for EventLogQueryResponseEventsItemResolution {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<EventLogQueryResponseEventsItemResolution> for &str {
    fn eq(&self, other: &EventLogQueryResponseEventsItemResolution) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `EventLogQueryResponseEventsItemVerificationStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventLogQueryResponseEventsItemVerificationStatus {
    /// The `verified` wire value.
    #[serde(rename = "verified")]
    Verified,
}
impl EventLogQueryResponseEventsItemVerificationStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
        }
    }
}
impl AsRef<str> for EventLogQueryResponseEventsItemVerificationStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for EventLogQueryResponseEventsItemVerificationStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for EventLogQueryResponseEventsItemVerificationStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<EventLogQueryResponseEventsItemVerificationStatus> for &str {
    fn eq(&self, other: &EventLogQueryResponseEventsItemVerificationStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `EventLogQueryResponseEventsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogQueryResponseEventsItem {
    /// The `eventId` wire field.
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// The `eventTime` wire field.
    #[serde(rename = "eventTime")]
    pub event_time: String,
    /// The `headerCount` wire field.
    #[serde(rename = "headerCount")]
    pub header_count: i64,
    /// The `ownerContractId` wire field.
    #[serde(rename = "ownerContractId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_contract_id: Option<String>,
    /// The `ownerEventName` wire field.
    #[serde(rename = "ownerEventName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_event_name: Option<String>,
    /// The `payloadSizeBytes` wire field.
    #[serde(rename = "payloadSizeBytes")]
    pub payload_size_bytes: i64,
    /// The `publisherDeploymentId` wire field.
    #[serde(rename = "publisherDeploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_deployment_id: Option<String>,
    /// The `publisherInstanceId` wire field.
    #[serde(rename = "publisherInstanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_instance_id: Option<String>,
    /// The `publisherKind` wire field.
    #[serde(rename = "publisherKind")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_kind: Option<EventLogQueryResponseEventsItemPublisherKind>,
    /// The `publisherParticipantDigest` wire field.
    #[serde(rename = "publisherParticipantDigest")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_participant_digest: Option<String>,
    /// The `publisherParticipantId` wire field.
    #[serde(rename = "publisherParticipantId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_participant_id: Option<String>,
    /// The `resolution` wire field.
    pub resolution: EventLogQueryResponseEventsItemResolution,
    /// The `streamSequence` wire field.
    #[serde(rename = "streamSequence")]
    pub stream_sequence: i64,
    /// The `subject` wire field.
    pub subject: String,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// The `verificationStatus` wire field.
    #[serde(rename = "verificationStatus")]
    pub verification_status: EventLogQueryResponseEventsItemVerificationStatus,
}
/// Generated schema type `EventLogQueryResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogQueryResponse {
    /// The `events` wire field.
    pub events: Vec<EventLogQueryResponseEventsItem>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    pub offset: i64,
    /// The `total` wire field.
    pub total: i64,
}
/// Generated schema type `EventLogWatchEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogWatchEvent(
    ///The wrapped wire value.
    pub BTreeMap<String, Value>,
);
/// Generated schema type `NotFoundErrorDataType`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotFoundErrorDataType {
    /// The `NotFoundError` wire value.
    #[serde(rename = "NotFoundError")]
    NotFoundError,
}
impl NotFoundErrorDataType {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NotFoundError => "NotFoundError",
        }
    }
}
impl AsRef<str> for NotFoundErrorDataType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for NotFoundErrorDataType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for NotFoundErrorDataType {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<NotFoundErrorDataType> for &str {
    fn eq(&self, other: &NotFoundErrorDataType) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `NotFoundErrorData`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotFoundErrorData {
    /// The `context` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<BTreeMap<String, Value>>,
    /// The `id` wire field.
    pub id: String,
    /// The `message` wire field.
    pub message: String,
    /// The `type` wire field.
    pub r#type: NotFoundErrorDataType,
}
