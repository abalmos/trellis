use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::time::Duration;

use futures_util::StreamExt;
use trellis_protocol::{
    parse_authorization_context, verify_authorization_context, AuthorizationContextPurpose,
    AuthorizationIssuerKey, AuthorizationIssuerState, AuthorizationVerificationPolicy,
    SignedAuthorizationContext, VerifiedAuthorizationContext,
};

use super::super::TrellisClientError;
use super::bootstrap_http::BootstrapHttp;
use super::own_context::{system_now_millis, AuthorizationContextCache};
use super::registry::{validate_digest_key, AuthorizationRegistryReader, REVOCATION_PREFIX};
use super::types::AuthorizationRegistryBinding;

#[cfg(feature = "runtime-internals")]
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct RuntimeAuthorizationTrust {
    /// Configured origin for public issuer-key resolution.
    pub trellis_origin: String,
    /// Locally owned issuer, if this runtime issues contexts itself.
    pub issuer: Option<AuthorizationIssuerKey>,
    /// Bounds applied to every resolved context.
    pub policy: AuthorizationVerificationPolicy,
}

#[derive(Clone, Debug)]
pub(crate) struct AuthorizationProviderCacheHealth {
    pub(crate) healthy: bool,
}

#[cfg(feature = "runtime-internals")]
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RuntimeAuthorizationIoCounters {
    pub context_resolves: u64,
}

const MAX_CACHED_CONTEXTS: usize = 256;

#[derive(Default)]
struct CachedVerifications {
    live: Option<VerifiedAuthorizationContext>,
    historical: Option<VerifiedAuthorizationContext>,
}

struct CachedContext {
    signed: SignedAuthorizationContext,
    issuer: AuthorizationIssuerKey,
    verified: Mutex<CachedVerifications>,
    epoch: u64,
    covered: Arc<AtomicBool>,
    watch: tokio::task::AbortHandle,
    leases: AtomicUsize,
    last_used: AtomicU64,
}

impl Drop for CachedContext {
    fn drop(&mut self) {
        self.covered.store(false, Ordering::Release);
        self.watch.abort();
    }
}

/// Lease keeping a cached authorization context and its revocation watch alive.
#[doc(hidden)]
pub struct AuthorizationContextLease {
    entry: Arc<CachedContext>,
    context: VerifiedAuthorizationContext,
}

impl Deref for AuthorizationContextLease {
    type Target = VerifiedAuthorizationContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl Drop for AuthorizationContextLease {
    fn drop(&mut self) {
        self.entry.leases.fetch_sub(1, Ordering::Release);
    }
}

#[derive(Default)]
struct ProviderState {
    contexts: HashMap<String, Arc<CachedContext>>,
    issuers: HashMap<String, AuthorizationIssuerKey>,
    // Negative evidence is retained no longer than a possible live context lease.
    revocations: HashMap<String, (i64, i64)>,
}

impl ProviderState {
    fn revocation_time(&self, digest: &str) -> Result<Option<i64>, ()> {
        if let Some((revoked_at, _)) = self.revocations.get(digest) {
            return Ok(Some(*revoked_at));
        }
        if self
            .contexts
            .get(digest)
            .is_some_and(|entry| !entry.covered.load(Ordering::Acquire))
        {
            return Err(());
        }
        Ok(None)
    }

    fn insert_context(
        &mut self,
        digest: String,
        entry: Arc<CachedContext>,
    ) -> Result<(), Arc<CachedContext>> {
        if self.contexts.len() >= MAX_CACHED_CONTEXTS && !self.contexts.contains_key(&digest) {
            let Some(oldest) = self
                .contexts
                .iter()
                .filter(|(_, entry)| entry.leases.load(Ordering::Acquire) == 0)
                .min_by_key(|(_, entry)| entry.last_used.load(Ordering::Acquire))
                .map(|(digest, _)| digest.clone())
            else {
                return Err(entry);
            };
            self.contexts.remove(&oldest);
        }
        self.contexts.insert(digest, entry);
        Ok(())
    }
}

/// Connection-scoped caller-context verification using online issuer keys.
///
/// Cached digests retain separate live and historical verification results and
/// require an exact revocation watch on the same NATS connection epoch.
#[derive(Clone)]
pub struct AuthorizationProviderCache {
    nats: async_nats::Client,
    registry: AuthorizationRegistryReader,
    http: BootstrapHttp,
    own: Option<Arc<AuthorizationContextCache>>,
    verification_policy: AuthorizationVerificationPolicy,
    state: Arc<RwLock<ProviderState>>,
    in_flight: Arc<Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>>,
    issuer_resolution: Arc<tokio::sync::Mutex<()>>,
    closed: Arc<AtomicBool>,
    context_resolves: Arc<AtomicU64>,
    access_clock: Arc<AtomicU64>,
}

impl AuthorizationProviderCache {
    pub(crate) async fn attach(
        nats: async_nats::Client,
        binding: &AuthorizationRegistryBinding,
        own: Arc<AuthorizationContextCache>,
    ) -> Result<Self, TrellisClientError> {
        let bundle = own.bundle()?;
        let policy = bundle
            .policy
            .verification_policy(own.corrected_now_seconds()?)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let cache = Self::open(
            nats,
            binding,
            own.http().clone(),
            Some(bundle.issuer),
            policy,
            Some(own.clone()),
        )
        .await?;
        cache
            .resolve_context(&own.context_digest()?, own.corrected_now_seconds()?)
            .await?;
        Ok(cache)
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn attach_runtime(
        nats: async_nats::Client,
        binding: &AuthorizationRegistryBinding,
        trust: RuntimeAuthorizationTrust,
    ) -> Result<Self, TrellisClientError> {
        Self::open(
            nats,
            binding,
            BootstrapHttp::new(&trust.trellis_origin)?,
            trust.issuer,
            trust.policy,
            None,
        )
        .await
    }

    async fn open(
        nats: async_nats::Client,
        binding: &AuthorizationRegistryBinding,
        http: BootstrapHttp,
        issuer: Option<AuthorizationIssuerKey>,
        verification_policy: AuthorizationVerificationPolicy,
        own: Option<Arc<AuthorizationContextCache>>,
    ) -> Result<Self, TrellisClientError> {
        if let Some(issuer) = &issuer {
            issuer
                .verifying_key()
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        }
        let registry = AuthorizationRegistryReader::open(nats.clone(), binding).await?;
        Ok(Self {
            nats,
            registry,
            http,
            own,
            verification_policy,
            state: Arc::new(RwLock::new(ProviderState {
                issuers: issuer
                    .into_iter()
                    .map(|issuer| (issuer.key_id.clone(), issuer))
                    .collect(),
                ..Default::default()
            })),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            issuer_resolution: Arc::new(tokio::sync::Mutex::new(())),
            closed: Arc::new(AtomicBool::new(false)),
            context_resolves: Arc::new(AtomicU64::new(0)),
            access_clock: Arc::new(AtomicU64::new(0)),
        })
    }

    pub(crate) async fn run(
        &self,
        mut stop: tokio::sync::watch::Receiver<()>,
    ) -> Result<(), TrellisClientError> {
        let mut cleanup = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = stop.changed() => break,
                _ = cleanup.tick() => {
                    let now = self.now_seconds()?;
                    let epoch = self.epoch();
                    let connected = self.health()?.healthy;
                    let mut state = self.write_state()?;
                    state.contexts.retain(|_, entry| {
                        entry.leases.load(Ordering::Acquire) > 0
                            || (connected && entry.epoch == epoch && entry.covered.load(Ordering::Acquire))
                    });
                    state.revocations.retain(|_, (_, expires_at)| *expires_at > now);
                }
            }
        }
        self.closed.store(true, Ordering::Release);
        Ok(())
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn run_runtime(
        &self,
        stop: tokio::sync::watch::Receiver<()>,
    ) -> Result<(), TrellisClientError> {
        self.run(stop).await
    }

    pub(crate) async fn wait_ready(
        &self,
        stop: tokio::sync::watch::Receiver<()>,
    ) -> Result<(), TrellisClientError> {
        if stop.has_changed().is_err() || !self.health()?.healthy {
            return Err(TrellisClientError::AuthorizationUnavailable(
                "provider is not connected".into(),
            ));
        }
        Ok(())
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn wait_until_ready(&self) -> Result<(), TrellisClientError> {
        if !self.health()?.healthy {
            return Err(TrellisClientError::AuthorizationUnavailable(
                "provider is not connected".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn health(&self) -> Result<AuthorizationProviderCacheHealth, TrellisClientError> {
        Ok(AuthorizationProviderCacheHealth {
            healthy: !self.closed.load(Ordering::Acquire)
                && self.nats.connection_state() == async_nats::connection::State::Connected,
        })
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_healthy(&self) -> bool {
        self.health().is_ok_and(|health| health.healthy)
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    #[must_use]
    pub fn runtime_io_counters(&self) -> RuntimeAuthorizationIoCounters {
        RuntimeAuthorizationIoCounters {
            context_resolves: self.context_resolves.load(Ordering::Relaxed),
        }
    }

    fn epoch(&self) -> u64 {
        self.nats.statistics().connects.load(Ordering::Acquire)
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_lease_cached_context(
        &self,
        digest: &str,
    ) -> Result<Option<AuthorizationContextLease>, TrellisClientError> {
        self.lease_cached_context(digest, false)
    }

    pub(crate) fn revocation_time(&self, digest: &str) -> Result<Option<i64>, TrellisClientError> {
        self.read_state()?.revocation_time(digest).map_err(|()| {
            TrellisClientError::AuthorizationUnavailable(
                "exact context revocation watch is unavailable".into(),
            )
        })
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_revocation_time(&self, digest: &str) -> Result<Option<i64>, TrellisClientError> {
        self.revocation_time(digest)
    }

    fn observe_revocation(&self, digest: &str, revoked_at: i64) -> Result<(), TrellisClientError> {
        let deadline = self
            .now_seconds()?
            .saturating_add(i64::from(
                self.verification_policy.maximum_context_lifetime_seconds,
            ))
            .saturating_add(i64::from(
                self.verification_policy.allowed_clock_skew_seconds,
            ));
        let mut state = self.write_state()?;
        state
            .revocations
            .entry(digest.to_owned())
            .and_modify(|(at, until)| {
                *at = (*at).max(revoked_at);
                *until = (*until).max(deadline);
            })
            .or_insert((revoked_at, deadline));
        if let Some(entry) = state.contexts.get(digest) {
            entry.covered.store(false, Ordering::Release);
        }
        drop(state);
        if let Some(own) = &self.own {
            if own.context_digest().is_ok_and(|current| current == digest) {
                own.clear()?;
                own.request_refresh();
            }
        }
        Ok(())
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn apply_runtime_revocation(
        &self,
        digest: &str,
        revoked_at: i64,
    ) -> Result<(), TrellisClientError> {
        validate_digest_key(digest)?;
        if revoked_at <= 0 {
            return Err(TrellisClientError::Bootstrap(
                "invalid revocation time".into(),
            ));
        }
        self.observe_revocation(digest, revoked_at)
    }

    fn now_seconds(&self) -> Result<i64, TrellisClientError> {
        self.own.as_ref().map_or_else(
            || system_now_millis().map(|now| now.div_euclid(1000)),
            |own| own.corrected_now_seconds(),
        )
    }

    pub(crate) fn policy(&self) -> Result<AuthorizationVerificationPolicy, TrellisClientError> {
        let mut policy = self.verification_policy.clone();
        policy.now_unix_seconds = self.now_seconds()?;
        Ok(policy)
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_policy(&self) -> Result<AuthorizationVerificationPolicy, TrellisClientError> {
        self.policy()
    }

    pub(crate) async fn resolve_context(
        &self,
        digest: &str,
        now: i64,
    ) -> Result<AuthorizationContextLease, TrellisClientError> {
        self.resolve_context_for(digest, now, false).await
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn resolve_admission_context(
        &self,
        digest: &str,
        now: i64,
    ) -> Result<AuthorizationContextLease, TrellisClientError> {
        self.resolve_context(digest, now).await
    }

    pub(crate) async fn resolve_event_context(
        &self,
        digest: &str,
        event_time: i64,
    ) -> Result<AuthorizationContextLease, TrellisClientError> {
        self.resolve_context_for(digest, event_time, true).await
    }

    pub(crate) async fn resolve_event_context_for_verification(
        &self,
        digest: &str,
        event_time: i64,
    ) -> Result<AuthorizationContextLease, crate::service::EventVerificationFailure> {
        self.resolve_event_context(digest, event_time)
            .await
            .map_err(classify_event_resolution_failure)
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn runtime_resolve_event_context(
        &self,
        digest: &str,
        event_time: i64,
    ) -> Result<AuthorizationContextLease, TrellisClientError> {
        self.resolve_event_context(digest, event_time).await
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn runtime_resolve_event_context_for_verification(
        &self,
        digest: &str,
        event_time: i64,
    ) -> Result<AuthorizationContextLease, crate::service::EventVerificationFailure> {
        self.resolve_event_context_for_verification(digest, event_time)
            .await
    }

    async fn resolve_context_for(
        &self,
        digest: &str,
        verification_time: i64,
        historical: bool,
    ) -> Result<AuthorizationContextLease, TrellisClientError> {
        validate_digest_key(digest)?;
        if !self.health()?.healthy {
            return Err(TrellisClientError::AuthorizationUnavailable(
                "provider is not connected".into(),
            ));
        }
        if self.revocation_time(digest)?.is_some() {
            return Err(TrellisClientError::Bootstrap(
                "authorization context is revoked".into(),
            ));
        }
        if let Some(context) = self.lease_cached_context(digest, historical)? {
            return Ok(context);
        }
        let pending = {
            let mut pending = self.in_flight.lock().map_err(|_| {
                TrellisClientError::AuthorizationUnavailable(
                    "provider resolution lock poisoned".into(),
                )
            })?;
            pending.retain(|_, lock| lock.strong_count() > 0);
            if let Some(lock) = pending.get(digest).and_then(Weak::upgrade) {
                lock
            } else {
                if pending.len() >= 32 {
                    return Err(TrellisClientError::AuthorizationUnavailable(
                        "provider cold-resolution capacity reached".into(),
                    ));
                }
                let lock = Arc::new(tokio::sync::Mutex::new(()));
                pending.insert(digest.to_owned(), Arc::downgrade(&lock));
                lock
            }
        };
        let _guard = pending.lock().await;
        if self.revocation_time(digest)?.is_some() {
            return Err(TrellisClientError::Bootstrap(
                "authorization context is revoked".into(),
            ));
        }
        if let Some(context) = self.lease_cached_context(digest, historical)? {
            return Ok(context);
        }
        self.discard_unusable_context(digest)?;
        tokio::time::timeout(
            Duration::from_secs(30),
            self.resolve_context_once(digest, verification_time, historical),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
    }

    async fn resolve_context_once(
        &self,
        digest: &str,
        verification_time: i64,
        historical: bool,
    ) -> Result<AuthorizationContextLease, TrellisClientError> {
        let epoch = self.epoch();
        self.context_resolves.fetch_add(1, Ordering::Relaxed);
        let value = self.registry.get_context(digest).await?.ok_or_else(|| {
            TrellisClientError::AuthorizationUnavailable(
                "context is missing from the registry".into(),
            )
        })?;
        let mut policy = self.policy()?;
        if value.len() > policy.maximum_context_bytes {
            return Err(TrellisClientError::Bootstrap(
                "authorization context exceeds size limit".into(),
            ));
        }
        let json = serde_json::from_slice(&value)?;
        let signed = parse_authorization_context(&json)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if signed
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
            != digest
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization context digest does not match its registry key".into(),
            ));
        }
        let key_id = &signed.unsigned.issuer_key_id;
        let known = self.read_state()?.issuers.get(key_id).cloned();
        let issuer = if let Some(issuer) = known {
            issuer
        } else {
            let _guard = self.issuer_resolution.lock().await;
            let known = self.read_state()?.issuers.get(key_id).cloned();
            if let Some(issuer) = known {
                issuer
            } else {
                let issuer = self.http.issuer_key(key_id).await?;
                self.write_state()?
                    .issuers
                    .insert(key_id.clone(), issuer.clone());
                issuer
            }
        };
        let now = self.now_seconds()?;
        let live = issuer.state == AuthorizationIssuerState::Active
            && signed.unsigned.not_before <= now
            && signed.unsigned.expires_at > now;
        let purpose = if historical {
            AuthorizationContextPurpose::HistoricalEvent
        } else {
            AuthorizationContextPurpose::Live
        };
        policy.now_unix_seconds = if historical { verification_time } else { now };
        let verified = verify_authorization_context(&issuer, &signed, &policy, purpose)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if !historical && !live {
            return Err(TrellisClientError::Bootstrap(
                "authorization context is not current".into(),
            ));
        }
        let mut watch = self.registry.watch_revocation(digest).await?;
        if let Some(value) = self.registry.get_revocation(digest).await? {
            self.observe_revocation(digest, parse_revocation_record(&value)?)?;
        }
        if self.revocation_time(digest)?.is_some() {
            return Err(TrellisClientError::Bootstrap(
                "authorization context is revoked".into(),
            ));
        }
        if !self.health()?.healthy || self.epoch() != epoch {
            return Err(TrellisClientError::AuthorizationUnavailable(
                "connection changed during context resolution".into(),
            ));
        }
        let covered = Arc::new(AtomicBool::new(true));
        let weak_state = Arc::downgrade(&self.state);
        let own = self.own.as_ref().map(Arc::downgrade);
        let watch_covered = covered.clone();
        let watch_digest = digest.to_owned();
        let revocation_deadline = now
            .saturating_add(i64::from(
                self.verification_policy.maximum_context_lifetime_seconds,
            ))
            .saturating_add(i64::from(
                self.verification_policy.allowed_clock_skew_seconds,
            ));
        // The task owns only weak cache references; dropping the entry aborts it.
        let task = tokio::spawn(async move {
            let entry = watch.next().await;
            watch_covered.store(false, Ordering::Release);
            let revoked_at = match entry {
                Some(Ok(entry))
                    if !entry.removed
                        && entry.key == format!("{REVOCATION_PREFIX}{watch_digest}") =>
                {
                    parse_revocation_record(&entry.value).ok()
                }
                _ => None,
            };
            if let Some(state) = weak_state.upgrade() {
                if let Ok(mut state) = state.write() {
                    if let Some(at) = revoked_at {
                        state
                            .revocations
                            .insert(watch_digest.clone(), (at, revocation_deadline));
                    }
                    if state.contexts.get(&watch_digest).is_some_and(|entry| {
                        Arc::ptr_eq(&entry.covered, &watch_covered)
                            && entry.leases.load(Ordering::Acquire) == 0
                    }) {
                        state.contexts.remove(&watch_digest);
                    }
                }
            }
            if revoked_at.is_some() {
                if let Some(own) = own.and_then(|own| own.upgrade()) {
                    if own
                        .context_digest()
                        .is_ok_and(|digest| digest == watch_digest)
                    {
                        if let Err(error) = own.clear() {
                            tracing::warn!(%error, "cannot discard revoked own context");
                        }
                        own.request_refresh();
                    }
                }
            }
        });
        let mut verifications = CachedVerifications::default();
        if historical {
            verifications.historical = Some(verified.clone());
        } else {
            verifications.live = Some(verified.clone());
        }
        let entry = Arc::new(CachedContext {
            signed,
            issuer,
            verified: Mutex::new(verifications),
            epoch,
            covered,
            watch: task.abort_handle(),
            leases: AtomicUsize::new(1),
            last_used: AtomicU64::new(self.next_access()),
        });
        let mut state = self.write_state()?;
        if state.revocations.contains_key(digest) || !entry.covered.load(Ordering::Acquire) {
            return Err(TrellisClientError::AuthorizationUnavailable(
                "revocation coverage changed during resolution".into(),
            ));
        }
        if state
            .contexts
            .get(digest)
            .is_some_and(|existing| existing.leases.load(Ordering::Acquire) > 0)
        {
            return Err(TrellisClientError::AuthorizationUnavailable(
                "provider context is still leased".into(),
            ));
        }
        state
            .insert_context(digest.to_owned(), entry.clone())
            .map_err(|_| {
                TrellisClientError::AuthorizationUnavailable(
                    "provider context cache capacity reached".into(),
                )
            })?;
        Ok(AuthorizationContextLease {
            entry,
            context: verified,
        })
    }

    fn lease_cached_context(
        &self,
        digest: &str,
        historical: bool,
    ) -> Result<Option<AuthorizationContextLease>, TrellisClientError> {
        if !self.health()?.healthy {
            return Ok(None);
        }
        let now = self.now_seconds()?;
        let epoch = self.epoch();
        let state = self.write_state()?;
        let Some(entry) = state.contexts.get(digest) else {
            return Ok(None);
        };
        if entry.epoch != epoch || !entry.covered.load(Ordering::Acquire) {
            return Ok(None);
        }
        let mut verifications = entry.verified.lock().map_err(|_| {
            TrellisClientError::AuthorizationUnavailable(
                "provider verification cache lock poisoned".into(),
            )
        })?;
        let cached = if historical {
            &mut verifications.historical
        } else {
            &mut verifications.live
        };
        if cached.is_none() {
            let mut policy = self.policy()?;
            policy.now_unix_seconds = if historical {
                policy.now_unix_seconds
            } else {
                now
            };
            *cached = Some(
                verify_authorization_context(
                    &entry.issuer,
                    &entry.signed,
                    &policy,
                    if historical {
                        AuthorizationContextPurpose::HistoricalEvent
                    } else {
                        AuthorizationContextPurpose::Live
                    },
                )
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?,
            );
        }
        let context = cached.clone().ok_or_else(|| {
            TrellisClientError::AuthorizationUnavailable(
                "provider verification result is unavailable".into(),
            )
        })?;
        if !historical && (context.not_before() > now || context.expires_at() <= now) {
            return Ok(None);
        }
        entry.leases.fetch_add(1, Ordering::AcqRel);
        entry.last_used.store(self.next_access(), Ordering::Release);
        Ok(Some(AuthorizationContextLease {
            entry: entry.clone(),
            context,
        }))
    }

    fn next_access(&self) -> u64 {
        self.access_clock.fetch_add(1, Ordering::Relaxed)
    }

    fn discard_unusable_context(&self, digest: &str) -> Result<(), TrellisClientError> {
        let mut state = self.write_state()?;
        let Some(entry) = state.contexts.get(digest) else {
            return Ok(());
        };
        if entry.leases.load(Ordering::Acquire) > 0 {
            return Err(TrellisClientError::AuthorizationUnavailable(
                "provider context is still leased".into(),
            ));
        }
        state.contexts.remove(digest);
        Ok(())
    }

    fn read_state(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, ProviderState>, TrellisClientError> {
        self.state.read().map_err(|_| {
            TrellisClientError::AuthorizationUnavailable("provider state lock poisoned".into())
        })
    }

    fn write_state(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, ProviderState>, TrellisClientError> {
        self.state.write().map_err(|_| {
            TrellisClientError::AuthorizationUnavailable("provider state lock poisoned".into())
        })
    }
}

fn classify_event_resolution_failure(
    error: TrellisClientError,
) -> crate::service::EventVerificationFailure {
    if matches!(
        error,
        TrellisClientError::AuthorizationUnavailable(_)
            | TrellisClientError::BootstrapHttp { .. }
            | TrellisClientError::Io(_)
            | TrellisClientError::Nats(_)
            | TrellisClientError::NatsConnect(_)
            | TrellisClientError::NatsRequest(_)
            | TrellisClientError::Timeout
    ) {
        crate::service::EventVerificationFailure::retryable(error.to_string())
    } else {
        crate::service::EventVerificationFailure::rejected(error.to_string())
    }
}

fn parse_revocation_record(value: &[u8]) -> Result<i64, TrellisClientError> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Record {
        revoked_at: i64,
    }
    let record: Record = serde_json::from_slice(value)
        .map_err(|error| TrellisClientError::AuthorizationUnavailable(error.to_string()))?;
    if record.revoked_at <= 0 {
        return Err(TrellisClientError::AuthorizationUnavailable(
            "invalid context revocation record".into(),
        ));
    }
    Ok(record.revoked_at)
}

#[cfg(test)]
mod wire_tests {
    use super::*;

    fn test_context() -> (
        SignedAuthorizationContext,
        AuthorizationIssuerKey,
        VerifiedAuthorizationContext,
    ) {
        let vectors: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../../conformance/authorization-context/vectors.json"
        ))
        .unwrap();
        let complete = &vectors["completeChain"];
        let value: serde_json::Value =
            serde_json::from_str(complete["contextCanonicalJson"].as_str().unwrap()).unwrap();
        let signed = parse_authorization_context(&value).unwrap();
        let issuer = AuthorizationIssuerKey {
            key_id: complete["issuerKeyId"].as_str().unwrap().to_owned(),
            public_key: complete["issuerPublicKey"].as_str().unwrap().to_owned(),
            state: AuthorizationIssuerState::Active,
        };
        let policy = AuthorizationVerificationPolicy::new(1_200, 30, 1_000, 100_000, 100).unwrap();
        let verified = verify_authorization_context(
            &issuer,
            &signed,
            &policy,
            AuthorizationContextPurpose::Live,
        )
        .unwrap();
        (signed, issuer, verified)
    }

    fn test_entry(
        signed: &SignedAuthorizationContext,
        issuer: &AuthorizationIssuerKey,
        verified: &VerifiedAuthorizationContext,
        last_used: u64,
        cancelled: Arc<AtomicBool>,
    ) -> Arc<CachedContext> {
        struct Cancellation(Arc<AtomicBool>);

        impl Drop for Cancellation {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Release);
            }
        }

        let cancellation = Cancellation(cancelled);
        let task = tokio::spawn(async move {
            let _cancellation = cancellation;
            std::future::pending::<()>().await;
        });
        Arc::new(CachedContext {
            signed: signed.clone(),
            issuer: issuer.clone(),
            verified: Mutex::new(CachedVerifications {
                live: Some(verified.clone()),
                historical: None,
            }),
            epoch: 1,
            covered: Arc::new(AtomicBool::new(true)),
            watch: task.abort_handle(),
            leases: AtomicUsize::new(0),
            last_used: AtomicU64::new(last_used),
        })
    }

    #[test]
    fn revocation_is_additively_tolerant() {
        assert_eq!(
            parse_revocation_record(br#"{"revokedAt":123}"#).unwrap(),
            123
        );
        assert_eq!(
            parse_revocation_record(br#"{"revokedAt":123,"future":true}"#).unwrap(),
            123
        );
        assert!(parse_revocation_record(br#"{"revokedAt":0}"#).is_err());
    }

    #[test]
    fn unavailable_evidence_requires_redelivery_but_invalid_digest_does_not() {
        for error in [
            TrellisClientError::AuthorizationUnavailable("watch disconnected".into()),
            TrellisClientError::BootstrapHttp {
                status: 503,
                code: "unavailable".into(),
            },
            TrellisClientError::BootstrapHttp {
                status: 404,
                code: "key_not_found".into(),
            },
        ] {
            assert!(matches!(
                classify_event_resolution_failure(error),
                crate::service::EventVerificationFailure::Retryable(_)
            ));
        }
        assert!(matches!(
            classify_event_resolution_failure(validate_digest_key("invalid").unwrap_err()),
            crate::service::EventVerificationFailure::Rejected(_)
        ));
    }

    #[tokio::test]
    async fn bounded_lru_retains_leased_revoked_context_until_release() {
        let (signed, issuer, verified) = test_context();
        let cancellations = (0..=MAX_CACHED_CONTEXTS)
            .map(|_| Arc::new(AtomicBool::new(false)))
            .collect::<Vec<_>>();
        let mut state = ProviderState::default();
        let first = test_entry(&signed, &issuer, &verified, 0, cancellations[0].clone());
        first.leases.store(1, Ordering::Release);
        assert!(state.insert_context("0".into(), first.clone()).is_ok());
        let lease = AuthorizationContextLease {
            entry: first.clone(),
            context: verified.clone(),
        };
        for index in 1..MAX_CACHED_CONTEXTS {
            assert!(state
                .insert_context(
                    index.to_string(),
                    test_entry(
                        &signed,
                        &issuer,
                        &verified,
                        index as u64,
                        cancellations[index].clone(),
                    ),
                )
                .is_ok());
        }

        assert!(state
            .insert_context(
                MAX_CACHED_CONTEXTS.to_string(),
                test_entry(
                    &signed,
                    &issuer,
                    &verified,
                    MAX_CACHED_CONTEXTS as u64,
                    cancellations[MAX_CACHED_CONTEXTS].clone(),
                ),
            )
            .is_ok());
        assert_eq!(state.contexts.len(), MAX_CACHED_CONTEXTS);
        assert!(state.contexts.contains_key("0"));
        assert!(!state.contexts.contains_key("1"));
        assert!(!cancellations[0].load(Ordering::Acquire));

        state.revocations.insert("0".into(), (1_201, 2_000));
        first.covered.store(false, Ordering::Release);
        assert_eq!(state.revocation_time("0"), Ok(Some(1_201)));
        assert!(state.contexts.contains_key("0"));

        drop(lease);
        drop(first);
        assert!(state
            .insert_context(
                "next".into(),
                test_entry(&signed, &issuer, &verified, 257, Arc::default()),
            )
            .is_ok());
        assert!(!state.contexts.contains_key("0"));
        tokio::task::yield_now().await;
        assert!(cancellations[0].load(Ordering::Acquire));
        assert!(cancellations[1].load(Ordering::Acquire));
    }
}
