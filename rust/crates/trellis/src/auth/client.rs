use super::{AdminSessionState, TrellisAuthError};
use crate::client::{SessionAuth, UserConnectOptions, UserSessionCredentials};
use crate::generated::Caller;

/// Connect an authenticated admin client from stored session state.
pub async fn connect_admin_client_async(
    state: &AdminSessionState,
) -> Result<Caller, TrellisAuthError> {
    let participant: serde_json::Value =
        serde_json::from_str(include_str!("../../artifacts/trellis.cli.participant.json"))?;
    let participant = trellis_protocol::parse_participant(&participant)?;
    Ok(Caller::connect_user(UserConnectOptions::new(
        &state.trellis_url,
        5_000,
        UserSessionCredentials {
            login_session_id: &state.login_session_id,
            session_key_seed_base64url: &state.session_seed,
        },
        participant.id(),
    ))
    .await?)
}

/// Derive the public session key for a base64url-encoded session seed.
pub fn session_public_key(seed_base64url: &str) -> Result<String, TrellisAuthError> {
    Ok(SessionAuth::from_seed_base64url(seed_base64url)?.session_key)
}
