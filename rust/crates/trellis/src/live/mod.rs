//! Live observation sessions shared by Feed and Operation observation.
//!
//! This module owns the per-connection session runtime: retained authority
//! guards, prepared provider and consumer sessions, the serialized control and
//! publication paths, and the public owned subscription handle. It is
//! crate-private assembly; the public surface is re-exported from the crate
//! root.

//! Live observation sessions shared by Feed and Operation observation.
//!
//! This module owns the per-connection session runtime: retained authority
//! guards, prepared provider and consumer sessions, the serialized control and
//! publication paths, and the public owned subscription handle. It is
//! crate-private assembly; the public surface is re-exported from the crate
//! root.
//!
//! # Staged surface
//!
//! The engine is being landed in work packages: the shared protocol, ACLs and
//! built-in provider identities are live, and the consumer Feed path is wired.
//! The provider router registration and Operation observation migration consume
//! the remaining primitives. Until that wiring lands, unused engine items are
//! allowed here with this single documented reason rather than scattered
//! per-item allowances; remove this attribute when the router and Operation
//! migration land.
#![allow(
    dead_code,
    reason = "live engine staged for router and Operation migration"
)]

pub(crate) mod authority;
pub(crate) mod client_open;
pub(crate) mod manager;

pub use manager::LiveSessionManager;
pub(crate) mod provider;
pub(crate) mod provider_engine;
pub(crate) mod subscription;
pub(crate) mod types;

pub use types::{
    CloseCleanupState, CloseRemoteState, LiveCancellation, LiveCloseReceipt, LiveEnd,
    LiveEndReason, LiveErrorCode, LiveStreamError,
};
