use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc, Mutex, RwLock,
};

use serde_json::Value;
use trellis_protocol::{
    parse_issuer_manifest, verify_authorization_context, verify_issuer_manifest,
    AuthorizationTrustRoot, AuthorizationVerificationPolicy,
};

use super::super::TrellisClientError;
use super::bootstrap_http::{persisted_signed_context, BootstrapHttp};
use super::types::{
    AuthorizationClientState, AuthorizationClientTrustState, AuthorizationContextBundle,
    AuthorizationContextStore, AuthorizationInstallation, AuthorizationRuntimeBinding,
    CachedAuthorizationState, CurrentContext, AUTHORIZATION_CLIENT_STATE_FORMAT_,
};
#[cfg(test)]
use super::types::{
    AuthorizationNativeTransport, AuthorizationRoutingMaterial, AuthorizationRuntimeTransports,
};

/// Verified process-local own authorization context used by reconnect callbacks.
///
/// The cache owns only the calling runtime's own session context: installation,
/// route JWT, persistence, and refresh scheduling. Provider-side resolution of
/// arbitrary caller contexts lives in the internal provider cache.
#[derive(Clone)]
pub struct AuthorizationContextCache {
    http: BootstrapHttp,
    binding: String,
    store: Arc<dyn AuthorizationContextStore>,
    state: Arc<RwLock<CachedAuthorizationState>>,
    clock_offset_ms: Arc<AtomicI64>,
    update: Arc<Mutex<()>>,
    refresh: Arc<tokio::sync::Mutex<()>>,
    refresh_requested: Arc<tokio::sync::Notify>,
    refresh_requested_digest: Arc<Mutex<Option<Option<String>>>>,
}

impl AuthorizationContextCache {
    /// Create a cache using explicit caller-owned persistence.
    pub fn new(
        trellis_url: &str,
        binding: impl Into<String>,
        store: Arc<dyn AuthorizationContextStore>,
    ) -> Result<Self, TrellisClientError> {
        let binding = binding.into();
        if binding.trim().is_empty() {
            return Err(TrellisClientError::Bootstrap(
                "authorization context storage binding is empty".into(),
            ));
        }
        Ok(Self {
            http: BootstrapHttp::new(trellis_url)?,
            binding,
            store,
            state: Arc::new(RwLock::new(CachedAuthorizationState::default())),
            clock_offset_ms: Arc::new(AtomicI64::new(0)),
            update: Arc::new(Mutex::new(())),
            refresh: Arc::new(tokio::sync::Mutex::new(())),
            refresh_requested: Arc::new(tokio::sync::Notify::new()),
            refresh_requested_digest: Arc::new(Mutex::new(None)),
        })
    }

    /// Create an explicitly ephemeral cache for tests or short-lived clients.
    pub fn ephemeral(
        trellis_url: &str,
        binding: impl Into<String>,
    ) -> Result<Self, TrellisClientError> {
        Self::new(
            trellis_url,
            binding,
            Arc::new(super::persistence::MemoryAuthorizationContextStore::default()),
        )
    }

    /// Restore and verify the atomically persisted current context, if present.
    pub async fn restore(&self, now_unix_seconds: i64) -> Result<bool, TrellisClientError> {
        let _update = self.lock_update()?;
        let Some(state) = self.store.load()? else {
            return Ok(false);
        };
        if state.binding != self.binding {
            return Err(TrellisClientError::Bootstrap(
                "authorization context storage belongs to another identity".into(),
            ));
        }
        self.clock_offset_ms
            .store(state.server_clock_offset_ms, Ordering::Relaxed);
        let now_unix_seconds = now_unix_seconds
            .checked_add(state.server_clock_offset_ms.div_euclid(1_000))
            .ok_or_else(|| TrellisClientError::Bootstrap("context time overflow".into()))?;
        if let Some(bundle) = state.context.as_ref() {
            let context = persisted_signed_context(bundle)?;
            if context.unsigned.issuer_manifest_generation < state.trust.minimum_manifest_generation
            {
                self.store.clear_context()?;
                self.state
                    .write()
                    .map_err(|_| {
                        TrellisClientError::Bootstrap("context cache lock poisoned".into())
                    })?
                    .runtime = Some(state.runtime);
                return Ok(false);
            }
        }
        let installation = match (state.context, state.routing) {
            (Some(context), Some(routing)) => AuthorizationInstallation {
                context,
                routing,
                runtime: state.runtime,
                server_clock_offset_ms: state.server_clock_offset_ms,
            },
            (None, None) => {
                self.state
                    .write()
                    .map_err(|_| {
                        TrellisClientError::Bootstrap("context cache lock poisoned".into())
                    })?
                    .runtime = Some(state.runtime);
                return Ok(false);
            }
            _ => {
                return Err(TrellisClientError::Bootstrap(
                    "persisted context and routing material are not atomic".into(),
                ));
            }
        };
        self.install_recoverable_locked(installation, now_unix_seconds)
    }

    pub(crate) async fn install_recoverable(
        &self,
        installation: AuthorizationInstallation,
        now_unix_seconds: i64,
    ) -> Result<bool, TrellisClientError> {
        let _update = self.lock_update()?;
        self.install_recoverable_locked(installation, now_unix_seconds)
    }

    fn install_recoverable_locked(
        &self,
        installation: AuthorizationInstallation,
        now_unix_seconds: i64,
    ) -> Result<bool, TrellisClientError> {
        let signed = persisted_signed_context(&installation.context)?;
        let verification_now = if signed.unsigned.expires_at <= now_unix_seconds {
            signed
                .unsigned
                .expires_at
                .saturating_sub(1)
                .max(signed.unsigned.not_before)
        } else {
            now_unix_seconds
        };
        let routing_expires_at = installation.routing.bootstrap_jwt_expires_at;
        self.install_locked(installation, verification_now)?;
        if signed.unsigned.expires_at <= now_unix_seconds || routing_expires_at <= now_unix_seconds
        {
            self.clear_locked()?;
            return Ok(false);
        }
        Ok(true)
    }

    /// Fetch and verify the complete trust chain before replacing the current context.
    pub async fn install(
        &self,
        installation: AuthorizationInstallation,
        now_unix_seconds: i64,
    ) -> Result<(), TrellisClientError> {
        let _update = self.lock_update()?;
        self.install_locked(installation, now_unix_seconds)
    }

    fn install_locked(
        &self,
        installation: AuthorizationInstallation,
        now_unix_seconds: i64,
    ) -> Result<(), TrellisClientError> {
        let AuthorizationInstallation {
            context: bundle,
            routing,
            runtime,
            server_clock_offset_ms,
        } = installation;
        let durable = self.store.load()?;
        if durable
            .as_ref()
            .is_some_and(|state| state.binding != self.binding)
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization context storage belongs to another identity".into(),
            ));
        }
        let manifest = parse_issuer_manifest(&bundle.trust.manifest)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let manifest_generation = manifest.unsigned.generation;
        let minimum_generation = durable.as_ref().map_or(manifest_generation, |state| {
            state
                .trust
                .minimum_manifest_generation
                .max(manifest_generation)
        });
        let root = AuthorizationTrustRoot::parse(&bundle.trust.root)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let root_digest = root
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if durable.as_ref().is_some_and(|state| {
            state.trust.authority != root.authority()
                || state.trust.root_key_id != root.key_id()
                || state.trust.root_digest != root_digest
        }) {
            return Err(TrellisClientError::Bootstrap(
                "authorization trust root changed".into(),
            ));
        }
        let policy = AuthorizationVerificationPolicy::new(
            now_unix_seconds,
            bundle.trust.policy.allowed_clock_skew_seconds,
            bundle.trust.policy.maximum_context_lifetime_seconds,
            bundle.trust.policy.maximum_context_bytes,
            bundle.trust.policy.maximum_permissions,
            bundle.trust.policy.maximum_capabilities,
            minimum_generation,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let verified_manifest = verify_issuer_manifest(&root, &manifest, &policy)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let manifest_digest = verified_manifest
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let context = persisted_signed_context(&bundle)?;
        let verified = verify_authorization_context(&root, &verified_manifest, &context, &policy)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let context_digest = verified.context_digest().to_owned();
        let context = &verified.signed_context().unsigned;
        if routing.bootstrap_jwt.trim().is_empty() || routing.bootstrap_jwt_expires_at <= 0 {
            return Err(TrellisClientError::Bootstrap(
                "authorization routing material is expired or empty".into(),
            ));
        }
        if runtime.session_id != context.session_id
            || runtime.participant_id != context.participant.id
            || runtime.participant_digest != context.participant.artifact_digest
            || runtime.needs_digest != context.participant.needs_digest
            || runtime.inbox_prefix.trim().is_empty()
            || runtime.transports.native.nats_servers.is_empty()
            || runtime
                .transports
                .native
                .nats_servers
                .iter()
                .any(|server| server.trim().is_empty())
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization runtime binding mismatch or empty native transport".into(),
            ));
        }
        let session_id = context.session_id.clone();
        let participant_artifact_digest = context.participant.artifact_digest.clone();
        let participant_needs_digest = context.participant.needs_digest.clone();
        let not_before = context.not_before;
        let expires_at = context.expires_at;
        let refresh_at = trellis_protocol::authorization_context_refresh_at(
            &context_digest,
            context.issued_at,
            not_before,
            expires_at,
            bundle.trust.policy.refresh_lead_seconds,
            bundle.trust.policy.refresh_jitter_seconds,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let trust = AuthorizationClientTrustState {
            format: "trellis.authorization-client-trust.v1".into(),
            authority: root.authority().to_owned(),
            root_key_id: root.key_id().to_owned(),
            root_digest,
            minimum_manifest_generation: verified_manifest.generation(),
            manifest_digest_at_minimum_generation: manifest_digest,
        };
        let next = AuthorizationClientState {
            format: AUTHORIZATION_CLIENT_STATE_FORMAT_.into(),
            binding: self.binding.clone(),
            trust,
            runtime: runtime.clone(),
            context: Some(bundle.clone()),
            routing: Some(routing.clone()),
            server_clock_offset_ms,
        };
        let persisted = self.store.commit(next.clone())?;
        if persisted != next {
            return Err(TrellisClientError::Bootstrap(
                "authorization context persistence did not commit exact state".into(),
            ));
        }
        let current = CurrentContext {
            bundle,
            context_digest,
            manifest_generation: verified_manifest.generation(),
            session_id,
            participant_digest: participant_artifact_digest,
            needs_digest: participant_needs_digest,
            not_before,
            expires_at,
            refresh_at,
        };
        *self
            .state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))? =
            CachedAuthorizationState {
                current: Some(current),
                runtime: Some(runtime),
                routing: Some(routing),
            };
        self.clock_offset_ms
            .store(server_clock_offset_ms, Ordering::Relaxed);
        Ok(())
    }

    /// Clear the active session context while retaining the durable trust floor.
    pub fn clear(&self) -> Result<(), TrellisClientError> {
        let _update = self.lock_update()?;
        self.clear_locked()
    }

    fn clear_locked(&self) -> Result<(), TrellisClientError> {
        self.store.clear_context()?;
        let mut state = self
            .state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        state.current = None;
        state.routing = None;
        Ok(())
    }

    /// Explicitly clear both active context and installation trust.
    pub fn reset_trust(&self) -> Result<(), TrellisClientError> {
        let _update = self.lock_update()?;
        self.store.reset_trust()?;
        *self
            .state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))? =
            CachedAuthorizationState::default();
        Ok(())
    }

    /// Return the current context digest for a reconnect proof.
    pub fn context_digest(&self) -> Result<String, TrellisClientError> {
        let now_unix_seconds = self.corrected_now_seconds()?;
        let state = self
            .state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        let current = state
            .current
            .as_ref()
            .filter(|context| {
                context.not_before <= now_unix_seconds && context.expires_at > now_unix_seconds
            })
            .ok_or_else(|| TrellisClientError::Bootstrap("authorization context expired".into()))?;
        Ok(current.context_digest.clone())
    }

    /// Return a copy of the currently verified bundle.
    pub fn bundle(&self) -> Result<AuthorizationContextBundle, TrellisClientError> {
        self.state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?
            .current
            .as_ref()
            .map(|current| current.bundle.clone())
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization context unavailable".into())
            })
    }

    /// Refresh the current context through the proof-bound auth endpoint.
    pub async fn refresh(
        &self,
        auth: &super::super::SessionAuth,
    ) -> Result<bool, TrellisClientError> {
        super::refresh::refresh(self, auth).await
    }

    pub(crate) async fn lock_refresh(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.refresh.lock().await
    }

    pub(crate) async fn advance_manifest_floor(
        &self,
        generation: u64,
        digest: &str,
    ) -> Result<bool, TrellisClientError> {
        let _update = self.lock_update()?;
        let mut state = self.store.load()?.ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization trust floor unavailable".into())
        })?;
        if generation < state.trust.minimum_manifest_generation {
            return Err(TrellisClientError::Bootstrap(
                "authorization issuer manifest rolled back".into(),
            ));
        }
        if generation == state.trust.minimum_manifest_generation {
            if digest != state.trust.manifest_digest_at_minimum_generation {
                return Err(TrellisClientError::Bootstrap(
                    "authorization issuer manifest equivocated".into(),
                ));
            }
            return Ok(false);
        }
        state.trust.minimum_manifest_generation = generation;
        state.trust.manifest_digest_at_minimum_generation = digest.to_owned();
        let persisted = self.store.commit(state.clone())?;
        if persisted != state {
            return Err(TrellisClientError::Bootstrap(
                "authorization trust floor persistence did not commit exact state".into(),
            ));
        }
        Ok(true)
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
        let now_unix_seconds = self.corrected_now_seconds()?;
        let state = self
            .state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        let Some(current) = state.current.as_ref() else {
            return Ok(std::time::Duration::from_secs(1));
        };
        let route_refresh_at = state.routing.as_ref().map_or(now_unix_seconds, |routing| {
            routing
                .bootstrap_jwt_expires_at
                .saturating_sub(i64::from(current.bundle.trust.policy.refresh_lead_seconds))
        });
        Ok(std::time::Duration::from_secs(
            u64::try_from(
                current
                    .refresh_at
                    .min(route_refresh_at)
                    .saturating_sub(now_unix_seconds)
                    .max(5),
            )
            .map_err(|_| TrellisClientError::Bootstrap("context refresh delay overflow".into()))?,
        ))
    }

    pub(crate) fn routing_jwt(&self) -> Result<String, TrellisClientError> {
        let now = self.corrected_now_seconds()?;
        self.state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?
            .routing
            .as_ref()
            .filter(|routing| routing.bootstrap_jwt_expires_at > now)
            .map(|routing| routing.bootstrap_jwt.clone())
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization routing JWT expired".into())
            })
    }

    pub(crate) fn runtime_binding(
        &self,
    ) -> Result<AuthorizationRuntimeBinding, TrellisClientError> {
        self.state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?
            .runtime
            .clone()
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization runtime unavailable".into())
            })
    }

    #[cfg(test)]
    pub(crate) fn set_server_clock_offset_ms(&self, offset_ms: i64) {
        self.clock_offset_ms.store(offset_ms, Ordering::Relaxed);
    }

    pub(crate) fn corrected_now_seconds(&self) -> Result<i64, TrellisClientError> {
        self.corrected_now_millis()
            .map(|value| value.div_euclid(1_000))
    }

    pub(crate) fn corrected_now_millis(&self) -> Result<i64, TrellisClientError> {
        let now = i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
                .as_millis(),
        )
        .map_err(|_| TrellisClientError::Bootstrap("context time overflow".into()))?;
        now.checked_add(self.clock_offset_ms.load(Ordering::Relaxed))
            .ok_or_else(|| TrellisClientError::Bootstrap("context time overflow".into()))
    }

    pub(crate) fn clock_offset_ms(&self) -> i64 {
        self.clock_offset_ms.load(Ordering::Relaxed)
    }

    pub(crate) fn refresh_evidence(
        &self,
    ) -> Result<(String, String, String, String, u64), TrellisClientError> {
        let state = self
            .state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        let current = state.current.as_ref().ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization context unavailable".into())
        })?;
        Ok((
            current.session_id.clone(),
            current.context_digest.clone(),
            current.participant_digest.clone(),
            current.needs_digest.clone(),
            current.manifest_generation,
        ))
    }

    pub(crate) fn state_snapshot(&self) -> Result<CachedAuthorizationState, TrellisClientError> {
        self.state
            .read()
            .map(|state| state.clone())
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))
    }

    fn lock_update(&self) -> Result<std::sync::MutexGuard<'_, ()>, TrellisClientError> {
        self.update.lock().map_err(|_| {
            TrellisClientError::Bootstrap("authorization context update lock poisoned".into())
        })
    }

    pub(crate) fn durable_state(
        &self,
    ) -> Result<Option<AuthorizationClientState>, TrellisClientError> {
        self.store.load()
    }

    pub(crate) fn http(&self) -> &BootstrapHttp {
        &self.http
    }
}

/// Trust input the provider cache reads from the own-context installation.
#[derive(Clone, Debug)]
pub(crate) struct ProviderTrustInput {
    pub(crate) root: Value,
    pub(crate) policy: super::types::AuthorizationTrustPolicy,
    pub(crate) minimum_manifest_generation: u64,
}

impl AuthorizationContextCache {
    pub(crate) fn provider_trust_input(&self) -> Result<ProviderTrustInput, TrellisClientError> {
        let durable = self.durable_state()?;
        let bundle = self.bundle()?;
        let manifest = parse_issuer_manifest(&bundle.trust.manifest)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let minimum_generation = durable
            .as_ref()
            .map_or(manifest.unsigned.generation, |state| {
                state
                    .trust
                    .minimum_manifest_generation
                    .max(manifest.unsigned.generation)
            });
        Ok(ProviderTrustInput {
            root: bundle.trust.root.clone(),
            policy: bundle.trust.policy.clone(),
            minimum_manifest_generation: minimum_generation,
        })
    }
}

/// Install an already-verified snapshot for unit tests without registry I/O.
#[cfg(test)]
pub(crate) fn inject_verified_for_test(
    cache: &AuthorizationContextCache,
    bundle: AuthorizationContextBundle,
    verified: trellis_protocol::VerifiedAuthorizationContext,
    _policy: AuthorizationVerificationPolicy,
) -> Result<(), TrellisClientError> {
    let context = &verified.signed_context().unsigned;
    let context_digest = verified.context_digest().to_owned();
    let refresh_at = trellis_protocol::authorization_context_refresh_at(
        &context_digest,
        context.issued_at,
        context.not_before,
        context.expires_at,
        bundle.trust.policy.refresh_lead_seconds,
        bundle.trust.policy.refresh_jitter_seconds,
    )
    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    let session_id = context.session_id.clone();
    let participant_digest = context.participant.artifact_digest.clone();
    let needs_digest = context.participant.needs_digest.clone();
    let not_before = context.not_before;
    let expires_at = context.expires_at;
    let current = CurrentContext {
        bundle,
        context_digest,
        manifest_generation: 0,
        session_id: session_id.clone(),
        participant_digest: participant_digest.clone(),
        needs_digest: needs_digest.clone(),
        not_before,
        expires_at,
        refresh_at,
    };
    let runtime = AuthorizationRuntimeBinding {
        session_id: session_id.clone(),
        participant_id: context.participant.id.clone(),
        participant_digest: participant_digest.clone(),
        needs_digest: needs_digest.clone(),
        inbox_prefix: "_INBOX.test".into(),
        transports: AuthorizationRuntimeTransports {
            native: AuthorizationNativeTransport {
                nats_servers: vec!["nats://localhost:4222".into()],
            },
        },
    };
    *cache
        .state
        .write()
        .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))? =
        CachedAuthorizationState {
            current: Some(current),
            runtime: Some(runtime),
            routing: None,
        };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::authorization::persistence::{test_state, MemoryAuthorizationContextStore};
    use crate::client::authorization::types::{
        AuthorizationRegistryBinding, AuthorizationTrustBundle, AuthorizationTrustPolicy,
    };

    #[tokio::test]
    async fn accepted_floor_persists_before_refresh_and_stale_restore_keeps_session() {
        let vectors: Value = serde_json::from_str(include_str!(
            "../../../../../../conformance/authorization-context/vectors.json"
        ))
        .unwrap();
        let chain = &vectors["completeChain"];
        let store = Arc::new(MemoryAuthorizationContextStore::default());
        let mut state = test_state(7, chain["manifestDigest"].as_str().unwrap());
        state.trust.root_key_id = chain["rootKeyId"].as_str().unwrap().into();
        state.trust.root_digest = chain["rootDigest"].as_str().unwrap().into();
        state.runtime.participant_id = "documents-web".into();
        state.runtime.participant_digest = "BAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQ".into();
        state.runtime.needs_digest = "BQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQU".into();
        state.context = Some(AuthorizationContextBundle {
            context: serde_json::from_str(chain["contextCanonicalJson"].as_str().unwrap()).unwrap(),
            trust: AuthorizationTrustBundle {
                root: serde_json::from_str(chain["rootCanonicalJson"].as_str().unwrap()).unwrap(),
                manifest: serde_json::from_str(chain["manifestCanonicalJson"].as_str().unwrap())
                    .unwrap(),
                authorization_registry: AuthorizationRegistryBinding {
                    trust_bucket: "trust".into(),
                    context_bucket: "contexts".into(),
                },
                policy: AuthorizationTrustPolicy {
                    allowed_clock_skew_seconds: 0,
                    maximum_context_lifetime_seconds: 600,
                    maximum_context_bytes: 65_536,
                    maximum_permissions: 256,
                    maximum_capabilities: 256,
                    refresh_lead_seconds: 60,
                    refresh_jitter_seconds: 0,
                },
            },
        });
        state.routing = Some(AuthorizationRoutingMaterial {
            bootstrap_jwt: "route".into(),
            bootstrap_jwt_expires_at: 2_000,
        });
        store.commit(state).unwrap();
        let other_participant = AuthorizationContextCache::new(
            "https://trellis.test",
            "app:other-participant:other-digest",
            store.clone(),
        )
        .unwrap();
        assert!(matches!(
            other_participant.restore(1_100).await,
            Err(TrellisClientError::Bootstrap(message))
                if message == "authorization context storage belongs to another identity"
        ));
        let cache = AuthorizationContextCache::new(
            "https://trellis.test",
            "service:dep:instance",
            store.clone(),
        )
        .unwrap();
        assert!(cache.restore(1_100).await.unwrap());

        for (offset_ms, local_now) in [(1_000_000, 100), (-1_000_000, 2_100)] {
            let mut durable = store.load().unwrap().unwrap();
            durable.server_clock_offset_ms = offset_ms;
            store.commit(durable).unwrap();
            let restarted = AuthorizationContextCache::new(
                "https://trellis.test",
                "service:dep:instance",
                store.clone(),
            )
            .unwrap();
            assert!(restarted.restore(local_now).await.unwrap());
        }

        assert!(cache.advance_manifest_floor(8, "manifest-8").await.unwrap());
        assert!(!cache.restore(1_100).await.unwrap());

        let durable = store.load().unwrap().unwrap();
        assert_eq!(durable.trust.minimum_manifest_generation, 8);
        assert_eq!(
            durable.trust.manifest_digest_at_minimum_generation,
            "manifest-8"
        );
        assert!(durable.context.is_none());
        assert!(durable.routing.is_none());
        assert_eq!(
            cache.state_snapshot().unwrap().runtime,
            Some(durable.runtime)
        );
    }
}
