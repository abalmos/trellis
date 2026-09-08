use std::sync::{Arc, Mutex, RwLock};

use trellis_protocol::{verify_authorization_context, AuthorizationContextPurpose};

use super::super::{SessionAuth, TrellisClientError};
use super::bootstrap_http::{persisted_signed_context, BootstrapHttp};
use super::types::{
    AuthorizationContextBundle, AuthorizationCredential, AuthorizationInstallation,
    AuthorizationRuntimeBinding, CachedAuthorizationState, CurrentContext,
};

/// Process-local context, route credential, and refresh scheduling for one connection.
/// Native credentials never require a writable authorization-state directory.
#[derive(Clone)]
pub struct AuthorizationContextCache {
    http: BootstrapHttp,
    pub(crate) credential: Arc<AuthorizationCredential>,
    pub(crate) connection_id: String,
    pub(crate) participant_id: String,
    pub(crate) name: Option<String>,
    pub(crate) session_key: String,
    state: Arc<RwLock<CachedAuthorizationState>>,
    refresh: Arc<tokio::sync::Mutex<()>>,
    refresh_requested: Arc<tokio::sync::Notify>,
    refresh_requested_digest: Arc<Mutex<Option<Option<String>>>>,
}

impl AuthorizationContextCache {
    pub(crate) fn new(
        trellis_url: &str,
        participant_id: String,
        connection_id: String,
        session_key: String,
        credential: AuthorizationCredential,
        name: Option<String>,
    ) -> Result<Self, TrellisClientError> {
        Ok(Self {
            http: BootstrapHttp::new(trellis_url)?,
            credential: Arc::new(credential),
            connection_id,
            participant_id,
            name,
            session_key,
            state: Arc::new(RwLock::new(CachedAuthorizationState::default())),
            refresh: Arc::new(tokio::sync::Mutex::new(())),
            refresh_requested: Arc::new(tokio::sync::Notify::new()),
            refresh_requested_digest: Arc::new(Mutex::new(None)),
        })
    }

    pub(crate) fn install(
        &self,
        installation: AuthorizationInstallation,
    ) -> Result<(), TrellisClientError> {
        let AuthorizationInstallation {
            context: bundle,
            routing,
            runtime,
            server_clock_offset_ms,
            authorization,
        } = installation;
        let now = system_now_millis()?
            .checked_add(server_clock_offset_ms)
            .ok_or_else(|| TrellisClientError::Bootstrap("context time overflow".into()))?
            .div_euclid(1000);
        let signed = persisted_signed_context(&bundle)?;
        let policy = bundle
            .policy
            .verification_policy(now)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let verified = verify_authorization_context(
            &bundle.issuer,
            &signed,
            &policy,
            AuthorizationContextPurpose::Live,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let context = &signed.unsigned;
        let credential_matches = match self.credential.as_ref() {
            AuthorizationCredential::Native { kind, identity } => {
                context.principal_kind == *kind
                    && context.identity_key_id.as_deref() == Some(identity.key_id().as_str())
                    && context.login_session_id.is_none()
            }
            AuthorizationCredential::User { login_session_id } => {
                context.principal_kind == trellis_protocol::AuthorizationPrincipalKind::User
                    && context.login_session_id.as_deref() == Some(login_session_id.as_str())
                    && context.identity_key_id.is_none()
            }
        };
        if !credential_matches
            || context.connection_id != self.connection_id
            || context.session_key != self.session_key
            || context.participant_id != self.participant_id
            || runtime.connection_id != context.connection_id
            || runtime.login_session_id != context.login_session_id
            || runtime.participant_id != context.participant_id
            || runtime.inbox_prefix != context.inbox_prefix
        {
            return Err(TrellisClientError::Bootstrap("bootstrap assignment does not match the credential, participant, and runtime connection".into()));
        }
        let native = runtime.transports.native.as_ref().ok_or_else(|| {
            TrellisClientError::AuthorizationUnavailable(
                "bootstrap did not offer a native NATS transport".into(),
            )
        })?;
        if native.nats_servers.is_empty()
            || routing.bootstrap_jwt.is_empty()
            || routing.bootstrap_jwt_expires_at <= now
        {
            return Err(TrellisClientError::Bootstrap(
                "bootstrap transport or route credential is unavailable".into(),
            ));
        }
        for endpoint in &native.nats_servers {
            endpoint
                .parse::<async_nats::ServerAddr>()
                .map_err(|error| {
                    TrellisClientError::Bootstrap(format!("invalid NATS endpoint: {error}"))
                })?;
        }
        let refresh_at = trellis_protocol::authorization_context_refresh_at(
            verified.context_digest(),
            context.issued_at,
            context.not_before,
            context.expires_at,
            bundle.policy.refresh_lead_seconds,
            bundle.policy.refresh_jitter_seconds,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let current = CurrentContext {
            context_digest: verified.context_digest().to_owned(),
            not_before: context.not_before,
            expires_at: context.expires_at,
            refresh_at,
            bundle,
        };
        *self
            .state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))? =
            CachedAuthorizationState {
                current: Some(current),
                runtime: Some(runtime),
                routing: Some(routing),
                server_clock_offset_ms,
                authorization,
            };
        Ok(())
    }

    /// Discard this connection's context and route without revoking its credential.
    pub fn clear(&self) -> Result<(), TrellisClientError> {
        let mut state = self
            .state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        state.current = None;
        state.routing = None;
        Ok(())
    }

    /// Return the digest of the currently valid context used for request proofs.
    pub fn context_digest(&self) -> Result<String, TrellisClientError> {
        let state = self
            .state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        let now = system_now_millis()?
            .checked_add(state.server_clock_offset_ms)
            .ok_or_else(|| TrellisClientError::Bootstrap("context time overflow".into()))?
            .div_euclid(1000);
        state
            .current
            .as_ref()
            .filter(|current| current.not_before <= now && current.expires_at > now)
            .map(|current| current.context_digest.clone())
            .ok_or_else(|| TrellisClientError::Bootstrap("authorization context expired".into()))
    }

    /// Return the current verified context and its online issuer entry.
    pub fn bundle(&self) -> Result<AuthorizationContextBundle, TrellisClientError> {
        self.state_snapshot()?
            .current
            .map(|current| current.bundle)
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization context unavailable".into())
            })
    }

    /// Renew through the credential's proof-bound native bootstrap or user refresh route.
    pub async fn refresh(&self, auth: &SessionAuth) -> Result<bool, TrellisClientError> {
        super::refresh::refresh(self, auth).await
    }

    pub(crate) async fn lock_refresh(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.refresh.lock().await
    }

    pub(crate) fn request_refresh(&self) {
        if let Ok(mut requested) = self.refresh_requested_digest.lock() {
            *requested = Some(self.context_digest().ok());
            self.refresh_requested.notify_one();
        }
    }

    pub(crate) async fn wait_refresh_request(&self) -> Option<String> {
        self.refresh_requested.notified().await;
        self.refresh_requested_digest
            .lock()
            .ok()
            .and_then(|mut requested| requested.take())
            .flatten()
    }

    pub(crate) fn refresh_delay(&self) -> Result<std::time::Duration, TrellisClientError> {
        let now = self.corrected_now_seconds()?;
        let state = self.state_snapshot()?;
        let Some(current) = state.current.as_ref() else {
            return Ok(std::time::Duration::from_secs(1));
        };
        let route_refresh = state.routing.as_ref().map_or(now, |route| {
            route
                .bootstrap_jwt_expires_at
                .saturating_sub(i64::from(current.bundle.policy.refresh_lead_seconds))
        });
        Ok(std::time::Duration::from_secs(
            u64::try_from(
                current
                    .refresh_at
                    .min(route_refresh)
                    .saturating_sub(now)
                    .max(5),
            )
            .map_err(|_| TrellisClientError::Bootstrap("context refresh delay overflow".into()))?,
        ))
    }

    pub(crate) fn routing_jwt(&self) -> Result<String, TrellisClientError> {
        let now = self.corrected_now_seconds()?;
        self.state_snapshot()?
            .routing
            .filter(|route| route.bootstrap_jwt_expires_at > now)
            .map(|route| route.bootstrap_jwt)
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization routing JWT expired".into())
            })
    }

    pub(crate) fn runtime_binding(
        &self,
    ) -> Result<AuthorizationRuntimeBinding, TrellisClientError> {
        self.state_snapshot()?.runtime.ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization runtime unavailable".into())
        })
    }

    pub(crate) fn corrected_now_seconds(&self) -> Result<i64, TrellisClientError> {
        self.corrected_now_millis().map(|now| now.div_euclid(1000))
    }

    pub(crate) fn corrected_now_millis(&self) -> Result<i64, TrellisClientError> {
        let offset = self
            .state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?
            .server_clock_offset_ms;
        system_now_millis()?
            .checked_add(offset)
            .ok_or_else(|| TrellisClientError::Bootstrap("context time overflow".into()))
    }

    pub(crate) fn state_snapshot(&self) -> Result<CachedAuthorizationState, TrellisClientError> {
        self.state
            .read()
            .map(|state| state.clone())
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))
    }

    pub(crate) fn http(&self) -> &BootstrapHttp {
        &self.http
    }
}

pub(super) fn system_now_millis() -> Result<i64, TrellisClientError> {
    i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
            .as_millis(),
    )
    .map_err(|_| TrellisClientError::Bootstrap("context time overflow".into()))
}
