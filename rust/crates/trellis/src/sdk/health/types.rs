//! Shared request and response types for `trellis.health@v1`.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
/// Generated schema type `HealthInspectRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectRequest {
    #[serde(rename = "contractId")]
    pub contract_id: String,
    #[serde(rename = "historyLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_limit: Option<i64>,
    #[serde(rename = "historySince")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_since: Option<String>,
    #[serde(rename = "instanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(rename = "participantKind")]
    pub participant_kind: String,
}
/// Generated schema type `HealthInspectResponse`.
/// Generated schema type `HealthInspectResponseHistoryItem`.
/// Generated schema type `HealthInspectResponseHistoryItemChecksItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseHistoryItemChecksItem {
    pub name: String,
    pub status: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseHistoryItem {
    pub checks: Vec<HealthInspectResponseHistoryItemChecksItem>,
    #[serde(rename = "effectiveStatus")]
    pub effective_status: String,
    #[serde(rename = "endedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "intervalId")]
    pub interval_id: i64,
    pub reason: String,
    #[serde(rename = "reportedStatus")]
    pub reported_status: String,
    #[serde(rename = "startedAt")]
    pub started_at: String,
}
/// Generated schema type `HealthInspectResponseInstancesItem`.
/// Generated schema type `HealthInspectResponseInstancesItemLatestSample`.
/// Generated schema type `HealthInspectResponseInstancesItemLatestSampleChecksItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseInstancesItemLatestSampleChecksItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<BTreeMap<String, Value>>,
    #[serde(rename = "latencyMs")]
    pub latency_ms: f64,
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `HealthInspectResponseInstancesItemLatestSampleParticipant`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseInstancesItemLatestSampleParticipant {
    #[serde(rename = "contractDigest")]
    pub contract_digest: String,
    #[serde(rename = "contractId")]
    pub contract_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<BTreeMap<String, Value>>,
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    pub kind: String,
    pub name: String,
    #[serde(rename = "publishIntervalMs")]
    pub publish_interval_ms: i64,
    pub runtime: String,
    #[serde(rename = "runtimeVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String>,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `HealthInspectResponseInstancesItemLatestSampleSample`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseInstancesItemLatestSampleSample {
    pub id: String,
    pub time: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseInstancesItemLatestSample {
    pub checks: Vec<HealthInspectResponseInstancesItemLatestSampleChecksItem>,
    pub participant: HealthInspectResponseInstancesItemLatestSampleParticipant,
    #[serde(rename = "reportedStatus")]
    pub reported_status: String,
    pub sample: HealthInspectResponseInstancesItemLatestSampleSample,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseInstancesItem {
    #[serde(rename = "ageMs")]
    pub age_ms: i64,
    #[serde(rename = "contractDigest")]
    pub contract_digest: String,
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    #[serde(rename = "effectiveStatus")]
    pub effective_status: String,
    #[serde(rename = "heartbeatDeadline")]
    pub heartbeat_deadline: String,
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "latestSample")]
    pub latest_sample: HealthInspectResponseInstancesItemLatestSample,
    #[serde(rename = "observedAt")]
    pub observed_at: String,
    #[serde(rename = "reportedStatus")]
    pub reported_status: String,
    #[serde(rename = "startedAt")]
    pub started_at: String,
}
/// Generated schema type `HealthInspectResponseParticipant`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseParticipant {
    #[serde(rename = "contractId")]
    pub contract_id: String,
    #[serde(rename = "effectiveStatus")]
    pub effective_status: String,
    #[serde(rename = "offlineInstances")]
    pub offline_instances: i64,
    #[serde(rename = "onlineInstances")]
    pub online_instances: i64,
    #[serde(rename = "participantKind")]
    pub participant_kind: String,
    #[serde(rename = "participantName")]
    pub participant_name: String,
}
/// Generated schema type `HealthInspectResponseProjection`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponseProjection {
    #[serde(rename = "completeSince")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete_since: Option<String>,
    #[serde(rename = "gapDetected")]
    pub gap_detected: bool,
    #[serde(rename = "lastStreamSequence")]
    pub last_stream_sequence: i64,
    #[serde(rename = "retainedFrom")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retained_from: Option<String>,
    pub revision: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthInspectResponse {
    #[serde(rename = "asOf")]
    pub as_of: String,
    pub history: Vec<HealthInspectResponseHistoryItem>,
    pub instances: Vec<HealthInspectResponseInstancesItem>,
    pub participant: HealthInspectResponseParticipant,
    pub projection: HealthInspectResponseProjection,
}
/// Generated schema type `HealthMetricsRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsRequest {
    #[serde(rename = "checkNames")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_names: Option<Vec<String>>,
    #[serde(rename = "contractId")]
    pub contract_id: String,
    pub end: String,
    #[serde(rename = "instanceIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_ids: Option<Vec<String>>,
    #[serde(rename = "participantKind")]
    pub participant_kind: String,
    pub start: String,
    #[serde(rename = "stepMs")]
    pub step_ms: i64,
}
/// Generated schema type `HealthMetricsResponse`.
/// Generated schema type `HealthMetricsResponseProjection`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponseProjection {
    #[serde(rename = "completeSince")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete_since: Option<String>,
    #[serde(rename = "gapDetected")]
    pub gap_detected: bool,
    #[serde(rename = "lastStreamSequence")]
    pub last_stream_sequence: i64,
    #[serde(rename = "retainedFrom")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retained_from: Option<String>,
    pub revision: i64,
}
/// Generated schema type `HealthMetricsResponseSeriesItem`.
/// Generated schema type `HealthMetricsResponseSeriesItemBucketsItem`.
/// Generated schema type `HealthMetricsResponseSeriesItemBucketsItemChecksItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponseSeriesItemBucketsItemChecksItem {
    #[serde(rename = "failedCount")]
    pub failed_count: i64,
    #[serde(rename = "latencyAverageMs")]
    pub latency_average_ms: f64,
    #[serde(rename = "latencyMaxMs")]
    pub latency_max_ms: f64,
    pub name: String,
    #[serde(rename = "okCount")]
    pub ok_count: i64,
    #[serde(rename = "sampleCount")]
    pub sample_count: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponseSeriesItemBucketsItem {
    pub checks: Vec<HealthMetricsResponseSeriesItemBucketsItemChecksItem>,
    #[serde(rename = "degradedMs")]
    pub degraded_ms: i64,
    pub end: String,
    #[serde(rename = "healthyMs")]
    pub healthy_ms: i64,
    #[serde(rename = "observedMs")]
    pub observed_ms: i64,
    #[serde(rename = "offlineMs")]
    pub offline_ms: i64,
    #[serde(rename = "sampleCount")]
    pub sample_count: i64,
    pub start: String,
    #[serde(rename = "unhealthyMs")]
    pub unhealthy_ms: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponseSeriesItem {
    pub buckets: Vec<HealthMetricsResponseSeriesItemBucketsItem>,
    #[serde(rename = "contractId")]
    pub contract_id: String,
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "participantKind")]
    pub participant_kind: String,
}
/// Generated schema type `HealthMetricsResponseSummary`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponseSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<f64>,
    #[serde(rename = "observedMs")]
    pub observed_ms: i64,
    #[serde(rename = "onlineMs")]
    pub online_ms: i64,
    #[serde(rename = "sampleCount")]
    pub sample_count: i64,
    pub transitions: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetricsResponse {
    #[serde(rename = "asOf")]
    pub as_of: String,
    pub projection: HealthMetricsResponseProjection,
    pub series: Vec<HealthMetricsResponseSeriesItem>,
    pub summary: HealthMetricsResponseSummary,
}
/// Generated schema type `HealthQueryRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthQueryRequest {
    #[serde(rename = "contractIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_ids: Option<Vec<String>>,
    #[serde(rename = "deploymentIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(rename = "participantKinds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_kinds: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<String>>,
}
/// Generated schema type `HealthQueryResponse`.
/// Generated schema type `HealthQueryResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthQueryResponseEntriesItem {
    #[serde(rename = "contractDigests")]
    pub contract_digests: Vec<String>,
    #[serde(rename = "contractId")]
    pub contract_id: String,
    #[serde(rename = "deploymentIds")]
    pub deployment_ids: Vec<String>,
    #[serde(rename = "effectiveStatus")]
    pub effective_status: String,
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: String,
    #[serde(rename = "offlineInstances")]
    pub offline_instances: i64,
    #[serde(rename = "onlineInstances")]
    pub online_instances: i64,
    #[serde(rename = "participantKind")]
    pub participant_kind: String,
    #[serde(rename = "participantName")]
    pub participant_name: String,
    pub runtimes: Vec<String>,
    pub versions: Vec<String>,
}
/// Generated schema type `HealthQueryResponseProjection`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthQueryResponseProjection {
    #[serde(rename = "completeSince")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete_since: Option<String>,
    #[serde(rename = "gapDetected")]
    pub gap_detected: bool,
    #[serde(rename = "lastStreamSequence")]
    pub last_stream_sequence: i64,
    #[serde(rename = "retainedFrom")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retained_from: Option<String>,
    pub revision: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthQueryResponse {
    #[serde(rename = "asOf")]
    pub as_of: String,
    pub count: i64,
    pub entries: Vec<HealthQueryResponseEntriesItem>,
    pub limit: i64,
    pub offset: i64,
    pub projection: HealthQueryResponseProjection,
}
/// Generated schema type `HealthStatusChangedEvent`.
/// Generated schema type `HealthStatusChangedEventHeader`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthStatusChangedEventHeader {
    pub id: String,
    pub time: String,
}
/// Generated schema type `HealthStatusChangedEventParticipant`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthStatusChangedEventParticipant {
    #[serde(rename = "contractId")]
    pub contract_id: String,
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    pub kind: String,
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthStatusChangedEvent {
    #[serde(rename = "changedAt")]
    pub changed_at: String,
    pub header: HealthStatusChangedEventHeader,
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: String,
    pub participant: HealthStatusChangedEventParticipant,
    #[serde(rename = "previousStatus")]
    pub previous_status: String,
    pub reason: String,
    #[serde(rename = "reportedStatus")]
    pub reported_status: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `HealthWatchInput`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthWatchInput {
    #[serde(rename = "contractIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_ids: Option<Vec<String>>,
    #[serde(rename = "deploymentIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_ids: Option<Vec<String>>,
    #[serde(rename = "instanceIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_ids: Option<Vec<String>>,
    #[serde(rename = "participantKinds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_kinds: Option<Vec<String>>,
}
/// Generated schema type `HealthWatchEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthWatchEvent(pub Value);
/// Generated schema type `HealthHeartbeatSample`.
/// Generated schema type `HealthHeartbeatSampleChecksItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthHeartbeatSampleChecksItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<BTreeMap<String, Value>>,
    #[serde(rename = "latencyMs")]
    pub latency_ms: f64,
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
/// Generated schema type `HealthHeartbeatSampleParticipant`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthHeartbeatSampleParticipant {
    #[serde(rename = "contractDigest")]
    pub contract_digest: String,
    #[serde(rename = "contractId")]
    pub contract_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<BTreeMap<String, Value>>,
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    pub kind: String,
    pub name: String,
    #[serde(rename = "publishIntervalMs")]
    pub publish_interval_ms: i64,
    pub runtime: String,
    #[serde(rename = "runtimeVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String>,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
/// Generated schema type `HealthHeartbeatSampleSample`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthHeartbeatSampleSample {
    pub id: String,
    pub time: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthHeartbeatSample {
    pub checks: Vec<HealthHeartbeatSampleChecksItem>,
    pub participant: HealthHeartbeatSampleParticipant,
    #[serde(rename = "reportedStatus")]
    pub reported_status: String,
    pub sample: HealthHeartbeatSampleSample,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
