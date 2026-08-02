use serde::{Deserialize, Serialize};

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
