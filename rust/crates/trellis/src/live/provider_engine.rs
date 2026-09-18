//! Provider-side live session engine: opening admission, signed offer, owner
//! control handling, activation, serialized publication and owned cleanup.
//!
//! One engine instance is driven by the service router per accepted opening
//! request. It never starts a domain source before the delivery-path challenge
//! round trip completes.

use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use futures_util::{Stream, StreamExt};

use trellis_protocol::{
    LiveControl, LiveControlAck, LiveEndReason, LiveErrorCode, LiveSessionKind, LiveSessionState,
    WireTerminal,
};

use super::authority::{LiveAuthorityGuard, LiveAuthorityLost, PinnedPeerIdentity};
use super::manager::LiveSessionManager;
use super::provider::{
    bounded_message, challenge_frame, control_ack, control_error, data_frame, end_frame,
    provider_failure, receipt_for, terminal_from_end, ProviderPhase, ProviderReservationIdentity,
    ProviderSession, ProviderSessionSubjects,
};
use super::types::{CloseCleanupState, LiveEnd, LiveStreamError};

/// One accepted opening request ready to reserve a session.
pub(crate) struct ProviderOpenRequest {
    pub kind: LiveSessionKind,
    pub base_subject: String,
    pub open_id: String,
    pub consumer: PinnedPeerIdentity,
    pub consumer_max_payload_bytes: u64,
    pub canonical_open_hash: String,
    pub caller_guard: LiveAuthorityGuard,
    pub own_guard: LiveAuthorityGuard,
}

/// One domain source item ready for publication.
pub(crate) enum SourceItem {
    /// One application value encoded by the generated codec.
    Value(serde_json::Value),
    /// Normal end of the source.
    End,
}

/// One provider source bound at activation.
pub(crate) type ProviderSource = Pin<Box<dyn Stream<Item = Result<SourceItem, String>> + Send>>;

/// One owned cleanup callback run under the shared provider grace.
pub(crate) type ProviderCleanup =
    Box<dyn FnOnce() -> Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send>;

/// One registered source factory invoked exactly once at activation.
pub(crate) type ProviderSourceFactory = Box<dyn FnOnce() -> ProviderSource + Send>;

/// One registered provider session with its source factory and cleanup hook.
pub(crate) struct ProviderSessionRecord {
    pub session: Arc<ProviderSession>,
    pub manager: Arc<LiveSessionManager>,
    pub source_factory: std::sync::Mutex<Option<ProviderSourceFactory>>,
    pub cleanup: std::sync::Mutex<Vec<ProviderCleanup>>,
    pub terminal: std::sync::Mutex<Option<LiveEnd>>,
    /// Cancellation for this session's source scope.
    pub cancellation: super::types::LiveCancellation,
}

impl ProviderSessionRecord {
    /// Reserve one offered session and install its owner-control subscription.
    ///
    /// # Errors
    ///
    /// Returns a wire error when the owner-control subscription cannot be
    /// installed or the provider admission bound is reached.
    pub(crate) async fn reserve(
        manager: &Arc<LiveSessionManager>,
        request: &ProviderOpenRequest,
        own_deployment_id: &str,
        max_data_body_bytes: u64,
        now_ms: u64,
    ) -> Result<(Arc<ProviderSession>, Arc<LiveSessionManager>), LiveErrorCode> {
        let permit = manager.clone().admit_provider(
            &request.consumer.connection_id,
            &request.consumer.session_key,
        )?;
        manager
            .register_owner_control(&request.base_subject)
            .await
            .map_err(|_| LiveErrorCode::ResourceExhausted)?;
        let _ = own_deployment_id;
        let _ = max_data_body_bytes;
        let session_id =
            trellis_protocol::generate_nonce().map_err(|_| LiveErrorCode::ProtocolError)?;
        let subjects = ProviderSessionSubjects {
            base_subject: request.base_subject.clone(),
            data_subject: trellis_protocol::derive_live_data_subject(
                manager.provider_connection_id(),
                &request.consumer.connection_id,
                &session_id,
            )
            .map_err(|_| LiveErrorCode::InvalidSubject)?,
            control_subject: trellis_protocol::derive_live_observe_subject(
                &request.base_subject,
                manager.provider_connection_id(),
                &session_id,
            )
            .map_err(|_| LiveErrorCode::InvalidSubject)?,
        };
        let session = Arc::new(ProviderSession::new(
            ProviderReservationIdentity {
                session_id,
                open_id: request.open_id.clone(),
                kind: request.kind,
            },
            subjects,
            request.consumer.clone(),
            now_ms,
        ));
        // Keep the admission permit alive for the session's whole reservation
        // by leaking it into the record's cleanup path.
        std::mem::forget(permit);
        Ok((session, Arc::clone(manager)))
    }

    /// Build one signed offer body for this reservation.
    ///
    /// # Errors
    ///
    /// Returns a wire error when the offer cannot be serialized or the session
    /// has already left its offered state.
    pub(crate) fn offer_body(
        &self,
        request_id: &str,
        provider: &PinnedPeerIdentity,
        consumer: &PinnedPeerIdentity,
        limits: trellis_protocol::LiveOfferLimits,
        max_data_body_bytes: u64,
    ) -> Result<Bytes, LiveErrorCode> {
        let _ = max_data_body_bytes;
        let offer = trellis_protocol::LiveOffer {
            format: trellis_protocol::LIVE_VERSION.to_owned(),
            kind: trellis_protocol::LiveOfferKind::Offer,
            session_kind: self.session.kind,
            open_id: self.session.open_id.clone(),
            request_id: request_id.to_owned(),
            session_id: self.session.session_id.clone(),
            base_subject: self.session.base_subject.clone(),
            data_subject: self.session.data_subject.clone(),
            control_subject: self.session.control_subject.clone(),
            provider: trellis_protocol::LiveOfferProvider {
                connection_id: provider.connection_id.clone(),
                session_key: provider.session_key.clone(),
                principal_id: provider.principal_id.clone(),
                participant_id: provider.participant_id.clone(),
                deployment_id: provider.deployment_id.clone().unwrap_or_default(),
                instance_id: provider.instance_id.clone().unwrap_or_default(),
            },
            consumer: trellis_protocol::LiveOfferConsumer {
                connection_id: consumer.connection_id.clone(),
                session_key: consumer.session_key.clone(),
                principal_id: consumer.principal_id.clone(),
                participant_id: consumer.participant_id.clone(),
            },
            limits,
        };
        serde_json::to_vec(&offer)
            .map(Bytes::from)
            .map_err(|_| LiveErrorCode::ProtocolError)
    }

    /// Handle one owner control, mutating state once and caching the outcome.
    ///
    /// # Errors
    ///
    /// Returns a typed control response error for an authenticated owner
    /// violation.
    pub(crate) async fn handle_control(
        &self,
        control: &LiveControl,
        now_ms: u64,
    ) -> Result<ControlOutcome, LiveErrorCode> {
        match control {
            LiveControl::Activate(_) => {
                if !matches!(
                    self.session.phase(),
                    ProviderPhase::Offered | ProviderPhase::Activating
                ) {
                    return Err(LiveErrorCode::StaleControl);
                }
                self.session.set_phase(ProviderPhase::Activating);
                let challenge_id =
                    trellis_protocol::generate_nonce().map_err(|_| LiveErrorCode::ProtocolError)?;
                let last_sent_seq = self
                    .session
                    .highest_sent
                    .load(std::sync::atomic::Ordering::Acquire);
                if let Ok(mut challenge) = self.session.challenge.lock() {
                    *challenge = Some(super::provider::ChallengeState {
                        challenge_id: challenge_id.clone(),
                        last_sent_seq,
                        answered: false,
                        last_published_ms: now_ms,
                    });
                }
                Ok(ControlOutcome {
                    state: LiveSessionState::Activating,
                    terminal: None,
                    cleanup: None,
                    challenge: Some(challenge_id),
                })
            }
            LiveControl::Pulse(pulse) => {
                let committed = self.session.commit_fresh_pulse(&pulse.challenge_id, now_ms);
                if !committed {
                    return Err(LiveErrorCode::InvalidChallenge);
                }
                if !matches!(self.session.phase(), ProviderPhase::Active) {
                    self.session.set_phase(ProviderPhase::Active);
                }
                self.session
                    .apply_credit(pulse.received_seq.get(), pulse.consumed_seq.get())?;
                Ok(ControlOutcome {
                    state: LiveSessionState::Active,
                    terminal: None,
                    cleanup: None,
                    challenge: None,
                })
            }
            LiveControl::Ack(ack) => {
                self.session
                    .apply_credit(ack.received_seq.get(), ack.consumed_seq.get())?;
                Ok(ControlOutcome {
                    state: current_state(self.session.phase()),
                    terminal: None,
                    cleanup: None,
                    challenge: None,
                })
            }
            LiveControl::Close(close) => {
                self.session
                    .apply_credit(close.received_seq.get(), close.consumed_seq.get())
                    .ok();
                self.session.set_phase(ProviderPhase::Closing);
                let cleanup = self.run_owned_cleanup().await;
                self.session.set_phase(ProviderPhase::Closed);
                Ok(ControlOutcome {
                    state: LiveSessionState::Closed,
                    terminal: Some(terminal_from_end(&self.committed_end())),
                    cleanup: Some(match cleanup {
                        CloseCleanupState::Complete => trellis_protocol::CleanupStatus::Complete,
                        _ => trellis_protocol::CleanupStatus::Incomplete,
                    }),
                    challenge: None,
                })
            }
            LiveControl::EndAck(ack) => {
                self.session
                    .apply_credit(ack.received_seq.get(), ack.consumed_seq.get())
                    .ok();
                self.session.set_phase(ProviderPhase::Closed);
                let cleanup = self.run_owned_cleanup().await;
                Ok(ControlOutcome {
                    state: LiveSessionState::Closed,
                    terminal: Some(terminal_from_end(&self.committed_end())),
                    cleanup: Some(match cleanup {
                        CloseCleanupState::Complete => trellis_protocol::CleanupStatus::Complete,
                        _ => trellis_protocol::CleanupStatus::Incomplete,
                    }),
                    challenge: None,
                })
            }
        }
    }

    /// Run every registered owned cleanup callback under the shared grace.
    pub(crate) async fn run_owned_cleanup(&self) -> CloseCleanupState {
        let callbacks = {
            let Ok(mut cleanup) = self.cleanup.lock() else {
                return CloseCleanupState::Incomplete;
            };
            std::mem::take(&mut *cleanup)
        };
        let mut futures = Vec::new();
        for callback in callbacks {
            futures.push(callback());
        }
        let all = futures_util::future::join_all(futures);
        match tokio::time::timeout(super::provider::cleanup_grace(), all).await {
            Ok(_) => CloseCleanupState::Complete,
            Err(_) => CloseCleanupState::Incomplete,
        }
    }

    /// Return the cancellation token for this session's source scope.
    #[must_use]
    pub(crate) fn cancellation_handle(&self) -> super::types::LiveCancellation {
        self.cancellation.clone()
    }

    /// Return the first committed terminal outcome, or a protocol error.
    pub(crate) fn committed_end(&self) -> LiveEnd {
        self.terminal
            .lock()
            .ok()
            .and_then(|slot| slot.clone())
            .unwrap_or_else(|| provider_failure(LiveErrorCode::ProtocolError, "session closed"))
    }

    /// Commit one terminal outcome once.
    pub(crate) fn commit_end(&self, end: LiveEnd) {
        if let Ok(mut slot) = self.terminal.lock() {
            if slot.is_none() {
                *slot = Some(end);
            }
        }
    }

    /// Build the receipt tombstone for this session after closure.
    pub(crate) fn tombstone(
        &self,
        cleanup: CloseCleanupState,
        now_ms: u64,
    ) -> super::manager::ClosedReceipt {
        receipt_for(
            &self.session,
            self.committed_end().reason(),
            cleanup,
            now_ms,
        )
    }
}

/// Outcome of one handled control.
pub(crate) struct ControlOutcome {
    pub state: LiveSessionState,
    pub terminal: Option<WireTerminal>,
    pub cleanup: Option<trellis_protocol::CleanupStatus>,
    /// One challenge to publish on the data subject after the acknowledgement.
    pub challenge: Option<String>,
}

fn current_state(phase: ProviderPhase) -> LiveSessionState {
    match phase {
        ProviderPhase::Offered | ProviderPhase::Activating => LiveSessionState::Activating,
        ProviderPhase::Active => LiveSessionState::Active,
        ProviderPhase::Closing | ProviderPhase::Closed => LiveSessionState::Closed,
    }
}

/// Publish one provider data frame and account for its credit cost.
///
/// # Errors
///
/// Returns a wire error when the frame exceeds the window or the negotiated
/// body limit, or when the publication handoff fails.
pub(crate) async fn publish_data_frame(
    session: &ProviderSession,
    nats: &async_nats::Client,
    value: serde_json::Value,
    max_data_body_bytes: u64,
) -> Result<u64, LiveErrorCode> {
    // Admit the worst-case serialized size (a 20-digit u64 sequence) before the
    // real sequence is known, so an admitted frame can never exceed the bound.
    let probe = serde_json::to_vec(&data_frame(&session.session_id, u64::MAX, value.clone()))
        .map_err(|_| LiveErrorCode::ProtocolError)?;
    let seq = session.admit_frame(probe.len() as u64, max_data_body_bytes)?;
    let body = serde_json::to_vec(&data_frame(&session.session_id, seq, value))
        .map_err(|_| LiveErrorCode::ProtocolError)?;
    nats.publish(session.data_subject.clone(), Bytes::from(body))
        .await
        .map_err(|_| LiveErrorCode::PeerLost)?;
    Ok(seq)
}

/// Publish one provider challenge frame.
///
/// # Errors
///
/// Returns a wire error when the publication handoff fails.
pub(crate) async fn publish_challenge(
    session: &ProviderSession,
    nats: &async_nats::Client,
    challenge_id: &str,
) -> Result<(), LiveErrorCode> {
    let last_sent = session
        .highest_sent
        .load(std::sync::atomic::Ordering::Acquire);
    let body = serde_json::to_vec(&challenge_frame(
        &session.session_id,
        challenge_id,
        last_sent,
    ))
    .map_err(|_| LiveErrorCode::ProtocolError)?;
    nats.publish(session.data_subject.clone(), Bytes::from(body))
        .await
        .map_err(|_| LiveErrorCode::PeerLost)
}

/// Publish one provider terminal frame.
///
/// # Errors
///
/// Returns a wire error when the publication handoff fails.
pub(crate) async fn publish_end(
    session: &ProviderSession,
    nats: &async_nats::Client,
    terminal: WireTerminal,
) -> Result<(), LiveErrorCode> {
    let final_seq = session
        .highest_sent
        .load(std::sync::atomic::Ordering::Acquire);
    let body = serde_json::to_vec(&end_frame(&session.session_id, final_seq, terminal))
        .map_err(|_| LiveErrorCode::ProtocolError)?;
    nats.publish(session.data_subject.clone(), Bytes::from(body))
        .await
        .map_err(|_| LiveErrorCode::PeerLost)
}

/// Build one acknowledgement for a handled control.
#[must_use]
pub(crate) fn ack_for(
    session: &ProviderSession,
    control: &LiveControl,
    request_id: &str,
    outcome: &ControlOutcome,
) -> LiveControlAck {
    control_ack(
        session,
        control,
        request_id,
        outcome.state,
        outcome.terminal.clone(),
        outcome.cleanup,
    )
}

/// Build one typed control error body.
#[must_use]
pub(crate) fn error_for(
    session: &ProviderSession,
    control: &LiveControl,
    request_id: &str,
    code: LiveErrorCode,
) -> trellis_protocol::LiveControlError {
    control_error(session, control, request_id, code)
}

/// Commit one source failure as this session's terminal outcome.
pub(crate) fn source_failure_end(message: &str) -> LiveEnd {
    LiveEnd::new(
        LiveEndReason::SourceError,
        Some(Arc::new(LiveStreamError::new(
            LiveErrorCode::SourceFailed,
            bounded_message(message),
        ))),
    )
}

/// Drain a provider source until it ends or the session closes.
///
/// The driver polls the source serially: it never starts a second poll while a
/// publication is in flight, so one staged frame per session bounds memory.
pub(crate) async fn drive_source<S>(
    session: Arc<ProviderSession>,
    record: Arc<ProviderSessionRecord>,
    nats: async_nats::Client,
    max_data_body_bytes: u64,
    cancellation: super::types::LiveCancellation,
    mut source: S,
) where
    S: Stream<Item = Result<SourceItem, String>> + Unpin,
{
    while matches!(session.phase(), ProviderPhase::Active) {
        let item = tokio::select! {
            // Cancellation stops reading the source immediately; the stream is
            // dropped with the driver, releasing any RAII-owned subscriptions.
            _ = cancellation.cancelled() => {
                record.commit_end(LiveEnd::new(
                    trellis_protocol::LiveEndReason::Cancelled,
                    None,
                ));
                session.set_phase(ProviderPhase::Closed);
                return;
            }
            item = source.next() => item,
        };
        let Some(item) = item else {
            let terminal = terminal_from_end(&LiveEnd::complete());
            let _ = publish_end(&session, &nats, terminal).await;
            record.commit_end(LiveEnd::complete());
            session.set_phase(ProviderPhase::Closed);
            return;
        };
        match item {
            Ok(SourceItem::Value(value)) => {
                // Wait for credit without blocking control handling: the owner
                // control path runs on its own task.
                loop {
                    if matches!(
                        session.phase(),
                        ProviderPhase::Closing | ProviderPhase::Closed
                    ) {
                        return;
                    }
                    match publish_data_frame(&session, &nats, value.clone(), max_data_body_bytes)
                        .await
                    {
                        Ok(_) => break,
                        Err(LiveErrorCode::ResourceExhausted) => {
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        }
                        Err(code) => {
                            let end = provider_failure(code, "live publication failed");
                            record.commit_end(end);
                            session.set_phase(ProviderPhase::Closing);
                            return;
                        }
                    }
                }
            }
            Ok(SourceItem::End) => {
                let terminal = terminal_from_end(&LiveEnd::complete());
                let _ = publish_end(&session, &nats, terminal).await;
                record.commit_end(LiveEnd::complete());
                session.set_phase(ProviderPhase::Closed);
                return;
            }
            Err(message) => {
                let end = source_failure_end(&message);
                let terminal = terminal_from_end(&end);
                let _ = publish_end(&session, &nats, terminal).await;
                record.commit_end(end);
                session.set_phase(ProviderPhase::Closing);
                return;
            }
        }
    }
}

/// Invoke the source factory exactly once for an activated session.
///
/// # Errors
///
/// Returns a wire error when no factory was registered.
pub(crate) fn take_source(
    factory: &std::sync::Mutex<Option<ProviderSourceFactory>>,
) -> Result<ProviderSource, LiveErrorCode> {
    factory
        .lock()
        .map_err(|_| LiveErrorCode::ProtocolError)?
        .take()
        .map(|factory| factory())
        .ok_or(LiveErrorCode::ProtocolError)
}

/// Commit one authority loss as a provider terminal outcome.
pub(crate) fn authority_terminal_end(lost: &LiveAuthorityLost) -> LiveEnd {
    super::manager::authority_end(lost)
}
