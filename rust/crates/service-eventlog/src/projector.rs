use std::time::Duration;

use async_nats::jetstream;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use futures_util::{stream, StreamExt};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use time::format_description::well_known::Rfc3339;
use trellis_rs::sdk::auth::types::AuthEventsValidateRequest;
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

    pub(crate) fn discard_completed(&mut self) {
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
                let runtime = runtime.clone();
                let store = store.clone();
                async move { process_message(runtime, store, message).await }
            }))
            .buffer_unordered(PROJECTOR_CONCURRENCY);
            while let Some(result) = projected.next().await {
                result?;
            }
        }
    });

    Ok(EventLogProjectorHandle { task: Some(task) })
}

#[expect(
    clippy::result_large_err,
    reason = "ServerError preserves typed projector diagnostics"
)]
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
    runtime: EventLogRuntime,
    store: EventLogStore,
    message: jetstream::Message,
) -> Result<(), ServerError> {
    match project_message(&runtime, message).await {
        Ok((message, event)) => {
            tokio::task::spawn_blocking(move || store.insert_event(&event))
                .await
                .map_err(|error| {
                    ServerError::Nats(format!("event log projector task failed: {error}"))
                })?
                .map_err(|error| {
                    ServerError::Nats(format!(
                        "event log projector failed to persist event: {error}"
                    ))
                })?;
            let _ = message.ack().await;
        }
        Err((message, error)) => {
            tracing::warn!(%error, subject = %message.subject, "dropping unprojectable event log message");
            let _ = message.ack().await;
        }
    }
    Ok(())
}

async fn project_message(
    runtime: &EventLogRuntime,
    message: jetstream::Message,
) -> Result<(jetstream::Message, ProjectedEvent), (jetstream::Message, String)> {
    match project_message_inner(runtime, &message).await {
        Ok(event) => Ok((message, event)),
        Err(error) => Err((message, error)),
    }
}

async fn project_message_inner(
    runtime: &EventLogRuntime,
    message: &jetstream::Message,
) -> Result<ProjectedEvent, String> {
    let info = message.info().map_err(|error| error.to_string())?;
    let headers = headers_json(message.headers.as_ref())?;
    let event_id = header_value(message, EVENT_ID_HEADER).map(str::to_string);
    let event_time = header_value(message, EVENT_TIME_HEADER)
        .map(str::to_string)
        .unwrap_or_else(|| {
            info.published
                .format(&Rfc3339)
                .unwrap_or_else(|_| now_timestamp_string())
        });
    let traceparent = header_value(message, "traceparent").map(str::to_string);
    let trace_id = traceparent
        .as_deref()
        .and_then(trace_id_from_traceparent)
        .map(str::to_string);
    let (payload_json, payload_text, decode_error) = decode_payload(&message.payload);
    let auth = validate_event(runtime, message, event_id.as_deref(), &event_time).await;

    Ok(ProjectedEvent {
        stream_sequence: info.stream_sequence,
        event_id,
        event_time,
        subject: message.subject.to_string(),
        owner_contract_id: None,
        owner_event_name: None,
        resolution: "unresolved".to_string(),
        verification_status: auth.status,
        publisher_kind: auth
            .publisher
            .as_ref()
            .map(|publisher| publisher.kind.as_str().to_string()),
        publisher_deployment_id: auth
            .publisher
            .as_ref()
            .and_then(|publisher| publisher.deployment_id.clone()),
        publisher_instance_id: auth
            .publisher
            .as_ref()
            .and_then(|publisher| publisher.instance_id.clone()),
        publisher_contract_id: auth
            .publisher
            .as_ref()
            .and_then(|publisher| publisher.contract_id.clone()),
        publisher_contract_digest: auth
            .publisher
            .as_ref()
            .and_then(|publisher| publisher.contract_digest.clone()),
        publisher_session_status: auth
            .publisher
            .as_ref()
            .map(|publisher| publisher.session_status.as_str().to_string()),
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

struct ValidationResult {
    status: String,
    publisher: Option<trellis_rs::sdk::auth::types::AuthEventsValidateResponsePublisher>,
}

async fn validate_event(
    runtime: &EventLogRuntime,
    message: &jetstream::Message,
    event_id: Option<&str>,
    event_time: &str,
) -> ValidationResult {
    let Some(event_id) = event_id else {
        return ValidationResult {
            status: "missing-proof".to_string(),
            publisher: None,
        };
    };
    let Some(session_key) = header_value(message, "session-key") else {
        return ValidationResult {
            status: "missing-proof".to_string(),
            publisher: None,
        };
    };
    let Some(proof) = header_value(message, "proof") else {
        return ValidationResult {
            status: "missing-proof".to_string(),
            publisher: None,
        };
    };
    let request = AuthEventsValidateRequest {
        event_id: event_id.to_string(),
        event_time: event_time.to_string(),
        payload_hash: payload_hash_base64url(&message.payload),
        proof: proof.to_string(),
        session_key: session_key.to_string(),
        subject: message.subject.to_string(),
    };
    match runtime.validate_event(&request).await {
        Ok(response) => ValidationResult {
            status: response.status.as_str().to_string(),
            publisher: response.publisher,
        },
        Err(error) => {
            tracing::warn!(%error, subject = %message.subject, "Auth.Events.Validate unavailable for event projection");
            ValidationResult {
                status: "auth-unavailable".to_string(),
                publisher: None,
            }
        }
    }
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

fn payload_hash_base64url(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    URL_SAFE_NO_PAD.encode(hasher.finalize())
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
