use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use futures_util::StreamExt;
use trellis_protocol::{
    parse_authorization_context, parse_issuer_manifest, verify_authorization_context,
    verify_issuer_manifest, AuthorizationTrustRoot, AuthorizationVerificationPolicy,
    VerifiedAuthorizationContext, VerifiedAuthorizationIssuerManifest,
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
    pub root: AuthorizationTrustRoot,
    pub policy: AuthorizationVerificationPolicy,
    pub minimum_manifest_generation: u64,
    pub minimum_manifest_digest: String,
    pub manifest: VerifiedAuthorizationIssuerManifest,
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
#[cfg(any(test, feature = "runtime-internals", feature = "test-support"))]
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
#[cfg(feature = "test-support")]
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

struct AuthorizationProviderState {
    root: Option<AuthorizationTrustRoot>,
    policy_floor: Option<(AuthorizationVerificationPolicy, u64, String)>,
    manifest: Option<(VerifiedAuthorizationIssuerManifest, String)>,
    verified_contexts: HashMap<String, VerifiedAuthorizationContext>,
    retention_deadlines: HashMap<String, i64>,
    revocations: HashMap<String, i64>,
    health: AuthorizationProviderCacheHealth,
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
    state: Arc<RwLock<AuthorizationProviderState>>,
    in_flight: Arc<Mutex<HashMap<String, Arc<PendingContext>>>>,
    context_resolves: Arc<AtomicU64>,
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
            state: Arc::new(RwLock::new(AuthorizationProviderState {
                root: None,
                policy_floor: None,
                manifest: None,
                verified_contexts: HashMap::new(),
                retention_deadlines: HashMap::new(),
                revocations: HashMap::new(),
                health: AuthorizationProviderCacheHealth {
                    manifest_revision: 0,
                    revocation_revision: 0,
                    last_update_at: 0,
                    healthy: false,
                },
            })),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            context_resolves: Arc::new(AtomicU64::new(0)),
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
            state: Arc::new(RwLock::new(AuthorizationProviderState {
                root: Some(trust.root),
                policy_floor: Some((
                    trust.policy,
                    trust.minimum_manifest_generation,
                    trust.minimum_manifest_digest,
                )),
                manifest: Some((trust.manifest, trust.manifest_digest)),
                verified_contexts: HashMap::new(),
                retention_deadlines: HashMap::new(),
                revocations: HashMap::new(),
                health: AuthorizationProviderCacheHealth {
                    manifest_revision: 0,
                    revocation_revision: 0,
                    last_update_at: 0,
                    healthy: false,
                },
            })),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            context_resolves: Arc::new(AtomicU64::new(0)),
            ready: Arc::new(tokio::sync::Notify::new()),
        })
    }

    /// Create an explicitly ready provider cache without a registry for unit tests.
    #[cfg(test)]
    pub(crate) fn new_for_test(
        own: Arc<AuthorizationContextCache>,
        input: ProviderTrustInput,
        root: AuthorizationTrustRoot,
        manifest: VerifiedAuthorizationIssuerManifest,
        manifest_digest: String,
    ) -> Result<Self, TrellisClientError> {
        let policy = AuthorizationVerificationPolicy::new(
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
            state: Arc::new(RwLock::new(AuthorizationProviderState {
                root: Some(root),
                policy_floor: Some((
                    policy,
                    input.minimum_manifest_generation,
                    manifest_digest.clone(),
                )),
                manifest: Some((manifest, manifest_digest)),
                verified_contexts: HashMap::new(),
                retention_deadlines: HashMap::new(),
                revocations: HashMap::new(),
                health: AuthorizationProviderCacheHealth {
                    manifest_revision: 0,
                    revocation_revision: 0,
                    last_update_at: 0,
                    healthy: true,
                },
            })),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            context_resolves: Arc::new(AtomicU64::new(0)),
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
            let manifest = parse_issuer_manifest(&own.bundle()?.trust.manifest)
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
        let root = AuthorizationTrustRoot::parse(&input.root)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let root_digest = root
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let policy = AuthorizationVerificationPolicy::new(
            own.corrected_now_seconds()?,
            input.policy.allowed_clock_skew_seconds,
            input.policy.maximum_context_lifetime_seconds,
            input.policy.maximum_context_bytes,
            input.policy.maximum_permissions,
            input.policy.maximum_capabilities,
            input.minimum_manifest_generation,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;

        let mut state = self.write_state()?;
        if let Some(current) = state.root.as_ref() {
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
        if let Some((_, generation, digest)) = state.policy_floor.as_ref() {
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
        state.root = Some(root);
        state.policy_floor = Some((policy, input.minimum_manifest_generation, floor_digest));
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
        Ok(self.read_state()?.health.clone())
    }

    /// Memory-only verified-context read without revocation or expiry filtering.
    pub(crate) fn verified_context_raw(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContext>, TrellisClientError> {
        Ok(self.read_state()?.verified_contexts.get(digest).cloned())
    }

    fn active_context_raw(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContext>, TrellisClientError> {
        Ok(self.read_state()?.verified_contexts.get(digest).cloned())
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_verified_context_raw(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContext>, TrellisClientError> {
        self.verified_context_raw(digest)
    }

    /// Memory-only revocation timestamp for a digest, when revoked.
    pub(crate) fn revocation_time(&self, digest: &str) -> Result<Option<i64>, TrellisClientError> {
        Ok(self.read_state()?.revocations.get(digest).copied())
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
        {
            let mut state = self.write_state()?;
            state
                .revocations
                .entry(digest.to_owned())
                .and_modify(|current| *current = (*current).max(revoked_at))
                .or_insert(revoked_at);
        }
        if let Some(own) = self.own.as_ref() {
            if own.context_digest().is_ok_and(|current| current == digest) {
                own.clear()?;
                own.request_refresh();
            }
        }
        Ok(())
    }

    /// Return the verification policy bound to the current trust material.
    pub(crate) fn policy(&self) -> Result<AuthorizationVerificationPolicy, TrellisClientError> {
        let policy_floor = self.read_state()?.policy_floor.clone().ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization context unavailable".into())
        })?;
        let mut policy = policy_floor.0;
        policy.now_unix_seconds = self.now_seconds()?;
        policy.minimum_manifest_generation = policy.minimum_manifest_generation.max(policy_floor.1);
        Ok(policy)
    }

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_policy(&self) -> Result<AuthorizationVerificationPolicy, TrellisClientError> {
        self.policy()
    }

    /// Return registry I/O counters observed since startup.
    #[cfg(any(test, feature = "runtime-internals", feature = "test-support"))]
    pub(crate) fn io_counters(&self) -> AuthorizationProviderIoCounters {
        AuthorizationProviderIoCounters {
            context_resolves: self.context_resolves.load(Ordering::Relaxed),
        }
    }

    /// Return exact provider I/O counters for live integration assertions.
    #[cfg(feature = "test-support")]
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
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    #[must_use]
    pub fn integration_test_provider_ready(&self) -> bool {
        self.health().map(|health| health.healthy).unwrap_or(false)
    }

    fn set_healthy(&self, healthy: bool) {
        if let Ok(mut state) = self.state.write() {
            state.health.healthy = healthy;
        }
    }

    fn record_healthy(&self) {
        let now = self.now_seconds().unwrap_or(0);
        if let Ok(mut state) = self.state.write() {
            state.health.last_update_at = now;
            state.health.healthy = true;
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
        let mut revocations = if let Some(own) = self.own.as_ref() {
            let digest = own.context_digest()?;
            registry.record_revocation_watch_initialization();
            Some(registry.watch_revocation(&digest).await?)
        } else {
            None
        };
        self.initialize_watch_state(registry, revocations.as_mut())
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
                entry = async {
                    match revocations.as_mut() {
                        Some(watch) => watch.next().await,
                        None => std::future::pending().await,
                    }
                } => {
                    let entry = entry.ok_or_else(|| TrellisClientError::Bootstrap(
                        "authorization revocation watch ended".into(),
                    ))??;
                    self.observe_revocation_entry(entry)?;
                }
            }
        }
    }

    /// Initialize current trust state from the exact manifest pointer.
    async fn initialize_watch_state(
        &self,
        registry: &AuthorizationRegistryReader,
        revocations: Option<&mut super::registry::RegistryWatch>,
    ) -> Result<(), TrellisClientError> {
        let (manifest, revision) = registry
            .get_manifest_current()
            .await?
            .ok_or_else(|| TrellisClientError::Bootstrap("manifest.current is missing".into()))?;
        self.observe_manifest(&manifest, revision).await?;
        if let Some(revocations) = revocations {
            if !revocations.initially_empty() {
                let entry = revocations.next().await.ok_or_else(|| {
                    TrellisClientError::Bootstrap(
                        "authorization revocation watch ended during initialization".into(),
                    )
                })??;
                self.observe_revocation_entry(entry)?;
            }
        }
        self.record_healthy();
        Ok(())
    }

    async fn observe_manifest(
        &self,
        pointer: &ManifestPointer,
        revision: u64,
    ) -> Result<(), TrellisClientError> {
        let (manifest_revision, was_healthy, current_generation, current_digest) = {
            let state = self.read_state()?;
            let (generation, digest) = state
                .manifest
                .as_ref()
                .map(|(manifest, digest)| (manifest.generation(), digest.clone()))
                .unwrap_or((0, String::new()));
            (
                state.health.manifest_revision,
                state.health.healthy,
                generation,
                digest,
            )
        };
        if manifest_revision >= revision {
            return Ok(());
        }
        self.check_manifest_floor(pointer.generation, &pointer.digest)?;
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
            let manifest = parse_issuer_manifest(&json)
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
            let verified = verify_issuer_manifest(&self.root_value()?, &manifest, &policy)
                .map_err(|error| {
                    TrellisClientError::Bootstrap(format!(
                        "issuer manifest {key} is not trusted: {error}"
                    ))
                })?;
            if let Some(own) = self.own.as_ref() {
                own.advance_manifest_floor(pointer.generation, &digest)
                    .await?;
            }

            let now = self.now_seconds().unwrap_or(0);
            {
                let mut state = self.write_state()?;
                let (mut policy, _, _) = state.policy_floor.clone().ok_or_else(|| {
                    TrellisClientError::Bootstrap("authorization context unavailable".into())
                })?;
                policy.minimum_manifest_generation = pointer.generation;
                state.policy_floor = Some((policy, pointer.generation, digest.clone()));
                state.manifest = Some((verified, digest));
                state.verified_contexts.clear();
                state.retention_deadlines.clear();
                state.health.manifest_revision = state.health.manifest_revision.max(revision);
                state.health.last_update_at = now;
                state.health.healthy = false;
            }
            if let Some(own) = self.own.as_ref() {
                own.request_refresh();
            }
            if !initial_snapshot {
                return Err(TrellisClientError::Bootstrap(
                    "issuer manifest advanced; restarting complete provider snapshot".into(),
                ));
            }
            return Ok(());
        }

        let now = self.now_seconds().unwrap_or(0);
        let mut state = self.write_state()?;
        state.health.manifest_revision = state.health.manifest_revision.max(revision);
        state.health.last_update_at = now;
        state.health.healthy = was_healthy;
        Ok(())
    }

    fn observe_revocation_entry(
        &self,
        entry: super::registry::RegistryWatchEntry,
    ) -> Result<(), TrellisClientError> {
        match super::registry::revocation_entry(entry) {
            RevocationWatchEntry::Applied {
                key,
                value,
                revision,
            } => self.observe_revocation_value(&key, &value, revision),
            RevocationWatchEntry::Removed { key, revision } => {
                self.observe_revocation_removal(&key, revision)
            }
        }
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
        let now = self.now_seconds().unwrap_or(0);
        {
            let mut state = self.write_state()?;
            state
                .revocations
                .entry(digest.to_owned())
                .and_modify(|current| *current = (*current).max(revoked_at))
                .or_insert(revoked_at);
            state.health.revocation_revision = state.health.revocation_revision.max(revision);
            state.health.last_update_at = now;
        }
        if let Some(own) = self.own.as_ref() {
            if own.context_digest().is_ok_and(|current| current == digest) {
                own.clear()?;
                own.request_refresh();
            }
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
        let now = self.now_seconds().unwrap_or(0);
        let mut state = self.write_state()?;
        state.health.revocation_revision = state.health.revocation_revision.max(revision);
        state.health.last_update_at = now;
        Ok(())
    }

    fn prune_contexts(&self, now: i64) -> Result<(), TrellisClientError> {
        let mut state = self.write_state()?;
        let expired = state
            .retention_deadlines
            .iter()
            .filter(|(_, retained_until)| **retained_until <= now)
            .map(|(digest, _)| digest.clone())
            .collect::<Vec<_>>();
        for digest in expired {
            state.retention_deadlines.remove(&digest);
            state.verified_contexts.remove(&digest);
        }
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

    fn root_value(&self) -> Result<AuthorizationTrustRoot, TrellisClientError> {
        self.read_state()?.root.clone().ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization context unavailable".into())
        })
    }

    fn check_manifest_floor(
        &self,
        generation: u64,
        digest: &str,
    ) -> Result<(), TrellisClientError> {
        let state = self.read_state()?;
        let Some((_, minimum_generation, minimum_digest)) = state.policy_floor.as_ref() else {
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

    #[cfg(feature = "runtime-internals")]
    #[doc(hidden)]
    pub fn runtime_current_manifest(
        &self,
    ) -> Result<VerifiedAuthorizationIssuerManifest, TrellisClientError> {
        self.read_state()?
            .manifest
            .as_ref()
            .map(|(manifest, _)| manifest.clone())
            .ok_or_else(|| TrellisClientError::Bootstrap("provider manifest unavailable".into()))
    }

    async fn resolve_context_for(
        &self,
        digest: &str,
        verification_time: i64,
        historical: bool,
    ) -> Result<VerifiedAuthorizationContext, TrellisClientError> {
        self.refresh_revocation(digest).await?;
        self.prune_contexts(self.now_seconds()?)?;
        let known = self.active_context_raw(digest)?;
        if let Some(context) = known {
            return Ok(context);
        }
        let manifest_generation = self
            .read_state()?
            .manifest
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
        let known = self.active_context_raw(digest)?;
        if let Some(context) = known {
            return Ok(context);
        }
        let outcome = self
            .resolve_context_once(digest, verification_time, historical)
            .await;
        self.drop_in_flight(digest, &pending);
        let context = outcome?;
        let mut state = self.write_state()?;
        if state
            .manifest
            .as_ref()
            .map(|(manifest, _)| manifest.generation())
            != Some(manifest_generation)
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization manifest advanced during context resolution".into(),
            ));
        }
        if !historical {
            state
                .retention_deadlines
                .insert(digest.to_owned(), context.expires_at());
            state
                .verified_contexts
                .insert(digest.to_owned(), context.clone());
        }
        Ok(context)
    }

    async fn refresh_revocation(&self, digest: &str) -> Result<(), TrellisClientError> {
        if self.revocation_time(digest)?.is_some() {
            return Ok(());
        }
        let Some(registry) = self.registry.as_ref() else {
            return Ok(());
        };
        if let Some(value) = registry.get_revocation(digest).await? {
            let revoked_at = parse_revocation_record(&value)?;
            let mut state = self.write_state()?;
            state
                .revocations
                .entry(digest.to_owned())
                .and_modify(|current| *current = (*current).max(revoked_at))
                .or_insert(revoked_at);
        }
        Ok(())
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
    ) -> Result<VerifiedAuthorizationContext, TrellisClientError> {
        let Some(registry) = self.registry.as_ref() else {
            return Err(TrellisClientError::Bootstrap(
                "authorization registry is unavailable".into(),
            ));
        };
        self.sync_trust_material()?;
        self.context_resolves.fetch_add(1, Ordering::Relaxed);
        let value = registry.get_context(digest).await?.ok_or_else(|| {
            TrellisClientError::Bootstrap(
                "authorization context is missing from the registry".into(),
            )
        })?;
        let json: serde_json::Value = serde_json::from_slice(&value)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let context = parse_authorization_context(&json)
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
                .read_state()?
                .manifest
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
        verify_authorization_context(&self.root_value()?, &manifest, &context, &policy).map_err(
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
        policy: &mut AuthorizationVerificationPolicy,
    ) -> Result<VerifiedAuthorizationIssuerManifest, TrellisClientError> {
        if let Some((manifest, digest)) = self
            .read_state()?
            .manifest
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
        let manifest = parse_issuer_manifest(&manifest_json)
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
        let manifest = verify_issuer_manifest(&self.root_value()?, &manifest, policy)
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

    fn read_state(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, AuthorizationProviderState>, TrellisClientError>
    {
        self.state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("provider state lock poisoned".into()))
    }

    fn write_state(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, AuthorizationProviderState>, TrellisClientError>
    {
        self.state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("provider state lock poisoned".into()))
    }

    /// Install an already-verified snapshot for unit tests without registry I/O.
    #[cfg(test)]
    pub(crate) fn inject_verified_for_test(
        &self,
        digest: &str,
        verified: VerifiedAuthorizationContext,
        revoked_at: Option<i64>,
    ) -> Result<(), TrellisClientError> {
        let mut state = self.write_state()?;
        state.verified_contexts.insert(digest.to_owned(), verified);
        if let Some(revoked_at) = revoked_at {
            state.revocations.insert(digest.to_owned(), revoked_at);
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
        "provider state lock poisoned",
        "provider resolution lock poisoned",
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
