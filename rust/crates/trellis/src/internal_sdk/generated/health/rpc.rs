//! Typed RPC descriptors for `trellis.health@v1`.
use serde::{Deserialize, Serialize};
use trellis_rs::generated::RpcDescriptor;
/// Empty request or response payload used by zero-argument RPCs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Empty {}
/// Descriptor for `Health.Inspect`.
pub struct HealthInspectRpc;
impl RpcDescriptor for HealthInspectRpc {
    type Input = super::types::HealthInspectRequest;
    type Output = super::types::HealthInspectResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::HEALTH_INSPECT_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::HEALTH_INSPECT_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Health.Inspect";
    const SUBJECT: &'static str = "rpc.v1.Health.Inspect";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.health::read"];
    const ERRORS: &'static [&'static str] =
        &["NotFoundError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `Health.Inspect`.
#[derive(Debug, Clone, PartialEq)]
pub enum HealthInspectError {
    /// `NotFoundError` error payload.
    NotFoundError(super::types::NotFoundErrorData),
    /// `UnexpectedError` error payload.
    UnexpectedError(trellis_rs::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(trellis_rs::generated::DeclaredErrorPayload),
}
impl trellis_rs::generated::DeclaredError for HealthInspectError {
    fn decode(
        payload: &trellis_rs::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("NotFoundError") => payload
                .decode_declared::<super::types::NotFoundErrorData>("NotFoundError")
                .map(|value| value.map(Self::NotFoundError)),
            Some("UnexpectedError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
/// Descriptor for `Health.Metrics`.
pub struct HealthMetricsRpc;
impl RpcDescriptor for HealthMetricsRpc {
    type Input = super::types::HealthMetricsRequest;
    type Output = super::types::HealthMetricsResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::HEALTH_METRICS_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::HEALTH_METRICS_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Health.Metrics";
    const SUBJECT: &'static str = "rpc.v1.Health.Metrics";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.health::read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `Health.Metrics`.
#[derive(Debug, Clone, PartialEq)]
pub enum HealthMetricsError {
    /// `UnexpectedError` error payload.
    UnexpectedError(trellis_rs::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(trellis_rs::generated::DeclaredErrorPayload),
}
impl trellis_rs::generated::DeclaredError for HealthMetricsError {
    fn decode(
        payload: &trellis_rs::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("UnexpectedError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
/// Descriptor for `Health.Query`.
pub struct HealthQueryRpc;
impl RpcDescriptor for HealthQueryRpc {
    type Input = super::types::HealthQueryRequest;
    type Output = super::types::HealthQueryResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::HEALTH_QUERY_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::HEALTH_QUERY_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Health.Query";
    const SUBJECT: &'static str = "rpc.v1.Health.Query";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.health::read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `Health.Query`.
#[derive(Debug, Clone, PartialEq)]
pub enum HealthQueryError {
    /// `UnexpectedError` error payload.
    UnexpectedError(trellis_rs::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(trellis_rs::generated::DeclaredErrorPayload),
}
impl trellis_rs::generated::DeclaredError for HealthQueryError {
    fn decode(
        payload: &trellis_rs::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("UnexpectedError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
}
