use super::{AdminSessionState, TrellisAuthError};
use std::sync::Arc;

use crate::client::{
    AuthorizationContextStore, FileAuthorizationContextStore, SessionAuth,
    UserAuthorizationContext, UserConnectOptions, UserSessionCredentials,
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
        &state.nats_servers,
        &state.inbox_prefix,
        5_000,
        UserSessionCredentials {
            bootstrap_jwt: &state.bootstrap_jwt,
            session_id: &state.session_id,
            session_key_seed_base64url: &state.session_seed,
            participant_digest: &state.participant_digest,
        },
        UserAuthorizationContext {
            bundle: state.authorization_context.clone(),
            binding,
            store,
        },
    ))
    .await?)
}

/// Derive the public session key for a base64url-encoded session seed.
pub fn session_public_key(seed_base64url: &str) -> Result<String, TrellisAuthError> {
    Ok(SessionAuth::from_seed_base64url(seed_base64url)?.session_key)
}
