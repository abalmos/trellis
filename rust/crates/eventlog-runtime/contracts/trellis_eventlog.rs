//! Rust source for the `trellis.eventlog@v1` contract manifest.

use serde_json::{json, Value};
use trellis_contracts::{
    ApiArtifact, ApiBuilder, ContractArtifacts, ContractBuilder, ContractCapabilityMetadata,
    ContractKind, ContractsError,
};

const READ_CAPABILITY: &str = "events.read";
const STREAM_CAPABILITY: &str = "events.stream";
const UNEXPECTED_ERROR: &str = "UnexpectedError";
const VALIDATION_ERROR: &str = "ValidationError";
const NOT_FOUND_ERROR: &str = "NotFoundError";

/// Build the canonical Event Log API artifact.
pub fn api_artifact() -> Result<ApiArtifact, ContractsError> {
    ApiBuilder::authoring(
        "trellis.eventlog@v1",
        "1.0.0",
        "Trellis Event Log",
        "Read-only Event Log API for Trellis event stream observability.",
    )
    .docs_with_summary(
        "Event stream observability APIs.",
        "Provides read-only event, consumer-health, metrics, and live invalidation surfaces for Trellis events.",
    )
    .capability(
        READ_CAPABILITY,
        ContractCapabilityMetadata {
            display_name: "Read Event Log data".to_string(),
            description: "View projected Trellis events and event consumer health.".to_string(),
            consequence: None,
        },
    )
    .capability(
        STREAM_CAPABILITY,
        ContractCapabilityMetadata {
            display_name: "Stream Event Log changes".to_string(),
            description: "Subscribe to Event Log live invalidation frames.".to_string(),
            consequence: None,
        },
    )
    .schema("EventLogQueryRequest", event_query_request_schema())
    .schema("EventLogQueryResponse", page_schema("events", "EventLogRow"))
    .schema("EventLogRow", event_row_schema())
    .schema("EventLogInspectRequest", event_inspect_request_schema())
    .schema("EventLogInspectResponse", open_schema())
    .schema("EventLogMetricsRequest", metrics_request_schema())
    .schema("EventLogMetricsResponse", metrics_response_schema())
    .schema("EventLogConsumersQueryRequest", consumers_query_request_schema())
    .schema("EventLogConsumersQueryResponse", page_schema("consumers", "EventConsumerStatusRow"))
    .schema("EventLogConsumersInspectRequest", consumers_inspect_request_schema())
    .schema("EventLogConsumersInspectResponse", open_schema())
    .schema("EventConsumerStatusRow", consumer_row_schema())
    .schema("EventLogWatchRequest", open_schema())
    .schema("EventLogWatchFrame", open_schema())
    .schema("NotFoundErrorData", not_found_error_schema())
    .error(NOT_FOUND_ERROR, NOT_FOUND_ERROR, "NotFoundErrorData")
    .rpc("EventLog.Query", read_rpc("EventLog.Query", "EventLogQueryRequest", "EventLogQueryResponse").with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR]))
    .rpc("EventLog.Inspect", read_rpc("EventLog.Inspect", "EventLogInspectRequest", "EventLogInspectResponse").with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR, NOT_FOUND_ERROR]))
    .rpc("EventLog.Metrics", read_rpc("EventLog.Metrics", "EventLogMetricsRequest", "EventLogMetricsResponse").with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR]))
    .rpc("EventLog.Consumers.Query", read_rpc("EventLog.Consumers.Query", "EventLogConsumersQueryRequest", "EventLogConsumersQueryResponse").with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR]))
    .rpc("EventLog.Consumers.Inspect", read_rpc("EventLog.Consumers.Inspect", "EventLogConsumersInspectRequest", "EventLogConsumersInspectResponse").with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR, NOT_FOUND_ERROR]))
    .feed(
        "EventLog.Watch",
        trellis_contracts::feed("v1", "feeds.v1.EventLog.Watch", "EventLogWatchRequest", "EventLogWatchFrame")
            .docs_with_summary("Watch event changes.", "Streams ready and invalidation frames for Event Log clients.")
            .with_subscribe_capabilities([STREAM_CAPABILITY]),
    )
    .build()
}

/// Build the native Event Log participant and API artifacts.
pub fn contract_artifacts() -> Result<ContractArtifacts, ContractsError> {
    let api = api_artifact()?.normalized_value()?;
    ContractBuilder::from_api("trellis.eventlog@v1", api, ContractKind::Service)?.build()
}

fn read_rpc(
    name: &str,
    input_schema: &str,
    output_schema: &str,
) -> trellis_contracts::ContractRpcMethod {
    trellis_contracts::rpc("v1", format!("rpc.v1.{name}"), input_schema, output_schema)
        .with_call_capabilities([READ_CAPABILITY])
}

fn open_schema() -> Value {
    json!({ "type": "object", "additionalProperties": true })
}

fn not_found_error_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "properties": {
            "type": { "const": "NotFoundError" },
            "message": { "type": "string" },
            "id": { "type": "string" },
            "context": open_schema()
        },
        "required": ["type", "message", "id"]
    })
}

fn page_schema(field: &str, item_schema: &str) -> Value {
    json!({
        "type": "object",
        "properties": {
            field: { "type": "array", "items": { "schema": item_schema } },
            "total": { "type": "integer" },
            "offset": { "type": "integer" },
            "limit": { "type": "integer" }
        },
        "required": [field, "total", "offset", "limit"]
    })
}

fn string_enum(values: &[&str]) -> Value {
    json!({ "anyOf": values.iter().map(|value| json!({ "const": value })).collect::<Vec<_>>() })
}

fn event_query_request_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "search": { "type": "string" },
            "subject": { "type": "string" },
            "ownerContractId": { "type": "string" },
            "ownerEventName": { "type": "string" },
            "includeEventTypes": { "type": "array", "items": event_type_schema() },
            "excludeEventTypes": { "type": "array", "items": event_type_schema() },
            "publisherDeploymentId": { "type": "string" },
            "publisherParticipantId": { "type": "string" },
            "consumerDeploymentId": { "type": "string" },
            "resolution": { "type": "array", "items": string_enum(&["resolved", "unresolved", "malformed"]) },
            "verificationStatus": { "type": "array", "items": verification_status_schema() },
            "integrityExceptionOnly": { "type": "boolean" },
            "consumerName": { "type": "string" },
            "window": string_enum(&["15m", "1h", "6h", "24h", "7d"]),
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 1, "maximum": 500 },
            "sort": open_schema()
        },
        "required": ["limit"]
    })
}

fn event_type_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "ownerContractId": { "type": "string" },
            "ownerEventName": { "type": "string" }
        },
        "required": ["ownerContractId", "ownerEventName"]
    })
}

fn event_row_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "eventId": { "type": "string" },
            "eventTime": { "type": "string" },
            "streamSequence": { "type": "integer" },
            "subject": { "type": "string" },
            "ownerContractId": { "type": "string" },
            "ownerEventName": { "type": "string" },
            "resolution": string_enum(&["resolved", "unresolved", "malformed"]),
            "verificationStatus": verification_status_schema(),
            "publisherKind": string_enum(&["service", "device", "user"]),
            "publisherDeploymentId": { "type": "string" },
            "publisherInstanceId": { "type": "string" },
            "publisherParticipantId": { "type": "string" },
            "publisherParticipantDigest": { "type": "string" },
            "traceId": { "type": "string" },
            "payloadSizeBytes": { "type": "integer" },
            "headerCount": { "type": "integer" }
        },
        "required": ["eventId", "eventTime", "streamSequence", "subject", "resolution", "verificationStatus", "payloadSizeBytes", "headerCount"]
    })
}

fn verification_status_schema() -> Value {
    string_enum(&["verified"])
}

fn event_inspect_request_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "eventId": { "type": "string" }, "streamSequence": { "type": "integer" } }
    })
}

fn metrics_request_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "window": string_enum(&["15m", "1h", "6h", "24h", "7d"]) }
    })
}

fn metrics_response_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "summary": {
                "type": "object",
                "properties": {
                    "total": { "type": "integer" },
                    "uniqueSubjects": { "type": "integer" },
                    "payloadSizeBytes": { "type": "integer" },
                    "integrityExceptions": { "type": "integer" },
                    "byResolution": resolution_counts_schema(),
                    "byVerificationStatus": verification_counts_schema(),
                    "eventTypes": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "ownerContractId": { "type": "string" },
                                "ownerEventName": { "type": "string" },
                                "count": { "type": "integer" }
                            },
                            "required": ["ownerContractId", "ownerEventName", "count"]
                        }
                    }
                },
                "required": ["total", "uniqueSubjects", "payloadSizeBytes", "integrityExceptions", "byResolution", "byVerificationStatus", "eventTypes"]
            },
            "buckets": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "start": { "type": "string" },
                        "total": { "type": "integer" },
                        "payloadSizeBytes": { "type": "integer" },
                        "integrityExceptions": { "type": "integer" },
                        "byResolution": resolution_counts_schema(),
                        "byVerificationStatus": verification_counts_schema()
                    },
                    "required": ["start", "total", "payloadSizeBytes", "integrityExceptions", "byResolution", "byVerificationStatus"]
                }
            }
        },
        "required": ["summary", "buckets"]
    })
}

fn resolution_counts_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "resolved": { "type": "integer" },
            "unresolved": { "type": "integer" },
            "malformed": { "type": "integer" }
        }
    })
}

fn verification_counts_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "verified": { "type": "integer" },
            "missing-proof": { "type": "integer" },
            "invalid-signature": { "type": "integer" },
            "missing-session": { "type": "integer" },
            "subject-denied": { "type": "integer" },
            "outside-session-window": { "type": "integer" },
            "auth-unavailable": { "type": "integer" }
        }
    })
}

fn consumers_query_request_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "deploymentId": { "type": "string" },
            "contractId": { "type": "string" },
            "ownerContractId": { "type": "string" },
            "subject": { "type": "string" },
            "status": { "type": "array", "items": consumer_status_schema() },
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 1, "maximum": 500 }
        },
        "required": ["limit"]
    })
}

fn consumers_inspect_request_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "consumerName": { "type": "string" }, "stream": { "type": "string" } },
        "required": ["consumerName"]
    })
}

fn consumer_row_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "deploymentId": { "type": "string" },
            "contractId": { "type": "string" },
            "group": { "type": "string" },
            "stream": { "type": "string" },
            "consumerName": { "type": "string" },
            "filterSubjects": { "type": "array", "items": { "type": "string" } },
            "status": consumer_status_schema(),
            "managedBy": string_enum(&["authority", "platform", "external"]),
            "pending": { "type": "integer" },
            "ackPending": { "type": "integer" },
            "waitingPulls": { "type": "integer" },
            "redelivered": { "type": "integer" },
            "ackWaitMs": { "type": "integer" },
            "maxDeliver": { "type": "integer" },
            "oldestPendingAt": { "type": "string" },
            "oldestPendingEventId": { "type": "string" }
        },
        "required": ["stream", "consumerName", "filterSubjects", "status", "pending", "ackPending", "waitingPulls"]
    })
}

fn consumer_status_schema() -> Value {
    string_enum(&[
        "current",
        "processing",
        "behind",
        "saturated",
        "inactive",
        "failing",
        "missing",
        "orphaned",
    ])
}
