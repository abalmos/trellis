//! `EventLog.Watch` feed implementation.

use futures_util::{stream, Stream, StreamExt};
use serde_json::{json, Value};
use trellis_rs::service::{ConnectedServiceRuntime, ServerError};

use crate::contract::EventLogContract;
use crate::projector::{EventLogRuntime, EventMessageStream};
use crate::storage::now_timestamp_string;
use crate::wire::EventLogWatchFeed;

/// Register the `EventLog.Watch` feed on the Event Log service runtime.
pub fn register_eventlog_watch_feed(
    runtime: &mut ConnectedServiceRuntime<EventLogContract>,
    eventlog_runtime: EventLogRuntime,
) {
    runtime.register_feed::<EventLogWatchFeed, _, _>(move |_ctx, input| {
        watch_events(input, eventlog_runtime.clone())
    });
}

fn watch_events(
    _input: Value,
    eventlog_runtime: EventLogRuntime,
) -> impl Stream<Item = Result<Value, ServerError>> + Send + 'static {
    stream::unfold(WatchState::Init(eventlog_runtime), next_watch_frame)
}

enum WatchState {
    Init(EventLogRuntime),
    Open(EventMessageStream),
    Done,
}

async fn next_watch_frame(state: WatchState) -> Option<(Result<Value, ServerError>, WatchState)> {
    match state {
        WatchState::Init(eventlog_runtime) => {
            let messages = match eventlog_runtime.live_events().await {
                Ok(messages) => messages,
                Err(error) => {
                    return Some((
                        Err(ServerError::Nats(format!(
                            "failed to start EventLog.Watch: {error}"
                        ))),
                        WatchState::Done,
                    ));
                }
            };
            return Some((
                Ok(json!({
                    "kind": "ready",
                    "cursor": "now",
                    "serverTime": now_timestamp_string(),
                })),
                WatchState::Open(messages),
            ));
        }
        WatchState::Open(mut messages) => match messages.next().await {
            Some(Ok(message)) => {
                let _ = message.ack().await;
                return Some((
                    Ok(json!({
                        "kind": "eventQueryInvalidated",
                        "reason": "new-event",
                        "serverTime": now_timestamp_string(),
                    })),
                    WatchState::Open(messages),
                ));
            }
            Some(Err(error)) => {
                return Some((
                    Err(ServerError::Nats(format!("EventLog.Watch failed: {error}"))),
                    WatchState::Done,
                ));
            }
            None => return None,
        },
        WatchState::Done => None,
    }
}
