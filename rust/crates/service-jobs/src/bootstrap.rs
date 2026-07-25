//! Service bootstrap helpers for the Jobs admin service.

use std::path::Path;
use std::time::Duration;

use trellis_rs::generated::TrellisClientError;
use trellis_rs::service::{
    ConnectedServiceRuntime, ServerError, ServiceConnectOptions, ServiceRuntimeError,
};

use crate::advisory::{start_advisory_loop, AdvisoryHandle};
use crate::contract::JobsContract;
use crate::janitor::{start_janitor_loop, JanitorHandle};
use crate::paths::jobs_db_path_from_env;
use crate::projector::{start_jobs_projector, JobsProjectorHandle};
use crate::query::{jobs_admin_resources, JobsAdminResources, JobsQuery};
use crate::router::register_jobs_rpc_handlers;
use crate::storage::SqliteJobsStore;
use crate::watch::{expire_obsolete_watch_consumers, register_jobs_watch_feed};
use crate::worker_presence::{start_worker_presence_projector, WorkerPresenceProjectorHandle};

/// Controls whether this process owns background jobs-service loops or only RPC serving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobsServiceMode {
    /// Serve RPCs only.
    RpcOnly,
    /// Serve RPCs and own projector, janitor, and advisory loops.
    Owner,
}

struct RuntimeLoops {
    advisory: AdvisoryHandle,
    janitor: JanitorHandle,
    projector: JobsProjectorHandle,
    worker_presence: WorkerPresenceProjectorHandle,
}

#[derive(Debug, Clone, Copy)]
enum RuntimeLoopName {
    Advisory,
    Janitor,
    Projector,
    WorkerPresence,
}

impl RuntimeLoopName {
    fn as_str(self) -> &'static str {
        match self {
            Self::Advisory => "advisory",
            Self::Janitor => "janitor",
            Self::Projector => "projector",
            Self::WorkerPresence => "worker presence",
        }
    }
}

impl JobsServiceMode {
    fn starts_runtime_loops(self) -> bool {
        matches!(self, Self::Owner)
    }
}

impl RuntimeLoops {
    async fn start(
        jobs_runtime: trellis_rs::jobs::JobsRuntime,
        resources: &JobsAdminResources,
        store: SqliteJobsStore,
    ) -> Result<Self, ServerError> {
        tracing::info!(
            jobs_stream = %resources.jobs_stream,
            advisories_stream = %resources.jobs_advisories_stream,
            "starting jobs service owner loops"
        );
        let advisory = start_advisory_loop(
            jobs_runtime.clone(),
            store.clone(),
            resources.jobs_advisories_stream.clone(),
        )
        .await?;
        tracing::debug!(loop_name = "advisory", "jobs service owner loop started");
        let janitor =
            start_janitor_loop(jobs_runtime.clone(), store.clone(), janitor_interval()).await?;
        tracing::debug!(loop_name = "janitor", "jobs service owner loop started");
        let projector = start_jobs_projector(
            jobs_runtime.clone(),
            store.clone(),
            resources.jobs_stream.clone(),
        )
        .await?;
        tracing::debug!(loop_name = "projector", "jobs service owner loop started");
        let worker_presence =
            start_worker_presence_projector(jobs_runtime, resources.jobs_stream.clone(), store)
                .await?;
        tracing::debug!(
            loop_name = "worker_presence",
            "jobs service owner loop started"
        );
        Ok(Self {
            advisory,
            janitor,
            projector,
            worker_presence,
        })
    }

    async fn stop(self) {
        tracing::info!("stopping jobs service owner loops");
        let ((), (), ()) = tokio::join!(
            self.projector.stop(),
            self.janitor.stop(),
            self.advisory.stop(),
        );
        self.worker_presence.stop().await;
        tracing::info!("stopped jobs service owner loops");
    }

    async fn wait_for_failure(&mut self) -> Result<(), ServerError> {
        let (loop_name, result) = tokio::select! {
            result = self.projector.wait() => (RuntimeLoopName::Projector, result),
            result = self.worker_presence.wait() => (RuntimeLoopName::WorkerPresence, result),
            result = self.janitor.wait() => (RuntimeLoopName::Janitor, result),
            result = self.advisory.wait() => (RuntimeLoopName::Advisory, result),
        };

        match loop_name {
            RuntimeLoopName::Advisory => self.advisory.discard_completed(),
            RuntimeLoopName::Janitor => self.janitor.discard_completed(),
            RuntimeLoopName::Projector => self.projector.discard_completed(),
            RuntimeLoopName::WorkerPresence => self.worker_presence.discard_completed(),
        }

        map_runtime_loop_result(loop_name.as_str(), result)
    }
}

const DEFAULT_JANITOR_INTERVAL: Duration = Duration::from_secs(30);

fn janitor_interval() -> Duration {
    let value = std::env::var("TRELLIS_JOBS_JANITOR_INTERVAL_MS").ok();
    parse_janitor_interval(value.as_deref())
}

fn parse_janitor_interval(value: Option<&str>) -> Duration {
    value
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|millis| *millis > 0)
        .map(Duration::from_millis)
        .unwrap_or(DEFAULT_JANITOR_INTERVAL)
}

#[expect(
    clippy::result_large_err,
    reason = "ServerError preserves typed service loop diagnostics"
)]
fn map_runtime_loop_result(
    loop_name: &str,
    result: Result<(), ServerError>,
) -> Result<(), ServerError> {
    match result {
        Ok(()) => Err(ServerError::Nats(format!(
            "jobs service {loop_name} loop exited unexpectedly"
        ))),
        Err(error) => Err(error),
    }
}

/// Connected jobs service wrapper that mirrors TS `connectService` ergonomics.
pub struct ConnectedJobsService {
    runtime: ConnectedServiceRuntime<JobsContract>,
    jobs_runtime: trellis_rs::jobs::JobsRuntime,
    jobs_store: SqliteJobsStore,
}

impl ConnectedJobsService {
    /// Construct a connected Jobs service wrapper from a high-level Trellis service runtime.
    #[expect(
        clippy::result_large_err,
        reason = "ServerError preserves typed service startup diagnostics"
    )]
    pub fn new(runtime: ConnectedServiceRuntime<JobsContract>) -> Result<Self, ServerError> {
        let jobs_runtime = runtime.jobs_runtime();
        Ok(Self {
            runtime,
            jobs_runtime,
            jobs_store: open_jobs_store_from_env()?,
        })
    }

    /// Run the Jobs admin service loops and request handler until shutdown.
    pub async fn run(self) -> Result<(), ServerError> {
        self.run_with_mode(JobsServiceMode::Owner).await
    }

    /// Run the Jobs admin service with an explicit loop ownership mode.
    pub async fn run_with_mode(mut self, mode: JobsServiceMode) -> Result<(), ServerError> {
        tracing::info!(?mode, "registering jobs admin runtime surfaces");
        let jobs_runtime = self.jobs_runtime.clone();
        let (resources, query, store) =
            build_jobs_runtime(jobs_runtime.clone(), self.jobs_store.clone())?;
        match expire_obsolete_watch_consumers(&jobs_runtime, &resources.jobs_stream).await {
            Ok(count) if count > 0 => {
                tracing::info!(count, "scheduled obsolete Jobs.Watch consumers for expiry");
            }
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(%error, "failed to expire obsolete Jobs.Watch consumers");
            }
        }
        register_jobs_rpc_handlers(&mut self.runtime, query);
        register_jobs_watch_feed(
            &mut self.runtime,
            jobs_runtime.clone(),
            resources.jobs_stream.clone(),
        );
        tracing::debug!(?mode, "jobs admin RPC handlers and watch feed registered");
        run_jobs_service_runtime(jobs_runtime, resources, store, mode, async move {
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
    reason = "ServerError preserves typed Jobs runtime diagnostics"
)]
fn build_jobs_runtime(
    jobs_runtime: trellis_rs::jobs::JobsRuntime,
    store: SqliteJobsStore,
) -> Result<(JobsAdminResources, JobsQuery, SqliteJobsStore), ServerError> {
    let resources = jobs_admin_resources();
    tracing::debug!(
        jobs_stream = %resources.jobs_stream,
        advisories_stream = %resources.jobs_advisories_stream,
        "resolved jobs admin resources"
    );
    let query = JobsQuery::with_store(jobs_runtime, store.clone());
    Ok((resources, query, store))
}

#[expect(
    clippy::result_large_err,
    reason = "ServerError preserves typed storage startup diagnostics"
)]
fn open_jobs_store_from_env() -> Result<SqliteJobsStore, ServerError> {
    let db_path = jobs_db_path_from_env();
    open_jobs_store(&db_path)
}

#[expect(
    clippy::result_large_err,
    reason = "ServerError preserves typed storage startup diagnostics"
)]
fn open_jobs_store(path: &Path) -> Result<SqliteJobsStore, ServerError> {
    tracing::info!(path = %path.display(), "opening jobs SQLite projection");
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            ServerError::Nats(format!(
                "failed to create Jobs SQLite projection directory '{}': {error}",
                parent.display()
            ))
        })?;
    }

    let store = SqliteJobsStore::open(path).map_err(|error| {
        ServerError::Nats(format!(
            "failed to open Jobs SQLite projection at '{}': {error}",
            path.display()
        ))
    })?;
    tracing::info!(path = %path.display(), "opened jobs SQLite projection");
    Ok(store)
}

async fn run_jobs_service_runtime<F>(
    jobs_runtime: trellis_rs::jobs::JobsRuntime,
    resources: JobsAdminResources,
    store: SqliteJobsStore,
    mode: JobsServiceMode,
    service_run: F,
) -> Result<(), ServerError>
where
    F: std::future::Future<Output = Result<(), ServerError>>,
{
    tracing::info!(?mode, "starting jobs service runtime");
    let mut loops = if mode.starts_runtime_loops() {
        Some(RuntimeLoops::start(jobs_runtime, &resources, store).await?)
    } else {
        tracing::info!(?mode, "jobs service owner loops disabled");
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
    if let Err(error) = &result {
        tracing::error!(%error, "jobs service runtime stopped with error");
    } else {
        tracing::info!("jobs service runtime stopped");
    }
    result
}

/// Errors returned while connecting or running the Jobs admin service.
#[derive(Debug, thiserror::Error)]
pub enum JobsServiceError {
    #[error(transparent)]
    Client(#[from] TrellisClientError),
    #[error(transparent)]
    Server(#[from] ServerError),
}

/// Connect a Jobs admin service client and eagerly resolve its bindings.
pub async fn connect_service(
    opts: ServiceConnectOptions<'_>,
) -> Result<ConnectedJobsService, JobsServiceError> {
    let runtime = ConnectedServiceRuntime::<JobsContract>::connect(opts)
        .await
        .map_err(map_service_runtime_error)?;
    ConnectedJobsService::new(runtime).map_err(JobsServiceError::Server)
}

fn map_service_runtime_error(error: ServiceRuntimeError) -> JobsServiceError {
    match error {
        ServiceRuntimeError::Client(error) => JobsServiceError::Client(error),
        ServiceRuntimeError::Server(error) => JobsServiceError::Server(error),
        other => JobsServiceError::Server(service_runtime_error_to_server_error(other)),
    }
}

fn service_runtime_error_to_server_error(error: ServiceRuntimeError) -> ServerError {
    match error {
        ServiceRuntimeError::Server(error) => error,
        other => ServerError::Nats(other.to_string()),
    }
}

/// Convenience helper that connects and immediately runs the Jobs admin service.
pub async fn connect_and_run(opts: ServiceConnectOptions<'_>) -> Result<(), JobsServiceError> {
    let connected = connect_service(opts).await?;
    connected.run().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use trellis_rs::generated::TrellisClientError;
    use trellis_rs::service::ServiceConnectOptions;

    use super::{
        connect_and_run, connect_service, map_runtime_loop_result, parse_janitor_interval,
        JobsServiceMode,
    };

    const VALID_SEED_BASE64URL: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    #[tokio::test]
    async fn connect_service_rejects_invalid_session_seed_before_network() {
        let options = ServiceConnectOptions::new(
            "http://127.0.0.1:1",
            "trellis-service-jobs",
            "dep_test",
            "trellis.jobs@v1",
            "participant-digest",
            "participant-needs-digest",
            VALID_SEED_BASE64URL,
            "not-base64url",
            std::sync::Arc::new(trellis_rs::client::MemoryAuthorizationContextStore::default()),
        )
        .with_timeout_ms(1_000);
        let result = connect_service(options).await;

        assert!(matches!(
            result,
            Err(super::JobsServiceError::Client(TrellisClientError::Base64(
                _
            ))) | Err(super::JobsServiceError::Client(
                TrellisClientError::InvalidSeedLen(_)
            ))
        ));
    }

    #[tokio::test]
    async fn connect_service_returns_bootstrap_error_for_invalid_trellis_url() {
        let options = ServiceConnectOptions::new(
            "not a url",
            "trellis-service-jobs",
            "dep_test",
            "trellis.jobs@v1",
            "participant-digest",
            "participant-needs-digest",
            VALID_SEED_BASE64URL,
            VALID_SEED_BASE64URL,
            std::sync::Arc::new(trellis_rs::client::MemoryAuthorizationContextStore::default()),
        )
        .with_timeout_ms(1_000);
        let result = connect_service(options).await;

        assert!(matches!(
            result,
            Err(super::JobsServiceError::Client(
                TrellisClientError::Bootstrap(_)
            ))
        ));
    }

    #[tokio::test]
    async fn connect_and_run_propagates_connect_error() {
        let options = ServiceConnectOptions::new(
            "http://127.0.0.1:1",
            "trellis-service-jobs",
            "dep_test",
            "trellis.jobs@v1",
            "participant-digest",
            "participant-needs-digest",
            VALID_SEED_BASE64URL,
            "not-base64url",
            std::sync::Arc::new(trellis_rs::client::MemoryAuthorizationContextStore::default()),
        )
        .with_timeout_ms(1_000);
        let result = connect_and_run(options).await;

        assert!(matches!(
            result,
            Err(super::JobsServiceError::Client(TrellisClientError::Base64(
                _
            ))) | Err(super::JobsServiceError::Client(
                TrellisClientError::InvalidSeedLen(_)
            ))
        ));
    }

    #[test]
    fn jobs_service_mode_controls_background_loop_ownership() {
        assert!(!JobsServiceMode::RpcOnly.starts_runtime_loops());
        assert!(JobsServiceMode::Owner.starts_runtime_loops());
    }

    #[test]
    fn unexpected_clean_runtime_loop_exit_is_treated_as_failure() {
        let error =
            map_runtime_loop_result("projector", Ok(())).expect_err("clean exit should fail");
        assert!(error
            .to_string()
            .contains("projector loop exited unexpectedly"));
    }

    #[test]
    fn janitor_interval_parser_defaults_for_missing_invalid_or_zero_values() {
        assert_eq!(
            parse_janitor_interval(None),
            std::time::Duration::from_secs(30)
        );
        assert_eq!(
            parse_janitor_interval(Some("nope")),
            std::time::Duration::from_secs(30)
        );
        assert_eq!(
            parse_janitor_interval(Some("0")),
            std::time::Duration::from_secs(30)
        );
        assert_eq!(
            parse_janitor_interval(Some("250")),
            std::time::Duration::from_millis(250)
        );
    }
}
