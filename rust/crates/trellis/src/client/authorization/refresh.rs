use std::sync::Arc;

use serde_json::json;
use trellis_protocol::{
    AuthorizationContextRefreshSessionProofInput, AuthorizationPrincipalKind,
    NativeBootstrapSessionProofInput, SessionProofInput,
};

use super::super::{proof::new_request_id, SessionAuth, TrellisClientError};
use super::own_context::{system_now_millis, AuthorizationContextCache};
use super::types::{AuthorizationCredential, AuthorizationInstallation};

fn is_terminal_refresh_error(code: &str) -> bool {
    matches!(
        code,
        "session_not_found"
            | "session_expired"
            | "session_revoked"
            | "user_not_found"
            | "user_inactive"
            | "identity_not_found"
            | "identity_revoked"
            | "grant_binding_missing"
            | "grant_binding_revoked"
            | "grant_binding_expired"
            | "participant_not_installed"
            | "deployment_inactive"
            | "instance_inactive"
            | "device_inactive"
            | "activation_required"
            | "delegation_expired"
            | "context_refresh_mismatch"
            | "invalid_proof"
    )
}

/// Obtain or renew connection authority using only the owner credential and proof.
pub(crate) async fn refresh(
    cache: &AuthorizationContextCache,
    auth: &SessionAuth,
) -> Result<bool, TrellisClientError> {
    if auth.session_key != cache.session_key {
        return Err(TrellisClientError::Bootstrap(
            "refresh signing key does not belong to this connection".into(),
        ));
    }
    let observed_digest = cache.context_digest().ok();
    let _refresh = cache.lock_refresh().await;
    if observed_digest != cache.context_digest().ok() {
        return Ok(false);
    }
    let previous = cache.state_snapshot()?;
    let request_started_at = system_now_millis()?;
    let issued_at = cache.corrected_now_millis()?;
    let mut request = json!({"requestId": new_request_id(), "connectionId": cache.connection_id});
    if let Some(name) = &cache.name {
        request["name"] = json!(name);
    }
    let (route, input, signer) = match cache.credential.as_ref() {
        AuthorizationCredential::Native { kind, identity } => {
            request["identityKeyId"] = json!(identity.key_id());
            request["sessionKey"] = json!(auth.session_key);
            request["iat"] = json!(issued_at);
            let input = NativeBootstrapSessionProofInput {
                origin: cache.http().origin(),
                unsigned_request: request.clone(),
            };
            match kind {
                AuthorizationPrincipalKind::Service => (
                    "/bootstrap/service",
                    SessionProofInput::service_bootstrap(input),
                    identity.as_ref(),
                ),
                AuthorizationPrincipalKind::Device => (
                    "/bootstrap/device",
                    SessionProofInput::device_bootstrap(input),
                    identity.as_ref(),
                ),
                AuthorizationPrincipalKind::User => {
                    return Err(TrellisClientError::Bootstrap(
                        "user login cannot use a native credential".into(),
                    ))
                }
            }
        }
        AuthorizationCredential::User { login_session_id } => {
            request["loginSessionId"] = json!(login_session_id);
            request["issuedAt"] = json!(issued_at);
            request["currentContextDigest"] = json!(observed_digest);
            (
                "/auth/context/refresh",
                SessionProofInput::authorization_context_refresh(
                    AuthorizationContextRefreshSessionProofInput {
                        origin: cache.http().origin(),
                        session_public_key: auth.session_key.clone(),
                        unsigned_request: request.clone(),
                    },
                ),
                auth,
            )
        }
    };
    let input = input.map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    request["proof"] = serde_json::to_value(signer.sign_session_proof(&input)?)?;
    let response = cache.http().post_json(route, &request).await?;
    let server_now = response["serverNow"]
        .as_i64()
        .filter(|now| (0..=9_007_199_254_740_991).contains(now))
        .ok_or_else(|| {
            TrellisClientError::Bootstrap("bootstrap omitted a safe server time".into())
        })?;
    let midpoint = request_started_at
        .checked_add(system_now_millis()?)
        .and_then(|sum| sum.checked_div(2))
        .ok_or_else(|| TrellisClientError::Bootstrap("bootstrap time overflow".into()))?;
    let mut runtime = response["runtime"].clone();
    runtime
        .as_object_mut()
        .ok_or_else(|| TrellisClientError::Bootstrap("bootstrap runtime is not an object".into()))?
        .insert("transports".into(), response["transports"].clone());
    cache.install(AuthorizationInstallation {
        context: serde_json::from_value(response["authorizationContext"].clone())?,
        routing: serde_json::from_value(response["routing"].clone())?,
        runtime: serde_json::from_value(runtime)?,
        server_clock_offset_ms: server_now
            .checked_sub(midpoint)
            .ok_or_else(|| TrellisClientError::Bootstrap("bootstrap time overflow".into()))?,
        authorization: response
            .get("authorization")
            .filter(|value| !value.is_null())
            .cloned(),
    })?;
    Ok(previous.runtime.as_ref() != Some(&cache.runtime_binding()?))
}

/// Background own-context refresh on the retained NATS connection.
pub(crate) fn spawn_authorization_context_refresh_task(
    contexts: Arc<AuthorizationContextCache>,
    auth: Arc<SessionAuth>,
    nats: async_nats::Client,
    timeout_ms: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let delay = match contexts.refresh_delay() {
                Ok(delay) => delay,
                Err(error) => {
                    tracing::warn!(%error, "authorization context refresh stopped");
                    return;
                }
            };
            let requested_digest = tokio::select! {
                () = tokio::time::sleep(delay) => None,
                digest = contexts.wait_refresh_request() => Some(digest),
            };
            if requested_digest.is_some_and(|digest| contexts.context_digest().ok() != digest) {
                continue;
            }
            let previous = match contexts.state_snapshot() {
                Ok(state) => state,
                Err(error) => {
                    tracing::warn!(%error, "authorization context refresh stopped");
                    return;
                }
            };
            match refresh(&contexts, &auth).await {
                Ok(_) => {
                    let refreshed = match contexts.state_snapshot() {
                        Ok(state) => state,
                        Err(error) => {
                            tracing::warn!(%error, "refreshed native runtime is invalid");
                            continue;
                        }
                    };
                    if let (Some(previous_runtime), Some(runtime)) =
                        (previous.runtime.as_ref(), refreshed.runtime.as_ref())
                    {
                        let changed = previous
                            .current
                            .as_ref()
                            .map(|current| &current.context_digest)
                            != refreshed
                                .current
                                .as_ref()
                                .map(|current| &current.context_digest);
                        if let Err(error) = super::super::connection::apply_native_runtime_refresh(
                            &nats,
                            previous_runtime,
                            runtime,
                            changed,
                            timeout_ms,
                        )
                        .await
                        {
                            tracing::warn!(%error, "native connection refresh will retry");
                        }
                    }
                }
                Err(TrellisClientError::BootstrapHttp { status, code })
                    if is_terminal_refresh_error(&code) =>
                {
                    tracing::warn!(status, "authorization context refresh rejected");
                    if let Err(error) = contexts.clear() {
                        tracing::warn!(%error, "failed to clear rejected authorization context");
                    }
                    let _ = nats.drain().await;
                    return;
                }
                Err(error) => {
                    tracing::warn!(%error, "authorization context refresh will retry");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::is_terminal_refresh_error;

    #[test]
    fn refresh_terminality_uses_exact_machine_codes() {
        assert!(is_terminal_refresh_error("session_revoked"));
        assert!(is_terminal_refresh_error("grant_binding_revoked"));
        assert!(is_terminal_refresh_error("context_refresh_mismatch"));
        assert!(!is_terminal_refresh_error("required_resources_unavailable"));
        assert!(!is_terminal_refresh_error("session_revoked later"));
    }
}
