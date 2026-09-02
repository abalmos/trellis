//! Typed RPC descriptors for `trellis.jobs@v1`.
use crate::generated::RpcDescriptor;
use serde::{Deserialize, Serialize};
/// Empty request or response payload used by zero-argument RPCs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Empty {}
/// Descriptor for `Jobs.Cancel`.
pub struct JobsCancelRpc;
impl RpcDescriptor for JobsCancelRpc {
    type Input = super::types::JobsCancelRequest;
    type Output = super::types::JobsCancelResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_CANCEL_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_CANCEL_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Jobs.Cancel";
    const SUBJECT: &'static str = "rpc.v1.Jobs.Cancel";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.jobs::mutate"];
    const ERRORS: &'static [&'static str] =
        &["NotFoundError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `Jobs.Cancel`.
#[derive(Debug, Clone, PartialEq)]
pub enum JobsCancelError {
    /// `NotFoundError` error payload.
    NotFoundError(super::types::NotFoundErrorData),
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for JobsCancelError {
    fn decode(
        payload: &crate::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("NotFoundError") => payload
                .decode_declared::<super::types::NotFoundErrorData>("NotFoundError")
                .map(|value| value.map(Self::NotFoundError)),
            Some("UnexpectedError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
/// Descriptor for `Jobs.DismissDLQ`.
pub struct JobsDismissDLQRpc;
impl RpcDescriptor for JobsDismissDLQRpc {
    type Input = super::types::JobsDismissDLQRequest;
    type Output = super::types::JobsDismissDLQResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_DISMISS_DLQ_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_DISMISS_DLQ_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Jobs.DismissDLQ";
    const SUBJECT: &'static str = "rpc.v1.Jobs.DismissDLQ";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.jobs::mutate"];
    const ERRORS: &'static [&'static str] =
        &["NotFoundError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `Jobs.DismissDLQ`.
#[derive(Debug, Clone, PartialEq)]
pub enum JobsDismissDLQError {
    /// `NotFoundError` error payload.
    NotFoundError(super::types::NotFoundErrorData),
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for JobsDismissDLQError {
    fn decode(
        payload: &crate::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("NotFoundError") => payload
                .decode_declared::<super::types::NotFoundErrorData>("NotFoundError")
                .map(|value| value.map(Self::NotFoundError)),
            Some("UnexpectedError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
/// Descriptor for `Jobs.GetKey`.
pub struct JobsGetKeyRpc;
impl RpcDescriptor for JobsGetKeyRpc {
    type Input = super::types::JobsGetKeyRequest;
    type Output = super::types::JobsGetKeyResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_GET_KEY_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_GET_KEY_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Jobs.GetKey";
    const SUBJECT: &'static str = "rpc.v1.Jobs.GetKey";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.jobs::read"];
    const ERRORS: &'static [&'static str] =
        &["NotFoundError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `Jobs.GetKey`.
#[derive(Debug, Clone, PartialEq)]
pub enum JobsGetKeyError {
    /// `NotFoundError` error payload.
    NotFoundError(super::types::NotFoundErrorData),
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for JobsGetKeyError {
    fn decode(
        payload: &crate::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("NotFoundError") => payload
                .decode_declared::<super::types::NotFoundErrorData>("NotFoundError")
                .map(|value| value.map(Self::NotFoundError)),
            Some("UnexpectedError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
/// Descriptor for `Jobs.Inspect`.
pub struct JobsInspectRpc;
impl RpcDescriptor for JobsInspectRpc {
    type Input = super::types::JobsInspectRequest;
    type Output = super::types::JobsInspectResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_INSPECT_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_INSPECT_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Jobs.Inspect";
    const SUBJECT: &'static str = "rpc.v1.Jobs.Inspect";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.jobs::read"];
    const ERRORS: &'static [&'static str] =
        &["NotFoundError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `Jobs.Inspect`.
#[derive(Debug, Clone, PartialEq)]
pub enum JobsInspectError {
    /// `NotFoundError` error payload.
    NotFoundError(super::types::NotFoundErrorData),
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for JobsInspectError {
    fn decode(
        payload: &crate::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("NotFoundError") => payload
                .decode_declared::<super::types::NotFoundErrorData>("NotFoundError")
                .map(|value| value.map(Self::NotFoundError)),
            Some("UnexpectedError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
/// Descriptor for `Jobs.ListDLQ`.
pub struct JobsListDLQRpc;
impl RpcDescriptor for JobsListDLQRpc {
    type Input = super::types::JobsListDLQRequest;
    type Output = super::types::JobsListDLQResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_LIST_DLQ_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_LIST_DLQ_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Jobs.ListDLQ";
    const SUBJECT: &'static str = "rpc.v1.Jobs.ListDLQ";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.jobs::read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `Jobs.ListDLQ`.
#[derive(Debug, Clone, PartialEq)]
pub enum JobsListDLQError {
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for JobsListDLQError {
    fn decode(
        payload: &crate::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("UnexpectedError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
/// Descriptor for `Jobs.ListServices`.
pub struct JobsListServicesRpc;
impl RpcDescriptor for JobsListServicesRpc {
    type Input = super::types::JobsListServicesRequest;
    type Output = super::types::JobsListServicesResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_LIST_SERVICES_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_LIST_SERVICES_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Jobs.ListServices";
    const SUBJECT: &'static str = "rpc.v1.Jobs.ListServices";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.jobs::read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `Jobs.ListServices`.
#[derive(Debug, Clone, PartialEq)]
pub enum JobsListServicesError {
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for JobsListServicesError {
    fn decode(
        payload: &crate::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("UnexpectedError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
/// Descriptor for `Jobs.Metrics`.
pub struct JobsMetricsRpc;
impl RpcDescriptor for JobsMetricsRpc {
    type Input = super::types::JobsMetricsRequest;
    type Output = super::types::JobsMetricsResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_METRICS_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_METRICS_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Jobs.Metrics";
    const SUBJECT: &'static str = "rpc.v1.Jobs.Metrics";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.jobs::read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `Jobs.Metrics`.
#[derive(Debug, Clone, PartialEq)]
pub enum JobsMetricsError {
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for JobsMetricsError {
    fn decode(
        payload: &crate::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("UnexpectedError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
/// Descriptor for `Jobs.Query`.
pub struct JobsQueryRpc;
impl RpcDescriptor for JobsQueryRpc {
    type Input = super::types::JobsQueryRequest;
    type Output = super::types::JobsQueryResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_QUERY_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_QUERY_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Jobs.Query";
    const SUBJECT: &'static str = "rpc.v1.Jobs.Query";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.jobs::read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `Jobs.Query`.
#[derive(Debug, Clone, PartialEq)]
pub enum JobsQueryError {
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for JobsQueryError {
    fn decode(
        payload: &crate::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("UnexpectedError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
/// Descriptor for `Jobs.ReplayDLQ`.
pub struct JobsReplayDLQRpc;
impl RpcDescriptor for JobsReplayDLQRpc {
    type Input = super::types::JobsReplayDLQRequest;
    type Output = super::types::JobsReplayDLQResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_REPLAY_DLQ_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_REPLAY_DLQ_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Jobs.ReplayDLQ";
    const SUBJECT: &'static str = "rpc.v1.Jobs.ReplayDLQ";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.jobs::mutate"];
    const ERRORS: &'static [&'static str] =
        &["NotFoundError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `Jobs.ReplayDLQ`.
#[derive(Debug, Clone, PartialEq)]
pub enum JobsReplayDLQError {
    /// `NotFoundError` error payload.
    NotFoundError(super::types::NotFoundErrorData),
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for JobsReplayDLQError {
    fn decode(
        payload: &crate::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("NotFoundError") => payload
                .decode_declared::<super::types::NotFoundErrorData>("NotFoundError")
                .map(|value| value.map(Self::NotFoundError)),
            Some("UnexpectedError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
/// Descriptor for `Jobs.Retry`.
pub struct JobsRetryRpc;
impl RpcDescriptor for JobsRetryRpc {
    type Input = super::types::JobsRetryRequest;
    type Output = super::types::JobsRetryResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_RETRY_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_RETRY_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Jobs.Retry";
    const SUBJECT: &'static str = "rpc.v1.Jobs.Retry";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.jobs::mutate"];
    const ERRORS: &'static [&'static str] =
        &["NotFoundError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `Jobs.Retry`.
#[derive(Debug, Clone, PartialEq)]
pub enum JobsRetryError {
    /// `NotFoundError` error payload.
    NotFoundError(super::types::NotFoundErrorData),
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for JobsRetryError {
    fn decode(
        payload: &crate::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("NotFoundError") => payload
                .decode_declared::<super::types::NotFoundErrorData>("NotFoundError")
                .map(|value| value.map(Self::NotFoundError)),
            Some("UnexpectedError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<crate::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
