use std::{env, io, sync::Arc};

use tracing_subscriber::EnvFilter;
use trellis_rs::{client::FileAuthorizationContextStore, service::ServiceConnectOptions};
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
    let provisioned_identity_seed_base64url = required_env("PROVISIONED_IDENTITY_SEED_BASE64URL")?;
    let deployment_id = required_env("TRELLIS_DEPLOYMENT_ID")?;
    let participant_id = required_env("TRELLIS_PARTICIPANT_ID")?;
    let participant_digest = required_env("TRELLIS_PARTICIPANT_DIGEST")?;
    let participant_needs_digest = required_env("TRELLIS_PARTICIPANT_NEEDS_DIGEST")?;
    let authorization_context_file = required_env("TRELLIS_AUTHORIZATION_CONTEXT_FILE")?;
    let timeout_ms = env::var("TRELLIS_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(2_000);
    let mode = service_mode();

    tracing::info!(service = SERVICE_NAME, %trellis_url, timeout_ms, ?mode, "starting event log service");

    let options = ServiceConnectOptions::new(
        &trellis_url,
        SERVICE_NAME,
        &deployment_id,
        &participant_id,
        &participant_digest,
        &participant_needs_digest,
        &provisioned_identity_seed_base64url,
        &session_key_seed_base64url,
        Arc::new(FileAuthorizationContextStore::new(
            authorization_context_file,
        )),
    )
    .with_timeout_ms(timeout_ms);

    let service = connect_service(options).await?;
    tracing::info!(service = SERVICE_NAME, "event log service connected");

    service.run_with_mode(mode).await?;
    tracing::info!(service = SERVICE_NAME, "event log service stopped");

    Ok(())
}
