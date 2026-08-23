//! Built-in Event Log subsystem.

use std::sync::Arc;

use trellis_eventlog_runtime::{
    start_eventlog_projector, EventLogQuery, EventLogStore, VerifiedEventPublisher,
};
use trellis_rs::service::{
    internal::run_builtin_authenticated_router, RequestValidator, ServerError,
};

use crate::shutdown::StopHandle;
use crate::supervisor::{RuntimeContext, RuntimeError, SubsystemHandle};
use crate::{StorageBackend, SubsystemName};

const EVENTLOG_SUBJECTS: &[&str] = &["rpc.v1.EventLog.>", "feed.v1.EventLog.>"];
const EVENTLOG_API_ID: &str = "trellis.eventlog@v1";

fn runtime_error(error: ServerError) -> RuntimeError {
    RuntimeError::Nats(error.to_string())
}

pub(crate) async fn start(context: &RuntimeContext) -> Result<SubsystemHandle, RuntimeError> {
    let owner = context.owner(crate::ownership::OwnerGroup::Eventlog)?;
    let stop = StopHandle::new();
    let task_stop = stop.clone();
    let mut validator_join =
        crate::platform::auth::verifier::ensure_read_only(context, task_stop.clone()).await?;
    let StorageBackend::Sqlite(storage) = context
        .config
        .eventlog_storage_backend()
        .map_err(RuntimeError::Config)?;
    let store = EventLogStore::open(&storage.path)
        .map_err(|error| RuntimeError::Nats(format!("failed to open Event Log SQLite: {error}")))?;
    let eventlog_runtime =
        trellis_rs::service::EventLogRuntime::from_nats(context.trellis_nats.clone());
    match eventlog_runtime.expire_obsolete_watch_consumers().await {
        Ok(count) if count > 0 => {
            tracing::info!(
                count,
                "scheduled obsolete EventLog.Watch consumers for expiry"
            );
        }
        Ok(_) => {}
        Err(error) => tracing::warn!(%error, "failed to expire obsolete EventLog.Watch consumers"),
    }
    let query = EventLogQuery::new(store.clone(), eventlog_runtime.clone());
    let mut router = trellis_eventlog_runtime::build_router_with_query(query);
    trellis_eventlog_runtime::register_eventlog_watch_feed(&mut router, eventlog_runtime.clone());
    let verifier = context.platform_verifier.get().cloned().ok_or_else(|| {
        RuntimeError::Platform("local authorization verifier is not ready".to_owned())
    })?;
    let validator: Arc<dyn RequestValidator> = Arc::new(verifier.clone());
    let event_verifier = Arc::new(
        move |input: trellis_eventlog_runtime::EventAuthorizationInput| {
            let verifier = verifier.clone();
            Box::pin(async move {
                verifier
                    .verify_event(
                        crate::platform::auth::verifier::RuntimeAuthorizationEventVerificationInput {
                            subject: &input.subject,
                            payload: &input.payload,
                            session_key: &input.session_key,
                            proof: &input.proof,
                            authorization_context: &input.authorization_context,
                            event_id: &input.event_id,
                            event_time: &input.event_time,
                        },
                    )
                    .await
                    .map(|publisher| VerifiedEventPublisher {
                        kind: publisher.kind,
                        deployment_id: publisher.deployment_id,
                        instance_id: publisher.instance_id,
                        participant_id: publisher.participant_id,
                        participant_digest: publisher.participant_digest,
                        session_id: publisher.session_id,
                    })
            })
                as std::pin::Pin<
                    Box<
                        dyn std::future::Future<
                                Output = Result<
                                    VerifiedEventPublisher,
                                    trellis_rs::service::EventVerificationFailure,
                                >,
                            > + Send,
                    >,
                >
        },
    );
    let mut projector = start_eventlog_projector(eventlog_runtime, store, event_verifier)
        .await
        .map_err(runtime_error)?;
    let nats = context.trellis_nats.clone();
    let join = tokio::spawn(async move {
        let _owner = owner;
        let api_loop = run_builtin_authenticated_router(
            nats,
            EVENTLOG_API_ID,
            EVENTLOG_SUBJECTS,
            router,
            validator,
        );
        tokio::pin!(api_loop);
        let result = {
            let validator_exit = async {
                match validator_join.as_mut() {
                    Some(join) => match join.await {
                        Ok(Ok(())) => Err(RuntimeError::Platform(
                            "authorization validator cache exited unexpectedly".to_owned(),
                        )),
                        Ok(Err(error)) => Err(error),
                        Err(error) => Err(RuntimeError::Platform(format!(
                            "authorization validator cache task failed: {error}"
                        ))),
                    },
                    None => std::future::pending().await,
                }
            };
            tokio::pin!(validator_exit);
            tokio::select! {
                biased;
                () = task_stop.stopped() => Ok(()),
                result = &mut api_loop => result.map_err(runtime_error),
                result = projector.wait() => {
                    projector.discard_completed();
                    match result {
                        Ok(()) => Err(RuntimeError::Nats(
                            "event log projector loop exited unexpectedly".to_string(),
                        )),
                        Err(error) => Err(runtime_error(error)),
                    }
                },
                result = &mut validator_exit => result,
            }
        };
        task_stop.stop();
        projector.stop().await;
        if let Some(join) = validator_join {
            let _ = join.await;
        }
        result
    });

    Ok(SubsystemHandle {
        name: SubsystemName::Eventlog,
        stop,
        join,
    })
}
