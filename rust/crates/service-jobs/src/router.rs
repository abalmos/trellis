//! Router construction for the Jobs admin service.

use trellis_rs::sdk::jobs::rpc::{
    JobsCancelRpc, JobsDismissDLQRpc, JobsGetKeyRpc, JobsInspectRpc, JobsListDLQRpc,
    JobsListServicesRpc, JobsQueryRpc, JobsReplayDLQRpc, JobsRetryRpc,
};
use trellis_rs::sdk::jobs::types::{
    JobsCancelRequest, JobsDismissDLQRequest, JobsGetKeyRequest, JobsInspectRequest,
    JobsListDLQRequest, JobsListServicesRequest, JobsQueryRequest, JobsReplayDLQRequest,
    JobsRetryRequest,
};
use trellis_rs::service::{ConnectedServiceRuntime, DeclaredRpcError, Router, ServerError};

use crate::contract::JobsContract;
use crate::query::{JobsQuery, JobsQueryError};

/// Register Jobs admin RPC handlers on the high-level Trellis service runtime.
pub fn register_jobs_rpc_handlers(
    runtime: &mut ConnectedServiceRuntime<JobsContract>,
    query: JobsQuery,
) {
    runtime.register_rpc::<JobsListServicesRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsListServicesRequest| {
            let query = query.clone();
            async move { query.list_services(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<JobsQueryRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsQueryRequest| {
            let query = query.clone();
            async move { query.query_jobs(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<JobsInspectRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsInspectRequest| {
            let query = query.clone();
            async move { query.inspect(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<JobsGetKeyRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsGetKeyRequest| {
            let query = query.clone();
            async move { query.get_key(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<JobsCancelRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsCancelRequest| {
            let query = query.clone();
            async move { query.cancel_job(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<JobsRetryRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsRetryRequest| {
            let query = query.clone();
            async move { query.retry_job(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<JobsListDLQRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsListDLQRequest| {
            let query = query.clone();
            async move { query.list_dlq(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<JobsReplayDLQRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsReplayDLQRequest| {
            let query = query.clone();
            async move { query.replay_dlq(&input).await.map_err(map_query_error) }
        }
    });
    runtime.register_rpc::<JobsDismissDLQRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsDismissDLQRequest| {
            let query = query.clone();
            async move { query.dismiss_dlq(&input).await.map_err(map_query_error) }
        }
    });
}

/// Build the Jobs admin RPC router backed by a SQL projection query adapter.
pub fn build_router_with_query(query: JobsQuery) -> Router {
    let mut router = Router::new();
    router.register_rpc::<JobsListServicesRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsListServicesRequest| {
            let query = query.clone();
            async move { query.list_services(&input).await.map_err(map_query_error) }
        }
    });
    router.register_rpc::<JobsQueryRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsQueryRequest| {
            let query = query.clone();
            async move { query.query_jobs(&input).await.map_err(map_query_error) }
        }
    });
    router.register_rpc::<JobsInspectRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsInspectRequest| {
            let query = query.clone();
            async move { query.inspect(&input).await.map_err(map_query_error) }
        }
    });
    router.register_rpc::<JobsGetKeyRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsGetKeyRequest| {
            let query = query.clone();
            async move { query.get_key(&input).await.map_err(map_query_error) }
        }
    });
    router.register_rpc::<JobsCancelRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsCancelRequest| {
            let query = query.clone();
            async move { query.cancel_job(&input).await.map_err(map_query_error) }
        }
    });
    router.register_rpc::<JobsRetryRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsRetryRequest| {
            let query = query.clone();
            async move { query.retry_job(&input).await.map_err(map_query_error) }
        }
    });
    router.register_rpc::<JobsListDLQRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsListDLQRequest| {
            let query = query.clone();
            async move { query.list_dlq(&input).await.map_err(map_query_error) }
        }
    });
    router.register_rpc::<JobsReplayDLQRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsReplayDLQRequest| {
            let query = query.clone();
            async move { query.replay_dlq(&input).await.map_err(map_query_error) }
        }
    });
    router.register_rpc::<JobsDismissDLQRpc, _, _>({
        let query = query.clone();
        move |_ctx, input: JobsDismissDLQRequest| {
            let query = query.clone();
            async move { query.dismiss_dlq(&input).await.map_err(map_query_error) }
        }
    });
    router
}

fn map_query_error(error: JobsQueryError) -> ServerError {
    match error {
        JobsQueryError::JobNotFound { key } => ServerError::DeclaredRpc(DeclaredRpcError::new(
            "NotFoundError",
            format!("Job '{key}' not found"),
            [
                ("resource", serde_json::json!("Job")),
                ("jobId", serde_json::json!(key)),
            ],
        )),
        JobsQueryError::JobStateConflict {
            key,
            expected,
            actual,
        } => ServerError::DeclaredRpc(DeclaredRpcError::new(
            "ValidationError",
            format!("Job '{key}' is in state '{actual}', expected {expected}"),
            [
                ("field", serde_json::json!("state")),
                ("jobKey", serde_json::json!(key)),
                ("expected", serde_json::json!(expected)),
                ("actual", serde_json::json!(actual)),
            ],
        )),
        JobsQueryError::Validation { field, details } => {
            ServerError::DeclaredRpc(DeclaredRpcError::new(
                "ValidationError",
                format!("Invalid {field}: {details}"),
                [
                    ("field", serde_json::json!(field)),
                    ("details", serde_json::json!(details)),
                ],
            ))
        }
        JobsQueryError::ConvertWireModel { model, details } => {
            ServerError::DeclaredRpc(DeclaredRpcError::new(
                "ValidationError",
                format!("Invalid {model}: {details}"),
                [
                    ("field", serde_json::json!(model)),
                    ("details", serde_json::json!(details)),
                ],
            ))
        }
        other => ServerError::Nats(format!("jobs RPC query failed: {other}")),
    }
}
