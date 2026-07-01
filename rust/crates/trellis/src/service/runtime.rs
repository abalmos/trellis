use super::request_loop::{run_nats_request_loop, RequestHandler};
use super::ServerError;

/// Queue-subscribe to one RPC subject for service request handling.
///
/// Uses the subject as the queue group so multiple service instances share
/// requests instead of each handling every request independently.
pub(crate) async fn subscribe_subject(
    client: &async_nats::Client,
    subject: &str,
) -> Result<async_nats::Subscriber, ServerError> {
    client
        .queue_subscribe(subject.to_string(), subject.to_string())
        .await
        .map_err(|error| {
            ServerError::Nats(format!(
                "failed to subscribe to subject '{subject}': {error}"
            ))
        })
}

/// Run one service request loop bound to a single subject.
pub(crate) async fn run_single_subject_service<H>(
    client: async_nats::Client,
    subject: &str,
    handler: H,
) -> Result<(), ServerError>
where
    H: RequestHandler,
{
    let subscriber = subscribe_subject(&client, subject).await?;
    run_nats_request_loop(client, subscriber, handler).await
}

/// Run one service request loop bound to multiple exact subjects.
pub(crate) async fn run_multi_subject_service<H>(
    client: async_nats::Client,
    subjects: &[&str],
    handler: H,
) -> Result<(), ServerError>
where
    H: RequestHandler,
{
    let mut subscribers = Vec::with_capacity(subjects.len());
    for subject in subjects {
        subscribers.push(subscribe_subject(&client, subject).await?);
    }

    run_nats_request_loop(
        client,
        futures_util::stream::select_all(subscribers),
        handler,
    )
    .await
}
