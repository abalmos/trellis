use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::support::assertions::{assert_case_registered, assert_generated_service_contract};

fn events_service_contract() -> trellis_test::TrellisTestContract {
    let artifacts = trellis_rs::contracts::ContractBuilder::authoring(
        "trellis.integration.events-service@v1",
        "trellis.integration.events-service@v1",
        "Trellis Integration Events Service",
        "Exercises generated event publish and subscribe surfaces.",
        trellis_rs::contracts::ContractKind::Service,
    )
    .capability(
        "publishRecords",
        trellis_rs::contracts::ContractCapabilityMetadata {
            display_name: "Publish records".to_string(),
            description: "Publish entity change records in the events fixture.".to_string(),
            consequence: None,
        },
    )
    .capability(
        "readRecords",
        trellis_rs::contracts::ContractCapabilityMetadata {
            display_name: "Read records".to_string(),
            description: "Subscribe to entity change records in the events fixture.".to_string(),
            consequence: None,
        },
    )
    .schema(
        "EntityChanged",
        serde_json::json!({
            "type": "object",
            "required": ["id", "value"],
            "properties": {
                "id": {"type": "string"},
                "value": {"type": "string"}
            }
        }),
    )
    .event(
        "Entity.Changed",
        trellis_rs::contracts::event("v1", "events.v1.Entity.Changed", "EntityChanged")
            .with_publish_capabilities(["publishRecords"])
            .with_subscribe_capabilities(["readRecords"]),
    )
    .build()
    .expect("build events service artifacts");
    trellis_test::TrellisTestContract::from_artifacts(artifacts)
        .expect("build events service test contract")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EntityChangedEvent {
    id: String,
    value: String,
}

struct EntityChangedEventDescriptor;

impl trellis_rs::client::EventDescriptor for EntityChangedEventDescriptor {
    type Event = EntityChangedEvent;

    const KEY: &'static str = "Entity.Changed";
    const SUBJECT: &'static str = "events.v1.Entity.Changed";
    const PUBLISH_CAPABILITIES: &'static [&'static str] = &["publishRecords"];
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &["readRecords"];
}

struct EventsServiceContract;

impl trellis_rs::service::GeneratedServiceContract for EventsServiceContract {
    const PARTICIPANT_ID: &'static str = "trellis.integration.events-service@v1";
    const CONTRACT_DIGEST: &'static str = "CUUoOdZ5fDKrzwiZ3MiJtHXeHUtsR6Y5byzKxIzkU5Q";
    const PARTICIPANT_NEEDS_DIGEST: &'static str = "5YkXLSCAdq4DVcQgLlDO-9_CavqUycwSwRMZn8rfw3c";
    const PARTICIPANT_JSON: &'static str = r#"{"description":"Exercises generated event publish and subscribe surfaces.","displayName":"Trellis Integration Events Service","format":"trellis.participant.v1","id":"trellis.integration.events-service@v1","implements":{"self":{"api":"trellis.integration.events-service@v1","apiDigest":"gqjD1EeOzOEfgtqr3-UY0U0dlJOJyFV7kLdZEBDDT6w"}},"kind":"service","schemas":{"EntityChanged":{"properties":{"id":{"type":"string"},"value":{"type":"string"}},"required":["id","value"],"type":"object"}}}"#;
    const API_JSON: &'static str = r#"{"capabilities":{"trellis.integration.events-service::publishRecords":{"allows":[{"action":"publish","target":{"api":"trellis.integration.events-service@v1","kind":"apiSurface","name":"Entity.Changed","surface":"event"}}]},"trellis.integration.events-service::readRecords":{"allows":[{"action":"subscribe","target":{"api":"trellis.integration.events-service@v1","kind":"apiSurface","name":"Entity.Changed","surface":"event"}}]}},"consent":{"trellis.integration.events-service::publishRecords":{"consequence":"","description":"Publish entity change records in the events fixture.","title":"Publish records"},"trellis.integration.events-service::readRecords":{"consequence":"","description":"Subscribe to entity change records in the events fixture.","title":"Read records"}},"description":"Exercises generated event publish and subscribe surfaces.","displayName":"Trellis Integration Events Service","events":{"Entity.Changed":{"event":{"schema":"EntityChanged"},"version":"v1"}},"format":"trellis.api.v1","id":"trellis.integration.events-service@v1","schemas":{"EntityChanged":{"properties":{"id":{"type":"string"},"value":{"type":"string"}},"required":["id","value"],"type":"object"}}}"#;
    const API_DIGEST: &'static str = "gqjD1EeOzOEfgtqr3-UY0U0dlJOJyFV7kLdZEBDDT6w";
    const REFERENCED_API_ARTIFACTS: &'static [(&'static str, &'static str)] = &[];
}

#[test]
fn events_service_contract_evidence_is_exact() {
    assert_generated_service_contract::<EventsServiceContract>(&events_service_contract());
}

struct AbortOnDrop<T> {
    handle: Option<JoinHandle<T>>,
}

impl<T> AbortOnDrop<T> {
    fn new(handle: JoinHandle<T>) -> Self {
        Self {
            handle: Some(handle),
        }
    }

    async fn abort_and_wait(mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
            let _ = handle.await;
        }
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        if let Some(handle) = &self.handle {
            handle.abort();
        }
    }
}

#[tokio::test]
async fn events_client_publishes_and_subscriber_receives() {
    assert_case_registered(
        "events.client-publishes-and-subscriber-receives",
        "events",
        "events",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    let service_contract = events_service_contract();

    let pubsub_client_contract = events_pubsub_client_contract(&service_contract)
        .expect("build events pubsub client test contract");

    let service_key = admin
        .provision_service_instance(&bootstrap_url, &service_contract, None, None)
        .await
        .expect("provision live events service instance");

    let service = trellis_rs::service::ConnectedServiceRuntime::<EventsServiceContract>::connect(
        runtime.service_connect_options("events-fixture-service", &service_key),
    )
    .await
    .expect("connect live Rust events service");

    let service_task = AbortOnDrop::new(tokio::spawn(async move { service.run().await }));

    let client = admin
        .connect_client(&bootstrap_url, &pubsub_client_contract)
        .await
        .expect("connect live Rust events pubsub client");
    let mut event_stream = client
        .subscribe::<EntityChangedEventDescriptor>()
        .await
        .expect("subscribe to Entity.Changed events");

    let event = EntityChangedEvent {
        id: "entity-events-1".to_string(),
        value: "published".to_string(),
    };
    client
        .publish::<EntityChangedEventDescriptor>(&event)
        .await
        .expect("publish Entity.Changed event");

    let received = tokio::time::timeout(Duration::from_secs(10), event_stream.next())
        .await
        .expect("event delivery timed out")
        .expect("event stream ended")
        .expect("receive Entity.Changed event");
    service_task.abort_and_wait().await;

    assert_eq!(received, event);

    assert_eq!(
        <EntityChangedEventDescriptor as trellis_rs::client::EventDescriptor>::KEY,
        "Entity.Changed"
    );
    assert_eq!(
        <EntityChangedEventDescriptor as trellis_rs::client::EventDescriptor>::SUBJECT,
        "events.v1.Entity.Changed"
    );
    assert_eq!(
        <EntityChangedEventDescriptor as trellis_rs::client::EventDescriptor>::PUBLISH_CAPABILITIES,
        &["publishRecords"]
    );
    assert_eq!(
        <EntityChangedEventDescriptor as trellis_rs::client::EventDescriptor>::SUBSCRIBE_CAPABILITIES,
        &["readRecords"]
    );
}

#[tokio::test]
async fn events_denies_publish_without_authority() {
    assert_case_registered(
        "events.denies-publish-without-authority",
        "events",
        "events",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    let service_contract = events_service_contract();

    let subscribe_only_client_contract = events_subscribe_only_client_contract(&service_contract)
        .expect("build events subscribe-only client test contract");

    admin
        .provision_service_instance(&bootstrap_url, &service_contract, None, None)
        .await
        .expect("provision live events service instance");

    let client = admin
        .connect_client(&bootstrap_url, &subscribe_only_client_contract)
        .await
        .expect("connect live Rust events subscribe-only client");

    let result = client
        .publish::<EntityChangedEventDescriptor>(&EntityChangedEvent {
            id: "entity-denied-1".to_string(),
            value: "should-not-publish".to_string(),
        })
        .await;

    assert!(
        result.is_err(),
        "expected publish to be denied for subscribe-only client"
    );

    let error = result.unwrap_err();
    eprintln!("denied publish error (expected): {error}");
}

fn events_pubsub_client_contract(
    service_contract: &trellis_test::TrellisTestContract,
) -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractBuilder::authoring(
        "trellis.integration.events-pubsub-client@v1",
        "trellis.integration.events-pubsub-client@v1",
        "Trellis Integration Events PubSub Client",
        "App/client participant with event publish and subscribe authority.",
        trellis_rs::contracts::ContractKind::App,
    )
    .use_ref(
        "eventsService",
        trellis_rs::contracts::use_contract("trellis.integration.events-service@v1")
            .with_event_publish(["Entity.Changed"])
            .with_event_subscribe(["Entity.Changed"]),
    );

    trellis_test::TrellisTestContract::from_builder_with_referenced_contracts(
        manifest,
        &[service_contract],
    )
}

fn events_subscribe_only_client_contract(
    service_contract: &trellis_test::TrellisTestContract,
) -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractBuilder::authoring(
        "trellis.integration.events-subscribe-only-client@v1",
        "trellis.integration.events-subscribe-only-client@v1",
        "Trellis Integration Events Subscribe-Only Client",
        "App/client participant without event publish authority.",
        trellis_rs::contracts::ContractKind::App,
    )
    .use_ref(
        "eventsService",
        trellis_rs::contracts::use_contract("trellis.integration.events-service@v1")
            .with_event_subscribe(["Entity.Changed"]),
    );

    trellis_test::TrellisTestContract::from_builder_with_referenced_contracts(
        manifest,
        &[service_contract],
    )
}

#[tokio::test]
async fn events_denies_subscribe_without_authority() {
    assert_case_registered(
        "events.denies-subscribe-without-authority",
        "events",
        "events",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    let service_contract = events_service_contract();

    let publish_only_client_contract = events_publish_only_client_contract(&service_contract)
        .expect("build events publish-only client test contract");
    let pubsub_client_contract = events_pubsub_client_contract(&service_contract)
        .expect("build events pubsub client test contract");

    admin
        .provision_service_instance(&bootstrap_url, &service_contract, None, None)
        .await
        .expect("provision live events service instance");

    let client = admin
        .connect_client(&bootstrap_url, &publish_only_client_contract)
        .await
        .expect("connect live Rust events publish-only client");

    let subscribe_result = client.subscribe::<EntityChangedEventDescriptor>().await;
    let Ok(mut stream) = subscribe_result else {
        return;
    };

    let publisher = admin
        .connect_client(&bootstrap_url, &pubsub_client_contract)
        .await
        .expect("connect live Rust events pubsub client");
    publisher
        .publish::<EntityChangedEventDescriptor>(&EntityChangedEvent {
            id: "entity-denied-subscribe-1".to_string(),
            value: "should-not-deliver".to_string(),
        })
        .await
        .expect("publish Entity.Changed event");

    if let Ok(Some(Ok(event))) =
        tokio::time::timeout(Duration::from_millis(500), stream.next()).await
    {
        panic!("unauthorized subscriber received event: {event:?}");
    }
}

fn events_publish_only_client_contract(
    service_contract: &trellis_test::TrellisTestContract,
) -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractBuilder::authoring(
        "trellis.integration.events-publish-only-client@v1",
        "trellis.integration.events-publish-only-client@v1",
        "Trellis Integration Events Publish-Only Client",
        "App/client participant without event subscribe authority.",
        trellis_rs::contracts::ContractKind::App,
    )
    .use_ref(
        "eventsService",
        trellis_rs::contracts::use_contract("trellis.integration.events-service@v1")
            .with_event_publish(["Entity.Changed"]),
    );

    trellis_test::TrellisTestContract::from_builder_with_referenced_contracts(
        manifest,
        &[service_contract],
    )
}
