//! Router construction for the built-in Event Log runtime.

use trellis_rs::service::{DeclaredRpcError, Router, ServerError};
use trellis_runtime_apis::apis::trellis_events_v1::rpc;

use crate::query::{EventLogQuery, EventLogQueryError};
use crate::wire::{generated_input, generated_output};

/// Build an Event Log RPC router backed by a SQL projection query adapter.
pub fn build_router_with_query(query: EventLogQuery) -> Router {
    let mut router = Router::new();
    router.register_rpc::<rpc::Query, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move {
                let input = generated_input(input, &["limit", "offset"])?;
                let output = query.query_events(&input).await.map_err(map_query_error)?;
                generated_output(
                    output,
                    &[
                        "headerCount",
                        "limit",
                        "offset",
                        "payloadSizeBytes",
                        "streamSequence",
                        "total",
                    ],
                )
            }
        }
    });
    router.register_rpc::<rpc::Inspect, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move {
                let input = generated_input(input, &["streamSequence"])?;
                let output = query.inspect_event(&input).await.map_err(map_query_error)?;
                generated_output(output, &[])
            }
        }
    });
    router.register_rpc::<rpc::Metrics, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move {
                let input = generated_input(input, &[])?;
                let output = query.metrics(&input).await.map_err(map_query_error)?;
                generated_output(
                    output,
                    &[
                        "authUnavailable",
                        "count",
                        "integrityExceptions",
                        "invalidSignature",
                        "malformed",
                        "missingProof",
                        "missingSession",
                        "outsideSessionWindow",
                        "payloadSizeBytes",
                        "resolved",
                        "subjectDenied",
                        "total",
                        "uniqueSubjects",
                        "unresolved",
                        "verified",
                    ],
                )
            }
        }
    });
    router.register_rpc::<rpc::ConsumersQuery, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move {
                let input = generated_input(input, &["limit", "offset"])?;
                let output = query
                    .query_consumers(&input)
                    .await
                    .map_err(map_query_error)?;
                generated_output(
                    output,
                    &[
                        "ackPending",
                        "ackWaitMs",
                        "limit",
                        "maxDeliver",
                        "offset",
                        "pending",
                        "redelivered",
                        "total",
                        "waitingPulls",
                    ],
                )
            }
        }
    });
    router.register_rpc::<rpc::ConsumersInspect, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move {
                let input = generated_input(input, &[])?;
                let output = query
                    .inspect_consumer(&input)
                    .await
                    .map_err(map_query_error)?;
                generated_output(output, &[])
            }
        }
    });
    router
}

fn map_query_error(error: EventLogQueryError) -> ServerError {
    match error {
        EventLogQueryError::EventNotFound => ServerError::DeclaredRpc(DeclaredRpcError::new(
            "NotFoundError",
            "Event not found",
            [("resource", serde_json::json!("Event"))],
        )),
        EventLogQueryError::ConsumerNotFound(name) => {
            ServerError::DeclaredRpc(DeclaredRpcError::new(
                "NotFoundError",
                format!("Consumer '{name}' not found"),
                [
                    ("resource", serde_json::json!("Consumer")),
                    ("consumerName", serde_json::json!(name)),
                ],
            ))
        }
        EventLogQueryError::Validation { field, details } => {
            ServerError::DeclaredRpc(DeclaredRpcError::new(
                "ValidationError",
                format!("Invalid {field}: {details}"),
                [
                    ("field", serde_json::json!(field)),
                    ("details", serde_json::json!(details)),
                ],
            ))
        }
        other => ServerError::Nats(format!("event log RPC query failed: {other}")),
    }
}
