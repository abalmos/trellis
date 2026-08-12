//! Typed RPC descriptors for `trellis.eventlog@v1`.
use crate::generated::RpcDescriptor;
use serde::{Deserialize, Serialize};
/// Empty request or response payload used by zero-argument RPCs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Empty {}
/// Descriptor for `EventLog.Consumers.Inspect`.
pub struct EventLogConsumersInspectRpc;
impl RpcDescriptor for EventLogConsumersInspectRpc {
    type Input = super::types::EventLogConsumersInspectRequest;
    type Output = Empty;
    const INPUT_SCHEMA_JSON: &'static str =
        super::schemas::EVENT_LOG_CONSUMERS_INSPECT_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str =
        super::schemas::EVENT_LOG_CONSUMERS_INSPECT_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "EventLog.Consumers.Inspect";
    const SUBJECT: &'static str = "rpc.v1.EventLog.Consumers.Inspect";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.eventlog::events.read"];
    const ERRORS: &'static [&'static str] =
        &["NotFoundError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `EventLog.Consumers.Inspect`.
#[derive(Debug, Clone, PartialEq)]
pub enum EventLogConsumersInspectError {
    /// `NotFoundError` error payload.
    NotFoundError(super::types::NotFoundErrorData),
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for EventLogConsumersInspectError {
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
/// Descriptor for `EventLog.Consumers.Query`.
pub struct EventLogConsumersQueryRpc;
impl RpcDescriptor for EventLogConsumersQueryRpc {
    type Input = super::types::EventLogConsumersQueryRequest;
    type Output = super::types::EventLogConsumersQueryResponse;
    const INPUT_SCHEMA_JSON: &'static str =
        super::schemas::EVENT_LOG_CONSUMERS_QUERY_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str =
        super::schemas::EVENT_LOG_CONSUMERS_QUERY_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "EventLog.Consumers.Query";
    const SUBJECT: &'static str = "rpc.v1.EventLog.Consumers.Query";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.eventlog::events.read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `EventLog.Consumers.Query`.
#[derive(Debug, Clone, PartialEq)]
pub enum EventLogConsumersQueryError {
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for EventLogConsumersQueryError {
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
/// Descriptor for `EventLog.Inspect`.
pub struct EventLogInspectRpc;
impl RpcDescriptor for EventLogInspectRpc {
    type Input = super::types::EventLogInspectRequest;
    type Output = Empty;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::EVENT_LOG_INSPECT_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::EVENT_LOG_INSPECT_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "EventLog.Inspect";
    const SUBJECT: &'static str = "rpc.v1.EventLog.Inspect";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.eventlog::events.read"];
    const ERRORS: &'static [&'static str] =
        &["NotFoundError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `EventLog.Inspect`.
#[derive(Debug, Clone, PartialEq)]
pub enum EventLogInspectError {
    /// `NotFoundError` error payload.
    NotFoundError(super::types::NotFoundErrorData),
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for EventLogInspectError {
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
/// Descriptor for `EventLog.Metrics`.
pub struct EventLogMetricsRpc;
impl RpcDescriptor for EventLogMetricsRpc {
    type Input = super::types::EventLogMetricsRequest;
    type Output = super::types::EventLogMetricsResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::EVENT_LOG_METRICS_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::EVENT_LOG_METRICS_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "EventLog.Metrics";
    const SUBJECT: &'static str = "rpc.v1.EventLog.Metrics";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.eventlog::events.read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `EventLog.Metrics`.
#[derive(Debug, Clone, PartialEq)]
pub enum EventLogMetricsError {
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for EventLogMetricsError {
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
/// Descriptor for `EventLog.Query`.
pub struct EventLogQueryRpc;
impl RpcDescriptor for EventLogQueryRpc {
    type Input = super::types::EventLogQueryRequest;
    type Output = super::types::EventLogQueryResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::EVENT_LOG_QUERY_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::EVENT_LOG_QUERY_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "EventLog.Query";
    const SUBJECT: &'static str = "rpc.v1.EventLog.Query";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.eventlog::events.read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `EventLog.Query`.
#[derive(Debug, Clone, PartialEq)]
pub enum EventLogQueryError {
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for EventLogQueryError {
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
