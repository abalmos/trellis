//! Service bootstrap helpers for the Event Log service.

use std::path::Path;

use trellis_rs::generated::TrellisClientError;
use trellis_rs::service::{
    ConnectedServiceRuntime, ServerError, ServiceConnectOptions, ServiceRuntimeError,
};

use crate::contract::EventLogContract;
use crate::paths::eventlog_db_path_from_env;
use crate::projector::{start_eventlog_projector, EventLogProjectorHandle, EventLogRuntime};
use crate::query::EventLogQuery;
use crate::router::register_eventlog_rpc_handlers;
use crate::storage::EventLogStore;
use crate::watch::register_eventlog_watch_feed;

/// Controls whether this process owns Event Log background loops or only RPC serving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLogServiceMode {
    /// Serve RPCs only.
    RpcOnly,
    /// Serve RPCs and own the JetStream projector loop.
    Owner,
}

struct RuntimeLoops {
    projector: EventLogProjectorHandle,
}

impl EventLogServiceMode {
    fn starts_runtime_loops(self) -> bool {
        matches!(self, Self::Owner)
    }
}

impl RuntimeLoops {
    async fn start(
        eventlog_runtime: EventLogRuntime,
        store: EventLogStore,
    ) -> Result<Self, ServerError> {
        let projector = start_eventlog_projector(eventlog_runtime, store).await?;
        Ok(Self { projector })
    }

    async fn stop(self) {
        self.projector.stop().await;
    }

    async fn wait_for_failure(&mut self) -> Result<(), ServerError> {
        let result = self.projector.wait().await;
        self.projector.discard_completed();
        match result {
            Ok(()) => Err(ServerError::Nats(
                "event log projector loop exited unexpectedly".to_string(),
            )),
            Err(error) => Err(error),
        }
    }
}

/// Connected Event Log service wrapper.
pub struct ConnectedEventLogService {
    runtime: ConnectedServiceRuntime<EventLogContract>,
    store: EventLogStore,
}

impl ConnectedEventLogService {
    /// Construct a connected Event Log service wrapper from a Trellis service runtime.
    #[expect(
        clippy::result_large_err,
        reason = "ServerError preserves typed service startup diagnostics"
    )]
    pub fn new(runtime: ConnectedServiceRuntime<EventLogContract>) -> Result<Self, ServerError> {
        Ok(Self {
            runtime,
            store: open_eventlog_store_from_env()?,
        })
    }

    /// Run the Event Log service loops and request handler until shutdown.
    pub async fn run(self) -> Result<(), ServerError> {
        self.run_with_mode(EventLogServiceMode::Owner).await
    }

    /// Run the Event Log service with an explicit loop ownership mode.
    pub async fn run_with_mode(mut self, mode: EventLogServiceMode) -> Result<(), ServerError> {
        tracing::info!(?mode, "registering Event Log runtime surfaces");
        let eventlog_runtime = self.runtime.eventlog_runtime();
        let query = EventLogQuery::new(self.store.clone(), eventlog_runtime.clone());
        match eventlog_runtime.expire_obsolete_watch_consumers().await {
            Ok(count) if count > 0 => {
                tracing::info!(
                    count,
                    "scheduled obsolete EventLog.Watch consumers for expiry"
                );
            }
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(%error, "failed to expire obsolete EventLog.Watch consumers");
            }
        }
        register_eventlog_rpc_handlers(&mut self.runtime, query);
        register_eventlog_watch_feed(&mut self.runtime, eventlog_runtime.clone());
        run_eventlog_service_runtime(eventlog_runtime, self.store, mode, async move {
            self.runtime
                .run()
                .await
                .map_err(service_runtime_error_to_server_error)
        })
        .await
    }
}

#[expect(
    clippy::result_large_err,
    reason = "ServerError preserves typed storage startup diagnostics"
)]
fn open_eventlog_store_from_env() -> Result<EventLogStore, ServerError> {
    let db_path = eventlog_db_path_from_env();
    open_eventlog_store(&db_path)
}

#[expect(
    clippy::result_large_err,
    reason = "ServerError preserves typed storage startup diagnostics"
)]
fn open_eventlog_store(path: &Path) -> Result<EventLogStore, ServerError> {
    tracing::info!(path = %path.display(), "opening Event Log SQLite projection");
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            ServerError::Nats(format!(
                "failed to create Event Log SQLite projection directory '{}': {error}",
                parent.display()
            ))
        })?;
    }
    EventLogStore::open(path).map_err(|error| {
        ServerError::Nats(format!(
            "failed to open Event Log SQLite projection at '{}': {error}",
            path.display()
        ))
    })
}

async fn run_eventlog_service_runtime<F>(
    eventlog_runtime: EventLogRuntime,
    store: EventLogStore,
    mode: EventLogServiceMode,
    service_run: F,
) -> Result<(), ServerError>
where
    F: std::future::Future<Output = Result<(), ServerError>>,
{
    let mut loops = if mode.starts_runtime_loops() {
        Some(RuntimeLoops::start(eventlog_runtime, store).await?)
    } else {
        tracing::info!(?mode, "Event Log owner loops disabled");
        None
    };

    let result = if let Some(loops_ref) = loops.as_mut() {
        tokio::select! {
            result = service_run => result,
            loop_result = loops_ref.wait_for_failure() => loop_result,
        }
    } else {
        service_run.await
    };

    if let Some(loops) = loops {
        loops.stop().await;
    }
    result
}

/// Errors returned while connecting or running the Event Log service.
#[derive(Debug, thiserror::Error)]
pub enum EventLogServiceError {
    /// Client-side bootstrap or transport error.
    #[error(transparent)]
    Client(#[from] TrellisClientError),
    /// Server-side runtime error.
    #[error(transparent)]
    Server(#[from] ServerError),
}

/// Connect an Event Log service client and eagerly resolve its bindings.
pub async fn connect_service(
    opts: ServiceConnectOptions<'_>,
) -> Result<ConnectedEventLogService, EventLogServiceError> {
    let runtime = ConnectedServiceRuntime::<EventLogContract>::connect(opts)
        .await
        .map_err(map_service_runtime_error)?;
    ConnectedEventLogService::new(runtime).map_err(EventLogServiceError::Server)
}

fn map_service_runtime_error(error: ServiceRuntimeError) -> EventLogServiceError {
    match error {
        ServiceRuntimeError::Client(error) => EventLogServiceError::Client(error),
        ServiceRuntimeError::Server(error) => EventLogServiceError::Server(error),
        other => EventLogServiceError::Server(service_runtime_error_to_server_error(other)),
    }
}

fn service_runtime_error_to_server_error(error: ServiceRuntimeError) -> ServerError {
    match error {
        ServiceRuntimeError::Server(error) => error,
        other => ServerError::Nats(other.to_string()),
    }
}

/// Convenience helper that connects and immediately runs the Event Log service.
pub async fn connect_and_run(opts: ServiceConnectOptions<'_>) -> Result<(), EventLogServiceError> {
    let connected = connect_service(opts).await?;
    connected.run().await?;
    Ok(())
}
