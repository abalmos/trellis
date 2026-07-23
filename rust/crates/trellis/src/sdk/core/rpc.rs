//! Typed RPC descriptors for `trellis.core@v1`.
use crate::generated::RpcDescriptor;
use serde::{Deserialize, Serialize};
/// Empty request or response payload used by zero-argument RPCs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Empty {}
/// Descriptor for `Trellis.Catalog`.
pub struct TrellisCatalogRpc;
impl RpcDescriptor for TrellisCatalogRpc {
    type Input = Empty;
    type Output = super::types::TrellisCatalogResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::TRELLIS_CATALOG_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::TRELLIS_CATALOG_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Trellis.Catalog";
    const SUBJECT: &'static str = "rpc.v1.Trellis.Catalog";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.core::catalog.read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `Trellis.Catalog`.
#[derive(Debug, Clone, PartialEq)]
pub enum TrellisCatalogError {
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for TrellisCatalogError {
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
/// Descriptor for `Trellis.Contract.Get`.
pub struct TrellisContractGetRpc;
impl RpcDescriptor for TrellisContractGetRpc {
    type Input = super::types::TrellisContractGetRequest;
    type Output = super::types::TrellisContractGetResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::TRELLIS_CONTRACT_GET_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str =
        super::schemas::TRELLIS_CONTRACT_GET_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "Trellis.Contract.Get";
    const SUBJECT: &'static str = "rpc.v1.Trellis.Contract.Get";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.core::contract.read"];
    const ERRORS: &'static [&'static str] = &["UnexpectedError", "ValidationError"];
}
/// Errors declared by `Trellis.Contract.Get`.
#[derive(Debug, Clone, PartialEq)]
pub enum TrellisContractGetError {
    /// `UnexpectedError` error payload.
    UnexpectedError(crate::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(crate::generated::DeclaredErrorPayload),
}
impl crate::generated::DeclaredError for TrellisContractGetError {
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
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.core::catalog.read"];
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
