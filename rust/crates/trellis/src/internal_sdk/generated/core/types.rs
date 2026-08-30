//! Shared request and response types for `trellis.core@v1`.
use serde::{Deserialize, Serialize};
/// Generated schema type `TrellisSurfaceStatusRequestAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisSurfaceStatusRequestAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
}
impl TrellisSurfaceStatusRequestAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Observe => "observe",
        }
    }
}
impl AsRef<str> for TrellisSurfaceStatusRequestAction {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisSurfaceStatusRequestAction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisSurfaceStatusRequestAction {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisSurfaceStatusRequestAction> for &str {
    fn eq(&self, other: &TrellisSurfaceStatusRequestAction) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisSurfaceStatusRequestKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisSurfaceStatusRequestKind {
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
}
impl TrellisSurfaceStatusRequestKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
        }
    }
}
impl AsRef<str> for TrellisSurfaceStatusRequestKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisSurfaceStatusRequestKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisSurfaceStatusRequestKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisSurfaceStatusRequestKind> for &str {
    fn eq(&self, other: &TrellisSurfaceStatusRequestKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisSurfaceStatusRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisSurfaceStatusRequest {
    /// The `action` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<TrellisSurfaceStatusRequestAction>,
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
    /// The `kind` wire field.
    pub kind: TrellisSurfaceStatusRequestKind,
    /// The `surface` wire field.
    pub surface: String,
}
/// Generated schema type `TrellisSurfaceStatusResponseStatusAvailableRuntime`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisSurfaceStatusResponseStatusAvailableRuntime {
    /// The `live` wire value.
    #[serde(rename = "live")]
    Live,
    /// The `no_live_implementer` wire value.
    #[serde(rename = "no_live_implementer")]
    NoLiveImplementer,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
}
impl TrellisSurfaceStatusResponseStatusAvailableRuntime {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::NoLiveImplementer => "no_live_implementer",
            Self::Disabled => "disabled",
        }
    }
}
impl AsRef<str> for TrellisSurfaceStatusResponseStatusAvailableRuntime {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisSurfaceStatusResponseStatusAvailableRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisSurfaceStatusResponseStatusAvailableRuntime {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisSurfaceStatusResponseStatusAvailableRuntime> for &str {
    fn eq(&self, other: &TrellisSurfaceStatusResponseStatusAvailableRuntime) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisSurfaceStatusResponseStatusUnavailableReason`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrellisSurfaceStatusResponseStatusUnavailableReason {
    /// The `authority_unavailable` wire value.
    #[serde(rename = "authority_unavailable")]
    AuthorityUnavailable,
}
impl TrellisSurfaceStatusResponseStatusUnavailableReason {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::AuthorityUnavailable => "authority_unavailable",
        }
    }
}
impl AsRef<str> for TrellisSurfaceStatusResponseStatusUnavailableReason {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for TrellisSurfaceStatusResponseStatusUnavailableReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for TrellisSurfaceStatusResponseStatusUnavailableReason {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<TrellisSurfaceStatusResponseStatusUnavailableReason> for &str {
    fn eq(&self, other: &TrellisSurfaceStatusResponseStatusUnavailableReason) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `TrellisSurfaceStatusResponseStatus`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state")]
pub enum TrellisSurfaceStatusResponseStatus {
    /// The `available` variant.
    #[serde(rename = "available")]
    Available {
        /// The `liveImplementer` wire field.
        #[serde(rename = "liveImplementer")]
        live_implementer: bool,
        /// The `runtime` wire field.
        runtime: TrellisSurfaceStatusResponseStatusAvailableRuntime,
    },
    /// The `unavailable` variant.
    #[serde(rename = "unavailable")]
    Unavailable {
        /// The `reason` wire field.
        reason: TrellisSurfaceStatusResponseStatusUnavailableReason,
    },
    /// The `unauthorized` variant.
    #[serde(rename = "unauthorized")]
    Unauthorized {
        /// The `missingCapabilities` wire field.
        #[serde(rename = "missingCapabilities")]
        missing_capabilities: Vec<String>,
    },
    /// The `unknown_contract` variant.
    #[serde(rename = "unknown_contract")]
    UnknownContract {
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
    },
    /// The `unknown_surface` variant.
    #[serde(rename = "unknown_surface")]
    UnknownSurface {
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `kind` wire field.
        kind: String,
        /// The `surface` wire field.
        surface: String,
    },
}
/// Generated schema type `TrellisSurfaceStatusResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrellisSurfaceStatusResponse {
    /// The `status` wire field.
    pub status: TrellisSurfaceStatusResponseStatus,
}
