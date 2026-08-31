use super::{AdminSessionState, TrellisAuthError};
use std::sync::Arc;

use crate::client::{
    AuthorizationContextStore, AuthorizationInstallation, FileAuthorizationContextStore,
    SessionAuth, UserAuthorizationContext, UserConnectOptions, UserSessionCredentials,
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
        None,
    )
    .await
}

pub(super) async fn connect_admin_client_with_context_store_async(
    state: &AdminSessionState,
    binding: String,
    store: Arc<dyn AuthorizationContextStore>,
    initial: Option<AuthorizationInstallation>,
) -> Result<Caller, TrellisAuthError> {
    Ok(Caller::connect_user(UserConnectOptions::new(
        &state.trellis_url,
        5_000,
        UserSessionCredentials {
            session_key_seed_base64url: &state.session_seed,
        },
        UserAuthorizationContext {
            initial,
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
