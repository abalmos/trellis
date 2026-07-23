use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// User record returned by `Auth.Sessions.Me`.
#[doc = concat!("Public Trellis data type `", stringify!(AuthenticatedUser), "`.")]
pub struct AuthenticatedUser {
    /// Stable principal identifier.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// Current principal lifecycle state.
    pub state: String,
    #[doc = concat!("The `", stringify!(capabilities), "` value.")]
    pub capabilities: Vec<String>,
    /// Required-nullable profile email.
    pub email: Option<String>,
    #[doc = concat!("The `", stringify!(image), "` value.")]
    pub image: Option<String>,
    /// Required-nullable display name.
    #[doc = concat!("The `", stringify!(name), "` value.")]
    pub name: Option<String>,
    #[serde(rename = "userId")]
    #[doc = concat!("The `", stringify!(user_id), "` value.")]
    pub user_id: String,
    /// Account creation time in Unix milliseconds.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// Last profile update time in Unix milliseconds.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// Required-nullable account disable time.
    #[serde(rename = "disabledAt")]
    pub disabled_at: Option<i64>,
    /// Required-nullable account revocation time.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// Optimistic account version.
    pub version: u64,
}

/// Transitional request payload for `Auth.Requests.Validate`.
///
/// This wire type is removed with ordinary request-proof v2 in Milestone 10.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequestsValidateRequest {
    /// Session Ed25519 public key.
    pub session_key: String,
    /// Detached request signature.
    pub proof: String,
    /// Requested NATS subject.
    pub subject: String,
    /// Base64url SHA-256 payload digest.
    pub payload_hash: String,
    /// Signature creation time in Unix seconds.
    pub iat: i64,
    /// Unique request identifier.
    pub request_id: String,
    /// Capabilities required by the called surface.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
}

/// Transitional response returned by `Auth.Requests.Validate`.
///
/// This wire type is removed with ordinary request-proof v2 in Milestone 10.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequestsValidateResponse {
    /// Whether the request is authorized.
    pub allowed: bool,
    /// Session-scoped reply inbox prefix.
    pub inbox_prefix: String,
    /// Authenticated caller projection.
    pub caller: Value,
}

/// Transitional request payload for `Auth.Events.Validate`.
///
/// This wire type is removed with local event-proof validation in Milestone 10.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthEventsValidateRequest {
    /// Session Ed25519 public key.
    pub session_key: String,
    /// Detached event signature.
    pub proof: String,
    /// Published event subject.
    pub subject: String,
    /// Base64url SHA-256 payload digest.
    pub payload_hash: String,
    /// Stable event identifier.
    pub event_id: String,
    /// RFC 3339 event time.
    pub event_time: String,
}

/// Transitional event-validation status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthEventValidationStatus {
    /// Event proof and authority are valid.
    Verified,
    /// Session is unknown.
    MissingSession,
    /// Signature is invalid.
    InvalidSignature,
    /// Event subject is not authorized.
    SubjectDenied,
    /// Event falls outside the retained session window.
    OutsideSessionWindow,
}

impl AuthEventValidationStatus {
    /// Return the protocol wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::MissingSession => "missing-session",
            Self::InvalidSignature => "invalid-signature",
            Self::SubjectDenied => "subject-denied",
            Self::OutsideSessionWindow => "outside-session-window",
        }
    }
}

/// Publisher projection returned by transitional event validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthEventPublisher {
    /// Principal kind.
    pub kind: String,
    /// Deployment identity for deployed principals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
    /// Runtime instance identity for deployed principals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// Participant API identifier when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
    /// Participant artifact digest when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_digest: Option<String>,
    /// Session lifecycle state.
    pub session_status: String,
}

/// Transitional response returned by `Auth.Events.Validate`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthEventsValidateResponse {
    /// Whether the event is authorized.
    pub allowed: bool,
    /// Verification result.
    pub status: AuthEventValidationStatus,
    /// Authenticated caller projection.
    pub caller: Option<Value>,
    /// Publisher projection used by Event Log indexing.
    pub publisher: Option<AuthEventPublisher>,
}

#[cfg(test)]
mod tests {
    use super::AuthenticatedUser;
    use serde_json::json;

    #[test]
    fn authenticated_user_matches_clean_session_shape() {
        let value = json!({
            "userId": "usr_123",
            "principalId": "usr_123",
            "state": "active",
            "name": "Ada",
            "email": "ada@example.com",
            "image": "https://example.com/ada.png",
            "capabilities": ["users.read"],
            "createdAt": 1,
            "updatedAt": 2,
            "disabledAt": null,
            "revokedAt": null,
            "version": 3,
        });

        let user: AuthenticatedUser = serde_json::from_value(value).expect("deserialize user");

        assert_eq!(user.user_id, "usr_123");
        assert_eq!(user.principal_id, "usr_123");
        assert_eq!(user.version, 3);
    }
}
