//! Workspace-private generated Trellis API projections for runtime internals.

#![allow(
    clippy::enum_variant_names,
    reason = "wire-schema error names are generated from canonical API artifacts"
)]

pub use trellis_rs::{generated, service};

/// Canonical generated participant artifact for the platform runtime.
pub const AUTH_RUNTIME_PARTICIPANT_JSON: &str = include_str!("trellis.auth-runtime.json");
/// Canonical built-in Console API artifact.
pub const CONSOLE_API_JSON: &str = include_str!("trellis-app.console@v1.api.json");
/// Canonical built-in Console participant artifact.
pub const CONSOLE_PARTICIPANT_JSON: &str = include_str!("trellis-app.console@v1.participant.json");
/// Canonical built-in activation Portal API artifact.
pub const PORTAL_API_JSON: &str = include_str!("trellis-app.portal@v1.api.json");
/// Canonical built-in activation Portal participant artifact.
pub const PORTAL_PARTICIPANT_JSON: &str = include_str!("trellis-app.portal@v1.participant.json");

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
