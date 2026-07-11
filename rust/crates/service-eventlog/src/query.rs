use serde_json::{json, Value};
use std::sync::Arc;
use trellis_rs::client::TrellisClient;

use crate::consumers::{inspect_consumer, query_consumers};
use crate::projector::EventLogRuntime;
use crate::storage::{EventLogFilter, EventLogStore, EventLogStoreError, EventTypeRef};

/// Event Log query adapter backed by SQLite and JetStream consumer info.
#[derive(Clone)]
pub struct EventLogQuery {
    store: EventLogStore,
    runtime: EventLogRuntime,
    auth_client: Arc<TrellisClient>,
}

/// Errors returned by Event Log RPC query handlers.
#[derive(Debug, thiserror::Error)]
pub enum EventLogQueryError {
    /// The requested event was not found.
    #[error("event not found")]
    EventNotFound,
    /// The requested consumer was not found.
    #[error("consumer not found: {0}")]
    ConsumerNotFound(String),
    /// Input validation failed.
    #[error("invalid {field}: {details}")]
    Validation {
        /// Field name.
        field: &'static str,
        /// Validation details.
        details: String,
    },
    /// SQLite projection query failed.
    #[error(transparent)]
    Store(#[from] EventLogStoreError),
    /// JetStream consumer query failed.
    #[error("consumer query failed: {0}")]
    Consumer(String),
}

impl EventLogQuery {
    /// Construct an Event Log query adapter.
    pub fn new(
        store: EventLogStore,
        runtime: EventLogRuntime,
        auth_client: Arc<TrellisClient>,
    ) -> Self {
        Self {
            store,
            runtime,
            auth_client,
        }
    }

    /// Run `EventLog.Query`.
    pub async fn query_events(&self, input: &Value) -> Result<Value, EventLogQueryError> {
        let filter = parse_event_filter(input)?;
        let (events, total) = self.store.query_events(&filter)?;
        Ok(json!({
            "events": events,
            "total": total,
            "offset": filter.offset,
            "limit": filter.limit,
        }))
    }

    /// Run `EventLog.Inspect`.
    pub async fn inspect_event(&self, input: &Value) -> Result<Value, EventLogQueryError> {
        let event_id = input.get("eventId").and_then(Value::as_str);
        let stream_sequence = input.get("streamSequence").and_then(Value::as_u64);
        if event_id.is_none() && stream_sequence.is_none() {
            return Err(EventLogQueryError::Validation {
                field: "eventId",
                details: "eventId or streamSequence is required".to_string(),
            });
        }
        self.store
            .inspect_event(event_id, stream_sequence)?
            .ok_or(EventLogQueryError::EventNotFound)
    }

    /// Run `EventLog.Metrics`.
    pub async fn metrics(&self, input: &Value) -> Result<Value, EventLogQueryError> {
        let window = input
            .get("window")
            .and_then(Value::as_str)
            .and_then(|window| {
                let (window_seconds, bucket_seconds) = window_config(window)?;
                Some((
                    window_since(window_seconds)?,
                    window_seconds,
                    bucket_seconds,
                ))
            });
        Ok(self.store.metrics(
            window
                .as_ref()
                .map(|(since, window_seconds, bucket_seconds)| {
                    (since.as_str(), *window_seconds, *bucket_seconds)
                }),
        )?)
    }

    /// Run `EventLog.Consumers.Query`.
    pub async fn query_consumers(&self, input: &Value) -> Result<Value, EventLogQueryError> {
        query_consumers(&self.runtime, &self.auth_client, input)
            .await
            .map_err(EventLogQueryError::Consumer)
    }

    /// Run `EventLog.Consumers.Inspect`.
    pub async fn inspect_consumer(&self, input: &Value) -> Result<Value, EventLogQueryError> {
        inspect_consumer(&self.runtime, &self.auth_client, input)
            .await
            .map_err(|error| {
                if error.contains("consumer not found") || error.contains("404") {
                    EventLogQueryError::ConsumerNotFound(
                        input
                            .get("consumerName")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                    )
                } else {
                    EventLogQueryError::Consumer(error)
                }
            })
    }
}

fn parse_event_filter(input: &Value) -> Result<EventLogFilter, EventLogQueryError> {
    let limit = input
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .clamp(1, 500);
    let sort = input.get("sort");
    Ok(EventLogFilter {
        search: input
            .get("search")
            .and_then(Value::as_str)
            .map(str::to_string),
        subject: input
            .get("subject")
            .and_then(Value::as_str)
            .map(str::to_string),
        owner_contract_id: input
            .get("ownerContractId")
            .and_then(Value::as_str)
            .map(str::to_string),
        owner_event_name: input
            .get("ownerEventName")
            .and_then(Value::as_str)
            .map(str::to_string),
        include_event_types: event_type_array(input, "includeEventTypes"),
        exclude_event_types: event_type_array(input, "excludeEventTypes"),
        publisher_deployment_id: input
            .get("publisherDeploymentId")
            .and_then(Value::as_str)
            .map(str::to_string),
        resolution: string_array(input, "resolution"),
        verification_status: string_array(input, "verificationStatus"),
        integrity_exception_only: input
            .get("integrityExceptionOnly")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        since: input
            .get("window")
            .and_then(Value::as_str)
            .and_then(|window| {
                window_config(window).and_then(|(seconds, _)| window_since(seconds))
            }),
        offset: input.get("offset").and_then(Value::as_u64).unwrap_or(0),
        limit,
        sort_field: sort
            .and_then(|value| value.get("field"))
            .and_then(Value::as_str)
            .unwrap_or("eventTime")
            .to_string(),
        sort_direction: sort
            .and_then(|value| value.get("direction"))
            .and_then(Value::as_str)
            .unwrap_or("desc")
            .to_string(),
    })
}

fn event_type_array(input: &Value, key: &str) -> Vec<EventTypeRef> {
    input
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            Some(EventTypeRef {
                owner_contract_id: item.get("ownerContractId")?.as_str()?.to_string(),
                owner_event_name: item.get("ownerEventName")?.as_str()?.to_string(),
            })
        })
        .collect()
}

fn string_array(input: &Value, key: &str) -> Vec<String> {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn window_config(window: &str) -> Option<(i64, i64)> {
    match window {
        "15m" => Some((15 * 60, 60)),
        "1h" => Some((60 * 60, 5 * 60)),
        "6h" => Some((6 * 60 * 60, 30 * 60)),
        "24h" => Some((24 * 60 * 60, 60 * 60)),
        "7d" => Some((7 * 24 * 60 * 60, 6 * 60 * 60)),
        _ => return None,
    }
}

fn window_since(seconds: i64) -> Option<String> {
    let since = time::OffsetDateTime::now_utc() - time::Duration::seconds(seconds);
    since
        .format(&time::format_description::well_known::Rfc3339)
        .ok()
}
