//! Narrow Event Log transport owned by the Trellis runtime.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream::consumer::FromConsumer;
use async_nats::jetstream::{self, consumer};
use futures_util::stream::BoxStream;
use futures_util::StreamExt;

use crate::client::TrellisClient;

const EVENT_STREAM: &str = "trellis";
const EVENT_SUBJECT_WILDCARD: &str = "events.v1.>";

/// Event Log-specific transport over the Trellis event stream.
#[derive(Clone)]
pub struct EventLogRuntime {
    client: Arc<TrellisClient>,
}

impl EventLogRuntime {
    /// Create the Event Log transport from an authenticated Trellis session.
    #[doc(hidden)]
    pub(crate) fn from_client(client: Arc<TrellisClient>) -> Self {
        Self { client }
    }

    /// Open a new-only ephemeral event stream consumer.
    pub async fn live_events(&self) -> Result<EventLogMessageStream, String> {
        let stream = jetstream::new(self.client.nats().clone())
            .get_stream(EVENT_STREAM)
            .await
            .map_err(|error| error.to_string())?;
        let consumer = stream
            .create_consumer(consumer::pull::Config {
                filter_subject: EVENT_SUBJECT_WILDCARD.to_string(),
                deliver_policy: consumer::DeliverPolicy::New,
                ack_policy: consumer::AckPolicy::Explicit,
                inactive_threshold: Duration::from_secs(5),
                metadata: platform_consumer_metadata("watch"),
                ..Default::default()
            })
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

    /// Mark shape-matched legacy Event Log watch durables for expiry.
    pub async fn expire_obsolete_watch_consumers(&self) -> Result<usize, String> {
        let stream = jetstream::new(self.client.nats().clone())
            .get_stream(EVENT_STREAM)
            .await
            .map_err(|error| error.to_string())?;
        let mut consumers = stream.consumers();
        let mut configs = Vec::new();
        while let Some(info) = consumers.next().await {
            let info = info.map_err(|error| error.to_string())?;
            if obsolete_eventlog_watch_consumer(&info.name, &info.config) {
                let mut config = consumer::pull::Config::try_from_consumer_config(info.config)
                    .map_err(|error| error.to_string())?;
                config.inactive_threshold = Duration::from_secs(5 * 60);
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

    /// Open the durable Event Log projector consumer.
    pub async fn event_consumer(
        &self,
        consumer_name: &str,
        replay_all: bool,
    ) -> Result<consumer::Consumer<consumer::pull::Config>, String> {
        let stream = jetstream::new(self.client.nats().clone())
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

    /// Return Event Log stream consumer metadata.
    pub async fn consumers(&self) -> Result<Vec<consumer::Info>, String> {
        let stream = jetstream::new(self.client.nats().clone())
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

    /// Return metadata for one Event Log stream consumer.
    pub async fn consumer(&self, name: &str) -> Result<consumer::Info, String> {
        let stream = jetstream::new(self.client.nats().clone())
            .get_stream(EVENT_STREAM)
            .await
            .map_err(|error| error.to_string())?;
        stream
            .consumer_info(name)
            .await
            .map_err(|error| error.to_string())
    }

    /// Validate one delivered event proof through Auth.
    pub async fn validate_event(
        &self,
        request: &crate::sdk::auth::types::AuthEventsValidateRequest,
    ) -> Result<
        crate::sdk::auth::types::AuthEventsValidateResponse,
        crate::client::TrellisClientError,
    > {
        self.client
            .call::<crate::sdk::auth::rpc::AuthEventsValidateRpc>(request)
            .await
    }

    /// Return one page of Auth-attributed durable event consumers.
    pub async fn event_consumers(
        &self,
        request: &crate::sdk::auth::types::AuthEventConsumersListRequest,
    ) -> Result<
        crate::sdk::auth::types::AuthEventConsumersListResponse,
        crate::client::TrellisClientError,
    > {
        self.client
            .call::<crate::sdk::auth::rpc::AuthEventConsumersListRpc>(request)
            .await
    }
}

/// Event Log stream messages retained behind the domain runtime.
pub type EventLogMessageStream = BoxStream<
    'static,
    Result<async_nats::jetstream::Message, Box<dyn std::error::Error + Send + Sync>>,
>;

fn platform_consumer_metadata(group: &str) -> HashMap<String, String> {
    HashMap::from([
        ("trellis.managed_by".to_string(), "platform".to_string()),
        (
            "trellis.contract_id".to_string(),
            "trellis.eventlog@v1".to_string(),
        ),
        ("trellis.group".to_string(), group.to_string()),
    ])
}

fn obsolete_eventlog_watch_consumer(name: &str, config: &consumer::Config) -> bool {
    let Some((seed, counter)) = name
        .strip_prefix("eventlog-watch-")
        .or_else(|| name.strip_prefix("event-log-watch-"))
        .and_then(|suffix| suffix.rsplit_once('-'))
    else {
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
        && config.deliver_policy == consumer::DeliverPolicy::All
        && config.ack_policy == consumer::AckPolicy::Explicit
        && config.metadata.keys().all(|key| key.starts_with("_nats."))
}
