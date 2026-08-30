//! Workspace-private generated Trellis API projections for runtime internals.

#![allow(
    clippy::enum_variant_names,
    reason = "wire-schema error names are generated from canonical API artifacts"
)]

pub use trellis_rs::{contracts, generated, service};

#[path = "../../trellis/src/internal_sdk/generated/auth/mod.rs"]
pub mod auth;
#[path = "../../trellis/src/internal_sdk/generated/core/mod.rs"]
pub mod core;
#[path = "../../trellis/src/internal_sdk/generated/eventlog/mod.rs"]
pub mod eventlog;
#[path = "../../trellis/src/internal_sdk/generated/health/mod.rs"]
pub mod health;
#[path = "../../trellis/src/internal_sdk/generated/jobs/mod.rs"]
pub mod jobs;
#[path = "../../trellis/src/internal_sdk/generated/state/mod.rs"]
pub mod state;
