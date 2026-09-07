use runtime_trellis::apis::test_runtime::EchoResponse;
use runtime_trellis::participants::test_provider::{ConnectedService, ServiceConnectOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let url = std::env::var("TRELLIS_URL")?;
    let identity = std::env::var("TRELLIS_IDENTITY_SEED")?;
    let mut service =
        ConnectedService::connect(ServiceConnectOptions::new(&url, &identity)).await?;
    service.handle().rpc().echo().echo(|_, input| async move {
        assert_eq!(input.value, "from TypeScript");
        Ok(EchoResponse {
            value: format!("Rust received {}", input.value),
        })
    });
    service.run().await?;
    Ok(())
}
