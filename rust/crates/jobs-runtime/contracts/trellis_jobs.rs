//! Rust source for the `trellis.jobs@v1` contract manifest.

use serde_json::{json, Value};
use trellis_contracts::{
    ApiArtifact, ApiBuilder, ContractArtifacts, ContractBuilder, ContractCapabilityMetadata,
    ContractKind, ContractsError,
};

const READ_CAPABILITY: &str = "admin.read";
const MUTATE_CAPABILITY: &str = "admin.mutate";
const STREAM_CAPABILITY: &str = "admin.stream";
const UNEXPECTED_ERROR: &str = "UnexpectedError";
const VALIDATION_ERROR: &str = "ValidationError";
const NOT_FOUND_ERROR: &str = "NotFoundError";

/// Build the canonical Jobs admin API artifact.
pub fn api_artifact() -> Result<ApiArtifact, ContractsError> {
    ApiBuilder::authoring(
        "trellis.jobs@v1",
        "1.0.0",
        "Trellis Jobs",
        "Trellis-managed background job administration API.",
    )
    .docs_with_summary(
        "Background job administration APIs.",
        "Provides service, job, retry, cancel, and dead-letter queue RPCs for Trellis-managed background work.",
    )
    .capability(
        READ_CAPABILITY,
        ContractCapabilityMetadata {
            display_name: "Read jobs admin data".to_string(),
            description: "View Jobs services, jobs, and dead-letter queues.".to_string(),
            consequence: None,
        },
    )
    .capability(
        MUTATE_CAPABILITY,
        ContractCapabilityMetadata {
            display_name: "Mutate jobs admin data".to_string(),
            description: "Cancel, retry, replay, or dismiss Jobs service work items.".to_string(),
            consequence: Some("Can change background job execution state.".to_string()),
        },
    )
    .capability(
        STREAM_CAPABILITY,
        ContractCapabilityMetadata {
            display_name: "Stream jobs admin data".to_string(),
            description: "Subscribe to Jobs live workbench updates.".to_string(),
            consequence: None,
        },
    )
    .schema("Empty", empty_schema())
    .schema("JobState", job_state_schema())
    .schema("JobContext", job_context_schema())
    .schema("JobConcurrencyMetadata", job_concurrency_metadata_schema())
    .schema("JobQueuePolicyMetadata", job_queue_policy_metadata_schema())
    .schema("JobLogEntry", job_log_entry_schema())
    .schema("JobProgress", job_progress_schema())
    .schema("JobErrorDetail", job_error_detail_schema())
    .schema("JobTrigger", job_trigger_schema())
    .schema("JobLineage", job_lineage_schema())
    .schema("JobWaitTarget", job_wait_target_schema())
    .schema("JobWaitEdge", job_wait_edge_schema())
    .schema("Job", job_schema())
    .schema("JobsListServicesRequest", page_request_schema())
    .schema(
        "JobsListServicesResponse",
        jobs_list_services_response_schema(),
    )
    .schema("JobsQueryRequest", jobs_query_request_schema())
    .schema("JobsWorkbenchJobRow", jobs_workbench_job_row_schema())
    .schema("JobsWorkbenchGroup", jobs_workbench_group_schema())
    .schema("JobsWorkbenchStats", jobs_workbench_stats_schema())
    .schema("JobsQueryResponse", jobs_query_response_schema())
    .schema("JobsMetricsRequest", jobs_metrics_request_schema())
    .schema("JobsMetricsLatency", jobs_metrics_latency_schema())
    .schema("JobsMetricsSummaryGroup", jobs_metrics_summary_group_schema())
    .schema("JobsMetricsBucketGroup", jobs_metrics_bucket_group_schema())
    .schema("JobsMetricsBucket", jobs_metrics_bucket_schema())
    .schema("JobsMetricsResponse", jobs_metrics_response_schema())
    .schema("JobTimelineEvent", job_timeline_event_schema())
    .schema("JobsInspectRequest", job_identity_schema())
    .schema("JobsInspectResponse", jobs_inspect_response_schema())
    .schema("JobsWatchRequest", jobs_watch_request_schema())
    .schema("JobsWatchFrame", jobs_watch_frame_schema())
    .schema("JobsGetKeyRequest", jobs_get_key_request_schema())
    .schema("JobsGetKeyResponse", jobs_get_key_response_schema())
    .schema("JobsCancelRequest", job_admin_action_request_schema())
    .schema("JobsCancelResponse", job_response_schema())
    .schema("JobsRetryRequest", job_admin_action_request_schema())
    .schema("JobsRetryResponse", job_response_schema())
    .schema("JobsListDLQRequest", job_list_dlq_request_schema())
    .schema("JobsListDLQResponse", jobs_list_dlq_response_schema())
    .schema("JobsReplayDLQRequest", job_admin_action_request_schema())
    .schema("JobsReplayDLQResponse", job_response_schema())
    .schema("JobsDismissDLQRequest", job_admin_action_request_schema())
    .schema("JobsDismissDLQResponse", job_response_schema())
    .schema("NotFoundErrorData", not_found_error_schema())
    .error(NOT_FOUND_ERROR, NOT_FOUND_ERROR, "NotFoundErrorData")
    .rpc(
        "Jobs.ListServices",
        admin_rpc(
            "Jobs.ListServices",
            "JobsListServicesRequest",
            "JobsListServicesResponse",
            READ_CAPABILITY,
        )
        .docs_with_summary(
            "List job services.",
            "Lists services that own or execute background job queues.",
        )
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR]),
    )
    .rpc(
        "Jobs.Query",
        admin_rpc(
            "Jobs.Query",
            "JobsQueryRequest",
            "JobsQueryResponse",
            READ_CAPABILITY,
        )
        .docs_with_summary(
            "Query jobs workbench data.",
            "Returns filtered, sorted, grouped Jobs workbench rows and stats.",
        )
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR]),
    )
    .rpc(
        "Jobs.Metrics",
        admin_rpc(
            "Jobs.Metrics",
            "JobsMetricsRequest",
            "JobsMetricsResponse",
            READ_CAPABILITY,
        )
        .docs_with_summary(
            "Query jobs operational metrics.",
            "Returns grouped job health summaries and time buckets for operator dashboards.",
        )
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR]),
    )
    .rpc(
        "Jobs.Inspect",
        admin_rpc(
            "Jobs.Inspect",
            "JobsInspectRequest",
            "JobsInspectResponse",
            READ_CAPABILITY,
        )
        .docs_with_summary(
            "Inspect a job.",
            "Returns one job with timeline, attempts, related jobs, errors, trigger, and lineage details.",
        )
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR, NOT_FOUND_ERROR]),
    )
    .rpc(
        "Jobs.GetKey",
        admin_rpc(
            "Jobs.GetKey",
            "JobsGetKeyRequest",
            "JobsGetKeyResponse",
            READ_CAPABILITY,
        )
        .docs_with_summary(
            "Read keyed job concurrency state.",
            "Returns projection-backed keyed concurrency state for one service job key.",
        )
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR, NOT_FOUND_ERROR]),
    )
    .rpc(
        "Jobs.Cancel",
        admin_rpc(
            "Jobs.Cancel",
            "JobsCancelRequest",
            "JobsCancelResponse",
            MUTATE_CAPABILITY,
        )
        .docs_with_summary("Cancel a job.", "Requests cancellation for one background job.")
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR, NOT_FOUND_ERROR]),
    )
    .rpc(
        "Jobs.Retry",
        admin_rpc(
            "Jobs.Retry",
            "JobsRetryRequest",
            "JobsRetryResponse",
            MUTATE_CAPABILITY,
        )
        .docs_with_summary("Retry a job.", "Moves a failed job back into retry processing.")
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR, NOT_FOUND_ERROR]),
    )
    .rpc(
        "Jobs.ListDLQ",
        admin_rpc(
            "Jobs.ListDLQ",
            "JobsListDLQRequest",
            "JobsListDLQResponse",
            READ_CAPABILITY,
        )
        .docs_with_summary("List dead-letter jobs.", "Lists jobs currently in dead-letter queues.")
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR]),
    )
    .rpc(
        "Jobs.ReplayDLQ",
        admin_rpc(
            "Jobs.ReplayDLQ",
            "JobsReplayDLQRequest",
            "JobsReplayDLQResponse",
            MUTATE_CAPABILITY,
        )
        .docs_with_summary("Replay a dead-letter job.", "Moves one dead-letter job back to processing.")
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR, NOT_FOUND_ERROR]),
    )
    .rpc(
        "Jobs.DismissDLQ",
        admin_rpc(
            "Jobs.DismissDLQ",
            "JobsDismissDLQRequest",
            "JobsDismissDLQResponse",
            MUTATE_CAPABILITY,
        )
        .docs_with_summary("Dismiss a dead-letter job.", "Marks one dead-letter job as dismissed.")
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR, NOT_FOUND_ERROR]),
    )
    .feed(
        "Jobs.Watch",
        trellis_contracts::feed(
            "v1",
            "feed.v1.Jobs.Watch",
            "JobsWatchRequest",
            "JobsWatchFrame",
        )
        .docs_with_summary(
            "Watch jobs workbench changes.",
            "Streams invalidation frames for Jobs workbench queries and job inspect views.",
        )
        .with_subscribe_capabilities([STREAM_CAPABILITY]),
    )
    .build()
}

/// Build the native Jobs participant and API artifacts.
pub fn contract_artifacts() -> Result<ContractArtifacts, ContractsError> {
    let api = api_artifact()?.normalized_value()?;
    ContractBuilder::from_api("trellis.jobs@v1", api, ContractKind::Service)?.build()
}

fn admin_rpc(
    name: &str,
    input_schema: &str,
    output_schema: &str,
    capability: &str,
) -> trellis_contracts::ContractRpcMethod {
    trellis_contracts::rpc("v1", format!("rpc.v1.{name}"), input_schema, output_schema)
        .with_call_capabilities([capability])
}

fn empty_schema() -> Value {
    json!({
        "type": "object",
        "properties": {}
    })
}

fn job_state_schema() -> Value {
    json!({
        "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "retry", "type": "string" },
            { "const": "completed", "type": "string" },
            { "const": "failed", "type": "string" },
            { "const": "cancelled", "type": "string" },
            { "const": "skipped", "type": "string" },
            { "const": "stale", "type": "string" },
            { "const": "expired", "type": "string" },
            { "const": "dead", "type": "string" },
            { "const": "dismissed", "type": "string" }
        ]
    })
}

fn job_log_entry_schema() -> Value {
    json!({
        "type": "object",
        "required": ["timestamp", "level", "message"],
        "properties": {
            "timestamp": { "type": "string", "format": "date-time" },
            "level": {
                "anyOf": [
                    { "const": "info", "type": "string" },
                    { "const": "warn", "type": "string" },
                    { "const": "error", "type": "string" }
                ]
            },
            "message": { "type": "string" }
        }
    })
}

fn job_progress_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "step": { "type": "string" },
            "current": { "type": "integer", "minimum": 0 },
            "total": { "type": "integer", "minimum": 0 },
            "message": { "type": "string" }
        }
    })
}

fn job_error_detail_schema() -> Value {
    json!({
        "type": "object",
        "required": ["message", "fingerprint"],
        "properties": {
            "message": { "type": "string" },
            "type": { "type": "string" },
            "stack": { "type": "string" },
            "causes": { "type": "array", "items": { "type": "object" } },
            "fingerprint": { "type": "string", "minLength": 1 },
            "firstSeen": { "type": "string", "format": "date-time" },
            "occurrenceCount": { "type": "integer", "minimum": 0 },
            "worker": {
                "type": "object",
                "properties": {
                    "service": { "type": "string" },
                    "instanceId": { "type": "string" },
                    "version": { "type": "string" },
                    "runtime": { "type": "string" }
                }
            }
        }
    })
}

fn job_trigger_schema() -> Value {
    json!({
        "type": "object",
        "required": ["kind"],
        "properties": {
            "kind": {
                "anyOf": [
                    { "const": "schedule", "type": "string" },
                    { "const": "operation", "type": "string" },
                    { "const": "rpc", "type": "string" },
                    { "const": "event", "type": "string" },
                    { "const": "manualReplay", "type": "string" },
                    { "const": "serviceCode", "type": "string" },
                    { "const": "parentJob", "type": "string" }
                ]
            },
            "id": { "type": "string" },
            "subject": { "type": "string" },
            "operationId": { "type": "string" },
            "parentJobId": { "type": "string" },
            "traceId": { "type": "string" },
            "requestId": { "type": "string" }
        }
    })
}

fn job_lineage_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "parentJobId": { "type": "string" },
            "rootJobId": { "type": "string" },
            "operationId": { "type": "string" },
            "relatedKeys": { "type": "array", "items": { "type": "string" } }
        }
    })
}

fn job_context_schema() -> Value {
    json!({
        "type": "object",
        "required": ["requestId", "traceId", "traceparent"],
        "properties": {
            "requestId": { "type": "string", "minLength": 1 },
            "traceId": { "type": "string", "pattern": "^[0-9a-f]{32}$" },
            "traceparent": {
                "type": "string",
                "pattern": "^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$"
            },
            "tracestate": { "type": "string", "minLength": 1 }
        }
    })
}

fn job_wait_target_schema() -> Value {
    json!({
        "type": "object",
        "required": ["kind"],
        "properties": {
            "kind": {
                "anyOf": [
                    { "const": "job", "type": "string" },
                    { "const": "operation", "type": "string" },
                    { "const": "external", "type": "string" }
                ]
            },
            "id": { "type": "string", "minLength": 1 },
            "operationId": { "type": "string", "minLength": 1 },
            "service": { "type": "string", "minLength": 1 },
            "system": { "type": "string", "minLength": 1 },
            "type": { "type": "string", "minLength": 1 },
            "operation": { "type": "string", "minLength": 1 },
            "key": { "type": "string", "minLength": 1 },
            "label": { "type": "string", "minLength": 1 }
        }
    })
}

fn job_wait_edge_schema() -> Value {
    json!({
        "type": "object",
        "required": ["id", "target", "startedAt"],
        "properties": {
            "id": { "type": "string", "minLength": 1 },
            "target": job_wait_target_schema(),
            "startedAt": { "type": "string", "format": "date-time" },
            "label": { "type": "string", "minLength": 1 }
        }
    })
}

fn job_schema() -> Value {
    json!({
        "type": "object",
        "required": [
            "id",
            "context",
            "service",
            "type",
            "state",
            "payload",
            "createdAt",
            "updatedAt",
            "tries",
            "maxTries"
        ],
        "properties": {
            "id": { "type": "string", "minLength": 1 },
            "context": job_context_schema(),
            "concurrency": job_concurrency_metadata_schema(),
            "service": { "type": "string", "minLength": 1 },
            "type": { "type": "string", "minLength": 1 },
            "state": job_state_schema(),
            "payload": {},
            "result": {},
            "createdAt": { "type": "string", "format": "date-time" },
            "updatedAt": { "type": "string", "format": "date-time" },
            "startedAt": { "type": "string", "format": "date-time" },
            "completedAt": { "type": "string", "format": "date-time" },
            "tries": { "type": "integer", "minimum": 0 },
            "maxTries": { "type": "integer", "minimum": 1 },
            "lastError": { "type": "string" },
            "errorDetail": job_error_detail_schema(),
            "deadline": { "type": "string", "format": "date-time" },
            "progress": job_progress_schema(),
            "trigger": job_trigger_schema(),
            "lineage": job_lineage_schema(),
            "waitingOn": { "type": "array", "items": job_wait_edge_schema() },
            "queuePolicy": job_queue_policy_metadata_schema(),
            "logs": { "type": "array", "items": job_log_entry_schema() }
        }
    })
}

fn job_concurrency_metadata_schema() -> Value {
    json!({
        "type": "object",
        "required": ["key", "keyHash"],
        "properties": {
            "key": { "type": "string", "minLength": 1 },
            "keyHash": { "type": "string", "minLength": 1 },
            "heartbeatAt": { "type": "string", "format": "date-time" },
            "leaseExpiresAt": { "type": "string", "format": "date-time" },
            "staleTakeoverCount": { "type": "integer", "minimum": 0 }
        }
    })
}

fn job_queue_policy_metadata_schema() -> Value {
    json!({
        "type": "object",
        "required": ["outcome"],
        "properties": {
            "outcome": { "type": "string", "minLength": 1 },
            "reason": { "type": "string", "minLength": 1 },
            "existingJobId": { "type": "string", "minLength": 1 },
            "replacedJobId": { "type": "string", "minLength": 1 }
        }
    })
}

fn job_identity_schema() -> Value {
    json!({
        "type": "object",
        "description": "Jobs admin ids are globally addressable; callers identify jobs by id only.",
        "required": ["id"],
        "properties": {
            "id": { "type": "string", "minLength": 1 }
        }
    })
}

fn job_admin_action_request_schema() -> Value {
    json!({
        "type": "object",
        "description": "Jobs admin ids are globally addressable; callers identify jobs by id only.",
        "required": ["id"],
        "properties": {
            "id": { "type": "string", "minLength": 1 },
            "reason": { "type": "string", "minLength": 1 }
        }
    })
}

fn jobs_query_request_schema() -> Value {
    json!({
        "type": "object",
        "required": ["limit"],
        "properties": {
            "service": { "type": "string", "minLength": 1 },
            "type": { "type": "string", "minLength": 1 },
            "state": { "type": "array", "items": job_state_schema() },
            "search": { "type": "string" },
            "queueKey": { "type": "string" },
            "trigger": { "type": "string" },
            "runtimeBand": {
                "anyOf": [
                    { "const": "queued", "type": "string" },
                    { "const": "running", "type": "string" },
                    { "const": "slow", "type": "string" },
                    { "const": "terminal", "type": "string" }
                ]
            },
            "groupBy": {
                "anyOf": [
                    { "const": "service", "type": "string" },
                    { "const": "type", "type": "string" },
                    { "const": "state", "type": "string" },
                    { "const": "queueKey", "type": "string" },
                    { "const": "trigger", "type": "string" },
                    { "const": "runtimeBand", "type": "string" }
                ]
            },
            "sort": {
                "type": "object",
                "required": ["field"],
                "properties": {
                    "field": {
                        "anyOf": [
                            { "const": "updatedAt", "type": "string" },
                            { "const": "queueAge", "type": "string" },
                            { "const": "runtime", "type": "string" },
                            { "const": "failureRate", "type": "string" },
                            { "const": "retries", "type": "string" },
                            { "const": "depth", "type": "string" }
                        ]
                    },
                    "direction": {
                        "anyOf": [
                            { "const": "asc", "type": "string" },
                            { "const": "desc", "type": "string" }
                        ]
                    }
                }
            },
            "window": {
                "anyOf": [
                    { "const": "1h", "type": "string" },
                    { "const": "24h", "type": "string" },
                    { "const": "7d", "type": "string" }
                ]
            },
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 1 }
        }
    })
}

fn jobs_metrics_request_schema() -> Value {
    json!({
        "type": "object",
        "required": ["window", "step", "groupBy"],
        "properties": {
            "service": { "type": "string", "minLength": 1 },
            "type": { "type": "string", "minLength": 1 },
            "state": { "type": "array", "items": job_state_schema() },
            "queueKey": { "type": "string" },
            "trigger": { "type": "string" },
            "window": {
                "anyOf": [
                    { "const": "15m", "type": "string" },
                    { "const": "1h", "type": "string" },
                    { "const": "6h", "type": "string" },
                    { "const": "24h", "type": "string" },
                    { "const": "7d", "type": "string" }
                ]
            },
            "step": {
                "anyOf": [
                    { "const": "1m", "type": "string" },
                    { "const": "5m", "type": "string" },
                    { "const": "15m", "type": "string" },
                    { "const": "1h", "type": "string" },
                    { "const": "6h", "type": "string" },
                    { "const": "1d", "type": "string" }
                ]
            },
            "groupBy": {
                "anyOf": [
                    { "const": "type", "type": "string" },
                    { "const": "service", "type": "string" },
                    { "const": "queueKey", "type": "string" },
                    { "const": "state", "type": "string" },
                    { "const": "trigger", "type": "string" }
                ]
            }
        }
    })
}

fn jobs_get_key_request_schema() -> Value {
    json!({
        "type": "object",
        "required": ["service", "type", "key"],
        "properties": {
            "service": { "type": "string", "minLength": 1 },
            "type": { "type": "string", "minLength": 1 },
            "key": { "type": "string", "minLength": 1 }
        }
    })
}

fn job_list_dlq_request_schema() -> Value {
    json!({
        "type": "object",
        "required": ["limit"],
        "properties": {
            "service": { "type": "string", "minLength": 1 },
            "type": { "type": "string", "minLength": 1 },
            "since": { "type": "string", "format": "date-time" },
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 1 }
        }
    })
}

fn page_request_schema() -> Value {
    json!({
        "type": "object",
        "required": ["limit"],
        "properties": {
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 1 }
        }
    })
}

fn worker_schema() -> Value {
    json!({
        "type": "object",
        "required": ["service", "jobType", "instanceId", "timestamp"],
        "properties": {
            "service": { "type": "string", "minLength": 1 },
            "jobType": { "type": "string", "minLength": 1 },
            "instanceId": { "type": "string", "minLength": 1 },
            "timestamp": { "type": "string", "format": "date-time" },
            "concurrency": { "type": "integer", "minimum": 1 },
            "version": { "type": "string", "minLength": 1 }
        }
    })
}

fn jobs_list_services_response_schema() -> Value {
    page_response_schema(service_entry_schema())
}

fn jobs_list_dlq_response_schema() -> Value {
    page_response_schema(job_schema())
}

fn jobs_workbench_job_row_schema() -> Value {
    json!({
        "type": "object",
        "required": ["id", "service", "type", "state", "createdAt", "updatedAt", "tries", "maxTries"],
        "properties": {
            "id": { "type": "string", "minLength": 1 },
            "service": { "type": "string", "minLength": 1 },
            "type": { "type": "string", "minLength": 1 },
            "state": job_state_schema(),
            "createdAt": { "type": "string", "format": "date-time" },
            "updatedAt": { "type": "string", "format": "date-time" },
            "startedAt": { "type": "string", "format": "date-time" },
            "completedAt": { "type": "string", "format": "date-time" },
            "tries": { "type": "integer", "minimum": 0 },
            "maxTries": { "type": "integer", "minimum": 1 },
            "queueKey": { "type": "string" },
            "queueAgeMs": { "type": "integer", "minimum": 0 },
            "runtimeMs": { "type": "integer", "minimum": 0 },
            "runtimeBand": { "type": "string" },
            "trigger": job_trigger_schema(),
            "lineage": job_lineage_schema(),
            "waitingOn": { "type": "array", "items": job_wait_edge_schema() },
            "lastError": { "type": "string" },
            "errorFingerprint": { "type": "string" },
            "context": job_context_schema(),
            "progress": job_progress_schema()
        }
    })
}

fn jobs_inspect_related_job_row_schema() -> Value {
    let mut schema = jobs_workbench_job_row_schema();
    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        properties.insert(
            "matchedBy".to_string(),
            json!({
                "anyOf": [
                    { "const": "trace", "type": "string" },
                    { "const": "parent", "type": "string" },
                    { "const": "root", "type": "string" },
                    { "const": "operation", "type": "string" },
                    { "const": "concurrency", "type": "string" },
                    { "const": "wait", "type": "string" }
                ]
            }),
        );
    }
    schema
}

fn jobs_workbench_group_schema() -> Value {
    json!({
        "type": "object",
        "required": ["key", "label", "count"],
        "properties": {
            "key": { "type": "string" },
            "label": { "type": "string" },
            "count": { "type": "integer", "minimum": 0 },
            "state": job_state_schema(),
            "depth": { "type": "integer", "minimum": 0 },
            "failureRate": { "type": "number", "minimum": 0 },
            "oldestCreatedAt": { "type": "string", "format": "date-time" },
            "latestUpdatedAt": { "type": "string", "format": "date-time" }
        }
    })
}

fn jobs_workbench_stats_schema() -> Value {
    json!({
        "type": "object",
        "required": ["total", "byState"],
        "properties": {
            "total": { "type": "integer", "minimum": 0 },
            "byState": { "type": "object", "patternProperties": { "^.*$": { "type": "integer", "minimum": 0 } } },
            "running": { "type": "integer", "minimum": 0 },
            "queued": { "type": "integer", "minimum": 0 },
            "failed": { "type": "integer", "minimum": 0 },
            "dead": { "type": "integer", "minimum": 0 },
            "slow": { "type": "integer", "minimum": 0 }
        }
    })
}

fn jobs_query_response_schema() -> Value {
    json!({
        "type": "object",
        "required": ["entries", "groups", "stats", "count", "offset", "limit"],
        "properties": {
            "entries": { "type": "array", "items": jobs_workbench_job_row_schema() },
            "groups": { "type": "array", "items": jobs_workbench_group_schema() },
            "stats": jobs_workbench_stats_schema(),
            "count": { "type": "integer", "minimum": 0 },
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 1 },
            "nextOffset": { "type": "integer", "minimum": 0 }
        }
    })
}

fn jobs_metrics_latency_schema() -> Value {
    json!({
        "type": "object",
        "required": ["count"],
        "properties": {
            "count": { "type": "integer", "minimum": 0 },
            "p50Ms": { "type": "integer", "minimum": 0 },
            "p95Ms": { "type": "integer", "minimum": 0 },
            "maxMs": { "type": "integer", "minimum": 0 }
        }
    })
}

fn jobs_metrics_summary_group_schema() -> Value {
    json!({
        "type": "object",
        "required": ["key", "label", "total", "byState", "runtime", "queueWait"],
        "properties": {
            "key": { "type": "string" },
            "label": { "type": "string" },
            "total": { "type": "integer", "minimum": 0 },
            "byState": { "type": "object", "patternProperties": { "^.*$": { "type": "integer", "minimum": 0 } } },
            "running": { "type": "integer", "minimum": 0 },
            "queued": { "type": "integer", "minimum": 0 },
            "failed": { "type": "integer", "minimum": 0 },
            "dead": { "type": "integer", "minimum": 0 },
            "slow": { "type": "integer", "minimum": 0 },
            "failureRate": { "type": "number", "minimum": 0 },
            "runtime": jobs_metrics_latency_schema(),
            "queueWait": jobs_metrics_latency_schema(),
            "oldestCreatedAt": { "type": "string", "format": "date-time" },
            "latestUpdatedAt": { "type": "string", "format": "date-time" }
        }
    })
}

fn jobs_metrics_bucket_group_schema() -> Value {
    json!({
        "type": "object",
        "required": ["key", "label", "submitted", "started", "completed", "failed", "retried", "dead", "cancelled", "dismissed", "runtime", "queueWait"],
        "properties": {
            "key": { "type": "string" },
            "label": { "type": "string" },
            "submitted": { "type": "integer", "minimum": 0 },
            "started": { "type": "integer", "minimum": 0 },
            "completed": { "type": "integer", "minimum": 0 },
            "failed": { "type": "integer", "minimum": 0 },
            "retried": { "type": "integer", "minimum": 0 },
            "dead": { "type": "integer", "minimum": 0 },
            "cancelled": { "type": "integer", "minimum": 0 },
            "dismissed": { "type": "integer", "minimum": 0 },
            "runtime": jobs_metrics_latency_schema(),
            "queueWait": jobs_metrics_latency_schema()
        }
    })
}

fn jobs_metrics_bucket_schema() -> Value {
    json!({
        "type": "object",
        "required": ["start", "end", "groups"],
        "properties": {
            "start": { "type": "string", "format": "date-time" },
            "end": { "type": "string", "format": "date-time" },
            "groups": { "type": "array", "items": jobs_metrics_bucket_group_schema() }
        }
    })
}

fn jobs_metrics_response_schema() -> Value {
    json!({
        "type": "object",
        "required": ["window", "step", "groupBy", "generatedAt", "summary", "buckets"],
        "properties": {
            "window": { "type": "string" },
            "step": { "type": "string" },
            "groupBy": { "type": "string" },
            "generatedAt": { "type": "string", "format": "date-time" },
            "summary": { "type": "array", "items": jobs_metrics_summary_group_schema() },
            "buckets": { "type": "array", "items": jobs_metrics_bucket_schema() }
        }
    })
}

fn page_response_schema(entry: Value) -> Value {
    json!({
        "type": "object",
        "required": ["entries", "count", "offset", "limit"],
        "properties": {
            "entries": { "type": "array", "items": entry },
            "count": { "type": "integer", "minimum": 0 },
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 1 },
            "nextOffset": { "type": "integer", "minimum": 0 }
        }
    })
}

fn service_entry_schema() -> Value {
    json!({
        "type": "object",
        "required": ["name", "healthy", "workers"],
        "properties": {
            "name": { "type": "string", "minLength": 1 },
            "healthy": { "type": "boolean" },
            "workers": { "type": "array", "items": worker_schema() }
        }
    })
}

fn job_timeline_event_schema() -> Value {
    json!({
        "type": "object",
        "required": ["sequence", "type", "state", "timestamp"],
        "properties": {
            "sequence": { "type": "integer", "minimum": 0 },
            "type": { "type": "string", "minLength": 1 },
            "state": job_state_schema(),
            "previousState": job_state_schema(),
            "timestamp": { "type": "string", "format": "date-time" },
            "tries": { "type": "integer", "minimum": 0 },
            "message": { "type": "string" },
            "error": { "type": "string" },
            "errorDetail": job_error_detail_schema(),
            "progress": job_progress_schema(),
            "waitEdge": job_wait_edge_schema(),
            "logs": { "type": "array", "items": job_log_entry_schema() },
            "workerInstanceId": { "type": "string" },
            "projected": { "type": "boolean" },
            "reason": { "type": "string" },
            "rawEvent": {}
        }
    })
}

fn jobs_inspect_response_schema() -> Value {
    json!({
        "type": "object",
        "required": ["job", "timeline", "attempts", "related", "errors"],
        "properties": {
            "job": job_schema(),
            "timeline": { "type": "array", "items": job_timeline_event_schema() },
            "attempts": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["try", "startedAt"],
                    "properties": {
                        "try": { "type": "integer", "minimum": 0 },
                        "startedAt": { "type": "string", "format": "date-time" },
                        "endedAt": { "type": "string", "format": "date-time" },
                        "state": job_state_schema(),
                        "error": job_error_detail_schema()
                    }
                }
            },
            "related": { "type": "array", "items": jobs_inspect_related_job_row_schema() },
            "errors": { "type": "array", "items": job_error_detail_schema() },
            "trigger": job_trigger_schema(),
            "lineage": job_lineage_schema()
        }
    })
}

fn jobs_watch_request_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "jobId": { "type": "string", "minLength": 1 },
            "query": jobs_query_request_schema(),
            "includeInitial": { "type": "boolean" }
        }
    })
}

fn jobs_watch_frame_schema() -> Value {
    json!({
        "anyOf": [
            {
                "type": "object",
                "required": ["kind", "timestamp"],
                "properties": {
                    "kind": { "const": "ready", "type": "string" },
                    "timestamp": { "type": "string", "format": "date-time" }
                }
            },
            {
                "type": "object",
                "required": ["kind", "id", "service", "type", "state", "updatedAt"],
                "properties": {
                    "kind": { "const": "jobChanged", "type": "string" },
                    "id": { "type": "string", "minLength": 1 },
                    "service": { "type": "string", "minLength": 1 },
                    "type": { "type": "string", "minLength": 1 },
                    "state": job_state_schema(),
                    "updatedAt": { "type": "string", "format": "date-time" }
                }
            },
            {
                "type": "object",
                "required": ["kind", "reason", "timestamp"],
                "properties": {
                    "kind": { "const": "queryInvalidated", "type": "string" },
                    "reason": {
                        "anyOf": [
                            { "const": "matched-job-changed", "type": "string" },
                            { "const": "unknown-match", "type": "string" }
                        ]
                    },
                    "timestamp": { "type": "string", "format": "date-time" }
                }
            },
            {
                "type": "object",
                "required": ["kind", "id", "timestamp"],
                "properties": {
                    "kind": { "const": "jobInspectChanged", "type": "string" },
                    "id": { "type": "string", "minLength": 1 },
                    "timestamp": { "type": "string", "format": "date-time" }
                }
            }
        ]
    })
}

fn jobs_get_key_response_schema() -> Value {
    json!({
        "type": "object",
        "required": [
            "service",
            "type",
            "key",
            "keyHash",
            "active",
            "queued",
            "queuedDepth",
            "staleTakeoverCount"
        ],
        "properties": {
            "service": { "type": "string", "minLength": 1 },
            "type": { "type": "string", "minLength": 1 },
            "key": { "type": "string", "minLength": 1 },
            "keyHash": { "type": "string", "minLength": 1 },
            "active": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["jobId", "instanceId", "startedAt", "heartbeatAt", "heartbeatAgeMs", "leaseExpiresAt"],
                    "properties": {
                        "jobId": { "type": "string", "minLength": 1 },
                        "instanceId": { "type": "string" },
                        "startedAt": { "type": "string", "format": "date-time" },
                        "heartbeatAt": { "type": "string", "format": "date-time" },
                        "heartbeatAgeMs": { "type": "integer", "minimum": 0 },
                        "leaseExpiresAt": { "type": "string", "format": "date-time" }
                    }
                }
            },
            "queued": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["jobId", "createdAt"],
                    "properties": {
                        "jobId": { "type": "string", "minLength": 1 },
                        "createdAt": { "type": "string", "format": "date-time" }
                    }
                }
            },
            "queuedDepth": { "type": "integer", "minimum": 0 },
            "staleTakeoverCount": { "type": "integer", "minimum": 0 },
            "latestPolicyReason": { "type": "string", "minLength": 1 }
        }
    })
}

fn not_found_error_schema() -> Value {
    json!({
        "type": "object",
        "required": ["id", "type", "message", "resource"],
        "properties": {
            "id": { "type": "string", "minLength": 1 },
            "type": { "type": "string", "const": "NotFoundError" },
            "message": { "type": "string" },
            "resource": { "type": "string", "minLength": 1 },
            "jobId": { "type": "string", "minLength": 1 },
            "context": { "type": "object", "patternProperties": { "^.*$": {} } },
            "traceId": { "type": "string" }
        }
    })
}

fn job_response_schema() -> Value {
    json!({
        "type": "object",
        "required": ["job"],
        "properties": {
            "job": job_schema()
        }
    })
}
