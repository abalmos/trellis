//! Typed RPC descriptors for `trellis.state@v1`.
use serde::{Deserialize, Serialize};
use trellis_rs::generated::RpcDescriptor;
/// Empty request or response payload used by zero-argument RPCs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Empty {}
/// Descriptor for `State.Admin.Delete`.
pub struct StateAdminDeleteRpc;
impl RpcDescriptor for StateAdminDeleteRpc {
    type Input = super::types::StateAdminDeleteRequest;
    type Output = super::types::StateAdminDeleteResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_ADMIN_DELETE_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_ADMIN_DELETE_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "State.Admin.Delete";
    const SUBJECT: &'static str = "rpc.v1.State.Admin.Delete";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.state::mutate"];
    const ERRORS: &'static [&'static str] = &["AuthError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `State.Admin.Delete`.
#[derive(Debug, Clone, PartialEq)]
pub enum StateAdminDeleteError {
    /// `AuthError` error payload.
    AuthError(trellis_rs::generated::AuthErrorPayload),
    /// `UnexpectedError` error payload.
    UnexpectedError(trellis_rs::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(trellis_rs::generated::DeclaredErrorPayload),
}
impl trellis_rs::generated::DeclaredError for StateAdminDeleteError {
    fn decode(
        payload: &trellis_rs::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("AuthError") => payload
                .decode_declared::<trellis_rs::generated::AuthErrorPayload>("AuthError")
                .map(|value| value.map(Self::AuthError)),
            Some("UnexpectedError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
    fn auth_error_reason(&self) -> Option<&str> {
        match self {
            Self::AuthError(payload) => Some(payload.reason.as_str()),
            _ => None,
        }
    }
}
/// Descriptor for `State.Admin.Get`.
pub struct StateAdminGetRpc;
impl RpcDescriptor for StateAdminGetRpc {
    type Input = super::types::StateAdminGetRequest;
    type Output = super::types::StateAdminGetResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_ADMIN_GET_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_ADMIN_GET_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "State.Admin.Get";
    const SUBJECT: &'static str = "rpc.v1.State.Admin.Get";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.state::read"];
    const ERRORS: &'static [&'static str] = &["AuthError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `State.Admin.Get`.
#[derive(Debug, Clone, PartialEq)]
pub enum StateAdminGetError {
    /// `AuthError` error payload.
    AuthError(trellis_rs::generated::AuthErrorPayload),
    /// `UnexpectedError` error payload.
    UnexpectedError(trellis_rs::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(trellis_rs::generated::DeclaredErrorPayload),
}
impl trellis_rs::generated::DeclaredError for StateAdminGetError {
    fn decode(
        payload: &trellis_rs::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("AuthError") => payload
                .decode_declared::<trellis_rs::generated::AuthErrorPayload>("AuthError")
                .map(|value| value.map(Self::AuthError)),
            Some("UnexpectedError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
    fn auth_error_reason(&self) -> Option<&str> {
        match self {
            Self::AuthError(payload) => Some(payload.reason.as_str()),
            _ => None,
        }
    }
}
/// Descriptor for `State.Admin.List`.
pub struct StateAdminListRpc;
impl RpcDescriptor for StateAdminListRpc {
    type Input = super::types::StateAdminListRequest;
    type Output = super::types::StateAdminListResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_ADMIN_LIST_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_ADMIN_LIST_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "State.Admin.List";
    const SUBJECT: &'static str = "rpc.v1.State.Admin.List";
    const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.state::read"];
    const ERRORS: &'static [&'static str] = &["AuthError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `State.Admin.List`.
#[derive(Debug, Clone, PartialEq)]
pub enum StateAdminListError {
    /// `AuthError` error payload.
    AuthError(trellis_rs::generated::AuthErrorPayload),
    /// `UnexpectedError` error payload.
    UnexpectedError(trellis_rs::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(trellis_rs::generated::DeclaredErrorPayload),
}
impl trellis_rs::generated::DeclaredError for StateAdminListError {
    fn decode(
        payload: &trellis_rs::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("AuthError") => payload
                .decode_declared::<trellis_rs::generated::AuthErrorPayload>("AuthError")
                .map(|value| value.map(Self::AuthError)),
            Some("UnexpectedError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
    fn auth_error_reason(&self) -> Option<&str> {
        match self {
            Self::AuthError(payload) => Some(payload.reason.as_str()),
            _ => None,
        }
    }
}
/// Descriptor for `State.Delete`.
pub struct StateDeleteRpc;
impl RpcDescriptor for StateDeleteRpc {
    type Input = super::types::StateDeleteRequest;
    type Output = super::types::StateDeleteResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_DELETE_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_DELETE_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "State.Delete";
    const SUBJECT: &'static str = "rpc.v1.State.Delete";
    const CALLER_CAPABILITIES: &'static [&'static str] = &[];
    const ERRORS: &'static [&'static str] = &["AuthError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `State.Delete`.
#[derive(Debug, Clone, PartialEq)]
pub enum StateDeleteError {
    /// `AuthError` error payload.
    AuthError(trellis_rs::generated::AuthErrorPayload),
    /// `UnexpectedError` error payload.
    UnexpectedError(trellis_rs::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(trellis_rs::generated::DeclaredErrorPayload),
}
impl trellis_rs::generated::DeclaredError for StateDeleteError {
    fn decode(
        payload: &trellis_rs::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("AuthError") => payload
                .decode_declared::<trellis_rs::generated::AuthErrorPayload>("AuthError")
                .map(|value| value.map(Self::AuthError)),
            Some("UnexpectedError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
    fn auth_error_reason(&self) -> Option<&str> {
        match self {
            Self::AuthError(payload) => Some(payload.reason.as_str()),
            _ => None,
        }
    }
}
/// Descriptor for `State.Get`.
pub struct StateGetRpc;
impl RpcDescriptor for StateGetRpc {
    type Input = super::types::StateGetRequest;
    type Output = super::types::StateGetResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_GET_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_GET_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "State.Get";
    const SUBJECT: &'static str = "rpc.v1.State.Get";
    const CALLER_CAPABILITIES: &'static [&'static str] = &[];
    const ERRORS: &'static [&'static str] = &["AuthError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `State.Get`.
#[derive(Debug, Clone, PartialEq)]
pub enum StateGetError {
    /// `AuthError` error payload.
    AuthError(trellis_rs::generated::AuthErrorPayload),
    /// `UnexpectedError` error payload.
    UnexpectedError(trellis_rs::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(trellis_rs::generated::DeclaredErrorPayload),
}
impl trellis_rs::generated::DeclaredError for StateGetError {
    fn decode(
        payload: &trellis_rs::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("AuthError") => payload
                .decode_declared::<trellis_rs::generated::AuthErrorPayload>("AuthError")
                .map(|value| value.map(Self::AuthError)),
            Some("UnexpectedError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
    fn auth_error_reason(&self) -> Option<&str> {
        match self {
            Self::AuthError(payload) => Some(payload.reason.as_str()),
            _ => None,
        }
    }
}
/// Descriptor for `State.List`.
pub struct StateListRpc;
impl RpcDescriptor for StateListRpc {
    type Input = super::types::StateListRequest;
    type Output = super::types::StateListResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_LIST_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_LIST_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "State.List";
    const SUBJECT: &'static str = "rpc.v1.State.List";
    const CALLER_CAPABILITIES: &'static [&'static str] = &[];
    const ERRORS: &'static [&'static str] = &["AuthError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `State.List`.
#[derive(Debug, Clone, PartialEq)]
pub enum StateListError {
    /// `AuthError` error payload.
    AuthError(trellis_rs::generated::AuthErrorPayload),
    /// `UnexpectedError` error payload.
    UnexpectedError(trellis_rs::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(trellis_rs::generated::DeclaredErrorPayload),
}
impl trellis_rs::generated::DeclaredError for StateListError {
    fn decode(
        payload: &trellis_rs::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("AuthError") => payload
                .decode_declared::<trellis_rs::generated::AuthErrorPayload>("AuthError")
                .map(|value| value.map(Self::AuthError)),
            Some("UnexpectedError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
    fn auth_error_reason(&self) -> Option<&str> {
        match self {
            Self::AuthError(payload) => Some(payload.reason.as_str()),
            _ => None,
        }
    }
}
/// Descriptor for `State.Put`.
pub struct StatePutRpc;
impl RpcDescriptor for StatePutRpc {
    type Input = super::types::StatePutRequest;
    type Output = super::types::StatePutResponse;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_PUT_INPUT_SCHEMA_JSON;
    const OUTPUT_SCHEMA_JSON: &'static str = super::schemas::STATE_PUT_OUTPUT_SCHEMA_JSON;
    const KEY: &'static str = "State.Put";
    const SUBJECT: &'static str = "rpc.v1.State.Put";
    const CALLER_CAPABILITIES: &'static [&'static str] = &[];
    const ERRORS: &'static [&'static str] = &["AuthError", "UnexpectedError", "ValidationError"];
}
/// Errors declared by `State.Put`.
#[derive(Debug, Clone, PartialEq)]
pub enum StatePutError {
    /// `AuthError` error payload.
    AuthError(trellis_rs::generated::AuthErrorPayload),
    /// `UnexpectedError` error payload.
    UnexpectedError(trellis_rs::generated::DeclaredErrorPayload),
    /// `ValidationError` error payload.
    ValidationError(trellis_rs::generated::DeclaredErrorPayload),
}
impl trellis_rs::generated::DeclaredError for StatePutError {
    fn decode(
        payload: &trellis_rs::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("AuthError") => payload
                .decode_declared::<trellis_rs::generated::AuthErrorPayload>("AuthError")
                .map(|value| value.map(Self::AuthError)),
            Some("UnexpectedError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<trellis_rs::generated::DeclaredErrorPayload>("ValidationError")
                .map(|value| value.map(Self::ValidationError)),
            _ => Ok(None),
        }
    }
    fn auth_error_reason(&self) -> Option<&str> {
        match self {
            Self::AuthError(payload) => Some(payload.reason.as_str()),
            _ => None,
        }
    }
}
