#![cfg(all(feature = "sqlite-storage", feature = "nats-leases"))]

use std::path::PathBuf;

use thiserror::Error;
use tokio::task::JoinHandle;
use ulid::Ulid;

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
    /// Process-unique owner identity used for runtime leases.
    pub(crate) owner_id: String,
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
    let stores = RuntimeStores::from_config(&config, options.mode)?;
    stores.migrate_all()?;
    let nats = config.resolve_nats_runtime()?;
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

    let context = RuntimeContext {
        config,
        mode: options.mode,
        stores,
        trellis_nats,
        owner_id,
    };
    let mut handles = start_subsystems(&context).await?;
    let server = crate::run_http_server(
        &context.config,
        context.mode,
        crate::shutdown::shutdown_signal(),
    );
    tokio::pin!(server);

    tokio::select! {
        server_result = &mut server => {
            stop_subsystems(handles).await?;
            server_result?;
            Ok(())
        }
        (index, task_result) = wait_for_subsystem(&mut handles), if !handles.is_empty() => {
            let failed = handles.swap_remove(index);
            stop_subsystems(handles).await?;
            match task_result {
                Ok(Ok(())) => Err(RuntimeError::SubsystemExited { subsystem: failed.name }),
                Ok(Err(error)) => Err(error),
                Err(source) => Err(RuntimeError::SubsystemTask {
                    subsystem: failed.name,
                    source,
                }),
            }
        }
    }
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

async fn stop_subsystems(handles: Vec<SubsystemHandle>) -> Result<(), RuntimeError> {
    for handle in &handles {
        handle.stop.stop();
    }

    for handle in handles {
        let subsystem = handle.name;
        handle
            .join
            .await
            .map_err(|source| RuntimeError::SubsystemTask { subsystem, source })??;
    }

    Ok(())
}

async fn start_subsystems(context: &RuntimeContext) -> Result<Vec<SubsystemHandle>, RuntimeError> {
    let mut handles = Vec::new();
    for subsystem in context.mode.subsystems() {
        let handle = match subsystem {
            SubsystemName::Platform => platform::start(context)?,
            SubsystemName::Jobs => jobs::start(context)?,
            SubsystemName::Health => health::start(context).await?,
            SubsystemName::Eventlog => eventlog::start(context)?,
        };
        handles.push(handle);
    }
    Ok(handles)
}
