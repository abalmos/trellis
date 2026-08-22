use std::time::Duration;
use std::{future::Future, pin::Pin, sync::Arc};

use async_nats::jetstream::{self, AckKind};
use futures_util::{stream, StreamExt};
use serde_json::{json, Value};
use trellis_rs::service::EventVerificationFailure;
use trellis_rs::service::ServerError;

use crate::storage::{now_timestamp_string, EventLogStore, EventLogStoreError, ProjectedEvent};

pub(crate) type EventMessageStream = trellis_rs::service::EventLogMessageStream;

const EVENT_STREAM: &str = "trellis";
const EVENT_SUBJECT_WILDCARD: &str = "events.v1.>";
const PROJECTOR_BATCH_SIZE: usize = 100;
const PROJECTOR_CONCURRENCY: usize = 32;
const EVENT_ID_HEADER: &str = "Nats-Msg-Id";
const EVENT_TIME_HEADER: &str = "Trellis-Event-Time";
pub(crate) const CONSUMER_METADATA_MANAGED_BY: &str = "trellis.managed_by";
pub(crate) const CONSUMER_METADATA_DEPLOYMENT_ID: &str = "trellis.deployment_id";
pub(crate) const CONSUMER_METADATA_CONTRACT_ID: &str = "trellis.contract_id";
pub(crate) const CONSUMER_METADATA_GROUP: &str = "trellis.group";

/// Event Log transport facade over the Trellis event JetStream stream.
pub type EventLogRuntime = trellis_rs::service::EventLogRuntime;

/// Raw event fields passed to the runtime-owned local authorization verifier.
#[derive(Clone, Debug)]
pub struct EventAuthorizationInput {
    pub subject: String,
    pub payload: Vec<u8>,
    pub session_key: String,
    pub proof: String,
    pub authorization_context: String,
    pub event_id: String,
    pub event_time: String,
}

/// Publisher fields proven by a context-bound event proof.
#[derive(Clone, Debug)]
pub struct VerifiedEventPublisher {
    pub kind: String,
    pub deployment_id: Option<String>,
    pub instance_id: Option<String>,
    pub participant_id: String,
    pub participant_digest: String,
    pub session_id: String,
}

/// Runtime-owned local event verifier callback.
pub type EventVerifier = Arc<
    dyn Fn(
            EventAuthorizationInput,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<VerifiedEventPublisher, EventVerificationFailure>>
                    + Send,
            >,
        > + Send
        + Sync,
>;

/// Handle for the background Event Log projector task.
pub struct EventLogProjectorHandle {
    task: Option<tokio::task::JoinHandle<Result<(), ServerError>>>,
}

impl EventLogProjectorHandle {
    /// Stop the projector task.
    pub async fn stop(self) {
        let Some(task) = self.task else {
            return;
        };
        task.abort();
        let _ = task.await;
    }

    /// Drop a completed projector task after observing its result.
    pub fn discard_completed(&mut self) {
        self.task = None;
    }

    /// Wait for the projector task to finish.
    pub async fn wait(&mut self) -> Result<(), ServerError> {
        let Some(task) = self.task.as_mut() else {
            return Ok(());
        };
        match task.await {
            Ok(result) => result,
            Err(error) if error.is_cancelled() => Ok(()),
            Err(error) => Err(ServerError::Nats(format!(
                "event log projector loop task failed: {error}"
            ))),
        }
    }
}

/// Start projecting Trellis event messages into SQLite.
pub async fn start_eventlog_projector(
    runtime: EventLogRuntime,
    store: EventLogStore,
    verifier: EventVerifier,
) -> Result<EventLogProjectorHandle, ServerError> {
    let consumer_name = projector_consumer_name(&store.projection_id().map_err(|error| {
        ServerError::Nats(format!(
            "failed to resolve Event Log projection identity: {error}"
        ))
    })?);
    let consumer = runtime
        .event_consumer(&consumer_name, false)
        .await
        .map_err(|error| {
            ServerError::Nats(format!(
                "failed to start Event Log projector consumer '{consumer_name}': {error}"
            ))
        })?;
    tracing::info!(stream = EVENT_STREAM, consumer = %consumer_name, filter = EVENT_SUBJECT_WILDCARD, "started Event Log projector consumer");

    let task = tokio::spawn(async move {
        loop {
            let mut batch = Vec::new();
            let mut messages = consumer
                .fetch()
                .max_messages(PROJECTOR_BATCH_SIZE)
                .expires(Duration::from_millis(500))
                .messages()
                .await
                .map_err(|error| {
                    ServerError::Nats(format!(
                        "event log projector failed to fetch batch: {error}"
                    ))
                })?;
            while let Some(message) = messages.next().await {
                collect_message(message, &mut batch)?;
            }
            if batch.is_empty() {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            let mut projected = stream::iter(batch.into_iter().map(|message| {
                let store = store.clone();
                let verifier = Arc::clone(&verifier);
                async move { process_message(store, message, verifier).await }
            }))
            .buffer_unordered(PROJECTOR_CONCURRENCY);
            while let Some(result) = projected.next().await {
                result?;
            }
        }
    });

    Ok(EventLogProjectorHandle { task: Some(task) })
}

fn collect_message(
    message: Result<jetstream::Message, impl std::fmt::Display>,
    batch: &mut Vec<jetstream::Message>,
) -> Result<(), ServerError> {
    batch.push(message.map_err(|error| {
        ServerError::Nats(format!(
            "event log projector failed to pull message: {error}"
        ))
    })?);
    Ok(())
}

async fn process_message(
    store: EventLogStore,
    message: jetstream::Message,
    verifier: EventVerifier,
) -> Result<(), ServerError> {
    let stream_sequence = match message.info() {
        Ok(info) => info.stream_sequence,
        Err(error) => {
            tracing::warn!(%error, subject = %message.subject, "retrying event log message with unavailable metadata");
            let _ = message
                .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))
                .await;
            return Ok(());
        }
    };
    let existing_store = store.clone();
    let already_projected = match tokio::task::spawn_blocking(move || {
        existing_store.contains_stream_sequence(stream_sequence)
    })
    .await
    {
        Ok(Ok(already_projected)) => already_projected,
        Ok(Err(error)) => {
            tracing::warn!(%error, subject = %message.subject, "retrying event log duplicate check");
            let _ = message
                .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))
                .await;
            return Ok(());
        }
        Err(error) => {
            tracing::warn!(%error, subject = %message.subject, "retrying event log duplicate-check task");
            let _ = message
                .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))
                .await;
            return Ok(());
        }
    };
    if already_projected {
        let _ = message.ack().await;
        return Ok(());
    }
    match project_message_inner(&message, verifier).await {
        Ok(event) => match tokio::task::spawn_blocking(move || store.insert_event(&event)).await {
            Ok(Ok(())) => {
                let _ = message.ack().await;
            }
            Ok(Err(error)) => {
                tracing::warn!(%error, subject = %message.subject, "retrying Event Log persistence");
                let _ = message
                    .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))
                    .await;
            }
            Err(error) => {
                tracing::warn!(%error, subject = %message.subject, "retrying Event Log persistence task");
                let _ = message
                    .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))
                    .await;
            }
        },
        Err(EventVerificationFailure::Retryable(error)) => {
            tracing::warn!(%error, subject = %message.subject, "retrying temporarily unverifiable event log message");
            let _ = message
                .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))
                .await;
        }
        Err(EventVerificationFailure::Rejected(error)) => {
            tracing::warn!(%error, subject = %message.subject, "dropping rejected event log message");
            let _ = message.ack().await;
        }
    }
    Ok(())
}

async fn project_message_inner(
    message: &jetstream::Message,
    verifier: EventVerifier,
) -> Result<ProjectedEvent, EventVerificationFailure> {
    let info = message
        .info()
        .map_err(|error| EventVerificationFailure::Retryable(error.to_string()))?;
    let headers =
        headers_json(message.headers.as_ref()).map_err(EventVerificationFailure::Rejected)?;
    let event_id =
        required_header(message, EVENT_ID_HEADER).map_err(EventVerificationFailure::Rejected)?;
    let event_time =
        required_header(message, EVENT_TIME_HEADER).map_err(EventVerificationFailure::Rejected)?;
    let publisher = verifier(EventAuthorizationInput {
        subject: message.subject.to_string(),
        payload: message.payload.to_vec(),
        session_key: required_header(message, "session-key")
            .map_err(EventVerificationFailure::Rejected)?,
        proof: required_header(message, "proof").map_err(EventVerificationFailure::Rejected)?,
        authorization_context: required_header(message, "authorization-context")
            .map_err(EventVerificationFailure::Rejected)?,
        event_id: event_id.clone(),
        event_time: event_time.clone(),
    })
    .await?;
    let traceparent = header_value(message, "traceparent").map(str::to_string);
    let trace_id = traceparent
        .as_deref()
        .and_then(trace_id_from_traceparent)
        .map(str::to_string);
    let (payload_json, payload_text, decode_error) = decode_payload(&message.payload);
    Ok(ProjectedEvent {
        stream_sequence: info.stream_sequence,
        event_id: Some(event_id),
        event_time,
        subject: message.subject.to_string(),
        owner_contract_id: None,
        owner_event_name: None,
        resolution: "unresolved".to_string(),
        verification_status: "verified".to_string(),
        publisher_kind: Some(publisher.kind),
        publisher_deployment_id: publisher.deployment_id,
        publisher_instance_id: publisher.instance_id,
        publisher_participant_id: Some(publisher.participant_id),
        publisher_participant_digest: Some(publisher.participant_digest),
        publisher_session_id: Some(publisher.session_id),
        trace_id,
        traceparent,
        payload_bytes: message.payload.to_vec(),
        headers_json: headers.to_string(),
        payload_json,
        payload_text,
        decode_error,
        projected_at: now_timestamp_string(),
    })
}

fn required_header(message: &jetstream::Message, name: &str) -> Result<String, String> {
    header_value(message, name)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("event header {name} is missing"))
}

fn header_value<'a>(message: &'a jetstream::Message, name: &str) -> Option<&'a str> {
    message
        .headers
        .as_ref()?
        .get(name)
        .map(|value| value.as_str())
}

fn headers_json(headers: Option<&async_nats::HeaderMap>) -> Result<Value, String> {
    let mut object = serde_json::Map::new();
    if let Some(headers) = headers {
        for (name, value) in headers.iter() {
            object.insert(
                name.to_string(),
                json!(value.iter().map(|value| value.as_str()).collect::<Vec<_>>()),
            );
        }
    }
    Ok(Value::Object(object))
}

fn decode_payload(payload: &[u8]) -> (Option<String>, Option<String>, Option<String>) {
    match std::str::from_utf8(payload) {
        Ok(text) => {
            let payload_json = serde_json::from_str::<Value>(text)
                .ok()
                .map(|value| value.to_string());
            (payload_json, Some(text.to_string()), None)
        }
        Err(error) => (None, None, Some(error.to_string())),
    }
}

fn trace_id_from_traceparent(traceparent: &str) -> Option<&str> {
    traceparent
        .split('-')
        .nth(1)
        .filter(|value| value.len() == 32)
}

fn projector_consumer_name(projection_id: &str) -> String {
    format!(
        "eventlog-projector-{}",
        sanitize_consumer_token(projection_id)
    )
}

pub(crate) fn is_eventlog_projector_consumer(name: &str) -> bool {
    name.starts_with("eventlog-projector-")
}

fn sanitize_consumer_token(value: &str) -> String {
    let token = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .collect::<String>();
    if token.is_empty() {
        "projection".to_string()
    } else {
        token
    }
}

impl From<EventLogStoreError> for ServerError {
    fn from(error: EventLogStoreError) -> Self {
        ServerError::Nats(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{is_eventlog_projector_consumer, projector_consumer_name};

    #[test]
    fn projector_consumer_name_removes_timestamp_punctuation() {
        assert_eq!(
            projector_consumer_name("1877448-2026-07-09T04:28:06.672734698Z"),
            "eventlog-projector-1877448-2026-07-09T042806672734698Z",
        );
    }

    #[test]
    fn eventlog_consumer_ownership_recognizes_projector_names() {
        assert!(is_eventlog_projector_consumer("eventlog-projector-current"));
        assert!(!is_eventlog_projector_consumer(
            "event-log-projector-legacy"
        ));
    }
}
