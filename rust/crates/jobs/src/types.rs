use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobContext {
    pub request_id: String,
    pub trace_id: String,
    pub traceparent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobTriggerKind {
    /// Job was started by a schedule.
    #[serde(rename = "schedule")]
    Schedule,
    /// Job was started by operation control code.
    #[serde(rename = "operation")]
    Operation,
    /// Job was started by an RPC handler.
    #[serde(rename = "rpc")]
    Rpc,
    /// Job was started by an event handler.
    #[serde(rename = "event")]
    Event,
    /// Job was manually replayed by an administrator.
    #[serde(rename = "manualReplay")]
    ManualReplay,
    /// Job was created directly by service code.
    #[serde(rename = "serviceCode")]
    ServiceCode,
    /// Job was created from inside another active job handler.
    #[serde(rename = "parentJob")]
    ParentJob,
}

/// Describes the source that created or retried a job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobTrigger {
    pub kind: JobTriggerKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Relates a job to parent/root work and optional operation context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobLineage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_keys: Option<Vec<String>>,
}

/// Metadata for an administrator-initiated lifecycle action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobAdminAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    Pending,
    Active,
    Retry,
    Completed,
    Failed,
    Cancelled,
    Expired,
    Skipped,
    Stale,
    Dead,
    Dismissed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobEventType {
    Created,
    Started,
    Retry,
    Progress,
    Logged,
    Completed,
    Failed,
    Cancelled,
    Expired,
    Skipped,
    Stale,
    Heartbeat,
    #[serde(rename = "staleCompletionIgnored")]
    StaleCompletionIgnored,
    Retried,
    Dead,
    Dismissed,
}

impl JobEventType {
    pub fn as_token(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Started => "started",
            Self::Retry => "retry",
            Self::Progress => "progress",
            Self::Logged => "logged",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
            Self::Skipped => "skipped",
            Self::Stale => "stale",
            Self::Heartbeat => "heartbeat",
            Self::StaleCompletionIgnored => "staleCompletionIgnored",
            Self::Retried => "retried",
            Self::Dead => "dead",
            Self::Dismissed => "dismissed",
        }
    }
}

/// Keyed-concurrency metadata carried by lifecycle events and projections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobConcurrency {
    pub key: String,
    pub key_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_takeover_count: Option<u64>,
}

/// Queue-policy outcome recorded on keyed lifecycle events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JobQueuePolicyOutcome {
    Accepted,
    Rejected,
    Coalesced,
    Replaced,
}

/// Queue-policy metadata carried by lifecycle events and projections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobQueuePolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<JobQueuePolicyOutcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_job_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobLogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobLogEntry {
    pub timestamp: String,
    pub level: JobLogLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct JobProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

/// Worker metadata associated with a captured job error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobErrorWorker {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
}

/// Structured detail for a job failure or retry reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobErrorDetail {
    pub message: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<JobErrorDetail>>,
    pub fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<JobErrorWorker>,
}

impl JobErrorDetail {
    /// Create an error detail from the only currently guaranteed handler error shape.
    pub fn from_message(service: &str, job_type: &str, message: &str) -> Self {
        Self {
            message: message.to_string(),
            error_type: None,
            stack: None,
            causes: None,
            fingerprint: error_fingerprint(service, job_type, message),
            first_seen: None,
            occurrence_count: None,
            worker: None,
        }
    }
}

/// Return the stable fingerprint for a service, job type, and normalized first error line.
pub fn error_fingerprint(service: &str, job_type: &str, message: &str) -> String {
    let first_line = message.lines().next().unwrap_or_default();
    let normalized = first_line.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in format!("{service}\0{job_type}\0{}", normalized.to_lowercase()).bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub context: JobContext,
    pub service: String,
    #[serde(rename = "type")]
    pub job_type: String,
    pub state: JobState,
    pub payload: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    pub tries: u64,
    pub max_tries: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobLogEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobConcurrency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobQueuePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobTrigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobLineage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobEvent {
    pub job_id: String,
    pub context: JobContext,
    pub service: String,
    pub job_type: String,
    pub event_type: JobEventType,
    pub state: JobState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_state: Option<JobState>,
    pub tries: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tries: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<JobErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobLogEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<JobConcurrency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_policy: Option<JobQueuePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<JobTrigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<JobLineage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_action: Option<JobAdminAction>,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerHeartbeat {
    pub service: String,
    pub job_type: String,
    pub instance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub timestamp: String,
}
