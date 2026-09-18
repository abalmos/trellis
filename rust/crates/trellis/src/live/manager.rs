//! Per-connection live-session ownership and admission.
//!
//! One manager exists per actual authenticated NATS connection owner. Generated
//! facades borrow it; separate connections receive separate managers. The
//! manager owns ephemeral session records, admission permits, owner-control
//! registrations and closed receipts in memory. There is no session database,
//! KV entry or central relay.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use trellis_protocol::{
    derive_live_observe_wildcard_subject, LiveEndReason, LiveErrorCode, LiveSessionKind,
    MAX_CONSUMER_SESSIONS, MAX_PROVIDER_SESSIONS, MAX_PROVIDER_SESSIONS_PER_CALLER, MAX_TOMBSTONES,
    TOMBSTONE_MS,
};

use super::authority::{LiveAuthorityGuard, LiveAuthorityLost};
use super::provider_engine::ProviderSessionRecord;
use super::types::{
    CloseCleanupState, CloseRemoteState, LiveCloseReceipt, LiveEnd, LiveStreamError,
};
use crate::client::SessionAuth;

/// One retained closed-session receipt for idempotent control handling.
#[derive(Clone, Debug)]
pub(crate) struct ClosedReceipt {
    pub session_id: String,
    pub owner_token: String,
    pub base_subject: String,
    pub reason: LiveEndReason,
    pub cleanup: CloseCleanupState,
    pub final_seq: u64,
    pub expires_at_ms: u64,
}

/// One owner-control registration for an exact route.
pub(crate) struct OwnerControlRegistration {
    pub base_subject: String,
    pub provider_connection_id: String,
    pub wildcard_subject: String,
    pub subscription: Option<async_nats::Subscriber>,
}

/// Admission counters for provider reservations.
struct ProviderAdmission {
    total: usize,
    per_caller: HashMap<String, usize>,
}

impl ProviderAdmission {
    fn new() -> Self {
        Self {
            total: 0,
            per_caller: HashMap::new(),
        }
    }

    fn admit(&mut self, caller_key: &str) -> Result<(), LiveErrorCode> {
        if self.total >= MAX_PROVIDER_SESSIONS {
            return Err(LiveErrorCode::ResourceExhausted);
        }
        let entry = self.per_caller.entry(caller_key.to_owned()).or_insert(0);
        if *entry >= MAX_PROVIDER_SESSIONS_PER_CALLER {
            return Err(LiveErrorCode::ResourceExhausted);
        }
        self.total += 1;
        *entry += 1;
        Ok(())
    }

    fn release(&mut self, caller_key: &str) {
        self.total = self.total.saturating_sub(1);
        if let Some(entry) = self.per_caller.get_mut(caller_key) {
            *entry = entry.saturating_sub(1);
            if *entry == 0 {
                self.per_caller.remove(caller_key);
            }
        }
    }
}

/// Reasons the manager itself is unavailable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ManagerUnavailable {
    /// The manager has been stopped by connection shutdown.
    Stopped,
    /// The local transport epoch changed since the manager started.
    EpochChanged,
}

impl ManagerUnavailable {
    pub(crate) const fn end_reason(self) -> LiveEndReason {
        match self {
            Self::Stopped => LiveEndReason::LocalShutdown,
            Self::EpochChanged => LiveEndReason::Disconnected,
        }
    }
}

/// One live session manager for an actual authenticated connection owner.
pub struct LiveSessionManager {
    nats: async_nats::Client,
    auth: Arc<SessionAuth>,
    contexts: Arc<crate::client::AuthorizationContextCache>,
    provider_connection_id: String,
    local_epoch: u64,
    stopped: AtomicBool,
    generation: AtomicU64,
    provider_admission: Mutex<ProviderAdmission>,
    consumer_sessions: AtomicU64,
    tombstones: Mutex<Vec<ClosedReceipt>>,
    owner_controls: RwLock<Vec<OwnerControlRegistration>>,
    /// Live authority guards retained by active sessions, keyed by digest.
    retained_guards: Mutex<HashMap<String, LiveAuthorityGuard>>,
    /// Provider sessions owned by this manager, keyed by session id.
    provider_sessions: Mutex<HashMap<String, Arc<ProviderSessionRecord>>>,
}

/// Inputs for one provider control-handling loop.
pub struct OwnerControlLoop {
    /// Exact owner-directed wildcard subscription for this provider connection.
    pub subscription: async_nats::Subscriber,
    /// The manager that owns this provider's reserved sessions.
    pub manager: Arc<LiveSessionManager>,
}

impl LiveSessionManager {
    /// Create one manager for an authenticated connection owner.
    #[must_use]
    pub(crate) fn new(
        nats: async_nats::Client,
        auth: Arc<SessionAuth>,
        contexts: Arc<crate::client::AuthorizationContextCache>,
        provider_connection_id: String,
    ) -> Arc<Self> {
        let local_epoch = nats.statistics().connects.load(Ordering::Acquire);
        Arc::new(Self {
            nats,
            auth,
            contexts,
            provider_connection_id,
            local_epoch,
            stopped: AtomicBool::new(false),
            generation: AtomicU64::new(1),
            provider_admission: Mutex::new(ProviderAdmission::new()),
            consumer_sessions: AtomicU64::new(0),
            tombstones: Mutex::new(Vec::new()),
            owner_controls: RwLock::new(Vec::new()),
            retained_guards: Mutex::new(HashMap::new()),
            provider_sessions: Mutex::new(HashMap::new()),
        })
    }

    /// Return the provider's logical runtime connection id.
    #[must_use]
    pub(crate) fn provider_connection_id(&self) -> &str {
        &self.provider_connection_id
    }

    /// Return the local transport epoch this manager was created on.
    #[must_use]
    pub(crate) fn local_epoch(&self) -> u64 {
        self.local_epoch
    }

    /// Return whether the manager still accepts new opens.
    pub(crate) fn is_available(&self) -> Result<(), ManagerUnavailable> {
        if self.stopped.load(Ordering::Acquire) {
            return Err(ManagerUnavailable::Stopped);
        }
        if self.nats.connection_state() != async_nats::connection::State::Connected
            || self.nats.statistics().connects.load(Ordering::Acquire) != self.local_epoch
        {
            return Err(ManagerUnavailable::EpochChanged);
        }
        Ok(())
    }

    /// Reserve one provider admission permit.
    ///
    /// # Errors
    ///
    /// Returns [`trellis_protocol::LiveErrorCode::ResourceExhausted`] when the
    /// combined or per-caller bound is reached.
    pub(crate) fn admit_provider(
        self: &Arc<Self>,
        consumer_connection_id: &str,
        consumer_session_key: &str,
    ) -> Result<ProviderPermit, LiveErrorCode> {
        let caller_key = format!("{consumer_connection_id}:{consumer_session_key}");
        self.provider_admission
            .lock()
            .map_err(|_| LiveErrorCode::ResourceExhausted)?
            .admit(&caller_key)?;
        Ok(ProviderPermit {
            manager: Arc::downgrade(self),
            caller_key,
        })
    }

    /// Reserve one consumer session permit.
    ///
    /// # Errors
    ///
    /// Returns [`trellis_protocol::LiveErrorCode::ResourceExhausted`] when the
    /// consumer bound is reached.
    pub(crate) fn admit_consumer(self: &Arc<Self>) -> Result<ConsumerPermit, LiveErrorCode> {
        let current = self.consumer_sessions.load(Ordering::Acquire);
        if current as usize >= MAX_CONSUMER_SESSIONS {
            return Err(LiveErrorCode::ResourceExhausted);
        }
        self.consumer_sessions.fetch_add(1, Ordering::AcqRel);
        Ok(ConsumerPermit {
            manager: Arc::downgrade(self),
        })
    }

    /// Install the nonqueued owner-control subscription for one exact route.
    ///
    /// The subscription carries no queue group so the exact provider that
    /// accepted a session handles its controls; another replica must not
    /// consume them.
    ///
    /// # Errors
    ///
    /// Returns a transport error when the subscription cannot be installed or
    /// flushed.
    pub(crate) async fn register_owner_control(
        &self,
        base_subject: &str,
    ) -> Result<(), async_nats::Error> {
        let wildcard =
            derive_live_observe_wildcard_subject(base_subject, &self.provider_connection_id)
                .map_err(|error| {
                    async_nats::Error::from(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        error.to_string(),
                    ))
                        as Box<dyn std::error::Error + Send + Sync>)
                })?;
        {
            let controls = self
                .owner_controls
                .read()
                .map_err(|_| std::io::Error::other("owner control lock poisoned"))?;
            if controls
                .iter()
                .any(|registration| registration.base_subject == base_subject)
            {
                return Ok(());
            }
        }
        let subscription = self.nats.subscribe(wildcard.clone()).await?;
        self.nats.flush().await?;
        self.owner_controls
            .write()
            .map_err(|_| std::io::Error::other("owner control lock poisoned"))?
            .push(OwnerControlRegistration {
                base_subject: base_subject.to_owned(),
                provider_connection_id: self.provider_connection_id.clone(),
                wildcard_subject: wildcard,
                subscription: Some(subscription),
            });
        Ok(())
    }

    /// Retain a live authority guard for an active session.
    ///
    /// # Errors
    ///
    /// Returns the typed [`LiveAuthorityLost`] reason when the exact covered
    /// evidence cannot be retained.
    pub(crate) async fn retain_guard(
        &self,
        cache: &crate::client::AuthorizationProviderCache,
        digest: &str,
        permission: trellis_protocol::PermissionAtom,
    ) -> Result<(), LiveAuthorityLost> {
        let guard = LiveAuthorityGuard::retain(cache, digest, permission).await?;
        self.retained_guards
            .lock()
            .map_err(|_| LiveAuthorityLost::CoverageUnknown)?
            .insert(digest.to_owned(), guard);
        Ok(())
    }

    /// Check every retained guard without network access.
    ///
    /// # Errors
    ///
    /// Returns the first typed [`LiveAuthorityLost`] reason.
    pub(crate) fn check_retained_guard(&self, digest: &str) -> Result<(), LiveAuthorityLost> {
        let guards = self
            .retained_guards
            .lock()
            .map_err(|_| LiveAuthorityLost::CoverageUnknown)?;
        match guards.get(digest) {
            Some(guard) => guard.check_now(),
            None => Err(LiveAuthorityLost::CoverageLost),
        }
    }

    /// Record one closed-session receipt in the bounded LRU.
    pub(crate) fn insert_receipt(&self, receipt: ClosedReceipt) {
        let Ok(mut tombstones) = self.tombstones.lock() else {
            return;
        };
        tombstones.retain(|existing| existing.session_id != receipt.session_id);
        tombstones.push(receipt);
        if tombstones.len() > MAX_TOMBSTONES {
            tombstones.remove(0);
        }
    }

    /// Find one retained receipt if it has not expired.
    #[must_use]
    pub(crate) fn receipt(&self, session_id: &str, now_ms: u64) -> Option<ClosedReceipt> {
        let Ok(tombstones) = self.tombstones.lock() else {
            return None;
        };
        tombstones
            .iter()
            .find(|receipt| receipt.session_id == session_id && receipt.expires_at_ms > now_ms)
            .cloned()
    }

    /// Stop the manager: fence new opens and release owner-control registrations.
    pub(crate) fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
        self.generation.fetch_add(1, Ordering::AcqRel);
        if let Ok(mut controls) = self.owner_controls.write() {
            controls.clear();
        }
    }

    /// Return the manager's current generation.
    #[must_use]
    pub(crate) fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    /// Register one reserved provider session owned by this manager.
    pub(crate) fn insert_provider_session(
        &self,
        session_id: String,
        record: Arc<ProviderSessionRecord>,
    ) {
        if let Ok(mut sessions) = self.provider_sessions.lock() {
            sessions.insert(session_id, record);
        }
    }

    /// Look up one provider session by id.
    #[must_use]
    pub(crate) fn provider_session(&self, session_id: &str) -> Option<Arc<ProviderSessionRecord>> {
        self.provider_sessions
            .lock()
            .ok()
            .and_then(|sessions| sessions.get(session_id).cloned())
    }

    /// Remove one provider session after its terminal receipt is recorded.
    pub(crate) fn remove_provider_session(&self, session_id: &str) {
        if let Ok(mut sessions) = self.provider_sessions.lock() {
            sessions.remove(session_id);
        }
    }

    /// Return the cloneable transport handle for this manager.
    #[must_use]
    pub(crate) fn nats_handle(&self) -> async_nats::Client {
        self.nats.clone()
    }

    /// Return the cloneable signing material for provider publications.
    #[must_use]
    pub(crate) fn auth_handle(&self) -> Arc<SessionAuth> {
        self.auth.clone()
    }

    /// Return the retained authorization cache for provider guards.
    #[must_use]
    pub(crate) fn contexts_handle(&self) -> Arc<crate::client::AuthorizationContextCache> {
        self.contexts.clone()
    }

    /// Count currently retained provider sessions.
    #[must_use]
    pub(crate) fn provider_session_count(&self) -> usize {
        self.provider_sessions
            .lock()
            .map_or(0, |sessions| sessions.len())
    }
}

/// Permit for one provider reservation; releases its admission slot on drop.
pub(crate) struct ProviderPermit {
    manager: std::sync::Weak<LiveSessionManager>,
    caller_key: String,
}

impl Drop for ProviderPermit {
    fn drop(&mut self) {
        if let Some(manager) = self.manager.upgrade() {
            if let Ok(mut admission) = manager.provider_admission.lock() {
                admission.release(&self.caller_key);
            }
        }
    }
}

/// Permit for one consumer session; releases its slot on drop.
pub(crate) struct ConsumerPermit {
    manager: std::sync::Weak<LiveSessionManager>,
}

impl Drop for ConsumerPermit {
    fn drop(&mut self) {
        if let Some(manager) = self.manager.upgrade() {
            manager.consumer_sessions.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

/// Map one authority loss into a terminal outcome with its bounded error.
#[must_use]
pub(crate) fn authority_end(lost: &LiveAuthorityLost) -> LiveEnd {
    let reason = lost.clone().end_reason();
    let code = match lost {
        LiveAuthorityLost::TransportUnavailable => LiveErrorCode::Disconnected,
        LiveAuthorityLost::EpochChanged => LiveErrorCode::Disconnected,
        LiveAuthorityLost::CoverageLost | LiveAuthorityLost::CoverageUnknown => {
            LiveErrorCode::AuthorizationUnavailable
        }
        LiveAuthorityLost::Revoked => LiveErrorCode::AuthorizationRevoked,
        LiveAuthorityLost::Expired => LiveErrorCode::AuthorizationExpired,
        LiveAuthorityLost::IdentityChanged | LiveAuthorityLost::BindingChanged => {
            LiveErrorCode::BindingChanged
        }
        LiveAuthorityLost::PermissionLost => LiveErrorCode::PermissionDenied,
    };
    LiveEnd::new(
        reason,
        Some(Arc::new(LiveStreamError::new(code, lost_message(lost)))),
    )
}

fn lost_message(lost: &LiveAuthorityLost) -> &'static str {
    match lost {
        LiveAuthorityLost::TransportUnavailable => "local transport is not usable",
        LiveAuthorityLost::EpochChanged => "local transport epoch changed",
        LiveAuthorityLost::CoverageLost => "exact revocation coverage is no longer retained",
        LiveAuthorityLost::Revoked => "authorization context was revoked",
        LiveAuthorityLost::CoverageUnknown => "revocation coverage is unknown",
        LiveAuthorityLost::Expired => "authorization context expired",
        LiveAuthorityLost::IdentityChanged => "pinned provider identity changed",
        LiveAuthorityLost::BindingChanged => "installed API binding changed",
        LiveAuthorityLost::PermissionLost => "required permission is no longer granted",
    }
}

/// Build one close receipt from a terminal outcome and remote states.
#[must_use]
pub(crate) fn close_receipt(
    end: LiveEnd,
    remote: CloseRemoteState,
    cleanup: CloseCleanupState,
) -> LiveCloseReceipt {
    LiveCloseReceipt::new(end, remote, cleanup)
}

/// Return whether one session kind uses the Operation control route.
#[must_use]
pub(crate) fn kind_uses_operation_control(kind: LiveSessionKind) -> bool {
    matches!(kind, LiveSessionKind::OperationWatch)
}

/// Tombstone expiry for one receipt recorded now.
#[must_use]
pub(crate) fn receipt_expiry(now_ms: u64) -> u64 {
    now_ms.saturating_add(TOMBSTONE_MS)
}
