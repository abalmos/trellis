//! Router construction for the Event Log service.

use trellis_rs::service::{ConnectedServiceRuntime, DeclaredRpcError, Router, ServerError};

use crate::contract::EventLogContract;
use crate::query::{EventLogQuery, EventLogQueryError};
use crate::wire::{
    EventLogConsumersInspectRpc, EventLogConsumersQueryRpc, EventLogInspectRpc, EventLogMetricsRpc,
    EventLogQueryRpc,
};

/// Register Event Log RPC handlers on the high-level Trellis service runtime.
pub fn register_eventlog_rpc_handlers(
    runtime: &mut ConnectedServiceRuntime<EventLogContract>,
    query: EventLogQuery,
) {
    runtime.register_rpc::<EventLogQueryRpc, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move { query.query_events(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<EventLogInspectRpc, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move { query.inspect_event(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<EventLogMetricsRpc, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move { query.metrics(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<EventLogConsumersQueryRpc, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move { query.query_consumers(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<EventLogConsumersInspectRpc, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move {
                query
                    .inspect_consumer(&input)
                    .await
                    .map_err(map_query_error)
            }
        }
    });
}

/// Build an Event Log RPC router backed by a SQL projection query adapter.
pub fn build_router_with_query(query: EventLogQuery) -> Router {
    let mut router = Router::new();
    router.register_rpc::<EventLogQueryRpc, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move { query.query_events(&input).await.map_err(map_query_error) }
        }
    });
    router.register_rpc::<EventLogInspectRpc, _, _>({
        let query = query.clone();
        move |_ctx, input| {
            let query = query.clone();
            async move { query.inspect_event(&input).await.map_err(map_query_error) }
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
