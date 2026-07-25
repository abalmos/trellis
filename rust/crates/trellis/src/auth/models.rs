use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::AuthenticatedUser;
use crate::client::{AuthorizationContextBundle, SessionAuth};

/// Persisted admin session details for the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[doc = concat!("Public Trellis data type `", stringify!(AdminSessionState), "`.")]
pub struct AdminSessionState {
    /// Base URL for the Trellis deployment.
    #[doc = concat!("The `", stringify!(trellis_url), "` value.")]
    pub trellis_url: String,
    /// Comma-separated runtime server list returned by Trellis.
    #[serde(rename = "nats_servers", alias = "servers")]
    #[doc = concat!("The `", stringify!(servers), "` value.")]
    pub servers: String,
    /// Session-key seed used to sign subsequent Trellis requests.
    #[doc = concat!("The `", stringify!(session_seed), "` value.")]
    pub session_seed: String,
    /// Public session key derived from `session_seed`.
    #[doc = concat!("The `", stringify!(session_key), "` value.")]
    pub session_key: String,
    /// Exact administration participant artifact digest.
    #[doc = concat!("The `", stringify!(participant_digest), "` value.")]
    pub participant_digest: String,
    /// Durable Trellis session identifier.
    #[doc = concat!("The `", stringify!(session_id), "` value.")]
    pub session_id: String,
    /// NATS reply-inbox prefix authorized for this session.
    #[doc = concat!("The `", stringify!(inbox_prefix), "` value.")]
    pub inbox_prefix: String,
    /// Deny-all Auth-account JWT used to enter NATS Auth Callout.
    #[doc = concat!("The `", stringify!(bootstrap_jwt), "` value.")]
    pub bootstrap_jwt: String,
    /// Signed authorization context and minimal trust metadata for reconnect and refresh.
    pub authorization_context: AuthorizationContextBundle,
    /// Session expiry in Unix milliseconds, when bounded.
    #[doc = concat!("The `", stringify!(expires_at), "` value.")]
    pub expires_at: Option<i64>,
}

/// A successfully bound user session.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[doc = concat!("Public Trellis data type `", stringify!(BoundSession), "`.")]
pub struct BoundSession {
    /// Inbox prefix authorized for the bound session.
    #[serde(rename = "inboxPrefix")]
    #[doc = concat!("The `", stringify!(inbox_prefix), "` value.")]
    pub inbox_prefix: String,
    /// Durable Trellis session identifier.
    #[doc = concat!("The `", stringify!(session_id), "` value.")]
    pub session_id: String,
    /// Session expiry in Unix milliseconds, when bounded.
    #[doc = concat!("The `", stringify!(expires_at), "` value.")]
    pub expires_at: Option<i64>,
    /// Comma-separated native transport endpoints for the session.
    #[serde(rename = "nats_servers", alias = "servers")]
    #[doc = concat!("The `", stringify!(servers), "` value.")]
    pub servers: String,
    /// Deny-all Auth-account JWT used to enter NATS Auth Callout.
    #[doc = concat!("The `", stringify!(bootstrap_jwt), "` value.")]
    pub bootstrap_jwt: String,
    /// Signed authorization context and minimal trust metadata.
    pub authorization_context: AuthorizationContextBundle,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(BindResponseBound), "`.")]
pub struct BindResponseBound {
    #[doc = concat!("The `", stringify!(session), "` value.")]
    pub session: BoundSessionRecord,
    #[doc = concat!("The `", stringify!(nats), "` value.")]
    pub nats: BoundNatsRecord,
    #[doc = concat!("The `", stringify!(authorization_context), "` value.")]
    pub authorization_context: AuthorizationContextBundle,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(BoundSessionRecord), "`.")]
pub struct BoundSessionRecord {
    #[doc = concat!("The `", stringify!(session_id), "` value.")]
    pub session_id: String,
    #[doc = concat!("The `", stringify!(inbox_prefix), "` value.")]
    pub inbox_prefix: String,
    #[doc = concat!("The `", stringify!(expires_at), "` value.")]
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[doc = concat!("Public Trellis data type `", stringify!(BoundNatsRecord), "`.")]
pub struct BoundNatsRecord {
    #[doc = concat!("The `", stringify!(jwt), "` value.")]
    pub jwt: String,
    #[doc = concat!("The `", stringify!(servers), "` value.")]
    pub servers: Vec<String>,
}

/// An in-progress agent login flow waiting for completion.
#[doc = concat!("Public Trellis data type `", stringify!(AgentLoginChallenge), "`.")]
pub struct AgentLoginChallenge {
    #[doc = concat!("The `", stringify!(flow_id), "` value.")]
    pub flow_id: String,
    #[doc = concat!("The `", stringify!(login_url), "` value.")]
    pub login_url: String,
    #[doc = concat!("The `", stringify!(session_seed), "` value.")]
    pub session_seed: String,
    #[doc = concat!("The `", stringify!(participant_digest), "` value.")]
    pub participant_digest: String,
    #[doc = concat!("The `", stringify!(auth), "` value.")]
    pub auth: SessionAuth,
}

/// Options for starting an agent login flow.
pub struct StartAgentLoginOpts<'a> {
    /// Base URL for the Trellis deployment.
    #[doc = concat!("The `", stringify!(trellis_url), "` value.")]
    pub trellis_url: &'a str,
}

/// Successful agent-login result after the admin user has been verified.
#[doc = concat!("Public Trellis data type `", stringify!(AdminLoginOutcome), "`.")]
pub struct AdminLoginOutcome {
    /// Persistable admin session state for later CLI reuse.
    #[doc = concat!("The `", stringify!(state), "` value.")]
    pub state: AdminSessionState,
    /// Authenticated user returned by `Auth.Sessions.Me` after bind succeeds.
    #[doc = concat!("The `", stringify!(user), "` value.")]
    pub user: AuthenticatedUser,
}

/// Result of starting admin reauthentication for a changed contract.
#[doc = concat!("Public Trellis value set `", stringify!(AdminReauthOutcome), "`.")]
#[allow(clippy::large_enum_variant)]
pub enum AdminReauthOutcome {
    /// Contract change was auto-approved and the session was rebound immediately.
    Bound(AdminLoginOutcome),
    /// External interaction is still required to finish the agent auth flow.
    Flow(AgentLoginChallenge),
}

/// Derived device identity material used by the device activation helpers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceIdentity), "`.")]
pub struct DeviceIdentity {
    #[serde(rename = "identitySeedBase64url")]
    #[doc = concat!("The `", stringify!(identity_seed_base64url), "` value.")]
    pub identity_seed_base64url: String,
    #[serde(rename = "publicIdentityKey")]
    #[doc = concat!("The `", stringify!(public_identity_key), "` value.")]
    pub public_identity_key: String,
    #[serde(rename = "activationKeyBase64url")]
    #[doc = concat!("The `", stringify!(activation_key_base64url), "` value.")]
    pub activation_key_base64url: String,
}

/// Encoded device activation payload carried in the activation QR.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceActivationPayload), "`.")]
pub struct DeviceActivationPayload {
    #[doc = concat!("The `", stringify!(v), "` value.")]
    pub v: u8,
    #[serde(rename = "publicIdentityKey")]
    #[doc = concat!("The `", stringify!(public_identity_key), "` value.")]
    pub public_identity_key: String,
    #[doc = concat!("The `", stringify!(nonce), "` value.")]
    pub nonce: String,
    #[serde(rename = "qrMac")]
    #[doc = concat!("The `", stringify!(qr_mac), "` value.")]
    pub qr_mac: String,
}

/// Signed pre-auth request sent to `/auth/devices/activate/wait`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceActivationWaitRequest), "`.")]
pub struct DeviceActivationWaitRequest {
    #[serde(rename = "flowId")]
    #[doc = concat!("The `", stringify!(flow_id), "` value.")]
    pub flow_id: String,
    #[serde(rename = "publicIdentityKey")]
    #[doc = concat!("The `", stringify!(public_identity_key), "` value.")]
    pub public_identity_key: String,
    #[serde(rename = "contractDigest", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(contract_digest), "` value.")]
    pub contract_digest: Option<String>,
    #[doc = concat!("The `", stringify!(nonce), "` value.")]
    pub nonce: String,
    #[doc = concat!("The `", stringify!(iat), "` value.")]
    pub iat: u64,
    #[doc = concat!("The `", stringify!(sig), "` value.")]
    pub sig: String,
}

/// Signed pre-auth request sent to `/auth/devices/connect-info`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceConnectInfoRequest), "`.")]
pub struct DeviceConnectInfoRequest {
    #[doc = concat!("The `", stringify!(public_identity_key), "` value.")]
    pub public_identity_key: String,
    #[doc = concat!("The `", stringify!(contract_digest), "` value.")]
    pub contract_digest: String,
    #[doc = concat!("The `", stringify!(iat), "` value.")]
    pub iat: u64,
    #[doc = concat!("The `", stringify!(sig), "` value.")]
    pub sig: String,
}

/// Native transport endpoints returned for an activated device.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceConnectInfoNativeTransport), "`.")]
pub struct DeviceConnectInfoNativeTransport {
    #[serde(rename = "natsServers")]
    #[doc = concat!("The `", stringify!(servers), "` value.")]
    pub servers: Vec<String>,
}

/// Transport endpoints returned for an activated device.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceConnectInfoTransports), "`.")]
pub struct DeviceConnectInfoTransports {
    #[doc = concat!("The `", stringify!(native), "` value.")]
    pub native: Option<DeviceConnectInfoNativeTransport>,
}

/// Sentinel credentials returned for an activated device connection.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceConnectInfoSentinel), "`.")]
pub struct DeviceConnectInfoSentinel {
    #[doc = concat!("The `", stringify!(jwt), "` value.")]
    pub jwt: String,
    #[doc = concat!("The `", stringify!(seed), "` value.")]
    pub seed: String,
}

/// Selected runtime transport credentials for an activated device.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceConnectInfoTransport), "`.")]
pub struct DeviceConnectInfoTransport {
    #[doc = concat!("The `", stringify!(sentinel), "` value.")]
    pub sentinel: DeviceConnectInfoSentinel,
}

/// Activated-device runtime auth settings returned by auth.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[doc = concat!("Public Trellis value set `", stringify!(DeviceConnectInfoAuthMode), "`.")]
pub enum DeviceConnectInfoAuthMode {
    /// Device authenticates with its durable device identity key.
    DeviceIdentity,
}

/// Activated-device runtime auth settings returned by auth.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceConnectInfoAuth), "`.")]
pub struct DeviceConnectInfoAuth {
    #[doc = concat!("The `", stringify!(mode), "` value.")]
    pub mode: DeviceConnectInfoAuthMode,
    #[doc = concat!("The `", stringify!(iat_skew_seconds), "` value.")]
    pub iat_skew_seconds: i64,
}

/// Current runtime connection information for an activated device.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceConnectInfo), "`.")]
pub struct DeviceConnectInfo {
    #[doc = concat!("The `", stringify!(instance_id), "` value.")]
    pub instance_id: String,
    #[doc = concat!("The `", stringify!(deployment_id), "` value.")]
    pub deployment_id: String,
    #[doc = concat!("The `", stringify!(contract_id), "` value.")]
    pub contract_id: String,
    #[doc = concat!("The `", stringify!(contract_digest), "` value.")]
    pub contract_digest: String,
    #[doc = concat!("The `", stringify!(transports), "` value.")]
    pub transports: DeviceConnectInfoTransports,
    #[doc = concat!("The `", stringify!(transport), "` value.")]
    pub transport: DeviceConnectInfoTransport,
    #[doc = concat!("The `", stringify!(auth), "` value.")]
    pub auth: DeviceConnectInfoAuth,
}

/// Ready response returned by `/auth/devices/connect-info`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceConnectInfoResponse), "`.")]
pub struct DeviceConnectInfoResponse {
    #[doc = concat!("The `", stringify!(status), "` value.")]
    pub status: String,
    #[doc = concat!("The `", stringify!(connect_info), "` value.")]
    pub connect_info: DeviceConnectInfo,
}

/// Options for refreshing activated-device runtime connection information.
pub struct GetDeviceConnectInfoOpts<'a> {
    #[doc = concat!("The `", stringify!(trellis_url), "` value.")]
    pub trellis_url: &'a str,
    #[doc = concat!("The `", stringify!(public_identity_key), "` value.")]
    pub public_identity_key: &'a str,
    #[doc = concat!("The `", stringify!(identity_seed_base64url), "` value.")]
    pub identity_seed_base64url: &'a str,
    #[doc = concat!("The `", stringify!(contract_digest), "` value.")]
    pub contract_digest: &'a str,
    #[doc = concat!("The `", stringify!(iat), "` value.")]
    pub iat: u64,
}

/// Activated wait response returned by auth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceActivationActivatedResponse), "`.")]
pub struct DeviceActivationActivatedResponse {
    #[doc = concat!("The `", stringify!(status), "` value.")]
    pub status: String,
    #[serde(rename = "activatedAt")]
    #[doc = concat!("The `", stringify!(activated_at), "` value.")]
    pub activated_at: String,
    #[serde(rename = "confirmationCode", skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(confirmation_code), "` value.")]
    pub confirmation_code: Option<String>,
    #[serde(rename = "connectInfo")]
    #[doc = concat!("The `", stringify!(connect_info), "` value.")]
    pub connect_info: serde_json::Value,
}

/// Rejected wait response returned by auth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceActivationRejectedResponse), "`.")]
pub struct DeviceActivationRejectedResponse {
    #[doc = concat!("The `", stringify!(status), "` value.")]
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(reason), "` value.")]
    pub reason: Option<String>,
}

/// Pending wait response returned by auth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DeviceActivationPendingResponse), "`.")]
pub struct DeviceActivationPendingResponse {
    #[doc = concat!("The `", stringify!(status), "` value.")]
    pub status: String,
}

/// Union of possible wait responses returned by auth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "lowercase")]
#[doc = concat!("Public Trellis value set `", stringify!(WaitForDeviceActivationResponse), "`.")]
pub enum WaitForDeviceActivationResponse {
    Activated {
        #[serde(rename = "activatedAt")]
        activated_at: String,
        #[serde(rename = "confirmationCode", skip_serializing_if = "Option::is_none")]
        confirmation_code: Option<String>,
        #[serde(rename = "connectInfo")]
        connect_info: serde_json::Value,
    },
    Rejected {
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    Pending,
}

/// Polling options for waiting on an activated device.
pub struct WaitForDeviceActivationOpts<'a> {
    #[doc = concat!("The `", stringify!(trellis_url), "` value.")]
    pub trellis_url: &'a str,
    #[doc = concat!("The `", stringify!(flow_id), "` value.")]
    pub flow_id: &'a str,
    #[doc = concat!("The `", stringify!(public_identity_key), "` value.")]
    pub public_identity_key: &'a str,
    #[doc = concat!("The `", stringify!(nonce), "` value.")]
    pub nonce: &'a str,
    #[doc = concat!("The `", stringify!(identity_seed_base64url), "` value.")]
    pub identity_seed_base64url: &'a str,
    #[doc = concat!("The `", stringify!(contract_digest), "` value.")]
    pub contract_digest: Option<&'a str>,
    #[doc = concat!("The `", stringify!(poll_interval), "` value.")]
    pub poll_interval: Duration,
}
