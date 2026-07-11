//! Rust source for the `trellis.health@v1` contract manifest.

use serde_json::{json, Value};
use trellis_contracts::{
    ContractCapabilityMetadata, ContractKind, ContractManifest, ContractManifestBuilder,
    ContractsError,
};

const READ_CAPABILITY: &str = "read";
const UNEXPECTED_ERROR: &str = "UnexpectedError";
const VALIDATION_ERROR: &str = "ValidationError";
const NOT_FOUND_ERROR: &str = "NotFoundError";

/// Build the canonical Health service contract manifest.
pub fn contract_manifest() -> Result<ContractManifest, ContractsError> {
    ContractManifestBuilder::new(
        "trellis.health@v1",
        "Trellis Health",
        "Trellis-managed participant health projection and operational history.",
        ContractKind::Service,
    )
    .docs_with_summary(
        "Participant health administration APIs.",
        "Provides current participant health, instance inspection, bounded metrics, invalidation feeds, and durable status transitions. Periodic heartbeat samples use a private runtime transport and are not contract events.",
    )
    .capability(
        READ_CAPABILITY,
        ContractCapabilityMetadata {
            display_name: "Read participant health".to_string(),
            description: "View current and historical participant health state.".to_string(),
            consequence: None,
        },
    )
    .schema("HealthHeartbeatSample", heartbeat_sample_schema())
    .schema("HealthProjectionDiagnostics", projection_diagnostics_schema())
    .schema("HealthQueryRequest", query_request_schema())
    .schema("HealthQueryResponse", query_response_schema())
    .schema("HealthInspectRequest", inspect_request_schema())
    .schema("HealthInspectResponse", inspect_response_schema())
    .schema("HealthMetricsRequest", metrics_request_schema())
    .schema("HealthMetricsResponse", metrics_response_schema())
    .schema("HealthWatchRequest", watch_request_schema())
    .schema("HealthWatchFrame", watch_frame_schema())
    .schema("HealthStatusChangedEvent", status_changed_event_schema())
    .schema("NotFoundErrorData", not_found_error_schema())
    .export_schema("HealthHeartbeatSample")
    .error(NOT_FOUND_ERROR, NOT_FOUND_ERROR, "NotFoundErrorData")
    .rpc(
        "Health.Query",
        health_rpc("Health.Query", "HealthQueryRequest", "HealthQueryResponse")
            .docs_with_summary(
                "Query participant health.",
                "Returns a server-authoritative, paginated health summary grouped by participant contract.",
            )
            .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR]),
    )
    .rpc(
        "Health.Inspect",
        health_rpc(
            "Health.Inspect",
            "HealthInspectRequest",
            "HealthInspectResponse",
        )
        .docs_with_summary(
            "Inspect participant health.",
            "Returns latest instance samples and bounded status intervals for one participant contract.",
        )
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR, NOT_FOUND_ERROR]),
    )
    .rpc(
        "Health.Metrics",
        health_rpc(
            "Health.Metrics",
            "HealthMetricsRequest",
            "HealthMetricsResponse",
        )
        .docs_with_summary(
            "Read participant health metrics.",
            "Returns time-bucketed availability, status duration, sample, check, and latency aggregates.",
        )
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR]),
    )
    .feed(
        "Health.Watch",
        trellis_contracts::feed(
            "v1",
            "feed.v1.Health.Watch",
            "HealthWatchRequest",
            "HealthWatchFrame",
        )
        .with_subscribe_capabilities([READ_CAPABILITY])
        .docs_with_summary(
            "Watch health projection invalidations.",
            "Streams projection revisions and affected participant identities so clients can refresh authoritative snapshots.",
        ),
    )
    .event(
        "Health.StatusChanged",
        trellis_contracts::event(
            "v1",
            "events.v1.Health.StatusChanged",
            "HealthStatusChangedEvent",
        )
        .with_subscribe_capabilities([READ_CAPABILITY])
        .docs_with_summary(
            "Observe effective health transitions.",
            "Emitted only when an instance effective status changes; periodic heartbeat samples are not emitted as events.",
        ),
    )
    .build()
}

fn health_rpc(
    name: &str,
    input_schema: &str,
    output_schema: &str,
) -> trellis_contracts::ContractRpcMethod {
    trellis_contracts::rpc("v1", format!("rpc.v1.{name}"), input_schema, output_schema)
        .with_call_capabilities([READ_CAPABILITY])
}

fn participant_kind_schema() -> Value {
    string_enum(&["service", "device"])
}

fn reported_status_schema() -> Value {
    string_enum(&["healthy", "degraded", "unhealthy"])
}

fn effective_status_schema() -> Value {
    string_enum(&["healthy", "degraded", "unhealthy", "offline"])
}

fn check_status_schema() -> Value {
    string_enum(&["ok", "failed"])
}

fn string_enum(values: &[&str]) -> Value {
    json!({
        "type": "string",
        "enum": values,
    })
}

fn event_header_schema() -> Value {
    json!({
        "type": "object",
        "required": ["id", "time"],
        "properties": {
            "id": { "type": "string", "minLength": 1, "maxLength": 128 },
            "time": { "type": "string", "format": "date-time" }
        }
    })
}

fn health_check_schema(include_info: bool) -> Value {
    let mut properties = serde_json::Map::from_iter([
        (
            "name".to_string(),
            json!({ "type": "string", "minLength": 1, "maxLength": 128 }),
        ),
        ("status".to_string(), check_status_schema()),
        (
            "latencyMs".to_string(),
            json!({ "type": "number", "minimum": 0, "maximum": 3_600_000 }),
        ),
        (
            "error".to_string(),
            json!({ "type": "string", "maxLength": 1024 }),
        ),
        (
            "summary".to_string(),
            json!({ "type": "string", "maxLength": 1024 }),
        ),
    ]);
    if include_info {
        properties.insert("info".to_string(), json!({ "type": "object" }));
    }
    json!({
        "type": "object",
        "required": ["name", "status", "latencyMs"],
        "properties": properties
    })
}

fn heartbeat_sample_schema() -> Value {
    json!({
        "type": "object",
        "required": ["sample", "participant", "reportedStatus", "checks"],
        "properties": {
            "sample": {
                "type": "object",
                "required": ["id", "time"],
                "properties": {
                    "id": {
                        "type": "string",
                        "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$"
                    },
                    "time": { "type": "string", "format": "date-time" }
                }
            },
            "participant": {
                "type": "object",
                "required": [
                    "name",
                    "kind",
                    "instanceId",
                    "contractId",
                    "contractDigest",
                    "startedAt",
                    "publishIntervalMs",
                    "runtime"
                ],
                "properties": {
                    "name": { "type": "string", "minLength": 1, "maxLength": 256 },
                    "kind": participant_kind_schema(),
                    "instanceId": { "type": "string", "minLength": 1, "maxLength": 128 },
                    "contractId": { "type": "string", "minLength": 1, "maxLength": 256 },
                    "contractDigest": { "type": "string", "minLength": 1, "maxLength": 256 },
                    "startedAt": { "type": "string", "format": "date-time" },
                    "publishIntervalMs": {
                        "type": "integer",
                        "minimum": 1000,
                        "maximum": 600000
                    },
                    "runtime": string_enum(&["deno", "node", "rust", "unknown"]),
                    "runtimeVersion": { "type": "string", "maxLength": 256 },
                    "version": { "type": "string", "maxLength": 256 },
                    "info": { "type": "object" }
                }
            },
            "reportedStatus": reported_status_schema(),
            "summary": { "type": "string", "maxLength": 1024 },
            "checks": {
                "type": "array",
                "maxItems": 64,
                "items": health_check_schema(true)
            }
        }
    })
}

fn projection_diagnostics_schema() -> Value {
    json!({
        "type": "object",
        "required": ["lastStreamSequence", "revision", "gapDetected"],
        "properties": {
            "lastStreamSequence": { "type": "integer", "minimum": 0 },
            "revision": { "type": "integer", "minimum": 0 },
            "gapDetected": { "type": "boolean" },
            "retainedFrom": { "type": "string", "format": "date-time" },
            "completeSince": { "type": "string", "format": "date-time" }
        }
    })
}

fn query_request_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "participantKinds": bounded_array(participant_kind_schema(), 2),
            "contractIds": bounded_string_array(100, 256),
            "deploymentIds": bounded_string_array(100, 128),
            "statuses": bounded_array(effective_status_schema(), 4),
            "search": { "type": "string", "maxLength": 256 },
            "limit": { "type": "integer", "minimum": 1, "maximum": 200 },
            "offset": { "type": "integer", "minimum": 0 }
        }
    })
}

fn query_response_schema() -> Value {
    json!({
        "type": "object",
        "required": ["entries", "count", "limit", "offset", "asOf", "projection"],
        "properties": {
            "entries": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": [
                        "participantKind",
                        "contractId",
                        "participantName",
                        "effectiveStatus",
                        "deploymentIds",
                        "contractDigests",
                        "onlineInstances",
                        "offlineInstances",
                        "lastSeenAt",
                        "versions",
                        "runtimes"
                    ],
                    "properties": {
                        "participantKind": participant_kind_schema(),
                        "contractId": { "type": "string" },
                        "participantName": { "type": "string" },
                        "effectiveStatus": effective_status_schema(),
                        "deploymentIds": {
                            "type": "array",
                            "items": { "type": "string", "minLength": 1, "maxLength": 128 },
                            "uniqueItems": true
                        },
                        "contractDigests": {
                            "type": "array",
                            "items": { "type": "string", "minLength": 1, "maxLength": 128 },
                            "uniqueItems": true
                        },
                        "onlineInstances": { "type": "integer", "minimum": 0 },
                        "offlineInstances": { "type": "integer", "minimum": 0 },
                        "lastSeenAt": { "type": "string", "format": "date-time" },
                        "versions": { "type": "array", "items": { "type": "string" } },
                        "runtimes": { "type": "array", "items": { "type": "string" } }
                    }
                }
            },
            "count": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 1 },
            "offset": { "type": "integer", "minimum": 0 },
            "asOf": { "type": "string", "format": "date-time" },
            "projection": projection_diagnostics_schema()
        }
    })
}

fn inspect_request_schema() -> Value {
    json!({
        "type": "object",
        "required": ["participantKind", "contractId"],
        "properties": {
            "participantKind": participant_kind_schema(),
            "contractId": { "type": "string", "minLength": 1, "maxLength": 256 },
            "instanceId": { "type": "string", "minLength": 1, "maxLength": 128 },
            "historySince": { "type": "string", "format": "date-time" },
            "historyLimit": { "type": "integer", "minimum": 1, "maximum": 500 }
        }
    })
}

fn inspect_response_schema() -> Value {
    json!({
        "type": "object",
        "required": ["participant", "instances", "history", "asOf", "projection"],
        "properties": {
            "participant": {
                "type": "object",
                "required": [
                    "participantKind",
                    "contractId",
                    "participantName",
                    "effectiveStatus",
                    "onlineInstances",
                    "offlineInstances"
                ],
                "properties": {
                    "participantKind": participant_kind_schema(),
                    "contractId": { "type": "string" },
                    "participantName": { "type": "string" },
                    "effectiveStatus": effective_status_schema(),
                    "onlineInstances": { "type": "integer", "minimum": 0 },
                    "offlineInstances": { "type": "integer", "minimum": 0 }
                }
            },
            "instances": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": [
                        "instanceId",
                        "deploymentId",
                        "contractDigest",
                        "reportedStatus",
                        "effectiveStatus",
                        "observedAt",
                        "heartbeatDeadline",
                        "ageMs",
                        "startedAt",
                        "latestSample"
                    ],
                    "properties": {
                        "instanceId": { "type": "string" },
                        "deploymentId": { "type": "string" },
                        "contractDigest": { "type": "string" },
                        "reportedStatus": reported_status_schema(),
                        "effectiveStatus": effective_status_schema(),
                        "observedAt": { "type": "string", "format": "date-time" },
                        "heartbeatDeadline": { "type": "string", "format": "date-time" },
                        "ageMs": { "type": "integer", "minimum": 0 },
                        "startedAt": { "type": "string", "format": "date-time" },
                        "latestSample": heartbeat_sample_schema()
                    }
                }
            },
            "history": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": [
                        "intervalId",
                        "instanceId",
                        "startedAt",
                        "reportedStatus",
                        "effectiveStatus",
                        "checks",
                        "reason"
                    ],
                    "properties": {
                        "intervalId": { "type": "integer", "minimum": 1 },
                        "instanceId": { "type": "string" },
                        "startedAt": { "type": "string", "format": "date-time" },
                        "endedAt": { "type": "string", "format": "date-time" },
                        "reportedStatus": reported_status_schema(),
                        "effectiveStatus": effective_status_schema(),
                        "checks": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["name", "status"],
                                "properties": {
                                    "name": { "type": "string" },
                                    "status": check_status_schema()
                                }
                            }
                        },
                        "reason": string_enum(&[
                            "first-sample",
                            "heartbeat-change",
                            "heartbeat-resumed",
                            "deadline-expired"
                        ])
                    }
                }
            },
            "asOf": { "type": "string", "format": "date-time" },
            "projection": projection_diagnostics_schema()
        }
    })
}

fn metrics_request_schema() -> Value {
    json!({
        "type": "object",
        "required": ["start", "end", "stepMs", "participantKind", "contractId"],
        "properties": {
            "start": { "type": "string", "format": "date-time" },
            "end": { "type": "string", "format": "date-time" },
            "stepMs": { "type": "integer", "minimum": 300000 },
            "participantKind": participant_kind_schema(),
            "contractId": { "type": "string", "minLength": 1, "maxLength": 256 },
            "instanceIds": bounded_string_array(100, 128),
            "checkNames": bounded_string_array(64, 128)
        }
    })
}

fn metrics_response_schema() -> Value {
    let bucket_schema = json!({
        "type": "object",
        "required": [
            "start",
            "end",
            "observedMs",
            "sampleCount",
            "healthyMs",
            "degradedMs",
            "unhealthyMs",
            "offlineMs",
            "checks"
        ],
        "properties": {
            "start": { "type": "string", "format": "date-time" },
            "end": { "type": "string", "format": "date-time" },
            "observedMs": { "type": "integer", "minimum": 0 },
            "sampleCount": { "type": "integer", "minimum": 0 },
            "healthyMs": { "type": "integer", "minimum": 0 },
            "degradedMs": { "type": "integer", "minimum": 0 },
            "unhealthyMs": { "type": "integer", "minimum": 0 },
            "offlineMs": { "type": "integer", "minimum": 0 },
            "checks": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": [
                        "name",
                        "sampleCount",
                        "okCount",
                        "failedCount",
                        "latencyAverageMs",
                        "latencyMaxMs"
                    ],
                    "properties": {
                        "name": { "type": "string" },
                        "sampleCount": { "type": "integer", "minimum": 0 },
                        "okCount": { "type": "integer", "minimum": 0 },
                        "failedCount": { "type": "integer", "minimum": 0 },
                        "latencyAverageMs": { "type": "number", "minimum": 0 },
                        "latencyMaxMs": { "type": "number", "minimum": 0 }
                    }
                }
            }
        }
    });
    json!({
        "type": "object",
        "required": ["series", "summary", "asOf", "projection"],
        "properties": {
            "series": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["participantKind", "contractId", "instanceId", "buckets"],
                    "properties": {
                        "participantKind": participant_kind_schema(),
                        "contractId": { "type": "string" },
                        "instanceId": { "type": "string" },
                        "buckets": { "type": "array", "items": bucket_schema }
                    }
                }
            },
            "summary": {
                "type": "object",
                "required": ["observedMs", "onlineMs", "sampleCount", "transitions"],
                "properties": {
                    "availability": { "type": "number", "minimum": 0, "maximum": 1 },
                    "observedMs": { "type": "integer", "minimum": 0 },
                    "onlineMs": { "type": "integer", "minimum": 0 },
                    "sampleCount": { "type": "integer", "minimum": 0 },
                    "transitions": { "type": "integer", "minimum": 0 }
                }
            },
            "asOf": { "type": "string", "format": "date-time" },
            "projection": projection_diagnostics_schema()
        }
    })
}

fn watch_request_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "participantKinds": bounded_array(participant_kind_schema(), 2),
            "contractIds": bounded_string_array(100, 256),
            "deploymentIds": bounded_string_array(100, 128),
            "instanceIds": bounded_string_array(100, 128)
        }
    })
}

fn watch_frame_schema() -> Value {
    json!({
        "anyOf": [
            {
                "type": "object",
                "required": ["type", "projectionRevision"],
                "properties": {
                    "type": { "const": "ready", "type": "string" },
                    "projectionRevision": { "type": "integer", "minimum": 0 }
                }
            },
            {
                "type": "object",
                "required": ["type", "projectionRevision"],
                "properties": {
                    "type": { "const": "healthInvalidated", "type": "string" },
                    "projectionRevision": { "type": "integer", "minimum": 0 },
                    "changes": {
                        "type": "array",
                        "maxItems": 100,
                        "items": {
                            "type": "object",
                            "required": [
                                "participantKind",
                                "contractId",
                                "instanceId",
                                "deploymentId"
                            ],
                            "properties": {
                                "participantKind": participant_kind_schema(),
                                "contractId": { "type": "string" },
                                "instanceId": { "type": "string" },
                                "deploymentId": { "type": "string" }
                            }
                        }
                    }
                }
            }
        ]
    })
}

fn status_changed_event_schema() -> Value {
    json!({
        "type": "object",
        "required": [
            "header",
            "participant",
            "previousStatus",
            "status",
            "reportedStatus",
            "reason",
            "changedAt",
            "lastSeenAt"
        ],
        "properties": {
            "header": event_header_schema(),
            "participant": {
                "type": "object",
                "required": ["kind", "contractId", "instanceId", "deploymentId", "name"],
                "properties": {
                    "kind": participant_kind_schema(),
                    "contractId": { "type": "string" },
                    "instanceId": { "type": "string" },
                    "deploymentId": { "type": "string" },
                    "name": { "type": "string" }
                }
            },
            "previousStatus": effective_status_schema(),
            "status": effective_status_schema(),
            "reportedStatus": reported_status_schema(),
            "reason": string_enum(&[
                "heartbeat-change",
                "heartbeat-resumed",
                "deadline-expired"
            ]),
            "changedAt": { "type": "string", "format": "date-time" },
            "lastSeenAt": { "type": "string", "format": "date-time" },
            "summary": { "type": "string", "maxLength": 1024 }
        }
    })
}

fn not_found_error_schema() -> Value {
    json!({
        "type": "object",
        "required": ["type", "resource", "id", "message"],
        "properties": {
            "type": { "const": "NotFoundError", "type": "string" },
            "resource": { "type": "string", "minLength": 1 },
            "id": { "type": "string", "minLength": 1 },
            "message": { "type": "string" },
            "context": { "type": "object", "patternProperties": { "^.*$": {} } },
            "traceId": { "type": "string" }
        }
    })
}

fn bounded_string_array(max_items: u64, max_length: u64) -> Value {
    bounded_array(
        json!({
            "type": "string",
            "minLength": 1,
            "maxLength": max_length,
        }),
        max_items,
    )
}

fn bounded_array(items: Value, max_items: u64) -> Value {
    json!({
        "type": "array",
        "maxItems": max_items,
        "uniqueItems": true,
        "items": items,
    })
}
