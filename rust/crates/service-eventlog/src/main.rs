use std::{env, io};

use tracing_subscriber::EnvFilter;
use trellis_rs::service::ServiceConnectOptions;
use trellis_service_eventlog::{connect_service, EventLogServiceMode, SERVICE_NAME};

fn required_env(name: &str) -> Result<String, String> {
    env::var(name).map_err(|_| format!("missing required env var: {name}"))
}

fn service_mode() -> EventLogServiceMode {
    match env::var("TRELLIS_EVENTLOG_MODE").as_deref() {
        Ok("rpc-only") => EventLogServiceMode::RpcOnly,
        _ => EventLogServiceMode::Owner,
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,trellis_service_eventlog=debug"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(io::stderr)
        .try_init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    init_tracing();

    let trellis_url = required_env("TRELLIS_URL")?;
    let session_key_seed_base64url = required_env("SESSION_KEY_SEED_BASE64URL")?;
    let timeout_ms = env::var("TRELLIS_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(2_000);
    let mode = service_mode();

    tracing::info!(service = SERVICE_NAME, %trellis_url, timeout_ms, ?mode, "starting event log service");

    let mut options =
        ServiceConnectOptions::new(&trellis_url, SERVICE_NAME, &session_key_seed_base64url);
    options.timeout_ms = timeout_ms;

    let service = connect_service(options).await?;
    tracing::info!(service = SERVICE_NAME, "event log service connected");

    service.run_with_mode(mode).await?;
    tracing::info!(service = SERVICE_NAME, "event log service stopped");

    Ok(())
}
