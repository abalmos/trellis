//! `EventLog.Watch` feed implementation.

use futures_util::{stream, Stream, StreamExt};
use serde_json::json;
use trellis_rs::service::{Router, ServerError};
use trellis_runtime_apis::apis::trellis_events_v1::{feeds, feeds::Watch};

use crate::projector::{EventLogRuntime, EventMessageStream};
use crate::storage::now_timestamp_string;
use crate::wire::generated_output;

/// Register the `EventLog.Watch` feed on a built-in Event Log router.
pub fn register_eventlog_watch_feed(router: &mut Router, eventlog_runtime: EventLogRuntime) {
    router.register_feed::<Watch, _, _>(move |_ctx, input| {
        watch_events(input, eventlog_runtime.clone())
    });
}

fn watch_events(
    _input: feeds::WatchInput,
    eventlog_runtime: EventLogRuntime,
) -> impl Stream<Item = Result<feeds::WatchEvent, ServerError>> + Send + 'static {
    stream::unfold(WatchState::Init(eventlog_runtime), next_watch_frame)
}

enum WatchState {
    Init(EventLogRuntime),
    Open(EventMessageStream),
    Done,
}

async fn next_watch_frame(
    state: WatchState,
) -> Option<(Result<feeds::WatchEvent, ServerError>, WatchState)> {
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
            Some((
                generated_output(
                    json!({
                        "kind": "ready",
                        "cursor": "now",
                        "serverTime": now_timestamp_string(),
                    }),
                    &[],
                ),
                WatchState::Open(messages),
            ))
        }
        WatchState::Open(mut messages) => match messages.next().await {
            Some(Ok(message)) => {
                let _ = message.ack().await;
                Some((
                    generated_output(
                        json!({
                            "kind": "eventQueryInvalidated",
                            "reason": "new-event",
                            "serverTime": now_timestamp_string(),
                        }),
                        &[],
                    ),
                    WatchState::Open(messages),
                ))
            }
            Some(Err(error)) => Some((
                Err(ServerError::Nats(format!("EventLog.Watch failed: {error}"))),
                WatchState::Done,
            )),
            None => None,
        },
        WatchState::Done => None,
    }
}
