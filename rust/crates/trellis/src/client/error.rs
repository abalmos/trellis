use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Structured payload returned by a remote RPC error response.
#[derive(Clone, Debug, PartialEq)]
pub struct RpcErrorPayload {
    raw: String,
    value: Option<Value>,
}

/// Structured payload returned by an undeclared remote Trellis error.
pub type RemoteErrorPayload = RpcErrorPayload;

impl RpcErrorPayload {
    /// Builds a payload from a raw JSON RPC error body.
    pub fn from_json_slice(raw: &[u8]) -> Result<Self, serde_json::Error> {
        let value = serde_json::from_slice::<Value>(raw)?;
        Ok(Self {
            raw: String::from_utf8_lossy(raw).into_owned(),
            value: Some(value),
        })
    }

    /// Builds a payload from a decoded JSON RPC error body.
    pub fn from_value(value: Value) -> Self {
        Self {
            raw: value.to_string(),
            value: Some(value),
        }
    }

    /// Builds a payload from an unstructured error message.
    pub fn from_message(message: impl Into<String>) -> Self {
        Self {
            raw: message.into(),
            value: None,
        }
    }

    /// Returns the original payload text.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Returns the decoded JSON payload when the RPC error body was structured.
    pub fn value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    /// Returns the remote error discriminator when present.
    pub fn error_type(&self) -> Option<&str> {
        self.value
            .as_ref()
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str)
    }

    /// Decode this payload as a declared RPC error when its discriminator matches.
    pub fn decode_declared<T>(&self, error_type: &str) -> Result<Option<T>, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let Some(value) = self.value.as_ref() else {
            return Ok(None);
        };
        if self.error_type() != Some(error_type) {
            return Ok(None);
        }
        serde_json::from_value(value.clone()).map(Some)
    }

    fn format_human(&self) -> String {
        if let Some(value) = &self.value {
            format_rpc_error_value(value, &self.raw)
        } else {
            self.raw.clone()
        }
    }

    /// Decode this payload as a `ValidationError` when the discriminator matches.
    pub fn decode_validation(&self) -> Result<Option<ValidationErrorPayload>, serde_json::Error> {
        self.decode_declared::<ValidationErrorPayload>("ValidationError")
    }

    /// Decode this payload as a `SchemaValidationError` when the discriminator matches.
    pub fn decode_schema_validation(
        &self,
    ) -> Result<Option<SchemaValidationErrorPayload>, serde_json::Error> {
        self.decode_declared::<SchemaValidationErrorPayload>("SchemaValidationError")
    }
}

/// One unannotated validation issue from a remote `ValidationError`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
}

/// One annotated validation issue from a remote `SchemaValidationError`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaValidationIssue {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_path: Option<String>,
    pub keyword: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Decoded `ValidationError` remote payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationErrorPayload {
    pub id: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    pub issues: Vec<ValidationIssue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default, rename = "traceId", skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

/// Decoded `SchemaValidationError` remote payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaValidationErrorPayload {
    pub id: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    pub issues: Vec<SchemaValidationIssue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default, rename = "traceId", skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

/// Standard fields available on a declared error without a data schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeclaredErrorPayload {
    /// Error instance identifier.
    pub id: String,
    /// Contract error discriminator.
    #[serde(rename = "type")]
    pub error_type: String,
    /// Human-facing error message.
    pub message: String,
    /// Optional structured context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Map<String, serde_json::Value>>,
    /// Optional distributed trace identifier.
    #[serde(default, rename = "traceId", skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

/// Standard fields returned for the built-in `AuthError` declaration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthErrorPayload {
    /// Error instance identifier.
    pub id: String,
    /// Contract error discriminator.
    #[serde(rename = "type")]
    pub error_type: String,
    /// Human-facing error message.
    pub message: String,
    /// Stable machine-readable authentication or authorization reason.
    pub reason: String,
    /// Optional structured context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Map<String, serde_json::Value>>,
    /// Optional distributed trace identifier.
    #[serde(default, rename = "traceId", skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

/// Decoder implemented by generated declared-error enums.
pub trait DeclaredError: Sized + std::fmt::Debug {
    /// Decode a matching declared error, or return `None` for an unknown discriminator.
    fn decode(payload: &RemoteErrorPayload) -> Result<Option<Self>, serde_json::Error>;

    /// Return the reason carried by a built-in `AuthError`, when declared.
    fn auth_error_reason(&self) -> Option<&str> {
        None
    }
}

/// Marker used by calls that declare no contract-specific errors.
#[derive(Debug)]
pub enum NoDeclaredError {}

impl std::fmt::Display for NoDeclaredError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

impl std::error::Error for NoDeclaredError {}

impl DeclaredError for NoDeclaredError {
    fn decode(_payload: &RemoteErrorPayload) -> Result<Option<Self>, serde_json::Error> {
        Ok(None)
    }
}

/// Standard validation failures detected locally or returned by a remote provider.
#[derive(Clone, Debug, PartialEq)]
pub enum ValidationFailure {
    /// Structural validation failure.
    Validation(ValidationErrorPayload),
    /// Annotated schema validation failure.
    Schema(SchemaValidationErrorPayload),
}

/// Authentication failure returned while making a connected call.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[error("{message}")]
pub struct AuthenticationError {
    message: String,
}

impl AuthenticationError {
    /// Build an authentication error from a runtime message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Remote protocol failure, including malformed contract payloads.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[error("{message}")]
pub struct ProtocolError {
    message: String,
}

impl ProtocolError {
    /// Build a protocol error from a runtime message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Connected transport failure.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[error("{message}")]
pub struct TransportError {
    message: String,
}

impl TransportError {
    /// Build a transport error from a runtime message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Errors returned by generated caller methods.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CallError<E>
where
    E: std::fmt::Debug,
{
    /// Contract-declared error.
    #[error("declared remote error: {0:?}")]
    Declared(Box<E>),
    /// Well-formed remote error not declared by this action.
    #[error("remote error: {}", .0.format_human())]
    Remote(RemoteErrorPayload),
    /// Standard validation failure.
    #[error("validation failed")]
    Validation(Box<ValidationFailure>),
    /// Request timeout.
    #[error("request timeout")]
    Timeout,
    /// Authentication failure.
    #[error(transparent)]
    Authentication(AuthenticationError),
    /// Invalid protocol frame or contract payload.
    #[error(transparent)]
    Protocol(ProtocolError),
    /// NATS or other connected transport failure.
    #[error(transparent)]
    Transport(TransportError),
}

impl<E> CallError<E>
where
    E: DeclaredError,
{
    pub(crate) fn from_client(error: TrellisClientError) -> Self {
        match error {
            TrellisClientError::RpcError(payload) => match E::decode(&payload) {
                Ok(Some(error)) => Self::Declared(Box::new(error)),
                Ok(None) => match payload.decode_schema_validation() {
                    Ok(Some(error)) => Self::Validation(Box::new(ValidationFailure::Schema(error))),
                    Ok(None) => match payload.decode_validation() {
                        Ok(Some(error)) => {
                            Self::Validation(Box::new(ValidationFailure::Validation(error)))
                        }
                        Ok(None) if payload.error_type().is_some() => Self::Remote(payload),
                        Ok(None) => Self::Protocol(ProtocolError::new(
                            "remote error payload has no string type discriminator",
                        )),
                        Err(error) => Self::Protocol(ProtocolError::new(error.to_string())),
                    },
                    Err(error) => Self::Protocol(ProtocolError::new(error.to_string())),
                },
                Err(error) => Self::Protocol(ProtocolError::new(error.to_string())),
            },
            TrellisClientError::Timeout => Self::Timeout,
            TrellisClientError::Json(error) => {
                Self::Protocol(ProtocolError::new(error.to_string()))
            }
            error => Self::Transport(TransportError::new(error.to_string())),
        }
    }
}

fn format_json_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(values) => values
            .iter()
            .map(format_json_value)
            .collect::<Vec<_>>()
            .join(","),
        Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

fn format_issue(issue: &Value) -> Option<String> {
    let obj = issue.as_object()?;
    let message = obj.get("message")?.as_str()?.trim();
    let path = obj
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim_start_matches('/');

    if path.is_empty() || message.contains(path) {
        Some(message.to_string())
    } else {
        Some(format!("{path}: {message}"))
    }
}

fn format_context(value: &Value) -> Option<String> {
    let obj = value.as_object()?;
    let fields = obj
        .iter()
        .filter(|(_, value)| !value.is_null())
        .map(|(key, value)| format!("{key}={}", format_json_value(value)))
        .collect::<Vec<_>>();
    if fields.is_empty() {
        None
    } else {
        Some(fields.join(", "))
    }
}

fn format_rpc_error_value(value: &Value, raw: &str) -> String {
    let issues = value
        .get("issues")
        .and_then(Value::as_array)
        .map(|issues| issues.iter().filter_map(format_issue).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut message = if issues.is_empty() {
        value
            .get("message")
            .and_then(Value::as_str)
            .map(|message| {
                message
                    .strip_prefix("Validation failed. ")
                    .unwrap_or(message)
                    .to_string()
            })
            .unwrap_or_else(|| raw.to_string())
    } else {
        issues.join("; ")
    };

    if let Some(context) = value.get("context").and_then(format_context) {
        message.push_str(&format!(" ({context})"));
    }

    message
}

#[cfg(test)]
fn format_rpc_error_payload(raw: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return raw.to_string();
    };

    format_rpc_error_value(&value, raw)
}

/// Errors returned by the Trellis client runtime.
#[derive(thiserror::Error, Debug)]
pub enum TrellisClientError {
    #[error("invalid base64url: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("invalid ed25519 seed length: {0} (expected 32)")]
    InvalidSeedLen(usize),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("nats error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("nats connect error: {0}")]
    NatsConnect(String),

    #[error("nats request error: {0}")]
    NatsRequest(String),

    #[error("Trellis HTTP request failed with status {status}: {code}")]
    BootstrapHttp { status: u16, code: String },

    #[error("service bootstrap error: {0}")]
    Bootstrap(String),

    #[error("request timeout")]
    Timeout,

    #[error("invalid json: {0}")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Subject(#[from] super::subject::SubjectError),

    #[error("rpc returned error: {}", .0.format_human())]
    RpcError(RpcErrorPayload),

    #[error("operation protocol error: {0}")]
    OperationProtocol(String),

    #[error("transfer protocol error: {0}")]
    TransferProtocol(String),

    #[error("transfer cancelled")]
    TransferCancelled,

    #[error("event subscription protocol error: {0}")]
    EventSubscriptionProtocol(String),

    #[error("feed protocol error: {0}")]
    FeedProtocol(String),
}

#[cfg(test)]
mod tests {
    use super::{format_rpc_error_payload, RpcErrorPayload};

    #[test]
    fn formats_validation_error_payload_human_readably() {
        let raw = r#"{"context":{"deploymentId":"demo"},"issues":[{"message":"service deployment not found","path":"/deploymentId"}],"message":"Validation failed. /deploymentId: service deployment not found.","type":"ValidationError"}"#;
        assert_eq!(
            format_rpc_error_payload(raw),
            "deploymentId: service deployment not found (deploymentId=demo)"
        );
    }

    #[test]
    fn leaves_non_json_payloads_unchanged() {
        assert_eq!(format_rpc_error_payload("plain error"), "plain error");
    }

    #[test]
    fn rpc_error_payload_preserves_structured_error_type() {
        let raw = r#"{"type":"UnexpectedError","message":"rust handler error marker"}"#;
        let payload = RpcErrorPayload::from_json_slice(raw.as_bytes()).unwrap();

        assert_eq!(payload.raw(), raw);
        assert_eq!(payload.error_type(), Some("UnexpectedError"));
    }

    #[test]
    fn rpc_error_payload_decodes_matching_declared_error() {
        #[derive(Debug, serde::Deserialize, PartialEq, Eq)]
        struct NotFoundError {
            resource: String,
        }

        let raw = r#"{"id":"err-1","type":"NotFoundError","message":"Workspace not found","resource":"Workspace"}"#;
        let payload = RpcErrorPayload::from_json_slice(raw.as_bytes()).unwrap();

        assert_eq!(
            payload
                .decode_declared::<NotFoundError>("NotFoundError")
                .unwrap(),
            Some(NotFoundError {
                resource: "Workspace".to_string()
            })
        );
        assert_eq!(
            payload
                .decode_declared::<NotFoundError>("OtherError")
                .unwrap(),
            None
        );
    }

    #[test]
    fn rpc_error_display_uses_formatted_payload() {
        let error =
            super::TrellisClientError::RpcError(RpcErrorPayload::from_value(serde_json::json!({
                "context": { "deploymentId": "demo" },
                "issues": [{ "message": "service deployment not found", "path": "/deploymentId" }],
                "message": "Validation failed. /deploymentId: service deployment not found.",
                "type": "ValidationError"
            })));

        assert_eq!(
            error.to_string(),
            "rpc returned error: deploymentId: service deployment not found (deploymentId=demo)"
        );
    }

    #[test]
    fn decode_schema_validation_returns_typed_payload() {
        let raw = r#"{"id":"err-1","type":"SchemaValidationError","message":"Schema validation failed.","issues":[{"path":"/items","keyword":"minItems","code":"test.items.required","message":"Add at least one item.","label":"Items"}],"context":{"requestId":"r1"}}"#;
        let payload = RpcErrorPayload::from_json_slice(raw.as_bytes()).unwrap();

        let decoded = payload.decode_schema_validation().unwrap();
        assert!(decoded.is_some(), "expected Some payload");
        let sv = decoded.unwrap();
        assert_eq!(sv.error_type, "SchemaValidationError");
        assert_eq!(sv.issues.len(), 1);
        assert_eq!(sv.issues[0].code, "test.items.required");
        assert_eq!(sv.issues[0].keyword, "minItems");
        assert_eq!(sv.issues[0].label.as_deref(), Some("Items"));
    }

    #[test]
    fn decode_validation_returns_typed_payload() {
        let raw = r#"{"id":"err-2","type":"ValidationError","message":"Validation failed.","issues":[{"path":"/name","message":"minLength failed"}]}"#;
        let payload = RpcErrorPayload::from_json_slice(raw.as_bytes()).unwrap();

        let decoded = payload.decode_validation().unwrap();
        assert!(decoded.is_some(), "expected Some payload");
        let v = decoded.unwrap();
        assert_eq!(v.error_type, "ValidationError");
        assert_eq!(v.issues.len(), 1);
        assert_eq!(v.issues[0].path, "/name");
    }

    #[test]
    fn decode_validation_returns_none_for_wrong_type() {
        let raw = r#"{"id":"err-3","type":"UnexpectedError","message":"Something broke"}"#;
        let payload = RpcErrorPayload::from_json_slice(raw.as_bytes()).unwrap();

        assert!(payload.decode_validation().unwrap().is_none());
        assert!(payload.decode_schema_validation().unwrap().is_none());
    }
}
