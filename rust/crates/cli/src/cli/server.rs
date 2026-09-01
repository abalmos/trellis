use std::path::PathBuf;

use clap::Args;
use trellis_runtime::RuntimeMode;

#[derive(Debug, Args)]
/// Run the Trellis runtime, auto-managing a local nats-server when no --nats URL is given.
pub struct ServerArgs {
    /// Runtime mode to run: all, platform, jobs, health, or eventlog.
    #[arg(default_value = "all")]
    pub mode: RuntimeMode,
    /// Path to the Trellis runtime TOML configuration.
    #[arg(long)]
    pub config: PathBuf,
    /// Use an external NATS server at this URL instead of managing a local one.
    #[arg(long, conflicts_with_all = ["nats_binary", "cache_dir", "nats_state_dir"])]
    pub nats: Option<String>,
    /// Issue a one-time password-reset URL for the sole active administrator.
    #[arg(long)]
    pub reset_admin: bool,
    /// Validate configuration and authorization trust, then exit.
    #[arg(long)]
    pub check: bool,
    /// Cache directory for the managed nats-server binary.
    #[arg(long, conflicts_with_all = ["nats", "nats_binary"])]
    pub cache_dir: Option<PathBuf>,
    /// Path to an existing nats-server binary to manage instead of downloading one.
    /// Trusted operator input: the binary is NOT re-verified against the Trellis pin
    /// or version; it must be a regular executable owned by root or the current user.
    #[arg(long, conflicts_with_all = ["nats", "cache_dir"])]
    pub nats_binary: Option<PathBuf>,
    /// Directory for ALL mutable managed-NATS files (nats.local.conf, jwt.local.conf,
    /// pid file, resolver JWTs, JetStream data); the bundle's nats/ dir stays read-only.
    /// Defaults to the bundle's nats/ dir (host/dev behavior).
    #[arg(long, conflicts_with = "nats")]
    pub nats_state_dir: Option<PathBuf>,
}
