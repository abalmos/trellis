//! Client-side live open: bounded opening request, signed-offer verification,
//! prepared handle installation, activation, and the data/control pump.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use futures_util::StreamExt;
use trellis_protocol::{
    derive_live_data_subject, LiveErrorCode, LiveFrame, LiveOffer, LiveOfferKind, LiveSessionKind,
    ACK_MAX_DELAY_MS, OPEN_RESERVATION_MS,
};

use crate::client::{AuthorizationProviderCache, TrellisClientError};

use super::authority::{LiveAuthorityGuard, LiveGuardRequirement, PinnedPeerIdentity};
use super::subscription::{
    activate_control, consumer_failure, credit_control, end_ack_control, peer_inactivity,
    pulse_control, ConsumerControl, ConsumerCore, ConsumerPhase,
};
use super::types::{LiveCancellation, LiveEnd, LiveEndReason};

/// Inputs for one client-side live open.
pub(crate) struct ClientOpen<'a> {
    pub kind: LiveSessionKind,
    pub api_id: &'a str,
    pub base_subject: &'a str,
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
    let subject = open.base_subject.to_owned();
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
    if offer.base_subject != open.base_subject {
        return Err(TrellisClientError::FeedProtocol(
            "offer base subject does not match the opening route".into(),
        ));
    }
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
    let _ = open.kind;
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
    let pump = ConsumerPump::new(core.clone(), control.clone(), cancellation.clone());
    let drain = pump.spawn(
        client.nats(),
        prepared.offer.data_subject.clone(),
        |value| {
            <D::Event as crate::generated::Codec>::decode(value)
                .map_err(|error| TrellisClientError::Codec(error.to_string()))
        },
    );
    Ok(crate::live::subscription::LiveSubscription::new(
        core,
        drain,
        control,
        cancellation,
        permit,
        provider_guard,
    ))
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

/// One live data pump over a verified prepared session.
pub(crate) struct ConsumerPump<T> {
    core: Arc<ConsumerCore<T>>,
    control: Arc<ConsumerControl>,
    cancellation: LiveCancellation,
}

impl<T> ConsumerPump<T> {
    /// Create one pump for a prepared session.
    #[must_use]
    pub(crate) fn new(
        core: Arc<ConsumerCore<T>>,
        control: Arc<ConsumerControl>,
        cancellation: LiveCancellation,
    ) -> Self {
        Self {
            core,
            control,
            cancellation,
        }
    }

    /// Run activation then the data/control loop until a terminal outcome.
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
        F: Fn(serde_json::Value) -> Result<T, TrellisClientError> + Send + 'static,
    {
        tokio::spawn(async move {
            tokio::select! {
                _ = self.core.start.notified() => {}
                _ = self.cancellation.cancelled() => return,
            }
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
            let session_id = self.core.session_id.clone();
            let control_seq = self.control.next_control_seq();
            let Ok(ack) = self
                .control
                .send_control(&activate_control(&session_id), control_seq)
                .await
            else {
                self.core.commit_end(consumer_failure(
                    LiveErrorCode::Disconnected,
                    "activation control could not be sent",
                ));
                return;
            };
            if ack.state == trellis_protocol::LiveSessionState::Closed {
                self.core.commit_end(consumer_failure(
                    LiveErrorCode::SetupTimeout,
                    "live session closed during activation",
                ));
                return;
            }
            self.core.set_phase(ConsumerPhase::Active);
            let mut next_expected: u64 = 1;
            let mut last_peer_activity = tokio::time::Instant::now();
            let activation_deadline = tokio::time::Instant::now() + reservation_budget();
            let mut credit_tick =
                tokio::time::interval(std::time::Duration::from_millis(ACK_MAX_DELAY_MS));
            credit_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let mut last_credit_sent: u64 = 0;
            let provider_guard: Option<super::authority::LiveAuthorityGuard> = None;
            let _ = provider_guard;
            loop {
                tokio::select! {
                    _ = self.cancellation.cancelled() => {
                        self.core.discard_queue();
                        self.core.commit_end(LiveEnd::new(LiveEndReason::Cancelled, None));
                        return;
                    }
                    _ = tokio::time::sleep_until(activation_deadline), if self.core.phase() == ConsumerPhase::Activating => {
                        self.core.commit_end(consumer_failure(
                            LiveErrorCode::SetupTimeout,
                            "live activation did not complete within the reservation",
                        ));
                        return;
                    }
                    _ = credit_tick.tick() => {
                        // Send accumulated consumption credit; a fully consumed
                        // idle stream still acks its position so the provider
                        // can release window state.
                        let received = self.core.received_seq();
                        let consumed = self.core.consumed_seq();
                        if consumed > last_credit_sent {
                            last_credit_sent = consumed;
                            let control_seq = self.control.next_control_seq();
                            let _ = self
                                .control
                                .send_control(
                                    &credit_control(&session_id, control_seq, received, consumed),
                                    control_seq,
                                )
                                .await;
                        }
                        // Provider inactivity: a silent pinned peer is lost.
                        if last_peer_activity.elapsed() >= peer_inactivity() {
                            self.core.commit_end(consumer_failure(
                                LiveErrorCode::PeerLost,
                                "live provider silent past the inactivity bound",
                            ));
                            return;
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
                        let Ok(frame) = trellis_protocol::parse_live_frame(&message.payload) else {
                            continue;
                        };
                        if !verify_provider_frame(&self.control, &message, &frame) {
                            continue;
                        }
                        last_peer_activity = tokio::time::Instant::now();
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
                                    Ok(value) => {
                                        let encoded_len =
                                            serde_json::to_vec(&data.value).map_or(0, |bytes| bytes.len() as u64);
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
                                    Err(_) => {
                                        self.core.commit_end(consumer_failure(
                                            LiveErrorCode::ProtocolError,
                                            "live payload did not decode",
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
                                let _ = self
                                    .control
                                    .send_control(
                                        &pulse_control(
                                            &session_id,
                                            control_seq,
                                            &challenge.challenge_id,
                                            self.core.received_seq(),
                                            self.core.consumed_seq(),
                                        ),
                                        control_seq,
                                    )
                                    .await;
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
                                } else {
                                    self.core.commit_end(end);
                                }
                                return;
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
