//! Rust-owned NATS authorization callout.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_nats::HeaderMap;
use base64::engine::general_purpose::STANDARD;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use futures_util::StreamExt;
use nats_jwt_rs::authorization::{AuthRequest, AuthResponse};
use nats_jwt_rs::types::{Permission, Permissions, ResponsePermission};
use nats_jwt_rs::user::User;
use nats_jwt_rs::Claims;
use nkeys::{KeyPair, KeyPairType, XKey};
use serde::Deserialize;
use trellis_protocol::{
    verify_session_proof_v1, AuthorizationPrincipalKindV1, SessionProofInputV1,
    SessionProofPolicyV1, SessionProofV1,
};

use super::auth::{
    compile_transport_permissions, AuthConnectionPresence, AuthEphemeralRepository,
    AuthorizationStateError, AuthorizationStateService, ConnectReplayRecord,
    NatsAuthEphemeralRepository, ParticipantBindingRepository, SessionRepository,
    SqliteAuthorizationStore,
};
use crate::shutdown::StopHandle;
use crate::supervisor::RuntimeError;

const AUTH_CALLOUT_SUBJECT: &str = "$SYS.REQ.USER.AUTH";
const AUTH_CALLOUT_QUEUE: &str = "trellis";
const DISCONNECT_SUBJECT: &str = "$SYS.ACCOUNT.*.DISCONNECT";
const SERVER_XKEY_HEADER: &str = "Nats-Server-Xkey";
const CONNECT_TOKEN_FORMAT: &str = "trellis.nats-connect-token.v1";
const PROOF_MAXIMUM_AGE_MS: i64 = 30_000;
const PROOF_MAXIMUM_FUTURE_SKEW_MS: i64 = 5_000;
const DEFAULT_USER_JWT_TTL_MS: i64 = 300_000;
const MAX_CONCURRENT_REQUESTS: usize = 32;
const MAX_CONCURRENT_PER_IP: usize = 8;
const MAX_CONCURRENT_PER_SERVER: usize = 16;
const SHUTDOWN_DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NatsConnectToken {
    format: String,
    request_id: String,
    issued_at: i64,
    session_id: String,
    participant_digest: String,
    context_digest: String,
    proof: SessionProofV1,
}

#[derive(Debug, Deserialize)]
struct DisconnectEvent {
    client: Option<DisconnectedClient>,
}

#[derive(Debug, Deserialize)]
struct DisconnectedClient {
    user_nkey: Option<String>,
}

#[derive(Clone, Debug)]
struct CalloutKeys {
    auth_signing_key: Arc<KeyPair>,
    target_signing_key: Arc<KeyPair>,
    xkey: XKey,
    auth_account: String,
    target_account: String,
}

impl CalloutKeys {
    fn from_files(
        auth_signing_seed_file: &Path,
        target_signing_seed_file: &Path,
        xkey_seed_file: &Path,
        auth_user_creds_file: &Path,
        target_user_creds_file: &Path,
    ) -> Result<Self, AuthorizationStateError> {
        let auth_signing_key = account_key(auth_signing_seed_file, "auth issuer")?;
        let target_signing_key = account_key(target_signing_seed_file, "target issuer")?;
        let auth_user = user_claims(auth_user_creds_file, "auth user")?;
        let target_user = user_claims(target_user_creds_file, "target user")?;
        let auth_account = issuer_account(&auth_user);
        let target_account = issuer_account(&target_user);
        let xkey_seed = read_secret(xkey_seed_file, "auth-callout xkey")?;
        let xkey = XKey::from_seed(&xkey_seed).map_err(|error| {
            AuthorizationStateError::InvalidRecord(format!(
                "auth-callout xkey seed is invalid: {error}"
            ))
        })?;
        Ok(Self {
            auth_signing_key: Arc::new(auth_signing_key),
            target_signing_key: Arc::new(target_signing_key),
            xkey,
            auth_account,
            target_account,
        })
    }

    fn validate_bootstrap_jwt(
        &self,
        jwt: &str,
        session_nkey: &str,
        now_seconds: i64,
    ) -> Result<(), AuthorizationStateError> {
        let claims =
            Claims::<User>::decode(jwt).map_err(|_| denied("session bootstrap JWT is invalid"))?;
        if claims.iss != self.auth_signing_key.public_key()
            || claims.sub != session_nkey
            || claims.payload().issuer_account.as_deref() != Some(self.auth_account.as_str())
            || claims
                .exp
                .is_some_and(|expires_at| expires_at <= now_seconds)
        {
            return invalid_denial();
        }
        let permissions = &claims.payload().permissions.permissions;
        if permissions.publish.allow.is_empty()
            && permissions.publish.deny == [">"]
            && permissions.subscribe.allow.is_empty()
            && permissions.subscribe.deny == [">"]
            && permissions.resp.is_none()
        {
            Ok(())
        } else {
            invalid_denial()
        }
    }

    fn authorized_user_jwt(
        &self,
        user_nkey: &str,
        session_id: &str,
        principal_kind: AuthorizationPrincipalKindV1,
        permissions: super::auth::TransportPermissions,
        expires_at_seconds: i64,
    ) -> Result<String, AuthorizationStateError> {
        let mut claims = User::new_claims(format!("trellis-{session_id}"), user_nkey.to_owned());
        claims.exp = Some(expires_at_seconds);
        let payload = claims.payload_mut();
        payload.issuer_account = Some(self.target_account.clone());
        payload.permissions.permissions = Permissions {
            publish: Permission {
                allow: permissions.publish,
                deny: Vec::new(),
            },
            subscribe: Permission {
                allow: permissions.subscribe,
                deny: Vec::new(),
            },
            resp: Some(ResponsePermission {
                max_messages: if principal_kind == AuthorizationPrincipalKindV1::Service {
                    65_535
                } else {
                    1
                },
                ttl: Duration::ZERO,
            }),
        };
        claims.encode(&self.target_signing_key).map_err(|error| {
            AuthorizationStateError::Storage(format!("failed to sign NATS user JWT: {error}"))
        })
    }

    fn response(
        &self,
        request: &AuthRequest,
        user_jwt: Option<String>,
        denial_code: Option<&str>,
    ) -> Result<Vec<u8>, AuthorizationStateError> {
        let mut claims = AuthResponse::generic_claim(request.user_nkey.clone());
        claims.aud = Some(request.server.id.clone());
        let response = claims.payload_mut();
        response.issuer_account = Some(self.target_account.clone());
        match user_jwt {
            Some(jwt) => response.jwt = jwt,
            None => response.error = denial_code.unwrap_or("internal_error").to_owned(),
        }
        let encoded = claims.encode(&self.auth_signing_key).map_err(|error| {
            AuthorizationStateError::Storage(format!(
                "failed to sign NATS authorization response: {error}"
            ))
        })?;
        Ok(encoded.into_bytes())
    }
}

/// Runtime-owned NATS authorization callout processor.
pub(crate) struct AuthCallout {
    subscriber: async_nats::Subscriber,
    disconnect_subscriber: async_nats::Subscriber,
    processor: CalloutProcessor,
}

#[derive(Clone)]
struct CalloutProcessor {
    client: async_nats::Client,
    authorization: AuthorizationStateService<SqliteAuthorizationStore>,
    repository: SqliteAuthorizationStore,
    contexts: super::auth::AuthorizationContextService,
    replay: NatsAuthEphemeralRepository,
    keys: CalloutKeys,
    user_jwt_ttl_ms: i64,
    limiter: Arc<CalloutLimiter>,
}

#[derive(Debug, Default)]
struct CalloutLimiter {
    state: std::sync::Mutex<CalloutLimitState>,
}

#[derive(Debug, Default)]
struct CalloutLimitState {
    by_ip: BTreeMap<String, usize>,
    by_server: BTreeMap<String, usize>,
}

#[derive(Debug)]
struct CalloutPermit {
    limiter: Arc<CalloutLimiter>,
    ip: String,
    server: String,
}

impl CalloutLimiter {
    fn try_acquire(self: &Arc<Self>, ip: &str, server: &str) -> Option<CalloutPermit> {
        let mut state = self.state.lock().ok()?;
        if state.by_ip.get(ip).copied().unwrap_or_default() >= MAX_CONCURRENT_PER_IP
            || state.by_server.get(server).copied().unwrap_or_default() >= MAX_CONCURRENT_PER_SERVER
        {
            return None;
        }
        *state.by_ip.entry(ip.to_owned()).or_default() += 1;
        *state.by_server.entry(server.to_owned()).or_default() += 1;
        Some(CalloutPermit {
            limiter: Arc::clone(self),
            ip: ip.to_owned(),
            server: server.to_owned(),
        })
    }
}

impl Drop for CalloutPermit {
    fn drop(&mut self) {
        let Ok(mut state) = self.limiter.state.lock() else {
            return;
        };
        decrement(&mut state.by_ip, &self.ip);
        decrement(&mut state.by_server, &self.server);
    }
}

impl AuthCallout {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn start(
        client: async_nats::Client,
        system_client: async_nats::Client,
        authorization: AuthorizationStateService<SqliteAuthorizationStore>,
        repository: SqliteAuthorizationStore,
        replay: NatsAuthEphemeralRepository,
        contexts: super::auth::AuthorizationContextService,
        auth_signing_seed_file: &Path,
        target_signing_seed_file: &Path,
        xkey_seed_file: &Path,
        auth_user_creds_file: &Path,
        target_user_creds_file: &Path,
        user_jwt_ttl_ms: Option<u64>,
    ) -> Result<Self, AuthorizationStateError> {
        let keys = CalloutKeys::from_files(
            auth_signing_seed_file,
            target_signing_seed_file,
            xkey_seed_file,
            auth_user_creds_file,
            target_user_creds_file,
        )?;
        let subscriber = client
            .queue_subscribe(AUTH_CALLOUT_SUBJECT, AUTH_CALLOUT_QUEUE.to_owned())
            .await
            .map_err(|error| {
                AuthorizationStateError::Storage(format!(
                    "failed to subscribe to NATS authorization callout: {error}"
                ))
            })?;
        client.flush().await.map_err(|error| {
            AuthorizationStateError::Storage(format!(
                "failed to activate NATS authorization callout subscription: {error}"
            ))
        })?;
        let disconnect_subscriber =
            system_client
                .subscribe(DISCONNECT_SUBJECT)
                .await
                .map_err(|error| {
                    AuthorizationStateError::Storage(format!(
                        "failed to subscribe to NATS disconnect events: {error}"
                    ))
                })?;
        system_client.flush().await.map_err(|error| {
            AuthorizationStateError::Storage(format!(
                "failed to activate NATS disconnect subscription: {error}"
            ))
        })?;
        let user_jwt_ttl_ms = user_jwt_ttl_ms
            .unwrap_or(DEFAULT_USER_JWT_TTL_MS as u64)
            .try_into()
            .map_err(|_| {
                AuthorizationStateError::InvalidRecord(
                    "NATS user JWT TTL exceeds i64 milliseconds".to_owned(),
                )
            })?;
        if user_jwt_ttl_ms <= 0 {
            return invalid("NATS user JWT TTL must be positive");
        }
        Ok(Self {
            subscriber,
            disconnect_subscriber,
            processor: CalloutProcessor {
                client,
                authorization,
                repository,
                contexts,
                replay,
                keys,
                user_jwt_ttl_ms,
                limiter: Arc::new(CalloutLimiter::default()),
            },
        })
    }

    pub(crate) async fn run(mut self, stop: StopHandle) -> Result<(), RuntimeError> {
        let mut requests = tokio::task::JoinSet::new();
        loop {
            tokio::select! {
                () = stop.stopped() => break,
                result = requests.join_next(), if !requests.is_empty() => {
                    handle_request_completion(result)?;
                }
                message = self.subscriber.next(), if requests.len() < MAX_CONCURRENT_REQUESTS => {
                    let Some(message) = message else {
                        return Err(RuntimeError::Platform(
                            "NATS authorization callout subscription closed".to_owned(),
                        ));
                    };
                    let processor = self.processor.clone();
                    requests.spawn(async move { processor.process(message).await });
                }
                message = self.disconnect_subscriber.next() => {
                    let Some(message) = message else {
                        return Err(RuntimeError::Platform(
                            "NATS disconnect subscription closed".to_owned(),
                        ));
                    };
                    self.processor.process_disconnect(&message.payload).await
                        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
                }
            }
        }

        let drain = async {
            while let Some(result) = requests.join_next().await {
                handle_request_completion(Some(result))?;
            }
            Ok::<(), RuntimeError>(())
        };
        if tokio::time::timeout(SHUTDOWN_DRAIN_TIMEOUT, drain)
            .await
            .is_err()
        {
            requests.abort_all();
        }
        Ok(())
    }
}

impl CalloutProcessor {
    async fn process_disconnect(&self, payload: &[u8]) -> Result<(), AuthorizationStateError> {
        let Ok(event) = serde_json::from_slice::<DisconnectEvent>(payload) else {
            return Ok(());
        };
        let Some(user_nkey) = event.client.and_then(|client| client.user_nkey) else {
            return Ok(());
        };
        if user_nkey.is_empty() {
            return Ok(());
        }
        self.replay.delete_connection_presence(&user_nkey).await
    }

    async fn process(&self, message: async_nats::Message) -> Result<(), AuthorizationStateError> {
        let reply = message
            .reply
            .clone()
            .ok_or_else(|| denied("authorization request has no reply subject"))?;
        let server_xkey = server_xkey(message.headers.as_ref())?;
        let decrypted = self
            .keys
            .xkey
            .open(&message.payload, &server_xkey)
            .map_err(|_| denied("authorization request could not be decrypted"))?;
        let encoded_request = std::str::from_utf8(&decrypted)
            .map_err(|_| denied("authorization request JWT is not UTF-8"))?;
        let claims = Claims::<AuthRequest>::decode(encoded_request)
            .map_err(|_| denied("authorization request JWT is invalid"))?;
        let request = claims.payload();
        let now_seconds = now_millis()? / 1_000;
        if claims.iss != request.server.id
            || claims.sub != self.keys.auth_account
            || claims.aud.as_deref() != Some("nats-authorization-request")
            || claims
                .exp
                .is_none_or(|expires_at| expires_at <= now_seconds)
            || claims
                .nbf
                .is_some_and(|not_before| not_before > now_seconds)
            || claims.iat > now_seconds.saturating_add(5) as u64
            || request.server.xkey.as_deref() != Some(server_xkey.public_key().as_str())
            || request.client_info.id == 0
            || request.client_info.host.is_empty()
            || request.client_info.nonce.is_empty()
            || !is_nkey(&request.server.id, KeyPairType::Server)
            || !is_nkey(&request.user_nkey, KeyPairType::User)
        {
            return Err(denied("authorization request identity is invalid"));
        }

        let permit = self
            .limiter
            .try_acquire(&request.client_info.host, &request.server.id);
        let (user_jwt, denial_code) = match permit {
            None => (None, Some("rate_limited")),
            Some(_permit) => match self.authorize(request).await {
                Ok(jwt) => (Some(jwt), None),
                Err(error) => {
                    tracing::debug!(error = %error, "NATS connection authorization denied");
                    (None, Some(callout_denial_code(&error)))
                }
            },
        };
        let response = self.keys.response(request, user_jwt, denial_code)?;
        let encrypted = self
            .keys
            .xkey
            .seal(&response, &server_xkey)
            .map_err(|error| {
                AuthorizationStateError::Storage(format!(
                    "failed to encrypt NATS authorization response: {error}"
                ))
            })?;
        self.client
            .publish(reply, encrypted.into())
            .await
            .map_err(|error| {
                AuthorizationStateError::Storage(format!(
                    "failed to publish NATS authorization response: {error}"
                ))
            })?;
        self.client.flush().await.map_err(|error| {
            AuthorizationStateError::Storage(format!(
                "failed to flush NATS authorization response: {error}"
            ))
        })
    }

    async fn authorize(&self, request: &AuthRequest) -> Result<String, AuthorizationStateError> {
        let now = now_millis()?;
        let now_seconds = now / 1_000;
        let connect = &request.connect_opts;
        let token: NatsConnectToken = serde_json::from_str(
            connect
                .auth_token
                .as_deref()
                .ok_or_else(|| denied("NATS connect token is missing"))?,
        )
        .map_err(|_| denied("NATS connect token is invalid"))?;
        if token.format != CONNECT_TOKEN_FORMAT {
            return Err(denied("NATS connect token format is invalid"));
        }
        let session_nkey = connect
            .nkey
            .as_deref()
            .ok_or_else(|| denied("session NKey is missing"))?;
        self.keys.validate_bootstrap_jwt(
            connect
                .jwt
                .as_deref()
                .ok_or_else(|| denied("session bootstrap JWT is missing"))?,
            session_nkey,
            now_seconds,
        )?;
        verify_nats_nonce_signature(
            session_nkey,
            &request.client_info.nonce,
            connect
                .sig
                .as_deref()
                .ok_or_else(|| denied("NATS nonce signature is missing"))?,
        )?;

        let session = self
            .repository
            .get_session(&token.session_id)
            .await?
            .ok_or_else(|| denied("session does not exist"))?;
        if token.participant_digest != session.participant_artifact_digest {
            return Err(denied("participant binding does not match the session"));
        }
        let input = SessionProofInputV1::nats_connect_context(
            token.request_id,
            token.issued_at,
            &token.session_id,
            &session.session_key_id,
            &session.session_public_key,
            session_nkey,
            &token.participant_digest,
            &token.context_digest,
            &request.client_info.nonce,
        )
        .map_err(|error| denied(error.to_string()))?;
        let policy = SessionProofPolicyV1::new(PROOF_MAXIMUM_AGE_MS, PROOF_MAXIMUM_FUTURE_SKEW_MS)
            .map_err(|error| denied(error.to_string()))?;
        let verified = verify_session_proof_v1(
            &input,
            &token.proof,
            &session.session_public_key,
            now,
            policy,
        )
        .map_err(|error| denied(error.to_string()))?;

        let state = self
            .authorization
            .resolve_issuable_state(&token.session_id, now)
            .await?;
        let context = self
            .contexts
            .require_issuable_context(&state, &token.context_digest, now_seconds)
            .await?;
        let replay_key = verified.replay_key();
        let admitted = self
            .replay
            .admit_connect_replay(ConnectReplayRecord {
                format: "trellis.session-proof-replay.v1".to_owned(),
                purpose: replay_key.purpose().to_string(),
                signer_key_id: replay_key.signer_key_id().to_owned(),
                request_id: replay_key.request_id().to_owned(),
                transcript_digest: replay_key.transcript_digest().to_owned(),
                admitted_at: now,
                expires_at: now
                    .checked_add(policy.replay_retention_ms())
                    .ok_or_else(|| denied("connect replay expiry overflowed"))?,
                version: 1,
            })
            .await?;
        if !admitted {
            return Err(denied("NATS connect proof was already used"));
        }

        let binding = self
            .repository
            .get_participant_binding(&state.participant.id, &state.participant.artifact_digest)
            .await?
            .ok_or_else(|| denied("participant binding is unavailable"))?;
        let permissions = compile_transport_permissions(&state, &binding)?;
        tracing::debug!(subscribe = ?permissions.subscribe, "compiled NATS inbox permissions");
        let expires_at_ms = [
            state.session_expires_at,
            state.effective_authority_expires_at,
            state.delegation_expires_at,
            Some(context.expires_at.saturating_mul(1_000)),
            Some(
                now.checked_add(self.user_jwt_ttl_ms)
                    .ok_or_else(|| denied("NATS user JWT expiry overflowed"))?,
            ),
        ]
        .into_iter()
        .flatten()
        .min()
        .ok_or_else(|| denied("NATS user JWT has no expiry bound"))?;
        if expires_at_ms <= now {
            return Err(denied("NATS user JWT expiry has elapsed"));
        }
        let expires_at_seconds = expires_at_ms / 1_000;
        if expires_at_seconds <= now_seconds {
            return Err(denied("NATS user JWT expiry is below one second"));
        }
        self.repository
            .touch_session(&token.session_id, now)
            .await?;
        let jwt = self.keys.authorized_user_jwt(
            &request.user_nkey,
            &token.session_id,
            state.principal.kind,
            permissions,
            expires_at_seconds,
        )?;
        let client_id = request.client_info.id.to_string();
        let connection_id = trellis_protocol::digest_json(&serde_json::json!({
            "serverId": request.server.id,
            "clientId": client_id,
            "userNkey": request.user_nkey,
        }))
        .map_err(|error| denied(error.to_string()))?;
        self.replay
            .put_connection_presence(AuthConnectionPresence {
                format: "trellis.auth-connection-presence.v1".to_owned(),
                connection_id: connection_id.clone(),
                session_id: token.session_id.clone(),
                context_id: context.context_id.clone(),
                context_digest: context.context_digest.clone(),
                server_id: request.server.id.clone(),
                client_id,
                user_nkey: request.user_nkey.clone(),
                remote_address: Some(request.client_info.host.clone()),
                connected_at: now,
                last_seen_at: now,
                version: 1,
            })
            .await?;
        let final_state = self
            .authorization
            .resolve_issuable_state(&token.session_id, now)
            .await;
        let final_check = match final_state {
            Ok(final_state) if final_state == state => self
                .contexts
                .require_issuable_context(&final_state, &token.context_digest, now_seconds)
                .await
                .map(|_| ()),
            Ok(_) => Err(AuthorizationStateError::ContextSnapshotChanged),
            Err(error) => Err(error),
        };
        if let Err(error) = final_check {
            self.replay
                .delete_connection_presence(&request.user_nkey)
                .await?;
            return Err(error);
        }
        tracing::debug!(
            session_id = %token.session_id,
            context_digest = %token.context_digest,
            "NATS authorization callout admitted"
        );
        Ok(jwt)
    }
}

fn handle_request_completion(
    result: Option<Result<Result<(), AuthorizationStateError>, tokio::task::JoinError>>,
) -> Result<(), RuntimeError> {
    match result {
        Some(Ok(Ok(()))) => Ok(()),
        Some(Ok(Err(error))) => {
            tracing::warn!(error = %error, "NATS authorization callout request denied");
            Ok(())
        }
        Some(Err(error)) => Err(RuntimeError::Platform(format!(
            "NATS authorization callout task failed: {error}"
        ))),
        None => Err(RuntimeError::Platform(
            "NATS authorization callout task set closed unexpectedly".to_owned(),
        )),
    }
}

fn callout_denial_code(error: &AuthorizationStateError) -> &'static str {
    match error {
        AuthorizationStateError::SessionMissing => "session_not_found",
        AuthorizationStateError::SessionExpired => "session_expired",
        AuthorizationStateError::SessionRevoked => "session_revoked",
        AuthorizationStateError::PrincipalMissing | AuthorizationStateError::PrincipalInactive => {
            "principal_inactive"
        }
        AuthorizationStateError::ParticipantMissing
        | AuthorizationStateError::ParticipantDigestMismatch
        | AuthorizationStateError::NeedsDigestMismatch => "participant_changed",
        AuthorizationStateError::AuthorityPending => "authority_pending",
        AuthorizationStateError::AuthorityMissing
        | AuthorizationStateError::AuthorityRejected
        | AuthorizationStateError::AuthorityRevoked
        | AuthorizationStateError::AuthorityStale
        | AuthorizationStateError::AuthorityExpired
        | AuthorizationStateError::RequiredDependencyUnavailable(_)
        | AuthorizationStateError::RequiredResourceUnavailable(_)
        | AuthorizationStateError::MaterializationStale
        | AuthorizationStateError::ContextLifetimeUnavailable
        | AuthorizationStateError::ContextSnapshotChanged => "authority_unavailable",
        AuthorizationStateError::DeploymentInactive => "deployment_inactive",
        AuthorizationStateError::InstanceInactive => "instance_inactive",
        AuthorizationStateError::DeviceInactive => "device_inactive",
        AuthorizationStateError::ActivationMissing => "delegation_missing",
        AuthorizationStateError::DelegationExpired => "delegation_expired",
        AuthorizationStateError::StorageConflict => "replay",
        AuthorizationStateError::Storage(_) => "internal_error",
        AuthorizationStateError::InvalidRecord(_) => "invalid_auth_token",
    }
}

fn decrement(counts: &mut BTreeMap<String, usize>, key: &str) {
    if let Some(count) = counts.get_mut(key) {
        *count -= 1;
        if *count == 0 {
            counts.remove(key);
        }
    }
}

fn server_xkey(headers: Option<&HeaderMap>) -> Result<XKey, AuthorizationStateError> {
    let encoded = headers
        .and_then(|headers| headers.get(SERVER_XKEY_HEADER))
        .map(ToString::to_string)
        .ok_or_else(|| denied("authorization request has no server XKey"))?;
    XKey::from_public_key(&encoded).map_err(|_| denied("authorization request XKey is invalid"))
}

fn verify_nats_nonce_signature(
    session_nkey: &str,
    nonce: &str,
    encoded_signature: &str,
) -> Result<(), AuthorizationStateError> {
    let key =
        KeyPair::from_public_key(session_nkey).map_err(|_| denied("session NKey is invalid"))?;
    if key.key_pair_type() != KeyPairType::User {
        return Err(denied("session NKey is not a user key"));
    }
    let signature = STANDARD
        .decode(encoded_signature)
        .or_else(|_| URL_SAFE_NO_PAD.decode(encoded_signature))
        .map_err(|_| denied("NATS nonce signature is invalid"))?;
    key.verify(nonce.as_bytes(), &signature)
        .map_err(|_| denied("NATS nonce signature is invalid"))
}

fn is_nkey(value: &str, expected: KeyPairType) -> bool {
    KeyPair::from_public_key(value).is_ok_and(|key| key.key_pair_type() == expected)
}

fn account_key(path: &Path, label: &str) -> Result<KeyPair, AuthorizationStateError> {
    let seed = read_secret(path, label)?;
    let key = KeyPair::from_seed(&seed).map_err(|error| {
        AuthorizationStateError::InvalidRecord(format!("{label} signing seed is invalid: {error}"))
    })?;
    if key.key_pair_type() != KeyPairType::Account {
        return invalid(format!("{label} signing seed must be an account NKey"));
    }
    Ok(key)
}

fn user_claims(path: &Path, label: &str) -> Result<Claims<User>, AuthorizationStateError> {
    let credentials = fs::read_to_string(path).map_err(|error| {
        AuthorizationStateError::Storage(format!("failed to read {label} creds: {error}"))
    })?;
    let jwt = credentials
        .lines()
        .skip_while(|line| *line != "-----BEGIN NATS USER JWT-----")
        .skip(1)
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(format!("{label} creds contain no user JWT"))
        })?;
    Claims::<User>::decode(jwt).map_err(|error| {
        AuthorizationStateError::InvalidRecord(format!("{label} JWT is invalid: {error}"))
    })
}

fn issuer_account(claims: &Claims<User>) -> String {
    claims
        .payload()
        .issuer_account
        .clone()
        .unwrap_or_else(|| claims.iss.clone())
}

fn read_secret(path: &Path, label: &str) -> Result<String, AuthorizationStateError> {
    fs::read_to_string(path)
        .map(|value| value.trim().to_owned())
        .map_err(|error| {
            AuthorizationStateError::Storage(format!("failed to read {label} seed: {error}"))
        })
}

fn now_millis() -> Result<i64, AuthorizationStateError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?
        .as_millis()
        .try_into()
        .map_err(|_| AuthorizationStateError::Storage("current time exceeds i64".to_owned()))
}

fn denied(message: impl Into<String>) -> AuthorizationStateError {
    AuthorizationStateError::InvalidRecord(message.into())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, AuthorizationStateError> {
    Err(AuthorizationStateError::InvalidRecord(message.into()))
}

fn invalid_denial<T>() -> Result<T, AuthorizationStateError> {
    Err(denied(
        "session bootstrap JWT is not an exact deny-all credential",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::auth::TransportPermissions;

    #[test]
    fn bootstrap_and_issued_jwts_preserve_the_account_boundary(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let auth_signing_key = Arc::new(KeyPair::new_account());
        let target_signing_key = Arc::new(KeyPair::new_account());
        let auth_account = KeyPair::new_account().public_key();
        let target_account = KeyPair::new_account().public_key();
        let keys = CalloutKeys {
            auth_signing_key: Arc::clone(&auth_signing_key),
            target_signing_key: Arc::clone(&target_signing_key),
            xkey: XKey::new(),
            auth_account: auth_account.clone(),
            target_account: target_account.clone(),
        };
        let session = KeyPair::new_user();
        let session_nkey = session.public_key();
        let deny = Permission {
            allow: Vec::new(),
            deny: vec![">".to_owned()],
        };
        let mut bootstrap = User::new_claims("session".to_owned(), session_nkey.clone());
        bootstrap.payload_mut().issuer_account = Some(auth_account);
        bootstrap.payload_mut().permissions.permissions = Permissions {
            publish: deny.clone(),
            subscribe: deny,
            resp: None,
        };
        let bootstrap = bootstrap.encode(&auth_signing_key)?;
        keys.validate_bootstrap_jwt(&bootstrap, &session_nkey, 1)?;

        let nonce = "server-nonce";
        let signature = session.sign(nonce.as_bytes())?;
        verify_nats_nonce_signature(&session_nkey, nonce, &STANDARD.encode(&signature))?;
        verify_nats_nonce_signature(&session_nkey, nonce, &URL_SAFE_NO_PAD.encode(&signature))?;

        let issued = keys.authorized_user_jwt(
            &KeyPair::new_user().public_key(),
            "ses_01",
            AuthorizationPrincipalKindV1::Service,
            TransportPermissions {
                publish: vec!["rpc.v1.Example".to_owned()],
                subscribe: vec!["_INBOX.example.>".to_owned()],
            },
            200,
        )?;
        let payload = issued
            .split('.')
            .nth(1)
            .ok_or("issued JWT has no payload")?;
        let claims: serde_json::Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload)?)?;
        assert_eq!(claims["iss"], target_signing_key.public_key());
        assert_eq!(claims["exp"], 200);
        assert_eq!(claims["nats"]["issuer_account"], target_account);
        assert_eq!(
            claims["nats"]["pub"]["allow"],
            serde_json::json!(["rpc.v1.Example"])
        );
        assert_eq!(
            claims["nats"]["sub"]["allow"],
            serde_json::json!(["_INBOX.example.>"])
        );
        assert_eq!(claims["nats"]["resp"]["max"], 65_535);
        Ok(())
    }

    #[test]
    fn callout_limiter_bounds_each_source() {
        let limiter = Arc::new(CalloutLimiter::default());
        let permits: Vec<_> = (0..MAX_CONCURRENT_PER_IP)
            .map(|_| limiter.try_acquire("127.0.0.1", "server-1").unwrap())
            .collect();
        assert!(limiter.try_acquire("127.0.0.1", "server-2").is_none());
        drop(permits);
        assert!(limiter.try_acquire("127.0.0.1", "server-2").is_some());
    }

    #[test]
    fn callout_denials_never_expose_internal_causes() {
        let secret = "postgres://admin:secret@internal/auth";
        let code = callout_denial_code(&AuthorizationStateError::Storage(secret.to_owned()));
        assert_eq!(code, "internal_error");
        assert!(!code.contains(secret));
    }
}
