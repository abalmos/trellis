//! Typed RPC descriptors for `trellis.health@v1`.
use crate::generated::RpcDescriptor;
use serde::{Deserialize, Serialize};
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
        &["UnexpectedError", "ValidationError", "NotFoundError"];
}
/// Errors declared by `Health.Inspect`.
#[derive(Debug, Clone, PartialEq)]
pub enum HealthInspectError {
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
    /// `NotFoundError` error payload.
    NotFoundError(super::types::NotFoundErrorData),
}
impl crate::generated::DeclaredError for HealthInspectError {
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
            Some("NotFoundError") => payload
                .decode_declared::<super::types::NotFoundErrorData>("NotFoundError")
                .map(|value| value.map(Self::NotFoundError)),
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
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for HealthMetricsError {
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
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for HealthQueryError {
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
