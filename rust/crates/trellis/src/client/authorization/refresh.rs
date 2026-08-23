use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use trellis_protocol::{
    session_proof_request_digest_v1, AuthorizationContextRefreshSessionProofInputV1,
    SessionProofInputV1,
};

use super::super::{proof::new_request_id, SessionAuth, TrellisClientError};
use super::own_context::AuthorizationContextCache;
use super::types::{AuthorizationContextBundle, AuthorizationRoutingMaterial};

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
    authorization_context: AuthorizationContextBundle,
    bootstrap_jwt: String,
    bootstrap_jwt_expires_at: i64,
}

/// Refresh the current context through the proof-bound auth endpoint.
///
/// This is the only HTTP request performed by a connected own-context cache and
/// is reserved for credential/context recovery.
pub(crate) async fn refresh(
    cache: &AuthorizationContextCache,
    auth: &SessionAuth,
) -> Result<(), TrellisClientError> {
    let observed_digest = cache.context_digest().ok();
    let _refresh = cache.lock_refresh().await;
    if observed_digest != cache.context_digest().ok() {
        return Ok(());
    }
    let now = cache.corrected_now_seconds()?;
    let state = cache.state_snapshot()?;
    let session = state
        .session
        .ok_or_else(|| TrellisClientError::Bootstrap("authorization session unavailable".into()))?;
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
        session_id: session.session_id,
        session_nkey: auth.nkey_pair()?.public_key(),
        current_context_digest: state
            .current
            .as_ref()
            .filter(|value| value.not_before <= now && value.expires_at > now)
            .map(|value| value.context_digest.clone()),
        expected_participant_digest: Some(session.participant_digest),
        expected_needs_digest: Some(session.needs_digest),
        known_root_key_id: durable.trust.root_key_id,
        minimum_manifest_generation: i64::try_from(durable.trust.minimum_manifest_generation)
            .map_err(|_| TrellisClientError::Bootstrap("manifest generation overflow".into()))?,
        proof: serde_json::json!({
            "format": "trellis.session-proof.v1",
            "signature": "",
        }),
    };
    let request_value = serde_json::to_value(&request)?;
    let request_digest = session_proof_request_digest_v1(&request_value)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    let input = SessionProofInputV1::authorization_context_refresh(
        AuthorizationContextRefreshSessionProofInputV1 {
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
    cache.set_server_clock_offset_ms(response.server_now - midpoint);
    cache
        .install(
            response.authorization_context,
            AuthorizationRoutingMaterial {
                bootstrap_jwt: response.bootstrap_jwt,
                bootstrap_jwt_expires_at: response.bootstrap_jwt_expires_at,
            },
            response.server_now.div_euclid(1_000),
        )
        .await
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
            match refresh(&contexts, &auth).await {
                Ok(()) => {}
                Err(TrellisClientError::BootstrapHttp { status, .. })
                    if matches!(status, 401 | 403 | 409) =>
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
