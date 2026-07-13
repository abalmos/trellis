//! Shared request and response types for `trellis.health@v1`.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
/// Generated schema type `HealthInspectRequestParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectRequestParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl HealthInspectRequestParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for HealthInspectRequestParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectRequestParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectRequestParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectRequestParticipantKind> for &str {
    fn eq(&self, other: &HealthInspectRequestParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectRequest {
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `historyLimit` wire field.
    #[serde(rename = "historyLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_limit: Option<i64>,
    /// The `historySince` wire field.
    #[serde(rename = "historySince")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_since: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: HealthInspectRequestParticipantKind,
}
/// Generated schema type `HealthInspectResponseHistoryItemChecksItemStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseHistoryItemChecksItemStatus {
    /// The `ok` wire value.
    #[serde(rename = "ok")]
    Ok,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
}
impl HealthInspectResponseHistoryItemChecksItemStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
        }
    }
}
impl AsRef<str> for HealthInspectResponseHistoryItemChecksItemStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseHistoryItemChecksItemStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseHistoryItemChecksItemStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseHistoryItemChecksItemStatus> for &str {
    fn eq(&self, other: &HealthInspectResponseHistoryItemChecksItemStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseHistoryItemChecksItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseHistoryItemChecksItem {
    /// The `name` wire field.
    pub name: String,
    /// The `status` wire field.
    pub status: HealthInspectResponseHistoryItemChecksItemStatus,
}
/// Generated schema type `HealthInspectResponseHistoryItemEffectiveStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseHistoryItemEffectiveStatus {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
    /// The `offline` wire value.
    #[serde(rename = "offline")]
    Offline,
}
impl HealthInspectResponseHistoryItemEffectiveStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
            Self::Offline => "offline",
        }
    }
}
impl AsRef<str> for HealthInspectResponseHistoryItemEffectiveStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseHistoryItemEffectiveStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseHistoryItemEffectiveStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseHistoryItemEffectiveStatus> for &str {
    fn eq(&self, other: &HealthInspectResponseHistoryItemEffectiveStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseHistoryItemReason`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseHistoryItemReason {
    /// The `first-sample` wire value.
    #[serde(rename = "first-sample")]
    FirstSample,
    /// The `heartbeat-change` wire value.
    #[serde(rename = "heartbeat-change")]
    HeartbeatChange,
    /// The `heartbeat-resumed` wire value.
    #[serde(rename = "heartbeat-resumed")]
    HeartbeatResumed,
    /// The `deadline-expired` wire value.
    #[serde(rename = "deadline-expired")]
    DeadlineExpired,
}
impl HealthInspectResponseHistoryItemReason {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::FirstSample => "first-sample",
            Self::HeartbeatChange => "heartbeat-change",
            Self::HeartbeatResumed => "heartbeat-resumed",
            Self::DeadlineExpired => "deadline-expired",
        }
    }
}
impl AsRef<str> for HealthInspectResponseHistoryItemReason {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseHistoryItemReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseHistoryItemReason {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseHistoryItemReason> for &str {
    fn eq(&self, other: &HealthInspectResponseHistoryItemReason) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseHistoryItemReportedStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseHistoryItemReportedStatus {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
}
impl HealthInspectResponseHistoryItemReportedStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
        }
    }
}
impl AsRef<str> for HealthInspectResponseHistoryItemReportedStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseHistoryItemReportedStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseHistoryItemReportedStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseHistoryItemReportedStatus> for &str {
    fn eq(&self, other: &HealthInspectResponseHistoryItemReportedStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseHistoryItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseHistoryItem {
    /// The `checks` wire field.
    pub checks: Vec<HealthInspectResponseHistoryItemChecksItem>,
    /// The `effectiveStatus` wire field.
    #[serde(rename = "effectiveStatus")]
    pub effective_status: HealthInspectResponseHistoryItemEffectiveStatus,
    /// The `endedAt` wire field.
    #[serde(rename = "endedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `intervalId` wire field.
    #[serde(rename = "intervalId")]
    pub interval_id: i64,
    /// The `reason` wire field.
    pub reason: HealthInspectResponseHistoryItemReason,
    /// The `reportedStatus` wire field.
    #[serde(rename = "reportedStatus")]
    pub reported_status: HealthInspectResponseHistoryItemReportedStatus,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
}
/// Generated schema type `HealthInspectResponseInstancesItemEffectiveStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseInstancesItemEffectiveStatus {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
    /// The `offline` wire value.
    #[serde(rename = "offline")]
    Offline,
}
impl HealthInspectResponseInstancesItemEffectiveStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
            Self::Offline => "offline",
        }
    }
}
impl AsRef<str> for HealthInspectResponseInstancesItemEffectiveStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseInstancesItemEffectiveStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseInstancesItemEffectiveStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseInstancesItemEffectiveStatus> for &str {
    fn eq(&self, other: &HealthInspectResponseInstancesItemEffectiveStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseInstancesItemLatestSampleChecksItemStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseInstancesItemLatestSampleChecksItemStatus {
    /// The `ok` wire value.
    #[serde(rename = "ok")]
    Ok,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
}
impl HealthInspectResponseInstancesItemLatestSampleChecksItemStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
        }
    }
}
impl AsRef<str> for HealthInspectResponseInstancesItemLatestSampleChecksItemStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseInstancesItemLatestSampleChecksItemStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseInstancesItemLatestSampleChecksItemStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseInstancesItemLatestSampleChecksItemStatus> for &str {
    fn eq(&self, other: &HealthInspectResponseInstancesItemLatestSampleChecksItemStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseInstancesItemLatestSampleChecksItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseInstancesItemLatestSampleChecksItem {
    /// The `error` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The `info` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<BTreeMap<String, Value>>,
    /// The `latencyMs` wire field.
    #[serde(rename = "latencyMs")]
    pub latency_ms: f64,
    /// The `name` wire field.
    pub name: String,
    /// The `status` wire field.
    pub status: HealthInspectResponseInstancesItemLatestSampleChecksItemStatus,
    /// The `summary` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `HealthInspectResponseInstancesItemLatestSampleParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseInstancesItemLatestSampleParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl HealthInspectResponseInstancesItemLatestSampleParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for HealthInspectResponseInstancesItemLatestSampleParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseInstancesItemLatestSampleParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseInstancesItemLatestSampleParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseInstancesItemLatestSampleParticipantKind> for &str {
    fn eq(&self, other: &HealthInspectResponseInstancesItemLatestSampleParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseInstancesItemLatestSampleParticipantRuntime`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseInstancesItemLatestSampleParticipantRuntime {
    /// The `deno` wire value.
    #[serde(rename = "deno")]
    Deno,
    /// The `node` wire value.
    #[serde(rename = "node")]
    Node,
    /// The `rust` wire value.
    #[serde(rename = "rust")]
    Rust,
    /// The `unknown` wire value.
    #[serde(rename = "unknown")]
    Unknown,
}
impl HealthInspectResponseInstancesItemLatestSampleParticipantRuntime {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Deno => "deno",
            Self::Node => "node",
            Self::Rust => "rust",
            Self::Unknown => "unknown",
        }
    }
}
impl AsRef<str> for HealthInspectResponseInstancesItemLatestSampleParticipantRuntime {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseInstancesItemLatestSampleParticipantRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseInstancesItemLatestSampleParticipantRuntime {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseInstancesItemLatestSampleParticipantRuntime> for &str {
    fn eq(&self, other: &HealthInspectResponseInstancesItemLatestSampleParticipantRuntime) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseInstancesItemLatestSampleParticipant`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseInstancesItemLatestSampleParticipant {
    /// The `contractDigest` wire field.
    #[serde(rename = "contractDigest")]
    pub contract_digest: String,
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `info` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<BTreeMap<String, Value>>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `kind` wire field.
    pub kind: HealthInspectResponseInstancesItemLatestSampleParticipantKind,
    /// The `name` wire field.
    pub name: String,
    /// The `publishIntervalMs` wire field.
    #[serde(rename = "publishIntervalMs")]
    pub publish_interval_ms: i64,
    /// The `runtime` wire field.
    pub runtime: HealthInspectResponseInstancesItemLatestSampleParticipantRuntime,
    /// The `runtimeVersion` wire field.
    #[serde(rename = "runtimeVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `HealthInspectResponseInstancesItemLatestSampleReportedStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseInstancesItemLatestSampleReportedStatus {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
}
impl HealthInspectResponseInstancesItemLatestSampleReportedStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
        }
    }
}
impl AsRef<str> for HealthInspectResponseInstancesItemLatestSampleReportedStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseInstancesItemLatestSampleReportedStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseInstancesItemLatestSampleReportedStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseInstancesItemLatestSampleReportedStatus> for &str {
    fn eq(&self, other: &HealthInspectResponseInstancesItemLatestSampleReportedStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseInstancesItemLatestSampleSample`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseInstancesItemLatestSampleSample {
    /// The `id` wire field.
    pub id: String,
    /// The `time` wire field.
    pub time: String,
}
/// Generated schema type `HealthInspectResponseInstancesItemLatestSample`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseInstancesItemLatestSample {
    /// The `checks` wire field.
    pub checks: Vec<HealthInspectResponseInstancesItemLatestSampleChecksItem>,
    /// The `participant` wire field.
    pub participant: HealthInspectResponseInstancesItemLatestSampleParticipant,
    /// The `reportedStatus` wire field.
    #[serde(rename = "reportedStatus")]
    pub reported_status: HealthInspectResponseInstancesItemLatestSampleReportedStatus,
    /// The `sample` wire field.
    pub sample: HealthInspectResponseInstancesItemLatestSampleSample,
    /// The `summary` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `HealthInspectResponseInstancesItemReportedStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseInstancesItemReportedStatus {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
}
impl HealthInspectResponseInstancesItemReportedStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
        }
    }
}
impl AsRef<str> for HealthInspectResponseInstancesItemReportedStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseInstancesItemReportedStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseInstancesItemReportedStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseInstancesItemReportedStatus> for &str {
    fn eq(&self, other: &HealthInspectResponseInstancesItemReportedStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseInstancesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseInstancesItem {
    /// The `ageMs` wire field.
    #[serde(rename = "ageMs")]
    pub age_ms: i64,
    /// The `contractDigest` wire field.
    #[serde(rename = "contractDigest")]
    pub contract_digest: String,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `effectiveStatus` wire field.
    #[serde(rename = "effectiveStatus")]
    pub effective_status: HealthInspectResponseInstancesItemEffectiveStatus,
    /// The `heartbeatDeadline` wire field.
    #[serde(rename = "heartbeatDeadline")]
    pub heartbeat_deadline: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `latestSample` wire field.
    #[serde(rename = "latestSample")]
    pub latest_sample: HealthInspectResponseInstancesItemLatestSample,
    /// The `observedAt` wire field.
    #[serde(rename = "observedAt")]
    pub observed_at: String,
    /// The `reportedStatus` wire field.
    #[serde(rename = "reportedStatus")]
    pub reported_status: HealthInspectResponseInstancesItemReportedStatus,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
}
/// Generated schema type `HealthInspectResponseParticipantEffectiveStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseParticipantEffectiveStatus {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
    /// The `offline` wire value.
    #[serde(rename = "offline")]
    Offline,
}
impl HealthInspectResponseParticipantEffectiveStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
            Self::Offline => "offline",
        }
    }
}
impl AsRef<str> for HealthInspectResponseParticipantEffectiveStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseParticipantEffectiveStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseParticipantEffectiveStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseParticipantEffectiveStatus> for &str {
    fn eq(&self, other: &HealthInspectResponseParticipantEffectiveStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseParticipantParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthInspectResponseParticipantParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl HealthInspectResponseParticipantParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for HealthInspectResponseParticipantParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthInspectResponseParticipantParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthInspectResponseParticipantParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthInspectResponseParticipantParticipantKind> for &str {
    fn eq(&self, other: &HealthInspectResponseParticipantParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthInspectResponseParticipant`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseParticipant {
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `effectiveStatus` wire field.
    #[serde(rename = "effectiveStatus")]
    pub effective_status: HealthInspectResponseParticipantEffectiveStatus,
    /// The `offlineInstances` wire field.
    #[serde(rename = "offlineInstances")]
    pub offline_instances: i64,
    /// The `onlineInstances` wire field.
    #[serde(rename = "onlineInstances")]
    pub online_instances: i64,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: HealthInspectResponseParticipantParticipantKind,
    /// The `participantName` wire field.
    #[serde(rename = "participantName")]
    pub participant_name: String,
}
/// Generated schema type `HealthInspectResponseProjection`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseProjection {
    /// The `completeSince` wire field.
    #[serde(rename = "completeSince")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete_since: Option<String>,
    /// The `gapDetected` wire field.
    #[serde(rename = "gapDetected")]
    pub gap_detected: bool,
    /// The `lastStreamSequence` wire field.
    #[serde(rename = "lastStreamSequence")]
    pub last_stream_sequence: i64,
    /// The `retainedFrom` wire field.
    #[serde(rename = "retainedFrom")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retained_from: Option<String>,
    /// The `revision` wire field.
    pub revision: i64,
}
/// Generated schema type `HealthInspectResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponse {
    /// The `asOf` wire field.
    #[serde(rename = "asOf")]
    pub as_of: String,
    /// The `history` wire field.
    pub history: Vec<HealthInspectResponseHistoryItem>,
    /// The `instances` wire field.
    pub instances: Vec<HealthInspectResponseInstancesItem>,
    /// The `participant` wire field.
    pub participant: HealthInspectResponseParticipant,
    /// The `projection` wire field.
    pub projection: HealthInspectResponseProjection,
}
/// Generated schema type `HealthMetricsRequestParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthMetricsRequestParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl HealthMetricsRequestParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for HealthMetricsRequestParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthMetricsRequestParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthMetricsRequestParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthMetricsRequestParticipantKind> for &str {
    fn eq(&self, other: &HealthMetricsRequestParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthMetricsRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsRequest {
    /// The `checkNames` wire field.
    #[serde(rename = "checkNames")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_names: Option<Vec<String>>,
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `end` wire field.
    pub end: String,
    /// The `instanceIds` wire field.
    #[serde(rename = "instanceIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_ids: Option<Vec<String>>,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: HealthMetricsRequestParticipantKind,
    /// The `start` wire field.
    pub start: String,
    /// The `stepMs` wire field.
    #[serde(rename = "stepMs")]
    pub step_ms: i64,
}
/// Generated schema type `HealthMetricsResponseProjection`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponseProjection {
    /// The `completeSince` wire field.
    #[serde(rename = "completeSince")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete_since: Option<String>,
    /// The `gapDetected` wire field.
    #[serde(rename = "gapDetected")]
    pub gap_detected: bool,
    /// The `lastStreamSequence` wire field.
    #[serde(rename = "lastStreamSequence")]
    pub last_stream_sequence: i64,
    /// The `retainedFrom` wire field.
    #[serde(rename = "retainedFrom")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retained_from: Option<String>,
    /// The `revision` wire field.
    pub revision: i64,
}
/// Generated schema type `HealthMetricsResponseSeriesItemBucketsItemChecksItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponseSeriesItemBucketsItemChecksItem {
    /// The `failedCount` wire field.
    #[serde(rename = "failedCount")]
    pub failed_count: i64,
    /// The `latencyAverageMs` wire field.
    #[serde(rename = "latencyAverageMs")]
    pub latency_average_ms: f64,
    /// The `latencyMaxMs` wire field.
    #[serde(rename = "latencyMaxMs")]
    pub latency_max_ms: f64,
    /// The `name` wire field.
    pub name: String,
    /// The `okCount` wire field.
    #[serde(rename = "okCount")]
    pub ok_count: i64,
    /// The `sampleCount` wire field.
    #[serde(rename = "sampleCount")]
    pub sample_count: i64,
}
/// Generated schema type `HealthMetricsResponseSeriesItemBucketsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponseSeriesItemBucketsItem {
    /// The `checks` wire field.
    pub checks: Vec<HealthMetricsResponseSeriesItemBucketsItemChecksItem>,
    /// The `degradedMs` wire field.
    #[serde(rename = "degradedMs")]
    pub degraded_ms: i64,
    /// The `end` wire field.
    pub end: String,
    /// The `healthyMs` wire field.
    #[serde(rename = "healthyMs")]
    pub healthy_ms: i64,
    /// The `observedMs` wire field.
    #[serde(rename = "observedMs")]
    pub observed_ms: i64,
    /// The `offlineMs` wire field.
    #[serde(rename = "offlineMs")]
    pub offline_ms: i64,
    /// The `sampleCount` wire field.
    #[serde(rename = "sampleCount")]
    pub sample_count: i64,
    /// The `start` wire field.
    pub start: String,
    /// The `unhealthyMs` wire field.
    #[serde(rename = "unhealthyMs")]
    pub unhealthy_ms: i64,
}
/// Generated schema type `HealthMetricsResponseSeriesItemParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthMetricsResponseSeriesItemParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl HealthMetricsResponseSeriesItemParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for HealthMetricsResponseSeriesItemParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthMetricsResponseSeriesItemParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthMetricsResponseSeriesItemParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthMetricsResponseSeriesItemParticipantKind> for &str {
    fn eq(&self, other: &HealthMetricsResponseSeriesItemParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthMetricsResponseSeriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponseSeriesItem {
    /// The `buckets` wire field.
    pub buckets: Vec<HealthMetricsResponseSeriesItemBucketsItem>,
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: HealthMetricsResponseSeriesItemParticipantKind,
}
/// Generated schema type `HealthMetricsResponseSummary`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponseSummary {
    /// The `availability` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<f64>,
    /// The `observedMs` wire field.
    #[serde(rename = "observedMs")]
    pub observed_ms: i64,
    /// The `onlineMs` wire field.
    #[serde(rename = "onlineMs")]
    pub online_ms: i64,
    /// The `sampleCount` wire field.
    #[serde(rename = "sampleCount")]
    pub sample_count: i64,
    /// The `transitions` wire field.
    pub transitions: i64,
}
/// Generated schema type `HealthMetricsResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponse {
    /// The `asOf` wire field.
    #[serde(rename = "asOf")]
    pub as_of: String,
    /// The `projection` wire field.
    pub projection: HealthMetricsResponseProjection,
    /// The `series` wire field.
    pub series: Vec<HealthMetricsResponseSeriesItem>,
    /// The `summary` wire field.
    pub summary: HealthMetricsResponseSummary,
}
/// Generated schema type `HealthQueryRequestParticipantKindsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthQueryRequestParticipantKindsItem {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl HealthQueryRequestParticipantKindsItem {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for HealthQueryRequestParticipantKindsItem {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthQueryRequestParticipantKindsItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthQueryRequestParticipantKindsItem {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthQueryRequestParticipantKindsItem> for &str {
    fn eq(&self, other: &HealthQueryRequestParticipantKindsItem) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthQueryRequestStatusesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthQueryRequestStatusesItem {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
    /// The `offline` wire value.
    #[serde(rename = "offline")]
    Offline,
}
impl HealthQueryRequestStatusesItem {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
            Self::Offline => "offline",
        }
    }
}
impl AsRef<str> for HealthQueryRequestStatusesItem {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthQueryRequestStatusesItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthQueryRequestStatusesItem {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthQueryRequestStatusesItem> for &str {
    fn eq(&self, other: &HealthQueryRequestStatusesItem) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthQueryRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthQueryRequest {
    /// The `contractIds` wire field.
    #[serde(rename = "contractIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_ids: Option<Vec<String>>,
    /// The `deploymentIds` wire field.
    #[serde(rename = "deploymentIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_ids: Option<Vec<String>>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `offset` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// The `participantKinds` wire field.
    #[serde(rename = "participantKinds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_kinds: Option<Vec<HealthQueryRequestParticipantKindsItem>>,
    /// The `search` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    /// The `statuses` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<HealthQueryRequestStatusesItem>>,
}
/// Generated schema type `HealthQueryResponseEntriesItemEffectiveStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthQueryResponseEntriesItemEffectiveStatus {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
    /// The `offline` wire value.
    #[serde(rename = "offline")]
    Offline,
}
impl HealthQueryResponseEntriesItemEffectiveStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
            Self::Offline => "offline",
        }
    }
}
impl AsRef<str> for HealthQueryResponseEntriesItemEffectiveStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthQueryResponseEntriesItemEffectiveStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthQueryResponseEntriesItemEffectiveStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthQueryResponseEntriesItemEffectiveStatus> for &str {
    fn eq(&self, other: &HealthQueryResponseEntriesItemEffectiveStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthQueryResponseEntriesItemParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthQueryResponseEntriesItemParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl HealthQueryResponseEntriesItemParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for HealthQueryResponseEntriesItemParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthQueryResponseEntriesItemParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthQueryResponseEntriesItemParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthQueryResponseEntriesItemParticipantKind> for &str {
    fn eq(&self, other: &HealthQueryResponseEntriesItemParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthQueryResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthQueryResponseEntriesItem {
    /// The `contractDigests` wire field.
    #[serde(rename = "contractDigests")]
    pub contract_digests: Vec<String>,
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `deploymentIds` wire field.
    #[serde(rename = "deploymentIds")]
    pub deployment_ids: Vec<String>,
    /// The `effectiveStatus` wire field.
    #[serde(rename = "effectiveStatus")]
    pub effective_status: HealthQueryResponseEntriesItemEffectiveStatus,
    /// The `lastSeenAt` wire field.
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: String,
    /// The `offlineInstances` wire field.
    #[serde(rename = "offlineInstances")]
    pub offline_instances: i64,
    /// The `onlineInstances` wire field.
    #[serde(rename = "onlineInstances")]
    pub online_instances: i64,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: HealthQueryResponseEntriesItemParticipantKind,
    /// The `participantName` wire field.
    #[serde(rename = "participantName")]
    pub participant_name: String,
    /// The `runtimes` wire field.
    pub runtimes: Vec<String>,
    /// The `versions` wire field.
    pub versions: Vec<String>,
}
/// Generated schema type `HealthQueryResponseProjection`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthQueryResponseProjection {
    /// The `completeSince` wire field.
    #[serde(rename = "completeSince")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete_since: Option<String>,
    /// The `gapDetected` wire field.
    #[serde(rename = "gapDetected")]
    pub gap_detected: bool,
    /// The `lastStreamSequence` wire field.
    #[serde(rename = "lastStreamSequence")]
    pub last_stream_sequence: i64,
    /// The `retainedFrom` wire field.
    #[serde(rename = "retainedFrom")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retained_from: Option<String>,
    /// The `revision` wire field.
    pub revision: i64,
}
/// Generated schema type `HealthQueryResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthQueryResponse {
    /// The `asOf` wire field.
    #[serde(rename = "asOf")]
    pub as_of: String,
    /// The `count` wire field.
    pub count: i64,
    /// The `entries` wire field.
    pub entries: Vec<HealthQueryResponseEntriesItem>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    pub offset: i64,
    /// The `projection` wire field.
    pub projection: HealthQueryResponseProjection,
}
/// Generated schema type `HealthStatusChangedEventHeader`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthStatusChangedEventHeader {
    /// The `id` wire field.
    pub id: String,
    /// The `time` wire field.
    pub time: String,
}
/// Generated schema type `HealthStatusChangedEventParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatusChangedEventParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl HealthStatusChangedEventParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for HealthStatusChangedEventParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthStatusChangedEventParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthStatusChangedEventParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthStatusChangedEventParticipantKind> for &str {
    fn eq(&self, other: &HealthStatusChangedEventParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthStatusChangedEventParticipant`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthStatusChangedEventParticipant {
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `kind` wire field.
    pub kind: HealthStatusChangedEventParticipantKind,
    /// The `name` wire field.
    pub name: String,
}
/// Generated schema type `HealthStatusChangedEventPreviousStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatusChangedEventPreviousStatus {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
    /// The `offline` wire value.
    #[serde(rename = "offline")]
    Offline,
}
impl HealthStatusChangedEventPreviousStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
            Self::Offline => "offline",
        }
    }
}
impl AsRef<str> for HealthStatusChangedEventPreviousStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthStatusChangedEventPreviousStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthStatusChangedEventPreviousStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthStatusChangedEventPreviousStatus> for &str {
    fn eq(&self, other: &HealthStatusChangedEventPreviousStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthStatusChangedEventReason`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatusChangedEventReason {
    /// The `heartbeat-change` wire value.
    #[serde(rename = "heartbeat-change")]
    HeartbeatChange,
    /// The `heartbeat-resumed` wire value.
    #[serde(rename = "heartbeat-resumed")]
    HeartbeatResumed,
    /// The `deadline-expired` wire value.
    #[serde(rename = "deadline-expired")]
    DeadlineExpired,
}
impl HealthStatusChangedEventReason {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::HeartbeatChange => "heartbeat-change",
            Self::HeartbeatResumed => "heartbeat-resumed",
            Self::DeadlineExpired => "deadline-expired",
        }
    }
}
impl AsRef<str> for HealthStatusChangedEventReason {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthStatusChangedEventReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthStatusChangedEventReason {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthStatusChangedEventReason> for &str {
    fn eq(&self, other: &HealthStatusChangedEventReason) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthStatusChangedEventReportedStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatusChangedEventReportedStatus {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
}
impl HealthStatusChangedEventReportedStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
        }
    }
}
impl AsRef<str> for HealthStatusChangedEventReportedStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthStatusChangedEventReportedStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthStatusChangedEventReportedStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthStatusChangedEventReportedStatus> for &str {
    fn eq(&self, other: &HealthStatusChangedEventReportedStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthStatusChangedEventStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatusChangedEventStatus {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
    /// The `offline` wire value.
    #[serde(rename = "offline")]
    Offline,
}
impl HealthStatusChangedEventStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
            Self::Offline => "offline",
        }
    }
}
impl AsRef<str> for HealthStatusChangedEventStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthStatusChangedEventStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthStatusChangedEventStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthStatusChangedEventStatus> for &str {
    fn eq(&self, other: &HealthStatusChangedEventStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthStatusChangedEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthStatusChangedEvent {
    /// The `changedAt` wire field.
    #[serde(rename = "changedAt")]
    pub changed_at: String,
    /// The `header` wire field.
    pub header: HealthStatusChangedEventHeader,
    /// The `lastSeenAt` wire field.
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: String,
    /// The `participant` wire field.
    pub participant: HealthStatusChangedEventParticipant,
    /// The `previousStatus` wire field.
    #[serde(rename = "previousStatus")]
    pub previous_status: HealthStatusChangedEventPreviousStatus,
    /// The `reason` wire field.
    pub reason: HealthStatusChangedEventReason,
    /// The `reportedStatus` wire field.
    #[serde(rename = "reportedStatus")]
    pub reported_status: HealthStatusChangedEventReportedStatus,
    /// The `status` wire field.
    pub status: HealthStatusChangedEventStatus,
    /// The `summary` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `HealthWatchInputParticipantKindsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthWatchInputParticipantKindsItem {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl HealthWatchInputParticipantKindsItem {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for HealthWatchInputParticipantKindsItem {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthWatchInputParticipantKindsItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthWatchInputParticipantKindsItem {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthWatchInputParticipantKindsItem> for &str {
    fn eq(&self, other: &HealthWatchInputParticipantKindsItem) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthWatchInput`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthWatchInput {
    /// The `contractIds` wire field.
    #[serde(rename = "contractIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_ids: Option<Vec<String>>,
    /// The `deploymentIds` wire field.
    #[serde(rename = "deploymentIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_ids: Option<Vec<String>>,
    /// The `instanceIds` wire field.
    #[serde(rename = "instanceIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_ids: Option<Vec<String>>,
    /// The `participantKinds` wire field.
    #[serde(rename = "participantKinds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_kinds: Option<Vec<HealthWatchInputParticipantKindsItem>>,
}
/// Generated schema type `HealthWatchEventHealthInvalidatedChangesItemParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthWatchEventHealthInvalidatedChangesItemParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl HealthWatchEventHealthInvalidatedChangesItemParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for HealthWatchEventHealthInvalidatedChangesItemParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthWatchEventHealthInvalidatedChangesItemParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthWatchEventHealthInvalidatedChangesItemParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthWatchEventHealthInvalidatedChangesItemParticipantKind> for &str {
    fn eq(&self, other: &HealthWatchEventHealthInvalidatedChangesItemParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthWatchEventHealthInvalidatedChangesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthWatchEventHealthInvalidatedChangesItem {
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: HealthWatchEventHealthInvalidatedChangesItemParticipantKind,
}
/// Generated schema type `HealthWatchEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum HealthWatchEvent {
    /// The `ready` variant.
    #[serde(rename = "ready")]
    Ready {
        /// The `projectionRevision` wire field.
        #[serde(rename = "projectionRevision")]
        projection_revision: i64,
    },
    /// The `healthInvalidated` variant.
    #[serde(rename = "healthInvalidated")]
    HealthInvalidated {
        /// The `changes` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        changes: Option<Vec<HealthWatchEventHealthInvalidatedChangesItem>>,
        /// The `projectionRevision` wire field.
        #[serde(rename = "projectionRevision")]
        projection_revision: i64,
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
/// Generated schema type `HealthHeartbeatSampleChecksItemStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthHeartbeatSampleChecksItemStatus {
    /// The `ok` wire value.
    #[serde(rename = "ok")]
    Ok,
    /// The `failed` wire value.
    #[serde(rename = "failed")]
    Failed,
}
impl HealthHeartbeatSampleChecksItemStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
        }
    }
}
impl AsRef<str> for HealthHeartbeatSampleChecksItemStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthHeartbeatSampleChecksItemStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthHeartbeatSampleChecksItemStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthHeartbeatSampleChecksItemStatus> for &str {
    fn eq(&self, other: &HealthHeartbeatSampleChecksItemStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthHeartbeatSampleChecksItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthHeartbeatSampleChecksItem {
    /// The `error` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The `info` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<BTreeMap<String, Value>>,
    /// The `latencyMs` wire field.
    #[serde(rename = "latencyMs")]
    pub latency_ms: f64,
    /// The `name` wire field.
    pub name: String,
    /// The `status` wire field.
    pub status: HealthHeartbeatSampleChecksItemStatus,
    /// The `summary` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `HealthHeartbeatSampleParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthHeartbeatSampleParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl HealthHeartbeatSampleParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for HealthHeartbeatSampleParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthHeartbeatSampleParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthHeartbeatSampleParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthHeartbeatSampleParticipantKind> for &str {
    fn eq(&self, other: &HealthHeartbeatSampleParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthHeartbeatSampleParticipantRuntime`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthHeartbeatSampleParticipantRuntime {
    /// The `deno` wire value.
    #[serde(rename = "deno")]
    Deno,
    /// The `node` wire value.
    #[serde(rename = "node")]
    Node,
    /// The `rust` wire value.
    #[serde(rename = "rust")]
    Rust,
    /// The `unknown` wire value.
    #[serde(rename = "unknown")]
    Unknown,
}
impl HealthHeartbeatSampleParticipantRuntime {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Deno => "deno",
            Self::Node => "node",
            Self::Rust => "rust",
            Self::Unknown => "unknown",
        }
    }
}
impl AsRef<str> for HealthHeartbeatSampleParticipantRuntime {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthHeartbeatSampleParticipantRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthHeartbeatSampleParticipantRuntime {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthHeartbeatSampleParticipantRuntime> for &str {
    fn eq(&self, other: &HealthHeartbeatSampleParticipantRuntime) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthHeartbeatSampleParticipant`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthHeartbeatSampleParticipant {
    /// The `contractDigest` wire field.
    #[serde(rename = "contractDigest")]
    pub contract_digest: String,
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `info` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<BTreeMap<String, Value>>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `kind` wire field.
    pub kind: HealthHeartbeatSampleParticipantKind,
    /// The `name` wire field.
    pub name: String,
    /// The `publishIntervalMs` wire field.
    #[serde(rename = "publishIntervalMs")]
    pub publish_interval_ms: i64,
    /// The `runtime` wire field.
    pub runtime: HealthHeartbeatSampleParticipantRuntime,
    /// The `runtimeVersion` wire field.
    #[serde(rename = "runtimeVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String>,
    /// The `startedAt` wire field.
    #[serde(rename = "startedAt")]
    pub started_at: String,
    /// The `version` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `HealthHeartbeatSampleReportedStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthHeartbeatSampleReportedStatus {
    /// The `healthy` wire value.
    #[serde(rename = "healthy")]
    Healthy,
    /// The `degraded` wire value.
    #[serde(rename = "degraded")]
    Degraded,
    /// The `unhealthy` wire value.
    #[serde(rename = "unhealthy")]
    Unhealthy,
}
impl HealthHeartbeatSampleReportedStatus {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
        }
    }
}
impl AsRef<str> for HealthHeartbeatSampleReportedStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for HealthHeartbeatSampleReportedStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for HealthHeartbeatSampleReportedStatus {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<HealthHeartbeatSampleReportedStatus> for &str {
    fn eq(&self, other: &HealthHeartbeatSampleReportedStatus) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `HealthHeartbeatSampleSample`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthHeartbeatSampleSample {
    /// The `id` wire field.
    pub id: String,
    /// The `time` wire field.
    pub time: String,
}
/// Generated schema type `HealthHeartbeatSample`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthHeartbeatSample {
    /// The `checks` wire field.
    pub checks: Vec<HealthHeartbeatSampleChecksItem>,
    /// The `participant` wire field.
    pub participant: HealthHeartbeatSampleParticipant,
    /// The `reportedStatus` wire field.
    #[serde(rename = "reportedStatus")]
    pub reported_status: HealthHeartbeatSampleReportedStatus,
    /// The `sample` wire field.
    pub sample: HealthHeartbeatSampleSample,
    /// The `summary` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
