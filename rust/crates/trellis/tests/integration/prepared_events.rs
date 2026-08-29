use std::sync::Arc;
use std::time::{Duration, Instant};

use async_nats::HeaderMap;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use trellis_rs::client::EventDescriptor;
use trellis_rs::service::{
    ServerError, ServiceEventListenOptions, ServiceEventListenerContext, ServiceEventListenerMode,
    ServiceRuntimeError,
};

use crate::support::assertions::assert_runtime_case_registered;

const CASE_ID: &str =
    "prepared-events.prepared-publish-preserves-custom-headers-and-annotates-handler-error";
const TRACEPARENT: &str = "00-0123456789abcdef0123456789abcdef-0123456789abcdef-01";
const STATUS: &str = "prepared-status";

const PREPARED_EVENTS_API_SOURCE_JSON: &str = r#"{
  "format": "trellis.api.v1",
  "id": "trellis.integration.prepared-events-rust@v1",
  "version": "1.0.0",
  "displayName": "Trellis Rust Prepared Events",
  "description": "Publishes and consumes prepared events for Rust integration parity.",
  "capabilities": {
    "publishEvents": {"allows": [{"target": {"kind": "apiSurface", "api": "trellis.integration.prepared-events-rust@v1", "surface": "event", "name": "Entity.Changed"}, "action": "publish"}]},
    "readEvents": {"allows": [
      {"target": {"kind": "apiSurface", "api": "trellis.integration.prepared-events-rust@v1", "surface": "event", "name": "Entity.Changed"}, "action": "subscribe"}
    ]}
  },
  "schemas": {
    "EntityChanged": {
      "type": "object",
      "required": ["id", "value"],
      "properties": {
        "id": { "type": "string" },
        "value": { "type": "string" },
        "header": { "type": "string" }
      }
    }
  },
  "events": {
    "Entity.Changed": {
      "version": "v1",
      "event": { "schema": "EntityChanged" }
    }
  }
}"#;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EntityChangedEvent {
    id: String,
    value: String,
    header: String,
}

struct EntityChanged;

struct PreparedEventsContract;

struct PreparedEventsListenerContract;

impl EventDescriptor for EntityChanged {
    type Event = EntityChangedEvent;

    const KEY: &'static str = "Entity.Changed";
    const SUBJECT: &'static str = "events.v1.Entity.Changed";
    const PUBLISH_CAPABILITIES: &'static [&'static str] = &["publishEvents"];
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &["readEvents"];
}

#[tokio::test]
async fn prepared_events_prepared_publish_preserves_custom_headers_and_annotates_handler_error() {
    assert_runtime_case_registered(CASE_ID, "prepared-events", "prepared_events");

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();
    let contract = trellis_test::TrellisTestContract::from_native_api_json(
        "trellis.integration.prepared-events-rust@v1",
        PREPARED_EVENTS_API_SOURCE_JSON,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build prepared-events contract");
    let service_key = admin
        .provision_service_instance(&bootstrap_url, &contract, None, None)
        .await
        .expect("provision prepared-events service instance");
    let service = trellis_test::connect_service_runtime::<PreparedEventsContract>(
        runtime.trellis_url(),
        &service_key,
    )
    .await
    .expect("connect prepared-events service runtime");
    let listener_contract =
        trellis_test::TrellisTestContract::from_builder_with_referenced_contracts(
            trellis_rs::contracts::ContractBuilder::authoring(
                "trellis.integration.prepared-events-listener-rust@v1",
                "trellis.integration.prepared-events-listener-rust@v1",
                "1.0.0",
                "Trellis Rust Prepared Events Listener",
                "Consumes prepared events with explicit authority.",
                trellis_rs::contracts::ContractKind::Service,
            )
            .use_ref(
                "preparedEvents",
                trellis_rs::contracts::use_contract("trellis.integration.prepared-events-rust@v1")
                    .with_event_subscribe(["Entity.Changed"]),
            ),
            &[&contract],
        )
        .expect("build prepared-events listener contract");
    let listener_key = admin
        .provision_service_instance(
            &bootstrap_url,
            &listener_contract,
            Some("prepared-events-listener"),
            None,
        )
        .await
        .expect("provision prepared-events listener instance");
    let listener_service = trellis_test::connect_service_runtime::<PreparedEventsListenerContract>(
        runtime.trellis_url(),
        &listener_key,
    )
    .await
    .expect("connect prepared-events listener runtime");

    let raw_subject = EntityChanged::SUBJECT;
    let mut raw_observer = async_nats::ConnectOptions::new()
        .credentials_file(runtime.workdir().join("nats/creds/trellis-auth.creds"))
        .await
        .expect("load prepared-event observer credentials")
        .connect(runtime.nats_url())
        .await
        .expect("connect prepared-event observer")
        .subscribe(raw_subject)
        .await
        .expect("subscribe raw observer");
    let observed = Arc::new(Mutex::new(
        None::<(EntityChangedEvent, ServiceEventListenerContext)>,
    ));
    let handler_observed = Arc::clone(&observed);
    let listener = listener_service
        .listen_event_with_api_id::<EntityChanged, _, _>(
            "trellis.integration.prepared-events-rust@v1",
            move |event, context| {
                let handler_observed = Arc::clone(&handler_observed);
                async move {
                    *handler_observed.lock().await = Some((event, context));
                    Err(ServerError::Nats("prepared handler denied".to_string()))
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Ephemeral,
                group: None,
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start prepared-events listener");

    let payload = EntityChangedEvent {
        id: "entity-prepared-rust-1".to_string(),
        value: "prepared".to_string(),
        header: "payload-header-value".to_string(),
    };
    let mut headers = HeaderMap::new();
    headers.insert("status", STATUS);
    headers.insert("traceparent", TRACEPARENT);
    let prepared = trellis_rs::client::prepare_event::<EntityChanged>(&payload)
        .expect("prepare event")
        .with_headers(headers);
    wait_for_publisher_context(&service, &listener_service).await;

    service
        .event_publisher()
        .publish_prepared(&prepared)
        .await
        .expect("publish prepared event");

    let (observed_event, observed_context) = wait_for_observed(&observed).await;
    let listener_error = tokio::time::timeout(Duration::from_secs(1), listener)
        .await
        .expect("listener returns handler error")
        .expect("listener task joins")
        .expect_err("handler error is surfaced");

    let raw = tokio::time::timeout(Duration::from_secs(5), raw_observer.next())
        .await
        .expect("raw observer receives event")
        .expect("raw observer message");
    let raw_headers = raw.headers.as_ref().expect("raw event headers");
    assert_eq!(
        raw_headers.get("status").map(|value| value.as_str()),
        Some(STATUS)
    );
    assert_eq!(
        raw_headers.get("traceparent").map(|value| value.as_str()),
        Some(TRACEPARENT)
    );
    assert_eq!(
        raw_headers.get("Nats-Msg-Id").map(|value| value.as_str()),
        Some(prepared.event_id())
    );
    assert_eq!(
        raw_headers
            .get("Trellis-Event-Time")
            .map(|value| value.as_str()),
        Some(prepared.event_time())
    );
    assert_eq!(raw.payload.as_ref(), prepared.payload());

    assert_eq!(observed_event, payload);
    assert_eq!(observed_context.id.as_deref(), Some(prepared.event_id()));
    assert_eq!(
        observed_context.time.as_deref(),
        Some(prepared.event_time())
    );
    assert_eq!(observed_context.traceparent.as_deref(), Some(TRACEPARENT));
    assert_eq!(
        observed_context
            .headers
            .get("status")
            .map(|value| value.as_str()),
        Some(STATUS)
    );

    match listener_error {
        ServiceRuntimeError::EventHandler { source, context } => {
            assert!(
                matches!(*source, ServerError::Nats(message) if message == "prepared handler denied")
            );
            assert_eq!(context.id.as_deref(), Some(prepared.event_id()));
            assert_eq!(context.time.as_deref(), Some(prepared.event_time()));
            assert_eq!(context.traceparent.as_deref(), Some(TRACEPARENT));
            assert_eq!(
                context.headers.get("status").map(|value| value.as_str()),
                Some(STATUS)
            );
        }
        other => panic!("expected annotated event handler error, got {other:?}"),
    }
}

async fn wait_for_publisher_context(
    publisher: &trellis_rs::service::ConnectedServiceRuntime<PreparedEventsContract>,
    listener: &trellis_rs::service::ConnectedServiceRuntime<PreparedEventsListenerContract>,
) {
    let digest = publisher.integration_test_authorization_context_digest();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if listener
            .integration_test_resolve_authorization_context(&digest)
            .await
            .is_ok()
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for publisher authorization context"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_for_observed(
    observed: &Arc<Mutex<Option<(EntityChangedEvent, ServiceEventListenerContext)>>>,
) -> (EntityChangedEvent, ServiceEventListenerContext) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(value) = observed.lock().await.clone() {
            return value;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for observed event"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
