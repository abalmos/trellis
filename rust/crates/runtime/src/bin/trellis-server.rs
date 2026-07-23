//! Command-line entrypoint for the Trellis Rust runtime process.

use std::path::PathBuf;

use clap::Parser;
use trellis_runtime::{RuntimeMode, RuntimeOptions};

/// Trellis runtime process entrypoint.
#[derive(Debug, Parser)]
#[command(version, about = "Run the Trellis runtime")]
struct Args {
    /// Runtime mode to run: all, platform, jobs, health, or eventlog.
    mode: RuntimeMode,
    /// Path to the Trellis runtime TOML configuration.
    #[arg(long)]
    config: PathBuf,
    /// Replace an existing pending first-administrator bootstrap URL.
    #[arg(long)]
    rotate_first_admin: bool,
}

/// Parses CLI arguments and starts the selected runtime process.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();
    let args = Args::parse();
    trellis_runtime::run(RuntimeOptions {
        mode: args.mode,
        config_path: args.config,
        rotate_first_admin: args.rotate_first_admin,
    })
    .await?;
    Ok(())
}
