use serde::{Deserialize, Serialize};

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
pub enum AdminReauthOutcome {
    /// Contract change was auto-approved and the session was rebound immediately.
    Bound(Box<AdminLoginOutcome>),
    /// External interaction is still required to finish the agent auth flow.
    Flow(Box<AgentLoginChallenge>),
}

/// Derived device identity material used to provision and bootstrap a device.
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
    /// Secret used to derive activation confirmation codes. Keep it device-local.
    pub activation_key_base64url: String,
}
