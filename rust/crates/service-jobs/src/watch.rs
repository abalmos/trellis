//! `Jobs.Watch` feed implementation.

use std::collections::HashMap;
use std::time::Duration;

use async_nats::jetstream::consumer::FromConsumer;
use async_nats::jetstream::{self, consumer};
use futures_util::{stream, stream::BoxStream, Stream, StreamExt};
use serde_json::json;
use trellis_rs::jobs::types::{JobEvent, JobState, JobTriggerKind};
use trellis_rs::sdk::jobs::feeds::JobsWatchFeedDescriptor;
use trellis_rs::sdk::jobs::types::{JobsWatchEvent, JobsWatchInput, JobsWatchInputQuery};
use trellis_rs::service::{ConnectedServiceRuntime, ServerError};

use crate::contract::JobsContract;

const JOBS_EVENTS_SUBJECT_WILDCARD: &str = "trellis.jobs.>";
const LIVE_WATCH_INACTIVE_THRESHOLD: Duration = Duration::from_secs(5);
const OBSOLETE_WATCH_INACTIVE_THRESHOLD: Duration = Duration::from_secs(5 * 60);

/// Register the `Jobs.Watch` feed on the Jobs service runtime.
pub fn register_jobs_watch_feed(
    runtime: &mut ConnectedServiceRuntime<JobsContract>,
    nats: async_nats::Client,
    jobs_stream: String,
) {
    runtime.register_feed::<JobsWatchFeedDescriptor, _, _>(move |_ctx, input| {
        watch_jobs(input, nats.clone(), jobs_stream.clone())
    });
}

fn watch_jobs(
    input: JobsWatchInput,
    nats: async_nats::Client,
    jobs_stream: String,
) -> impl Stream<Item = Result<JobsWatchEvent, ServerError>> + Send + 'static {
    let filter_subject = input
        .job_id
        .as_deref()
        .map(|job_id| format!("trellis.jobs.*.*.{job_id}.>"))
        .unwrap_or_else(|| JOBS_EVENTS_SUBJECT_WILDCARD.to_string());
    stream::unfold(
        WatchState::Init {
            nats,
            jobs_stream,
            filter_subject,
            input,
        },
        next_watch_frame,
    )
}

enum WatchState {
    Init {
        nats: async_nats::Client,
        jobs_stream: String,
        filter_subject: String,
        input: JobsWatchInput,
    },
    Open {
        messages: BoxStream<'static, Result<jetstream::Message, String>>,
        input: JobsWatchInput,
    },
    Done,
}

async fn next_watch_frame(
    state: WatchState,
) -> Option<(Result<JobsWatchEvent, ServerError>, WatchState)> {
    match state {
        WatchState::Init {
            nats,
            jobs_stream,
            filter_subject,
            input,
        } => {
            let messages = match live_watch_messages(&nats, &jobs_stream, &filter_subject).await {
                Ok(messages) => messages,
                Err(error) => {
                    return Some((
                        Err(ServerError::Nats(format!(
                            "failed to start Jobs.Watch: {error}"
                        ))),
                        WatchState::Done,
                    ));
                }
            };
            return Some((
                Ok(ready_frame(now_timestamp_string())),
                WatchState::Open { messages, input },
            ));
        }
        WatchState::Open {
            mut messages,
            input,
        } => loop {
            let message = match messages.next().await {
                Some(Ok(message)) => message,
                Some(Err(error)) => {
                    return Some((
                        Err(ServerError::Nats(format!("Jobs.Watch failed: {error}"))),
                        WatchState::Done,
                    ));
                }
                None => return None,
            };
            let event = serde_json::from_slice::<JobEvent>(&message.payload).ok();
            let _ = message.ack().await;
            let Some(event) = event else {
                continue;
            };
            if let Some(frame) = watch_frame_for_event(&input, &event) {
                return Some((Ok(frame), WatchState::Open { messages, input }));
            }
        },
        WatchState::Done => None,
    }
}

async fn live_watch_messages(
    nats: &async_nats::Client,
    stream_name: &str,
    filter_subject: &str,
) -> Result<BoxStream<'static, Result<jetstream::Message, String>>, String> {
    let jetstream = jetstream::new(nats.clone());
    let stream = jetstream
        .get_stream(stream_name)
        .await
        .map_err(|error| error.to_string())?;
    let consumer = stream
        .create_consumer(live_watch_consumer_config(filter_subject))
        .await
        .map_err(|error| error.to_string())?;
    consumer
        .messages()
        .await
        .map(|messages| {
            messages
                .map(|message| message.map_err(|error| error.to_string()))
                .boxed()
        })
        .map_err(|error| error.to_string())
}

fn live_watch_consumer_config(filter_subject: &str) -> consumer::pull::Config {
    consumer::pull::Config {
        filter_subject: filter_subject.to_string(),
        deliver_policy: consumer::DeliverPolicy::New,
        ack_policy: consumer::AckPolicy::Explicit,
        inactive_threshold: LIVE_WATCH_INACTIVE_THRESHOLD,
        metadata: platform_watch_metadata(),
        ..Default::default()
    }
}

fn platform_watch_metadata() -> HashMap<String, String> {
    HashMap::from([
        ("trellis.managed_by".to_string(), "platform".to_string()),
        (
            "trellis.contract_id".to_string(),
            "trellis.jobs@v1".to_string(),
        ),
        ("trellis.group".to_string(), "watch".to_string()),
    ])
}

pub(crate) async fn expire_obsolete_watch_consumers(
    nats: &async_nats::Client,
    stream_name: &str,
) -> Result<usize, String> {
    let jetstream = jetstream::new(nats.clone());
    let stream = jetstream
        .get_stream(stream_name)
        .await
        .map_err(|error| error.to_string())?;
    let mut consumers = stream.consumers();
    let mut configs = Vec::new();
    while let Some(info) = consumers.next().await {
        let info = info.map_err(|error| error.to_string())?;
        if is_obsolete_jobs_watch_consumer(&info.name, &info.config) {
            let mut config = consumer::pull::Config::try_from_consumer_config(info.config)
                .map_err(|error| error.to_string())?;
            config.inactive_threshold = OBSOLETE_WATCH_INACTIVE_THRESHOLD;
            config.metadata.extend(platform_watch_metadata());
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

fn is_obsolete_jobs_watch_consumer(name: &str, config: &consumer::Config) -> bool {
    let Some((seed, counter)) = name
        .strip_prefix("jobs-watch-")
        .and_then(|suffix| suffix.rsplit_once('-'))
    else {
        return false;
    };
    let watch_filter = config.filter_subject == JOBS_EVENTS_SUBJECT_WILDCARD
        || (config.filter_subject.starts_with("trellis.jobs.*.*.")
            && config.filter_subject.ends_with(".>"));
    !seed.is_empty()
        && seed.len() <= 48
        && seed
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        && counter.parse::<u64>().is_ok()
        && config.durable_name.as_deref() == Some(name)
        && watch_filter
        && config.filter_subjects.is_empty()
        && config.deliver_policy == consumer::DeliverPolicy::All
        && config.ack_policy == consumer::AckPolicy::Explicit
        && config.metadata.keys().all(|key| key.starts_with("_nats."))
}

fn ready_frame(timestamp: String) -> JobsWatchEvent {
    JobsWatchEvent(json!({
        "kind": "ready",
        "timestamp": timestamp,
    }))
}

fn watch_frame_for_event(input: &JobsWatchInput, event: &JobEvent) -> Option<JobsWatchEvent> {
    if input.job_id.as_deref() == Some(event.job_id.as_str()) {
        return Some(JobsWatchEvent(json!({
            "kind": "jobInspectChanged",
            "id": event.job_id,
            "timestamp": event.timestamp,
        })));
    }

    input.query.as_ref().and_then(|query| {
        match query_invalidation_reason(query, event) {
            QueryInvalidation::No => None,
            QueryInvalidation::Matched => Some("matched-job-changed"),
            QueryInvalidation::Unknown => Some("unknown-match"),
        }
        .map(|reason| {
            JobsWatchEvent(json!({
                "kind": "queryInvalidated",
                "reason": reason,
                "timestamp": event.timestamp,
            }))
        })
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueryInvalidation {
    No,
    Matched,
    Unknown,
}

fn query_invalidation_reason(query: &JobsWatchInputQuery, event: &JobEvent) -> QueryInvalidation {
    if query
        .service
        .as_ref()
        .is_some_and(|service| service != &event.service)
        || query
            .r#type
            .as_ref()
            .is_some_and(|job_type| job_type != &event.job_type)
        || query.state.as_ref().is_some_and(|states| {
            !states
                .iter()
                .any(|state| state.as_str() == job_state_token(event.state))
        })
        || query.trigger.as_ref().is_some_and(|trigger| {
            event.trigger.as_ref().is_some_and(|event_trigger| {
                trigger.as_str() != trigger_kind_token(event_trigger.kind)
            })
        })
    {
        return QueryInvalidation::No;
    }

    if query.queue_key.as_ref().is_some_and(|queue_key| {
        event
            .concurrency
            .as_ref()
            .map(|concurrency| queue_key != &concurrency.key)
            .unwrap_or(false)
    }) {
        return QueryInvalidation::No;
    }

    if query
        .search
        .as_deref()
        .is_some_and(|search| !search.trim().is_empty())
        || query.runtime_band.is_some()
        || (query.queue_key.is_some() && event.concurrency.is_none())
        || (query.trigger.is_some() && event.trigger.is_none())
    {
        QueryInvalidation::Unknown
    } else {
        QueryInvalidation::Matched
    }
}

fn now_timestamp_string() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn job_state_token(state: JobState) -> &'static str {
    match state {
        JobState::Pending => "pending",
        JobState::Active => "active",
        JobState::Retry => "retry",
        JobState::Completed => "completed",
        JobState::Failed => "failed",
        JobState::Cancelled => "cancelled",
        JobState::Expired => "expired",
        JobState::Skipped => "skipped",
        JobState::Stale => "stale",
        JobState::Dead => "dead",
        JobState::Dismissed => "dismissed",
    }
}

fn trigger_kind_token(kind: JobTriggerKind) -> &'static str {
    match kind {
        JobTriggerKind::Schedule => "schedule",
        JobTriggerKind::Operation => "operation",
        JobTriggerKind::Rpc => "rpc",
        JobTriggerKind::Event => "event",
        JobTriggerKind::ManualReplay => "manualReplay",
        JobTriggerKind::ServiceCode => "serviceCode",
        JobTriggerKind::ParentJob => "parentJob",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use trellis_rs::jobs::events::created_event;
    use trellis_rs::jobs::types::{JobContext, JobState};

    use super::*;

    #[test]
    fn live_watch_consumer_is_ephemeral_and_new_only() {
        let config = live_watch_consumer_config("trellis.jobs.>");

        assert!(config.durable_name.is_none());
        assert_eq!(config.deliver_policy, consumer::DeliverPolicy::New);
        assert_eq!(config.inactive_threshold, Duration::from_secs(5));
        assert_eq!(
            config
                .metadata
                .get("trellis.managed_by")
                .map(String::as_str),
            Some("platform")
        );
    }

    #[test]
    fn obsolete_watch_requires_the_legacy_name_and_config_shape() {
        let mut config = consumer::Config {
            durable_name: Some("jobs-watch-session_1-7".to_string()),
            filter_subject: JOBS_EVENTS_SUBJECT_WILDCARD.to_string(),
            ack_policy: consumer::AckPolicy::Explicit,
            ..Default::default()
        };
        assert!(is_obsolete_jobs_watch_consumer(
            "jobs-watch-session_1-7",
            &config
        ));

        config.filter_subject = "external.jobs.>".to_string();
        assert!(!is_obsolete_jobs_watch_consumer(
            "jobs-watch-session_1-7",
            &config
        ));

        config.filter_subject = JOBS_EVENTS_SUBJECT_WILDCARD.to_string();
        config
            .metadata
            .insert("owner".to_string(), "external".to_string());
        assert!(!is_obsolete_jobs_watch_consumer(
            "jobs-watch-session_1-7",
            &config
        ));
    }

    #[test]
    fn query_invalidation_matches_scalar_filters() {
        let event = created_event(
            "documents",
            "document-process",
            "job-1",
            &context(),
            json!({ "documentId": "doc-1" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );
        let query = JobsWatchInputQuery {
            group_by: None,
            limit: 50,
            offset: None,
            queue_key: None,
            runtime_band: None,
            search: None,
            service: Some("documents".to_string()),
            sort: None,
            state: Some(vec!["pending".to_string()]),
            trigger: None,
            r#type: Some("document-process".to_string()),
            window: None,
        };

        assert_eq!(
            query_invalidation_reason(&query, &event),
            QueryInvalidation::Matched
        );
    }

    #[test]
    fn query_invalidation_rejects_scalar_mismatches() {
        let mut event = created_event(
            "documents",
            "document-process",
            "job-1",
            &context(),
            json!({ "documentId": "doc-1" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );
        event.state = JobState::Failed;
        let query = JobsWatchInputQuery {
            group_by: None,
            limit: 50,
            offset: None,
            queue_key: None,
            runtime_band: None,
            search: None,
            service: Some("documents".to_string()),
            sort: None,
            state: Some(vec!["pending".to_string()]),
            trigger: None,
            r#type: None,
            window: None,
        };

        assert_eq!(
            query_invalidation_reason(&query, &event),
            QueryInvalidation::No
        );
    }

    #[test]
    fn query_invalidation_is_unknown_for_text_search() {
        let event = created_event(
            "documents",
            "document-process",
            "job-1",
            &context(),
            json!({ "documentId": "doc-1" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );
        let query = JobsWatchInputQuery {
            group_by: None,
            limit: 50,
            offset: None,
            queue_key: None,
            runtime_band: None,
            search: Some("doc-1".to_string()),
            service: Some("documents".to_string()),
            sort: None,
            state: None,
            trigger: None,
            r#type: None,
            window: None,
        };

        assert_eq!(
            query_invalidation_reason(&query, &event),
            QueryInvalidation::Unknown
        );
    }

    #[test]
    fn query_invalidation_is_unknown_for_missing_trigger_data() {
        let event = created_event(
            "documents",
            "document-process",
            "job-1",
            &context(),
            json!({ "documentId": "doc-1" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );
        let query = JobsWatchInputQuery {
            group_by: None,
            limit: 50,
            offset: None,
            queue_key: None,
            runtime_band: None,
            search: None,
            service: Some("documents".to_string()),
            sort: None,
            state: None,
            trigger: Some("schedule".to_string()),
            r#type: None,
            window: None,
        };

        assert_eq!(
            query_invalidation_reason(&query, &event),
            QueryInvalidation::Unknown
        );
    }

    fn context() -> JobContext {
        JobContext {
            request_id: "request-job-1".to_string(),
            trace_id: "0123456789abcdef0123456789abcdef".to_string(),
            traceparent: "00-0123456789abcdef0123456789abcdef-0123456789abcdef-01".to_string(),
            tracestate: None,
        }
    }
}
