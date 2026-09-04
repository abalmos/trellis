//! Typed operation descriptors for `trellis.auth@v1`.
use trellis_rs::generated::{NoOperationUpdates, OperationDescriptor};
use trellis_rs::service::OperationFailureLike;
/// Descriptor for `Auth.DeviceUserAuthorities.Resolve`.
pub struct AuthDeviceUserAuthoritiesResolveOperation;
impl OperationDescriptor for AuthDeviceUserAuthoritiesResolveOperation {
    type Input = super::types::AuthDeviceUserAuthoritiesResolveInput;
    type Progress = super::types::AuthDeviceUserAuthoritiesResolveProgress;
    type Output = super::types::AuthDeviceUserAuthoritiesResolveOutput;
    type Update = serde_json::Value;
    type UpdateEvidence = NoOperationUpdates;
    type Error = AuthDeviceUserAuthoritiesResolveOperationError;
    const INPUT_SCHEMA_JSON: &'static str =
        super::schemas::AUTH_DEVICE_USER_AUTHORITIES_RESOLVE_INPUT_SCHEMA_JSON;
    const PROGRESS_SCHEMA_JSON: Option<&'static str> =
        Some(super::schemas::AUTH_DEVICE_USER_AUTHORITIES_RESOLVE_PROGRESS_SCHEMA_JSON);
    const OUTPUT_SCHEMA_JSON: &'static str =
        super::schemas::AUTH_DEVICE_USER_AUTHORITIES_RESOLVE_OUTPUT_SCHEMA_JSON;
    const UPDATE_SCHEMA_JSON: Option<&'static str> = None;
    const SIGNAL_INPUT_SCHEMAS_JSON: &'static str =
        super::schemas::AUTH_DEVICE_USER_AUTHORITIES_RESOLVE_SIGNAL_INPUT_SCHEMAS_JSON;
    const ERRORS: &'static [&'static str] = &["AuthError", "UnexpectedError", "ValidationError"];
    const KEY: &'static str = "Auth.DeviceUserAuthorities.Resolve";
    const SUBJECT: &'static str = "operations.v1.Auth.DeviceUserAuthorities.Resolve";
    const CALLER_CAPABILITIES: &'static [&'static str] = &[];
    const OBSERVE_CAPABILITIES: &'static [&'static str] = &[];
    const CANCEL_CAPABILITIES: &'static [&'static str] = &[];
    const CONTROL_CAPABILITIES: &'static [&'static str] = &[];
    const CANCELABLE: bool = false;
}
/// Errors declared by `Auth.DeviceUserAuthorities.Resolve`.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthDeviceUserAuthoritiesResolveOperationError {
    /// `AuthError` failure.
    AuthError(trellis_rs::generated::AuthErrorPayload),
    /// `UnexpectedError` failure.
    UnexpectedError(super::types::AuthErrorDetails),
    /// `ValidationError` failure.
    ValidationError(super::types::AuthErrorDetails),
}
impl trellis_rs::generated::DeclaredError for AuthDeviceUserAuthoritiesResolveOperationError {
    fn decode(
        payload: &trellis_rs::generated::RemoteErrorPayload,
    ) -> Result<Option<Self>, serde_json::Error> {
        match payload.error_type() {
            Some("AuthError") => payload
                .decode_declared::<trellis_rs::generated::AuthErrorPayload>("AuthError")
                .map(|value| value.map(Self::AuthError)),
            Some("UnexpectedError") => payload
                .decode_declared::<super::types::AuthErrorDetails>("UnexpectedError")
                .map(|value| value.map(Self::UnexpectedError)),
            Some("ValidationError") => payload
                .decode_declared::<super::types::AuthErrorDetails>("ValidationError")
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
impl OperationFailureLike for AuthDeviceUserAuthoritiesResolveOperationError {
    fn error_type(&self) -> &str {
        match self {
            Self::AuthError(..) => "AuthError",
            Self::UnexpectedError(..) => "UnexpectedError",
            Self::ValidationError(..) => "ValidationError",
        }
    }
    fn message(&self) -> String {
        self.fields()
            .remove("message")
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| self.error_type().to_string())
    }
    fn fields(&self) -> serde_json::Map<String, serde_json::Value> {
        let value = match self {
            Self::AuthError(payload) => serde_json::to_value(payload),
            Self::UnexpectedError(payload) => serde_json::to_value(payload),
            Self::ValidationError(payload) => serde_json::to_value(payload),
        };
        value
            .ok()
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default()
    }
}
