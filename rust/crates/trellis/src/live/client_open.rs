//! Client-side live open: bounded opening request, signed-offer verification,
//! prepared handle installation, activation, and the data/control pump.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use futures_util::StreamExt;
use trellis_protocol::{
    derive_live_data_subject, LiveErrorCode, LiveFrame, LiveOffer, LiveOfferKind, LiveSessionKind,
    OPEN_RESERVATION_MS,
};

use crate::client::{AuthorizationProviderCache, TrellisClientError};

use super::authority::{LiveAuthorityGuard, LiveGuardRequirement, PinnedPeerIdentity};
use super::deadlines::{DeadlineAction, LiveDeadlines};
use super::subscription::{
    activate_control, consumer_failure, credit_control, end_ack_control, pulse_control,
    ConsumerControl, ConsumerCore, ConsumerPhase,
};
use super::types::{LiveCancellation, LiveEnd, LiveEndReason};

/// Inputs for one client-side live open.
pub(crate) struct ClientOpen<'a> {
    pub kind: LiveSessionKind,
    pub api_id: &'a str,
    /// Descriptor/binding-derived route the signed offer must echo.
    pub base_subject: &'a str,
    /// NATS subject the opening request is published on.
    ///
    /// Feeds publish on `base_subject`. Operation watch publishes on the
    /// operation control subject while the offer still binds `base_subject`.
    pub publish_subject: &'a str,
    pub body: Bytes,
    pub open_id: String,
    pub receive_max_payload_bytes: u64,
}

/// One verified prepared observation, before its first iteration.
pub(crate) struct PreparedClientSession {
    pub offer: LiveOffer,
    pub peer: PinnedPeerIdentity,
    pub max_data_body_bytes: u64,
    pub context_digest: String,
}

/// Outcome of one complete client open.
pub(crate) enum ClientOpenOutcome {
    Feed(PreparedClientSession),
    Operation(PreparedClientSession),
}

/// Perform one bounded opening exchange and verify the signed offer.
///
/// # Errors
///
/// Returns a setup error for an incompatible peer, invalid offer, mismatched
/// identity/limits, or a lost opening reply. No prepared handle is returned.
pub(crate) async fn open_client_session(
    client: &crate::client::TrellisClient,
    provider: &AuthorizationProviderCache,
    open: ClientOpen<'_>,
) -> Result<PreparedClientSession, TrellisClientError> {
    let subject = open.publish_subject.to_owned();
    let reply = format!(
        "{}.{}",
        client.inbox_prefix(),
        LIVE_INBOX_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let headers = client.signed_headers(&subject, &reply, &open.body)?;
    let request_id = headers
        .get("request-id")
        .map(ToString::to_string)
        .unwrap_or_default();
    let mut subscriber = tokio::time::timeout(
        Duration::from_millis(client.timeout_ms()),
        client.nats().subscribe(reply.clone()),
    )
    .await
    .map_err(|_| TrellisClientError::Timeout)?
    .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;
    tokio::time::timeout(
        Duration::from_millis(client.timeout_ms()),
        client
            .nats()
            .publish_with_reply_and_headers(subject, reply, headers, open.body.clone()),
    )
    .await
    .map_err(|_| TrellisClientError::Timeout)?
    .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

    let response = tokio::time::timeout(
        Duration::from_millis(client.timeout_ms()),
        subscriber.next(),
    )
    .await
    .map_err(|_| TrellisClientError::Timeout)?
    .ok_or(TrellisClientError::Timeout)?;

    verify_offer(client, provider, &open, &request_id, &response).await
}

async fn verify_offer(
    client: &crate::client::TrellisClient,
    provider: &AuthorizationProviderCache,
    open: &ClientOpen<'_>,
    request_id: &str,
    response: &async_nats::Message,
) -> Result<PreparedClientSession, TrellisClientError> {
    trellis_protocol::validate_control_body(&response.payload)
        .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?;
    let value: serde_json::Value = serde_json::from_slice(&response.payload)
        .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?;
    let kind = value.get("type").and_then(|kind| kind.as_str());
    if kind != Some("offer") {
        // A signed open-error or a legacy finite envelope is surfaced as a
        // setup failure; it is never interpreted as a session offer.
        let code = value
            .get("code")
            .and_then(|code| code.as_str())
            .or_else(|| value.get("type").and_then(|kind| kind.as_str()))
            .unwrap_or("invalid_request");
        return Err(TrellisClientError::FeedProtocol(format!(
            "live open rejected with '{code}': {value}"
        )));
    }
    let offer: LiveOffer = serde_json::from_value(value)
        .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?;
    if offer.kind != LiveOfferKind::Offer {
        return Err(TrellisClientError::FeedProtocol(
            "live open response is not an offer".into(),
        ));
    }
    if offer.open_id != open.open_id {
        return Err(TrellisClientError::FeedProtocol(
            "offer answers a different logical open".into(),
        ));
    }
    // The offer must carry the provider's current context and a proof that
    // binds this exact response subject and raw bytes.
    let context_digest = response
        .headers
        .as_ref()
        .and_then(|headers| headers.get("authorization-context"))
        .map(ToString::to_string)
        .ok_or_else(|| {
            TrellisClientError::FeedProtocol("offer omitted its authorization context".into())
        })?;
    let session_key = response
        .headers
        .as_ref()
        .and_then(|headers| headers.get("session-key"))
        .map(ToString::to_string)
        .ok_or_else(|| TrellisClientError::FeedProtocol("offer omitted its signer".into()))?;
    let proof = response
        .headers
        .as_ref()
        .and_then(|headers| headers.get("trellis-live-proof"))
        .map(ToString::to_string)
        .ok_or_else(|| TrellisClientError::FeedProtocol("offer omitted its proof".into()))?;
    trellis_protocol::verify_live_server_proof_encoded(
        &trellis_protocol::LiveServerProof::parse(proof)
            .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?,
        &context_digest,
        response.subject.as_str(),
        &response.payload,
        &session_key,
    )
    .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?;
    let policy = provider
        .policy()
        .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?;
    let lease = provider
        .resolve_context(&context_digest, policy.now_unix_seconds)
        .await
        .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?;
    let peer = PinnedPeerIdentity::from_signed(lease.signed_context());
    if peer.session_key != session_key {
        return Err(TrellisClientError::FeedProtocol(
            "offer context does not bind the signing session key".into(),
        ));
    }
    if trellis_protocol::encode_subject_token(&session_key) != offer.provider.session_key {
        return Err(TrellisClientError::FeedProtocol(
            "offer signer does not match its advertised identity".into(),
        ));
    }
    if offer.request_id != request_id {
        return Err(TrellisClientError::FeedProtocol(
            "offer answers a different opening request".into(),
        ));
    }
    // The exact subjects must recompute from this session's identities.
    let expected_data = derive_live_data_subject(
        &offer.provider.connection_id,
        &offer.consumer.connection_id,
        &offer.session_id,
    )
    .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?;
    if expected_data != offer.data_subject {
        return Err(TrellisClientError::FeedProtocol(
            "offer data subject is not canonical for this session".into(),
        ));
    }
    let expected_control = trellis_protocol::derive_live_observe_subject(
        &offer.base_subject,
        &offer.provider.connection_id,
        &offer.session_id,
    )
    .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?;
    if expected_control != offer.control_subject {
        return Err(TrellisClientError::FeedProtocol(
            "offer control subject is not canonical for this session".into(),
        ));
    }
    verify_offer_identity(open, &offer)?;
    // The selected deployment must match the consumer's installed binding.
    let selected = provider
        .provider_deployment_id(open.api_id)
        .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?;
    if selected != offer.provider.deployment_id {
        return Err(TrellisClientError::FeedProtocol(
            "offer provider is not the selected deployment for this API".into(),
        ));
    }
    let negotiated = trellis_protocol::negotiate_max_data_body_bytes(
        open.receive_max_payload_bytes,
        client.nats().max_payload() as u64,
    )
    .map_err(|error| TrellisClientError::FeedProtocol(error.to_string()))?;
    if offer.limits.max_data_body_bytes > negotiated {
        return Err(TrellisClientError::FeedProtocol(
            "offer exceeds the negotiated data body limit".into(),
        ));
    }
    if offer.kind != LiveOfferKind::Offer {
        return Err(TrellisClientError::FeedProtocol(
            "unsupported live offer kind".into(),
        ));
    }
    let _ = client;
    Ok(PreparedClientSession {
        max_data_body_bytes: offer.limits.max_data_body_bytes,
        offer,
        peer,
        context_digest,
    })
}

/// Install one prepared Feed observation as an owned public handle.
///
/// The handle owns the caller's prepared state and the exact data
/// subscription; the first poll starts activation and the pump.
///
/// # Errors
///
/// Returns a setup error when the data subscription cannot be flushed or the
/// consumer admission bound is reached.
pub(crate) async fn install_feed_handle<D>(
    client: &crate::client::TrellisClient,
    prepared: PreparedClientSession,
) -> Result<crate::live::subscription::LiveSubscription<D::Event>, TrellisClientError>
where
    D: crate::generated::FeedDescriptor,
    D::Event: crate::generated::Codec + Send + 'static,
{
    install_prepared_handle(client, prepared, |value| {
        <D::Event as crate::generated::Codec>::decode(value)
            .map(Some)
            .map_err(|error| TrellisClientError::Codec(error.to_string()))
    })
    .await
}

/// Install one prepared Operation watch observation as an owned public handle.
///
/// Data frames are JSON snapshot/event envelopes decoded by `decode`. A `None`
/// result skips the frame (keepalive) after releasing credit.
///
/// # Errors
///
/// Returns a setup error when the data subscription cannot be flushed or the
/// consumer admission bound is reached.
pub(crate) async fn install_operation_watch_handle<T, F>(
    client: &crate::client::TrellisClient,
    prepared: PreparedClientSession,
    decode: F,
) -> Result<crate::live::subscription::LiveSubscription<T>, TrellisClientError>
where
    T: Send + 'static,
    F: Fn(serde_json::Value) -> Result<Option<T>, TrellisClientError> + Send + 'static,
{
    install_prepared_handle(client, prepared, decode).await
}

async fn install_prepared_handle<T, F>(
    client: &crate::client::TrellisClient,
    prepared: PreparedClientSession,
    decode: F,
) -> Result<crate::live::subscription::LiveSubscription<T>, TrellisClientError>
where
    T: Send + 'static,
    F: Fn(serde_json::Value) -> Result<Option<T>, TrellisClientError> + Send + 'static,
{
    let manager = client.live_manager().ok_or_else(|| {
        TrellisClientError::Bootstrap("live manager is unavailable for this connection".into())
    })?;
    manager.is_available().map_err(|unavailable| {
        TrellisClientError::AuthorizationUnavailable(format!(
            "live manager unavailable: {unavailable:?}"
        ))
    })?;
    let permit = manager.clone().admit_consumer().map_err(|code| {
        TrellisClientError::FeedProtocol(format!("admission rejected: {code:?}"))
    })?;
    let core = Arc::new(ConsumerCore::new(prepared.offer.session_id.clone()));
    core.set_phase(ConsumerPhase::Prepared);
    let control = Arc::new(ConsumerControl {
        nats: client.nats(),
        auth: client.auth_handle(),
        contexts: client.authorization_contexts_handle()?,
        inbox_prefix: client.inbox_prefix().to_owned(),
        session_id: prepared.offer.session_id.clone(),
        control_subject: prepared.offer.control_subject.clone(),
        pinned_session_key: prepared.peer.session_key.clone(),
        pinned_identity: prepared.peer.clone(),
        close_started: std::sync::atomic::AtomicBool::new(false),
        last_control_seq: std::sync::atomic::AtomicU64::new(0),
    });
    let provider_guard = LiveAuthorityGuard::retain(
        client.authorization_provider(),
        &prepared.context_digest,
        LiveGuardRequirement::PeerProvider {
            expected: prepared.peer.clone(),
        },
    )
    .await
    .map_err(|lost| {
        TrellisClientError::AuthorizationUnavailable(format!("provider guard: {lost:?}"))
    })?;
    let cancellation = LiveCancellation::new();
    let pump = ConsumerPump::new(
        core.clone(),
        control.clone(),
        cancellation.clone(),
        prepared.max_data_body_bytes,
    );
    let drain = pump.spawn(client.nats(), prepared.offer.data_subject.clone(), decode);
    Ok(crate::live::subscription::LiveSubscription::new(
        core,
        drain,
        control,
        cancellation,
        permit,
        provider_guard,
    ))
}

/// Reject an offer whose route or session kind does not match the open.
fn verify_offer_identity(
    open: &ClientOpen<'_>,
    offer: &LiveOffer,
) -> Result<(), TrellisClientError> {
    if offer.base_subject != open.base_subject {
        return Err(TrellisClientError::FeedProtocol(
            "offer base subject does not match the opening route".into(),
        ));
    }
    if offer.session_kind != open.kind {
        return Err(TrellisClientError::FeedProtocol(
            "offer session kind does not match the opening kind".into(),
        ));
    }
    Ok(())
}

/// Build one `open-error`-shaped setup failure from a raw body.
#[must_use]
pub(crate) fn open_error_from_body(body: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    value
        .get("code")
        .and_then(|code| code.as_str())
        .map(str::to_owned)
}

/// Return the opening reservation budget.
#[must_use]
pub(crate) fn reservation_budget() -> Duration {
    Duration::from_millis(OPEN_RESERVATION_MS)
}

/// Sleep until an optional deadline; wait forever when none is scheduled.
async fn wait_until(deadline: Option<tokio::time::Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}

/// One live data pump over a verified prepared session.
pub(crate) struct ConsumerPump<T> {
    core: Arc<ConsumerCore<T>>,
    control: Arc<ConsumerControl>,
    cancellation: LiveCancellation,
    max_data_body_bytes: u64,
}

impl<T> ConsumerPump<T> {
    /// Create one pump for a prepared session.
    #[must_use]
    pub(crate) fn new(
        core: Arc<ConsumerCore<T>>,
        control: Arc<ConsumerControl>,
        cancellation: LiveCancellation,
        max_data_body_bytes: u64,
    ) -> Self {
        Self {
            core,
            control,
            cancellation,
            max_data_body_bytes,
        }
    }

    /// Run activation then the data/control loop until a terminal outcome.
    ///
    /// The consumer becomes ACTIVE only after a matching pulse acknowledgement
    /// verifies; ordinary data or a stale challenge never renews liveness. The
    /// monotonic deadline owner supplies peer, credit and draining deadlines.
    ///
    /// # Errors
    ///
    /// Never returns an error: every failure commits one terminal outcome on
    /// the consumer core so the application sees exactly one diagnosis.
    pub(crate) fn spawn<F>(
        self,
        nats: async_nats::Client,
        data_subject: String,
        decode: F,
    ) -> tokio::task::JoinHandle<()>
    where
        T: Send + 'static,
        F: Fn(serde_json::Value) -> Result<Option<T>, TrellisClientError> + Send + 'static,
    {
        tokio::spawn(async move {
            tokio::select! {
                _ = self.core.start.notified() => {}
                _ = self.cancellation.cancelled() => return,
            }
            let session_id = self.core.session_id.clone();
            let mut deadlines = LiveDeadlines::prepared(tokio::time::Instant::now());
            let mut subscription = match nats.subscribe(data_subject).await {
                Ok(subscription) => subscription,
                Err(_) => {
                    self.core.commit_end(consumer_failure(
                        LiveErrorCode::Disconnected,
                        "live data subscription could not be installed",
                    ));
                    return;
                }
            };
            if nats.flush().await.is_err() {
                self.core.commit_end(consumer_failure(
                    LiveErrorCode::Disconnected,
                    "live data subscription could not be flushed",
                ));
                return;
            }
            self.core.set_phase(ConsumerPhase::Activating);

            // Activation: bounded fresh-proof retries within the reservation.
            let reservation_deadline = tokio::time::Instant::now() + reservation_budget();
            let activate = activate_control(&session_id);
            let activate_seq = self.control.next_control_seq();
            loop {
                let attempt = tokio::select! {
                    _ = self.cancellation.cancelled() => {
                        self.core.discard_queue();
                        self.core.commit_end(LiveEnd::new(LiveEndReason::Cancelled, None));
                        return;
                    }
                    result = self.control.send_control(&activate, activate_seq) => result,
                };
                match attempt {
                    Ok(ack) if ack.state != trellis_protocol::LiveSessionState::Closed => break,
                    Ok(_) => {
                        self.core.commit_end(consumer_failure(
                            LiveErrorCode::SetupTimeout,
                            "live session closed during activation",
                        ));
                        return;
                    }
                    Err(_) if tokio::time::Instant::now() < reservation_deadline => continue,
                    Err(_) => {
                        self.core.commit_end(consumer_failure(
                            LiveErrorCode::SetupTimeout,
                            "live activation did not complete within the reservation",
                        ));
                        return;
                    }
                }
            }

            let mut next_expected: u64 = 1;
            let mut last_credit_sent: u64 = 0;
            loop {
                if self.core.committed_end().is_some() {
                    return;
                }
                if matches!(self.core.phase(), ConsumerPhase::Draining) {
                    // The data subscription and peer pulse work are stopped;
                    // only the bounded local drain and its stall deadline remain.
                    let next = deadlines.next_due();
                    tokio::select! {
                        _ = self.cancellation.cancelled() => {
                            self.core.discard_queue();
                            self.core.commit_end(LiveEnd::new(LiveEndReason::Cancelled, None));
                            return;
                        }
                        _ = wait_until(next) => {
                            if deadlines.evaluate(tokio::time::Instant::now())
                                == Some(DeadlineAction::ConsumerStalled)
                            {
                                self.core.discard_queue();
                                self.core.commit_end(consumer_failure(
                                    LiveErrorCode::ConsumerSlow,
                                    "live draining queue was not consumed",
                                ));
                            }
                        }
                        _ = self.core.end_notify.notified() => {}
                    }
                    return;
                }
                let next = deadlines.next_due();
                tokio::select! {
                    _ = self.cancellation.cancelled() => {
                        self.core.discard_queue();
                        self.core.commit_end(LiveEnd::new(LiveEndReason::Cancelled, None));
                        return;
                    }
                    _ = self.core.credit_notify.notified() => {
                        let consumed = self.core.consumed_seq();
                        if consumed > last_credit_sent {
                            deadlines.note_consumption(
                                tokio::time::Instant::now(),
                                consumed - last_credit_sent,
                            );
                        }
                    }
                    _ = wait_until(next) => {
                        let now = tokio::time::Instant::now();
                        match deadlines.evaluate(now) {
                            Some(DeadlineAction::ReservationExpired) => {
                                self.core.commit_end(consumer_failure(
                                    LiveErrorCode::SetupTimeout,
                                    "live activation did not complete within the reservation",
                                ));
                                return;
                            }
                            Some(DeadlineAction::PeerInactive) => {
                                self.core.commit_end(consumer_failure(
                                    LiveErrorCode::PeerLost,
                                    "live provider silent past the inactivity bound",
                                ));
                                return;
                            }
                            Some(DeadlineAction::CreditDue) => {
                                let received = self.core.received_seq();
                                let consumed = self.core.consumed_seq();
                                deadlines.credit_sent();
                                if consumed > last_credit_sent {
                                    last_credit_sent = consumed;
                                    let control_seq = self.control.next_control_seq();
                                    let _ = self
                                        .control
                                        .send_control(
                                            &credit_control(
                                                &session_id,
                                                control_seq,
                                                received,
                                                consumed,
                                            ),
                                            control_seq,
                                        )
                                        .await;
                                }
                            }
                            _ => {}
                        }
                    }
                    message = subscription.next() => {
                        let Some(message) = message else {
                            self.core.commit_end(consumer_failure(
                                LiveErrorCode::PeerLost,
                                "live delivery subscription ended",
                            ));
                            return;
                        };
                        // Frames must authenticate against the pinned provider
                        // before they influence state; garbage is discarded.
                        let Ok(frame) = trellis_protocol::parse_live_frame(
                            &message.payload,
                            self.max_data_body_bytes,
                        ) else {
                            continue;
                        };
                        if !verify_provider_frame(&self.control, &message, &frame) {
                            continue;
                        }
                        match &frame {
                            LiveFrame::Data(data) => {
                                let seq = data.seq.get();
                                if seq > next_expected {
                                    self.core.commit_end(consumer_failure(
                                        LiveErrorCode::DeliveryGap,
                                        "live delivery sequence gap",
                                    ));
                                    return;
                                }
                                if seq < next_expected {
                                    // Already received; discard without credit.
                                    continue;
                                }
                                next_expected = seq + 1;
                                match decode(data.value.clone()) {
                                    Ok(Some(value)) => {
                                        let encoded_len =
                                            serde_json::to_vec(&data.value).map_or(0, |bytes| {
                                                bytes.len() as u64
                                            });
                                        if !self.core.admit(super::subscription::AdmittedItem {
                                            value,
                                            encoded_len,
                                        }) {
                                            self.core.commit_end(consumer_failure(
                                                LiveErrorCode::ConsumerSlow,
                                                "bounded live ingress exceeded",
                                            ));
                                            return;
                                        }
                                        self.core.record_received();
                                    }
                                    Ok(None) => {
                                        // Keepalive and other filtered frames still
                                        // occupy sequence space and release credit.
                                        self.core.record_received();
                                        self.core.release_filtered();
                                    }
                                    Err(error) => {
                                        self.core.commit_end(consumer_failure(
                                            LiveErrorCode::ProtocolError,
                                            error.to_string(),
                                        ));
                                        return;
                                    }
                                }
                            }
                            LiveFrame::Challenge(challenge) => {
                                if challenge.last_sent_seq.get() > self.core.received_seq() {
                                    self.core.commit_end(consumer_failure(
                                        LiveErrorCode::DeliveryGap,
                                        "live challenge referred to unreceived data",
                                    ));
                                    return;
                                }
                                let control_seq = self.control.next_control_seq();
                                let pulse = pulse_control(
                                    &session_id,
                                    control_seq,
                                    &challenge.challenge_id,
                                    self.core.received_seq(),
                                    self.core.consumed_seq(),
                                );
                                let ack = tokio::select! {
                                    _ = self.cancellation.cancelled() => {
                                        self.core.discard_queue();
                                        self.core.commit_end(LiveEnd::new(LiveEndReason::Cancelled, None));
                                        return;
                                    }
                                    result = self.control.send_control(&pulse, control_seq) => result,
                                };
                                // Only a matching, authenticated pulse ack commits
                                // activation or renews the fresh-round-trip clock.
                                if let Ok(ack) = ack {
                                    if ack.state == trellis_protocol::LiveSessionState::Active {
                                        let now = tokio::time::Instant::now();
                                        if matches!(self.core.phase(), ConsumerPhase::Activating) {
                                            deadlines.commit_active(now, false);
                                            self.core.set_phase(ConsumerPhase::Active);
                                        } else {
                                            deadlines.fresh_round_trip(now, false);
                                        }
                                    }
                                }
                            }
                            LiveFrame::End(end) => {
                                if end.final_seq.get() != self.core.received_seq() {
                                    self.core.commit_end(consumer_failure(
                                        LiveErrorCode::DeliveryGap,
                                        "live end did not follow the complete sequence",
                                    ));
                                    return;
                                }
                                // One end-ack confirms transport receipt, not
                                // application processing; queued items drain
                                // locally afterwards.
                                let control_seq = self.control.next_control_seq();
                                let _ = self
                                    .control
                                    .send_control(
                                        &end_ack_control(
                                            &session_id,
                                            control_seq,
                                            end.final_seq.get(),
                                            self.core.received_seq(),
                                            self.core.consumed_seq(),
                                        ),
                                        control_seq,
                                    )
                                    .await;
                                let end = match &end.terminal.error {
                                    Some(error) => LiveEnd::new(
                                        end.terminal.reason,
                                        Some(std::sync::Arc::new(super::types::LiveStreamError::new(
                                            error.code,
                                            error.message.clone(),
                                        ))),
                                    ),
                                    None => LiveEnd::new(end.terminal.reason, None),
                                };
                                if end.is_complete() {
                                    self.core.set_pending_end(end);
                                    self.core.set_phase(ConsumerPhase::Draining);
                                    deadlines.begin_draining();
                                    deadlines.data_admitted(tokio::time::Instant::now());
                                } else {
                                    self.core.commit_end(end);
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        })
    }
}

/// Verify one provider-origin frame against the pinned session identity.
///
/// A frame that fails authentication is discarded without influencing state,
/// so injected garbage cannot close a valid session.
fn verify_provider_frame(
    control: &ConsumerControl,
    message: &async_nats::Message,
    frame: &LiveFrame,
) -> bool {
    let headers = message.headers.as_ref();
    let context_digest = headers
        .and_then(|headers| headers.get("authorization-context"))
        .map(ToString::to_string);
    let session_key = headers
        .and_then(|headers| headers.get("session-key"))
        .map(ToString::to_string);
    let proof = headers
        .and_then(|headers| headers.get("trellis-live-proof"))
        .map(ToString::to_string);
    let (Some(context_digest), Some(session_key), Some(proof)) =
        (context_digest, session_key, proof)
    else {
        return false;
    };
    if session_key != control.pinned_session_key
        || session_key != control.pinned_identity.session_key
    {
        return false;
    }
    // The session id binds every frame to this observation.
    if frame.session_id() != control.session_id {
        return false;
    }
    let Ok(proof) = trellis_protocol::LiveServerProof::parse(proof) else {
        return false;
    };
    // The exact provider context is resolved and verified asynchronously by
    // the pump before it inspects frame state; this synchronous pre-check only
    // rejects frames that cannot be authentic at all.
    trellis_protocol::verify_live_server_proof_encoded(
        &proof,
        &context_digest,
        &message.subject,
        &message.payload,
        &session_key,
    )
    .is_ok()
}

static LIVE_INBOX_COUNTER: AtomicU64 = AtomicU64::new(1);

#[cfg(test)]
mod tests {
    use super::{verify_offer_identity, ClientOpen};
    use bytes::Bytes;
    use trellis_protocol::{
        LiveOffer, LiveOfferConsumer, LiveOfferKind, LiveOfferLimits, LiveOfferProvider,
        LiveSessionKind, HEARTBEAT_INTERVAL_MS, OPEN_RESERVATION_MS, PEER_INACTIVITY_MS,
        WINDOW_BYTES, WINDOW_FRAMES,
    };

    fn offer(kind: LiveSessionKind, base_subject: &str) -> LiveOffer {
        LiveOffer {
            format: trellis_protocol::LIVE_VERSION.into(),
            kind: LiveOfferKind::Offer,
            session_kind: kind,
            open_id: "open".into(),
            request_id: "req".into(),
            session_id: "session".into(),
            base_subject: base_subject.into(),
            data_subject: "live.v1.data.A.B.C".into(),
            control_subject: format!("{base_subject}.observe.A.C"),
            provider: LiveOfferProvider {
                connection_id: "P".into(),
                session_key: "K".into(),
                principal_id: "pr".into(),
                participant_id: "pa".into(),
                deployment_id: "dep".into(),
                instance_id: "inst".into(),
            },
            consumer: LiveOfferConsumer {
                connection_id: "C".into(),
                session_key: "KC".into(),
                principal_id: "pru".into(),
                participant_id: "pau".into(),
            },
            limits: LiveOfferLimits {
                max_data_body_bytes: 1_044_480,
                window_frames: WINDOW_FRAMES,
                window_bytes: WINDOW_BYTES,
                reservation_ms: OPEN_RESERVATION_MS,
                heartbeat_interval_ms: HEARTBEAT_INTERVAL_MS,
                peer_inactivity_ms: PEER_INACTIVITY_MS,
                consumer_stall_ms: trellis_protocol::CONSUMER_STALL_MS,
            },
        }
    }

    fn open<'a>(
        kind: LiveSessionKind,
        base_subject: &'a str,
        publish_subject: &'a str,
    ) -> ClientOpen<'a> {
        ClientOpen {
            kind,
            api_id: "api@v1",
            base_subject,
            publish_subject,
            body: Bytes::new(),
            open_id: "open".into(),
            receive_max_payload_bytes: 1024,
        }
    }

    #[test]
    fn feed_open_publishes_on_the_same_base_subject() {
        let base = "feed.v1.Watch";
        let open = open(LiveSessionKind::Feed, base, base);
        verify_offer_identity(&open, &offer(LiveSessionKind::Feed, base)).expect("feed offer");
    }

    #[test]
    fn operation_watch_offer_binds_the_operation_route_not_control() {
        let base = "operation.v1.Billing.Refund";
        let publish = "operation.v1.Billing.Refund.control";
        let open = open(LiveSessionKind::OperationWatch, base, publish);
        verify_offer_identity(&open, &offer(LiveSessionKind::OperationWatch, base))
            .expect("operation watch offer");
    }

    #[test]
    fn operation_watch_rejects_control_subject_as_offer_base() {
        let base = "operation.v1.Billing.Refund";
        let publish = "operation.v1.Billing.Refund.control";
        let open = open(LiveSessionKind::OperationWatch, base, publish);
        let error = verify_offer_identity(&open, &offer(LiveSessionKind::OperationWatch, publish))
            .expect_err("control is not the offer base");
        assert!(error.to_string().contains("base subject"));
    }

    #[test]
    fn operation_watch_rejects_feed_session_kind() {
        let base = "operation.v1.Billing.Refund";
        let open = open(
            LiveSessionKind::OperationWatch,
            base,
            "operation.v1.Billing.Refund.control",
        );
        let error = verify_offer_identity(&open, &offer(LiveSessionKind::Feed, base))
            .expect_err("kind must match");
        assert!(error.to_string().contains("session kind"));
    }
}
