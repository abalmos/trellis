//! Internal built-in Event Log runtime for Trellis.
//!
//! This crate hosts the read-only `trellis.eventlog@v1` API and maintains the
//! SQLite projection used by the Trellis runtime.

mod consumers;
mod projector;
mod query;
mod router;
pub mod storage;
mod watch;
mod wire;

pub use projector::{
    start_eventlog_projector, EventAuthorizationInput, EventLogProjectorHandle, EventLogRuntime,
    EventVerifier, VerifiedEventPublisher,
};
pub use query::{EventLogQuery, EventLogQueryError};
pub use router::build_router_with_query;
pub use storage::{EventLogStore, EventLogStoreError, ProjectedEvent};
pub use watch::register_eventlog_watch_feed;
