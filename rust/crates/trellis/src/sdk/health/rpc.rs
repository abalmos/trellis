//! Typed RPC descriptors for `trellis.health@v1`.
use crate::client::RpcDescriptor;
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
