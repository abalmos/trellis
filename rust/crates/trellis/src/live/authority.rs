//! Retained live-authority guards for active observation sessions.
//!
//! A live session must remain authorized for its entire lifetime without a
//! network lookup per frame. This module wraps the existing provider cache's
//! retained lease and locally covered evidence in a typed check that fails
//! closed. It deliberately does **not** reuse the cache's permissive
//! `current_context_allows` shortcut: that helper converts an unknown-coverage
//! revocation query through `.ok().flatten()` and only checks
//! `health().is_ok()`, both of which can read as authorization success when the
//! exact coverage is actually unknown.

use std::sync::Arc;

use trellis_protocol::{LiveErrorCode, PermissionAtom, ProtocolError};

use crate::client::{
    AuthorizationContextLease, AuthorizationProviderCache, AuthorizationVerificationCore,
    RequestVerificationInput, TrellisClientError,
};

/// Immutable identity tuple a live session pins for its whole lifetime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PinnedPeerIdentity {
    /// Logical runtime connection id from the signed context.
    pub connection_id: String,
    /// Runtime signing public key.
    pub session_key: String,
    /// Principal id.
    pub principal_id: String,
    /// Participant id.
    pub participant_id: String,
    /// Deployment id, absent for undeployed principals.
    pub deployment_id: Option<String>,
    /// Instance id, absent for undeployed principals.
    pub instance_id: Option<String>,
}

/// Why a retained live authority is no longer usable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LiveAuthorityLost {
    /// The local transport or provider connection is not usable.
    TransportUnavailable,
    /// The local transport epoch changed since the guard was installed.
    EpochChanged,
    /// The exact revocation coverage entry is no longer retained.
    CoverageLost,
    /// The context digest has recorded revocation evidence.
    Revoked,
    /// Revocation coverage is unknown; the guard fails closed.
    CoverageUnknown,
    /// The context's signed validity window no longer covers the current time.
    Expired,
    /// The pinned identity tuple no longer matches the retained context.
    IdentityChanged,
    /// The installed API binding generation changed.
    BindingChanged,
    /// The exact required permission is no longer granted.
    PermissionLost,
}

impl LiveAuthorityLost {
    /// Return the terminal reason this loss maps to.
    #[must_use]
    pub(crate) fn end_reason(self) -> trellis_protocol::LiveEndReason {
        match self {
            Self::TransportUnavailable => trellis_protocol::LiveEndReason::LocalShutdown,
            Self::EpochChanged => trellis_protocol::LiveEndReason::Disconnected,
            Self::CoverageLost | Self::Revoked | Self::CoverageUnknown => {
                trellis_protocol::LiveEndReason::AuthorizationLost
            }
            Self::Expired => trellis_protocol::LiveEndReason::AuthorizationLost,
            Self::IdentityChanged | Self::BindingChanged => {
                trellis_protocol::LiveEndReason::BindingChanged
            }
            Self::PermissionLost => trellis_protocol::LiveEndReason::AuthorizationLost,
        }
    }
}

/// Extra evidence a retained guard must keep true besides the covered lease.
pub(crate) enum LiveGuardRequirement {
    /// Local consumer or admitted remote caller Subscribe/Observe atom.
    Observer(PermissionAtom),
    /// Publishing endpoint: installed identity, not the caller's Subscribe grant.
    LocalProvider,
    /// Consumer authenticating a remote provider's pinned identity.
    PeerProvider { expected: PinnedPeerIdentity },
}

/// One retained live-authority guard over an existing provider-cache lease.
pub(crate) struct LiveAuthorityGuard {
    cache: AuthorizationProviderCache,
    lease: AuthorizationContextLease,
    expected_epoch: u64,
    identity: PinnedPeerIdentity,
    permission: Option<PermissionAtom>,
    peer: Option<PinnedPeerIdentity>,
}

impl LiveAuthorityGuard {
    /// Retain a live guard for one exact digest, permission, and pinned identity.
    ///
    /// # Errors
    ///
    /// Returns [`LiveAuthorityLost`] when the cache cannot currently retain the
    /// exact covered evidence the session needs.
    pub(crate) async fn retain(
        cache: &AuthorizationProviderCache,
        digest: &str,
        requirement: LiveGuardRequirement,
    ) -> Result<Self, LiveAuthorityLost> {
        let expected_epoch = cache.epoch();
        let lease = cache
            .retain_live_guard_lease(digest, expected_epoch)
            .await
            .map_err(|error| match error {
                TrellisClientError::AuthorizationUnavailable(_) => LiveAuthorityLost::CoverageLost,
                _ => LiveAuthorityLost::CoverageUnknown,
            })?;
        let identity = pinned_identity(&lease);
        let (permission, peer) = match requirement {
            LiveGuardRequirement::Observer(permission) => {
                if !lease.allows(&permission) {
                    return Err(LiveAuthorityLost::PermissionLost);
                }
                (Some(permission), None)
            }
            LiveGuardRequirement::LocalProvider => (None, None),
            LiveGuardRequirement::PeerProvider { expected } => {
                if identity != expected {
                    return Err(LiveAuthorityLost::IdentityChanged);
                }
                (None, Some(expected))
            }
        };
        Ok(Self {
            cache: cache.clone(),
            lease,
            expected_epoch,
            identity,
            permission,
            peer,
        })
    }

    /// Return the pinned identity this session retains.
    #[must_use]
    pub(crate) fn identity(&self) -> &PinnedPeerIdentity {
        &self.identity
    }

    /// Return the retained context digest.
    #[must_use]
    pub(crate) fn context_digest(&self) -> &str {
        self.lease.context_digest()
    }

    /// Perform the synchronous local authority check.
    ///
    /// Uses only retained local state: no network read, no cold context
    /// resolution, no reverification of the opening request's old proof.
    ///
    /// # Errors
    ///
    /// Returns the precise [`LiveAuthorityLost`] reason, never a lossy boolean.
    pub(crate) fn check_now(&self) -> Result<(), LiveAuthorityLost> {
        let health = self
            .cache
            .health()
            .map_err(|_| LiveAuthorityLost::TransportUnavailable)?;
        if !health.healthy {
            return Err(LiveAuthorityLost::TransportUnavailable);
        }
        if self.cache.epoch() != self.expected_epoch || self.lease.epoch() != self.expected_epoch {
            return Err(LiveAuthorityLost::EpochChanged);
        }
        if !self.cache.live_guard_entry_is_covered(&self.lease) {
            return Err(LiveAuthorityLost::CoverageLost);
        }
        match self.cache.revocation_time(self.context_digest()) {
            // Recorded revocation evidence.
            Ok(Some(_)) => return Err(LiveAuthorityLost::Revoked),
            // Unknown coverage must fail closed, not read as cleared.
            Err(_) => return Err(LiveAuthorityLost::CoverageUnknown),
            Ok(None) => {}
        }
        self.lease
            .assert_current(
                &self
                    .cache
                    .policy()
                    .map_err(|_| LiveAuthorityLost::CoverageUnknown)?,
            )
            .map_err(|_| LiveAuthorityLost::Expired)?;
        if pinned_identity(&self.lease) != self.identity {
            return Err(LiveAuthorityLost::IdentityChanged);
        }
        if let Some(expected) = &self.peer {
            if &self.identity != expected {
                return Err(LiveAuthorityLost::IdentityChanged);
            }
        }
        if let Some(permission) = &self.permission {
            if !self.lease.allows(permission) {
                return Err(LiveAuthorityLost::PermissionLost);
            }
        }
        Ok(())
    }

    /// Authenticate one owner-control request against this retained caller.
    ///
    /// Unverified, foreign-reply, and mismatched-identity requests fail so the
    /// caller can drop them without reflection.
    pub(crate) fn verify_control_request(
        &self,
        subject: &str,
        reply: &str,
        payload: &[u8],
        headers: &async_nats::HeaderMap,
    ) -> Result<(), LiveErrorCode> {
        self.check_now()
            .map_err(|_| LiveErrorCode::PermissionDenied)?;
        let session_key = header_once(headers, "session-key")?;
        if session_key != self.identity.session_key {
            return Err(LiveErrorCode::PermissionDenied);
        }
        let context_digest = header_once(headers, "authorization-context")?;
        if context_digest != self.context_digest() {
            return Err(LiveErrorCode::PermissionDenied);
        }
        let proof = header_once(headers, "proof")?;
        let request_id = header_once(headers, "request-id")?;
        let iat = header_once(headers, "iat")?
            .parse::<i64>()
            .map_err(|_| LiveErrorCode::InvalidRequest)?;
        let prefix = self.lease.inbox_prefix();
        if reply != prefix && !reply.starts_with(&format!("{prefix}.")) {
            return Err(LiveErrorCode::InvalidSubject);
        }
        let required = match &self.permission {
            Some(permission) => std::slice::from_ref(permission),
            None => &[],
        };
        let policy = self
            .cache
            .policy()
            .map_err(|_| LiveErrorCode::AuthorizationUnavailable)?;
        AuthorizationVerificationCore::new()
            .verify_request(RequestVerificationInput {
                context: &self.lease,
                context_digest: &context_digest,
                subject,
                payload,
                iat,
                request_id: &request_id,
                reply_subject: Some(reply),
                proof: &proof,
                policy: &policy,
                required_permissions: required,
            })
            .map_err(|_| LiveErrorCode::PermissionDenied)?;
        Ok(())
    }

    /// Replace the guard's retained lease with a newer verified installation.
    ///
    /// The pinned identity, permission, and expected epoch must still hold; an
    /// identity-preserving refresh on the same transport epoch does not close
    /// the session or reset its sequence/credit state.
    ///
    /// # Errors
    ///
    /// Returns [`LiveAuthorityLost`] when the replacement does not preserve the
    /// session's pinned identity or required authority.
    pub(crate) async fn replace(&mut self, digest: &str) -> Result<(), LiveAuthorityLost> {
        let lease = self
            .cache
            .retain_live_guard_lease(digest, self.expected_epoch)
            .await
            .map_err(|_| LiveAuthorityLost::CoverageLost)?;
        if pinned_identity(&lease) != self.identity {
            return Err(LiveAuthorityLost::IdentityChanged);
        }
        if let Some(expected) = &self.peer {
            if pinned_identity(&lease) != *expected {
                return Err(LiveAuthorityLost::IdentityChanged);
            }
        }
        if let Some(permission) = &self.permission {
            if !lease.allows(permission) {
                return Err(LiveAuthorityLost::PermissionLost);
            }
        }
        self.lease = lease;
        Ok(())
    }
}

impl PinnedPeerIdentity {
    /// Project one signed context into its immutable identity tuple.
    #[must_use]
    pub(crate) fn from_signed(signed: &trellis_protocol::SignedAuthorizationContext) -> Self {
        Self {
            connection_id: signed.unsigned.connection_id.clone(),
            session_key: signed.unsigned.session_key.clone(),
            principal_id: signed.unsigned.principal_id.clone(),
            participant_id: signed.unsigned.participant_id.clone(),
            deployment_id: signed.unsigned.deployment_id.clone(),
            instance_id: signed.unsigned.instance_id.clone(),
        }
    }
}

fn pinned_identity(lease: &AuthorizationContextLease) -> PinnedPeerIdentity {
    PinnedPeerIdentity::from_signed(lease.signed_context())
}

fn header_once(headers: &async_nats::HeaderMap, name: &str) -> Result<String, LiveErrorCode> {
    let value = headers
        .get(name)
        .ok_or(LiveErrorCode::InvalidRequest)?
        .to_string();
    if value.is_empty() {
        return Err(LiveErrorCode::InvalidRequest);
    }
    Ok(value)
}

/// One retained guard pair for a session that both publishes and consumes.
pub(crate) struct LiveAuthorityPair {
    /// Guard over the session owner's own installation.
    pub own: LiveAuthorityGuard,
    /// Guard over the pinned peer's installation.
    pub peer: LiveAuthorityGuard,
}

impl LiveAuthorityPair {
    /// Check both guards, returning the first typed loss.
    ///
    /// # Errors
    ///
    /// Returns the first [`LiveAuthorityLost`] reason from either side.
    pub(crate) fn check_now(&self) -> Result<(), LiveAuthorityLost> {
        self.own.check_now()?;
        self.peer.check_now()
    }
}

/// Map a protocol error into the closest authority-loss reason.
#[must_use]
pub(crate) fn authority_loss_from_protocol(error: &ProtocolError) -> LiveAuthorityLost {
    match error {
        ProtocolError::Live { message, .. } if message.contains("revoked") => {
            LiveAuthorityLost::Revoked
        }
        ProtocolError::Authorization { .. } => LiveAuthorityLost::PermissionLost,
        _ => LiveAuthorityLost::CoverageUnknown,
    }
}

/// Retained handle to the manager's authority listener registration.
pub(crate) struct LiveAuthorityListener {
    _registration: Arc<()>,
}
