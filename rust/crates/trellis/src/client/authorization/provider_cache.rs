use std::collections::HashMap;
#[cfg(feature = "integration-test-scoping")]
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use futures_util::StreamExt;
use trellis_protocol::{
    parse_authorization_context_v1, parse_issuer_manifest_v1, verify_authorization_context_v1,
    verify_issuer_manifest_v1, AuthorizationTrustRootV1, AuthorizationVerificationPolicyV1,
    VerifiedAuthorizationContextV1, VerifiedAuthorizationIssuerManifestV1,
};

use super::super::TrellisClientError;
use super::own_context::AuthorizationContextCache;
#[cfg(test)]
use super::own_context::ProviderTrustInput;
use super::registry::{AuthorizationRegistryReader, ManifestPointer, RevocationWatchEntry};
use super::types::AuthorizationRegistryBinding;

#[cfg(feature = "runtime-internals")]
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct RuntimeAuthorizationTrust {
    pub root: AuthorizationTrustRootV1,
    pub policy: AuthorizationVerificationPolicyV1,
    pub minimum_manifest_generation: u64,
    pub minimum_manifest_digest: String,
    pub manifest: VerifiedAuthorizationIssuerManifestV1,
    pub manifest_digest: String,
}

/// Observable health for the manifest and revocation registry watches.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorizationProviderCacheHealth {
    pub(crate) manifest_revision: u64,
    pub(crate) revocation_revision: u64,
    pub(crate) last_update_at: i64,
    pub(crate) healthy: bool,
}

/// Registry I/O counters observed since provider-cache start.
#[allow(dead_code)] // exercised by local verifier tests and live integration hooks
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AuthorizationProviderIoCounters {
    /// Registry context reads performed by `resolve_context`.
    pub(crate) context_resolves: u64,
}

#[cfg(feature = "runtime-internals")]
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RuntimeAuthorizationIoCounters {
    pub context_resolves: u64,
}

/// Authorization-registry I/O counters exposed only to live integration tests.
#[cfg(feature = "integration-test-scoping")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IntegrationTestAuthorizationIoCounters {
    /// Exact context reads.
    pub context_gets: u64,
    /// Exact manifest and pointer reads.
    pub trust_gets: u64,
    /// Revocation watch initializations.
    pub revocation_watch_initializations: u64,
    /// Coalesced unknown-context resolutions.
    pub context_resolves: u64,
}

/// One in-flight coalesced context resolution lock shared by concurrent callers.
struct PendingContext {
    lock: tokio::sync::Mutex<()>,
}

/// Provider-side caller-context resolution and revocation state.
///
/// The provider cache resolves unknown caller context digests through the
/// connected NATS authorization registry, watches manifest advance and
/// revocation updates, and caches verified contexts until retention expiry.
/// Cache hits perform no HTTP, no Auth RPC, no SQLite, and no NATS I/O.
#[derive(Clone)]
pub struct AuthorizationProviderCache {
    nats: Option<async_nats::Client>,
    registry: Option<AuthorizationRegistryReader>,
    own: Option<Arc<AuthorizationContextCache>>,
    root: Arc<RwLock<Option<AuthorizationTrustRootV1>>>,
    policy_floor: Arc<RwLock<Option<(AuthorizationVerificationPolicyV1, u64, String)>>>,
    manifest: Arc<RwLock<Option<(VerifiedAuthorizationIssuerManifestV1, String)>>>,
    verified_contexts: Arc<RwLock<HashMap<String, VerifiedAuthorizationContextV1>>>,
    retention_deadlines: Arc<RwLock<HashMap<String, i64>>>,
    revocations: Arc<RwLock<HashMap<String, i64>>>,
    in_flight: Arc<Mutex<HashMap<String, Arc<PendingContext>>>>,
    context_resolves: Arc<AtomicU64>,
    #[cfg(feature = "integration-test-scoping")]
    fail_next_context_read: Arc<AtomicBool>,
    #[cfg(feature = "integration-test-scoping")]
    fail_next_readiness_check: Arc<AtomicBool>,
    health: Arc<RwLock<AuthorizationProviderCacheHealth>>,
    ready: Arc<tokio::sync::Notify>,
}

impl AuthorizationProviderCache {
    /// Attach a connected provider cache to the NATS authorization registry.
    pub(crate) async fn attach(
        nats: async_nats::Client,
        binding: &AuthorizationRegistryBinding,
        own: Arc<AuthorizationContextCache>,
    ) -> Result<Self, TrellisClientError> {
        let registry = AuthorizationRegistryReader::open(nats.clone(), binding).await?;
        let cache = Self {
            nats: Some(nats),
            registry: Some(registry),
            own: Some(own),
            root: Arc::new(RwLock::new(None)),
            policy_floor: Arc::new(RwLock::new(None)),
            manifest: Arc::new(RwLock::new(None)),
            verified_contexts: Arc::new(RwLock::new(HashMap::new())),
            retention_deadlines: Arc::new(RwLock::new(HashMap::new())),
            revocations: Arc::new(RwLock::new(HashMap::new())),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            context_resolves: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "integration-test-scoping")]
            fail_next_context_read: Arc::new(AtomicBool::new(false)),
            #[cfg(feature = "integration-test-scoping")]
            fail_next_readiness_check: Arc::new(AtomicBool::new(false)),
            health: Arc::new(RwLock::new(AuthorizationProviderCacheHealth {
                manifest_revision: 0,
                revocation_revision: 0,
                last_update_at: 0,
                healthy: false,
            })),
            ready: Arc::new(tokio::sync::Notify::new()),
        };
        cache.sync_trust_material()?;
        Ok(cache)
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn attach_runtime(
        nats: async_nats::Client,
        binding: &AuthorizationRegistryBinding,
        trust: RuntimeAuthorizationTrust,
    ) -> Result<Self, TrellisClientError> {
        let registry = AuthorizationRegistryReader::open(nats.clone(), binding).await?;
        Ok(Self {
            nats: Some(nats),
            registry: Some(registry),
            own: None,
            root: Arc::new(RwLock::new(Some(trust.root))),
            policy_floor: Arc::new(RwLock::new(Some((
                trust.policy,
                trust.minimum_manifest_generation,
                trust.minimum_manifest_digest,
            )))),
            manifest: Arc::new(RwLock::new(Some((trust.manifest, trust.manifest_digest)))),
            verified_contexts: Arc::new(RwLock::new(HashMap::new())),
            retention_deadlines: Arc::new(RwLock::new(HashMap::new())),
            revocations: Arc::new(RwLock::new(HashMap::new())),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            context_resolves: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "integration-test-scoping")]
            fail_next_context_read: Arc::new(AtomicBool::new(false)),
            #[cfg(feature = "integration-test-scoping")]
            fail_next_readiness_check: Arc::new(AtomicBool::new(false)),
            health: Arc::new(RwLock::new(AuthorizationProviderCacheHealth {
                manifest_revision: 0,
                revocation_revision: 0,
                last_update_at: 0,
                healthy: false,
            })),
            ready: Arc::new(tokio::sync::Notify::new()),
        })
    }

    /// Create an explicitly ready provider cache without a registry for unit tests.
    #[cfg(test)]
    pub(crate) fn new_for_test(
        own: Arc<AuthorizationContextCache>,
        input: ProviderTrustInput,
        root: AuthorizationTrustRootV1,
        manifest: VerifiedAuthorizationIssuerManifestV1,
        manifest_digest: String,
    ) -> Result<Self, TrellisClientError> {
        let policy = AuthorizationVerificationPolicyV1::new(
            own.corrected_now_seconds()?,
            input.policy.allowed_clock_skew_seconds,
            input.policy.maximum_context_lifetime_seconds,
            input.policy.maximum_context_bytes,
            input.policy.maximum_permissions,
            input.policy.maximum_capabilities,
            input.minimum_manifest_generation,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let cache = Self {
            nats: None,
            registry: None,
            own: Some(own),
            root: Arc::new(RwLock::new(Some(root))),
            policy_floor: Arc::new(RwLock::new(Some((
                policy,
                input.minimum_manifest_generation,
                manifest_digest.clone(),
            )))),
            manifest: Arc::new(RwLock::new(Some((manifest, manifest_digest)))),
            verified_contexts: Arc::new(RwLock::new(HashMap::new())),
            retention_deadlines: Arc::new(RwLock::new(HashMap::new())),
            revocations: Arc::new(RwLock::new(HashMap::new())),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            context_resolves: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "integration-test-scoping")]
            fail_next_context_read: Arc::new(AtomicBool::new(false)),
            #[cfg(feature = "integration-test-scoping")]
            fail_next_readiness_check: Arc::new(AtomicBool::new(false)),
            health: Arc::new(RwLock::new(AuthorizationProviderCacheHealth {
                manifest_revision: 0,
                revocation_revision: 0,
                last_update_at: 0,
                healthy: true,
            })),
            ready: Arc::new(tokio::sync::Notify::new()),
        };
        Ok(cache)
    }

    /// Refresh the pinned root and policy floor from the own-context installation.
    pub(crate) fn sync_trust_material(&self) -> Result<(), TrellisClientError> {
        let Some(own) = self.own.as_ref() else {
            return Ok(());
        };
        let input = own.provider_trust_input()?;
        let durable = own.durable_state()?;
        let floor_digest = if durable.as_ref().is_some_and(|state| {
            state.trust.minimum_manifest_generation == input.minimum_manifest_generation
        }) {
            durable
                .as_ref()
                .map(|state| state.trust.manifest_digest_at_minimum_generation.clone())
                .ok_or_else(|| {
                    TrellisClientError::Bootstrap("authorization trust floor is unavailable".into())
                })?
        } else {
            let manifest = parse_issuer_manifest_v1(&own.bundle()?.trust.manifest)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
            manifest
                .digest()
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
        };
        if floor_digest.is_empty() {
            return Err(TrellisClientError::Bootstrap(
                "authorization trust floor digest is empty".into(),
            ));
        }
        let root = AuthorizationTrustRootV1::parse(&input.root)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let root_digest = root
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if let Some(current) = self
            .root
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("provider trust lock poisoned".into()))?
            .as_ref()
        {
            if current.authority() != root.authority()
                || current.key_id() != root.key_id()
                || current
                    .digest()
                    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
                    != root_digest
            {
                return Err(TrellisClientError::Bootstrap(
                    "authorization trust root changed".into(),
                ));
            }
        }
        *self
            .root
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("provider trust lock poisoned".into()))? =
            Some(root);
        let mut policy_floor = self
            .policy_floor
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("provider trust lock poisoned".into()))?;
        if let Some((_, generation, digest)) = policy_floor.as_ref() {
            if input.minimum_manifest_generation < *generation {
                return Err(TrellisClientError::Bootstrap(
                    "authorization manifest floor rolled back".into(),
                ));
            }
            if input.minimum_manifest_generation == *generation && floor_digest != *digest {
                return Err(TrellisClientError::Bootstrap(
                    "authorization manifest floor equivocates".into(),
                ));
            }
        }
        let policy = AuthorizationVerificationPolicyV1::new(
            own.corrected_now_seconds()?,
            input.policy.allowed_clock_skew_seconds,
            input.policy.maximum_context_lifetime_seconds,
            input.policy.maximum_context_bytes,
            input.policy.maximum_permissions,
            input.policy.maximum_capabilities,
            input.minimum_manifest_generation,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        *policy_floor = Some((policy, input.minimum_manifest_generation, floor_digest));
        Ok(())
    }

    /// Run the manifest and revocation watch loop until `stop` closes.
    pub(crate) async fn run(
        &self,
        mut stop: tokio::sync::watch::Receiver<()>,
    ) -> Result<(), TrellisClientError> {
        loop {
            if stop.has_changed().is_err() {
                return Ok(());
            }
            self.set_healthy(false);
            match self.watch_once(&mut stop).await {
                Ok(()) => return Ok(()),
                Err(error) => {
                    tracing::warn!(%error, "authorization provider registry watch restarting");
                    tokio::select! {
                        _ = stop.changed() => return Ok(()),
                        () = tokio::time::sleep(Duration::from_secs(1)) => {}
                    }
                }
            }
        }
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn run_runtime(
        &self,
        stop: tokio::sync::watch::Receiver<()>,
    ) -> Result<(), TrellisClientError> {
        self.run(stop).await
    }

    /// Wait until the provider cache is ready or `stop` closes.
    #[allow(dead_code)] // exercised by live integration hooks
    pub(crate) async fn wait_ready(
        &self,
        mut stop: tokio::sync::watch::Receiver<()>,
    ) -> Result<(), TrellisClientError> {
        loop {
            // Register the notification before checking health so a readiness
            // transition racing the check cannot be missed.
            let notified = self.ready.notified();
            if self.health()?.healthy {
                return Ok(());
            }
            tokio::select! {
                _ = stop.changed() => {
                    return Err(TrellisClientError::Bootstrap(
                        "authorization provider stopped before readiness".into(),
                    ));
                }
                () = notified => {}
            }
        }
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn wait_until_ready(&self) -> Result<(), TrellisClientError> {
        loop {
            let notified = self.ready.notified();
            if self.health()?.healthy {
                return Ok(());
            }
            notified.await;
        }
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
        let counters = self.io_counters();
        RuntimeAuthorizationIoCounters {
            context_resolves: counters.context_resolves,
        }
    }

    pub(crate) fn health(&self) -> Result<AuthorizationProviderCacheHealth, TrellisClientError> {
        self.health
            .read()
            .map(|health| health.clone())
            .map_err(|_| TrellisClientError::Bootstrap("provider health lock poisoned".into()))
    }

    /// Memory-only verified-context read without revocation or expiry filtering.
    pub(crate) fn verified_context_raw(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContextV1>, TrellisClientError> {
        Ok(self
            .verified_contexts
            .read()
            .map_err(|_| {
                TrellisClientError::Bootstrap("provider context cache lock poisoned".into())
            })?
            .get(digest)
            .cloned())
    }

    fn active_context_raw(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContextV1>, TrellisClientError> {
        Ok(self
            .verified_contexts
            .read()
            .map_err(|_| {
                TrellisClientError::Bootstrap("provider context cache lock poisoned".into())
            })?
            .get(digest)
            .cloned())
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_verified_context_raw(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContextV1>, TrellisClientError> {
        self.verified_context_raw(digest)
    }

    /// Memory-only revocation timestamp for a digest, when revoked.
    pub(crate) fn revocation_time(&self, digest: &str) -> Result<Option<i64>, TrellisClientError> {
        Ok(self
            .revocations
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("provider revocation lock poisoned".into()))?
            .get(digest)
            .copied())
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_revocation_time(&self, digest: &str) -> Result<Option<i64>, TrellisClientError> {
        self.revocation_time(digest)
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn apply_runtime_revocation(
        &self,
        digest: &str,
        revoked_at: i64,
    ) -> Result<(), TrellisClientError> {
        let mut revocations = self.revocations.write().map_err(|_| {
            TrellisClientError::Bootstrap("provider revocation lock poisoned".into())
        })?;
        revocations
            .entry(digest.to_owned())
            .and_modify(|current| *current = (*current).max(revoked_at))
            .or_insert(revoked_at);
        drop(revocations);
        if let Some(own) = self.own.as_ref() {
            if own.context_digest().is_ok_and(|current| current == digest) {
                own.clear()?;
                own.request_refresh();
            }
        }
        Ok(())
    }

    /// Return the verification policy bound to the current trust material.
    pub(crate) fn policy(&self) -> Result<AuthorizationVerificationPolicyV1, TrellisClientError> {
        let policy_floor = self
            .policy_floor
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("provider trust lock poisoned".into()))?
            .clone()
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization context unavailable".into())
            })?;
        let mut policy = policy_floor.0;
        policy.now_unix_seconds = self.now_seconds()?;
        policy.minimum_manifest_generation = policy.minimum_manifest_generation.max(policy_floor.1);
        Ok(policy)
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_policy(&self) -> Result<AuthorizationVerificationPolicyV1, TrellisClientError> {
        self.policy()
    }

    /// Return registry I/O counters observed since startup.
    #[allow(dead_code)] // exercised by local verifier tests and live integration hooks
    pub(crate) fn io_counters(&self) -> AuthorizationProviderIoCounters {
        AuthorizationProviderIoCounters {
            context_resolves: self.context_resolves.load(Ordering::Relaxed),
        }
    }

    /// Return exact provider I/O counters for live integration assertions.
    #[cfg(feature = "integration-test-scoping")]
    #[must_use]
    pub fn integration_test_io_counters(&self) -> IntegrationTestAuthorizationIoCounters {
        let registry = self
            .registry
            .as_ref()
            .map_or_else(Default::default, AuthorizationRegistryReader::io_counters);
        let provider = self.io_counters();
        IntegrationTestAuthorizationIoCounters {
            context_gets: registry.context_gets,
            trust_gets: registry.trust_gets,
            revocation_watch_initializations: registry.revocation_watch_initializations,
            context_resolves: provider.context_resolves,
        }
    }

    /// Return whether the provider cache is ready, for live integration
    /// assertions on the watch lifecycle.
    #[cfg(feature = "integration-test-scoping")]
    #[doc(hidden)]
    #[must_use]
    pub fn integration_test_provider_ready(&self) -> bool {
        self.health().map(|health| health.healthy).unwrap_or(false)
    }

    /// Fail the next exact context registry read for live redelivery coverage.
    #[cfg(feature = "integration-test-scoping")]
    #[doc(hidden)]
    pub fn integration_test_fail_next_context_read(&self) {
        self.fail_next_context_read.store(true, Ordering::Relaxed);
    }

    /// Fail the next provider readiness check for live redelivery coverage.
    #[cfg(feature = "integration-test-scoping")]
    #[doc(hidden)]
    pub fn integration_test_fail_next_readiness_check(&self) {
        self.fail_next_readiness_check
            .store(true, Ordering::Relaxed);
    }

    #[cfg(feature = "integration-test-scoping")]
    pub(crate) fn integration_test_take_readiness_failure(&self) -> bool {
        self.fail_next_readiness_check
            .swap(false, Ordering::Relaxed)
    }

    fn set_healthy(&self, healthy: bool) {
        if let Ok(mut health) = self.health.write() {
            health.healthy = healthy;
        }
    }

    fn record_healthy(&self) {
        if let Ok(mut health) = self.health.write() {
            health.last_update_at = self.now_seconds().unwrap_or(0);
            health.healthy = true;
        }
        self.ready.notify_waiters();
    }

    async fn watch_once(
        &self,
        stop: &mut tokio::sync::watch::Receiver<()>,
    ) -> Result<(), TrellisClientError> {
        let Some(registry) = self.registry.as_ref() else {
            return Ok(());
        };
        self.sync_trust_material()?;
        // Subscribe before reading the pointer so racing updates remain queued.
        let mut manifests = registry.watch_manifest_current().await?;
        let mut revocations = registry.watch_revocations().await?;
        self.initialize_watch_state(registry, &mut revocations)
            .await?;
        let mut was_connected = true;
        let mut connection_check = tokio::time::interval(Duration::from_millis(100));
        connection_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = stop.changed() => return Ok(()),
                _ = connection_check.tick(), if self.nats.is_some() => {
                    let connected = self.nats.as_ref().is_some_and(|nats| {
                        nats.connection_state() == async_nats::connection::State::Connected
                    });
                    if !connected {
                        if was_connected {
                            if let Some(own) = self.own.as_ref() {
                                own.request_refresh();
                            }
                        }
                        was_connected = false;
                        self.set_healthy(false);
                    } else if !was_connected {
                        return Err(TrellisClientError::Bootstrap(
                            "NATS reconnected; restarting authorization watches".into(),
                        ));
                    }
                }
                entry = manifests.next() => {
                    let entry = entry.ok_or_else(|| {
                        TrellisClientError::Bootstrap("manifest.current watch ended".into())
                    })?.map_err(|error| {
                        TrellisClientError::Bootstrap(format!(
                            "manifest.current watch failed: {error}"
                        ))
                    })?;
                    let pointer = manifest_pointer_from_entry(&entry)?;
                    self.observe_manifest(&pointer, entry.revision).await?;
                }
                entry = revocations.next() => {
                    let entry = entry.ok_or_else(|| {
                        TrellisClientError::Bootstrap("revocation watch ended".into())
                    })?.map_err(|error| {
                        TrellisClientError::Bootstrap(format!(
                            "revocation watch failed: {error}"
                        ))
                    })?;
                    match super::registry::revocation_entry(entry) {
                        RevocationWatchEntry::Applied { key, value, revision } => {
                            self.observe_revocation_value(&key, &value, revision)?;
                        }
                        RevocationWatchEntry::Removed { key, revision } => {
                            self.observe_revocation_removal(&key, revision)?;
                        }
                    }
                }
            }
        }
    }

    /// Initialize current state from the exact pointer and the history-bearing
    /// revocation watch that remains active for live updates.
    async fn initialize_watch_state(
        &self,
        registry: &AuthorizationRegistryReader,
        revocations: &mut super::registry::RegistryWatch,
    ) -> Result<(), TrellisClientError> {
        let (manifest, revision) = registry
            .get_manifest_current()
            .await?
            .ok_or_else(|| TrellisClientError::Bootstrap("manifest.current is missing".into()))?;
        self.observe_manifest(&manifest, revision).await?;
        if revocations.initially_empty() {
            registry.record_revocation_watch_initialization();
            self.record_healthy();
            return Ok(());
        }
        loop {
            let entry = revocations.next().await.ok_or_else(|| {
                TrellisClientError::Bootstrap("initial revocation watch ended".into())
            })?;
            let entry = entry.map_err(|error| {
                TrellisClientError::Bootstrap(format!("initial revocation watch failed: {error}"))
            })?;
            let seen_current = entry.seen_current;
            match super::registry::revocation_entry(entry) {
                RevocationWatchEntry::Applied {
                    key,
                    value,
                    revision,
                } => {
                    self.observe_revocation_value(&key, &value, revision)?;
                }
                RevocationWatchEntry::Removed { key, revision } => {
                    self.observe_revocation_removal(&key, revision)?;
                }
            }
            if seen_current {
                break;
            }
        }
        registry.record_revocation_watch_initialization();
        self.record_healthy();
        Ok(())
    }

    async fn observe_manifest(
        &self,
        pointer: &ManifestPointer,
        revision: u64,
    ) -> Result<(), TrellisClientError> {
        if self.health()?.manifest_revision >= revision {
            return Ok(());
        }
        let was_healthy = self.health()?.healthy;
        let (current_generation, current_digest) = {
            let manifest = self.manifest.read().map_err(|_| {
                TrellisClientError::Bootstrap("provider manifest lock poisoned".into())
            })?;
            manifest
                .as_ref()
                .map(|(manifest, digest)| (manifest.generation(), digest.clone()))
                .unwrap_or((0, String::new()))
        };
        self.check_manifest_floor(pointer.generation, &pointer.digest)?;
        let mut advanced = false;
        let initial_snapshot = current_generation == 0;
        if pointer.generation < current_generation {
            return Err(TrellisClientError::Bootstrap(
                "manifest.current rolled back".into(),
            ));
        }
        if pointer.generation == current_generation && pointer.digest != current_digest {
            return Err(TrellisClientError::Bootstrap(
                "manifest.current equivocates at the accepted generation".into(),
            ));
        }
        if pointer.generation > current_generation {
            self.set_healthy(false);
            let Some(registry) = self.registry.as_ref() else {
                return Err(TrellisClientError::Bootstrap(
                    "authorization registry is unavailable".into(),
                ));
            };
            let key = format!("manifest.{}", pointer.generation);
            let value = registry
                .get_manifest(pointer.generation)
                .await?
                .ok_or_else(|| {
                    TrellisClientError::Bootstrap(format!("issuer manifest {key} is missing"))
                })?;
            let json: serde_json::Value = serde_json::from_slice(&value)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
            let manifest = parse_issuer_manifest_v1(&json)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
            let digest = manifest
                .digest()
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
            if manifest.unsigned.generation != pointer.generation || digest != pointer.digest {
                return Err(TrellisClientError::Bootstrap(
                    "manifest.current does not match its immutable manifest".into(),
                ));
            }
            let mut policy = self.policy()?;
            policy.now_unix_seconds = self.now_seconds()?;
            policy.minimum_manifest_generation =
                policy.minimum_manifest_generation.max(current_generation);
            let verified = verify_issuer_manifest_v1(&self.root_value()?, &manifest, &policy)
                .map_err(|error| {
                    TrellisClientError::Bootstrap(format!(
                        "issuer manifest {key} is not trusted: {error}"
                    ))
                })?;
            if let Some(own) = self.own.as_ref() {
                own.advance_manifest_floor(pointer.generation, &digest)
                    .await?;
            }
            {
                let mut floor = self.policy_floor.write().map_err(|_| {
                    TrellisClientError::Bootstrap("provider trust lock poisoned".into())
                })?;
                let (mut policy, _, _) = floor.clone().ok_or_else(|| {
                    TrellisClientError::Bootstrap("authorization context unavailable".into())
                })?;
                policy.minimum_manifest_generation = pointer.generation;
                *floor = Some((policy, pointer.generation, digest.clone()));
            }
            *self.manifest.write().map_err(|_| {
                TrellisClientError::Bootstrap("provider manifest lock poisoned".into())
            })? = Some((verified, digest));
            self.verified_contexts
                .write()
                .map_err(|_| {
                    TrellisClientError::Bootstrap("provider context cache lock poisoned".into())
                })?
                .clear();
            advanced = true;
            if let Some(own) = self.own.as_ref() {
                own.request_refresh();
            }
        }
        {
            let mut health = self.health.write().map_err(|_| {
                TrellisClientError::Bootstrap("provider health lock poisoned".into())
            })?;
            health.manifest_revision = health.manifest_revision.max(revision);
            health.last_update_at = self.now_seconds().unwrap_or(0);
            health.healthy = was_healthy && !advanced;
        }
        if advanced && !initial_snapshot {
            return Err(TrellisClientError::Bootstrap(
                "issuer manifest advanced; restarting complete provider snapshot".into(),
            ));
        }
        Ok(())
    }

    fn observe_revocation_value(
        &self,
        key: &str,
        value: &[u8],
        revision: u64,
    ) -> Result<(), TrellisClientError> {
        let digest = key
            .strip_prefix(super::registry::REVOCATION_PREFIX)
            .filter(|digest| !digest.is_empty())
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization revocation key is invalid".into())
            })?;
        let revoked_at = parse_revocation_record(value)?;
        let mut revocations = self.revocations.write().map_err(|_| {
            TrellisClientError::Bootstrap("provider revocation lock poisoned".into())
        })?;
        revocations
            .entry(digest.to_owned())
            .and_modify(|current| *current = (*current).max(revoked_at))
            .or_insert(revoked_at);
        drop(revocations);
        if let Some(own) = self.own.as_ref() {
            if own.context_digest().is_ok_and(|current| current == digest) {
                own.clear()?;
                own.request_refresh();
            }
        }
        if let Ok(mut health) = self.health.write() {
            health.revocation_revision = health.revocation_revision.max(revision);
            health.last_update_at = self.now_seconds().unwrap_or(0);
        }
        Ok(())
    }

    fn observe_revocation_removal(
        &self,
        key: &str,
        revision: u64,
    ) -> Result<(), TrellisClientError> {
        let digest = key
            .strip_prefix(super::registry::REVOCATION_PREFIX)
            .filter(|digest| !digest.is_empty())
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization revocation key is invalid".into())
            })?;
        // Revocation is monotonic: registry cleanup must never make a revoked context valid again.
        let _ = digest;
        if let Ok(mut health) = self.health.write() {
            health.revocation_revision = health.revocation_revision.max(revision);
            health.last_update_at = self.now_seconds().unwrap_or(0);
        }
        Ok(())
    }

    fn prune_contexts(&self, now: i64) -> Result<(), TrellisClientError> {
        let expired = {
            let mut deadlines = self.retention_deadlines.write().map_err(|_| {
                TrellisClientError::Bootstrap("provider retention lock poisoned".into())
            })?;
            let expired = deadlines
                .iter()
                .filter(|(_, retained_until)| **retained_until <= now)
                .map(|(digest, _)| digest.clone())
                .collect::<Vec<_>>();
            for digest in &expired {
                deadlines.remove(digest);
            }
            expired
        };
        if expired.is_empty() {
            return Ok(());
        }
        let mut contexts = self.verified_contexts.write().map_err(|_| {
            TrellisClientError::Bootstrap("provider context cache lock poisoned".into())
        })?;
        for digest in &expired {
            contexts.remove(digest);
        }
        drop(contexts);
        Ok(())
    }

    fn now_seconds(&self) -> Result<i64, TrellisClientError> {
        if let Some(own) = self.own.as_ref() {
            return own.corrected_now_seconds();
        }
        i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
                .as_secs(),
        )
        .map_err(|_| TrellisClientError::Bootstrap("provider time overflow".into()))
    }

    fn root_value(&self) -> Result<AuthorizationTrustRootV1, TrellisClientError> {
        self.root
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("provider trust lock poisoned".into()))?
            .clone()
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization context unavailable".into())
            })
    }

    fn check_manifest_floor(
        &self,
        generation: u64,
        digest: &str,
    ) -> Result<(), TrellisClientError> {
        let policy_floor = self
            .policy_floor
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("provider trust lock poisoned".into()))?;
        let Some((_, minimum_generation, minimum_digest)) = policy_floor.as_ref() else {
            return Err(TrellisClientError::Bootstrap(
                "authorization context unavailable".into(),
            ));
        };
        if generation < *minimum_generation {
            return Err(TrellisClientError::Bootstrap(
                "manifest is below the durable authorization floor".into(),
            ));
        }
        if generation == *minimum_generation && digest != minimum_digest {
            return Err(TrellisClientError::Bootstrap(
                "manifest equivocates with the durable authorization floor".into(),
            ));
        }
        Ok(())
    }

    /// Resolve an unknown context digest from the registry at most once per
    /// digest, coalescing concurrent callers onto one registry read.
    pub(crate) async fn resolve_context(
        &self,
        digest: &str,
        now: i64,
    ) -> Result<VerifiedAuthorizationContextV1, TrellisClientError> {
        self.resolve_context_for(digest, now, false).await
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn resolve_admission_context(
        &self,
        digest: &str,
        now: i64,
    ) -> Result<VerifiedAuthorizationContextV1, TrellisClientError> {
        if !self.health()?.healthy || self.revocation_time(digest)?.is_some() {
            return Err(TrellisClientError::Bootstrap(
                "authorization context is not admissible".into(),
            ));
        }
        let context = self.resolve_context(digest, now).await?;
        if context.context_digest() != digest
            || context.not_before() > now
            || context.expires_at() <= now
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization context is not admissible".into(),
            ));
        }
        Ok(context)
    }

    /// Resolve an unknown context against its retained historical trust evidence.
    pub(crate) async fn resolve_event_context(
        &self,
        digest: &str,
        event_time: i64,
    ) -> Result<VerifiedAuthorizationContextV1, TrellisClientError> {
        self.resolve_context_for(digest, event_time, true).await
    }

    pub(crate) async fn resolve_event_context_for_verification(
        &self,
        digest: &str,
        event_time: i64,
    ) -> Result<VerifiedAuthorizationContextV1, crate::service::EventVerificationFailure> {
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
    ) -> Result<VerifiedAuthorizationContextV1, TrellisClientError> {
        self.resolve_event_context(digest, event_time).await
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub async fn runtime_resolve_event_context_for_verification(
        &self,
        digest: &str,
        event_time: i64,
    ) -> Result<VerifiedAuthorizationContextV1, crate::service::EventVerificationFailure> {
        self.resolve_event_context_for_verification(digest, event_time)
            .await
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_current_manifest(
        &self,
    ) -> Result<VerifiedAuthorizationIssuerManifestV1, TrellisClientError> {
        self.manifest
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("provider manifest lock poisoned".into()))?
            .as_ref()
            .map(|(manifest, _)| manifest.clone())
            .ok_or_else(|| TrellisClientError::Bootstrap("provider manifest unavailable".into()))
    }

    async fn resolve_context_for(
        &self,
        digest: &str,
        verification_time: i64,
        historical: bool,
    ) -> Result<VerifiedAuthorizationContextV1, TrellisClientError> {
        self.prune_contexts(self.now_seconds()?)?;
        let known = self.active_context_raw(digest)?;
        if let Some(context) = known {
            return Ok(context);
        }
        let manifest_generation = self
            .manifest
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("provider manifest lock poisoned".into()))?
            .as_ref()
            .map(|(manifest, _)| manifest.generation())
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization manifest is unavailable".into())
            })?;
        let pending = {
            let mut in_flight = self.in_flight.lock().map_err(|_| {
                TrellisClientError::Bootstrap("provider resolution lock poisoned".into())
            })?;
            Arc::clone(in_flight.entry(digest.to_owned()).or_insert_with(|| {
                Arc::new(PendingContext {
                    lock: tokio::sync::Mutex::new(()),
                })
            }))
        };
        let _guard = pending.lock.lock().await;
        // A concurrent caller may have resolved the digest while we waited.
        let known = self.active_context_raw(digest)?;
        if let Some(context) = known {
            return Ok(context);
        }
        let outcome = self
            .resolve_context_once(digest, verification_time, historical)
            .await;
        self.drop_in_flight(digest, &pending);
        let context = outcome?;
        let manifest = self
            .manifest
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("provider manifest lock poisoned".into()))?;
        if manifest.as_ref().map(|(manifest, _)| manifest.generation()) != Some(manifest_generation)
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization manifest advanced during context resolution".into(),
            ));
        }
        if !historical {
            let retained_until = context.expires_at();
            self.retention_deadlines
                .write()
                .map_err(|_| {
                    TrellisClientError::Bootstrap("provider retention lock poisoned".into())
                })?
                .insert(digest.to_owned(), retained_until);
            let mut active = self.verified_contexts.write().map_err(|_| {
                TrellisClientError::Bootstrap("provider context cache lock poisoned".into())
            })?;
            active.insert(digest.to_owned(), context.clone());
        }
        drop(manifest);
        Ok(context)
    }

    fn drop_in_flight(&self, digest: &str, pending: &Arc<PendingContext>) {
        let Ok(mut in_flight) = self.in_flight.lock() else {
            return;
        };
        if in_flight
            .get(digest)
            .is_some_and(|entry| Arc::ptr_eq(entry, pending))
        {
            in_flight.remove(digest);
        }
    }

    async fn resolve_context_once(
        &self,
        digest: &str,
        verification_time: i64,
        historical: bool,
    ) -> Result<VerifiedAuthorizationContextV1, TrellisClientError> {
        let Some(registry) = self.registry.as_ref() else {
            return Err(TrellisClientError::Bootstrap(
                "authorization registry is unavailable".into(),
            ));
        };
        self.sync_trust_material()?;
        self.context_resolves.fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "integration-test-scoping")]
        if self.fail_next_context_read.swap(false, Ordering::Relaxed) {
            return Err(TrellisClientError::NatsRequest(
                "injected authorization context read failure".into(),
            ));
        }
        let value = registry.get_context(digest).await?.ok_or_else(|| {
            TrellisClientError::Bootstrap(
                "authorization context is missing from the registry".into(),
            )
        })?;
        let json: serde_json::Value = serde_json::from_slice(&value)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let context = parse_authorization_context_v1(&json)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if context
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
            != digest
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization context digest does not match its registry key".into(),
            ));
        }
        let generation = context.unsigned.issuer_manifest_generation;
        if !historical {
            let current_generation = self
                .manifest
                .read()
                .map_err(|_| {
                    TrellisClientError::Bootstrap("provider manifest lock poisoned".into())
                })?
                .as_ref()
                .map(|(manifest, _)| manifest.generation())
                .ok_or_else(|| {
                    TrellisClientError::Bootstrap("provider manifest unavailable".into())
                })?;
            if generation != current_generation {
                return Err(TrellisClientError::Bootstrap(
                    "authorization context manifest is not current".into(),
                ));
            }
        }
        let mut policy = self.policy()?;
        policy.now_unix_seconds = verification_time;
        if historical {
            policy.minimum_manifest_generation = generation;
        } else {
            policy.minimum_manifest_generation = policy.minimum_manifest_generation.max(generation);
        }
        let manifest = self
            .resolve_manifest(
                registry,
                &context.unsigned.issuer_key_id,
                generation,
                historical,
                &mut policy,
            )
            .await?;
        verify_authorization_context_v1(&self.root_value()?, &manifest, &context, &policy).map_err(
            |error| {
                TrellisClientError::Bootstrap(format!(
                    "authorization context is not trusted: {error}"
                ))
            },
        )
    }

    async fn resolve_manifest(
        &self,
        registry: &AuthorizationRegistryReader,
        issuer_key_id: &str,
        generation: u64,
        historical: bool,
        policy: &mut AuthorizationVerificationPolicyV1,
    ) -> Result<VerifiedAuthorizationIssuerManifestV1, TrellisClientError> {
        if let Some((manifest, digest)) = self
            .manifest
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("provider manifest lock poisoned".into()))?
            .as_ref()
            .filter(|(manifest, _)| manifest.generation() == generation)
            .cloned()
        {
            if !historical {
                self.check_manifest_floor(generation, &digest)?;
            }
            if manifest
                .manifest()
                .unsigned
                .issuers
                .iter()
                .any(|issuer| issuer.key_id == issuer_key_id)
            {
                return Ok(manifest);
            }
        }

        let manifest_value = registry.get_manifest(generation).await?.ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization context issuer manifest is missing".into())
        })?;
        let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_value)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let manifest = parse_issuer_manifest_v1(&manifest_json)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let digest = manifest
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if manifest.unsigned.generation != generation {
            return Err(TrellisClientError::Bootstrap(
                "issuer manifest generation does not match context".into(),
            ));
        }
        if !historical {
            self.check_manifest_floor(generation, &digest)?;
        }
        let manifest = verify_issuer_manifest_v1(&self.root_value()?, &manifest, policy)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if !manifest
            .manifest()
            .unsigned
            .issuers
            .iter()
            .any(|issuer| issuer.key_id == issuer_key_id)
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization context issuer is absent from its manifest".into(),
            ));
        }
        Ok(manifest)
    }

    /// Install an already-verified snapshot for unit tests without registry I/O.
    #[cfg(test)]
    pub(crate) fn inject_verified_for_test(
        &self,
        digest: &str,
        verified: VerifiedAuthorizationContextV1,
        revoked_at: Option<i64>,
    ) -> Result<(), TrellisClientError> {
        self.verified_contexts
            .write()
            .map_err(|_| {
                TrellisClientError::Bootstrap("provider context cache lock poisoned".into())
            })?
            .insert(digest.to_owned(), verified);
        if let Some(revoked_at) = revoked_at {
            self.revocations
                .write()
                .map_err(|_| {
                    TrellisClientError::Bootstrap("provider revocation lock poisoned".into())
                })?
                .insert(digest.to_owned(), revoked_at);
        }
        Ok(())
    }
}

fn classify_event_resolution_failure(
    error: TrellisClientError,
) -> crate::service::EventVerificationFailure {
    let retryable = matches!(
        error,
        TrellisClientError::Io(_)
            | TrellisClientError::Nats(_)
            | TrellisClientError::NatsConnect(_)
            | TrellisClientError::NatsRequest(_)
            | TrellisClientError::Timeout
    ) || matches!(&error, TrellisClientError::Bootstrap(message) if [
        "authorization registry is unavailable",
        "cannot read authorization context",
        "authorization context is missing from the registry",
        "cannot read issuer manifest",
        "authorization context issuer manifest is missing",
        "authorization manifest advanced during context resolution",
        "provider trust lock poisoned",
        "provider manifest lock poisoned",
        "provider context cache lock poisoned",
        "provider resolution lock poisoned",
        "provider retention lock poisoned",
        "context store lock poisoned",
        "authorization trust floor unavailable",
        "authorization trust floor digest is empty",
        "authorization context unavailable",
        "provider time overflow",
    ].iter().any(|prefix| message.starts_with(prefix)));
    if retryable {
        crate::service::EventVerificationFailure::retryable(error.to_string())
    } else {
        crate::service::EventVerificationFailure::rejected(error.to_string())
    }
}

/// Parse and validate a revocation record.
fn parse_revocation_record(value: &[u8]) -> Result<i64, TrellisClientError> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Record {
        revoked_at: i64,
    }
    let record: Record = serde_json::from_slice(value)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    if record.revoked_at <= 0 {
        return Err(TrellisClientError::Bootstrap(
            "authorization context revocation is invalid".into(),
        ));
    }
    Ok(record.revoked_at)
}

/// Extract a manifest pointer from a `manifest.current` watch entry.
fn manifest_pointer_from_entry(
    entry: &super::registry::RegistryWatchEntry,
) -> Result<ManifestPointer, TrellisClientError> {
    super::registry::parse_api_authoring_source_pointer(&entry.value)
}

#[cfg(test)]
mod wire_tests {
    use super::parse_revocation_record;

    #[test]
    fn revocation_is_additively_tolerant() {
        assert_eq!(
            parse_revocation_record(br#"{"revokedAt":123}"#).expect("exact revocation"),
            123
        );
        assert_eq!(
            parse_revocation_record(br#"{"revokedAt":123,"future":true}"#)
                .expect("extended revocation"),
            123
        );
        assert!(parse_revocation_record(br#"{"revokedAt":0,"future":true}"#).is_err());
    }
}
