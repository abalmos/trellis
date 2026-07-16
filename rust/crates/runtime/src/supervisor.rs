#![cfg(all(feature = "sqlite-storage", feature = "nats-leases"))]

use std::collections::BTreeMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;

use thiserror::Error;
use tokio::task::JoinHandle;
use ulid::Ulid;

const HTTP_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
const SUBSYSTEM_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const OWNERSHIP_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

use crate::leases::{LeaseError, LeaseKey};
use crate::ownership::{OwnerContext, OwnerGroup, RuntimeOwnership};
use crate::shutdown::StopHandle;
use crate::storage::{RuntimeStores, StoreError};
use crate::{
    eventlog, health, jobs, platform, RuntimeConfig, RuntimeMode, ServerError, SubsystemName,
};

/// Runtime startup options for `trellis-server`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeOptions {
    /// Runtime mode selected by the command line.
    pub mode: RuntimeMode,
    /// Path to the TOML runtime config file.
    pub config_path: PathBuf,
}

/// Error returned while starting or running the Trellis runtime.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Runtime configuration could not be loaded or validated.
    #[error(transparent)]
    Config(#[from] crate::ConfigError),
    /// Runtime HTTP server failed.
    #[error(transparent)]
    Server(#[from] ServerError),
    /// Runtime storage failed.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// Runtime NATS connection failed.
    #[error("runtime NATS connection failed: {0}")]
    Nats(String),
    /// The runtime lease KV bucket could not be opened or created.
    #[error("failed to open runtime lease bucket for owner {owner_id}: {source}")]
    LeaseBucketOpen {
        /// Process-unique owner identity.
        owner_id: String,
        /// Low-level lease failure.
        #[source]
        source: Box<LeaseError>,
    },
    /// A selected singleton owner lease is already held.
    #[error("runtime owner lease {key:?} for {subsystem} is already held; owner {owner_id} cannot start: {source}")]
    OwnerHeld {
        /// Selected subsystem owner group.
        subsystem: SubsystemName,
        /// Stable owner lease key.
        key: LeaseKey,
        /// Process-unique owner identity.
        owner_id: String,
        /// Low-level held error.
        #[source]
        source: Box<LeaseError>,
    },
    /// A selected singleton owner lease could not be acquired.
    #[error(
        "failed to acquire runtime owner lease {key:?} for {subsystem} as {owner_id}: {source}"
    )]
    OwnerAcquire {
        /// Selected subsystem owner group.
        subsystem: SubsystemName,
        /// Stable owner lease key.
        key: LeaseKey,
        /// Process-unique owner identity.
        owner_id: String,
        /// Low-level lease failure.
        #[source]
        source: Box<LeaseError>,
    },
    /// A held owner lease was lost or could not be renewed.
    #[error("runtime owner lease {key:?} for {subsystem} was lost by {owner_id}: {source}")]
    OwnerRenewal {
        /// Selected subsystem owner group.
        subsystem: SubsystemName,
        /// Stable owner lease key.
        key: LeaseKey,
        /// Process-unique owner identity.
        owner_id: String,
        /// Low-level lease failure.
        #[source]
        source: Box<LeaseError>,
    },
    /// The critical owner renewal task exited without a stop request.
    #[error("runtime owner renewal task exited unexpectedly for {owner_id}")]
    OwnerRenewalTaskExited {
        /// Process-unique owner identity.
        owner_id: String,
    },
    /// The critical owner renewal task panicked or was cancelled.
    #[error("runtime owner renewal task failed for {owner_id}: {source}")]
    OwnerRenewalTaskFailed {
        /// Process-unique owner identity.
        owner_id: String,
        /// Tokio task join failure.
        #[source]
        source: tokio::task::JoinError,
    },
    /// The ownership renewal task did not stop within the shutdown bound.
    #[error("runtime owner renewal task did not stop within the shutdown bound for {owner_id}")]
    OwnerRenewalShutdownTimeout {
        /// Process-unique owner identity.
        owner_id: String,
    },
    /// A held owner lease could not be released safely.
    #[error(
        "failed to release runtime owner lease {key:?} for {subsystem} as {owner_id}: {source}"
    )]
    OwnerRelease {
        /// Selected subsystem owner group.
        subsystem: SubsystemName,
        /// Stable owner lease key.
        key: LeaseKey,
        /// Process-unique owner identity.
        owner_id: String,
        /// Low-level lease failure.
        #[source]
        source: Box<LeaseError>,
    },
    /// A selected subsystem was not given its acquired owner context.
    #[error("runtime owner context is missing for {subsystem}")]
    OwnerContextMissing {
        /// Selected subsystem owner group.
        subsystem: SubsystemName,
    },
    /// Health subsystem setup or runtime failed.
    #[error("health subsystem failed: {0}")]
    Health(String),
    /// A subsystem scaffold failed to start.
    #[error("failed to start runtime subsystem {subsystem}: {reason}")]
    Subsystem {
        /// Subsystem that failed to start.
        subsystem: SubsystemName,
        /// Failure reason.
        reason: &'static str,
    },
    /// A subsystem task failed after startup.
    #[error("runtime subsystem {subsystem} task failed: {source}")]
    SubsystemTask {
        /// Subsystem whose task failed.
        subsystem: SubsystemName,
        /// Tokio task join failure.
        #[source]
        source: tokio::task::JoinError,
    },
    /// A long-running subsystem exited without a stop request.
    #[error("runtime subsystem {subsystem} exited unexpectedly")]
    SubsystemExited {
        /// Subsystem that stopped unexpectedly.
        subsystem: SubsystemName,
    },
    /// The runtime HTTP server did not drain within the shutdown bound.
    #[error("runtime HTTP server did not drain within the shutdown bound")]
    HttpShutdownTimeout,
    /// A subsystem did not stop within the cooperative shutdown bound.
    #[error("runtime subsystem {subsystem} did not stop within the shutdown bound")]
    SubsystemShutdownTimeout {
        /// Subsystem that exceeded the shutdown bound.
        subsystem: SubsystemName,
    },
}

/// Shared runtime context passed to subsystem startup scaffolds.
#[derive(Debug)]
pub(crate) struct RuntimeContext {
    /// Loaded and validated runtime configuration.
    pub(crate) config: RuntimeConfig,
    /// Runtime mode selected for this process.
    pub(crate) mode: RuntimeMode,
    /// SQLite stores opened for the selected subsystems.
    pub(crate) stores: RuntimeStores,
    /// Trellis-account runtime NATS client shared by built-in subsystems.
    pub(crate) trellis_nats: async_nats::Client,
    /// Fixed owner contexts for selected runtime subsystems.
    owners: BTreeMap<OwnerGroup, OwnerContext>,
}

impl RuntimeContext {
    pub(crate) fn owner(&self, group: OwnerGroup) -> Result<OwnerContext, RuntimeError> {
        self.owners
            .get(&group)
            .cloned()
            .ok_or(RuntimeError::OwnerContextMissing {
                subsystem: group.subsystem(),
            })
    }
}

/// Handle for a started subsystem scaffold.
#[derive(Debug)]
pub(crate) struct SubsystemHandle {
    /// Started subsystem name.
    pub(crate) name: SubsystemName,
    /// Cooperative stop request handle for the subsystem task.
    pub(crate) stop: StopHandle,
    /// Join handle for the subsystem task.
    pub(crate) join: JoinHandle<Result<(), RuntimeError>>,
}

/// Loads configuration, validates selected subsystem storage, and runs the runtime.
pub async fn run(options: RuntimeOptions) -> Result<(), RuntimeError> {
    let config = RuntimeConfig::load_from_path(&options.config_path)?;
    config.validate_for_mode(options.mode)?;
    let nats = config.resolve_nats_runtime()?;
    let leases = config.resolve_leases()?;
    let trellis_nats = async_nats::ConnectOptions::new()
        .credentials_file(&nats.trellis_creds_path)
        .await
        .map_err(|error| RuntimeError::Nats(error.to_string()))?
        .connect(&nats.servers)
        .await
        .map_err(|error| RuntimeError::Nats(error.to_string()))?;
    let owner_id = format!(
        "{}-{}",
        config.instance_name.as_deref().unwrap_or("trellis-runtime"),
        Ulid::new()
    );
    let mut ownership =
        RuntimeOwnership::acquire(trellis_nats.clone(), &leases, owner_id, options.mode).await?;
    let result = run_owned(config, options.mode, trellis_nats.clone(), &mut ownership).await;
    let release_result = ownership.shutdown().await;
    if let Err(error) = trellis_nats.flush().await {
        tracing::warn!(error = %error, "failed to flush runtime NATS client during shutdown");
    }
    match (result, release_result) {
        (Err(primary), Err(release)) => {
            tracing::error!(error = %release, "runtime ownership shutdown also failed");
            Err(primary)
        }
        (Err(primary), Ok(())) => Err(primary),
        (Ok(()), Err(release)) => Err(release),
        (Ok(()), Ok(())) => Ok(()),
    }
}

async fn run_owned(
    config: RuntimeConfig,
    mode: RuntimeMode,
    trellis_nats: async_nats::Client,
    ownership: &mut RuntimeOwnership,
) -> Result<(), RuntimeError> {
    let stores = RuntimeStores::from_config(&config, mode)?;
    stores.migrate_all()?;
    let context = RuntimeContext {
        config,
        mode,
        stores,
        trellis_nats,
        owners: ownership.contexts(),
    };
    let mut handles = start_subsystems(&context).await?;
    let root_stop = StopHandle::new();
    let server_stop = root_stop.clone();
    let mut server = Box::pin(crate::run_http_server(
        &context.config,
        context.mode,
        async move {
            tokio::select! {
                () = crate::shutdown::shutdown_signal() => {}
                () = server_stop.stopped() => {}
            }
        },
    ));
    let (primary, server_finished) = tokio::select! {
        server_result = server.as_mut() => {
            (server_result.map_err(RuntimeError::from), true)
        }
        (index, task_result) = wait_for_subsystem(&mut handles), if !handles.is_empty() => {
            let failed = handles.swap_remove(index);
            let result = match task_result {
                Ok(Ok(())) => Err(RuntimeError::SubsystemExited { subsystem: failed.name }),
                Ok(Err(error)) => Err(error),
                Err(source) => Err(RuntimeError::SubsystemTask {
                    subsystem: failed.name,
                    source,
                }),
            };
            (result, false)
        }
        renewal_error = ownership.wait_for_renewal_failure() => {
            (Err(renewal_error), false)
        }
    };

    root_stop.stop();
    if let Err(error) = finish_shutdown(
        server,
        server_finished,
        handles,
        HTTP_SHUTDOWN_TIMEOUT,
        SUBSYSTEM_SHUTDOWN_TIMEOUT,
    )
    .await
    {
        if primary.is_err() {
            tracing::error!(error = %error, "runtime shutdown also failed");
        } else {
            return Err(error);
        }
    }
    primary
}

async fn wait_for_subsystem(
    handles: &mut [SubsystemHandle],
) -> (
    usize,
    Result<Result<(), RuntimeError>, tokio::task::JoinError>,
) {
    let waits = handles
        .iter_mut()
        .enumerate()
        .map(|(index, handle)| Box::pin(async move { (index, (&mut handle.join).await) }))
        .collect::<Vec<_>>();
    futures_util::future::select_all(waits).await.0
}

async fn stop_subsystems(mut handles: Vec<SubsystemHandle>) -> Result<(), RuntimeError> {
    for handle in &handles {
        handle.stop.stop();
    }
    join_subsystems(&mut handles, SUBSYSTEM_SHUTDOWN_TIMEOUT).await
}

async fn finish_shutdown<F>(
    mut server: Pin<Box<F>>,
    server_finished: bool,
    mut handles: Vec<SubsystemHandle>,
    http_timeout: Duration,
    subsystem_timeout: Duration,
) -> Result<(), RuntimeError>
where
    F: Future<Output = Result<(), ServerError>>,
{
    for handle in &handles {
        handle.stop.stop();
    }

    let mut first_error = None;
    if !server_finished {
        match tokio::time::timeout(http_timeout, server.as_mut()).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => first_error = Some(RuntimeError::Server(error)),
            Err(_) => {
                drop(server);
                first_error = Some(RuntimeError::HttpShutdownTimeout);
            }
        }
    }

    if let Err(error) = join_subsystems(&mut handles, subsystem_timeout).await {
        if first_error.is_some() {
            tracing::error!(error = %error, "runtime subsystem shutdown also failed");
        } else {
            first_error = Some(error);
        }
    }

    first_error.map_or(Ok(()), Err)
}

async fn join_subsystems(
    handles: &mut [SubsystemHandle],
    timeout: Duration,
) -> Result<(), RuntimeError> {
    let mut first_error = None;
    for handle in handles {
        let subsystem = handle.name;
        match tokio::time::timeout(timeout, &mut handle.join).await {
            Ok(Ok(Ok(()))) => {}
            Ok(Ok(Err(error))) => {
                if first_error.is_some() {
                    tracing::error!(error = %error, "additional runtime subsystem shutdown failed");
                } else {
                    first_error = Some(error);
                }
            }
            Ok(Err(source)) => {
                let error = RuntimeError::SubsystemTask { subsystem, source };
                if first_error.is_some() {
                    tracing::error!(error = %error, "additional runtime subsystem shutdown failed");
                } else {
                    first_error = Some(error);
                }
            }
            Err(_) => {
                handle.join.abort();
                let _ = (&mut handle.join).await;
                let error = RuntimeError::SubsystemShutdownTimeout { subsystem };
                if first_error.is_some() {
                    tracing::error!(error = %error, "additional runtime subsystem shutdown failed");
                } else {
                    first_error = Some(error);
                }
            }
        }
    }

    first_error.map_or(Ok(()), Err)
}

async fn start_subsystems(context: &RuntimeContext) -> Result<Vec<SubsystemHandle>, RuntimeError> {
    let mut handles = Vec::new();
    for subsystem in context.mode.subsystems() {
        let result = match subsystem {
            SubsystemName::Platform => platform::start(context),
            SubsystemName::Jobs => jobs::start(context),
            SubsystemName::Health => health::start(context).await,
            SubsystemName::Eventlog => eventlog::start(context),
        };
        match result {
            Ok(handle) => handles.push(handle),
            Err(primary) => {
                handles.reverse();
                if let Err(cleanup) = stop_subsystems(handles).await {
                    tracing::error!(error = %cleanup, "started subsystem cleanup also failed");
                }
                return Err(primary);
            }
        }
    }
    Ok(handles)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn subsystem_stop_is_signaled_before_pending_http_drain_is_awaited() {
        let stop = StopHandle::new();
        let task_stop = stop.clone();
        let notified = Arc::new(AtomicBool::new(false));
        let task_notified = Arc::clone(&notified);
        let (sent, received) = tokio::sync::oneshot::channel();
        let join = tokio::spawn(async move {
            task_stop.stopped().await;
            task_notified.store(true, Ordering::SeqCst);
            sent.send(()).expect("HTTP test receiver remains open");
            std::future::pending().await
        });
        let server = Box::pin(async move {
            received.await.expect("subsystem reports stop notification");
            std::future::pending::<Result<(), ServerError>>().await
        });

        let error = finish_shutdown(
            server,
            false,
            vec![SubsystemHandle {
                name: SubsystemName::Jobs,
                stop,
                join,
            }],
            Duration::from_millis(20),
            Duration::from_millis(20),
        )
        .await
        .expect_err("pending HTTP drain must time out");

        assert!(notified.load(Ordering::SeqCst));
        assert!(matches!(error, RuntimeError::HttpShutdownTimeout));
    }
}
