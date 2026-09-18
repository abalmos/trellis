//! Consumer-side live subscription: prepared handle, activation, control pump,
//! bounded ingress, credit and close.

use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use futures_util::{Stream, StreamExt};

use trellis_protocol::{
    LiveControl, LiveControlAck, LiveControlActivate, LiveControlClose, LiveControlCredit,
    LiveControlEndAck, LiveControlPulse, LiveEndReason, LiveErrorCode, LiveFrame, LiveOffer, U64s,
    ACK_FRAME_THRESHOLD, ACK_MAX_DELAY_MS, CHALLENGE_RETRY_MS, CLOSE_EXCHANGE_MS, CLOSE_RETRY_MS,
    CONSUMER_STALL_MS, HEARTBEAT_INTERVAL_MS, PEER_INACTIVITY_MS, WINDOW_BYTES, WINDOW_FRAMES,
};

use super::authority::{LiveAuthorityGuard, LiveAuthorityLost};
use super::types::{
    end_reason_for_code, CloseCleanupState, CloseRemoteState, LiveCancellation, LiveCloseReceipt,
    LiveEnd, LiveStreamError,
};
use crate::client::TrellisClientError;

/// One verified and admitted application item.
pub(crate) struct AdmittedItem<T> {
    pub value: T,
    pub encoded_len: u64,
}

/// Consumer-visible session phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConsumerPhase {
    Prepared,
    Activating,
    Active,
    Draining,
    Closing,
    Closed,
}

/// The consumer session core shared by the handle and its pump.
pub(crate) struct ConsumerCore<T> {
    pub(crate) session_id: String,
    pub(crate) phase: Mutex<ConsumerPhase>,
    /// Bounded application queue; count and encoded bytes stay within window.
    pub(crate) queue: Mutex<VecDeque<AdmittedItem<T>>>,
    pub(crate) queued_bytes: AtomicU64,
    pub(crate) received_seq: AtomicU64,
    pub(crate) consumed_seq: AtomicU64,
    pub(crate) cancelled: AtomicBool,
    pub(crate) end: Mutex<Option<LiveEnd>>,
    pub(crate) end_notify: tokio::sync::Notify,
    /// One pending item already admitted to the application.
    pub(crate) pending: Mutex<Option<AdmittedItem<T>>>,
    /// Waker registered by the application's last pending poll.
    pub(crate) waker: Mutex<Option<std::task::Waker>>,
    /// Whether the committed failure was already yielded once.
    pub(crate) error_reported: AtomicBool,
    /// A verified remote normal end, pending until the queue drains.
    pub(crate) pending_end: Mutex<Option<LiveEnd>>,
}

impl<T> ConsumerCore<T> {
    pub(crate) fn new(session_id: String) -> Self {
        Self {
            session_id,
            phase: Mutex::new(ConsumerPhase::Prepared),
            queue: Mutex::new(VecDeque::new()),
            queued_bytes: AtomicU64::new(0),
            received_seq: AtomicU64::new(0),
            consumed_seq: AtomicU64::new(0),
            cancelled: AtomicBool::new(false),
            end: Mutex::new(None),
            end_notify: tokio::sync::Notify::new(),
            pending: Mutex::new(None),
            waker: Mutex::new(None),
            error_reported: AtomicBool::new(false),
            pending_end: Mutex::new(None),
        }
    }

    /// Record one verified remote normal end for draining.
    pub(crate) fn set_pending_end(&self, end: LiveEnd) {
        if let Ok(mut slot) = self.pending_end.lock() {
            if slot.is_none() {
                *slot = Some(end);
            }
        }
        self.wake();
    }

    /// Take the pending normal end once the queue is empty.
    ///
    /// Returns the committed end when draining completed, or `None` while
    /// items still wait to be handed to the application.
    pub(crate) fn drain_complete(&self) -> Option<LiveEnd> {
        let queue_empty = self.queue.lock().is_ok_and(|queue| queue.is_empty());
        if !queue_empty {
            return None;
        }
        let pending = self
            .pending_end
            .lock()
            .ok()
            .and_then(|mut slot| slot.take())?;
        self.commit_end(pending.clone());
        Some(pending)
    }

    /// Return the consumed cursor.
    pub(crate) fn consumed_seq(&self) -> u64 {
        self.consumed_seq.load(Ordering::Acquire)
    }

    /// Return the received cursor.
    pub(crate) fn received_seq(&self) -> u64 {
        self.received_seq.load(Ordering::Acquire)
    }

    /// Record one verified received data frame.
    pub(crate) fn record_received(&self) {
        self.received_seq.fetch_add(1, Ordering::AcqRel);
    }

    /// Return whether a committed failure was already yielded.
    pub(crate) fn reported_error(&self) -> bool {
        self.error_reported.load(Ordering::Acquire)
    }

    /// Mark the committed failure as yielded.
    pub(crate) fn mark_error_reported(&self) {
        self.error_reported.store(true, Ordering::Release);
    }

    /// Wake the application's pending poll, if any.
    pub(crate) fn wake(&self) {
        let waker = self.waker.lock().ok().and_then(|mut slot| slot.take());
        if let Some(waker) = waker {
            waker.wake();
        }
    }

    pub(crate) fn phase(&self) -> ConsumerPhase {
        self.phase
            .lock()
            .map_or(ConsumerPhase::Closed, |phase| *phase)
    }

    pub(crate) fn set_phase(&self, phase: ConsumerPhase) {
        if let Ok(mut current) = self.phase.lock() {
            *current = phase;
        }
    }

    /// Commit one terminal outcome once; later calls are ignored.
    pub(crate) fn commit_end(&self, end: LiveEnd) {
        let Ok(mut slot) = self.end.lock() else {
            return;
        };
        if slot.is_none() {
            *slot = Some(end);
            drop(slot);
            self.set_phase(ConsumerPhase::Closed);
            self.end_notify.notify_waiters();
            self.wake();
        }
    }

    /// Return the committed terminal outcome, if any.
    pub(crate) fn committed_end(&self) -> Option<LiveEnd> {
        self.end.lock().ok().and_then(|slot| slot.clone())
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.committed_end().is_some()
    }

    /// Admit one verified application item within the wire window.
    ///
    /// Returns `false` when the bounded queue would exceed the window; the
    /// caller closes the session with `consumer_slow`.
    pub(crate) fn admit(&self, item: AdmittedItem<T>) -> bool {
        let queued = self.queued_bytes.load(Ordering::Acquire);
        let (count, bytes) = {
            let Ok(queue) = self.queue.lock() else {
                return false;
            };
            (queue.len() as u64, queued)
        };
        if count + 1 > WINDOW_FRAMES || bytes + item.encoded_len > WINDOW_BYTES {
            return false;
        }
        let encoded_len = item.encoded_len;
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push_back(item);
        drop(queue);
        self.queued_bytes.fetch_add(encoded_len, Ordering::AcqRel);
        self.wake();
        true
    }

    /// Take one queued item and advance the consumed cursor.
    pub(crate) fn consume(&self) -> Option<AdmittedItem<T>> {
        let item = {
            let Ok(mut queue) = self.queue.lock() else {
                return None;
            };
            queue.pop_front()
        };
        if let Some(item) = item.as_ref() {
            self.queued_bytes
                .fetch_sub(item.encoded_len, Ordering::AcqRel);
            self.consumed_seq.fetch_add(1, Ordering::AcqRel);
        }
        item
    }

    /// Discard every queued item without advancing the consumed cursor.
    ///
    /// Abandoned items are not handoff to the application, so they neither
    /// grant credit nor count as consumed.
    pub(crate) fn discard_queue(&self) {
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.clear();
        self.queued_bytes.store(0, Ordering::Release);
    }
}

/// The public Rust owned live subscription handle.
pub struct LiveSubscription<T> {
    pub(crate) core: Arc<ConsumerCore<T>>,
    pub(crate) _drain: tokio::task::JoinHandle<()>,
    pub(crate) control: Arc<ConsumerControl>,
    pub(crate) cancellation: LiveCancellation,
    pub(crate) closed_once: Arc<tokio::sync::Notify>,
    /// Set once the first poll installed the activation path.
    pub(crate) activated: bool,
}

impl<T> LiveSubscription<T> {
    /// Construct one owned handle from its prepared core and running pump.
    ///
    /// The core's bounded queue is the single ingress; the handle's receiver is
    /// fed by the core's consumption path, so `next()` both hands an item to the
    /// application and releases its credit.
    pub(crate) fn new(
        core: Arc<ConsumerCore<T>>,
        drain: tokio::task::JoinHandle<()>,
        control: Arc<ConsumerControl>,
        cancellation: LiveCancellation,
    ) -> Self {
        Self {
            core,
            _drain: drain,
            control,
            cancellation,
            closed_once: Arc::new(tokio::sync::Notify::new()),
            activated: false,
        }
    }

    /// Return the committed terminal outcome, waiting for closure.
    pub async fn closed(&self) -> LiveEnd {
        loop {
            if let Some(end) = self.core.committed_end() {
                return end;
            }
            self.core.end_notify.notified().await;
        }
    }

    /// Return whether the subscription has completed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.core.is_closed()
    }

    /// Return the session identifier (redacted from metrics by policy).
    #[must_use]
    pub(crate) fn session_id(&self) -> &str {
        &self.core.session_id
    }

    /// Explicitly close the observation and await the bounded close exchange.
    pub async fn close(&mut self) -> Result<LiveCloseReceipt, TrellisClientError> {
        self.cancellation.cancel();
        let receipt = self.control.begin_close().await;
        Ok(receipt)
    }
}

impl<T> Drop for LiveSubscription<T> {
    fn drop(&mut self) {
        // Synchronous local fence: no new yields, no new scheduling. Remote
        // cleanup is best-effort through the control owner's runtime handle.
        self.core.cancelled.store(true, Ordering::Release);
        self.cancellation.cancel();
        self.control.schedule_local_drop_cleanup();
        self.core.discard_queue();
        self._drain.abort();
    }
}

impl<T> Stream for LiveSubscription<T> {
    type Item = Result<T, TrellisClientError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Queued application bodies are delivered in order, and each delivery
        // advances the consumed cursor that releases credit.
        if let Some(item) = self.core.consume() {
            return Poll::Ready(Some(Ok(item.value)));
        }
        // A verified normal end resolves once every queued item was handed out.
        if self.core.drain_complete().is_some() {
            return Poll::Ready(None);
        }
        match self.core.committed_end() {
            Some(end) => match end.error() {
                // A committed failure is reported once; later polls return done.
                Some(error) if !end.is_complete() && !self.core.reported_error() => {
                    self.core.mark_error_reported();
                    Poll::Ready(Some(Err(TrellisClientError::Live(error.clone()))))
                }
                _ => Poll::Ready(None),
            },
            None => {
                // Register for the next admission or terminal outcome, then
                // re-check to avoid a lost wakeup between the two steps.
                if let Ok(mut slot) = self.core.waker.lock() {
                    *slot = Some(cx.waker().clone());
                }
                if self.core.consume().is_some() || self.core.committed_end().is_some() {
                    cx.waker().wake_by_ref();
                }
                Poll::Pending
            }
        }
    }
}

/// Control owner for one consumer session.
///
/// Owns the NATS control path: fresh request proofs per attempt, logical
/// idempotence, close exchange and local shutdown notifications. It holds the
/// cloneable transport and signing material directly, so it never keeps the
/// whole connection client alive through a reference cycle.
pub(crate) struct ConsumerControl {
    pub(crate) nats: async_nats::Client,
    pub(crate) auth: Arc<crate::client::SessionAuth>,
    pub(crate) contexts: Arc<crate::client::AuthorizationContextCache>,
    pub(crate) inbox_prefix: String,
    pub(crate) session_id: String,
    pub(crate) control_subject: String,
    /// Provider runtime key pinned at offer acceptance.
    pub(crate) pinned_session_key: String,
    /// Provider identity tuple pinned at offer acceptance.
    pub(crate) pinned_identity: super::authority::PinnedPeerIdentity,
    pub(crate) close_started: AtomicBool,
    pub(crate) last_control_seq: AtomicU64,
}

impl ConsumerControl {
    /// Reserve the next logical control sequence for this session.
    pub(crate) fn next_control_seq(&self) -> u64 {
        self.last_control_seq.fetch_add(1, Ordering::AcqRel) + 1
    }
}

impl ConsumerControl {
    fn signed_headers(
        &self,
        reply: &str,
        body: &[u8],
    ) -> Result<async_nats::HeaderMap, TrellisClientError> {
        let context_digest = self
            .contexts
            .context_digest()
            .map_err(|error| TrellisClientError::AuthorizationUnavailable(error.to_string()))?;
        let iat = crate::client::now_iat_seconds() as i64;
        let request_id = crate::client::new_request_id();
        let proof = self.auth.create_request_proof(
            &context_digest,
            &self.control_subject,
            reply,
            body,
            iat,
            &request_id,
        )?;
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("authorization-context", context_digest.as_str());
        headers.insert("session-key", self.auth.session_key.as_str());
        headers.insert("proof", proof.as_str());
        headers.insert("iat", iat.to_string().as_str());
        headers.insert("request-id", request_id.as_str());
        Ok(headers)
    }
}

impl ConsumerControl {
    pub(crate) fn schedule_local_drop_cleanup(&self) {
        // Only schedule when a runtime is already available; Drop must never
        // create one or block on async work.
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            return;
        };
        if self.close_started.swap(true, Ordering::AcqRel) {
            return;
        }
        let nats = self.nats.clone();
        let subject = self.control_subject.clone();
        let session_id = self.session_id.clone();
        handle.spawn(async move {
            let _ = nats
                .publish(subject, control_close_bytes(&session_id, 1))
                .await;
        });
    }

    /// Send one logical control and await its signed acknowledgement.
    ///
    /// Each attempt uses a fresh request proof and reply inbox; the logical
    /// body and sequence stay identical across retries. One attempt has the
    /// protocol control-timeout budget and cannot exceed the caller's deadline.
    pub(crate) async fn send_control(
        &self,
        control: &LiveControl,
        control_seq: u64,
    ) -> Result<LiveControlAck, TrellisClientError> {
        self.last_control_seq.store(control_seq, Ordering::Release);
        let body = control_body_bytes(control)?;
        let reply = format!(
            "{}.{}",
            self.inbox_prefix,
            CONTROL_INBOX_COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let headers = self.signed_headers(&reply, &body)?;
        let mut subscriber = self
            .nats
            .subscribe(reply.clone())
            .await
            .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;
        self.nats
            .publish_with_reply_and_headers(self.control_subject.clone(), reply, headers, body)
            .await
            .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;
        let response = tokio::time::timeout(
            std::time::Duration::from_millis(trellis_protocol::CONTROL_TIMEOUT_MS),
            subscriber.next(),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .ok_or(TrellisClientError::Timeout)?;
        parse_control_response(&response)
    }

    /// Send a bounded close exchange and report remote confirmation.
    pub(crate) async fn begin_close(&self) -> LiveCloseReceipt {
        let end = LiveEnd::new(LiveEndReason::Cancelled, None);
        let _ = self.close_started.swap(true, Ordering::AcqRel);
        let cleanup = self
            .retry_until_confirmed()
            .await
            .unwrap_or(CloseCleanupState::Unknown);
        let remote = if cleanup == CloseCleanupState::Unknown {
            CloseRemoteState::Unconfirmed
        } else {
            CloseRemoteState::Confirmed
        };
        LiveCloseReceipt::new(end, remote, cleanup)
    }

    /// Retry the same logical close within one total exchange budget.
    async fn retry_until_confirmed(&self) -> Option<CloseCleanupState> {
        let deadline = tokio::time::Instant::now() + close_exchange();
        loop {
            let control = close_control(
                &self.session_id,
                self.last_control_seq
                    .load(Ordering::Acquire)
                    .saturating_add(1),
                trellis_protocol::ConsumerCloseReason::Cancelled,
                0,
                0,
            );
            let attempt_deadline =
                deadline.min(tokio::time::Instant::now() + std::time::Duration::from_millis(1_000));
            let attempt = tokio::time::timeout_at(
                attempt_deadline,
                self.send_control(&control, control.control_seq().get()),
            )
            .await;
            match attempt {
                Ok(Ok(ack)) => {
                    return Some(match ack.cleanup {
                        Some(trellis_protocol::CleanupStatus::Complete) => {
                            CloseCleanupState::Complete
                        }
                        Some(trellis_protocol::CleanupStatus::Incomplete) => {
                            CloseCleanupState::Incomplete
                        }
                        None => CloseCleanupState::Unknown,
                    });
                }
                Ok(Err(_)) | Err(_) if tokio::time::Instant::now() < deadline => {
                    tokio::time::sleep(std::time::Duration::from_millis(1_000)).await;
                }
                _ => return None,
            }
        }
    }
}

/// Serialize one control body for the wire.
pub(crate) fn control_body_bytes(
    control: &LiveControl,
) -> Result<bytes::Bytes, TrellisClientError> {
    let value = match control {
        LiveControl::Activate(body) => serde_json::to_value(body)?,
        LiveControl::Pulse(body) => serde_json::to_value(body)?,
        LiveControl::Ack(body) => serde_json::to_value(body)?,
        LiveControl::Close(body) => serde_json::to_value(body)?,
        LiveControl::EndAck(body) => serde_json::to_value(body)?,
    };
    Ok(bytes::Bytes::from(serde_json::to_vec(&value)?))
}

fn control_close_bytes(session_id: &str, control_seq: u64) -> bytes::Bytes {
    let body = serde_json::json!({
        "format": trellis_protocol::LIVE_VERSION,
        "type": "control",
        "sessionId": session_id,
        "controlSeq": control_seq.to_string(),
        "action": "close",
        "reason": "local_shutdown",
        "receivedSeq": "0",
        "consumedSeq": "0",
    });
    bytes::Bytes::from(serde_json::to_vec(&body).expect("static control body"))
}

/// Verified offer data retained by a prepared consumer handle.
pub(crate) struct PreparedOffer {
    pub offer: LiveOffer,
    pub authority: LiveAuthorityGuard,
}

/// Build one sanitized terminal outcome for a consumer-side failure.
#[must_use]
pub(crate) fn consumer_failure(code: LiveErrorCode, message: impl Into<String>) -> LiveEnd {
    LiveEnd::new(
        end_reason_for_code(code),
        Some(Arc::new(LiveStreamError::new(code, message))),
    )
}

/// Build one authority-loss terminal outcome.
#[must_use]
pub(crate) fn authority_failure(lost: &LiveAuthorityLost) -> LiveEnd {
    super::manager::authority_end(lost)
}

/// Return the peer-inactivity deadline as a duration.
#[must_use]
pub(crate) fn peer_inactivity() -> std::time::Duration {
    std::time::Duration::from_millis(PEER_INACTIVITY_MS)
}

/// Return the consumer-stall deadline as a duration.
#[must_use]
pub(crate) fn consumer_stall() -> std::time::Duration {
    std::time::Duration::from_millis(CONSUMER_STALL_MS)
}

/// Return the heartbeat interval as a duration.
#[must_use]
pub(crate) fn heartbeat_interval() -> std::time::Duration {
    std::time::Duration::from_millis(HEARTBEAT_INTERVAL_MS)
}

/// Return the challenge retry interval as a duration.
#[must_use]
pub(crate) fn challenge_retry() -> std::time::Duration {
    std::time::Duration::from_millis(CHALLENGE_RETRY_MS)
}

/// Return the total close exchange budget as a duration.
#[must_use]
pub(crate) fn close_exchange() -> std::time::Duration {
    std::time::Duration::from_millis(CLOSE_EXCHANGE_MS)
}

/// Return the close retry interval as a duration.
#[must_use]
pub(crate) fn close_retry() -> std::time::Duration {
    std::time::Duration::from_millis(CLOSE_RETRY_MS)
}

/// Return the credit ack delay as a duration.
#[must_use]
pub(crate) fn ack_max_delay() -> std::time::Duration {
    std::time::Duration::from_millis(ACK_MAX_DELAY_MS)
}

/// Return the credit frame threshold.
#[must_use]
pub(crate) const fn ack_frame_threshold() -> u64 {
    ACK_FRAME_THRESHOLD
}

/// Build the activate control body for one session.
#[must_use]
pub(crate) fn activate_control(session_id: &str) -> LiveControl {
    LiveControl::Activate(LiveControlActivate {
        action: trellis_protocol::ActivateAction::Activate,
        received_seq: U64s::new(0),
        consumed_seq: U64s::new(0),
        format: trellis_protocol::LIVE_VERSION.to_owned(),
        kind: trellis_protocol::LiveControlKind::Control,
        session_id: session_id.to_owned(),
        control_seq: U64s::new(1),
    })
}

/// Build one pulse control body.
#[must_use]
pub(crate) fn pulse_control(
    session_id: &str,
    control_seq: u64,
    challenge_id: &str,
    received: u64,
    consumed: u64,
) -> LiveControl {
    LiveControl::Pulse(LiveControlPulse {
        action: trellis_protocol::PulseAction::Pulse,
        challenge_id: challenge_id.to_owned(),
        received_seq: U64s::new(received),
        consumed_seq: U64s::new(consumed),
        format: trellis_protocol::LIVE_VERSION.to_owned(),
        kind: trellis_protocol::LiveControlKind::Control,
        session_id: session_id.to_owned(),
        control_seq: U64s::new(control_seq),
    })
}

/// Build one credit control body.
#[must_use]
pub(crate) fn credit_control(
    session_id: &str,
    control_seq: u64,
    received: u64,
    consumed: u64,
) -> LiveControl {
    LiveControl::Ack(LiveControlCredit {
        action: trellis_protocol::AckAction::Ack,
        received_seq: U64s::new(received),
        consumed_seq: U64s::new(consumed),
        format: trellis_protocol::LIVE_VERSION.to_owned(),
        kind: trellis_protocol::LiveControlKind::Control,
        session_id: session_id.to_owned(),
        control_seq: U64s::new(control_seq),
    })
}

/// Build one close control body.
#[must_use]
pub(crate) fn close_control(
    session_id: &str,
    control_seq: u64,
    reason: trellis_protocol::ConsumerCloseReason,
    received: u64,
    consumed: u64,
) -> LiveControl {
    LiveControl::Close(LiveControlClose {
        action: trellis_protocol::CloseAction::Close,
        reason,
        received_seq: U64s::new(received),
        consumed_seq: U64s::new(consumed),
        format: trellis_protocol::LIVE_VERSION.to_owned(),
        kind: trellis_protocol::LiveControlKind::Control,
        session_id: session_id.to_owned(),
        control_seq: U64s::new(control_seq),
    })
}

/// Build one end-ack control body.
#[must_use]
pub(crate) fn end_ack_control(
    session_id: &str,
    control_seq: u64,
    final_seq: u64,
    received: u64,
    consumed: u64,
) -> LiveControl {
    LiveControl::EndAck(LiveControlEndAck {
        action: trellis_protocol::EndAckAction::EndAck,
        final_seq: U64s::new(final_seq),
        received_seq: U64s::new(received),
        consumed_seq: U64s::new(consumed),
        format: trellis_protocol::LIVE_VERSION.to_owned(),
        kind: trellis_protocol::LiveControlKind::Control,
        session_id: session_id.to_owned(),
        control_seq: U64s::new(control_seq),
    })
}

/// Validate one provider frame against session sequence state.
///
/// # Errors
///
/// Returns the matching wire code for a gap, replay, or invalid terminal.
pub(crate) fn validate_frame_sequence(
    frame: &LiveFrame,
    expected_next: u64,
    received: u64,
) -> Result<Option<u64>, LiveErrorCode> {
    match frame {
        LiveFrame::Data(data) => {
            let seq = data.seq.get();
            if seq > expected_next {
                return Err(LiveErrorCode::DeliveryGap);
            }
            if seq < expected_next {
                // Duplicate already-received sequence: discard without credit.
                return Ok(None);
            }
            Ok(Some(seq))
        }
        LiveFrame::Challenge(challenge) => {
            if challenge.last_sent_seq.get() > received {
                return Err(LiveErrorCode::DeliveryGap);
            }
            Ok(None)
        }
        LiveFrame::End(end) => {
            if end.final_seq.get() != received {
                return Err(LiveErrorCode::DeliveryGap);
            }
            Ok(None)
        }
    }
}

/// Parse one signed provider control response.
///
/// # Errors
///
/// Returns a protocol error for an unknown response shape; a `control-error`
/// surfaces its wire code.
pub(crate) fn parse_control_response(
    message: &async_nats::Message,
) -> Result<LiveControlAck, TrellisClientError> {
    let value: serde_json::Value = serde_json::from_slice(&message.payload)
        .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?;
    match value.get("type").and_then(|kind| kind.as_str()) {
        Some("control-ack") => serde_json::from_value(value)
            .map_err(|error| TrellisClientError::FeedProtocol(error.to_string())),
        Some("control-error") => {
            let code = value
                .get("code")
                .and_then(|code| code.as_str())
                .unwrap_or("protocol_error");
            Err(TrellisClientError::FeedProtocol(format!(
                "live control rejected with '{code}'"
            )))
        }
        _ => Err(TrellisClientError::FeedProtocol(
            "unexpected live control response".into(),
        )),
    }
}

static CONTROL_INBOX_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Return the serialized byte length of one encoded application value.
#[must_use]
pub(crate) fn encoded_len(value: &serde_json::Value) -> u64 {
    serde_json::to_vec(value).map_or(0, |bytes| bytes.len() as u64)
}
