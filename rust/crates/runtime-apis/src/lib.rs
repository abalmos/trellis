//! Workspace-private generated Trellis API projections for runtime internals.

#![allow(
    clippy::enum_variant_names,
    reason = "wire-schema error names are generated from canonical API artifacts"
)]

pub use trellis_rs::{generated, service};

/// Canonical generated participant artifact for the platform runtime.
pub const AUTH_RUNTIME_PARTICIPANT_JSON: &str = include_str!("trellis.auth-runtime.json");

#[path = "../../trellis/src/internal_sdk/generated/auth/lib.rs"]
pub mod auth;
#[path = "../../trellis/src/internal_sdk/generated/core/mod.rs"]
pub mod core;
#[path = "../../trellis/src/internal_sdk/generated/eventlog/lib.rs"]
pub mod eventlog;
#[path = "../../trellis/src/internal_sdk/generated/health/lib.rs"]
pub mod health;
#[path = "../../trellis/src/internal_sdk/generated/jobs/lib.rs"]
pub mod jobs;
#[path = "../../trellis/src/internal_sdk/generated/state/lib.rs"]
pub mod state;
