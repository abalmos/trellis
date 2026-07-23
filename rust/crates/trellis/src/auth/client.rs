use super::{
    AdminSessionState, AuthRequestsValidateRequest, AuthRequestsValidateResponse, TrellisAuthError,
};
use crate::client::{SessionAuth, UserConnectOptions};
use crate::generated::Caller;

/// Connect an authenticated admin client from stored session state.
pub async fn connect_admin_client_async(
    state: &AdminSessionState,
) -> Result<Caller, TrellisAuthError> {
    Ok(Caller::connect_user(UserConnectOptions::new(
        &state.servers,
        &state.bootstrap_jwt,
        &state.session_id,
        &state.inbox_prefix,
        &state.session_seed,
        &state.participant_digest,
        5_000,
    ))
    .await?)
}

/// Derive the public session key for a base64url-encoded session seed.
pub fn session_public_key(seed_base64url: &str) -> Result<String, TrellisAuthError> {
    Ok(SessionAuth::from_seed_base64url(seed_base64url)?.session_key)
}

/// Transitional ordinary-request validator client removed in Milestone 10.
pub struct TransitionalAuthClient<'a> {
    inner: &'a Caller,
}

impl<'a> TransitionalAuthClient<'a> {
    /// Wrap an authenticated generated caller.
    pub fn new(inner: &'a Caller) -> Self {
        Self { inner }
    }

    /// Validate one signed request through `Auth.Requests.Validate`.
    pub async fn validate_request(
        &self,
        input: &AuthRequestsValidateRequest,
    ) -> Result<AuthRequestsValidateResponse, TrellisAuthError> {
        let request = serde_json::to_value(input)?;
        let response = self
            .inner
            .request_json_value("rpc.v1.Auth.Requests.Validate", &request)
            .await?;
        Ok(serde_json::from_value(response)?)
    }
}
