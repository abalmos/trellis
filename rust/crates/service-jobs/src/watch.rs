//! `Jobs.Watch` feed implementation.

use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::{stream, Stream, StreamExt};
use serde_json::json;
use trellis_rs::jobs::types::{JobEvent, JobState, JobTriggerKind};
use trellis_rs::jobs::JobsRuntime;
use trellis_rs::sdk::jobs::feeds::JobsWatchFeedDescriptor;
use trellis_rs::sdk::jobs::types::{JobsWatchEvent, JobsWatchInput, JobsWatchInputQuery};
use trellis_rs::service::{ConnectedServiceRuntime, ServerError, ServiceHandlerContext};

use crate::contract::JobsContract;

const JOBS_EVENTS_SUBJECT_WILDCARD: &str = "trellis.jobs.>";
static WATCH_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Register the `Jobs.Watch` feed on the Jobs service runtime.
pub fn register_jobs_watch_feed(
    runtime: &mut ConnectedServiceRuntime<JobsContract>,
    jobs_runtime: JobsRuntime,
    jobs_stream: String,
) {
    runtime.register_feed::<JobsWatchFeedDescriptor, _, _>(move |ctx, input| {
        watch_jobs(ctx, input, jobs_runtime.clone(), jobs_stream.clone())
    });
}

fn watch_jobs(
    ctx: ServiceHandlerContext,
    input: JobsWatchInput,
    jobs_runtime: JobsRuntime,
    jobs_stream: String,
) -> impl Stream<Item = Result<JobsWatchEvent, ServerError>> + Send + 'static {
    let request = ctx.into_request_context();
    let filter_subject = input
        .job_id
        .as_deref()
        .map(|job_id| format!("trellis.jobs.*.*.{job_id}.>"))
        .unwrap_or_else(|| JOBS_EVENTS_SUBJECT_WILDCARD.to_string());
    let consumer_name = watch_consumer_name(
        request
            .request_id
            .as_deref()
            .or(request.session_key.as_deref())
            .unwrap_or("anonymous"),
    );
    let subscribed_at = now_timestamp_string();

    stream::once({
        let subscribed_at = subscribed_at.clone();
        async move { Ok(ready_frame(subscribed_at)) }
    })
    .chain(stream::unfold(
        WatchState::Init {
            jobs_runtime,
            jobs_stream,
            consumer_name,
            filter_subject,
            subscribed_at,
            input,
        },
        next_watch_frame,
    ))
}

enum WatchState {
    Init {
        jobs_runtime: JobsRuntime,
        jobs_stream: String,
        consumer_name: String,
        filter_subject: String,
        subscribed_at: String,
        input: JobsWatchInput,
    },
    Open {
        messages: trellis_rs::jobs::JobsRuntimeMessageStream,
        subscribed_at: String,
        input: JobsWatchInput,
    },
    Done,
}

async fn next_watch_frame(
    mut state: WatchState,
) -> Option<(Result<JobsWatchEvent, ServerError>, WatchState)> {
    loop {
        match state {
            WatchState::Init {
                jobs_runtime,
                jobs_stream,
                consumer_name,
                filter_subject,
                subscribed_at,
                input,
            } => {
                let messages = match jobs_runtime
                    .filtered_messages(&jobs_stream, &consumer_name, &filter_subject)
                    .await
                {
                    Ok(messages) => messages,
                    Err(error) => {
                        return Some((
                            Err(ServerError::Nats(format!(
                                "failed to start Jobs.Watch consumer '{consumer_name}' on stream '{jobs_stream}': {error}"
                            ))),
                            WatchState::Done,
                        ));
                    }
                };
                state = WatchState::Open {
                    messages,
                    subscribed_at,
                    input,
                };
            }
            WatchState::Open {
                mut messages,
                subscribed_at,
                input,
            } => loop {
                let message = match messages.next().await {
                    Some(Ok(message)) => message,
                    Some(Err(error)) => {
                        return Some((
                            Err(ServerError::Nats(format!(
                                "Jobs.Watch failed to pull from stream: {error}"
                            ))),
                            WatchState::Done,
                        ));
                    }
                    None => return None,
                };
                let event = serde_json::from_slice::<JobEvent>(message.payload()).ok();
                let _ = message.ack().await;
                let Some(event) = event else {
                    continue;
                };
                if event.timestamp < subscribed_at {
                    continue;
                }
                if let Some(frame) = watch_frame_for_event(&input, &event) {
                    return Some((
                        Ok(frame),
                        WatchState::Open {
                            messages,
                            subscribed_at,
                            input,
                        },
                    ));
                }
            },
            WatchState::Done => return None,
        }
    }
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

fn watch_consumer_name(seed: &str) -> String {
    let suffix = WATCH_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("jobs-watch-{}-{suffix}", sanitize_consumer_token(seed))
}

fn sanitize_consumer_token(value: &str) -> String {
    let token = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .take(48)
        .collect::<String>();
    if token.is_empty() {
        "anonymous".to_string()
    } else {
        token
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
