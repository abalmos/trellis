//! Provider-side live session engine: opening admission, signed offer, owner
//! control handling, activation, serialized publication and owned cleanup.
//!
//! One engine instance is driven by the service router per accepted opening
//! request. It never starts a domain source before the delivery-path challenge
//! round trip completes.

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use tokio::time::Instant;

use trellis_protocol::{
    LiveControl, LiveControlAck, LiveEndReason, LiveErrorCode, LiveSessionKind, LiveSessionState,
    WireTerminal,
};

use super::authority::{LiveAuthorityGuard, LiveAuthorityLost, PinnedPeerIdentity};
use super::deadlines::{DeadlineAction, LiveDeadlines};
use super::manager::{LiveSessionManager, ProviderPermit};
use super::provider::{
    bounded_message, challenge_frame, control_ack, control_error, data_frame, end_frame,
    provider_failure, receipt_for, terminal_from_end, ProviderPhase, ProviderReservationIdentity,
    ProviderSession, ProviderSessionSubjects,
};
use super::types::{feed_end_reason, CloseCleanupState, LiveEnd, LiveStreamError};

/// One accepted opening request ready to reserve a session.
pub(crate) struct ProviderOpenRequest {
    pub kind: LiveSessionKind,
    pub base_subject: String,
    pub open_id: String,
    pub consumer: PinnedPeerIdentity,
    pub consumer_max_payload_bytes: u64,
    pub canonical_open_hash: String,
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
    pub manager: Weak<LiveSessionManager>,
    pub permit: std::sync::Mutex<Option<ProviderPermit>>,
    pub own_guard: LiveAuthorityGuard,
    pub caller_guard: LiveAuthorityGuard,
    pub source_factory: std::sync::Mutex<Option<ProviderSourceFactory>>,
    pub cleanup: std::sync::Mutex<Vec<ProviderCleanup>>,
    pub terminal: std::sync::Mutex<Option<LiveEnd>>,
    /// Cancellation for this session's source scope.
    pub cancellation: super::types::LiveCancellation,
    pub source_started: AtomicBool,
    pub max_data_body_bytes: u64,
    /// Single monotonic deadline owner for this session.
    pub deadlines: std::sync::Mutex<LiveDeadlines>,
    /// Wakes the session timer when a deadline changes.
    pub deadline_notify: tokio::sync::Notify,
    /// Guards one-time closure and receipt recording.
    pub finished: AtomicBool,
    /// Server-side Feed ownership and its single end observation.
    pub feed: std::sync::Mutex<Option<crate::telemetry::lifecycle::FeedGuard>>,
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
    ) -> Result<(Arc<ProviderSession>, ProviderPermit), LiveErrorCode> {
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
        Ok((session, permit))
    }

    pub(crate) fn manager(&self) -> Result<Arc<LiveSessionManager>, LiveErrorCode> {
        self.manager.upgrade().ok_or(LiveErrorCode::Disconnected)
    }

    pub(crate) fn signed_headers(
        &self,
        subject: &str,
        body: &[u8],
    ) -> Result<async_nats::HeaderMap, LiveErrorCode> {
        self.own_guard
            .check_now()
            .map_err(|_| LiveErrorCode::PermissionDenied)?;
        self.caller_guard
            .check_now()
            .map_err(|_| LiveErrorCode::PermissionDenied)?;
        let manager = self.manager()?;
        let digest = manager
            .contexts_handle()
            .context_digest()
            .map_err(|_| LiveErrorCode::AuthorizationUnavailable)?;
        if digest.is_empty() {
            return Err(LiveErrorCode::AuthorizationUnavailable);
        }
        let proof = trellis_protocol::sign_live_server_proof(
            &digest,
            subject,
            body,
            manager.auth_handle().live_signing_key(),
        )
        .map_err(|_| LiveErrorCode::AuthorizationUnavailable)?;
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("authorization-context", digest.as_str());
        headers.insert("session-key", manager.auth_handle().session_key.as_str());
        headers.insert("trellis-live-proof", proof.as_str());
        Ok(headers)
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
                session_key: trellis_protocol::encode_subject_token(&provider.session_key),
                principal_id: provider.principal_id.clone(),
                participant_id: provider.participant_id.clone(),
                deployment_id: provider.deployment_id.clone().unwrap_or_default(),
                instance_id: provider.instance_id.clone().unwrap_or_default(),
            },
            consumer: trellis_protocol::LiveOfferConsumer {
                connection_id: consumer.connection_id.clone(),
                session_key: trellis_protocol::encode_subject_token(&consumer.session_key),
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
        now: Instant,
    ) -> Result<ControlOutcome, LiveErrorCode> {
        match control {
            LiveControl::Activate(_) => {
                let phase = self.session.phase();
                if !matches!(phase, ProviderPhase::Offered | ProviderPhase::Activating) {
                    return Err(LiveErrorCode::StaleControl);
                }
                // A duplicate activate re-delivers the same outstanding challenge
                // rather than minting a second nonce.
                if matches!(phase, ProviderPhase::Activating) {
                    let existing = self
                        .session
                        .challenge
                        .lock()
                        .ok()
                        .and_then(|challenge| challenge.clone());
                    if let Some(challenge) = existing {
                        return Ok(ControlOutcome {
                            state: LiveSessionState::Activating,
                            terminal: None,
                            cleanup: None,
                            challenge: Some(challenge.challenge_id),
                            start_source: false,
                        });
                    }
                }
                let challenge_id =
                    trellis_protocol::generate_nonce().map_err(|_| LiveErrorCode::ProtocolError)?;
                let last_sent_seq = self.session.highest_sent.load(Ordering::Acquire);
                self.session.set_phase(ProviderPhase::Activating);
                if let Ok(mut challenge) = self.session.challenge.lock() {
                    *challenge = Some(super::provider::ChallengeState {
                        challenge_id: challenge_id.clone(),
                        last_sent_seq,
                    });
                }
                self.with_deadlines(|deadlines| {
                    deadlines.begin_activating(now, challenge_id.clone());
                });
                self.deadline_changed();
                Ok(ControlOutcome {
                    state: LiveSessionState::Activating,
                    terminal: None,
                    cleanup: None,
                    challenge: Some(challenge_id),
                    start_source: false,
                })
            }
            LiveControl::Pulse(pulse) => {
                let activating = matches!(self.session.phase(), ProviderPhase::Activating);
                let accepted = self
                    .with_deadlines(|deadlines| {
                        if deadlines.outstanding_challenge() != Some(pulse.challenge_id.as_str()) {
                            return false;
                        }
                        if activating {
                            deadlines.commit_active(now, true);
                        } else {
                            deadlines.fresh_round_trip(now, true);
                        }
                        true
                    })
                    .unwrap_or(false);
                if !accepted {
                    return Err(LiveErrorCode::InvalidChallenge);
                }
                if let Ok(mut challenge) = self.session.challenge.lock() {
                    *challenge = None;
                }
                if activating {
                    self.session.set_phase(ProviderPhase::Active);
                }
                self.apply_credit_and_note(
                    pulse.received_seq.get(),
                    pulse.consumed_seq.get(),
                    now,
                )?;
                self.deadline_changed();
                Ok(ControlOutcome {
                    state: LiveSessionState::Active,
                    terminal: None,
                    cleanup: None,
                    challenge: None,
                    start_source: activating && !self.source_started.swap(true, Ordering::AcqRel),
                })
            }
            LiveControl::Ack(ack) => {
                self.apply_credit_and_note(ack.received_seq.get(), ack.consumed_seq.get(), now)?;
                Ok(ControlOutcome {
                    state: current_state(self.session.phase()),
                    terminal: None,
                    cleanup: None,
                    challenge: None,
                    start_source: false,
                })
            }
            LiveControl::Close(close) => {
                self.session
                    .apply_credit(close.received_seq.get(), close.consumed_seq.get())
                    .ok();
                self.commit_end(LiveEnd::new(LiveEndReason::Cancelled, None));
                self.begin_close(now);
                let cleanup = self.run_owned_cleanup().await;
                let outcome = self.finish_closed(now, cleanup).await;
                Ok(ControlOutcome {
                    state: LiveSessionState::Closed,
                    terminal: Some(terminal_from_end(&outcome)),
                    cleanup: Some(match cleanup {
                        CloseCleanupState::Complete => trellis_protocol::CleanupStatus::Complete,
                        _ => trellis_protocol::CleanupStatus::Incomplete,
                    }),
                    challenge: None,
                    start_source: false,
                })
            }
            LiveControl::EndAck(ack) => {
                self.session
                    .apply_credit(ack.received_seq.get(), ack.consumed_seq.get())
                    .ok();
                self.commit_end(LiveEnd::new(LiveEndReason::Cancelled, None));
                self.begin_close(now);
                let cleanup = self.run_owned_cleanup().await;
                let outcome = self.finish_closed(now, cleanup).await;
                Ok(ControlOutcome {
                    state: LiveSessionState::Closed,
                    terminal: Some(terminal_from_end(&outcome)),
                    cleanup: Some(match cleanup {
                        CloseCleanupState::Complete => trellis_protocol::CleanupStatus::Complete,
                        _ => trellis_protocol::CleanupStatus::Incomplete,
                    }),
                    challenge: None,
                    start_source: false,
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
        let _permit = self.permit.lock().ok().and_then(|mut slot| slot.take());
        receipt_for(
            &self.session,
            self.committed_end().reason(),
            cleanup,
            now_ms,
        )
    }

    /// Authenticate and apply one owner-control message, then reply if safe.
    pub(crate) async fn dispatch_control(
        self: &Arc<Self>,
        nats: &async_nats::Client,
        message: async_nats::Message,
    ) {
        let Some(reply) = message.reply.clone() else {
            return;
        };
        let Some(headers) = message.headers.as_ref() else {
            return;
        };
        if self
            .caller_guard
            .verify_control_request(
                message.subject.as_str(),
                reply.as_str(),
                &message.payload,
                headers,
            )
            .is_err()
        {
            return;
        }
        let Ok(control) = trellis_protocol::parse_live_control(&message.payload) else {
            return;
        };
        if control.session_id() != self.session.session_id {
            return;
        }
        if message.subject.as_str() != self.session.control_subject {
            return;
        }
        let now = Instant::now();
        let request_id = headers
            .get("request-id")
            .map_or_else(String::new, ToString::to_string);
        match self.handle_control(&control, now).await {
            Ok(outcome) => {
                let ack = ack_for(&self.session, &control, &request_id, &outcome);
                let Ok(body) = serde_json::to_vec(&ack) else {
                    return;
                };
                let Ok(signed) = self.signed_headers(reply.as_str(), &body) else {
                    return;
                };
                // Only a successfully handed-off signed acknowledgement permits
                // the source to start (R11). A failed handoff closes the
                // reservation instead of starting a source on a later retry.
                if nats
                    .publish_with_headers(reply, signed, Bytes::from(body))
                    .await
                    .is_err()
                {
                    self.commit_end(provider_failure(
                        LiveErrorCode::PeerLost,
                        "activation acknowledgement could not be handed off",
                    ));
                    self.begin_close(now);
                    let cleanup = self.run_owned_cleanup().await;
                    let _ = self.finish_closed(now, cleanup).await;
                    return;
                }
                if let Some(challenge_id) = outcome.challenge.as_ref() {
                    let _ = publish_challenge(self, nats, challenge_id).await;
                }
                if outcome.start_source {
                    if let Ok(mut feed) = self.feed.lock() {
                        *feed = Some(crate::telemetry::lifecycle::FeedGuard::acquire("server"));
                    }
                    if let Ok(source) = take_source(&self.source_factory) {
                        let driver_record = Arc::clone(self);
                        let driver_nats = nats.clone();
                        let driver_cancellation = self.cancellation_handle();
                        let max_data_body_bytes = self.max_data_body_bytes;
                        tokio::spawn(async move {
                            drive_source(
                                Arc::clone(&driver_record.session),
                                driver_record,
                                driver_nats,
                                max_data_body_bytes,
                                driver_cancellation,
                                source,
                            )
                            .await;
                        });
                    }
                }
            }
            Err(code) => {
                let error = error_for(&self.session, &control, &request_id, code);
                let Ok(body) = serde_json::to_vec(&error) else {
                    return;
                };
                let Ok(signed) = self.signed_headers(reply.as_str(), &body) else {
                    return;
                };
                let _ = nats
                    .publish_with_headers(reply, signed, Bytes::from(body))
                    .await;
            }
        }
    }

    pub(crate) async fn finish_closed(&self, now: Instant, cleanup: CloseCleanupState) -> LiveEnd {
        if self.finished.swap(true, Ordering::AcqRel) {
            return self.committed_end();
        }
        let now_ms = crate::client::now_iat_seconds() * 1_000;
        let receipt = self.tombstone(cleanup, now_ms);
        self.with_deadlines(|deadlines| deadlines.closed(now));
        self.deadline_changed();
        if let Ok(manager) = self.manager() {
            manager.insert_receipt(receipt);
            manager.remove_provider_session(&self.session.session_id);
        }
        let reason = feed_end_reason(self.committed_end().reason());
        if let Ok(mut feed) = self.feed.lock() {
            match feed.as_mut() {
                Some(guard) => guard.finish(reason),
                None => crate::telemetry::lifecycle::record_feed_end("server", reason),
            }
        }
        self.committed_end()
    }

    /// Lock the deadline owner and apply one transition.
    fn with_deadlines<R>(&self, f: impl FnOnce(&mut LiveDeadlines) -> R) -> Option<R> {
        self.deadlines
            .lock()
            .ok()
            .map(|mut deadlines| f(&mut deadlines))
    }

    /// Wake the session timer after a deadline changed.
    fn deadline_changed(&self) {
        self.deadline_notify.notify_one();
    }

    /// Return the next deadline for the session timer, if any.
    #[must_use]
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        self.deadlines
            .lock()
            .ok()
            .and_then(|deadlines| deadlines.next_due())
    }

    /// Apply one accepted credit cursor and refresh the stall/liveness clocks.
    fn apply_credit_and_note(
        &self,
        received: u64,
        consumed: u64,
        now: Instant,
    ) -> Result<(), LiveErrorCode> {
        let before = self.session.highest_consumed.load(Ordering::Acquire);
        self.session.apply_credit(received, consumed)?;
        let after = self.session.highest_consumed.load(Ordering::Acquire);
        let advanced = after > before;
        self.with_deadlines(|deadlines| {
            if advanced {
                deadlines.note_stall_reset(now);
            }
            if self.session.outstanding_empty() {
                deadlines.outstanding_cleared();
            }
        });
        self.deadline_changed();
        Ok(())
    }

    /// Enter the bounded close exchange.
    pub(crate) fn begin_close(&self, now: Instant) {
        self.session.set_phase(ProviderPhase::Closing);
        self.with_deadlines(|deadlines| deadlines.begin_closing(now));
        self.deadline_changed();
    }

    /// Record that one application frame became outstanding.
    pub(crate) fn note_data_admitted(&self, now: Instant) {
        self.with_deadlines(|deadlines| deadlines.data_admitted(now));
        self.deadline_changed();
    }

    /// Evaluate and perform one due deadline action.
    ///
    /// Returns `true` when the session finished and its timer should stop. The
    /// driver performs the action and then recomputes the next due event.
    pub(crate) async fn evaluate_deadlines(self: &Arc<Self>, nats: &async_nats::Client) -> bool {
        let now = Instant::now();
        let action = self
            .deadlines
            .lock()
            .ok()
            .and_then(|deadlines| deadlines.evaluate(now));
        match action {
            None => false,
            Some(DeadlineAction::ReservationExpired) => {
                self.commit_end(LiveEnd::new(LiveEndReason::SetupTimeout, None));
                self.begin_close(now);
                let cleanup = self.run_owned_cleanup().await;
                let _ = self.finish_closed(now, cleanup).await;
                true
            }
            Some(DeadlineAction::ChallengeRetry) => {
                let challenge = self
                    .session
                    .challenge
                    .lock()
                    .ok()
                    .and_then(|challenge| challenge.clone());
                if let Some(challenge) = challenge {
                    let _ = publish_challenge(self, nats, &challenge.challenge_id).await;
                }
                self.with_deadlines(|deadlines| deadlines.rearm_challenge_retry(now));
                self.deadline_changed();
                false
            }
            Some(DeadlineAction::ChallengeDue) => {
                let Ok(challenge_id) = trellis_protocol::generate_nonce() else {
                    self.commit_end(provider_failure(
                        LiveErrorCode::ProtocolError,
                        "challenge nonce unavailable",
                    ));
                    self.begin_close(now);
                    let cleanup = self.run_owned_cleanup().await;
                    let _ = self.finish_closed(now, cleanup).await;
                    return true;
                };
                let last_sent_seq = self.session.highest_sent.load(Ordering::Acquire);
                if let Ok(mut challenge) = self.session.challenge.lock() {
                    *challenge = Some(super::provider::ChallengeState {
                        challenge_id: challenge_id.clone(),
                        last_sent_seq,
                    });
                }
                self.with_deadlines(|deadlines| {
                    deadlines.begin_challenge(now, challenge_id.clone())
                });
                self.deadline_changed();
                let _ = publish_challenge(self, nats, &challenge_id).await;
                false
            }
            Some(DeadlineAction::PeerInactive) => {
                self.commit_end(provider_failure(
                    LiveErrorCode::PeerLost,
                    "live consumer silent past the inactivity bound",
                ));
                self.begin_close(now);
                let cleanup = self.run_owned_cleanup().await;
                let _ = self.finish_closed(now, cleanup).await;
                true
            }
            Some(DeadlineAction::ConsumerStalled) => {
                self.commit_end(provider_failure(
                    LiveErrorCode::ConsumerSlow,
                    "live consumer did not consume outstanding data",
                ));
                self.begin_close(now);
                let cleanup = self.run_owned_cleanup().await;
                let _ = self.finish_closed(now, cleanup).await;
                true
            }
            Some(DeadlineAction::CloseExchangeElapsed) => {
                self.session.set_phase(ProviderPhase::Closing);
                let cleanup = self.run_owned_cleanup().await;
                let _ = self.finish_closed(now, cleanup).await;
                true
            }
            Some(DeadlineAction::CreditDue) => {
                // The provider never schedules consumer credit; clear a stale
                // deadline so the timer cannot spin on a past instant.
                self.with_deadlines(|deadlines| deadlines.credit_sent());
                self.deadline_changed();
                false
            }
            Some(DeadlineAction::CleanupGraceElapsed) => {
                self.with_deadlines(|deadlines| deadlines.mark_cleanup_grace_elapsed());
                self.deadline_changed();
                false
            }
        }
    }
}

/// Outcome of one handled control.
pub(crate) struct ControlOutcome {
    pub state: LiveSessionState,
    pub terminal: Option<WireTerminal>,
    pub cleanup: Option<trellis_protocol::CleanupStatus>,
    /// One challenge to publish on the data subject after the acknowledgement.
    pub challenge: Option<String>,
    /// Start the source after a verified first Pulse acknowledgement is sent.
    pub start_source: bool,
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
    record: &ProviderSessionRecord,
    nats: &async_nats::Client,
    value: serde_json::Value,
    max_data_body_bytes: u64,
) -> Result<u64, LiveErrorCode> {
    let session = &record.session;
    let seq = session
        .highest_sent
        .load(Ordering::Acquire)
        .saturating_add(1);
    let body = serde_json::to_vec(&data_frame(&session.session_id, seq, value))
        .map_err(|_| LiveErrorCode::ProtocolError)?;
    let admitted = session.admit_frame(body.len() as u64, max_data_body_bytes)?;
    if admitted != seq {
        return Err(LiveErrorCode::ProtocolError);
    }
    let headers = record.signed_headers(&session.data_subject, &body)?;
    nats.publish_with_headers(session.data_subject.clone(), headers, Bytes::from(body))
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
    record: &ProviderSessionRecord,
    nats: &async_nats::Client,
    challenge_id: &str,
) -> Result<(), LiveErrorCode> {
    let session = &record.session;
    let last_sent = session.highest_sent.load(Ordering::Acquire);
    let body = serde_json::to_vec(&challenge_frame(
        &session.session_id,
        challenge_id,
        last_sent,
    ))
    .map_err(|_| LiveErrorCode::ProtocolError)?;
    let headers = record.signed_headers(&session.data_subject, &body)?;
    nats.publish_with_headers(session.data_subject.clone(), headers, Bytes::from(body))
        .await
        .map_err(|_| LiveErrorCode::PeerLost)
}

/// Publish one provider terminal frame.
///
/// # Errors
///
/// Returns a wire error when the publication handoff fails.
pub(crate) async fn publish_end(
    record: &ProviderSessionRecord,
    nats: &async_nats::Client,
    terminal: WireTerminal,
) -> Result<(), LiveErrorCode> {
    let session = &record.session;
    let final_seq = session.highest_sent.load(Ordering::Acquire);
    let body = serde_json::to_vec(&end_frame(&session.session_id, final_seq, terminal))
        .map_err(|_| LiveErrorCode::ProtocolError)?;
    let headers = record.signed_headers(&session.data_subject, &body)?;
    nats.publish_with_headers(session.data_subject.clone(), headers, Bytes::from(body))
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
                record.begin_close(Instant::now());
                return;
            }
            item = source.next() => item,
        };
        let Some(item) = item else {
            let terminal = terminal_from_end(&LiveEnd::complete());
            let _ = publish_end(&record, &nats, terminal).await;
            record.commit_end(LiveEnd::complete());
            record.begin_close(Instant::now());
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
                    match publish_data_frame(&record, &nats, value.clone(), max_data_body_bytes)
                        .await
                    {
                        Ok(_) => {
                            record.note_data_admitted(Instant::now());
                            break;
                        }
                        Err(LiveErrorCode::ResourceExhausted) => {
                            tokio::select! {
                                _ = cancellation.cancelled() => {
                                    record.commit_end(LiveEnd::new(
                                        trellis_protocol::LiveEndReason::Cancelled,
                                        None,
                                    ));
                                    record.begin_close(Instant::now());
                                    return;
                                }
                                _ = session.credit.notified() => {}
                            }
                        }
                        Err(code) => {
                            let end = provider_failure(code, "live publication failed");
                            record.commit_end(end);
                            record.begin_close(Instant::now());
                            return;
                        }
                    }
                }
            }
            Ok(SourceItem::End) => {
                let terminal = terminal_from_end(&LiveEnd::complete());
                let _ = publish_end(&record, &nats, terminal).await;
                record.commit_end(LiveEnd::complete());
                record.begin_close(Instant::now());
                return;
            }
            Err(message) => {
                let end = source_failure_end(&message);
                let terminal = terminal_from_end(&end);
                let _ = publish_end(&record, &nats, terminal).await;
                record.commit_end(end);
                record.begin_close(Instant::now());
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
