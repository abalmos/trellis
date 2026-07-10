//! Event Log service runtime for Trellis.
//!
//! This crate hosts the read-only `trellis.eventlog@v1` API and maintains a
//! SQLite projection of the Trellis JetStream event stream.

mod bootstrap;
mod consumers;
mod contract;
mod paths;
mod projector;
mod query;
mod router;
pub mod storage;
mod watch;
mod wire;

pub use bootstrap::{
    connect_and_run, connect_service, ConnectedEventLogService, EventLogServiceError,
    EventLogServiceMode,
};
pub use contract::{
    contract_manifest, expected_contract, EventLogContract, CONTRACT_DIGEST, CONTRACT_ID,
    CONTRACT_JSON, EVENTLOG_RPC_SUBJECTS, SERVICE_NAME,
};
pub use projector::{start_eventlog_projector, EventLogProjectorHandle, EventLogRuntime};
pub use query::{EventLogQuery, EventLogQueryError};
pub use router::{build_router_with_query, register_eventlog_rpc_handlers};
pub use storage::{EventLogStore, EventLogStoreError, ProjectedEvent};
pub use watch::register_eventlog_watch_feed;
