use std::pin::Pin;

use async_nats::jetstream::{self, consumer};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};

use crate::client::TrellisClient;
use crate::jobs::publisher::JobEventHeaders;
use crate::jobs::types::JobEvent;

/// Jobs subsystem runtime transport facade.
///
/// This keeps NATS and JetStream details inside the Trellis library while Jobs
/// subsystem services work with Jobs-domain operations.
#[derive(Debug, Clone)]
pub struct JobsRuntime {
    nats: async_nats::Client,
}

impl JobsRuntime {
    /// Create a Jobs runtime facade from a connected Trellis client.
    pub fn from_client(client: &TrellisClient) -> Self {
        Self {
            nats: client.nats().clone(),
        }
    }

    /// Publish one encoded job lifecycle event to its subject.
    pub async fn publish_event(&self, subject: String, event: &JobEvent) -> Result<(), String> {
        let payload = serde_json::to_vec(event).map_err(|error| error.to_string())?;
        self.publish_event_payload(subject, JobEventHeaders::from(&event.context), payload)
            .await
    }

    /// Publish an already encoded job lifecycle event payload.
    pub async fn publish_event_payload(
        &self,
        subject: String,
        headers: JobEventHeaders,
        payload: Vec<u8>,
    ) -> Result<(), String> {
        let mut nats_headers = async_nats::HeaderMap::new();
        nats_headers.insert("request-id", headers.request_id.as_str());
        nats_headers.insert("traceparent", headers.traceparent.as_str());
        if let Some(tracestate) = headers.tracestate.as_deref() {
            nats_headers.insert("tracestate", tracestate);
        }
        self.nats
            .publish_with_headers(subject, nats_headers, payload.into())
            .await
            .map_err(|error| error.to_string())
    }

    /// Open a pull consumer over a Jobs-owned stream/filter pair.
    pub async fn filtered_messages(
        &self,
        stream_name: &str,
        consumer_name: &str,
        filter_subject: &str,
    ) -> Result<JobsRuntimeMessageStream, String> {
        let jetstream = jetstream::new(self.nats.clone());
        let stream = jetstream
            .get_stream(stream_name)
            .await
            .map_err(|error| error.to_string())?;
        let consumer = stream
            .get_or_create_consumer(
                consumer_name,
                consumer::pull::Config {
                    durable_name: Some(consumer_name.to_string()),
                    filter_subject: filter_subject.to_string(),
                    ack_policy: consumer::AckPolicy::Explicit,
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| error.to_string())?;
        let messages = consumer
            .messages()
            .await
            .map_err(|error| error.to_string())?;
        Ok(Box::pin(messages.map(|message| {
            message
                .map(JobsRuntimeMessage::new)
                .map_err(|error| error.to_string())
        })))
    }

    /// Read a raw payload by stream sequence.
    pub async fn raw_payload(&self, stream_name: &str, sequence: u64) -> Result<Bytes, String> {
        let jetstream = jetstream::new(self.nats.clone());
        let stream = jetstream
            .get_stream(stream_name)
            .await
            .map_err(|error| error.to_string())?;
        stream
            .get_raw_message(sequence)
            .await
            .map(|message| message.payload)
            .map_err(|error| error.to_string())
    }
}

/// Stream of Jobs runtime messages.
pub type JobsRuntimeMessageStream =
    Pin<Box<dyn Stream<Item = Result<JobsRuntimeMessage, String>> + Send>>;

/// Message delivered by a Jobs runtime stream consumer.
pub struct JobsRuntimeMessage {
    inner: async_nats::jetstream::Message,
}

impl JobsRuntimeMessage {
    fn new(inner: async_nats::jetstream::Message) -> Self {
        Self { inner }
    }

    /// Return the message subject.
    pub fn subject(&self) -> &str {
        self.inner.subject.as_ref()
    }

    /// Return the raw message payload.
    pub fn payload(&self) -> &[u8] {
        &self.inner.payload
    }

    /// Acknowledge successful message handling.
    pub async fn ack(&self) -> Result<(), String> {
        self.inner.ack().await.map_err(|error| error.to_string())
    }
}
