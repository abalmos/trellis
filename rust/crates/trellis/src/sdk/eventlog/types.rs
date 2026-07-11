//! Shared request and response types for `trellis.eventlog@v1`.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
/// Generated schema type `EventLogConsumersInspectRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogConsumersInspectRequest {
    #[serde(rename = "consumerName")]
    pub consumer_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
}
/// Generated schema type `EventLogConsumersQueryRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogConsumersQueryRequest {
    #[serde(rename = "contractId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
    #[serde(rename = "deploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
    pub limit: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(rename = "ownerContractId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_contract_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}
/// Generated schema type `EventLogConsumersQueryResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogConsumersQueryResponse {
    pub consumers: Vec<Value>,
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}
/// Generated schema type `EventLogInspectRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogInspectRequest {
    #[serde(rename = "eventId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    #[serde(rename = "streamSequence")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_sequence: Option<i64>,
}
/// Generated schema type `EventLogMetricsRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
}
/// Generated schema type `EventLogMetricsResponse`.
/// Generated schema type `EventLogMetricsResponseBucketsItem`.
/// Generated schema type `EventLogMetricsResponseBucketsItemByResolution`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseBucketsItemByResolution {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub malformed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved: Option<i64>,
}
/// Generated schema type `EventLogMetricsResponseBucketsItemByVerificationStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseBucketsItemByVerificationStatus {
    #[serde(rename = "auth-unavailable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_unavailable: Option<i64>,
    #[serde(rename = "invalid-signature")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_signature: Option<i64>,
    #[serde(rename = "missing-proof")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_proof: Option<i64>,
    #[serde(rename = "missing-session")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_session: Option<i64>,
    #[serde(rename = "outside-session-window")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outside_session_window: Option<i64>,
    #[serde(rename = "subject-denied")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_denied: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseBucketsItem {
    #[serde(rename = "byResolution")]
    pub by_resolution: EventLogMetricsResponseBucketsItemByResolution,
    #[serde(rename = "byVerificationStatus")]
    pub by_verification_status: EventLogMetricsResponseBucketsItemByVerificationStatus,
    #[serde(rename = "integrityExceptions")]
    pub integrity_exceptions: i64,
    #[serde(rename = "payloadSizeBytes")]
    pub payload_size_bytes: i64,
    pub start: String,
    pub total: i64,
}
/// Generated schema type `EventLogMetricsResponseSummary`.
/// Generated schema type `EventLogMetricsResponseSummaryByResolution`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseSummaryByResolution {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub malformed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved: Option<i64>,
}
/// Generated schema type `EventLogMetricsResponseSummaryByVerificationStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseSummaryByVerificationStatus {
    #[serde(rename = "auth-unavailable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_unavailable: Option<i64>,
    #[serde(rename = "invalid-signature")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_signature: Option<i64>,
    #[serde(rename = "missing-proof")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_proof: Option<i64>,
    #[serde(rename = "missing-session")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_session: Option<i64>,
    #[serde(rename = "outside-session-window")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outside_session_window: Option<i64>,
    #[serde(rename = "subject-denied")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_denied: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<i64>,
}
/// Generated schema type `EventLogMetricsResponseSummaryEventTypesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseSummaryEventTypesItem {
    pub count: i64,
    #[serde(rename = "ownerContractId")]
    pub owner_contract_id: String,
    #[serde(rename = "ownerEventName")]
    pub owner_event_name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponseSummary {
    #[serde(rename = "byResolution")]
    pub by_resolution: EventLogMetricsResponseSummaryByResolution,
    #[serde(rename = "byVerificationStatus")]
    pub by_verification_status: EventLogMetricsResponseSummaryByVerificationStatus,
    #[serde(rename = "eventTypes")]
    pub event_types: Vec<EventLogMetricsResponseSummaryEventTypesItem>,
    #[serde(rename = "integrityExceptions")]
    pub integrity_exceptions: i64,
    #[serde(rename = "payloadSizeBytes")]
    pub payload_size_bytes: i64,
    pub total: i64,
    #[serde(rename = "uniqueSubjects")]
    pub unique_subjects: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogMetricsResponse {
    pub buckets: Vec<EventLogMetricsResponseBucketsItem>,
    pub summary: EventLogMetricsResponseSummary,
}
/// Generated schema type `EventLogQueryRequest`.
/// Generated schema type `EventLogQueryRequestExcludeEventTypesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogQueryRequestExcludeEventTypesItem {
    #[serde(rename = "ownerContractId")]
    pub owner_contract_id: String,
    #[serde(rename = "ownerEventName")]
    pub owner_event_name: String,
}
/// Generated schema type `EventLogQueryRequestIncludeEventTypesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogQueryRequestIncludeEventTypesItem {
    #[serde(rename = "ownerContractId")]
    pub owner_contract_id: String,
    #[serde(rename = "ownerEventName")]
    pub owner_event_name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogQueryRequest {
    #[serde(rename = "consumerDeploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer_deployment_id: Option<String>,
    #[serde(rename = "consumerName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer_name: Option<String>,
    #[serde(rename = "excludeEventTypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_event_types: Option<Vec<EventLogQueryRequestExcludeEventTypesItem>>,
    #[serde(rename = "includeEventTypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_event_types: Option<Vec<EventLogQueryRequestIncludeEventTypesItem>>,
    #[serde(rename = "integrityExceptionOnly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity_exception_only: Option<bool>,
    pub limit: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(rename = "ownerContractId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_contract_id: Option<String>,
    #[serde(rename = "ownerEventName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_event_name: Option<String>,
    #[serde(rename = "publisherContractId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_contract_id: Option<String>,
    #[serde(rename = "publisherDeploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_deployment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "verificationStatus")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_status: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
}
/// Generated schema type `EventLogQueryResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogQueryResponse {
    pub events: Vec<Value>,
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}
/// Generated schema type `EventLogWatchEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventLogWatchEvent(pub BTreeMap<String, Value>);
