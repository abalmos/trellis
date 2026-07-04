//! Shared request and response types for `trellis.jobs@v1`.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
/// Generated schema type `JobsCancelRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelRequest {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
/// Generated schema type `JobsCancelResponse`.
/// Generated schema type `JobsCancelResponseJob`.
/// Generated schema type `JobsCancelResponseJobConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobConcurrency {
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    pub key: String,
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsCancelResponseJobContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobContext {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "traceId")]
    pub trace_id: String,
    pub traceparent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsCancelResponseJobErrorDetail`.
/// Generated schema type `JobsCancelResponseJobErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobErrorDetailWorker {
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobErrorDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    pub fingerprint: String,
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    pub message: String,
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsCancelResponseJobErrorDetailWorker>,
}
/// Generated schema type `JobsCancelResponseJobLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobLineage {
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsCancelResponseJobLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobLogsItem {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}
/// Generated schema type `JobsCancelResponseJobProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsCancelResponseJobQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobQueuePolicy {
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsCancelResponseJobTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJobTrigger {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: String,
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponseJob {
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsCancelResponseJobConcurrency>,
    pub context: JobsCancelResponseJobContext,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsCancelResponseJobErrorDetail>,
    pub id: String,
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsCancelResponseJobLineage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsCancelResponseJobLogsItem>>,
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    pub payload: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsCancelResponseJobProgress>,
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsCancelResponseJobQueuePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    pub service: String,
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub state: String,
    pub tries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsCancelResponseJobTrigger>,
    pub r#type: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsCancelResponse {
    pub job: JobsCancelResponseJob,
}
/// Generated schema type `JobsDismissDLQRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQRequest {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
/// Generated schema type `JobsDismissDLQResponse`.
/// Generated schema type `JobsDismissDLQResponseJob`.
/// Generated schema type `JobsDismissDLQResponseJobConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobConcurrency {
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    pub key: String,
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsDismissDLQResponseJobContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobContext {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "traceId")]
    pub trace_id: String,
    pub traceparent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsDismissDLQResponseJobErrorDetail`.
/// Generated schema type `JobsDismissDLQResponseJobErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobErrorDetailWorker {
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobErrorDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    pub fingerprint: String,
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    pub message: String,
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsDismissDLQResponseJobErrorDetailWorker>,
}
/// Generated schema type `JobsDismissDLQResponseJobLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobLineage {
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsDismissDLQResponseJobLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobLogsItem {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}
/// Generated schema type `JobsDismissDLQResponseJobProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsDismissDLQResponseJobQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobQueuePolicy {
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsDismissDLQResponseJobTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJobTrigger {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: String,
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponseJob {
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsDismissDLQResponseJobConcurrency>,
    pub context: JobsDismissDLQResponseJobContext,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsDismissDLQResponseJobErrorDetail>,
    pub id: String,
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsDismissDLQResponseJobLineage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsDismissDLQResponseJobLogsItem>>,
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    pub payload: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsDismissDLQResponseJobProgress>,
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsDismissDLQResponseJobQueuePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    pub service: String,
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub state: String,
    pub tries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsDismissDLQResponseJobTrigger>,
    pub r#type: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsDismissDLQResponse {
    pub job: JobsDismissDLQResponseJob,
}
/// Generated schema type `JobsGetKeyRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsGetKeyRequest {
    pub key: String,
    pub service: String,
    pub r#type: String,
}
/// Generated schema type `JobsGetKeyResponse`.
/// Generated schema type `JobsGetKeyResponseActiveItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsGetKeyResponseActiveItem {
    #[serde(rename = "heartbeatAgeMs")]
    pub heartbeat_age_ms: i64,
    #[serde(rename = "heartbeatAt")]
    pub heartbeat_at: String,
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "jobId")]
    pub job_id: String,
    #[serde(rename = "leaseExpiresAt")]
    pub lease_expires_at: String,
    #[serde(rename = "startedAt")]
    pub started_at: String,
}
/// Generated schema type `JobsGetKeyResponseQueuedItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsGetKeyResponseQueuedItem {
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "jobId")]
    pub job_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsGetKeyResponse {
    pub active: Vec<JobsGetKeyResponseActiveItem>,
    pub key: String,
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    #[serde(rename = "latestPolicyReason")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_policy_reason: Option<String>,
    pub queued: Vec<JobsGetKeyResponseQueuedItem>,
    #[serde(rename = "queuedDepth")]
    pub queued_depth: i64,
    pub service: String,
    #[serde(rename = "staleTakeoverCount")]
    pub stale_takeover_count: i64,
    pub r#type: String,
}
/// Generated schema type `JobsInspectRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectRequest {
    pub id: String,
}
/// Generated schema type `JobsInspectResponse`.
/// Generated schema type `JobsInspectResponseAttemptsItem`.
/// Generated schema type `JobsInspectResponseAttemptsItemError`.
/// Generated schema type `JobsInspectResponseAttemptsItemErrorWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseAttemptsItemErrorWorker {
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseAttemptsItemError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    pub fingerprint: String,
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    pub message: String,
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsInspectResponseAttemptsItemErrorWorker>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseAttemptsItem {
    #[serde(rename = "endedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JobsInspectResponseAttemptsItemError>,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub r#try: i64,
}
/// Generated schema type `JobsInspectResponseErrorsItem`.
/// Generated schema type `JobsInspectResponseErrorsItemWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseErrorsItemWorker {
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseErrorsItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    pub fingerprint: String,
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    pub message: String,
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsInspectResponseErrorsItemWorker>,
}
/// Generated schema type `JobsInspectResponseJob`.
/// Generated schema type `JobsInspectResponseJobConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobConcurrency {
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    pub key: String,
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsInspectResponseJobContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobContext {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "traceId")]
    pub trace_id: String,
    pub traceparent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsInspectResponseJobErrorDetail`.
/// Generated schema type `JobsInspectResponseJobErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobErrorDetailWorker {
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobErrorDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    pub fingerprint: String,
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    pub message: String,
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsInspectResponseJobErrorDetailWorker>,
}
/// Generated schema type `JobsInspectResponseJobLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobLineage {
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseJobLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobLogsItem {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}
/// Generated schema type `JobsInspectResponseJobProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsInspectResponseJobQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobQueuePolicy {
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseJobTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJobTrigger {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: String,
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseJob {
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsInspectResponseJobConcurrency>,
    pub context: JobsInspectResponseJobContext,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsInspectResponseJobErrorDetail>,
    pub id: String,
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsInspectResponseJobLineage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsInspectResponseJobLogsItem>>,
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    pub payload: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsInspectResponseJobProgress>,
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsInspectResponseJobQueuePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    pub service: String,
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub state: String,
    pub tries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsInspectResponseJobTrigger>,
    pub r#type: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
/// Generated schema type `JobsInspectResponseLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseLineage {
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseRelatedItem`.
/// Generated schema type `JobsInspectResponseRelatedItemContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItemContext {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "traceId")]
    pub trace_id: String,
    pub traceparent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsInspectResponseRelatedItemLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItemLineage {
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseRelatedItemProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItemProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsInspectResponseRelatedItemTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItemTrigger {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: String,
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseRelatedItem {
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<JobsInspectResponseRelatedItemContext>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "errorFingerprint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_fingerprint: Option<String>,
    pub id: String,
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsInspectResponseRelatedItemLineage>,
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsInspectResponseRelatedItemProgress>,
    #[serde(rename = "queueAgeMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_age_ms: Option<i64>,
    #[serde(rename = "queueKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_key: Option<String>,
    #[serde(rename = "runtimeBand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_band: Option<String>,
    #[serde(rename = "runtimeMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_ms: Option<i64>,
    pub service: String,
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub state: String,
    pub tries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsInspectResponseRelatedItemTrigger>,
    pub r#type: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
/// Generated schema type `JobsInspectResponseTimelineItem`.
/// Generated schema type `JobsInspectResponseTimelineItemErrorDetail`.
/// Generated schema type `JobsInspectResponseTimelineItemErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItemErrorDetailWorker {
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItemErrorDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    pub fingerprint: String,
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    pub message: String,
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsInspectResponseTimelineItemErrorDetailWorker>,
}
/// Generated schema type `JobsInspectResponseTimelineItemLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItemLogsItem {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}
/// Generated schema type `JobsInspectResponseTimelineItemProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItemProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTimelineItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsInspectResponseTimelineItemErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsInspectResponseTimelineItemLogsItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(rename = "previousState")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsInspectResponseTimelineItemProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projected: Option<bool>,
    #[serde(rename = "rawEvent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_event: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub sequence: i64,
    pub state: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tries: Option<i64>,
    pub r#type: String,
    #[serde(rename = "workerInstanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_instance_id: Option<String>,
}
/// Generated schema type `JobsInspectResponseTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponseTrigger {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: String,
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsInspectResponse {
    pub attempts: Vec<JobsInspectResponseAttemptsItem>,
    pub errors: Vec<JobsInspectResponseErrorsItem>,
    pub job: JobsInspectResponseJob,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsInspectResponseLineage>,
    pub related: Vec<JobsInspectResponseRelatedItem>,
    pub timeline: Vec<JobsInspectResponseTimelineItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsInspectResponseTrigger>,
}
/// Generated schema type `JobsListDLQRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQRequest {
    pub limit: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}
/// Generated schema type `JobsListDLQResponse`.
/// Generated schema type `JobsListDLQResponseEntriesItem`.
/// Generated schema type `JobsListDLQResponseEntriesItemConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemConcurrency {
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    pub key: String,
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemContext {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "traceId")]
    pub trace_id: String,
    pub traceparent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemErrorDetail`.
/// Generated schema type `JobsListDLQResponseEntriesItemErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemErrorDetailWorker {
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemErrorDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    pub fingerprint: String,
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    pub message: String,
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsListDLQResponseEntriesItemErrorDetailWorker>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemLineage {
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemLogsItem {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}
/// Generated schema type `JobsListDLQResponseEntriesItemProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemQueuePolicy {
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsListDLQResponseEntriesItemTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItemTrigger {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: String,
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponseEntriesItem {
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsListDLQResponseEntriesItemConcurrency>,
    pub context: JobsListDLQResponseEntriesItemContext,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsListDLQResponseEntriesItemErrorDetail>,
    pub id: String,
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsListDLQResponseEntriesItemLineage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsListDLQResponseEntriesItemLogsItem>>,
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    pub payload: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsListDLQResponseEntriesItemProgress>,
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsListDLQResponseEntriesItemQueuePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    pub service: String,
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub state: String,
    pub tries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsListDLQResponseEntriesItemTrigger>,
    pub r#type: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListDLQResponse {
    pub count: i64,
    pub entries: Vec<JobsListDLQResponseEntriesItem>,
    pub limit: i64,
    #[serde(rename = "nextOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    pub offset: i64,
}
/// Generated schema type `JobsListServicesRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListServicesRequest {
    pub limit: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}
/// Generated schema type `JobsListServicesResponse`.
/// Generated schema type `JobsListServicesResponseEntriesItem`.
/// Generated schema type `JobsListServicesResponseEntriesItemWorkersItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListServicesResponseEntriesItemWorkersItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<i64>,
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "jobType")]
    pub job_type: String,
    pub service: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListServicesResponseEntriesItem {
    pub healthy: bool,
    pub name: String,
    pub workers: Vec<JobsListServicesResponseEntriesItemWorkersItem>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsListServicesResponse {
    pub count: i64,
    pub entries: Vec<JobsListServicesResponseEntriesItem>,
    pub limit: i64,
    #[serde(rename = "nextOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    pub offset: i64,
}
/// Generated schema type `JobsQueryRequest`.
/// Generated schema type `JobsQueryRequestSort`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryRequestSort {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    pub field: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryRequest {
    #[serde(rename = "groupBy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
    pub limit: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(rename = "queueKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_key: Option<String>,
    #[serde(rename = "runtimeBand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_band: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<JobsQueryRequestSort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
}
/// Generated schema type `JobsQueryResponse`.
/// Generated schema type `JobsQueryResponseEntriesItem`.
/// Generated schema type `JobsQueryResponseEntriesItemContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItemContext {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "traceId")]
    pub trace_id: String,
    pub traceparent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsQueryResponseEntriesItemLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItemLineage {
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsQueryResponseEntriesItemProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItemProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsQueryResponseEntriesItemTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItemTrigger {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: String,
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseEntriesItem {
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<JobsQueryResponseEntriesItemContext>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "errorFingerprint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_fingerprint: Option<String>,
    pub id: String,
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsQueryResponseEntriesItemLineage>,
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsQueryResponseEntriesItemProgress>,
    #[serde(rename = "queueAgeMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_age_ms: Option<i64>,
    #[serde(rename = "queueKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_key: Option<String>,
    #[serde(rename = "runtimeBand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_band: Option<String>,
    #[serde(rename = "runtimeMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_ms: Option<i64>,
    pub service: String,
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub state: String,
    pub tries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsQueryResponseEntriesItemTrigger>,
    pub r#type: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
/// Generated schema type `JobsQueryResponseGroupsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseGroupsItem {
    pub count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<i64>,
    #[serde(rename = "failureRate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_rate: Option<f64>,
    pub key: String,
    pub label: String,
    #[serde(rename = "latestUpdatedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_updated_at: Option<String>,
    #[serde(rename = "oldestCreatedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}
/// Generated schema type `JobsQueryResponseStats`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponseStats {
    #[serde(rename = "byState")]
    pub by_state: BTreeMap<String, i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dead: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queued: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slow: Option<i64>,
    pub total: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsQueryResponse {
    pub count: i64,
    pub entries: Vec<JobsQueryResponseEntriesItem>,
    pub groups: Vec<JobsQueryResponseGroupsItem>,
    pub limit: i64,
    #[serde(rename = "nextOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    pub offset: i64,
    pub stats: JobsQueryResponseStats,
}
/// Generated schema type `JobsReplayDLQRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQRequest {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
/// Generated schema type `JobsReplayDLQResponse`.
/// Generated schema type `JobsReplayDLQResponseJob`.
/// Generated schema type `JobsReplayDLQResponseJobConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobConcurrency {
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    pub key: String,
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsReplayDLQResponseJobContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobContext {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "traceId")]
    pub trace_id: String,
    pub traceparent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsReplayDLQResponseJobErrorDetail`.
/// Generated schema type `JobsReplayDLQResponseJobErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobErrorDetailWorker {
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobErrorDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    pub fingerprint: String,
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    pub message: String,
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsReplayDLQResponseJobErrorDetailWorker>,
}
/// Generated schema type `JobsReplayDLQResponseJobLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobLineage {
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsReplayDLQResponseJobLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobLogsItem {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}
/// Generated schema type `JobsReplayDLQResponseJobProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsReplayDLQResponseJobQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobQueuePolicy {
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsReplayDLQResponseJobTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJobTrigger {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: String,
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponseJob {
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsReplayDLQResponseJobConcurrency>,
    pub context: JobsReplayDLQResponseJobContext,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsReplayDLQResponseJobErrorDetail>,
    pub id: String,
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsReplayDLQResponseJobLineage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsReplayDLQResponseJobLogsItem>>,
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    pub payload: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsReplayDLQResponseJobProgress>,
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsReplayDLQResponseJobQueuePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    pub service: String,
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub state: String,
    pub tries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsReplayDLQResponseJobTrigger>,
    pub r#type: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsReplayDLQResponse {
    pub job: JobsReplayDLQResponseJob,
}
/// Generated schema type `JobsRetryRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryRequest {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
/// Generated schema type `JobsRetryResponse`.
/// Generated schema type `JobsRetryResponseJob`.
/// Generated schema type `JobsRetryResponseJobConcurrency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobConcurrency {
    #[serde(rename = "heartbeatAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    pub key: String,
    #[serde(rename = "keyHash")]
    pub key_hash: String,
    #[serde(rename = "leaseExpiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    #[serde(rename = "staleTakeoverCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<i64>,
}
/// Generated schema type `JobsRetryResponseJobContext`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobContext {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "traceId")]
    pub trace_id: String,
    pub traceparent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}
/// Generated schema type `JobsRetryResponseJobErrorDetail`.
/// Generated schema type `JobsRetryResponseJobErrorDetailWorker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobErrorDetailWorker {
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobErrorDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<BTreeMap<String, Value>>>,
    pub fingerprint: String,
    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    pub message: String,
    #[serde(rename = "occurrenceCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobsRetryResponseJobErrorDetailWorker>,
}
/// Generated schema type `JobsRetryResponseJobLineage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobLineage {
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "relatedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
    #[serde(rename = "rootJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
}
/// Generated schema type `JobsRetryResponseJobLogsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobLogsItem {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}
/// Generated schema type `JobsRetryResponseJobProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}
/// Generated schema type `JobsRetryResponseJobQueuePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobQueuePolicy {
    #[serde(rename = "existingJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(rename = "replacedJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}
/// Generated schema type `JobsRetryResponseJobTrigger`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJobTrigger {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: String,
    #[serde(rename = "operationId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(rename = "parentJobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "traceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponseJob {
    #[serde(rename = "completedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobsRetryResponseJobConcurrency>,
    pub context: JobsRetryResponseJobContext,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(rename = "errorDetail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobsRetryResponseJobErrorDetail>,
    pub id: String,
    #[serde(rename = "lastError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobsRetryResponseJobLineage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobsRetryResponseJobLogsItem>>,
    #[serde(rename = "maxTries")]
    pub max_tries: i64,
    pub payload: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobsRetryResponseJobProgress>,
    #[serde(rename = "queuePolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobsRetryResponseJobQueuePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    pub service: String,
    #[serde(rename = "startedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub state: String,
    pub tries: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobsRetryResponseJobTrigger>,
    pub r#type: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsRetryResponse {
    pub job: JobsRetryResponseJob,
}
/// Generated schema type `JobsWatchInput`.
/// Generated schema type `JobsWatchInputQuery`.
/// Generated schema type `JobsWatchInputQuerySort`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsWatchInputQuerySort {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    pub field: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsWatchInputQuery {
    #[serde(rename = "groupBy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
    pub limit: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(rename = "queueKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_key: Option<String>,
    #[serde(rename = "runtimeBand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_band: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<JobsWatchInputQuerySort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsWatchInput {
    #[serde(rename = "includeInitial")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_initial: Option<bool>,
    #[serde(rename = "jobId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<JobsWatchInputQuery>,
}
/// Generated schema type `JobsWatchEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobsWatchEvent(pub Value);
