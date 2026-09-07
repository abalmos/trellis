use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::time::Duration;

use futures_util::StreamExt;
use trellis_protocol::{
    parse_authorization_context, verify_authorization_context, AuthorizationContextPurpose,
    AuthorizationIssuerKey, AuthorizationIssuerState, AuthorizationVerificationPolicy,
    VerifiedAuthorizationContext,
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

struct CachedContext {
    context: VerifiedAuthorizationContext,
    epoch: u64,
    covered: Arc<AtomicBool>,
    watch: tokio::task::AbortHandle,
}

impl Drop for CachedContext {
    fn drop(&mut self) {
        self.covered.store(false, Ordering::Release);
        self.watch.abort();
    }
}

#[derive(Default)]
struct ProviderState {
    contexts: HashMap<String, CachedContext>,
    issuers: HashMap<String, AuthorizationIssuerKey>,
    // Negative evidence is retained no longer than a possible live context lease.
    revocations: HashMap<String, (i64, i64)>,
}

/// Connection-scoped caller-context verification using online issuer keys.
///
/// Live cache entries require an exact revocation watch on the same NATS
/// connection epoch. Expired contexts are resolved afresh for historical events.
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
                    state.contexts.retain(|_, entry| connected && entry.epoch == epoch && entry.context.expires_at() > now);
                    state.revocations.retain(|_, (_, expires_at)| *expires_at > now);
                }
            }
        }
        self.closed.store(true, Ordering::Release);
        self.write_state()?.contexts.clear();
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

    pub(crate) fn verified_context_raw(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContext>, TrellisClientError> {
        if !self.health()?.healthy {
            return Ok(None);
        }
        let now = self.now_seconds()?;
        let epoch = self.epoch();
        Ok(self
            .read_state()?
            .contexts
            .get(digest)
            .filter(|entry| {
                entry.epoch == epoch
                    && entry.covered.load(Ordering::Acquire)
                    && entry.context.not_before() <= now
                    && entry.context.expires_at() > now
            })
            .map(|entry| entry.context.clone()))
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_verified_context_raw(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContext>, TrellisClientError> {
        self.verified_context_raw(digest)
    }

    pub(crate) fn revocation_time(&self, digest: &str) -> Result<Option<i64>, TrellisClientError> {
        Ok(self
            .read_state()?
            .revocations
            .get(digest)
            .map(|(at, _)| *at))
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
    ) -> Result<VerifiedAuthorizationContext, TrellisClientError> {
        self.resolve_context_for(digest, now, false).await
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn resolve_admission_context(
        &self,
        digest: &str,
        now: i64,
    ) -> Result<VerifiedAuthorizationContext, TrellisClientError> {
        self.resolve_context(digest, now).await
    }

    pub(crate) async fn resolve_event_context(
        &self,
        digest: &str,
        event_time: i64,
    ) -> Result<VerifiedAuthorizationContext, TrellisClientError> {
        self.resolve_context_for(digest, event_time, true).await
    }

    pub(crate) async fn resolve_event_context_for_verification(
        &self,
        digest: &str,
        event_time: i64,
    ) -> Result<VerifiedAuthorizationContext, crate::service::EventVerificationFailure> {
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
    ) -> Result<VerifiedAuthorizationContext, TrellisClientError> {
        self.resolve_event_context(digest, event_time).await
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn runtime_resolve_event_context_for_verification(
        &self,
        digest: &str,
        event_time: i64,
    ) -> Result<VerifiedAuthorizationContext, crate::service::EventVerificationFailure> {
        self.resolve_event_context_for_verification(digest, event_time)
            .await
    }

    async fn resolve_context_for(
        &self,
        digest: &str,
        verification_time: i64,
        historical: bool,
    ) -> Result<VerifiedAuthorizationContext, TrellisClientError> {
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
        if let Some(context) = self.verified_context_raw(digest)? {
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
        if let Some(context) = self.verified_context_raw(digest)? {
            return Ok(context);
        }
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
    ) -> Result<VerifiedAuthorizationContext, TrellisClientError> {
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
        let cacheable = issuer.state == AuthorizationIssuerState::Active
            && signed.unsigned.not_before <= now
            && signed.unsigned.expires_at > now;
        let purpose = if historical && !cacheable {
            AuthorizationContextPurpose::HistoricalEvent
        } else {
            AuthorizationContextPurpose::Live
        };
        policy.now_unix_seconds = if cacheable { now } else { verification_time };
        let verified = verify_authorization_context(&issuer, &signed, &policy, purpose)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if !historical && !cacheable {
            return Err(TrellisClientError::Bootstrap(
                "authorization context is not current".into(),
            ));
        }
        let watch = if cacheable {
            Some(self.registry.watch_revocation(digest).await?)
        } else {
            None
        };
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
        if let Some(mut watch) = watch {
            let now = self.now_seconds()?;
            let lifetime = verified
                .expires_at()
                .checked_sub(now)
                .filter(|seconds| *seconds > 0)
                .ok_or_else(|| {
                    TrellisClientError::AuthorizationUnavailable(
                        "context expired during resolution".into(),
                    )
                })?;
            let deadline = tokio::time::Instant::now() + Duration::from_secs(lifetime as u64);
            let covered = Arc::new(AtomicBool::new(true));
            let weak_state = Arc::downgrade(&self.state);
            let own = self.own.as_ref().map(Arc::downgrade);
            let watch_covered = covered.clone();
            let watch_digest = digest.to_owned();
            let expires_at = verified.expires_at();
            // The task owns only weak cache references; dropping the entry aborts it.
            let task = tokio::spawn(async move {
                let entry = tokio::select! {
                    _ = tokio::time::sleep_until(deadline) => None,
                    entry = watch.next() => entry,
                };
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
                                .insert(watch_digest.clone(), (at, expires_at));
                        }
                        if state
                            .contexts
                            .get(&watch_digest)
                            .is_some_and(|entry| Arc::ptr_eq(&entry.covered, &watch_covered))
                        {
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
            let entry = CachedContext {
                context: verified.clone(),
                epoch,
                covered,
                watch: task.abort_handle(),
            };
            let mut state = self.write_state()?;
            if state.revocations.contains_key(digest) || !entry.covered.load(Ordering::Acquire) {
                return Err(TrellisClientError::AuthorizationUnavailable(
                    "revocation coverage changed during resolution".into(),
                ));
            }
            state.contexts.insert(digest.to_owned(), entry);
        }
        Ok(verified)
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
}
