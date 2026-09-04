//! Shared request and response types for `trellis.jobs@v1`.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
/// Generated schema type `JobsCancelRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelRequest {
    /// The `id` wire field.
    pub id: String,
    /// The `reason` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
/// Generated schema type `JobsCancelResponseJobConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobConcurrency {
    /// The `heartbeatAt` wire field.
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    /// The `key` wire field.
    pub key: String,
    /// The `keyHash` wire field.
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    /// The `leaseExpiresAt` wire field.
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    /// The `staleTakeoverCount` wire field.
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsCancelResponseJobContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobContext {
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    pub request_id: String,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    pub trace_id: String,
    /// The `traceparent` wire field.
    pub traceparent: String,
    /// The `tracestate` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsCancelResponseJobErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobErrorDetailWorker {
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The `runtime` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `JobsCancelResponseJobErrorDetail`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobErrorDetail {
    /// The `causes` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    /// The `fingerprint` wire field.
    pub fingerprint: String,
    /// The `firstSeen` wire field.
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    /// The `message` wire field.
    pub message: String,
    /// The `occurrenceCount` wire field.
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    /// The `stack` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `worker` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsCancelResponseJobErrorDetailWorker>,
}
/// Generated schema type `JobsCancelResponseJobLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobLineage {
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `relatedKeys` wire field.
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    /// The `rootJobId` wire field.
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsCancelResponseJobLogsItemLevel`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsCancelResponseJobLogsItemLevel {
    /// The `info` wire value.
    #[serde(rename = "info")]
    Info,
    /// The `warn` wire value.
    #[serde(rename = "warn")]
    Warn,
    /// The `error` wire value.
    #[serde(rename = "error")]
    Error,
}
impl JobsCancelResponseJobLogsItemLevel {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
impl AsRef<str> for JobsCancelResponseJobLogsItemLevel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsCancelResponseJobLogsItemLevel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsCancelResponseJobLogsItemLevel {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsCancelResponseJobLogsItemLevel> for &str {
    fn eq(&self, other: &JobsCancelResponseJobLogsItemLevel) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsCancelResponseJobLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobLogsItem {
    /// The `level` wire field.
    pub level: JobsCancelResponseJobLogsItemLevel,
    /// The `message` wire field.
    pub message: String,
    /// The `timestamp` wire field.
    pub timestamp: String,
}
/// Generated schema type `JobsCancelResponseJobProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobProgress {
    /// The `current` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    /// The `message` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The `step` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// The `total` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsCancelResponseJobQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobQueuePolicy {
    /// The `existingJobId` wire field.
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    /// The `outcome` wire field.
    pub outcome: String,
    /// The `reason` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The `replacedJobId` wire field.
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsCancelResponseJobState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsCancelResponseJobState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsCancelResponseJobState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsCancelResponseJobState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsCancelResponseJobState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsCancelResponseJobState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsCancelResponseJobState> for &str {
    fn eq(&self, other: &JobsCancelResponseJobState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsCancelResponseJobTriggerKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsCancelResponseJobTriggerKind {
    /// The `schedule` wire value.
    #[serde(rename = "schedule")]
    Schedule,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `manualReplay` wire value.
    #[serde(rename = "manualReplay")]
    ManualReplay,
    /// The `serviceCode` wire value.
    #[serde(rename = "serviceCode")]
    ServiceCode,
    /// The `parentJob` wire value.
    #[serde(rename = "parentJob")]
    ParentJob,
}
impl JobsCancelResponseJobTriggerKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Schedule => "schedule",
            Self::Operation => "operation",
            Self::Rpc => "rpc",
            Self::Event => "event",
            Self::ManualReplay => "manualReplay",
            Self::ServiceCode => "serviceCode",
            Self::ParentJob => "parentJob",
        }
    }
}
impl AsRef<str> for JobsCancelResponseJobTriggerKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsCancelResponseJobTriggerKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsCancelResponseJobTriggerKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsCancelResponseJobTriggerKind> for &str {
    fn eq(&self, other: &JobsCancelResponseJobTriggerKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsCancelResponseJobTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobTrigger {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsCancelResponseJobTriggerKind,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// The `subject` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
/// Generated schema type `JobsCancelResponseJobWaitingOnItemTargetKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsCancelResponseJobWaitingOnItemTargetKind {
    /// The `job` wire value.
    #[serde(rename = "job")]
    Job,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `external` wire value.
    #[serde(rename = "external")]
    External,
}
impl JobsCancelResponseJobWaitingOnItemTargetKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Operation => "operation",
            Self::External => "external",
        }
    }
}
impl AsRef<str> for JobsCancelResponseJobWaitingOnItemTargetKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsCancelResponseJobWaitingOnItemTargetKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsCancelResponseJobWaitingOnItemTargetKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsCancelResponseJobWaitingOnItemTargetKind> for &str {
    fn eq(&self, other: &JobsCancelResponseJobWaitingOnItemTargetKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsCancelResponseJobWaitingOnItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobWaitingOnItemTarget {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsCancelResponseJobWaitingOnItemTargetKind,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `operation` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `system` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}
/// Generated schema type `JobsCancelResponseJobWaitingOnItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobWaitingOnItem {
    /// The `id` wire field.
    pub id: String,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `target` wire field.
    pub target: JobsCancelResponseJobWaitingOnItemTarget,
}
/// Generated schema type `JobsCancelResponseJob`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJob {
    /// The `completedAt` wire field.
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// The `concurrency` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsCancelResponseJobConcurrency>,
    /// The `context` wire field.
    pub context: JobsCancelResponseJobContext,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// The `deadline` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// The `errorDetail` wire field.
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsCancelResponseJobErrorDetail>,
    /// The `id` wire field.
    pub id: String,
    /// The `lastError` wire field.
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// The `lineage` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsCancelResponseJobLineage>,
    /// The `logs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsCancelResponseJobLogsItem>>,
    /// The `maxTries` wire field.
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    /// The `payload` wire field.
    pub payload: Value,
    /// The `progress` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsCancelResponseJobProgress>,
    /// The `queuePolicy` wire field.
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsCancelResponseJobQueuePolicy>,
    /// The `result` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The `service` wire field.
    pub service: String,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// The `state` wire field.
    pub state: JobsCancelResponseJobState,
    /// The `tries` wire field.
    pub tries: i64,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsCancelResponseJobTrigger>,
    /// The `type` wire field.
    pub r#type: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `waitingOn` wire field.
    #[serde(rename = "waitingOn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_on: Option<Vec<JobsCancelResponseJobWaitingOnItem>>,
}
/// Generated schema type `JobsCancelResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponse {
    /// The `job` wire field.
    pub job: JobsCancelResponseJob,
}
/// Generated schema type `JobsDismissDLQRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQRequest {
    /// The `id` wire field.
    pub id: String,
    /// The `reason` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
/// Generated schema type `JobsDismissDLQResponseJobConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobConcurrency {
    /// The `heartbeatAt` wire field.
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    /// The `key` wire field.
    pub key: String,
    /// The `keyHash` wire field.
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    /// The `leaseExpiresAt` wire field.
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    /// The `staleTakeoverCount` wire field.
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsDismissDLQResponseJobContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobContext {
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    pub request_id: String,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    pub trace_id: String,
    /// The `traceparent` wire field.
    pub traceparent: String,
    /// The `tracestate` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsDismissDLQResponseJobErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobErrorDetailWorker {
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The `runtime` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `JobsDismissDLQResponseJobErrorDetail`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobErrorDetail {
    /// The `causes` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    /// The `fingerprint` wire field.
    pub fingerprint: String,
    /// The `firstSeen` wire field.
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    /// The `message` wire field.
    pub message: String,
    /// The `occurrenceCount` wire field.
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    /// The `stack` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `worker` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsDismissDLQResponseJobErrorDetailWorker>,
}
/// Generated schema type `JobsDismissDLQResponseJobLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobLineage {
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `relatedKeys` wire field.
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    /// The `rootJobId` wire field.
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsDismissDLQResponseJobLogsItemLevel`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsDismissDLQResponseJobLogsItemLevel {
    /// The `info` wire value.
    #[serde(rename = "info")]
    Info,
    /// The `warn` wire value.
    #[serde(rename = "warn")]
    Warn,
    /// The `error` wire value.
    #[serde(rename = "error")]
    Error,
}
impl JobsDismissDLQResponseJobLogsItemLevel {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
impl AsRef<str> for JobsDismissDLQResponseJobLogsItemLevel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsDismissDLQResponseJobLogsItemLevel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsDismissDLQResponseJobLogsItemLevel {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsDismissDLQResponseJobLogsItemLevel> for &str {
    fn eq(&self, other: &JobsDismissDLQResponseJobLogsItemLevel) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsDismissDLQResponseJobLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobLogsItem {
    /// The `level` wire field.
    pub level: JobsDismissDLQResponseJobLogsItemLevel,
    /// The `message` wire field.
    pub message: String,
    /// The `timestamp` wire field.
    pub timestamp: String,
}
/// Generated schema type `JobsDismissDLQResponseJobProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobProgress {
    /// The `current` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    /// The `message` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The `step` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// The `total` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsDismissDLQResponseJobQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobQueuePolicy {
    /// The `existingJobId` wire field.
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    /// The `outcome` wire field.
    pub outcome: String,
    /// The `reason` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The `replacedJobId` wire field.
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsDismissDLQResponseJobState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsDismissDLQResponseJobState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsDismissDLQResponseJobState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsDismissDLQResponseJobState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsDismissDLQResponseJobState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsDismissDLQResponseJobState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsDismissDLQResponseJobState> for &str {
    fn eq(&self, other: &JobsDismissDLQResponseJobState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsDismissDLQResponseJobTriggerKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsDismissDLQResponseJobTriggerKind {
    /// The `schedule` wire value.
    #[serde(rename = "schedule")]
    Schedule,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `manualReplay` wire value.
    #[serde(rename = "manualReplay")]
    ManualReplay,
    /// The `serviceCode` wire value.
    #[serde(rename = "serviceCode")]
    ServiceCode,
    /// The `parentJob` wire value.
    #[serde(rename = "parentJob")]
    ParentJob,
}
impl JobsDismissDLQResponseJobTriggerKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Schedule => "schedule",
            Self::Operation => "operation",
            Self::Rpc => "rpc",
            Self::Event => "event",
            Self::ManualReplay => "manualReplay",
            Self::ServiceCode => "serviceCode",
            Self::ParentJob => "parentJob",
        }
    }
}
impl AsRef<str> for JobsDismissDLQResponseJobTriggerKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsDismissDLQResponseJobTriggerKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsDismissDLQResponseJobTriggerKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsDismissDLQResponseJobTriggerKind> for &str {
    fn eq(&self, other: &JobsDismissDLQResponseJobTriggerKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsDismissDLQResponseJobTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobTrigger {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsDismissDLQResponseJobTriggerKind,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// The `subject` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
/// Generated schema type `JobsDismissDLQResponseJobWaitingOnItemTargetKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsDismissDLQResponseJobWaitingOnItemTargetKind {
    /// The `job` wire value.
    #[serde(rename = "job")]
    Job,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `external` wire value.
    #[serde(rename = "external")]
    External,
}
impl JobsDismissDLQResponseJobWaitingOnItemTargetKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Operation => "operation",
            Self::External => "external",
        }
    }
}
impl AsRef<str> for JobsDismissDLQResponseJobWaitingOnItemTargetKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsDismissDLQResponseJobWaitingOnItemTargetKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsDismissDLQResponseJobWaitingOnItemTargetKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsDismissDLQResponseJobWaitingOnItemTargetKind> for &str {
    fn eq(&self, other: &JobsDismissDLQResponseJobWaitingOnItemTargetKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsDismissDLQResponseJobWaitingOnItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobWaitingOnItemTarget {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsDismissDLQResponseJobWaitingOnItemTargetKind,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `operation` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `system` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}
/// Generated schema type `JobsDismissDLQResponseJobWaitingOnItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobWaitingOnItem {
    /// The `id` wire field.
    pub id: String,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `target` wire field.
    pub target: JobsDismissDLQResponseJobWaitingOnItemTarget,
}
/// Generated schema type `JobsDismissDLQResponseJob`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJob {
    /// The `completedAt` wire field.
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// The `concurrency` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsDismissDLQResponseJobConcurrency>,
    /// The `context` wire field.
    pub context: JobsDismissDLQResponseJobContext,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// The `deadline` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// The `errorDetail` wire field.
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsDismissDLQResponseJobErrorDetail>,
    /// The `id` wire field.
    pub id: String,
    /// The `lastError` wire field.
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// The `lineage` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsDismissDLQResponseJobLineage>,
    /// The `logs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsDismissDLQResponseJobLogsItem>>,
    /// The `maxTries` wire field.
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    /// The `payload` wire field.
    pub payload: Value,
    /// The `progress` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsDismissDLQResponseJobProgress>,
    /// The `queuePolicy` wire field.
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsDismissDLQResponseJobQueuePolicy>,
    /// The `result` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The `service` wire field.
    pub service: String,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// The `state` wire field.
    pub state: JobsDismissDLQResponseJobState,
    /// The `tries` wire field.
    pub tries: i64,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsDismissDLQResponseJobTrigger>,
    /// The `type` wire field.
    pub r#type: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `waitingOn` wire field.
    #[serde(rename = "waitingOn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_on: Option<Vec<JobsDismissDLQResponseJobWaitingOnItem>>,
}
/// Generated schema type `JobsDismissDLQResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponse {
    /// The `job` wire field.
    pub job: JobsDismissDLQResponseJob,
}
/// Generated schema type `JobsGetKeyRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsGetKeyRequest {
    /// The `key` wire field.
    pub key: String,
    /// The `service` wire field.
    pub service: String,
    /// The `type` wire field.
    pub r#type: String,
}
/// Generated schema type `JobsGetKeyResponseActiveItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsGetKeyResponseActiveItem {
    /// The `heartbeatAgeMs` wire field.
    #[serde(rename = "heartbeatAgeMs")]
    pub heartbeat_age_ms: i64,
    /// The `heartbeatAt` wire field.
    #[serde(rename = "heartbeatAt")]
    pub heartbeat_at: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `jobId` wire field.
    #[serde(rename = "jobId")]
    pub job_id: String,
    /// The `leaseExpiresAt` wire field.
    #[serde(rename = "leaseExpiresAt")]
    pub lease_expires_at: String,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
}
/// Generated schema type `JobsGetKeyResponseQueuedItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsGetKeyResponseQueuedItem {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// The `jobId` wire field.
    #[serde(rename = "jobId")]
    pub job_id: String,
}
/// Generated schema type `JobsGetKeyResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsGetKeyResponse {
    /// The `active` wire field.
    pub active: Vec<JobsGetKeyResponseActiveItem>,
    /// The `key` wire field.
    pub key: String,
    /// The `keyHash` wire field.
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    /// The `latestPolicyReason` wire field.
    #[serde(rename = "latestPolicyReason")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_policy_reason: Option<String>,
    /// The `queued` wire field.
    pub queued: Vec<JobsGetKeyResponseQueuedItem>,
    /// The `queuedDepth` wire field.
    #[serde(rename = "queuedDepth")]
    pub queued_depth: i64,
    /// The `service` wire field.
    pub service: String,
    /// The `staleTakeoverCount` wire field.
    #[serde(rename = "staleTakeoverCount")]
    pub stale_takeover_count: i64,
    /// The `type` wire field.
    pub r#type: String,
}
/// Generated schema type `JobsInspectRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectRequest {
    /// The `id` wire field.
    pub id: String,
}
/// Generated schema type `JobsInspectResponseAttemptsItemErrorWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseAttemptsItemErrorWorker {
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The `runtime` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `JobsInspectResponseAttemptsItemError`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseAttemptsItemError {
    /// The `causes` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    /// The `fingerprint` wire field.
    pub fingerprint: String,
    /// The `firstSeen` wire field.
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    /// The `message` wire field.
    pub message: String,
    /// The `occurrenceCount` wire field.
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    /// The `stack` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `worker` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsInspectResponseAttemptsItemErrorWorker>,
}
/// Generated schema type `JobsInspectResponseAttemptsItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseAttemptsItemState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsInspectResponseAttemptsItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsInspectResponseAttemptsItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseAttemptsItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseAttemptsItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseAttemptsItemState> for &str {
    fn eq(&self, other: &JobsInspectResponseAttemptsItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseAttemptsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseAttemptsItem {
    /// The `endedAt` wire field.
    #[serde(rename = "endedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    /// The `error` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JobsInspectResponseAttemptsItemError>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<JobsInspectResponseAttemptsItemState>,
    /// The `try` wire field.
    pub r#try: i64,
}
/// Generated schema type `JobsInspectResponseErrorsItemWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseErrorsItemWorker {
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The `runtime` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `JobsInspectResponseErrorsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseErrorsItem {
    /// The `causes` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    /// The `fingerprint` wire field.
    pub fingerprint: String,
    /// The `firstSeen` wire field.
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    /// The `message` wire field.
    pub message: String,
    /// The `occurrenceCount` wire field.
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    /// The `stack` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `worker` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsInspectResponseErrorsItemWorker>,
}
/// Generated schema type `JobsInspectResponseJobConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobConcurrency {
    /// The `heartbeatAt` wire field.
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    /// The `key` wire field.
    pub key: String,
    /// The `keyHash` wire field.
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    /// The `leaseExpiresAt` wire field.
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    /// The `staleTakeoverCount` wire field.
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsInspectResponseJobContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobContext {
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    pub request_id: String,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    pub trace_id: String,
    /// The `traceparent` wire field.
    pub traceparent: String,
    /// The `tracestate` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsInspectResponseJobErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobErrorDetailWorker {
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The `runtime` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `JobsInspectResponseJobErrorDetail`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobErrorDetail {
    /// The `causes` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    /// The `fingerprint` wire field.
    pub fingerprint: String,
    /// The `firstSeen` wire field.
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    /// The `message` wire field.
    pub message: String,
    /// The `occurrenceCount` wire field.
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    /// The `stack` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `worker` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsInspectResponseJobErrorDetailWorker>,
}
/// Generated schema type `JobsInspectResponseJobLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobLineage {
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `relatedKeys` wire field.
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    /// The `rootJobId` wire field.
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseJobLogsItemLevel`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseJobLogsItemLevel {
    /// The `info` wire value.
    #[serde(rename = "info")]
    Info,
    /// The `warn` wire value.
    #[serde(rename = "warn")]
    Warn,
    /// The `error` wire value.
    #[serde(rename = "error")]
    Error,
}
impl JobsInspectResponseJobLogsItemLevel {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
impl AsRef<str> for JobsInspectResponseJobLogsItemLevel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseJobLogsItemLevel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseJobLogsItemLevel {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseJobLogsItemLevel> for &str {
    fn eq(&self, other: &JobsInspectResponseJobLogsItemLevel) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseJobLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobLogsItem {
    /// The `level` wire field.
    pub level: JobsInspectResponseJobLogsItemLevel,
    /// The `message` wire field.
    pub message: String,
    /// The `timestamp` wire field.
    pub timestamp: String,
}
/// Generated schema type `JobsInspectResponseJobProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobProgress {
    /// The `current` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    /// The `message` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The `step` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// The `total` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsInspectResponseJobQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobQueuePolicy {
    /// The `existingJobId` wire field.
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    /// The `outcome` wire field.
    pub outcome: String,
    /// The `reason` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The `replacedJobId` wire field.
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseJobState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseJobState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsInspectResponseJobState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsInspectResponseJobState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseJobState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseJobState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseJobState> for &str {
    fn eq(&self, other: &JobsInspectResponseJobState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseJobTriggerKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseJobTriggerKind {
    /// The `schedule` wire value.
    #[serde(rename = "schedule")]
    Schedule,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `manualReplay` wire value.
    #[serde(rename = "manualReplay")]
    ManualReplay,
    /// The `serviceCode` wire value.
    #[serde(rename = "serviceCode")]
    ServiceCode,
    /// The `parentJob` wire value.
    #[serde(rename = "parentJob")]
    ParentJob,
}
impl JobsInspectResponseJobTriggerKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Schedule => "schedule",
            Self::Operation => "operation",
            Self::Rpc => "rpc",
            Self::Event => "event",
            Self::ManualReplay => "manualReplay",
            Self::ServiceCode => "serviceCode",
            Self::ParentJob => "parentJob",
        }
    }
}
impl AsRef<str> for JobsInspectResponseJobTriggerKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseJobTriggerKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseJobTriggerKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseJobTriggerKind> for &str {
    fn eq(&self, other: &JobsInspectResponseJobTriggerKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseJobTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobTrigger {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsInspectResponseJobTriggerKind,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// The `subject` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseJobWaitingOnItemTargetKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseJobWaitingOnItemTargetKind {
    /// The `job` wire value.
    #[serde(rename = "job")]
    Job,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `external` wire value.
    #[serde(rename = "external")]
    External,
}
impl JobsInspectResponseJobWaitingOnItemTargetKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Operation => "operation",
            Self::External => "external",
        }
    }
}
impl AsRef<str> for JobsInspectResponseJobWaitingOnItemTargetKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseJobWaitingOnItemTargetKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseJobWaitingOnItemTargetKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseJobWaitingOnItemTargetKind> for &str {
    fn eq(&self, other: &JobsInspectResponseJobWaitingOnItemTargetKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseJobWaitingOnItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobWaitingOnItemTarget {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsInspectResponseJobWaitingOnItemTargetKind,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `operation` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `system` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}
/// Generated schema type `JobsInspectResponseJobWaitingOnItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobWaitingOnItem {
    /// The `id` wire field.
    pub id: String,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `target` wire field.
    pub target: JobsInspectResponseJobWaitingOnItemTarget,
}
/// Generated schema type `JobsInspectResponseJob`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJob {
    /// The `completedAt` wire field.
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// The `concurrency` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsInspectResponseJobConcurrency>,
    /// The `context` wire field.
    pub context: JobsInspectResponseJobContext,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// The `deadline` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// The `errorDetail` wire field.
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsInspectResponseJobErrorDetail>,
    /// The `id` wire field.
    pub id: String,
    /// The `lastError` wire field.
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// The `lineage` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsInspectResponseJobLineage>,
    /// The `logs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsInspectResponseJobLogsItem>>,
    /// The `maxTries` wire field.
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    /// The `payload` wire field.
    pub payload: Value,
    /// The `progress` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsInspectResponseJobProgress>,
    /// The `queuePolicy` wire field.
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsInspectResponseJobQueuePolicy>,
    /// The `result` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The `service` wire field.
    pub service: String,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// The `state` wire field.
    pub state: JobsInspectResponseJobState,
    /// The `tries` wire field.
    pub tries: i64,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsInspectResponseJobTrigger>,
    /// The `type` wire field.
    pub r#type: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `waitingOn` wire field.
    #[serde(rename = "waitingOn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_on: Option<Vec<JobsInspectResponseJobWaitingOnItem>>,
}
/// Generated schema type `JobsInspectResponseLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseLineage {
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `relatedKeys` wire field.
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    /// The `rootJobId` wire field.
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseRelatedItemContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItemContext {
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    pub request_id: String,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    pub trace_id: String,
    /// The `traceparent` wire field.
    pub traceparent: String,
    /// The `tracestate` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsInspectResponseRelatedItemLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItemLineage {
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `relatedKeys` wire field.
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    /// The `rootJobId` wire field.
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseRelatedItemMatchedBy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseRelatedItemMatchedBy {
    /// The `trace` wire value.
    #[serde(rename = "trace")]
    Trace,
    /// The `parent` wire value.
    #[serde(rename = "parent")]
    Parent,
    /// The `root` wire value.
    #[serde(rename = "root")]
    Root,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `concurrency` wire value.
    #[serde(rename = "concurrency")]
    Concurrency,
    /// The `wait` wire value.
    #[serde(rename = "wait")]
    Wait,
}
impl JobsInspectResponseRelatedItemMatchedBy {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Parent => "parent",
            Self::Root => "root",
            Self::Operation => "operation",
            Self::Concurrency => "concurrency",
            Self::Wait => "wait",
        }
    }
}
impl AsRef<str> for JobsInspectResponseRelatedItemMatchedBy {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseRelatedItemMatchedBy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseRelatedItemMatchedBy {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseRelatedItemMatchedBy> for &str {
    fn eq(&self, other: &JobsInspectResponseRelatedItemMatchedBy) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseRelatedItemProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItemProgress {
    /// The `current` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    /// The `message` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The `step` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// The `total` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsInspectResponseRelatedItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseRelatedItemState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsInspectResponseRelatedItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsInspectResponseRelatedItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseRelatedItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseRelatedItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseRelatedItemState> for &str {
    fn eq(&self, other: &JobsInspectResponseRelatedItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseRelatedItemTriggerKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseRelatedItemTriggerKind {
    /// The `schedule` wire value.
    #[serde(rename = "schedule")]
    Schedule,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `manualReplay` wire value.
    #[serde(rename = "manualReplay")]
    ManualReplay,
    /// The `serviceCode` wire value.
    #[serde(rename = "serviceCode")]
    ServiceCode,
    /// The `parentJob` wire value.
    #[serde(rename = "parentJob")]
    ParentJob,
}
impl JobsInspectResponseRelatedItemTriggerKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Schedule => "schedule",
            Self::Operation => "operation",
            Self::Rpc => "rpc",
            Self::Event => "event",
            Self::ManualReplay => "manualReplay",
            Self::ServiceCode => "serviceCode",
            Self::ParentJob => "parentJob",
        }
    }
}
impl AsRef<str> for JobsInspectResponseRelatedItemTriggerKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseRelatedItemTriggerKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseRelatedItemTriggerKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseRelatedItemTriggerKind> for &str {
    fn eq(&self, other: &JobsInspectResponseRelatedItemTriggerKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseRelatedItemTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItemTrigger {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsInspectResponseRelatedItemTriggerKind,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// The `subject` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseRelatedItemWaitingOnItemTargetKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseRelatedItemWaitingOnItemTargetKind {
    /// The `job` wire value.
    #[serde(rename = "job")]
    Job,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `external` wire value.
    #[serde(rename = "external")]
    External,
}
impl JobsInspectResponseRelatedItemWaitingOnItemTargetKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Operation => "operation",
            Self::External => "external",
        }
    }
}
impl AsRef<str> for JobsInspectResponseRelatedItemWaitingOnItemTargetKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseRelatedItemWaitingOnItemTargetKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseRelatedItemWaitingOnItemTargetKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseRelatedItemWaitingOnItemTargetKind> for &str {
    fn eq(&self, other: &JobsInspectResponseRelatedItemWaitingOnItemTargetKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseRelatedItemWaitingOnItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItemWaitingOnItemTarget {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsInspectResponseRelatedItemWaitingOnItemTargetKind,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `operation` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `system` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}
/// Generated schema type `JobsInspectResponseRelatedItemWaitingOnItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItemWaitingOnItem {
    /// The `id` wire field.
    pub id: String,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `target` wire field.
    pub target: JobsInspectResponseRelatedItemWaitingOnItemTarget,
}
/// Generated schema type `JobsInspectResponseRelatedItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItem {
    /// The `completedAt` wire field.
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// The `context` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<JobsInspectResponseRelatedItemContext>,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// The `errorFingerprint` wire field.
    #[serde(rename = "errorFingerprint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_fingerprint: Option<String>,
    /// The `id` wire field.
    pub id: String,
    /// The `lastError` wire field.
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// The `lineage` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsInspectResponseRelatedItemLineage>,
    /// The `matchedBy` wire field.
    #[serde(rename = "matchedBy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_by: Option<JobsInspectResponseRelatedItemMatchedBy>,
    /// The `maxTries` wire field.
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    /// The `progress` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsInspectResponseRelatedItemProgress>,
    /// The `queueAgeMs` wire field.
    #[serde(rename = "queueAgeMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_age_ms: Option<i64>,
    /// The `queueKey` wire field.
    #[serde(rename = "queueKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_key: Option<String>,
    /// The `runtimeBand` wire field.
    #[serde(rename = "runtimeBand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_band: Option<String>,
    /// The `runtimeMs` wire field.
    #[serde(rename = "runtimeMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_ms: Option<i64>,
    /// The `service` wire field.
    pub service: String,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// The `state` wire field.
    pub state: JobsInspectResponseRelatedItemState,
    /// The `tries` wire field.
    pub tries: i64,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsInspectResponseRelatedItemTrigger>,
    /// The `type` wire field.
    pub r#type: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `waitingOn` wire field.
    #[serde(rename = "waitingOn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_on: Option<Vec<JobsInspectResponseRelatedItemWaitingOnItem>>,
}
/// Generated schema type `JobsInspectResponseTimelineItemErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItemErrorDetailWorker {
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The `runtime` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `JobsInspectResponseTimelineItemErrorDetail`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItemErrorDetail {
    /// The `causes` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    /// The `fingerprint` wire field.
    pub fingerprint: String,
    /// The `firstSeen` wire field.
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    /// The `message` wire field.
    pub message: String,
    /// The `occurrenceCount` wire field.
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    /// The `stack` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `worker` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsInspectResponseTimelineItemErrorDetailWorker>,
}
/// Generated schema type `JobsInspectResponseTimelineItemLogsItemLevel`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseTimelineItemLogsItemLevel {
    /// The `info` wire value.
    #[serde(rename = "info")]
    Info,
    /// The `warn` wire value.
    #[serde(rename = "warn")]
    Warn,
    /// The `error` wire value.
    #[serde(rename = "error")]
    Error,
}
impl JobsInspectResponseTimelineItemLogsItemLevel {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
impl AsRef<str> for JobsInspectResponseTimelineItemLogsItemLevel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseTimelineItemLogsItemLevel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseTimelineItemLogsItemLevel {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseTimelineItemLogsItemLevel> for &str {
    fn eq(&self, other: &JobsInspectResponseTimelineItemLogsItemLevel) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseTimelineItemLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItemLogsItem {
    /// The `level` wire field.
    pub level: JobsInspectResponseTimelineItemLogsItemLevel,
    /// The `message` wire field.
    pub message: String,
    /// The `timestamp` wire field.
    pub timestamp: String,
}
/// Generated schema type `JobsInspectResponseTimelineItemPreviousState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseTimelineItemPreviousState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsInspectResponseTimelineItemPreviousState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsInspectResponseTimelineItemPreviousState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseTimelineItemPreviousState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseTimelineItemPreviousState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseTimelineItemPreviousState> for &str {
    fn eq(&self, other: &JobsInspectResponseTimelineItemPreviousState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseTimelineItemProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItemProgress {
    /// The `current` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    /// The `message` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The `step` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// The `total` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsInspectResponseTimelineItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseTimelineItemState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsInspectResponseTimelineItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsInspectResponseTimelineItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseTimelineItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseTimelineItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseTimelineItemState> for &str {
    fn eq(&self, other: &JobsInspectResponseTimelineItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseTimelineItemWaitEdgeTargetKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseTimelineItemWaitEdgeTargetKind {
    /// The `job` wire value.
    #[serde(rename = "job")]
    Job,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `external` wire value.
    #[serde(rename = "external")]
    External,
}
impl JobsInspectResponseTimelineItemWaitEdgeTargetKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Operation => "operation",
            Self::External => "external",
        }
    }
}
impl AsRef<str> for JobsInspectResponseTimelineItemWaitEdgeTargetKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseTimelineItemWaitEdgeTargetKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseTimelineItemWaitEdgeTargetKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseTimelineItemWaitEdgeTargetKind> for &str {
    fn eq(&self, other: &JobsInspectResponseTimelineItemWaitEdgeTargetKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseTimelineItemWaitEdgeTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItemWaitEdgeTarget {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsInspectResponseTimelineItemWaitEdgeTargetKind,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `operation` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `system` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}
/// Generated schema type `JobsInspectResponseTimelineItemWaitEdge`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItemWaitEdge {
    /// The `id` wire field.
    pub id: String,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `target` wire field.
    pub target: JobsInspectResponseTimelineItemWaitEdgeTarget,
}
/// Generated schema type `JobsInspectResponseTimelineItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItem {
    /// The `error` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The `errorDetail` wire field.
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsInspectResponseTimelineItemErrorDetail>,
    /// The `logs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsInspectResponseTimelineItemLogsItem>>,
    /// The `message` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The `previousState` wire field.
    #[serde(rename = "previousState")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_state: Option<JobsInspectResponseTimelineItemPreviousState>,
    /// The `progress` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsInspectResponseTimelineItemProgress>,
    /// The `projected` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projected: Option<bool>,
    /// The `rawEvent` wire field.
    #[serde(rename = "rawEvent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_event: Option<Value>,
    /// The `reason` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The `sequence` wire field.
    pub sequence: i64,
    /// The `state` wire field.
    pub state: JobsInspectResponseTimelineItemState,
    /// The `timestamp` wire field.
    pub timestamp: String,
    /// The `tries` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tries: Option<i64>,
    /// The `type` wire field.
    pub r#type: String,
    /// The `waitEdge` wire field.
    #[serde(rename = "waitEdge")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_edge: Option<JobsInspectResponseTimelineItemWaitEdge>,
    /// The `workerInstanceId` wire field.
    #[serde(rename = "workerInstanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_instance_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseTriggerKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsInspectResponseTriggerKind {
    /// The `schedule` wire value.
    #[serde(rename = "schedule")]
    Schedule,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `manualReplay` wire value.
    #[serde(rename = "manualReplay")]
    ManualReplay,
    /// The `serviceCode` wire value.
    #[serde(rename = "serviceCode")]
    ServiceCode,
    /// The `parentJob` wire value.
    #[serde(rename = "parentJob")]
    ParentJob,
}
impl JobsInspectResponseTriggerKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Schedule => "schedule",
            Self::Operation => "operation",
            Self::Rpc => "rpc",
            Self::Event => "event",
            Self::ManualReplay => "manualReplay",
            Self::ServiceCode => "serviceCode",
            Self::ParentJob => "parentJob",
        }
    }
}
impl AsRef<str> for JobsInspectResponseTriggerKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsInspectResponseTriggerKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsInspectResponseTriggerKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsInspectResponseTriggerKind> for &str {
    fn eq(&self, other: &JobsInspectResponseTriggerKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsInspectResponseTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTrigger {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsInspectResponseTriggerKind,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// The `subject` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
/// Generated schema type `JobsInspectResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponse {
    /// The `attempts` wire field.
    pub attempts: Vec<JobsInspectResponseAttemptsItem>,
    /// The `errors` wire field.
    pub errors: Vec<JobsInspectResponseErrorsItem>,
    /// The `job` wire field.
    pub job: JobsInspectResponseJob,
    /// The `lineage` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsInspectResponseLineage>,
    /// The `related` wire field.
    pub related: Vec<JobsInspectResponseRelatedItem>,
    /// The `timeline` wire field.
    pub timeline: Vec<JobsInspectResponseTimelineItem>,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsInspectResponseTrigger>,
}
/// Generated schema type `JobsListDLQRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQRequest {
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `since` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemConcurrency {
    /// The `heartbeatAt` wire field.
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    /// The `key` wire field.
    pub key: String,
    /// The `keyHash` wire field.
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    /// The `leaseExpiresAt` wire field.
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    /// The `staleTakeoverCount` wire field.
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemContext {
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    pub request_id: String,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    pub trace_id: String,
    /// The `traceparent` wire field.
    pub traceparent: String,
    /// The `tracestate` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemErrorDetailWorker {
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The `runtime` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemErrorDetail`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemErrorDetail {
    /// The `causes` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    /// The `fingerprint` wire field.
    pub fingerprint: String,
    /// The `firstSeen` wire field.
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    /// The `message` wire field.
    pub message: String,
    /// The `occurrenceCount` wire field.
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    /// The `stack` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `worker` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsListDLQResponseEntriesItemErrorDetailWorker>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemLineage {
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `relatedKeys` wire field.
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    /// The `rootJobId` wire field.
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemLogsItemLevel`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsListDLQResponseEntriesItemLogsItemLevel {
    /// The `info` wire value.
    #[serde(rename = "info")]
    Info,
    /// The `warn` wire value.
    #[serde(rename = "warn")]
    Warn,
    /// The `error` wire value.
    #[serde(rename = "error")]
    Error,
}
impl JobsListDLQResponseEntriesItemLogsItemLevel {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
impl AsRef<str> for JobsListDLQResponseEntriesItemLogsItemLevel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsListDLQResponseEntriesItemLogsItemLevel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsListDLQResponseEntriesItemLogsItemLevel {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsListDLQResponseEntriesItemLogsItemLevel> for &str {
    fn eq(&self, other: &JobsListDLQResponseEntriesItemLogsItemLevel) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsListDLQResponseEntriesItemLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemLogsItem {
    /// The `level` wire field.
    pub level: JobsListDLQResponseEntriesItemLogsItemLevel,
    /// The `message` wire field.
    pub message: String,
    /// The `timestamp` wire field.
    pub timestamp: String,
}
/// Generated schema type `JobsListDLQResponseEntriesItemProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemProgress {
    /// The `current` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    /// The `message` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The `step` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// The `total` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemQueuePolicy {
    /// The `existingJobId` wire field.
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    /// The `outcome` wire field.
    pub outcome: String,
    /// The `reason` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The `replacedJobId` wire field.
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsListDLQResponseEntriesItemState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsListDLQResponseEntriesItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsListDLQResponseEntriesItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsListDLQResponseEntriesItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsListDLQResponseEntriesItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsListDLQResponseEntriesItemState> for &str {
    fn eq(&self, other: &JobsListDLQResponseEntriesItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsListDLQResponseEntriesItemTriggerKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsListDLQResponseEntriesItemTriggerKind {
    /// The `schedule` wire value.
    #[serde(rename = "schedule")]
    Schedule,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `manualReplay` wire value.
    #[serde(rename = "manualReplay")]
    ManualReplay,
    /// The `serviceCode` wire value.
    #[serde(rename = "serviceCode")]
    ServiceCode,
    /// The `parentJob` wire value.
    #[serde(rename = "parentJob")]
    ParentJob,
}
impl JobsListDLQResponseEntriesItemTriggerKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Schedule => "schedule",
            Self::Operation => "operation",
            Self::Rpc => "rpc",
            Self::Event => "event",
            Self::ManualReplay => "manualReplay",
            Self::ServiceCode => "serviceCode",
            Self::ParentJob => "parentJob",
        }
    }
}
impl AsRef<str> for JobsListDLQResponseEntriesItemTriggerKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsListDLQResponseEntriesItemTriggerKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsListDLQResponseEntriesItemTriggerKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsListDLQResponseEntriesItemTriggerKind> for &str {
    fn eq(&self, other: &JobsListDLQResponseEntriesItemTriggerKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsListDLQResponseEntriesItemTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemTrigger {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsListDLQResponseEntriesItemTriggerKind,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// The `subject` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemWaitingOnItemTargetKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsListDLQResponseEntriesItemWaitingOnItemTargetKind {
    /// The `job` wire value.
    #[serde(rename = "job")]
    Job,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `external` wire value.
    #[serde(rename = "external")]
    External,
}
impl JobsListDLQResponseEntriesItemWaitingOnItemTargetKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Operation => "operation",
            Self::External => "external",
        }
    }
}
impl AsRef<str> for JobsListDLQResponseEntriesItemWaitingOnItemTargetKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsListDLQResponseEntriesItemWaitingOnItemTargetKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsListDLQResponseEntriesItemWaitingOnItemTargetKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsListDLQResponseEntriesItemWaitingOnItemTargetKind> for &str {
    fn eq(&self, other: &JobsListDLQResponseEntriesItemWaitingOnItemTargetKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsListDLQResponseEntriesItemWaitingOnItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemWaitingOnItemTarget {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsListDLQResponseEntriesItemWaitingOnItemTargetKind,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `operation` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `system` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemWaitingOnItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemWaitingOnItem {
    /// The `id` wire field.
    pub id: String,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `target` wire field.
    pub target: JobsListDLQResponseEntriesItemWaitingOnItemTarget,
}
/// Generated schema type `JobsListDLQResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItem {
    /// The `completedAt` wire field.
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// The `concurrency` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsListDLQResponseEntriesItemConcurrency>,
    /// The `context` wire field.
    pub context: JobsListDLQResponseEntriesItemContext,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// The `deadline` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// The `errorDetail` wire field.
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsListDLQResponseEntriesItemErrorDetail>,
    /// The `id` wire field.
    pub id: String,
    /// The `lastError` wire field.
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// The `lineage` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsListDLQResponseEntriesItemLineage>,
    /// The `logs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsListDLQResponseEntriesItemLogsItem>>,
    /// The `maxTries` wire field.
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    /// The `payload` wire field.
    pub payload: Value,
    /// The `progress` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsListDLQResponseEntriesItemProgress>,
    /// The `queuePolicy` wire field.
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsListDLQResponseEntriesItemQueuePolicy>,
    /// The `result` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The `service` wire field.
    pub service: String,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// The `state` wire field.
    pub state: JobsListDLQResponseEntriesItemState,
    /// The `tries` wire field.
    pub tries: i64,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsListDLQResponseEntriesItemTrigger>,
    /// The `type` wire field.
    pub r#type: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `waitingOn` wire field.
    #[serde(rename = "waitingOn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_on: Option<Vec<JobsListDLQResponseEntriesItemWaitingOnItem>>,
}
/// Generated schema type `JobsListDLQResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponse {
    /// The `count` wire field.
    pub count: i64,
    /// The `entries` wire field.
    pub entries: Vec<JobsListDLQResponseEntriesItem>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `nextOffset` wire field.
    #[serde(rename = "nextOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    /// The `offset` wire field.
    pub offset: i64,
}
/// Generated schema type `JobsListServicesRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListServicesRequest {
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}
/// Generated schema type `JobsListServicesResponseEntriesItemWorkersItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListServicesResponseEntriesItemWorkersItem {
    /// The `concurrency` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<i64>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `jobType` wire field.
    #[serde(rename = "jobType")]
    pub job_type: String,
    /// The `service` wire field.
    pub service: String,
    /// The `timestamp` wire field.
    pub timestamp: String,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `JobsListServicesResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListServicesResponseEntriesItem {
    /// The `healthy` wire field.
    pub healthy: bool,
    /// The `name` wire field.
    pub name: String,
    /// The `workers` wire field.
    pub workers: Vec<JobsListServicesResponseEntriesItemWorkersItem>,
}
/// Generated schema type `JobsListServicesResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListServicesResponse {
    /// The `count` wire field.
    pub count: i64,
    /// The `entries` wire field.
    pub entries: Vec<JobsListServicesResponseEntriesItem>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `nextOffset` wire field.
    #[serde(rename = "nextOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    /// The `offset` wire field.
    pub offset: i64,
}
/// Generated schema type `JobsMetricsRequestGroupBy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsMetricsRequestGroupBy {
    /// The `type` wire value.
    #[serde(rename = "type")]
    Type,
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `queueKey` wire value.
    #[serde(rename = "queueKey")]
    QueueKey,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `trigger` wire value.
    #[serde(rename = "trigger")]
    Trigger,
}
impl JobsMetricsRequestGroupBy {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Service => "service",
            Self::QueueKey => "queueKey",
            Self::State => "state",
            Self::Trigger => "trigger",
        }
    }
}
impl AsRef<str> for JobsMetricsRequestGroupBy {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsMetricsRequestGroupBy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsMetricsRequestGroupBy {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsMetricsRequestGroupBy> for &str {
    fn eq(&self, other: &JobsMetricsRequestGroupBy) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsMetricsRequestStateItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsMetricsRequestStateItem {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsMetricsRequestStateItem {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsMetricsRequestStateItem {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsMetricsRequestStateItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsMetricsRequestStateItem {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsMetricsRequestStateItem> for &str {
    fn eq(&self, other: &JobsMetricsRequestStateItem) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsMetricsRequestStep`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsMetricsRequestStep {
    /// The `1m` wire value.
    #[serde(rename = "1m")]
    V1m,
    /// The `5m` wire value.
    #[serde(rename = "5m")]
    V5m,
    /// The `15m` wire value.
    #[serde(rename = "15m")]
    V15m,
    /// The `1h` wire value.
    #[serde(rename = "1h")]
    V1h,
    /// The `6h` wire value.
    #[serde(rename = "6h")]
    V6h,
    /// The `1d` wire value.
    #[serde(rename = "1d")]
    V1d,
}
impl JobsMetricsRequestStep {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::V1m => "1m",
            Self::V5m => "5m",
            Self::V15m => "15m",
            Self::V1h => "1h",
            Self::V6h => "6h",
            Self::V1d => "1d",
        }
    }
}
impl AsRef<str> for JobsMetricsRequestStep {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsMetricsRequestStep {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsMetricsRequestStep {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsMetricsRequestStep> for &str {
    fn eq(&self, other: &JobsMetricsRequestStep) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsMetricsRequestWindow`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsMetricsRequestWindow {
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
impl JobsMetricsRequestWindow {
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
impl AsRef<str> for JobsMetricsRequestWindow {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsMetricsRequestWindow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsMetricsRequestWindow {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsMetricsRequestWindow> for &str {
    fn eq(&self, other: &JobsMetricsRequestWindow) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsMetricsRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsMetricsRequest {
    /// The `groupBy` wire field.
    #[serde(rename = "groupBy")]
    pub group_by: JobsMetricsRequestGroupBy,
    /// The `queueKey` wire field.
    #[serde(rename = "queueKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_key: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<Vec<JobsMetricsRequestStateItem>>,
    /// The `step` wire field.
    pub step: JobsMetricsRequestStep,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `window` wire field.
    pub window: JobsMetricsRequestWindow,
}
/// Generated schema type `JobsMetricsResponseBucketsItemGroupsItemQueueWait`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsMetricsResponseBucketsItemGroupsItemQueueWait {
    /// The `count` wire field.
    pub count: i64,
    /// The `maxMs` wire field.
    #[serde(rename = "maxMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ms: Option<i64>,
    /// The `p50Ms` wire field.
    #[serde(rename = "p50Ms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p50_ms: Option<i64>,
    /// The `p95Ms` wire field.
    #[serde(rename = "p95Ms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p95_ms: Option<i64>,
}
/// Generated schema type `JobsMetricsResponseBucketsItemGroupsItemRuntime`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsMetricsResponseBucketsItemGroupsItemRuntime {
    /// The `count` wire field.
    pub count: i64,
    /// The `maxMs` wire field.
    #[serde(rename = "maxMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ms: Option<i64>,
    /// The `p50Ms` wire field.
    #[serde(rename = "p50Ms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p50_ms: Option<i64>,
    /// The `p95Ms` wire field.
    #[serde(rename = "p95Ms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p95_ms: Option<i64>,
}
/// Generated schema type `JobsMetricsResponseBucketsItemGroupsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsMetricsResponseBucketsItemGroupsItem {
    /// The `cancelled` wire field.
    pub cancelled: i64,
    /// The `completed` wire field.
    pub completed: i64,
    /// The `dead` wire field.
    pub dead: i64,
    /// The `dismissed` wire field.
    pub dismissed: i64,
    /// The `failed` wire field.
    pub failed: i64,
    /// The `key` wire field.
    pub key: String,
    /// The `label` wire field.
    pub label: String,
    /// The `queueWait` wire field.
    #[serde(rename = "queueWait")]
    pub queue_wait: JobsMetricsResponseBucketsItemGroupsItemQueueWait,
    /// The `retried` wire field.
    pub retried: i64,
    /// The `runtime` wire field.
    pub runtime: JobsMetricsResponseBucketsItemGroupsItemRuntime,
    /// The `started` wire field.
    pub started: i64,
    /// The `submitted` wire field.
    pub submitted: i64,
}
/// Generated schema type `JobsMetricsResponseBucketsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsMetricsResponseBucketsItem {
    /// The `end` wire field.
    pub end: String,
    /// The `groups` wire field.
    pub groups: Vec<JobsMetricsResponseBucketsItemGroupsItem>,
    /// The `start` wire field.
    pub start: String,
}
/// Generated schema type `JobsMetricsResponseSummaryItemQueueWait`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsMetricsResponseSummaryItemQueueWait {
    /// The `count` wire field.
    pub count: i64,
    /// The `maxMs` wire field.
    #[serde(rename = "maxMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ms: Option<i64>,
    /// The `p50Ms` wire field.
    #[serde(rename = "p50Ms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p50_ms: Option<i64>,
    /// The `p95Ms` wire field.
    #[serde(rename = "p95Ms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p95_ms: Option<i64>,
}
/// Generated schema type `JobsMetricsResponseSummaryItemRuntime`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsMetricsResponseSummaryItemRuntime {
    /// The `count` wire field.
    pub count: i64,
    /// The `maxMs` wire field.
    #[serde(rename = "maxMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ms: Option<i64>,
    /// The `p50Ms` wire field.
    #[serde(rename = "p50Ms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p50_ms: Option<i64>,
    /// The `p95Ms` wire field.
    #[serde(rename = "p95Ms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p95_ms: Option<i64>,
}
/// Generated schema type `JobsMetricsResponseSummaryItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsMetricsResponseSummaryItem {
    /// The `byState` wire field.
    #[serde(rename = "byState")]
    pub by_state: BTreeMap<String, i64>,
    /// The `dead` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dead: Option<i64>,
    /// The `failed` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<i64>,
    /// The `failureRate` wire field.
    #[serde(rename = "failureRate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_rate: Option<f64>,
    /// The `key` wire field.
    pub key: String,
    /// The `label` wire field.
    pub label: String,
    /// The `latestUpdatedAt` wire field.
    #[serde(rename = "latestUpdatedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_updated_at: Option<String>,
    /// The `oldestCreatedAt` wire field.
    #[serde(rename = "oldestCreatedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_created_at: Option<String>,
    /// The `queueWait` wire field.
    #[serde(rename = "queueWait")]
    pub queue_wait: JobsMetricsResponseSummaryItemQueueWait,
    /// The `queued` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queued: Option<i64>,
    /// The `running` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running: Option<i64>,
    /// The `runtime` wire field.
    pub runtime: JobsMetricsResponseSummaryItemRuntime,
    /// The `slow` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slow: Option<i64>,
    /// The `total` wire field.
    pub total: i64,
}
/// Generated schema type `JobsMetricsResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsMetricsResponse {
    /// The `buckets` wire field.
    pub buckets: Vec<JobsMetricsResponseBucketsItem>,
    /// The `generatedAt` wire field.
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
    /// The `groupBy` wire field.
    #[serde(rename = "groupBy")]
    pub group_by: String,
    /// The `step` wire field.
    pub step: String,
    /// The `summary` wire field.
    pub summary: Vec<JobsMetricsResponseSummaryItem>,
    /// The `window` wire field.
    pub window: String,
}
/// Generated schema type `JobsQueryRequestGroupBy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsQueryRequestGroupBy {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `type` wire value.
    #[serde(rename = "type")]
    Type,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `queueKey` wire value.
    #[serde(rename = "queueKey")]
    QueueKey,
    /// The `trigger` wire value.
    #[serde(rename = "trigger")]
    Trigger,
    /// The `runtimeBand` wire value.
    #[serde(rename = "runtimeBand")]
    RuntimeBand,
}
impl JobsQueryRequestGroupBy {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Type => "type",
            Self::State => "state",
            Self::QueueKey => "queueKey",
            Self::Trigger => "trigger",
            Self::RuntimeBand => "runtimeBand",
        }
    }
}
impl AsRef<str> for JobsQueryRequestGroupBy {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsQueryRequestGroupBy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsQueryRequestGroupBy {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsQueryRequestGroupBy> for &str {
    fn eq(&self, other: &JobsQueryRequestGroupBy) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsQueryRequestRuntimeBand`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsQueryRequestRuntimeBand {
    /// The `queued` wire value.
    #[serde(rename = "queued")]
    Queued,
    /// The `running` wire value.
    #[serde(rename = "running")]
    Running,
    /// The `slow` wire value.
    #[serde(rename = "slow")]
    Slow,
    /// The `terminal` wire value.
    #[serde(rename = "terminal")]
    Terminal,
}
impl JobsQueryRequestRuntimeBand {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Slow => "slow",
            Self::Terminal => "terminal",
        }
    }
}
impl AsRef<str> for JobsQueryRequestRuntimeBand {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsQueryRequestRuntimeBand {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsQueryRequestRuntimeBand {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsQueryRequestRuntimeBand> for &str {
    fn eq(&self, other: &JobsQueryRequestRuntimeBand) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsQueryRequestSortDirection`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsQueryRequestSortDirection {
    /// The `asc` wire value.
    #[serde(rename = "asc")]
    Asc,
    /// The `desc` wire value.
    #[serde(rename = "desc")]
    Desc,
}
impl JobsQueryRequestSortDirection {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}
impl AsRef<str> for JobsQueryRequestSortDirection {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsQueryRequestSortDirection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsQueryRequestSortDirection {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsQueryRequestSortDirection> for &str {
    fn eq(&self, other: &JobsQueryRequestSortDirection) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsQueryRequestSortField`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsQueryRequestSortField {
    /// The `updatedAt` wire value.
    #[serde(rename = "updatedAt")]
    UpdatedAt,
    /// The `queueAge` wire value.
    #[serde(rename = "queueAge")]
    QueueAge,
    /// The `runtime` wire value.
    #[serde(rename = "runtime")]
    Runtime,
    /// The `failureRate` wire value.
    #[serde(rename = "failureRate")]
    FailureRate,
    /// The `retries` wire value.
    #[serde(rename = "retries")]
    Retries,
    /// The `depth` wire value.
    #[serde(rename = "depth")]
    Depth,
}
impl JobsQueryRequestSortField {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::UpdatedAt => "updatedAt",
            Self::QueueAge => "queueAge",
            Self::Runtime => "runtime",
            Self::FailureRate => "failureRate",
            Self::Retries => "retries",
            Self::Depth => "depth",
        }
    }
}
impl AsRef<str> for JobsQueryRequestSortField {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsQueryRequestSortField {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsQueryRequestSortField {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsQueryRequestSortField> for &str {
    fn eq(&self, other: &JobsQueryRequestSortField) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsQueryRequestSort`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryRequestSort {
    /// The `direction` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<JobsQueryRequestSortDirection>,
    /// The `field` wire field.
    pub field: JobsQueryRequestSortField,
}
/// Generated schema type `JobsQueryRequestStateItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsQueryRequestStateItem {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsQueryRequestStateItem {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsQueryRequestStateItem {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsQueryRequestStateItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsQueryRequestStateItem {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsQueryRequestStateItem> for &str {
    fn eq(&self, other: &JobsQueryRequestStateItem) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsQueryRequestWindow`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsQueryRequestWindow {
    /// The `1h` wire value.
    #[serde(rename = "1h")]
    V1h,
    /// The `24h` wire value.
    #[serde(rename = "24h")]
    V24h,
    /// The `7d` wire value.
    #[serde(rename = "7d")]
    V7d,
}
impl JobsQueryRequestWindow {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::V1h => "1h",
            Self::V24h => "24h",
            Self::V7d => "7d",
        }
    }
}
impl AsRef<str> for JobsQueryRequestWindow {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsQueryRequestWindow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsQueryRequestWindow {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsQueryRequestWindow> for &str {
    fn eq(&self, other: &JobsQueryRequestWindow) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsQueryRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryRequest {
    /// The `groupBy` wire field.
    #[serde(rename = "groupBy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<JobsQueryRequestGroupBy>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// The `queueKey` wire field.
    #[serde(rename = "queueKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_key: Option<String>,
    /// The `runtimeBand` wire field.
    #[serde(rename = "runtimeBand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_band: Option<JobsQueryRequestRuntimeBand>,
    /// The `search` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `sort` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<JobsQueryRequestSort>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<Vec<JobsQueryRequestStateItem>>,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `window` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<JobsQueryRequestWindow>,
}
/// Generated schema type `JobsQueryResponseEntriesItemContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItemContext {
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    pub request_id: String,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    pub trace_id: String,
    /// The `traceparent` wire field.
    pub traceparent: String,
    /// The `tracestate` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsQueryResponseEntriesItemLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItemLineage {
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `relatedKeys` wire field.
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    /// The `rootJobId` wire field.
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsQueryResponseEntriesItemProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItemProgress {
    /// The `current` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    /// The `message` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The `step` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// The `total` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsQueryResponseEntriesItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsQueryResponseEntriesItemState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsQueryResponseEntriesItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsQueryResponseEntriesItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsQueryResponseEntriesItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsQueryResponseEntriesItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsQueryResponseEntriesItemState> for &str {
    fn eq(&self, other: &JobsQueryResponseEntriesItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsQueryResponseEntriesItemTriggerKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsQueryResponseEntriesItemTriggerKind {
    /// The `schedule` wire value.
    #[serde(rename = "schedule")]
    Schedule,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `manualReplay` wire value.
    #[serde(rename = "manualReplay")]
    ManualReplay,
    /// The `serviceCode` wire value.
    #[serde(rename = "serviceCode")]
    ServiceCode,
    /// The `parentJob` wire value.
    #[serde(rename = "parentJob")]
    ParentJob,
}
impl JobsQueryResponseEntriesItemTriggerKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Schedule => "schedule",
            Self::Operation => "operation",
            Self::Rpc => "rpc",
            Self::Event => "event",
            Self::ManualReplay => "manualReplay",
            Self::ServiceCode => "serviceCode",
            Self::ParentJob => "parentJob",
        }
    }
}
impl AsRef<str> for JobsQueryResponseEntriesItemTriggerKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsQueryResponseEntriesItemTriggerKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsQueryResponseEntriesItemTriggerKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsQueryResponseEntriesItemTriggerKind> for &str {
    fn eq(&self, other: &JobsQueryResponseEntriesItemTriggerKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsQueryResponseEntriesItemTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItemTrigger {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsQueryResponseEntriesItemTriggerKind,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// The `subject` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
/// Generated schema type `JobsQueryResponseEntriesItemWaitingOnItemTargetKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsQueryResponseEntriesItemWaitingOnItemTargetKind {
    /// The `job` wire value.
    #[serde(rename = "job")]
    Job,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `external` wire value.
    #[serde(rename = "external")]
    External,
}
impl JobsQueryResponseEntriesItemWaitingOnItemTargetKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Operation => "operation",
            Self::External => "external",
        }
    }
}
impl AsRef<str> for JobsQueryResponseEntriesItemWaitingOnItemTargetKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsQueryResponseEntriesItemWaitingOnItemTargetKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsQueryResponseEntriesItemWaitingOnItemTargetKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsQueryResponseEntriesItemWaitingOnItemTargetKind> for &str {
    fn eq(&self, other: &JobsQueryResponseEntriesItemWaitingOnItemTargetKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsQueryResponseEntriesItemWaitingOnItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItemWaitingOnItemTarget {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsQueryResponseEntriesItemWaitingOnItemTargetKind,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `operation` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `system` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}
/// Generated schema type `JobsQueryResponseEntriesItemWaitingOnItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItemWaitingOnItem {
    /// The `id` wire field.
    pub id: String,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `target` wire field.
    pub target: JobsQueryResponseEntriesItemWaitingOnItemTarget,
}
/// Generated schema type `JobsQueryResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItem {
    /// The `completedAt` wire field.
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// The `context` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<JobsQueryResponseEntriesItemContext>,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// The `errorFingerprint` wire field.
    #[serde(rename = "errorFingerprint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_fingerprint: Option<String>,
    /// The `id` wire field.
    pub id: String,
    /// The `lastError` wire field.
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// The `lineage` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsQueryResponseEntriesItemLineage>,
    /// The `maxTries` wire field.
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    /// The `progress` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsQueryResponseEntriesItemProgress>,
    /// The `queueAgeMs` wire field.
    #[serde(rename = "queueAgeMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_age_ms: Option<i64>,
    /// The `queueKey` wire field.
    #[serde(rename = "queueKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_key: Option<String>,
    /// The `runtimeBand` wire field.
    #[serde(rename = "runtimeBand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_band: Option<String>,
    /// The `runtimeMs` wire field.
    #[serde(rename = "runtimeMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_ms: Option<i64>,
    /// The `service` wire field.
    pub service: String,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// The `state` wire field.
    pub state: JobsQueryResponseEntriesItemState,
    /// The `tries` wire field.
    pub tries: i64,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsQueryResponseEntriesItemTrigger>,
    /// The `type` wire field.
    pub r#type: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `waitingOn` wire field.
    #[serde(rename = "waitingOn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_on: Option<Vec<JobsQueryResponseEntriesItemWaitingOnItem>>,
}
/// Generated schema type `JobsQueryResponseGroupsItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsQueryResponseGroupsItemState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsQueryResponseGroupsItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsQueryResponseGroupsItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsQueryResponseGroupsItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsQueryResponseGroupsItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsQueryResponseGroupsItemState> for &str {
    fn eq(&self, other: &JobsQueryResponseGroupsItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsQueryResponseGroupsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseGroupsItem {
    /// The `count` wire field.
    pub count: i64,
    /// The `depth` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<i64>,
    /// The `failureRate` wire field.
    #[serde(rename = "failureRate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_rate: Option<f64>,
    /// The `key` wire field.
    pub key: String,
    /// The `label` wire field.
    pub label: String,
    /// The `latestUpdatedAt` wire field.
    #[serde(rename = "latestUpdatedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_updated_at: Option<String>,
    /// The `oldestCreatedAt` wire field.
    #[serde(rename = "oldestCreatedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_created_at: Option<String>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<JobsQueryResponseGroupsItemState>,
}
/// Generated schema type `JobsQueryResponseStats`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseStats {
    /// The `byState` wire field.
    #[serde(rename = "byState")]
    pub by_state: BTreeMap<String, i64>,
    /// The `dead` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dead: Option<i64>,
    /// The `failed` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<i64>,
    /// The `queued` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queued: Option<i64>,
    /// The `running` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running: Option<i64>,
    /// The `slow` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slow: Option<i64>,
    /// The `total` wire field.
    pub total: i64,
}
/// Generated schema type `JobsQueryResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponse {
    /// The `count` wire field.
    pub count: i64,
    /// The `entries` wire field.
    pub entries: Vec<JobsQueryResponseEntriesItem>,
    /// The `groups` wire field.
    pub groups: Vec<JobsQueryResponseGroupsItem>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `nextOffset` wire field.
    #[serde(rename = "nextOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    /// The `offset` wire field.
    pub offset: i64,
    /// The `stats` wire field.
    pub stats: JobsQueryResponseStats,
}
/// Generated schema type `JobsReplayDLQRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQRequest {
    /// The `id` wire field.
    pub id: String,
    /// The `reason` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
/// Generated schema type `JobsReplayDLQResponseJobConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobConcurrency {
    /// The `heartbeatAt` wire field.
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    /// The `key` wire field.
    pub key: String,
    /// The `keyHash` wire field.
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    /// The `leaseExpiresAt` wire field.
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    /// The `staleTakeoverCount` wire field.
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsReplayDLQResponseJobContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobContext {
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    pub request_id: String,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    pub trace_id: String,
    /// The `traceparent` wire field.
    pub traceparent: String,
    /// The `tracestate` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsReplayDLQResponseJobErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobErrorDetailWorker {
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The `runtime` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `JobsReplayDLQResponseJobErrorDetail`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobErrorDetail {
    /// The `causes` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    /// The `fingerprint` wire field.
    pub fingerprint: String,
    /// The `firstSeen` wire field.
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    /// The `message` wire field.
    pub message: String,
    /// The `occurrenceCount` wire field.
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    /// The `stack` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `worker` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsReplayDLQResponseJobErrorDetailWorker>,
}
/// Generated schema type `JobsReplayDLQResponseJobLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobLineage {
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `relatedKeys` wire field.
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    /// The `rootJobId` wire field.
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsReplayDLQResponseJobLogsItemLevel`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsReplayDLQResponseJobLogsItemLevel {
    /// The `info` wire value.
    #[serde(rename = "info")]
    Info,
    /// The `warn` wire value.
    #[serde(rename = "warn")]
    Warn,
    /// The `error` wire value.
    #[serde(rename = "error")]
    Error,
}
impl JobsReplayDLQResponseJobLogsItemLevel {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
impl AsRef<str> for JobsReplayDLQResponseJobLogsItemLevel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsReplayDLQResponseJobLogsItemLevel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsReplayDLQResponseJobLogsItemLevel {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsReplayDLQResponseJobLogsItemLevel> for &str {
    fn eq(&self, other: &JobsReplayDLQResponseJobLogsItemLevel) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsReplayDLQResponseJobLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobLogsItem {
    /// The `level` wire field.
    pub level: JobsReplayDLQResponseJobLogsItemLevel,
    /// The `message` wire field.
    pub message: String,
    /// The `timestamp` wire field.
    pub timestamp: String,
}
/// Generated schema type `JobsReplayDLQResponseJobProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobProgress {
    /// The `current` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    /// The `message` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The `step` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// The `total` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsReplayDLQResponseJobQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobQueuePolicy {
    /// The `existingJobId` wire field.
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    /// The `outcome` wire field.
    pub outcome: String,
    /// The `reason` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The `replacedJobId` wire field.
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsReplayDLQResponseJobState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsReplayDLQResponseJobState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsReplayDLQResponseJobState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsReplayDLQResponseJobState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsReplayDLQResponseJobState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsReplayDLQResponseJobState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsReplayDLQResponseJobState> for &str {
    fn eq(&self, other: &JobsReplayDLQResponseJobState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsReplayDLQResponseJobTriggerKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsReplayDLQResponseJobTriggerKind {
    /// The `schedule` wire value.
    #[serde(rename = "schedule")]
    Schedule,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `manualReplay` wire value.
    #[serde(rename = "manualReplay")]
    ManualReplay,
    /// The `serviceCode` wire value.
    #[serde(rename = "serviceCode")]
    ServiceCode,
    /// The `parentJob` wire value.
    #[serde(rename = "parentJob")]
    ParentJob,
}
impl JobsReplayDLQResponseJobTriggerKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Schedule => "schedule",
            Self::Operation => "operation",
            Self::Rpc => "rpc",
            Self::Event => "event",
            Self::ManualReplay => "manualReplay",
            Self::ServiceCode => "serviceCode",
            Self::ParentJob => "parentJob",
        }
    }
}
impl AsRef<str> for JobsReplayDLQResponseJobTriggerKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsReplayDLQResponseJobTriggerKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsReplayDLQResponseJobTriggerKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsReplayDLQResponseJobTriggerKind> for &str {
    fn eq(&self, other: &JobsReplayDLQResponseJobTriggerKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsReplayDLQResponseJobTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobTrigger {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsReplayDLQResponseJobTriggerKind,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// The `subject` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
/// Generated schema type `JobsReplayDLQResponseJobWaitingOnItemTargetKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsReplayDLQResponseJobWaitingOnItemTargetKind {
    /// The `job` wire value.
    #[serde(rename = "job")]
    Job,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `external` wire value.
    #[serde(rename = "external")]
    External,
}
impl JobsReplayDLQResponseJobWaitingOnItemTargetKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Operation => "operation",
            Self::External => "external",
        }
    }
}
impl AsRef<str> for JobsReplayDLQResponseJobWaitingOnItemTargetKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsReplayDLQResponseJobWaitingOnItemTargetKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsReplayDLQResponseJobWaitingOnItemTargetKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsReplayDLQResponseJobWaitingOnItemTargetKind> for &str {
    fn eq(&self, other: &JobsReplayDLQResponseJobWaitingOnItemTargetKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsReplayDLQResponseJobWaitingOnItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobWaitingOnItemTarget {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsReplayDLQResponseJobWaitingOnItemTargetKind,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `operation` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `system` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}
/// Generated schema type `JobsReplayDLQResponseJobWaitingOnItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobWaitingOnItem {
    /// The `id` wire field.
    pub id: String,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `target` wire field.
    pub target: JobsReplayDLQResponseJobWaitingOnItemTarget,
}
/// Generated schema type `JobsReplayDLQResponseJob`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJob {
    /// The `completedAt` wire field.
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// The `concurrency` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsReplayDLQResponseJobConcurrency>,
    /// The `context` wire field.
    pub context: JobsReplayDLQResponseJobContext,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// The `deadline` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// The `errorDetail` wire field.
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsReplayDLQResponseJobErrorDetail>,
    /// The `id` wire field.
    pub id: String,
    /// The `lastError` wire field.
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// The `lineage` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsReplayDLQResponseJobLineage>,
    /// The `logs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsReplayDLQResponseJobLogsItem>>,
    /// The `maxTries` wire field.
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    /// The `payload` wire field.
    pub payload: Value,
    /// The `progress` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsReplayDLQResponseJobProgress>,
    /// The `queuePolicy` wire field.
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsReplayDLQResponseJobQueuePolicy>,
    /// The `result` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The `service` wire field.
    pub service: String,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// The `state` wire field.
    pub state: JobsReplayDLQResponseJobState,
    /// The `tries` wire field.
    pub tries: i64,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsReplayDLQResponseJobTrigger>,
    /// The `type` wire field.
    pub r#type: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `waitingOn` wire field.
    #[serde(rename = "waitingOn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_on: Option<Vec<JobsReplayDLQResponseJobWaitingOnItem>>,
}
/// Generated schema type `JobsReplayDLQResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponse {
    /// The `job` wire field.
    pub job: JobsReplayDLQResponseJob,
}
/// Generated schema type `JobsRetryRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryRequest {
    /// The `id` wire field.
    pub id: String,
    /// The `reason` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
/// Generated schema type `JobsRetryResponseJobConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobConcurrency {
    /// The `heartbeatAt` wire field.
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    /// The `key` wire field.
    pub key: String,
    /// The `keyHash` wire field.
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    /// The `leaseExpiresAt` wire field.
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    /// The `staleTakeoverCount` wire field.
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsRetryResponseJobContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobContext {
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    pub request_id: String,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    pub trace_id: String,
    /// The `traceparent` wire field.
    pub traceparent: String,
    /// The `tracestate` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsRetryResponseJobErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobErrorDetailWorker {
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The `runtime` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `JobsRetryResponseJobErrorDetail`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobErrorDetail {
    /// The `causes` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    /// The `fingerprint` wire field.
    pub fingerprint: String,
    /// The `firstSeen` wire field.
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    /// The `message` wire field.
    pub message: String,
    /// The `occurrenceCount` wire field.
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    /// The `stack` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `worker` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsRetryResponseJobErrorDetailWorker>,
}
/// Generated schema type `JobsRetryResponseJobLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobLineage {
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `relatedKeys` wire field.
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    /// The `rootJobId` wire field.
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsRetryResponseJobLogsItemLevel`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsRetryResponseJobLogsItemLevel {
    /// The `info` wire value.
    #[serde(rename = "info")]
    Info,
    /// The `warn` wire value.
    #[serde(rename = "warn")]
    Warn,
    /// The `error` wire value.
    #[serde(rename = "error")]
    Error,
}
impl JobsRetryResponseJobLogsItemLevel {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
impl AsRef<str> for JobsRetryResponseJobLogsItemLevel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsRetryResponseJobLogsItemLevel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsRetryResponseJobLogsItemLevel {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsRetryResponseJobLogsItemLevel> for &str {
    fn eq(&self, other: &JobsRetryResponseJobLogsItemLevel) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsRetryResponseJobLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobLogsItem {
    /// The `level` wire field.
    pub level: JobsRetryResponseJobLogsItemLevel,
    /// The `message` wire field.
    pub message: String,
    /// The `timestamp` wire field.
    pub timestamp: String,
}
/// Generated schema type `JobsRetryResponseJobProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobProgress {
    /// The `current` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    /// The `message` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The `step` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// The `total` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsRetryResponseJobQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobQueuePolicy {
    /// The `existingJobId` wire field.
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    /// The `outcome` wire field.
    pub outcome: String,
    /// The `reason` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The `replacedJobId` wire field.
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsRetryResponseJobState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsRetryResponseJobState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsRetryResponseJobState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsRetryResponseJobState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsRetryResponseJobState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsRetryResponseJobState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsRetryResponseJobState> for &str {
    fn eq(&self, other: &JobsRetryResponseJobState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsRetryResponseJobTriggerKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsRetryResponseJobTriggerKind {
    /// The `schedule` wire value.
    #[serde(rename = "schedule")]
    Schedule,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `manualReplay` wire value.
    #[serde(rename = "manualReplay")]
    ManualReplay,
    /// The `serviceCode` wire value.
    #[serde(rename = "serviceCode")]
    ServiceCode,
    /// The `parentJob` wire value.
    #[serde(rename = "parentJob")]
    ParentJob,
}
impl JobsRetryResponseJobTriggerKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Schedule => "schedule",
            Self::Operation => "operation",
            Self::Rpc => "rpc",
            Self::Event => "event",
            Self::ManualReplay => "manualReplay",
            Self::ServiceCode => "serviceCode",
            Self::ParentJob => "parentJob",
        }
    }
}
impl AsRef<str> for JobsRetryResponseJobTriggerKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsRetryResponseJobTriggerKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsRetryResponseJobTriggerKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsRetryResponseJobTriggerKind> for &str {
    fn eq(&self, other: &JobsRetryResponseJobTriggerKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsRetryResponseJobTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobTrigger {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsRetryResponseJobTriggerKind,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `parentJobId` wire field.
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// The `subject` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
/// Generated schema type `JobsRetryResponseJobWaitingOnItemTargetKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsRetryResponseJobWaitingOnItemTargetKind {
    /// The `job` wire value.
    #[serde(rename = "job")]
    Job,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `external` wire value.
    #[serde(rename = "external")]
    External,
}
impl JobsRetryResponseJobWaitingOnItemTargetKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Operation => "operation",
            Self::External => "external",
        }
    }
}
impl AsRef<str> for JobsRetryResponseJobWaitingOnItemTargetKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsRetryResponseJobWaitingOnItemTargetKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsRetryResponseJobWaitingOnItemTargetKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsRetryResponseJobWaitingOnItemTargetKind> for &str {
    fn eq(&self, other: &JobsRetryResponseJobWaitingOnItemTargetKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsRetryResponseJobWaitingOnItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobWaitingOnItemTarget {
    /// The `id` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `kind` wire field.
    pub kind: JobsRetryResponseJobWaitingOnItemTargetKind,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `operation` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// The `operationId` wire field.
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `system` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}
/// Generated schema type `JobsRetryResponseJobWaitingOnItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobWaitingOnItem {
    /// The `id` wire field.
    pub id: String,
    /// The `label` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `target` wire field.
    pub target: JobsRetryResponseJobWaitingOnItemTarget,
}
/// Generated schema type `JobsRetryResponseJob`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJob {
    /// The `completedAt` wire field.
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// The `concurrency` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsRetryResponseJobConcurrency>,
    /// The `context` wire field.
    pub context: JobsRetryResponseJobContext,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// The `deadline` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// The `errorDetail` wire field.
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsRetryResponseJobErrorDetail>,
    /// The `id` wire field.
    pub id: String,
    /// The `lastError` wire field.
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// The `lineage` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsRetryResponseJobLineage>,
    /// The `logs` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsRetryResponseJobLogsItem>>,
    /// The `maxTries` wire field.
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    /// The `payload` wire field.
    pub payload: Value,
    /// The `progress` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsRetryResponseJobProgress>,
    /// The `queuePolicy` wire field.
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsRetryResponseJobQueuePolicy>,
    /// The `result` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The `service` wire field.
    pub service: String,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// The `state` wire field.
    pub state: JobsRetryResponseJobState,
    /// The `tries` wire field.
    pub tries: i64,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsRetryResponseJobTrigger>,
    /// The `type` wire field.
    pub r#type: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `waitingOn` wire field.
    #[serde(rename = "waitingOn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_on: Option<Vec<JobsRetryResponseJobWaitingOnItem>>,
}
/// Generated schema type `JobsRetryResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponse {
    /// The `job` wire field.
    pub job: JobsRetryResponseJob,
}
/// Generated schema type `JobsWatchInputQueryGroupBy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsWatchInputQueryGroupBy {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `type` wire value.
    #[serde(rename = "type")]
    Type,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `queueKey` wire value.
    #[serde(rename = "queueKey")]
    QueueKey,
    /// The `trigger` wire value.
    #[serde(rename = "trigger")]
    Trigger,
    /// The `runtimeBand` wire value.
    #[serde(rename = "runtimeBand")]
    RuntimeBand,
}
impl JobsWatchInputQueryGroupBy {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Type => "type",
            Self::State => "state",
            Self::QueueKey => "queueKey",
            Self::Trigger => "trigger",
            Self::RuntimeBand => "runtimeBand",
        }
    }
}
impl AsRef<str> for JobsWatchInputQueryGroupBy {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsWatchInputQueryGroupBy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsWatchInputQueryGroupBy {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsWatchInputQueryGroupBy> for &str {
    fn eq(&self, other: &JobsWatchInputQueryGroupBy) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsWatchInputQueryRuntimeBand`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsWatchInputQueryRuntimeBand {
    /// The `queued` wire value.
    #[serde(rename = "queued")]
    Queued,
    /// The `running` wire value.
    #[serde(rename = "running")]
    Running,
    /// The `slow` wire value.
    #[serde(rename = "slow")]
    Slow,
    /// The `terminal` wire value.
    #[serde(rename = "terminal")]
    Terminal,
}
impl JobsWatchInputQueryRuntimeBand {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Slow => "slow",
            Self::Terminal => "terminal",
        }
    }
}
impl AsRef<str> for JobsWatchInputQueryRuntimeBand {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsWatchInputQueryRuntimeBand {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsWatchInputQueryRuntimeBand {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsWatchInputQueryRuntimeBand> for &str {
    fn eq(&self, other: &JobsWatchInputQueryRuntimeBand) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsWatchInputQuerySortDirection`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsWatchInputQuerySortDirection {
    /// The `asc` wire value.
    #[serde(rename = "asc")]
    Asc,
    /// The `desc` wire value.
    #[serde(rename = "desc")]
    Desc,
}
impl JobsWatchInputQuerySortDirection {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}
impl AsRef<str> for JobsWatchInputQuerySortDirection {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsWatchInputQuerySortDirection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsWatchInputQuerySortDirection {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsWatchInputQuerySortDirection> for &str {
    fn eq(&self, other: &JobsWatchInputQuerySortDirection) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsWatchInputQuerySortField`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsWatchInputQuerySortField {
    /// The `updatedAt` wire value.
    #[serde(rename = "updatedAt")]
    UpdatedAt,
    /// The `queueAge` wire value.
    #[serde(rename = "queueAge")]
    QueueAge,
    /// The `runtime` wire value.
    #[serde(rename = "runtime")]
    Runtime,
    /// The `failureRate` wire value.
    #[serde(rename = "failureRate")]
    FailureRate,
    /// The `retries` wire value.
    #[serde(rename = "retries")]
    Retries,
    /// The `depth` wire value.
    #[serde(rename = "depth")]
    Depth,
}
impl JobsWatchInputQuerySortField {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::UpdatedAt => "updatedAt",
            Self::QueueAge => "queueAge",
            Self::Runtime => "runtime",
            Self::FailureRate => "failureRate",
            Self::Retries => "retries",
            Self::Depth => "depth",
        }
    }
}
impl AsRef<str> for JobsWatchInputQuerySortField {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsWatchInputQuerySortField {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsWatchInputQuerySortField {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsWatchInputQuerySortField> for &str {
    fn eq(&self, other: &JobsWatchInputQuerySortField) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsWatchInputQuerySort`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsWatchInputQuerySort {
    /// The `direction` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<JobsWatchInputQuerySortDirection>,
    /// The `field` wire field.
    pub field: JobsWatchInputQuerySortField,
}
/// Generated schema type `JobsWatchInputQueryStateItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsWatchInputQueryStateItem {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsWatchInputQueryStateItem {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsWatchInputQueryStateItem {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsWatchInputQueryStateItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsWatchInputQueryStateItem {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsWatchInputQueryStateItem> for &str {
    fn eq(&self, other: &JobsWatchInputQueryStateItem) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsWatchInputQueryWindow`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsWatchInputQueryWindow {
    /// The `1h` wire value.
    #[serde(rename = "1h")]
    V1h,
    /// The `24h` wire value.
    #[serde(rename = "24h")]
    V24h,
    /// The `7d` wire value.
    #[serde(rename = "7d")]
    V7d,
}
impl JobsWatchInputQueryWindow {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::V1h => "1h",
            Self::V24h => "24h",
            Self::V7d => "7d",
        }
    }
}
impl AsRef<str> for JobsWatchInputQueryWindow {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsWatchInputQueryWindow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsWatchInputQueryWindow {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsWatchInputQueryWindow> for &str {
    fn eq(&self, other: &JobsWatchInputQueryWindow) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsWatchInputQuery`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsWatchInputQuery {
    /// The `groupBy` wire field.
    #[serde(rename = "groupBy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<JobsWatchInputQueryGroupBy>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// The `queueKey` wire field.
    #[serde(rename = "queueKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_key: Option<String>,
    /// The `runtimeBand` wire field.
    #[serde(rename = "runtimeBand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_band: Option<JobsWatchInputQueryRuntimeBand>,
    /// The `search` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    /// The `service` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The `sort` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<JobsWatchInputQuerySort>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<Vec<JobsWatchInputQueryStateItem>>,
    /// The `trigger` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    /// The `type` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The `window` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<JobsWatchInputQueryWindow>,
}
/// Generated schema type `JobsWatchInput`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsWatchInput {
    /// The `includeInitial` wire field.
    #[serde(rename = "includeInitial")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_initial: Option<bool>,
    /// The `jobId` wire field.
    #[serde(rename = "jobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    /// The `query` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<JobsWatchInputQuery>,
}
/// Generated schema type `JobsWatchEventJobChangedState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsWatchEventJobChangedState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `retry` wire value.
    #[serde(rename = "retry")]
    Retry,
    /// The `completed` wire value.
    #[serde(rename = "completed")]
    Completed,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
    /// The `cancelled` wire value.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// The `skipped` wire value.
    #[serde(rename = "skipped")]
    Skipped,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `dead` wire value.
    #[serde(rename = "dead")]
    Dead,
    /// The `dismissed` wire value.
    #[serde(rename = "dismissed")]
    Dismissed,
}
impl JobsWatchEventJobChangedState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Retry => "retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}
impl AsRef<str> for JobsWatchEventJobChangedState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsWatchEventJobChangedState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsWatchEventJobChangedState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsWatchEventJobChangedState> for &str {
    fn eq(&self, other: &JobsWatchEventJobChangedState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsWatchEventQueryInvalidatedReason`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobsWatchEventQueryInvalidatedReason {
    /// The `matched-job-changed` wire value.
    #[serde(rename = "matched-job-changed")]
    MatchedJobChanged,
    /// The `unknown-match` wire value.
    #[serde(rename = "unknown-match")]
    UnknownMatch,
}
impl JobsWatchEventQueryInvalidatedReason {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MatchedJobChanged => "matched-job-changed",
            Self::UnknownMatch => "unknown-match",
        }
    }
}
impl AsRef<str> for JobsWatchEventQueryInvalidatedReason {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for JobsWatchEventQueryInvalidatedReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for JobsWatchEventQueryInvalidatedReason {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<JobsWatchEventQueryInvalidatedReason> for &str {
    fn eq(&self, other: &JobsWatchEventQueryInvalidatedReason) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `JobsWatchEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum JobsWatchEvent {
    /// The `ready` variant.
    #[serde(rename = "ready")]
    Ready {
        /// The `timestamp` wire field.
        timestamp: String,
    },
    /// The `jobChanged` variant.
    #[serde(rename = "jobChanged")]
    JobChanged {
        /// The `id` wire field.
        id: String,
        /// The `service` wire field.
        service: String,
        /// The `state` wire field.
        state: JobsWatchEventJobChangedState,
        /// The `type` wire field.
        r#type: String,
        /// The `updatedAt` wire field.
        #[serde(rename = "updatedAt")]
        updated_at: String,
    },
    /// The `queryInvalidated` variant.
    #[serde(rename = "queryInvalidated")]
    QueryInvalidated {
        /// The `reason` wire field.
        reason: JobsWatchEventQueryInvalidatedReason,
        /// The `timestamp` wire field.
        timestamp: String,
    },
    /// The `jobInspectChanged` variant.
    #[serde(rename = "jobInspectChanged")]
    JobInspectChanged {
        /// The `id` wire field.
        id: String,
        /// The `timestamp` wire field.
        timestamp: String,
    },
}
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
    /// The `jobId` wire field.
    #[serde(rename = "jobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    /// The `message` wire field.
    pub message: String,
    /// The `resource` wire field.
    pub resource: String,
    /// The `traceId` wire field.
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// The `type` wire field.
    pub r#type: NotFoundErrorDataType,
}
