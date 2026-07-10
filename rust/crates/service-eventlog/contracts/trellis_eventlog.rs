//! Rust source for the `trellis.eventlog@v1` contract manifest.

use serde_json::{json, Value};
use trellis_contracts::{
    ContractCapabilityMetadata, ContractKind, ContractManifest, ContractManifestBuilder,
    ContractUseRef, ContractUseRpc, ContractsError,
};

const READ_CAPABILITY: &str = "events.read";
const STREAM_CAPABILITY: &str = "events.stream";
const UNEXPECTED_ERROR: &str = "UnexpectedError";
const VALIDATION_ERROR: &str = "ValidationError";
const NOT_FOUND_ERROR: &str = "NotFoundError";

/// Build the canonical Event Log contract manifest.
pub fn contract_manifest() -> Result<ContractManifest, ContractsError> {
    let mut manifest = ContractManifestBuilder::new(
        "trellis.eventlog@v1",
        "Trellis Event Log",
        "Read-only Event Log API for Trellis event stream observability.",
        ContractKind::Service,
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
    .use_ref(
        "auth",
        ContractUseRef {
            contract: "trellis.auth@v1".to_string(),
            rpc: Some(ContractUseRpc {
                call: Some(vec!["Auth.EventConsumers.List".to_string()]),
            }),
            operations: None,
            events: None,
            feeds: None,
        },
    )
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
    .build()?;

    manifest.uses.required_mut().remove("core");
    manifest.uses.optional_mut().remove("core");
    manifest.uses.optional_mut().remove("auth");
    Ok(manifest)
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
        "additionalProperties": false,
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
        "additionalProperties": false,
        "properties": {
            "search": { "type": "string" },
            "subject": { "type": "string" },
            "ownerContractId": { "type": "string" },
            "ownerEventName": { "type": "string" },
            "includeEventTypes": { "type": "array", "items": event_type_schema() },
            "excludeEventTypes": { "type": "array", "items": event_type_schema() },
            "publisherDeploymentId": { "type": "string" },
            "publisherContractId": { "type": "string" },
            "consumerDeploymentId": { "type": "string" },
            "resolution": { "type": "array", "items": string_enum(&["resolved", "unresolved", "malformed"]) },
            "verificationStatus": { "type": "array", "items": verification_status_schema() },
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
        "additionalProperties": false,
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
        "additionalProperties": false,
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
            "publisherContractId": { "type": "string" },
            "publisherContractDigest": { "type": "string" },
            "traceId": { "type": "string" },
            "payloadSizeBytes": { "type": "integer" },
            "headerCount": { "type": "integer" }
        },
        "required": ["eventId", "eventTime", "streamSequence", "subject", "resolution", "verificationStatus", "payloadSizeBytes", "headerCount"]
    })
}

fn verification_status_schema() -> Value {
    string_enum(&[
        "verified",
        "missing-proof",
        "invalid-signature",
        "missing-session",
        "subject-denied",
        "outside-session-window",
        "auth-unavailable",
    ])
}

fn event_inspect_request_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": { "eventId": { "type": "string" }, "streamSequence": { "type": "integer" } }
    })
}

fn metrics_request_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": { "window": string_enum(&["15m", "1h", "6h", "24h", "7d"]) }
    })
}

fn metrics_response_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "summary": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "total": { "type": "integer" },
                    "uniqueSubjects": { "type": "integer" },
                    "payloadSizeBytes": { "type": "integer" },
                    "byResolution": {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "resolved": { "type": "integer" },
                            "unresolved": { "type": "integer" },
                            "malformed": { "type": "integer" }
                        }
                    },
                    "byVerificationStatus": {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "verified": { "type": "integer" },
                            "missing-proof": { "type": "integer" },
                            "invalid-signature": { "type": "integer" },
                            "missing-session": { "type": "integer" },
                            "subject-denied": { "type": "integer" },
                            "outside-session-window": { "type": "integer" },
                            "auth-unavailable": { "type": "integer" }
                        }
                    },
                    "eventTypes": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "additionalProperties": false,
                            "properties": {
                                "ownerContractId": { "type": "string" },
                                "ownerEventName": { "type": "string" },
                                "count": { "type": "integer" }
                            },
                            "required": ["ownerContractId", "ownerEventName", "count"]
                        }
                    }
                },
                "required": ["total", "uniqueSubjects", "payloadSizeBytes", "byResolution", "byVerificationStatus", "eventTypes"]
            },
            "buckets": { "type": "array", "items": open_schema() }
        },
        "required": ["summary", "buckets"]
    })
}

fn consumers_query_request_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
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
        "additionalProperties": false,
        "properties": { "consumerName": { "type": "string" }, "stream": { "type": "string" } },
        "required": ["consumerName"]
    })
}

fn consumer_row_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "deploymentId": { "type": "string" },
            "contractId": { "type": "string" },
            "group": { "type": "string" },
            "stream": { "type": "string" },
            "consumerName": { "type": "string" },
            "filterSubjects": { "type": "array", "items": { "type": "string" } },
            "status": consumer_status_schema(),
            "pending": { "type": "integer" },
            "ackPending": { "type": "integer" },
            "waitingPulls": { "type": "integer" },
            "redelivered": { "type": "integer" },
            "concurrency": { "type": "integer" },
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
