use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use trellis_protocol::{
    session_proof_request_digest, AuthorizationContextRefreshSessionProofInput, SessionProofInput,
};

use super::super::{proof::new_request_id, SessionAuth, TrellisClientError};
use super::own_context::AuthorizationContextCache;
use super::types::{
    AuthorizationContextBundle, AuthorizationInstallation, AuthorizationNativeTransport,
    AuthorizationRoutingMaterial, AuthorizationRuntimeBinding, AuthorizationRuntimeTransports,
};

fn is_terminal_refresh_error(code: &str) -> bool {
    matches!(
        code,
        "session_not_found"
            | "session_expired"
            | "session_revoked"
            | "user_not_found"
            | "user_inactive"
            | "participant_not_found"
            | "participant_changed"
            | "contract_changed"
            | "authority_not_found"
            | "authority_rejected"
            | "authority_revoked"
            | "authority_expired"
            | "deployment_inactive"
            | "instance_inactive"
            | "device_inactive"
            | "activation_required"
            | "delegation_expired"
            | "context_refresh_mismatch"
            | "invalid_proof"
    )
}

/// Proof-bound refresh request through the credential/context recovery endpoint.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshRequest {
    request_id: String,
    issued_at: i64,
    session_id: String,
    session_nkey: String,
    current_context_digest: Option<String>,
    expected_participant_digest: Option<String>,
    expected_needs_digest: Option<String>,
    known_root_key_id: String,
    minimum_manifest_generation: i64,
    proof: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefreshResponse {
    server_now: i64,
    session: RefreshSession,
    nats: RefreshNats,
    authorization_context: AuthorizationContextBundle,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefreshSession {
    session_id: String,
    inbox_prefix: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefreshNats {
    jwt: String,
    jwt_expires_at: i64,
    transports: RefreshTransports,
}

#[derive(Deserialize)]
struct RefreshTransports {
    native: Option<RefreshTransport>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefreshTransport {
    nats_servers: Vec<String>,
}

/// Refresh the current context through the proof-bound auth endpoint.
///
/// This is the only HTTP request performed by a connected own-context cache and
/// is reserved for credential/context recovery.
pub(crate) async fn refresh(
    cache: &AuthorizationContextCache,
    auth: &SessionAuth,
) -> Result<bool, TrellisClientError> {
    let observed_digest = cache.context_digest().ok();
    let _refresh = cache.lock_refresh().await;
    if observed_digest != cache.context_digest().ok() {
        return Ok(false);
    }
    let now = cache.corrected_now_seconds()?;
    let state = cache.state_snapshot()?;
    let runtime = state
        .runtime
        .ok_or_else(|| TrellisClientError::Bootstrap("authorization session unavailable".into()))?;
    let previous_runtime = runtime.clone();
    let durable = cache.durable_state()?.ok_or_else(|| {
        TrellisClientError::Bootstrap("authorization trust floor unavailable".into())
    })?;
    let request_started_at = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
            .as_millis(),
    )
    .map_err(|_| TrellisClientError::Bootstrap("context refresh time overflow".into()))?;
    let issued_at = request_started_at
        .checked_add(cache.clock_offset_ms())
        .ok_or_else(|| TrellisClientError::Bootstrap("context refresh time overflow".into()))?;
    let mut request = RefreshRequest {
        request_id: new_request_id(),
        issued_at,
        session_id: runtime.session_id,
        session_nkey: auth.nkey_pair()?.public_key(),
        current_context_digest: state
            .current
            .as_ref()
            .filter(|value| value.not_before <= now && value.expires_at > now)
            .map(|value| value.context_digest.clone()),
        expected_participant_digest: Some(runtime.participant_digest.clone()),
        expected_needs_digest: Some(runtime.needs_digest.clone()),
        known_root_key_id: durable.trust.root_key_id,
        minimum_manifest_generation: i64::try_from(durable.trust.minimum_manifest_generation)
            .map_err(|_| TrellisClientError::Bootstrap("manifest generation overflow".into()))?,
        proof: serde_json::json!({
            "format": "trellis.session-proof.v1",
            "signature": "",
        }),
    };
    let request_value = serde_json::to_value(&request)?;
    let request_digest = session_proof_request_digest(&request_value)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    let input = SessionProofInput::authorization_context_refresh(
        AuthorizationContextRefreshSessionProofInput {
            request_id: request.request_id.clone(),
            issued_at: request.issued_at,
            session_id: request.session_id.clone(),
            session_key_id: auth.key_id(),
            current_context_digest: request.current_context_digest.clone(),
            expected_participant_digest: request.expected_participant_digest.clone(),
            expected_needs_digest: request.expected_needs_digest.clone(),
            known_root_key_id: request.known_root_key_id.clone(),
            minimum_manifest_generation: request.minimum_manifest_generation,
            request_digest,
        },
    )
    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    request.proof = serde_json::to_value(auth.sign_session_proof(&input)?)?;
    let response: RefreshResponse = serde_json::from_value(
        cache
            .http()
            .post_json("/auth/context/refresh", &request)
            .await?,
    )
    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    let response_received_at = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
            .as_millis(),
    )
    .map_err(|_| TrellisClientError::Bootstrap("context refresh time overflow".into()))?;
    let midpoint = request_started_at
        .checked_add(response_received_at)
        .and_then(|sum| sum.checked_div(2))
        .ok_or_else(|| TrellisClientError::Bootstrap("context refresh time overflow".into()))?;
    let server_clock_offset_ms = response.server_now - midpoint;
    if response.session.session_id != request.session_id
        || response.session.inbox_prefix.trim().is_empty()
        || response
            .nats
            .transports
            .native
            .as_ref()
            .is_none_or(|transport| transport.nats_servers.is_empty())
    {
        return Err(TrellisClientError::Bootstrap(
            "context refresh returned invalid session or native transport metadata".into(),
        ));
    }
    let native = response.nats.transports.native.ok_or_else(|| {
        TrellisClientError::Bootstrap("context refresh omitted native transport".into())
    })?;
    cache
        .install(
            AuthorizationInstallation {
                context: response.authorization_context,
                routing: AuthorizationRoutingMaterial {
                    bootstrap_jwt: response.nats.jwt,
                    bootstrap_jwt_expires_at: response.nats.jwt_expires_at,
                },
                runtime: AuthorizationRuntimeBinding {
                    session_id: response.session.session_id,
                    participant_id: runtime.participant_id,
                    participant_digest: runtime.participant_digest,
                    needs_digest: runtime.needs_digest,
                    inbox_prefix: response.session.inbox_prefix,
                    transports: AuthorizationRuntimeTransports {
                        native: AuthorizationNativeTransport {
                            nats_servers: native.nats_servers,
                        },
                    },
                },
                server_clock_offset_ms,
            },
            response.server_now.div_euclid(1_000),
        )
        .await?;
    Ok(cache.runtime_binding()? != previous_runtime)
}

/// Background own-context refresh task.
pub(crate) fn spawn_authorization_context_refresh_task(
    contexts: Arc<AuthorizationContextCache>,
    auth: Arc<SessionAuth>,
    nats: async_nats::Client,
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
            let previous = match contexts.runtime_binding() {
                Ok(runtime) => runtime,
                Err(error) => {
                    tracing::warn!(%error, "authorization context refresh stopped");
                    return;
                }
            };
            let previous_context_digest = match contexts.context_digest() {
                Ok(context) => context,
                Err(error) => {
                    tracing::warn!(%error, "authorization context refresh stopped");
                    return;
                }
            };
            match refresh(&contexts, &auth).await {
                Ok(_) => {
                    let refreshed = match contexts.runtime_binding() {
                        Ok(runtime) => runtime,
                        Err(error) => {
                            tracing::warn!(%error, "refreshed native runtime is invalid");
                            continue;
                        }
                    };
                    let credentials_changed = match contexts.context_digest() {
                        Ok(context) => previous_context_digest != context,
                        Err(error) => {
                            tracing::warn!(%error, "refreshed authorization context is invalid");
                            continue;
                        }
                    };
                    if let Err(error) = super::super::connection::apply_native_runtime_refresh(
                        &nats,
                        &previous,
                        &refreshed,
                        credentials_changed,
                    )
                    .await
                    {
                        tracing::warn!(%error, "native connection refresh will retry");
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
        assert!(is_terminal_refresh_error("user_inactive"));
        assert!(is_terminal_refresh_error("context_refresh_mismatch"));
        assert!(!is_terminal_refresh_error("authorization_pending"));
        assert!(!is_terminal_refresh_error("session_revoked later"));
    }
}
