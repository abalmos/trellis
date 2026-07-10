//! `EventLog.Watch` feed implementation.

use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::{stream, Stream, StreamExt};
use serde_json::{json, Value};
use trellis_rs::service::{ConnectedServiceRuntime, ServerError, ServiceHandlerContext};

use crate::contract::EventLogContract;
use crate::projector::{EventLogRuntime, EventMessageStream};
use crate::storage::now_timestamp_string;
use crate::wire::EventLogWatchFeed;

static WATCH_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Register the `EventLog.Watch` feed on the Event Log service runtime.
pub fn register_eventlog_watch_feed(
    runtime: &mut ConnectedServiceRuntime<EventLogContract>,
    eventlog_runtime: EventLogRuntime,
) {
    runtime.register_feed::<EventLogWatchFeed, _, _>(move |ctx, input| {
        watch_events(ctx, input, eventlog_runtime.clone())
    });
}

fn watch_events(
    ctx: ServiceHandlerContext,
    _input: Value,
    eventlog_runtime: EventLogRuntime,
) -> impl Stream<Item = Result<Value, ServerError>> + Send + 'static {
    let request = ctx.into_request_context();
    let consumer_name = watch_consumer_name(
        request
            .request_id
            .as_deref()
            .or(request.session_key.as_deref())
            .unwrap_or("anonymous"),
    );
    stream::once(async move {
        Ok(json!({
            "kind": "ready",
            "cursor": "now",
            "serverTime": now_timestamp_string(),
        }))
    })
    .chain(stream::unfold(
        WatchState::Init {
            eventlog_runtime,
            consumer_name,
        },
        next_watch_frame,
    ))
}

enum WatchState {
    Init {
        eventlog_runtime: EventLogRuntime,
        consumer_name: String,
    },
    Open {
        messages: EventMessageStream,
    },
    Done,
}

async fn next_watch_frame(
    mut state: WatchState,
) -> Option<(Result<Value, ServerError>, WatchState)> {
    loop {
        match state {
            WatchState::Init {
                eventlog_runtime,
                consumer_name,
            } => {
                let messages = match eventlog_runtime.event_messages(&consumer_name, false).await {
                    Ok(messages) => messages,
                    Err(error) => {
                        return Some((
                            Err(ServerError::Nats(format!(
                                "failed to start EventLog.Watch consumer '{consumer_name}': {error}"
                            ))),
                            WatchState::Done,
                        ));
                    }
                };
                state = WatchState::Open { messages };
            }
            WatchState::Open { mut messages } => match messages.next().await {
                Some(Ok(message)) => {
                    let _ = message.ack().await;
                    return Some((
                        Ok(json!({
                            "kind": "eventQueryInvalidated",
                            "reason": "new-event",
                            "serverTime": now_timestamp_string(),
                        })),
                        WatchState::Open { messages },
                    ));
                }
                Some(Err(error)) => {
                    return Some((
                        Err(ServerError::Nats(format!(
                            "EventLog.Watch failed to pull from stream: {error}"
                        ))),
                        WatchState::Done,
                    ));
                }
                None => return None,
            },
            WatchState::Done => return None,
        }
    }
}

fn watch_consumer_name(seed: &str) -> String {
    let suffix = WATCH_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("eventlog-watch-{}-{suffix}", sanitize_consumer_token(seed))
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
