//! Router integration for live Feed routes.
//!
//! A live-capable Feed route answers one bounded opening request with a signed
//! offer and hands ownership of the reservation to the connection's live
//! session manager. It never enters an infinite reply loop and never starts a
//! domain source before the delivery-path challenge round trip completes.
//!
//! # Staged surface
//!
//! This module is landed ahead of the router registration that consumes it (the
//! Feed migration work package). Until `Router::register_feed` dispatches live
//! open envelopes here, unused items carry this single documented reason rather
//! than scattered per-item allowances; remove it when the migration lands.
#![allow(dead_code, reason = "live Feed route staged for the Feed migration")]

use bytes::Bytes;
use futures_util::StreamExt;

use trellis_protocol::{
    derive_live_data_subject, derive_live_observe_wildcard_subject, LiveErrorCode, LiveOfferLimits,
    LiveSessionKind, OPEN_RESERVATION_MS,
};

use crate::client::TrellisClient;
use crate::live::authority::{LiveAuthorityGuard, LiveAuthorityLost, PinnedPeerIdentity};
use crate::live::manager::LiveSessionManager;
use crate::live::provider::cleanup_grace;
use crate::live::provider_engine::{
    ack_for, drive_source, error_for, publish_challenge, take_source, ProviderOpenRequest,
    ProviderSessionRecord, SourceItem,
};
use crate::live::types::LiveEnd;
use crate::service::error::{ServerError, ValidationIssue};
use crate::service::request_loop::LivePreparedResponse;
use crate::service::router::RequestContext;

/// One connection owner that can serve live reservations for its routes.
///
/// The router holds a strong client reference so the live provider path shares
/// the exact authenticated connection, provider cache, and signing identity of
/// the service that registered the route.
#[derive(Clone)]
pub struct LiveProviderOwner {
    client: std::sync::Arc<TrellisClient>,
}

impl LiveProviderOwner {
    /// Construct one live provider owner from a connected client.
    #[must_use]
    pub(crate) fn new(client: std::sync::Arc<TrellisClient>) -> Self {
        Self { client }
    }

    /// Return the owning client.
    #[must_use]
    pub(crate) fn client(&self) -> &std::sync::Arc<TrellisClient> {
        &self.client
    }

    /// Return the connection's live manager.
    ///
    /// # Errors
    ///
    /// Returns an error when the connection has no live manager.
    pub(crate) fn manager(&self) -> Result<&std::sync::Arc<LiveSessionManager>, ServerError> {
        self.client.live_manager().ok_or_else(|| {
            ServerError::Nats("live session manager is unavailable for this connection".to_owned())
        })
    }
}

/// Inputs for one provider Feed opening.
pub(crate) struct FeedOpenInputs {
    pub api_id: String,
    pub base_subject: String,
    pub provider_instance_id: String,
    pub provider_deployment_id: String,
}

/// The consumer's verified opening request for one provider session.
pub(crate) struct FeedOpenRequest {
    pub request: RequestContext,
    pub inputs: FeedOpenInputs,
    pub opening: FeedOpeningMeta,
    pub encoded_input: serde_json::Value,
    pub cancellation: crate::live::LiveCancellation,
}

/// Parse and validate one Feed opening body.
///
/// # Errors
///
/// Returns a validation error for a malformed envelope, an unsupported
/// protocol, or an invalid native input codec value.
pub(crate) fn parse_feed_open<TInput>(payload: &[u8]) -> Result<FeedOpening<TInput>, ServerError>
where
    TInput: crate::generated::Codec,
{
    trellis_protocol::validate_open_body(payload).map_err(|error| ServerError::Validation {
        issues: Box::new(vec![ValidationIssue {
            path: String::new(),
            message: error.to_string(),
        }]),
    })?;
    let value: serde_json::Value =
        serde_json::from_slice(payload).map_err(|error| ServerError::Validation {
            issues: Box::new(vec![ValidationIssue {
                path: String::new(),
                message: format!("Invalid JSON: {error}"),
            }]),
        })?;
    if value.get("format").and_then(|format| format.as_str())
        != Some(trellis_protocol::LIVE_VERSION)
    {
        return Err(ServerError::Validation {
            issues: Box::new(vec![ValidationIssue {
                path: "/format".to_owned(),
                message: "unsupported live protocol version".to_owned(),
            }]),
        });
    }
    if value.get("type").and_then(|kind| kind.as_str()) != Some("open") {
        return Err(ServerError::Validation {
            issues: Box::new(vec![ValidationIssue {
                path: "/type".to_owned(),
                message: "feed opening must carry the open discriminator".to_owned(),
            }]),
        });
    }
    let open_id = value
        .get("openId")
        .and_then(|open_id| open_id.as_str())
        .ok_or_else(|| ServerError::Validation {
            issues: Box::new(vec![ValidationIssue {
                path: "/openId".to_owned(),
                message: "feed opening omitted its open id".to_owned(),
            }]),
        })?;
    trellis_protocol::parse_nonce(open_id, ["openId"]).map_err(|error| {
        ServerError::Validation {
            issues: Box::new(vec![ValidationIssue {
                path: "/openId".to_owned(),
                message: error.to_string(),
            }]),
        }
    })?;
    let receive_max_payload_bytes = value
        .get("receiveMaxPayloadBytes")
        .and_then(|limit| limit.as_u64())
        .ok_or_else(|| ServerError::Validation {
            issues: Box::new(vec![ValidationIssue {
                path: "/receiveMaxPayloadBytes".to_owned(),
                message: "feed opening omitted its receive payload limit".to_owned(),
            }]),
        })?;
    let input_value = value
        .get("input")
        .cloned()
        .ok_or_else(|| ServerError::Validation {
            issues: Box::new(vec![ValidationIssue {
                path: "/input".to_owned(),
                message: "feed opening omitted its native input".to_owned(),
            }]),
        })?;
    let input = TInput::decode(input_value).map_err(|error| ServerError::Validation {
        issues: Box::new(vec![ValidationIssue {
            path: "/input".to_owned(),
            message: error.to_string(),
        }]),
    })?;
    Ok(FeedOpening {
        open_id: open_id.to_owned(),
        receive_max_payload_bytes,
        input,
    })
}

/// One parsed Feed opening.
pub(crate) struct FeedOpening<TInput> {
    pub open_id: String,
    pub receive_max_payload_bytes: u64,
    pub input: TInput,
}

/// Protocol metadata of one parsed Feed opening, independent of the native
/// input so the decoded input can move into the delayed source factory.
pub(crate) struct FeedOpeningMeta {
    pub open_id: String,
    pub receive_max_payload_bytes: u64,
}

/// Adapt one generated Feed handler stream into the engine's source items.
///
/// Each encoded event is emitted as one application value and a normal stream
/// completion becomes the engine's explicit `End` item.
pub(crate) fn source_from_handler<TEvent, S>(
    stream: S,
) -> std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<SourceItem, String>> + Send>>
where
    TEvent: crate::generated::Codec + 'static,
    S: futures_util::Stream<Item = Result<TEvent, ServerError>> + Send + 'static,
{
    Box::pin(stream.map(|item| {
        match item {
            Ok(event) => crate::generated::Codec::encode(&event)
                .map(SourceItem::Value)
                .map_err(|error| error.to_string()),
            Err(error) => Err(error.to_string()),
        }
    }))
}

/// One reserved provider Feed open ready to return its offer.
pub(crate) struct ReservedFeed {
    pub prepared: LivePreparedResponse,
}

/// Reserve one Feed session, publish its activation challenge task, and build
/// the signed offer reply.
///
/// # Errors
///
/// Returns a setup error when the manager is unavailable, the transport epoch
/// changed, admission is exhausted, or the offer cannot be authenticated.
pub(crate) async fn reserve_feed<D, F>(
    client: &TrellisClient,
    manager: &std::sync::Arc<LiveSessionManager>,
    request: &FeedOpenRequest,
    source_factory: F,
) -> Result<ReservedFeed, ServerError>
where
    D: crate::generated::FeedDescriptor,
    F: FnOnce() -> std::pin::Pin<
            Box<dyn futures_util::Stream<Item = Result<SourceItem, String>> + Send>,
        > + Send
        + 'static,
{
    let context = &request.request;
    let inputs = &request.inputs;
    let opening = &request.opening;
    let encoded_input = request.encoded_input.clone();
    let cancellation = request.cancellation.clone();
    manager.is_available().map_err(|unavailable| {
        ServerError::Nats(format!("live manager unavailable: {unavailable:?}"))
    })?;
    let caller = context
        .caller
        .as_ref()
        .ok_or_else(|| ServerError::RequestDenied {
            subject: context.subject.clone(),
            session_key: context.session_key.clone().unwrap_or_default(),
        })?;
    let request_id = context
        .request_id
        .clone()
        .ok_or_else(|| ServerError::Nats("feed request is missing a request id".to_owned()))?;
    let own_context_digest = client
        .authorization_context_digest()
        .map_err(|error| ServerError::Nats(error.to_string()))?;

    let consumer = PinnedPeerIdentity {
        connection_id: caller.connection_id.clone(),
        session_key: caller.session_key.clone(),
        principal_id: caller.principal_id.clone(),
        participant_id: caller.participant_id.clone(),
        deployment_id: caller.deployment_id.clone(),
        instance_id: caller.instance_id.clone(),
    };
    let permission = trellis_protocol::PermissionAtom::new(
        trellis_protocol::PermissionTarget::api_surface(
            D::API_ID,
            trellis_protocol::ApiSurfaceKind::Feed,
            D::DESCRIPTOR_NAME,
        )
        .map_err(|error| ServerError::Nats(error.to_string()))?,
        trellis_protocol::PermissionAction::Subscribe,
    )
    .map_err(|error| ServerError::Nats(error.to_string()))?;
    // Retain both sides' authority before reserving anything expensive.
    let own_guard = LiveAuthorityGuard::retain(
        client.authorization_provider(),
        &own_context_digest,
        permission.clone(),
    )
    .await
    .map_err(|lost| ServerError::Nats(format!("provider authority unavailable: {lost:?}")))?;
    let _caller_guard = LiveAuthorityGuard::retain(
        client.authorization_provider(),
        &caller.context_digest,
        permission,
    )
    .await
    .map_err(|lost| ServerError::Nats(format!("caller authority unavailable: {lost:?}")))?;
    let negotiated = trellis_protocol::negotiate_max_data_body_bytes(
        opening.receive_max_payload_bytes,
        client.nats().max_payload() as u64,
    )
    .map_err(|error| ServerError::Validation {
        issues: Box::new(vec![ValidationIssue {
            path: "/receiveMaxPayloadBytes".to_owned(),
            message: error.to_string(),
        }]),
    })?;
    let canonical_open_hash =
        trellis_protocol::logical_open_hash(&trellis_protocol::LogicalOpenIdentity {
            kind: LiveSessionKind::Feed,
            base_subject: inputs.base_subject.clone(),
            open_id: opening.open_id.clone(),
            consumer_connection_id: consumer.connection_id.clone(),
            consumer_session_key: consumer.session_key.clone(),
            consumer_principal_id: consumer.principal_id.clone(),
            consumer_participant_id: consumer.participant_id.clone(),
            receive_max_payload_bytes: opening.receive_max_payload_bytes,
            feed_input: Some(encoded_input),
            operation_id: None,
            include_updates: None,
        })
        .map_err(|error| ServerError::Nats(error.to_string()))?;
    let now_ms = crate::client::now_iat_seconds() * 1_000;
    let open_request = ProviderOpenRequest {
        kind: LiveSessionKind::Feed,
        base_subject: inputs.base_subject.clone(),
        open_id: opening.open_id.clone(),
        consumer: consumer.clone(),
        consumer_max_payload_bytes: opening.receive_max_payload_bytes,
        canonical_open_hash,
        caller_guard: _caller_guard,
        own_guard,
    };
    let (session, manager) = ProviderSessionRecord::reserve(
        manager,
        &open_request,
        &inputs.provider_deployment_id,
        negotiated,
        now_ms,
    )
    .await
    .map_err(|code| ServerError::Nats(format!("live reservation rejected: {code:?}")))?;
    let limits = LiveOfferLimits {
        max_data_body_bytes: negotiated,
        window_frames: trellis_protocol::WINDOW_FRAMES,
        window_bytes: trellis_protocol::WINDOW_BYTES,
        reservation_ms: OPEN_RESERVATION_MS,
        heartbeat_interval_ms: trellis_protocol::HEARTBEAT_INTERVAL_MS,
        peer_inactivity_ms: trellis_protocol::PEER_INACTIVITY_MS,
        consumer_stall_ms: trellis_protocol::CONSUMER_STALL_MS,
    };
    let record = std::sync::Arc::new(ProviderSessionRecord {
        session: std::sync::Arc::clone(&session),
        manager: std::sync::Arc::clone(&manager),
        source_factory: std::sync::Mutex::new(Some(Box::new(source_factory))),
        cleanup: std::sync::Mutex::new(Vec::new()),
        terminal: std::sync::Mutex::new(None),
        cancellation: cancellation.clone(),
    });
    manager.insert_provider_session(session.session_id.clone(), std::sync::Arc::clone(&record));
    let offer = record
        .offer_body(
            &request_id,
            &client
                .own_pinned_identity()
                .map_err(|error| ServerError::Nats(error.to_string()))?,
            &consumer,
            limits,
            negotiated,
        )
        .map_err(|code| ServerError::Nats(format!("offer build failed: {code:?}")))?;
    // Authenticate the exact offer bytes with the provider's live proof.
    let headers = crate::service::live_router::sign_live_offer(
        client.auth(),
        &own_context_digest,
        &context.reply_to.clone().unwrap_or_default(),
        &offer,
    )
    .map_err(|error| ServerError::Nats(error.to_string()))?;
    // Install the owner-control subscription before the offer is published so
    // an immediate activation cannot be lost.
    spawn_session_drivers(client.nats(), std::sync::Arc::clone(&record), negotiated)
        .await
        .map_err(|code| ServerError::Nats(format!("live control subscription failed: {code:?}")))?;
    Ok(ReservedFeed {
        prepared: LivePreparedResponse {
            offer,
            headers,
            manager,
            request_id,
        },
    })
}

/// Spawn the provider's owner-control and activation driver tasks.
///
/// The tasks own their session record and stop on session close; they never
/// hold a lock across a network await.
pub(crate) async fn spawn_session_drivers(
    nats: async_nats::Client,
    record: std::sync::Arc<ProviderSessionRecord>,
    max_data_body_bytes: u64,
) -> Result<(), LiveErrorCode> {
    let control_record = std::sync::Arc::clone(&record);
    let control_nats = nats.clone();
    // A single owner-control pump drives activation, credit and closure for
    // this session. Its subscription is established and flushed before the
    // offer becomes visible to the consumer.
    let mut subscription = control_nats
        .subscribe(control_record.session.control_subject.clone())
        .await
        .map_err(|_| LiveErrorCode::ResourceExhausted)?;
    control_nats
        .flush()
        .await
        .map_err(|_| LiveErrorCode::Disconnected)?;
    tokio::spawn(async move {
        let mut challenge_interval = tokio::time::interval(cleanup_grace());
        loop {
            tokio::select! {
                message = subscription.next() => {
                    let Some(message) = message else { return; };
                    let Ok(control) = trellis_protocol::parse_live_control(&message.payload) else {
                        continue;
                    };
                    let now_ms = crate::client::now_iat_seconds() * 1_000;
                    let request_id = message
                        .headers
                        .as_ref()
                        .and_then(|headers| headers.get("request-id"))
                        .map_or_else(String::new, ToString::to_string);
                    match control_record.handle_control(&control, now_ms).await {
                        Ok(outcome) => {
                            if let Some(challenge_id) = outcome.challenge.clone() {
                                let _ = publish_challenge(
                                    &control_record.session,
                                    &control_nats,
                                    &challenge_id,
                                )
                                .await;
                                // The source starts only after the first fresh
                                // pulse commits activation.
                                let activated = wait_for_activation(
                                    &control_record,
                                    challenge_id,
                                )
                                .await;
                                if activated {
                                    if let Ok(source) =
                                        take_source(&control_record.source_factory)
                                    {
                                        let driver_record =
                                            std::sync::Arc::clone(&control_record);
                                        let driver_nats = control_nats.clone();
                                        let driver_cancellation =
                                            control_record.cancellation_handle();
                                        tokio::spawn(async move {
                                            drive_source(
                                                std::sync::Arc::clone(&driver_record.session),
                                                std::sync::Arc::clone(&driver_record),
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
                            let ack = ack_for(
                                &control_record.session,
                                &control,
                                &request_id,
                                &outcome,
                            );
                            let body = serde_json::to_vec(&ack).unwrap_or_default();
                            let headers = sign_control_reply(
                                &control_record,
                                &message,
                                &body,
                            );
                            if let Some(reply) = message.reply.as_ref() {
                                let _ = control_nats
                                    .publish_with_headers(
                                        reply.clone(),
                                        headers,
                                        Bytes::from(body),
                                    )
                                    .await;
                            }
                            if outcome.state == trellis_protocol::LiveSessionState::Closed {
                                let cleanup =
                                    control_record.run_owned_cleanup().await;
                                let receipt = control_record.tombstone(cleanup, now_ms);
                                control_record
                                    .manager
                                    .insert_receipt(receipt);
                                control_record
                                    .manager
                                    .remove_provider_session(
                                        &control_record.session.session_id,
                                    );
                                return;
                            }
                        }
                        Err(code) => {
                            let error = error_for(
                                &control_record.session,
                                &control,
                                &request_id,
                                code,
                            );
                            let body = serde_json::to_vec(&error).unwrap_or_default();
                            let headers = sign_control_reply(
                                &control_record,
                                &message,
                                &body,
                            );
                            if let Some(reply) = message.reply.as_ref() {
                                let _ = control_nats
                                    .publish_with_headers(
                                        reply.clone(),
                                        headers,
                                        Bytes::from(body),
                                    )
                                    .await;
                            }
                        }
                    }
                }
                _ = challenge_interval.tick() => {
                    // Reservation expiry fences an unactivated source and
                    // repeats the same outstanding challenge every two
                    // seconds while activating.
                    let now_ms = crate::client::now_iat_seconds() * 1_000;
                    if control_record.session.reservation_elapsed(now_ms) {
                        control_record.commit_end(LiveEnd::new(
                            trellis_protocol::LiveEndReason::SetupTimeout,
                            None,
                        ));
                        control_record.session.set_phase(
                            crate::live::provider::ProviderPhase::Closing,
                        );
                        let cleanup = control_record.run_owned_cleanup().await;
                        let receipt = control_record.tombstone(cleanup, now_ms);
                        control_record.manager.insert_receipt(receipt);
                        control_record
                            .manager
                            .remove_provider_session(&control_record.session.session_id);
                        return;
                    }
                    if matches!(
                        control_record.session.phase(),
                        crate::live::provider::ProviderPhase::Activating
                    ) {
                        let challenge = control_record
                            .session
                            .challenge
                            .lock()
                            .ok()
                            .and_then(|challenge| {
                                challenge.as_ref().map(|challenge| {
                                    challenge.challenge_id.clone()
                                })
                            });
                        if let Some(challenge_id) = challenge {
                            let _ = publish_challenge(
                                &control_record.session,
                                &control_nats,
                                &challenge_id,
                            )
                            .await;
                        }
                    }
                }
            }
        }
    });
    Ok(())
}

/// Wait for the first fresh pulse to commit activation.
///
/// Returns immediately on a committed active state; the caller then starts the
/// source exactly once.
async fn wait_for_activation(
    record: &std::sync::Arc<ProviderSessionRecord>,
    challenge_id: String,
) -> bool {
    let deadline = tokio::time::Instant::now() + cleanup_grace();
    loop {
        if matches!(
            record.session.phase(),
            crate::live::provider::ProviderPhase::Active
        ) {
            return true;
        }
        if record
            .session
            .challenge
            .lock()
            .ok()
            .and_then(|challenge| {
                challenge
                    .as_ref()
                    .map(|challenge| challenge.answered && challenge.challenge_id == challenge_id)
            })
            .unwrap_or(false)
        {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

/// Sign one control reply with the provider's live server proof.
fn sign_control_reply(
    record: &std::sync::Arc<ProviderSessionRecord>,
    request: &async_nats::Message,
    body: &[u8],
) -> async_nats::HeaderMap {
    let context_digest = record
        .manager
        .contexts_handle()
        .context_digest()
        .unwrap_or_default();
    let mut headers = async_nats::HeaderMap::new();
    headers.insert("authorization-context", context_digest.as_str());
    headers.insert(
        "session-key",
        record.manager.auth_handle().session_key.as_str(),
    );
    if let Ok(proof) = trellis_protocol::sign_live_server_proof(
        &context_digest,
        &request.subject,
        body,
        record.manager.auth_handle().live_signing_key(),
    ) {
        headers.insert("trellis-live-proof", proof.as_str());
    }
    headers
}

/// Sign one provider-origin message with the provider's live proof.
///
/// # Errors
///
/// Returns an error when the proof cannot be built.
pub(crate) fn sign_live_offer(
    auth: &crate::client::SessionAuth,
    context_digest: &str,
    subject: &str,
    body: &[u8],
) -> Result<async_nats::HeaderMap, trellis_protocol::ProtocolError> {
    let proof = trellis_protocol::sign_live_server_proof(
        context_digest,
        subject,
        body,
        auth.live_signing_key(),
    )?;
    let mut headers = async_nats::HeaderMap::new();
    headers.insert("authorization-context", context_digest);
    headers.insert("session-key", auth.session_key.as_str());
    headers.insert("trellis-live-proof", proof.as_str());
    Ok(headers)
}

/// Return the exact owner-control wildcard for one Feed route.
///
/// # Errors
///
/// Returns an error for an invalid subject or connection id.
pub(crate) fn owner_control_wildcard(
    base_subject: &str,
    provider_connection_id: &str,
) -> Result<String, trellis_protocol::ProtocolError> {
    derive_live_observe_wildcard_subject(base_subject, provider_connection_id)
}

/// Return the exact data subject for one session.
///
/// # Errors
///
/// Returns an error for an invalid connection id or session id.
pub(crate) fn data_subject(
    provider_connection_id: &str,
    consumer_connection_id: &str,
    session_id: &str,
) -> Result<String, trellis_protocol::ProtocolError> {
    derive_live_data_subject(provider_connection_id, consumer_connection_id, session_id)
}

/// Return whether one parsed opening body is a live open envelope.
#[must_use]
pub(crate) fn is_live_open_body(payload: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(payload).is_ok_and(|value| {
        value.get("format").and_then(|format| format.as_str())
            == Some(trellis_protocol::LIVE_VERSION)
            && value.get("type").and_then(|kind| kind.as_str()) == Some("open")
    })
}

/// Map one authority loss into a setup rejection.
#[must_use]
pub(crate) fn authority_rejection(lost: &LiveAuthorityLost) -> ServerError {
    ServerError::Nats(format!("live authority unavailable: {lost:?}"))
}

/// Map one wire error into a setup rejection.
#[must_use]
pub(crate) fn wire_rejection(code: LiveErrorCode) -> ServerError {
    ServerError::Nats(format!("live open rejected: {code:?}"))
}
