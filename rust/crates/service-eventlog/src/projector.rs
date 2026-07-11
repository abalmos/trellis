use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream::consumer::FromConsumer;
use async_nats::jetstream::{self, consumer};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use futures_util::{stream, stream::BoxStream, StreamExt};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use time::format_description::well_known::Rfc3339;
use trellis_rs::client::TrellisClient;
use trellis_rs::sdk::auth::rpc::AuthEventsValidateRpc;
use trellis_rs::sdk::auth::types::AuthEventsValidateRequest;
use trellis_rs::service::ServerError;

use crate::storage::{now_timestamp_string, EventLogStore, EventLogStoreError, ProjectedEvent};

pub(crate) type EventMessageStream =
    BoxStream<'static, Result<jetstream::Message, Box<dyn std::error::Error + Send + Sync>>>;

const EVENT_STREAM: &str = "trellis";
const EVENT_SUBJECT_WILDCARD: &str = "events.v1.>";
const PROJECTOR_BATCH_SIZE: usize = 100;
const PROJECTOR_CONCURRENCY: usize = 32;
const LIVE_WATCH_INACTIVE_THRESHOLD: Duration = Duration::from_secs(5);
const OBSOLETE_WATCH_INACTIVE_THRESHOLD: Duration = Duration::from_secs(5 * 60);
const EVENT_ID_HEADER: &str = "Nats-Msg-Id";
const EVENT_TIME_HEADER: &str = "Trellis-Event-Time";
pub(crate) const CONSUMER_METADATA_MANAGED_BY: &str = "trellis.managed_by";
pub(crate) const CONSUMER_METADATA_DEPLOYMENT_ID: &str = "trellis.deployment_id";
pub(crate) const CONSUMER_METADATA_CONTRACT_ID: &str = "trellis.contract_id";
pub(crate) const CONSUMER_METADATA_GROUP: &str = "trellis.group";

/// Event Log transport facade over the Trellis event JetStream stream.
#[derive(Debug, Clone)]
pub struct EventLogRuntime {
    nats: async_nats::Client,
}

impl EventLogRuntime {
    /// Create an Event Log runtime facade from a connected Trellis client.
    pub fn from_client(client: &TrellisClient) -> Self {
        Self {
            nats: client.internal_nats().clone(),
        }
    }

    pub(crate) async fn live_events(&self) -> Result<EventMessageStream, String> {
        let jetstream = jetstream::new(self.nats.clone());
        let stream = jetstream
            .get_stream(EVENT_STREAM)
            .await
            .map_err(|error| error.to_string())?;
        let consumer = stream
            .create_consumer(live_event_consumer_config())
            .await
            .map_err(|error| error.to_string())?;
        consumer
            .messages()
            .await
            .map(|messages| {
                messages
                    .map(|message| {
                        message.map_err(|error| {
                            Box::new(error) as Box<dyn std::error::Error + Send + Sync>
                        })
                    })
                    .boxed()
            })
            .map_err(|error| error.to_string())
    }

    pub(crate) async fn expire_obsolete_watch_consumers(&self) -> Result<usize, String> {
        let jetstream = jetstream::new(self.nats.clone());
        let stream = jetstream
            .get_stream(EVENT_STREAM)
            .await
            .map_err(|error| error.to_string())?;
        let mut consumers = stream.consumers();
        let mut configs = Vec::new();
        while let Some(info) = consumers.next().await {
            let info = info.map_err(|error| error.to_string())?;
            if is_obsolete_eventlog_watch_consumer(&info.name, &info.config) {
                let mut config = consumer::pull::Config::try_from_consumer_config(info.config)
                    .map_err(|error| error.to_string())?;
                config.inactive_threshold = OBSOLETE_WATCH_INACTIVE_THRESHOLD;
                config.metadata.extend(platform_consumer_metadata("watch"));
                configs.push(config);
            }
        }
        for config in configs.iter().cloned() {
            stream
                .update_consumer(config)
                .await
                .map_err(|error| error.to_string())?;
        }
        Ok(configs.len())
    }

    pub(crate) async fn event_consumer(
        &self,
        consumer_name: &str,
        replay_all: bool,
    ) -> Result<consumer::Consumer<consumer::pull::Config>, String> {
        let jetstream = jetstream::new(self.nats.clone());
        let stream = jetstream
            .get_stream(EVENT_STREAM)
            .await
            .map_err(|error| error.to_string())?;
        stream
            .get_or_create_consumer(
                consumer_name,
                consumer::pull::Config {
                    durable_name: Some(consumer_name.to_string()),
                    filter_subject: EVENT_SUBJECT_WILDCARD.to_string(),
                    deliver_policy: if replay_all {
                        consumer::DeliverPolicy::All
                    } else {
                        consumer::DeliverPolicy::New
                    },
                    ack_policy: consumer::AckPolicy::Explicit,
                    metadata: platform_consumer_metadata("projector"),
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| error.to_string())
    }

    pub(crate) async fn consumers(&self) -> Result<Vec<consumer::Info>, String> {
        let jetstream = jetstream::new(self.nats.clone());
        let stream = jetstream
            .get_stream(EVENT_STREAM)
            .await
            .map_err(|error| error.to_string())?;
        let mut consumers = stream.consumers();
        let mut rows = Vec::new();
        while let Some(info) = consumers.next().await {
            rows.push(info.map_err(|error| error.to_string())?);
        }
        Ok(rows)
    }

    pub(crate) async fn consumer(&self, name: &str) -> Result<consumer::Info, String> {
        let jetstream = jetstream::new(self.nats.clone());
        let stream = jetstream
            .get_stream(EVENT_STREAM)
            .await
            .map_err(|error| error.to_string())?;
        stream
            .consumer_info(name)
            .await
            .map_err(|error| error.to_string())
    }
}

fn live_event_consumer_config() -> consumer::pull::Config {
    consumer::pull::Config {
        filter_subject: EVENT_SUBJECT_WILDCARD.to_string(),
        deliver_policy: consumer::DeliverPolicy::New,
        ack_policy: consumer::AckPolicy::Explicit,
        inactive_threshold: LIVE_WATCH_INACTIVE_THRESHOLD,
        metadata: platform_consumer_metadata("watch"),
        ..Default::default()
    }
}

fn platform_consumer_metadata(group: &str) -> HashMap<String, String> {
    HashMap::from([
        (
            CONSUMER_METADATA_MANAGED_BY.to_string(),
            "platform".to_string(),
        ),
        (
            CONSUMER_METADATA_CONTRACT_ID.to_string(),
            "trellis.eventlog@v1".to_string(),
        ),
        (CONSUMER_METADATA_GROUP.to_string(), group.to_string()),
    ])
}

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
    auth_client: Arc<TrellisClient>,
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
                let auth_client = Arc::clone(&auth_client);
                let store = store.clone();
                async move { process_message(auth_client, store, message).await }
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
    auth_client: Arc<TrellisClient>,
    store: EventLogStore,
    message: jetstream::Message,
) -> Result<(), ServerError> {
    match project_message(&auth_client, message).await {
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
    auth_client: &TrellisClient,
    message: jetstream::Message,
) -> Result<(jetstream::Message, ProjectedEvent), (jetstream::Message, String)> {
    match project_message_inner(auth_client, &message).await {
        Ok(event) => Ok((message, event)),
        Err(error) => Err((message, error)),
    }
}

async fn project_message_inner(
    auth_client: &TrellisClient,
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
    let auth = validate_event(auth_client, message, event_id.as_deref(), &event_time).await;

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
            .map(|publisher| publisher.kind.clone()),
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
            .map(|publisher| publisher.session_status.clone()),
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
    auth_client: &TrellisClient,
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
    match auth_client.call::<AuthEventsValidateRpc>(&request).await {
        Ok(response) => ValidationResult {
            status: response.status,
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
    name.starts_with("eventlog-projector-") || name.starts_with("event-log-projector-")
}

pub(crate) fn is_obsolete_eventlog_watch_consumer(name: &str, config: &consumer::Config) -> bool {
    let suffix = name
        .strip_prefix("eventlog-watch-")
        .or_else(|| name.strip_prefix("event-log-watch-"));
    let Some((seed, counter)) = suffix.and_then(|suffix| suffix.rsplit_once('-')) else {
        return false;
    };
    !seed.is_empty()
        && seed.len() <= 48
        && seed
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        && counter.parse::<u64>().is_ok()
        && config.durable_name.as_deref() == Some(name)
        && config.filter_subject == EVENT_SUBJECT_WILDCARD
        && config.filter_subjects.is_empty()
        && config.deliver_policy == consumer::DeliverPolicy::New
        && config.ack_policy == consumer::AckPolicy::Explicit
        && config.metadata.keys().all(|key| key.starts_with("_nats."))
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_nats::jetstream::consumer;

    use super::{
        is_eventlog_projector_consumer, is_obsolete_eventlog_watch_consumer,
        live_event_consumer_config, projector_consumer_name, CONSUMER_METADATA_MANAGED_BY,
        EVENT_SUBJECT_WILDCARD,
    };

    #[test]
    fn live_watch_consumer_is_ephemeral_and_new_only() {
        let config = live_event_consumer_config();

        assert!(config.durable_name.is_none());
        assert_eq!(config.deliver_policy, consumer::DeliverPolicy::New);
        assert_eq!(config.inactive_threshold, Duration::from_secs(5));
        assert_eq!(
            config
                .metadata
                .get(CONSUMER_METADATA_MANAGED_BY)
                .map(String::as_str),
            Some("platform")
        );
    }

    #[test]
    fn projector_consumer_name_removes_timestamp_punctuation() {
        assert_eq!(
            projector_consumer_name("1877448-2026-07-09T04:28:06.672734698Z"),
            "eventlog-projector-1877448-2026-07-09T042806672734698Z",
        );
    }

    #[test]
    fn eventlog_consumer_ownership_recognizes_current_and_legacy_names() {
        assert!(is_eventlog_projector_consumer("eventlog-projector-current"));
        assert!(is_eventlog_projector_consumer("event-log-projector-legacy"));
    }

    #[test]
    fn obsolete_watch_requires_the_legacy_name_and_config_shape() {
        let mut config = consumer::Config {
            durable_name: Some("eventlog-watch-session_1-7".to_string()),
            filter_subject: EVENT_SUBJECT_WILDCARD.to_string(),
            deliver_policy: consumer::DeliverPolicy::New,
            ack_policy: consumer::AckPolicy::Explicit,
            ..Default::default()
        };
        assert!(is_obsolete_eventlog_watch_consumer(
            "eventlog-watch-session_1-7",
            &config
        ));

        config.filter_subject = "events.v1.External.>".to_string();
        assert!(!is_obsolete_eventlog_watch_consumer(
            "eventlog-watch-session_1-7",
            &config
        ));

        config.filter_subject = EVENT_SUBJECT_WILDCARD.to_string();
        config
            .metadata
            .insert("owner".to_string(), "external".to_string());
        assert!(!is_obsolete_eventlog_watch_consumer(
            "eventlog-watch-session_1-7",
            &config
        ));
    }
}

impl From<EventLogStoreError> for ServerError {
    fn from(error: EventLogStoreError) -> Self {
        ServerError::Nats(error.to_string())
    }
}
