use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Registry bucket metadata for a jobs binding.
#[doc = concat!("Public Trellis data type `", stringify!(JobsRegistry), "`.")]
pub struct JobsRegistry {
    #[doc = concat!("The `", stringify!(bucket), "` value.")]
    pub bucket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Jobs resource bindings attached to materialized deployment authority.
#[doc = concat!("Public Trellis data type `", stringify!(JobsBindings), "`.")]
pub struct JobsBindings {
    #[doc = concat!("The `", stringify!(namespace), "` value.")]
    pub namespace: String,
    #[doc = concat!("The `", stringify!(queues), "` value.")]
    pub queues: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(registry), "` value.")]
    pub registry: Option<JobsRegistry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Resource bindings granted through materialized deployment authority.
#[doc = concat!("Public Trellis data type `", stringify!(ResourceBindings), "`.")]
pub struct ResourceBindings {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(jobs), "` value.")]
    pub jobs: Option<JobsBindings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(kv), "` value.")]
    pub kv: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
/// Identity bound to the authenticated user session.
#[doc = concat!("Public Trellis data type `", stringify!(AuthenticatedIdentity), "`.")]
pub struct AuthenticatedIdentity {
    #[doc = concat!("The `", stringify!(identity_id), "` value.")]
    pub identity_id: String,
    #[doc = concat!("The `", stringify!(provider), "` value.")]
    pub provider: String,
    #[doc = concat!("The `", stringify!(subject), "` value.")]
    pub subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// User record returned by `Auth.Sessions.Me`.
#[doc = concat!("Public Trellis data type `", stringify!(AuthenticatedUser), "`.")]
pub struct AuthenticatedUser {
    #[doc = concat!("The `", stringify!(active), "` value.")]
    pub active: bool,
    #[doc = concat!("The `", stringify!(capabilities), "` value.")]
    pub capabilities: Vec<String>,
    #[doc = concat!("The `", stringify!(email), "` value.")]
    pub email: String,
    #[doc = concat!("The `", stringify!(identity), "` value.")]
    pub identity: AuthenticatedIdentity,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(image), "` value.")]
    pub image: Option<String>,
    #[serde(rename = "lastLogin")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(last_login), "` value.")]
    pub last_login: Option<String>,
    #[doc = concat!("The `", stringify!(name), "` value.")]
    pub name: String,
    #[serde(rename = "userId")]
    #[doc = concat!("The `", stringify!(user_id), "` value.")]
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Sentinel credentials returned alongside a successful bind.
#[doc = concat!("Public Trellis data type `", stringify!(SentinelCredsRecord), "`.")]
pub struct SentinelCredsRecord {
    #[doc = concat!("The `", stringify!(jwt), "` value.")]
    pub jwt: String,
    #[doc = concat!("The `", stringify!(seed), "` value.")]
    pub seed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// One named transport endpoint family returned alongside a successful bind.
#[doc = concat!("Public Trellis data type `", stringify!(ClientTransportRecord), "`.")]
pub struct ClientTransportRecord {
    #[serde(rename = "natsServers")]
    #[doc = concat!("The `", stringify!(servers), "` value.")]
    pub servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Transport variants returned alongside a successful bind.
#[doc = concat!("Public Trellis data type `", stringify!(ClientTransportsRecord), "`.")]
pub struct ClientTransportsRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(native), "` value.")]
    pub native: Option<ClientTransportRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(websocket), "` value.")]
    pub websocket: Option<ClientTransportRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
/// Request payload for `POST /auth/requests`.
#[doc = concat!("Public Trellis data type `", stringify!(AuthStartRequest), "`.")]
pub struct AuthStartRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(provider), "` value.")]
    pub provider: Option<String>,
    #[doc = concat!("The `", stringify!(redirect_to), "` value.")]
    pub redirect_to: String,
    #[doc = concat!("The `", stringify!(session_key), "` value.")]
    pub session_key: String,
    #[doc = concat!("The `", stringify!(sig), "` value.")]
    pub sig: String,
    #[doc = concat!("The `", stringify!(contract), "` value.")]
    pub contract: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(context), "` value.")]
    pub context: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
/// Response payload for `POST /auth/requests`.
#[doc = concat!("Public Trellis value set `", stringify!(AuthStartResponse), "`.")]
pub enum AuthStartResponse {
    Bound {
        expires: String,
        #[serde(rename = "inboxPrefix")]
        inbox_prefix: String,
        sentinel: SentinelCredsRecord,
        transports: ClientTransportsRecord,
    },
    FlowStarted {
        #[serde(rename = "flowId")]
        flow_id: String,
        #[serde(rename = "loginUrl")]
        login_url: String,
    },
}

/// Filter parameters for `Auth.IdentityGrants.List`.
pub type ListIdentityGrantsRequest = crate::sdk::auth::types::AuthIdentityGrantsListRequest;

/// Contract evidence returned by `Auth.IdentityGrants.List`.
pub type IdentityGrantContractEvidenceRecord =
    crate::sdk::auth::types::AuthIdentityGrantsListResponseEntriesItemContractEvidence;

/// Stored identity grant returned by `Auth.IdentityGrants.List`.
pub type IdentityGrantEntryRecord =
    crate::sdk::auth::types::AuthIdentityGrantsListResponseEntriesItem;

/// Request payload for `Auth.IdentityGrants.Revoke`.
pub type RevokeIdentityGrantRequest = crate::sdk::auth::types::AuthIdentityGrantsRevokeRequest;

/// Request payload for `Auth.Requests.Validate`.
pub type AuthRequestsValidateRequest = crate::sdk::auth::types::AuthRequestsValidateRequest;

/// Response payload returned by `Auth.Requests.Validate`.
pub type AuthRequestsValidateResponse = crate::sdk::auth::types::AuthRequestsValidateResponse;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[doc = concat!("Public Trellis data type `", stringify!(LogoutResponse), "`.")]
pub struct LogoutResponse {
    #[doc = concat!("The `", stringify!(success), "` value.")]
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::{AuthenticatedIdentity, AuthenticatedUser};
    use serde_json::json;

    #[test]
    fn authenticated_user_uses_account_first_session_shape() {
        let value = json!({
            "userId": "usr_123",
            "active": true,
            "name": "Ada",
            "email": "ada@example.com",
            "image": "https://example.com/ada.png",
            "identity": {
                "identityId": "idn_github_123",
                "provider": "github",
                "subject": "123",
            },
            "capabilities": ["users.read"],
            "lastLogin": "2026-04-10T00:00:00.000Z",
        });

        let user: AuthenticatedUser = serde_json::from_value(value).expect("deserialize user");

        assert_eq!(user.user_id, "usr_123");
        assert_eq!(
            user.identity,
            AuthenticatedIdentity {
                identity_id: "idn_github_123".to_string(),
                provider: "github".to_string(),
                subject: "123".to_string(),
            }
        );
        assert_eq!(user.last_login.as_deref(), Some("2026-04-10T00:00:00.000Z"));
    }
}
