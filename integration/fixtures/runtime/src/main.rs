use std::sync::Arc;

use trellis_participant_test_provider::{ConnectedService, ServiceConnectOptions};
use trellis_rs::client::MemoryAuthorizationContextStore;
use trellis_sdk_test_runtime::EchoResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let url = std::env::var("TRELLIS_URL")?;
    let deployment = std::env::var("TRELLIS_DEPLOYMENT")?;
    let identity = std::env::var("TRELLIS_IDENTITY_SEED")?;
    let session = std::env::var("TRELLIS_SESSION_SEED")?;
    let name = std::env::var("TRELLIS_INSTANCE")?;
    let mut service = ConnectedService::connect(ServiceConnectOptions::new(
        &url,
        &name,
        &deployment,
        &identity,
        &session,
        Arc::new(MemoryAuthorizationContextStore::default()),
    ))
    .await?;
    service.handle().rpc().echo().echo(|_, input| async move {
        assert_eq!(input.value, "from TypeScript");
        Ok(EchoResponse { value: format!("Rust received {}", input.value) })
    });
    service.run().await?;
    Ok(())
}
