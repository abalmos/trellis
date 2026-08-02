use super::{AdminSessionState, TrellisAuthError};
use std::sync::Arc;

use crate::client::{
    AuthorizationContextStore, FileAuthorizationContextStore, SessionAuth, UserConnectOptions,
};
use crate::generated::Caller;

/// Connect an authenticated admin client from stored session state.
pub async fn connect_admin_client_async(
    state: &AdminSessionState,
) -> Result<Caller, TrellisAuthError> {
    connect_admin_client_with_context_store_async(
        state,
        format!("installation:{}", state.trellis_url),
        Arc::new(FileAuthorizationContextStore::new(
            super::session_store::admin_authorization_context_state_path(),
        )),
    )
    .await
}

pub(super) async fn connect_admin_client_with_context_store_async(
    state: &AdminSessionState,
    binding: String,
    store: Arc<dyn AuthorizationContextStore>,
) -> Result<Caller, TrellisAuthError> {
    Ok(Caller::connect_user(UserConnectOptions::new(
        &state.trellis_url,
        &state.servers,
        &state.bootstrap_jwt,
        &state.session_id,
        &state.inbox_prefix,
        &state.session_seed,
        &state.participant_digest,
        state.authorization_context.clone(),
        5_000,
        binding,
        store,
    ))
    .await?)
}

/// Derive the public session key for a base64url-encoded session seed.
pub fn session_public_key(seed_base64url: &str) -> Result<String, TrellisAuthError> {
    Ok(SessionAuth::from_seed_base64url(seed_base64url)?.session_key)
}
