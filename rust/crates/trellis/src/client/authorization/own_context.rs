use std::sync::{Arc, Mutex, RwLock};

use trellis_protocol::{verify_authorization_context, AuthorizationContextPurpose};

use super::super::{SessionAuth, TrellisClientError};
use super::bootstrap_http::{persisted_signed_context, BootstrapHttp};
use super::types::{
    AuthorizationContextBundle, AuthorizationCredential, AuthorizationInstallation,
    AuthorizationRuntimeBinding, CachedAuthorizationState, CurrentContext,
};

#[derive(Clone, Debug)]
pub(crate) struct AuthorizationRefreshRequest {
    pub(crate) context_digest: Option<String>,
    pub(crate) refresh_credential: bool,
}

#[derive(Clone, Debug)]
struct PreparedInstallation {
    current: CurrentContext,
    runtime: AuthorizationRuntimeBinding,
    routing: super::types::AuthorizationRoutingMaterial,
    api_bindings: std::collections::BTreeMap<String, super::types::AuthorizationApiBinding>,
    server_clock_offset_ms: i64,
    authorization: Option<serde_json::Value>,
    availability: crate::generated::AvailabilitySnapshot,
}

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
    availability: tokio::sync::watch::Sender<crate::generated::AvailabilitySnapshot>,
    refresh: Arc<tokio::sync::Mutex<()>>,
    refresh_requested: Arc<tokio::sync::Notify>,
    refresh_request: Arc<Mutex<Option<AuthorizationRefreshRequest>>>,
    candidate: Arc<RwLock<Option<PreparedInstallation>>>,
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
        let (availability, _) = tokio::sync::watch::channel(Default::default());
        Ok(Self {
            http: BootstrapHttp::new(trellis_url, false)?,
            credential: Arc::new(credential),
            connection_id,
            participant_id,
            name,
            session_key,
            state: Arc::new(RwLock::new(CachedAuthorizationState::default())),
            availability,
            refresh: Arc::new(tokio::sync::Mutex::new(())),
            refresh_requested: Arc::new(tokio::sync::Notify::new()),
            refresh_request: Arc::new(Mutex::new(None)),
            candidate: Arc::new(RwLock::new(None)),
        })
    }

    fn replace_installation(
        &self,
        installation: AuthorizationInstallation,
        promote: bool,
    ) -> Result<(), TrellisClientError> {
        let AuthorizationInstallation {
            context: bundle,
            routing,
            runtime,
            api_bindings,
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
            AuthorizationCredential::Native { kind, identity, .. } => {
                context.principal_kind == *kind
                    && context.identity_key_id.as_deref() == Some(identity.key_id().as_str())
                    && context.login_session_id.is_none()
            }
            AuthorizationCredential::User {
                login_session_id, ..
            } => {
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
        let resources = authorization
            .as_ref()
            .and_then(|value| value.get("resourceRuntime"))
            .cloned()
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
        let availability = crate::generated::AvailabilitySnapshot::replacing(
            context.grants.permissions().to_vec(),
            resources,
            &self.availability.borrow(),
        );
        let current = CurrentContext {
            context_digest: verified.context_digest().to_owned(),
            not_before: context.not_before,
            expires_at: context.expires_at,
            refresh_at,
            bundle,
        };
        if !promote {
            // A planned refresh keeps the active installation usable until the
            // candidate is promoted after transport reauthorization.
            let mut candidate = self.candidate.write().map_err(|_| {
                TrellisClientError::Bootstrap("context candidate lock poisoned".into())
            })?;
            *candidate = Some(PreparedInstallation {
                current,
                runtime,
                routing,
                api_bindings,
                server_clock_offset_ms,
                authorization,
                availability,
            });
            tracing::info!(
                context_digest = verified.context_digest(),
                "prepared verified authorization candidate"
            );
            return Ok(());
        }
        let mut state = self
            .state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        *state = CachedAuthorizationState {
            current: Some(current),
            runtime: Some(runtime),
            routing: Some(routing),
            api_bindings,
            server_clock_offset_ms,
            authorization,
        };
        drop(state);
        *self.candidate.write().map_err(|_| {
            TrellisClientError::Bootstrap("context candidate lock poisoned".into())
        })? = None;
        self.availability.send_replace(availability);
        tracing::info!(
            context_digest = verified.context_digest(),
            promoted = true,
            "installed verified authorization context"
        );
        Ok(())
    }

    pub(crate) fn install_initial(
        &self,
        installation: AuthorizationInstallation,
    ) -> Result<(), TrellisClientError> {
        self.replace_installation(installation, true)
    }

    pub(crate) fn prepare(
        &self,
        installation: AuthorizationInstallation,
    ) -> Result<String, TrellisClientError> {
        self.replace_installation(installation, false)?;
        self.candidate_digest()
    }

    pub(crate) fn candidate_digest(&self) -> Result<String, TrellisClientError> {
        self.candidate
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context candidate lock poisoned".into()))?
            .as_ref()
            .map(|candidate| candidate.current.context_digest.clone())
            .ok_or_else(|| {
                TrellisClientError::AuthorizationUnavailable(
                    "authorization candidate is unavailable".into(),
                )
            })
    }

    /// Return the runtime, digest, and route JWT that the next transport
    /// reauthorization must present: the candidate when one is prepared.
    pub(crate) fn applied_transport(
        &self,
    ) -> Result<(AuthorizationRuntimeBinding, String, String), TrellisClientError> {
        if let Some(candidate) = self
            .candidate
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context candidate lock poisoned".into()))?
            .as_ref()
        {
            return Ok((
                candidate.runtime.clone(),
                candidate.current.context_digest.clone(),
                candidate.routing.bootstrap_jwt.clone(),
            ));
        }
        let state = self.state_snapshot()?;
        Ok((
            state.runtime.ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization runtime unavailable".into())
            })?,
            state
                .current
                .ok_or_else(|| {
                    TrellisClientError::Bootstrap("authorization context unavailable".into())
                })?
                .context_digest,
            state
                .routing
                .ok_or_else(|| {
                    TrellisClientError::Bootstrap("authorization routing JWT unavailable".into())
                })?
                .bootstrap_jwt,
        ))
    }

    pub(crate) fn promote(&self, expected_digest: &str) -> Result<(), TrellisClientError> {
        let mut state = self
            .state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        let mut candidate = self
            .candidate
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("context candidate lock poisoned".into()))?;
        let prepared = candidate.take().ok_or_else(|| {
            TrellisClientError::AuthorizationUnavailable(
                "authorization candidate is unavailable".into(),
            )
        })?;
        if prepared.current.context_digest != expected_digest || state.current.is_none() {
            return Err(TrellisClientError::AuthorizationUnavailable(
                "authorization candidate changed before promotion".into(),
            ));
        }
        *state = CachedAuthorizationState {
            current: Some(prepared.current),
            runtime: Some(prepared.runtime),
            routing: Some(prepared.routing),
            api_bindings: prepared.api_bindings,
            server_clock_offset_ms: prepared.server_clock_offset_ms,
            authorization: prepared.authorization,
        };
        drop(candidate);
        drop(state);
        self.availability.send_replace(prepared.availability);
        tracing::info!(
            context_digest = self.retained_context_digest().ok(),
            "promoted authorization installation"
        );
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
        state.api_bindings.clear();
        drop(state);
        *self.candidate.write().map_err(|_| {
            TrellisClientError::Bootstrap("context candidate lock poisoned".into())
        })? = None;
        let suspended =
            crate::generated::AvailabilitySnapshot::suspended(&self.availability.borrow().clone());
        self.availability.send_replace(suspended);
        Ok(())
    }

    pub(crate) fn suspend(&self) {
        let suspended =
            crate::generated::AvailabilitySnapshot::suspended(&self.availability.borrow().clone());
        self.availability.send_replace(suspended);
        tracing::info!(
            context_digest = self.context_digest().ok(),
            "suspended authorization installation"
        );
    }

    /// Return the digest of the currently valid context used for request proofs.
    pub fn context_digest(&self) -> Result<String, TrellisClientError> {
        if !self.availability.borrow().is_usable() {
            return Err(TrellisClientError::AuthorizationUnavailable(
                "authorization installation is suspended".into(),
            ));
        }
        self.retained_context_digest()
    }

    pub(crate) fn retained_context_digest(&self) -> Result<String, TrellisClientError> {
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
        super::refresh::refresh(self, auth, true)
            .await
            .map(|(_, changed)| changed)
    }

    pub(crate) async fn prepare_refresh(
        &self,
        auth: &SessionAuth,
    ) -> Result<(String, bool), TrellisClientError> {
        super::refresh::refresh(self, auth, false).await
    }

    pub(crate) async fn lock_refresh(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.refresh.lock().await
    }

    pub(crate) fn request_refresh(&self) {
        self.request_reconciliation(true);
    }

    pub(crate) fn request_coverage_reconciliation(&self) {
        self.request_reconciliation(false);
    }

    fn request_reconciliation(&self, refresh_credential: bool) {
        if let Ok(mut requested) = self.refresh_request.lock() {
            let digest = self.retained_context_digest().ok();
            match requested.as_mut() {
                Some(request) => request.refresh_credential |= refresh_credential,
                None => {
                    *requested = Some(AuthorizationRefreshRequest {
                        context_digest: digest,
                        refresh_credential,
                    });
                }
            }
            self.refresh_requested.notify_one();
        }
    }

    pub(crate) async fn wait_refresh_request(&self) -> AuthorizationRefreshRequest {
        self.refresh_requested.notified().await;
        self.refresh_request
            .lock()
            .ok()
            .and_then(|mut requested| requested.take())
            .unwrap_or(AuthorizationRefreshRequest {
                context_digest: self.retained_context_digest().ok(),
                refresh_credential: false,
            })
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

    pub(crate) fn transport_credentials(&self) -> Result<(String, String), TrellisClientError> {
        if let Some(candidate) = self
            .candidate
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context candidate lock poisoned".into()))?
            .as_ref()
        {
            if candidate.routing.bootstrap_jwt_expires_at <= self.corrected_now_seconds()? {
                return Err(TrellisClientError::Bootstrap(
                    "authorization routing JWT expired".into(),
                ));
            }
            return Ok((
                candidate.routing.bootstrap_jwt.clone(),
                candidate.current.context_digest.clone(),
            ));
        }
        Ok((self.routing_jwt()?, self.retained_context_digest()?))
    }

    pub(crate) fn runtime_binding(
        &self,
    ) -> Result<AuthorizationRuntimeBinding, TrellisClientError> {
        self.state_snapshot()?.runtime.ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization runtime unavailable".into())
        })
    }

    pub(crate) fn provider_deployment_id(
        &self,
        api_id: &str,
    ) -> Result<String, TrellisClientError> {
        self.state_snapshot()?
            .api_bindings
            .get(api_id)
            .map(|binding| binding.provider_deployment_id.clone())
            .ok_or_else(|| {
                TrellisClientError::AuthorizationUnavailable(format!(
                    "bootstrap did not bind API '{api_id}' to a provider deployment"
                ))
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

    pub(crate) fn availability(&self) -> crate::generated::AvailabilitySnapshot {
        self.availability.borrow().clone()
    }

    pub(crate) fn watch_availability(
        &self,
    ) -> tokio::sync::watch::Receiver<crate::generated::AvailabilitySnapshot> {
        self.availability.subscribe()
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
