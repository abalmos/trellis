use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use runtime_trellis::participants::test_events::{ConnectedService, ServiceConnectOptions};
use trellis_rs::client::MemoryAuthorizationContextStore;
use trellis_rs::generated::EventDescriptor;
use runtime_trellis::apis::test_events::events::BetaEventDescriptor;
use runtime_trellis::apis::test_events::rpc::Empty;
use runtime_trellis::apis::test_events::{BetaEvent, ObservedResponse};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        BetaEventDescriptor::publish_subject(&BetaEvent {
            site: "rust".to_owned(),
            value: "payload".to_owned(),
        })?,
        "events.v1.Beta.rust"
    );
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
    let consumers = service.event_consumers();
    let events = consumers.events();
    let seen = Arc::new(Mutex::new(BTreeSet::new()));
    let alpha = || {
        let seen = Arc::clone(&seen);
        events.alpha(move |event, _| {
            seen.lock().unwrap().insert(event.value);
            async { Ok(()) }
        })
    };
    let beta = || {
        let seen = Arc::clone(&seen);
        events.beta(move |event, _| {
            seen.lock().unwrap().insert(event.value);
            async { Ok(()) }
        })
    };
    let (alpha, _beta) = if std::env::var("REVERSE")?.parse::<bool>()? {
        let beta = beta().await?;
        (alpha().await?, beta)
    } else {
        (alpha().await?, beta().await?)
    };
    let alpha = Arc::new(Mutex::new(Some(alpha)));
    service.handle().rpc().drop_alpha().drop_alpha(move |_, _| {
        alpha.lock().unwrap().take();
        async { Ok(Empty {}) }
    });
    service.handle().rpc().observed().observed(move |_, _| {
        let values = seen.lock().unwrap().iter().cloned().collect();
        async { Ok(ObservedResponse { values }) }
    });
    service.run().await?;
    Ok(())
}
