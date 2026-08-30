//! Typed RPC descriptors for `trellis.core@v1`.
use crate::generated::RpcDescriptor;
use serde::{Deserialize, Serialize};
/// Empty request or response payload used by zero-argument RPCs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Empty {}
/// Descriptor for `Trellis.Surface.Status`.
pub struct TrellisSurfaceStatusRpc;
impl RpcDescriptor for TrellisSurfaceStatusRpc {
    type Input = super::types::TrellisSurfaceStatusRequest;
    type Output = super::types::TrellisSurfaceStatusResponse;
    const INPUT_SCHEMA_JSON: &'static str =
        super::schemas::TRELLIS_SURFACE_STATUS_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str =
        super::schemas::TRELLIS_SURFACE_STATUS_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Trellis.Surface.Status";
    const SUBJECT: &'static str = "rpc.v1.Trellis.Surface.Status";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.core::authority.read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `Trellis.Surface.Status`.
#[derive(Debug, Clone, PartialEq)]
pub enum TrellisSurfaceStatusError {
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for TrellisSurfaceStatusError {
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
