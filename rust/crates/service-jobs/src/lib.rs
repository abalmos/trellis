//! Jobs admin service runtime for Trellis.
//!
//! This crate implements the admin-side loops and RPC hosting for the standard
//! `trellis.jobs@v1` Trellis API: SQLite-backed queries, stream projection, janitor
//! expiry, and advisory handling. Service-local job execution lives in the
//! internal `trellis-jobs` crate.

mod advisory;
mod bootstrap;
mod contract;
mod janitor;
mod paths;
mod projector;
mod query;
mod router;
pub mod storage;
mod watch;
pub mod worker_presence;

pub use advisory::{
    map_dead_event_from_advisory_job, start_advisory_loop, AdvisoryHandle, MappedDeadEvent,
    MaxDeliveriesAdvisory,
};
pub use bootstrap::{
    connect_and_run, connect_service, ConnectedJobsService, JobsServiceError, JobsServiceMode,
};
pub use contract::{
    contract_manifest, expected_contract, rpc, JobsContract, CONTRACT_DIGEST, CONTRACT_ID,
    CONTRACT_JSON, JOBS_RPC_SUBJECTS, SERVICE_NAME,
};
pub use janitor::{
    plan_expired_events, run_janitor_once, start_janitor_loop, JanitorError, JanitorHandle,
    JanitorRunStats, PlannedExpiredEvent,
};
pub use query::{JobsAdminResources, JobsQuery, JobsQueryError};
pub use router::{build_router_with_query, register_jobs_rpc_handlers};
pub use storage::{ListJobsFilter, SqliteJobsStore, SqliteJobsStoreError};
pub use watch::register_jobs_watch_feed;
