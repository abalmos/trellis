//! Typed RPC descriptors for `trellis.eventlog@v1`.
use crate::client::RpcDescriptor;
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
        &["UnexpectedError", "ValidationError", "NotFoundError"];
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
        &["UnexpectedError", "ValidationError", "NotFoundError"];
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
