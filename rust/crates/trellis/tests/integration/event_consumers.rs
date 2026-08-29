use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, Mutex, Notify};
use trellis_rs::client::EventDescriptor;
use trellis_rs::service::{
    ConnectedServiceRuntime, ServerError, ServiceEventListenOptions, ServiceEventListenerContext,
    ServiceEventListenerMode, ServiceRuntimeError,
};

use crate::support::assertions::assert_runtime_case_registered;

const SOURCE_CONTRACT_ID: &str = "trellis.integration.event-consumers-source-rust@v1";

struct EventConsumerContract;

const SOURCE_API_SOURCE_JSON: &str = r#"{
  "format": "trellis.api.v1",
  "id": "trellis.integration.event-consumers-source-rust@v1",
  "version": "1.0.0",
  "displayName": "Trellis Rust Event Consumers Source",
  "description": "Publishes source events for Rust durable consumer integration tests.",
  "capabilities": {
    "publishEvents": {"allows": [
      {"target": {"kind": "apiSurface", "api": "trellis.integration.event-consumers-source-rust@v1", "surface": "event", "name": "Source.Pinged"}, "action": "publish"},
      {"target": {"kind": "apiSurface", "api": "trellis.integration.event-consumers-source-rust@v1", "surface": "event", "name": "Source.Ponged"}, "action": "publish"}
    ]},
    "readEvents": {"allows": [
      {"target": {"kind": "apiSurface", "api": "trellis.integration.event-consumers-source-rust@v1", "surface": "event", "name": "Source.Pinged"}, "action": "subscribe"},
      {"target": {"kind": "apiSurface", "api": "trellis.integration.event-consumers-source-rust@v1", "surface": "event", "name": "Source.Ponged"}, "action": "subscribe"}
    ]}
  },
  "schemas": {
    "EventRecord": {
      "type": "object",
      "required": ["id", "value"],
      "properties": {
        "id": { "type": "string" },
        "value": { "type": "string" }
      }
    }
  },
    "events": {
    "Source.Pinged": {
      "version": "v1",
      "event": { "schema": "EventRecord" }
    },
    "Source.Ponged": {
      "version": "v1",
      "event": { "schema": "EventRecord" }
    }
  }
}"#;

const MISSING_GROUP_CONSUMER_JSON: &str = "missing-group";
const AMBIGUOUS_GROUP_CONSUMER_JSON: &str = "ambiguous-group";
const DEPENDENCY_CONSUMER_JSON: &str = "dependency";
const PARALLEL_DEPENDENCY_CONSUMER_JSON: &str = "parallel-dependency";
const GROUPED_DEPENDENCY_CONSUMER_JSON: &str = "grouped-dependency";
const SELF_CONSUMER_JSON: &str = "self";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EventRecord {
    id: String,
    value: String,
}

struct SourcePingedEvent;

impl trellis_rs::client::EventDescriptor for SourcePingedEvent {
    type Event = EventRecord;

    const KEY: &'static str = "Source.Pinged";
    const SUBJECT: &'static str = "events.v1.Source.Pinged";
    const PUBLISH_CAPABILITIES: &'static [&'static str] = &["publishEvents"];
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &["readEvents"];
}

struct SelfPingedEvent;

impl trellis_rs::client::EventDescriptor for SelfPingedEvent {
    type Event = EventRecord;

    const KEY: &'static str = "Self.Pinged";
    const SUBJECT: &'static str = "events.v1.Self.Pinged";
    const PUBLISH_CAPABILITIES: &'static [&'static str] = &[];
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &[];
}

struct SourcePongedEvent;

impl trellis_rs::client::EventDescriptor for SourcePongedEvent {
    type Event = EventRecord;

    const KEY: &'static str = "Source.Ponged";
    const SUBJECT: &'static str = "events.v1.Source.Ponged";
    const PUBLISH_CAPABILITIES: &'static [&'static str] = &["publishEvents"];
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &["readEvents"];
}

struct SelfPongedEvent;

impl trellis_rs::client::EventDescriptor for SelfPongedEvent {
    type Event = EventRecord;

    const KEY: &'static str = "Self.Ponged";
    const SUBJECT: &'static str = "events.v1.Self.Ponged";
    const PUBLISH_CAPABILITIES: &'static [&'static str] = &[];
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &[];
}

#[tokio::test]
async fn event_consumers_durable_listen_without_declared_group_returns_err() {
    assert_runtime_case_registered(
        "event-consumers.durable-listen-without-declared-group-returns-err",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        MISSING_GROUP_CONSUMER_JSON,
        "event-consumers-missing-group-rust",
    )
    .await;

    let result = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            |_event, _context| async { Ok(()) },
            ServiceEventListenOptions::default(),
        )
        .await;

    let expected_subject = SourcePingedEvent::SUBJECT;
    assert!(
        matches!(
            result,
            Err(ServiceRuntimeError::MissingEventConsumerGroup { ref subject })
                if subject == expected_subject
        ),
        "expected missing group error, got {result:?}"
    );
}

#[tokio::test]
async fn event_consumers_parallel_group_runs_messages_concurrently() {
    assert_runtime_case_registered(
        "event-consumers.parallel-group-runs-messages-concurrently",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        PARALLEL_DEPENDENCY_CONSUMER_JSON,
        "event-consumers-parallel-dependency-rust",
    )
    .await;

    let first_started = Arc::new(Notify::new());
    let second_started = Arc::new(Notify::new());
    let release_first = Arc::new(Notify::new());
    let first_finished = Arc::new(Notify::new());
    let listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            {
                let first_started = Arc::clone(&first_started);
                let second_started = Arc::clone(&second_started);
                let release_first = Arc::clone(&release_first);
                let first_finished = Arc::clone(&first_finished);
                move |event, context| {
                    let first_started = Arc::clone(&first_started);
                    let second_started = Arc::clone(&second_started);
                    let release_first = Arc::clone(&release_first);
                    let first_finished = Arc::clone(&first_finished);
                    async move {
                        assert_eq!(context.group.as_deref(), Some("ingest"));
                        match event.value.as_str() {
                            "first" => {
                                first_started.notify_one();
                                release_first.notified().await;
                                first_finished.notify_one();
                            }
                            "second" => second_started.notify_one(),
                            value => panic!("unexpected parallel fixture value {value}"),
                        }
                        Ok(())
                    }
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_string()),
                durable_name: None,
                concurrency: 2,
            },
        )
        .await
        .expect("start parallel durable listener");
    wait_for_waiting_count(
        &runtime,
        SourcePingedEvent::SUBJECT,
        bound_name(&consumer, "ingest"),
        2,
    )
    .await;

    let conflicting = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            |_event, _context| async { Ok(()) },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await;
    assert!(matches!(
        conflicting,
        Err(ServiceRuntimeError::EventListenerConcurrencyMismatch {
            ref group,
            existing: 2,
            requested: 1,
        }) if group == "ingest"
    ));

    let publisher_contract = publisher_contract();
    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract)
        .await
        .expect("connect event publisher client");
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-parallel-first".to_string(),
            value: "first".to_string(),
        })
        .await
        .expect("publish first parallel event");
    tokio::time::timeout(Duration::from_secs(5), first_started.notified())
        .await
        .expect("first parallel handler did not start");

    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-parallel-second".to_string(),
            value: "second".to_string(),
        })
        .await
        .expect("publish second parallel event");
    tokio::time::timeout(Duration::from_secs(5), second_started.notified())
        .await
        .expect("second handler did not start while first was blocked");

    release_first.notify_one();
    tokio::time::timeout(Duration::from_secs(5), first_finished.notified())
        .await
        .expect("first parallel handler did not finish");
    listener.abort();
    let _ = listener.await;
}

#[tokio::test]
async fn event_consumers_handler_failure_redelivers_same_event() {
    assert_runtime_case_registered(
        "event-consumers.handler-failure-redelivers-same-event",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        PARALLEL_DEPENDENCY_CONSUMER_JSON,
        "event-consumers-redelivery-rust",
    )
    .await;

    let attempts = Arc::new(AtomicUsize::new(0));
    let succeeded = Arc::new(Notify::new());
    let listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            {
                let attempts = Arc::clone(&attempts);
                let succeeded = Arc::clone(&succeeded);
                move |event, _context| {
                    let attempts = Arc::clone(&attempts);
                    let succeeded = Arc::clone(&succeeded);
                    async move {
                        assert_eq!(event.id, "rust-event-consumers-redelivery");
                        if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                            return Err(ServerError::Nats("fail once".to_owned()));
                        }
                        succeeded.notify_one();
                        Ok(())
                    }
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_owned()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start redelivery listener");
    wait_for_waiting_count(
        &runtime,
        SourcePingedEvent::SUBJECT,
        bound_name(&consumer, "ingest"),
        1,
    )
    .await;

    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract())
        .await
        .expect("connect event publisher client");
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-redelivery".to_owned(),
            value: "redeliver".to_owned(),
        })
        .await
        .expect("publish redelivery event");
    tokio::time::timeout(Duration::from_secs(10), succeeded.notified())
        .await
        .expect("failed event was not redelivered");
    assert_eq!(attempts.load(Ordering::SeqCst), 2);

    listener.abort();
    let _ = listener.await;
}

#[tokio::test]
async fn event_consumers_invalid_authorization_proof_terms() {
    assert_runtime_case_registered(
        "event-consumers.invalid-authorization-proof-terms",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        PARALLEL_DEPENDENCY_CONSUMER_JSON,
        "event-consumers-invalid-auth-proof-rust",
    )
    .await;
    let ack_observer = runtime
        .start_jetstream_ack_observer()
        .await
        .expect("start JetStream ACK observer");
    let observed = Arc::new(Mutex::new(Vec::<String>::new()));
    let listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            {
                let observed = Arc::clone(&observed);
                move |event, _context| {
                    let observed = Arc::clone(&observed);
                    async move {
                        observed.lock().await.push(event.id);
                        Ok(())
                    }
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_owned()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start authorization listener");
    wait_for_waiting_count(
        &runtime,
        SourcePingedEvent::SUBJECT,
        bound_name(&consumer, "ingest"),
        1,
    )
    .await;
    let durable = matching_named_consumers(
        &runtime,
        SourcePingedEvent::SUBJECT,
        bound_name(&consumer, "ingest"),
    )
    .await
    .remove(0);
    let durable_name = consumer_name(&durable).to_owned();
    let mut raw_events = consumer
        .integration_test_nats()
        .subscribe(durable.filter_subjects[0].clone())
        .await
        .expect("subscribe to raw test events");

    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract())
        .await
        .expect("connect event publisher");
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-auth-valid".into(),
            value: "valid".into(),
        })
        .await
        .expect("publish valid signed event");
    let raw = tokio::time::timeout(Duration::from_secs(5), raw_events.next())
        .await
        .expect("timed out waiting for raw event")
        .expect("raw event subscription ended");
    wait_for_observed_vec_id(&observed, "rust-event-auth-valid").await;

    let mut headers = raw.headers.expect("published event headers");
    headers.insert("proof", "invalid-event-proof");
    headers.insert("Nats-Msg-Id", format!("evt_invalid_{}", ulid::Ulid::new()));
    publisher
        .integration_test_nats()
        .publish_with_headers(raw.subject, headers, raw.payload)
        .await
        .expect("publish cryptographically invalid event");
    wait_for_ack_payload(&ack_observer, &durable_name, "+TERM").await;
    assert_eq!(
        observed.lock().await.clone(),
        vec!["rust-event-auth-valid".to_owned()]
    );

    listener.abort();
    let _ = listener.await;
    ack_observer.stop().await;
}

async fn wait_for_observed_vec_id(observed: &Arc<Mutex<Vec<String>>>, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        if observed.lock().await.iter().any(|id| id == expected) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "event {expected} was not processed"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_for_ack_payload(
    observer: &trellis_test::TrellisJetStreamAckObserver,
    consumer: &str,
    payload: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if observer.frames().iter().any(|frame| {
            frame.subject.contains(consumer)
                && (frame.payload == payload
                    || (payload == "-NAK" && frame.payload.starts_with("-NAK")))
        }) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "consumer {consumer} did not emit {payload}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[tokio::test]
async fn event_consumers_strict_group_rejects_parallel_workers() {
    assert_runtime_case_registered(
        "event-consumers.strict-group-rejects-parallel-workers",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        DEPENDENCY_CONSUMER_JSON,
        "event-consumers-dependency-rust",
    )
    .await;

    let result = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            |_event, _context| async { Ok(()) },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_string()),
                durable_name: None,
                concurrency: 2,
            },
        )
        .await;
    assert!(matches!(
        result,
        Err(ServiceRuntimeError::StrictEventListenerConcurrency { ref group })
            if group == "ingest"
    ));
}

#[tokio::test]
async fn event_consumers_ambiguous_group_without_opts_group_returns_err_and_specifying_group_works()
{
    assert_runtime_case_registered(
        "event-consumers.ambiguous-group-without-opts-group-returns-err-and-specifying-group-works",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        AMBIGUOUS_GROUP_CONSUMER_JSON,
        "event-consumers-ambiguous-group-rust",
    )
    .await;

    let ambiguous = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            |_event, _context| async { Ok(()) },
            ServiceEventListenOptions::default(),
        )
        .await;
    let expected_subject = SourcePingedEvent::SUBJECT;
    assert!(
        matches!(
            ambiguous,
            Err(ServiceRuntimeError::AmbiguousEventConsumerGroup { ref subject, ref groups })
                if subject == expected_subject
                    && groups.as_slice() == ["primary", "secondary"]
        ),
        "expected ambiguous group error, got {ambiguous:?}"
    );

    let observed = Arc::new(Mutex::new(
        None::<(EventRecord, ServiceEventListenerContext)>,
    ));
    let handler_observed = Arc::clone(&observed);
    let listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            move |event, context| {
                let handler_observed = Arc::clone(&handler_observed);
                async move {
                    *handler_observed.lock().await = Some((event, context));
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("primary".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start explicit primary listener");

    let publisher_contract = publisher_contract();
    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract)
        .await
        .expect("connect event publisher client");
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-primary".to_string(),
            value: "primary".to_string(),
        })
        .await
        .expect("publish source event");

    wait_for_observed(&observed, "rust-event-consumers-primary", Some("primary")).await;
    listener.abort();
    let _ = listener.await;
}

#[tokio::test]
async fn event_consumers_caller_provided_durable_name_returns_err() {
    assert_runtime_case_registered(
        "event-consumers.caller-provided-durable-name-returns-err",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        DEPENDENCY_CONSUMER_JSON,
        "event-consumers-dependency-rust",
    )
    .await;

    let result = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            |_event, _context| async { Ok(()) },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: None,
                durable_name: Some("caller-name".to_string()),
                concurrency: 1,
            },
        )
        .await;

    assert!(
        matches!(
            result,
            Err(ServiceRuntimeError::CallerDurableName { ref durable_name })
                if durable_name == "caller-name"
        ),
        "expected caller durable name error, got {result:?}"
    );
}

#[tokio::test]
async fn event_consumers_bound_dependency_consumer_uses_trellis_provisioned_consumer_only() {
    assert_runtime_case_registered(
        "event-consumers.bound-dependency-consumer-uses-trellis-provisioned-consumer-only",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        DEPENDENCY_CONSUMER_JSON,
        "event-consumers-dependency-rust",
    )
    .await;
    let binding = consumer
        .resources()
        .event_consumers
        .get("ingest")
        .expect("ingest event consumer binding");
    let before = matching_named_consumers(
        &runtime,
        SourcePingedEvent::SUBJECT,
        bound_name(&consumer, "ingest"),
    )
    .await;
    assert_eq!(before.len(), 1);
    assert_eq!(consumer_name(&before[0]), binding.consumer_name);

    let observed = Arc::new(Mutex::new(
        None::<(EventRecord, ServiceEventListenerContext)>,
    ));
    let handler_observed = Arc::clone(&observed);
    let listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            move |event, context| {
                let handler_observed = Arc::clone(&handler_observed);
                async move {
                    *handler_observed.lock().await = Some((event, context));
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start bound dependency listener");
    let after = matching_named_consumers(
        &runtime,
        SourcePingedEvent::SUBJECT,
        bound_name(&consumer, "ingest"),
    )
    .await;
    assert_eq!(after.len(), 1);
    assert_eq!(consumer_name(&after[0]), consumer_name(&before[0]));

    let publisher_contract = publisher_contract();
    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract)
        .await
        .expect("connect event publisher client");
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-bound".to_string(),
            value: "bound".to_string(),
        })
        .await
        .expect("publish source event");

    wait_for_observed(&observed, "rust-event-consumers-bound", Some("ingest")).await;
    listener.abort();
    let _ = listener.await;
}

#[tokio::test]
async fn event_consumers_transient_missing_consumer_retries_after_reconcile() {
    assert_runtime_case_registered(
        "event-consumers.transient-missing-consumer-retries-after-reconcile",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer_contract = test_contract(DEPENDENCY_CONSUMER_JSON);
    let instance_name = runtime.integration_name("event-consumers-dependency-rust");
    let consumer_key = admin
        .provision_service_instance(
            &bootstrap_url,
            &consumer_contract,
            Some(&instance_name),
            None,
        )
        .await
        .expect("provision event consumer service instance");
    let deployment_id = consumer_key.deployment_id.clone();
    let consumer = trellis_test::connect_service_runtime::<EventConsumerContract>(
        runtime.trellis_url(),
        &consumer_key,
    )
    .await
    .expect("connect event consumer service runtime");
    let before = matching_named_consumers(
        &runtime,
        SourcePingedEvent::SUBJECT,
        bound_name(&consumer, "ingest"),
    )
    .await;
    assert_eq!(before.len(), 1);
    assert!(
        runtime
            .delete_trellis_jetstream_consumer(consumer_name(&before[0]))
            .await
            .expect("delete Trellis JetStream consumer"),
        "expected provisioned consumer to be deleted"
    );
    wait_for_matching_consumer_count(
        &runtime,
        SourcePingedEvent::SUBJECT,
        bound_name(&consumer, "ingest"),
        0,
    )
    .await;

    let observed = Arc::new(Mutex::new(
        None::<(EventRecord, ServiceEventListenerContext)>,
    ));
    let handler_observed = Arc::clone(&observed);
    let listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            move |event, context| {
                let handler_observed = Arc::clone(&handler_observed);
                async move {
                    *handler_observed.lock().await = Some((event, context));
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start missing durable listener");

    admin
        .reconcile(&bootstrap_url, &deployment_id)
        .await
        .expect("reconcile test deployment");
    admin
        .wait_ready(&bootstrap_url, &deployment_id)
        .await
        .expect("wait for test deployment ready");
    wait_for_matching_consumer_count(
        &runtime,
        SourcePingedEvent::SUBJECT,
        bound_name(&consumer, "ingest"),
        1,
    )
    .await;
    wait_for_waiting_count(
        &runtime,
        SourcePingedEvent::SUBJECT,
        bound_name(&consumer, "ingest"),
        1,
    )
    .await;

    let publisher_contract = publisher_contract();
    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract)
        .await
        .expect("connect event publisher client");
    let publisher_context = publisher
        .integration_test_authorization_context_digest()
        .expect("read publisher authorization context");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if consumer
            .integration_test_resolve_authorization_context(&publisher_context)
            .await
            .is_ok()
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for publisher authorization context"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-transient-missing".to_string(),
            value: "recovered".to_string(),
        })
        .await
        .expect("publish recovered source event");

    wait_for_observed(
        &observed,
        "rust-event-consumers-transient-missing",
        Some("ingest"),
    )
    .await;
    listener.abort();
    let _ = listener.await;
}

#[tokio::test]
async fn event_consumers_readiness_lost_does_not_nak_delivered_group_message() {
    assert_runtime_case_registered(
        "event-consumers.readiness-lost-does-not-nak-delivered-group-message",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        GROUPED_DEPENDENCY_CONSUMER_JSON,
        "event-consumers-grouped-dependency-rust",
    )
    .await;
    let ack_observer = runtime
        .start_jetstream_ack_observer()
        .await
        .expect("start JetStream ACK observer");

    let observed_ping = Arc::new(Mutex::new(None::<String>));
    let handler_observed_ping = Arc::clone(&observed_ping);
    let ping_listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            move |event, context| {
                let handler_observed_ping = Arc::clone(&handler_observed_ping);
                async move {
                    assert_eq!(context.group.as_deref(), Some("paired"));
                    *handler_observed_ping.lock().await = Some(event.id);
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("paired".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start grouped ping listener");

    let observed_pong = Arc::new(Mutex::new(None::<String>));
    let handler_observed_pong = Arc::clone(&observed_pong);
    let (handler_started_tx, handler_started_rx) = oneshot::channel::<()>();
    let handler_started_tx = Arc::new(Mutex::new(Some(handler_started_tx)));
    let (release_handler_tx, release_handler_rx) = oneshot::channel::<()>();
    let release_handler_rx = Arc::new(Mutex::new(Some(release_handler_rx)));
    let pong_listener = consumer
        .listen_event_with_api_id::<SourcePongedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            move |event, context| {
                let handler_observed_pong = Arc::clone(&handler_observed_pong);
                let handler_started_tx = Arc::clone(&handler_started_tx);
                let release_handler_rx = Arc::clone(&release_handler_rx);
                async move {
                    assert_eq!(context.group.as_deref(), Some("paired"));
                    if let Some(sender) = handler_started_tx.lock().await.take() {
                        let _ = sender.send(());
                    }
                    if let Some(receiver) = release_handler_rx.lock().await.take() {
                        let _ = receiver.await;
                    }
                    *handler_observed_pong.lock().await = Some(event.id);
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("paired".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start grouped pong listener");
    wait_for_matching_grouped_waiting_count(
        &runtime,
        SourcePingedEvent::SUBJECT,
        SourcePongedEvent::SUBJECT,
        bound_name(&consumer, "paired"),
        1,
    )
    .await;
    let grouped_consumers = matching_named_grouped_consumers(
        &runtime,
        SourcePingedEvent::SUBJECT,
        SourcePongedEvent::SUBJECT,
        bound_name(&consumer, "paired"),
    )
    .await;
    let durable_name = consumer_name(&grouped_consumers[0]).to_string();

    let publisher_contract = publisher_contract();
    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract)
        .await
        .expect("connect event publisher client");
    publisher
        .publish::<SourcePongedEvent>(&EventRecord {
            id: "rust-event-consumers-readiness-lost".to_string(),
            value: "readiness-lost".to_string(),
        })
        .await
        .expect("publish grouped source event");
    tokio::time::timeout(Duration::from_secs(5), handler_started_rx)
        .await
        .expect("timed out waiting for grouped handler to start")
        .expect("grouped handler started");

    ping_listener.abort();
    let _ = ping_listener.await;
    let _ = release_handler_tx.send(());
    wait_for_matching_grouped_ack_pending_and_waiting_count(
        &runtime,
        SourcePingedEvent::SUBJECT,
        SourcePongedEvent::SUBJECT,
        bound_name(&consumer, "paired"),
        1,
        0,
    )
    .await;

    let ack_frames = ack_observer
        .frames()
        .into_iter()
        .filter(|frame| frame.subject.contains(&durable_name))
        .collect::<Vec<_>>();
    assert_eq!(ack_observer.errors(), Vec::<String>::new());
    assert!(
        !ack_frames.iter().any(|frame| frame.payload == "-NAK"),
        "readiness-loss cleanup NAKed delivered message: {ack_frames:?}"
    );
    assert_eq!(*observed_ping.lock().await, None);
    assert_eq!(
        observed_pong.lock().await.as_deref(),
        Some("rust-event-consumers-readiness-lost")
    );

    pong_listener.abort();
    let _ = pong_listener.await;
    ack_observer.stop().await;
}

#[tokio::test]
async fn event_consumers_ephemeral_listener_avoids_durable_metadata_and_jetstream_consumer() {
    assert_runtime_case_registered(
        "event-consumers.ephemeral-listener-avoids-durable-metadata-and-jetstream-consumer",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        MISSING_GROUP_CONSUMER_JSON,
        "event-consumers-missing-group-rust",
    )
    .await;
    let durable_count = matching_consumers(&runtime, SourcePingedEvent::SUBJECT)
        .await
        .len();

    let observed = Arc::new(Mutex::new(Vec::<String>::new()));
    let handler_observed = Arc::clone(&observed);
    let listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            move |event, context| {
                let handler_observed = Arc::clone(&handler_observed);
                async move {
                    assert_eq!(context.mode, ServiceEventListenerMode::Ephemeral);
                    assert_eq!(context.group, None);
                    handler_observed.lock().await.push(event.id);
                    Ok(())
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
        .expect("start ephemeral listener");
    assert_eq!(
        matching_consumers(&runtime, SourcePingedEvent::SUBJECT)
            .await
            .len(),
        durable_count
    );

    let publisher_contract = publisher_contract();
    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract)
        .await
        .expect("connect event publisher client");
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-ephemeral".to_string(),
            value: "ephemeral".to_string(),
        })
        .await
        .expect("publish source event");

    wait_for_observed_entry(&observed, "rust-event-consumers-ephemeral").await;
    assert_eq!(
        matching_consumers(&runtime, SourcePingedEvent::SUBJECT)
            .await
            .len(),
        durable_count
    );
    drop(listener);

    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-ephemeral-after-drop".to_string(),
            value: "ephemeral-after-drop".to_string(),
        })
        .await
        .expect("publish source event after listener drop");

    assert_no_observed_entry(&observed, "rust-event-consumers-ephemeral-after-drop").await;
}

#[tokio::test]
async fn event_consumers_duplicate_handlers_share_single_group_waiter() {
    assert_runtime_case_registered(
        "event-consumers.duplicate-handlers-share-single-group-waiter",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        DEPENDENCY_CONSUMER_JSON,
        "event-consumers-dependency-rust",
    )
    .await;

    let observed = Arc::new(Mutex::new(Vec::<String>::new()));
    let first_observed = Arc::clone(&observed);
    let first_listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            move |event, context| {
                let first_observed = Arc::clone(&first_observed);
                async move {
                    assert_eq!(context.group.as_deref(), Some("ingest"));
                    first_observed
                        .lock()
                        .await
                        .push(format!("first:{}", event.id));
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start first duplicate listener");
    let second_observed = Arc::clone(&observed);
    let second_listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            move |event, context| {
                let second_observed = Arc::clone(&second_observed);
                async move {
                    assert_eq!(context.group.as_deref(), Some("ingest"));
                    second_observed
                        .lock()
                        .await
                        .push(format!("second:{}", event.id));
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start second duplicate listener");

    wait_for_waiting_count(
        &runtime,
        SourcePingedEvent::SUBJECT,
        bound_name(&consumer, "ingest"),
        1,
    )
    .await;

    let publisher_contract = publisher_contract();
    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract)
        .await
        .expect("connect event publisher client");
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-duplicate".to_string(),
            value: "duplicate".to_string(),
        })
        .await
        .expect("publish source event");

    wait_for_duplicate_observed(&observed, "rust-event-consumers-duplicate").await;
    first_listener.abort();
    second_listener.abort();
    let _ = first_listener.await;
    let _ = second_listener.await;
}

#[tokio::test]
async fn event_consumers_self_owned_durable_consumer_receives_self_published_event() {
    assert_runtime_case_registered(
        "event-consumers.self-owned-durable-consumer-receives-self-published-event",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let service = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        SELF_CONSUMER_JSON,
        "event-consumers-self-rust",
    )
    .await;
    let binding = service
        .resources()
        .event_consumers
        .get("ingest")
        .expect("ingest self event consumer binding");
    let before = matching_consumers(&runtime, SelfPingedEvent::SUBJECT).await;
    assert!(
        before
            .iter()
            .any(|consumer| consumer_name(consumer) == binding.consumer_name),
        "binding={binding:?} consumers={before:?}"
    );

    let observed = Arc::new(Mutex::new(
        None::<(EventRecord, ServiceEventListenerContext)>,
    ));
    let handler_observed = Arc::clone(&observed);
    let listener = service
        .listen_event::<SelfPingedEvent, _, _>(
            move |event, context| {
                let handler_observed = Arc::clone(&handler_observed);
                async move {
                    *handler_observed.lock().await = Some((event, context));
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start self-owned durable listener");
    let after = matching_consumers(&runtime, SelfPingedEvent::SUBJECT).await;
    assert_eq!(after.len(), before.len());
    assert!(after
        .iter()
        .any(|consumer| consumer_name(consumer) == binding.consumer_name));

    let event = EventRecord {
        id: "rust-event-consumers-self".to_string(),
        value: "self".to_string(),
    };
    service
        .event_publisher()
        .publish::<SelfPingedEvent>(&event)
        .await
        .expect("publish self-owned event");

    wait_for_observed(&observed, "rust-event-consumers-self", Some("ingest")).await;
    listener.abort();
    let _ = listener.await;
}

#[tokio::test]
async fn event_consumers_abort_re_register_restarts_delivery() {
    assert_runtime_case_registered(
        "event-consumers.abort-re-register-restarts-delivery",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let service = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        SELF_CONSUMER_JSON,
        "event-consumers-self-rust",
    )
    .await;
    let binding = service
        .resources()
        .event_consumers
        .get("ingest")
        .expect("ingest self event consumer binding");
    let before = matching_consumers(&runtime, SelfPingedEvent::SUBJECT).await;
    assert!(before
        .iter()
        .any(|consumer| consumer_name(consumer) == binding.consumer_name));

    let observed = Arc::new(Mutex::new(Vec::<String>::new()));
    let first_observed = Arc::clone(&observed);
    let first_listener = service
        .listen_event::<SelfPingedEvent, _, _>(
            move |event, context| {
                let first_observed = Arc::clone(&first_observed);
                async move {
                    assert_eq!(context.group.as_deref(), Some("ingest"));
                    first_observed
                        .lock()
                        .await
                        .push(format!("first:{}", event.id));
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start first self-owned durable listener");
    service
        .event_publisher()
        .publish::<SelfPingedEvent>(&EventRecord {
            id: "rust-event-consumers-abort-first".to_string(),
            value: "first".to_string(),
        })
        .await
        .expect("publish first self-owned event");
    wait_for_observed_entry(&observed, "first:rust-event-consumers-abort-first").await;

    first_listener.abort();
    let _ = first_listener.await;
    service
        .event_publisher()
        .publish::<SelfPingedEvent>(&EventRecord {
            id: "rust-event-consumers-abort-second".to_string(),
            value: "second".to_string(),
        })
        .await
        .expect("publish queued self-owned event");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!observed
        .lock()
        .await
        .contains(&"first:rust-event-consumers-abort-second".to_string()));

    let second_observed = Arc::clone(&observed);
    let second_listener = service
        .listen_event::<SelfPingedEvent, _, _>(
            move |event, context| {
                let second_observed = Arc::clone(&second_observed);
                async move {
                    assert_eq!(context.group.as_deref(), Some("ingest"));
                    second_observed
                        .lock()
                        .await
                        .push(format!("second:{}", event.id));
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("re-register self-owned durable listener");

    wait_for_observed_entry(&observed, "second:rust-event-consumers-abort-second").await;
    assert!(!observed
        .lock()
        .await
        .contains(&"first:rust-event-consumers-abort-second".to_string()));
    let after = matching_consumers(&runtime, SelfPingedEvent::SUBJECT).await;
    assert!(after
        .iter()
        .any(|consumer| consumer_name(consumer) == binding.consumer_name));
    drop(second_listener);

    service
        .event_publisher()
        .publish::<SelfPingedEvent>(&EventRecord {
            id: "rust-event-consumers-drop-third".to_string(),
            value: "third".to_string(),
        })
        .await
        .expect("publish queued self-owned event after drop");
    assert_no_observed_entry(&observed, "second:rust-event-consumers-drop-third").await;
}

#[tokio::test]
async fn event_consumers_stop_teardown_stops_durable_delivery() {
    assert_runtime_case_registered(
        "event-consumers.stop-teardown-stops-durable-delivery",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        DEPENDENCY_CONSUMER_JSON,
        "event-consumers-dependency-rust",
    )
    .await;

    let observed = Arc::new(Mutex::new(Vec::<String>::new()));
    let handler_observed = Arc::clone(&observed);
    let listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            move |event, context| {
                let handler_observed = Arc::clone(&handler_observed);
                async move {
                    assert_eq!(context.group.as_deref(), Some("ingest"));
                    handler_observed.lock().await.push(event.id);
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start dependency durable listener");
    let durable_name = bound_name(&consumer, "ingest").to_owned();
    wait_for_waiting_count(&runtime, SourcePingedEvent::SUBJECT, &durable_name, 1).await;

    let publisher_contract = publisher_contract();
    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract)
        .await
        .expect("connect event publisher client");
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-stop-before".to_string(),
            value: "before-stop".to_string(),
        })
        .await
        .expect("publish event before service stop");
    wait_for_observed_entry(&observed, "rust-event-consumers-stop-before").await;

    drop(consumer);
    wait_for_waiting_count(&runtime, SourcePingedEvent::SUBJECT, &durable_name, 0).await;
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-stop-after".to_string(),
            value: "after-stop".to_string(),
        })
        .await
        .expect("publish event after service stop");
    wait_for_pending_count(&runtime, SourcePingedEvent::SUBJECT, &durable_name, 1).await;
    assert_no_observed_entry(&observed, "rust-event-consumers-stop-after").await;

    listener.abort();
    let _ = listener.await;
}

#[tokio::test]
async fn event_consumers_grouped_consumer_waits_for_all_handlers_before_consuming_queued_event() {
    assert_runtime_case_registered(
        "event-consumers.grouped-consumer-waits-for-all-handlers-before-consuming-queued-event",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(
            &bootstrap_url,
            &source_contract,
            Some(&runtime.integration_name("source")),
            None,
        )
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        GROUPED_DEPENDENCY_CONSUMER_JSON,
        "event-consumers-grouped-dependency-rust",
    )
    .await;

    let observed_ping = Arc::new(Mutex::new(None::<String>));
    let handler_observed_ping = Arc::clone(&observed_ping);
    let ping_listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            move |event, context| {
                let handler_observed_ping = Arc::clone(&handler_observed_ping);
                async move {
                    assert_eq!(context.group.as_deref(), Some("paired"));
                    *handler_observed_ping.lock().await = Some(event.id);
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("paired".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start grouped ping listener");

    let publisher_contract = publisher_contract();
    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract)
        .await
        .expect("connect event publisher client");
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-consumers-grouped".to_string(),
            value: "queued".to_string(),
        })
        .await
        .expect("publish queued source event");

    wait_for_grouped_pending_count(&runtime, bound_name(&consumer, "paired"), 1).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(*observed_ping.lock().await, None);

    let pong_listener = consumer
        .listen_event_with_api_id::<SourcePongedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            |_event, context| async move {
                assert_eq!(context.group.as_deref(), Some("paired"));
                Ok(())
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("paired".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start grouped pong listener");

    wait_for_observed_id(&observed_ping, "rust-event-consumers-grouped").await;
    ping_listener.abort();
    pong_listener.abort();
    let _ = ping_listener.await;
    let _ = pong_listener.await;
}

#[tokio::test]
async fn event_consumers_self_owned_grouped_consumer_waits_for_all_handlers_before_consuming_queued_event(
) {
    assert_runtime_case_registered("event-consumers.self-owned-grouped-consumer-waits-for-all-handlers-before-consuming-queued-event",
    "event-consumers",
    "event_consumers",);

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let service = connect_consumer(
        &runtime,
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        SELF_CONSUMER_JSON,
        "event-consumers-self-rust",
    )
    .await;

    let observed_ping = Arc::new(Mutex::new(None::<String>));
    let handler_observed_ping = Arc::clone(&observed_ping);
    let ping_listener = service
        .listen_event::<SelfPingedEvent, _, _>(
            move |event, context| {
                let handler_observed_ping = Arc::clone(&handler_observed_ping);
                async move {
                    assert_eq!(context.group.as_deref(), Some("paired"));
                    *handler_observed_ping.lock().await = Some(event.id);
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("paired".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start self-owned grouped ping listener");

    service
        .event_publisher()
        .publish::<SelfPingedEvent>(&EventRecord {
            id: "rust-event-consumers-self-grouped".to_string(),
            value: "queued".to_string(),
        })
        .await
        .expect("publish queued self-owned event");

    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(*observed_ping.lock().await, None);

    let pong_listener = service
        .listen_event::<SelfPongedEvent, _, _>(
            |_event, context| async move {
                assert_eq!(context.group.as_deref(), Some("paired"));
                Ok(())
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("paired".to_string()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start self-owned grouped pong listener");

    wait_for_observed_id(&observed_ping, "rust-event-consumers-self-grouped").await;
    ping_listener.abort();
    pong_listener.abort();
    let _ = ping_listener.await;
    let _ = pong_listener.await;
}

async fn wait_for_duplicate_observed(observed: &Arc<Mutex<Vec<String>>>, event_id: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    let expected = [format!("first:{event_id}"), format!("second:{event_id}")];
    loop {
        let mut actual = observed.lock().await.clone();
        actual.sort();
        if actual == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for both duplicate handlers"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_observed_entry(observed: &Arc<Mutex<Vec<String>>>, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if observed.lock().await.iter().any(|entry| entry == expected) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for event");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn assert_no_observed_entry(observed: &Arc<Mutex<Vec<String>>>, unexpected: &str) {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        assert!(
            !observed
                .lock()
                .await
                .iter()
                .any(|entry| entry == unexpected),
            "stopped service received event {unexpected}"
        );
        if Instant::now() >= deadline {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_waiting_count(
    runtime: &trellis_test::TrellisTestRuntime,
    subject: &str,
    consumer_name: &str,
    expected: usize,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let consumers = matching_named_consumers(runtime, subject, consumer_name).await;
        if consumers.len() == 1 && consumers[0].num_waiting == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for JetStream waiter count"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_pending_count(
    runtime: &trellis_test::TrellisTestRuntime,
    subject: &str,
    consumer_name: &str,
    expected: usize,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let consumers = matching_named_consumers(runtime, subject, consumer_name).await;
        if consumers.len() == 1 && consumers[0].num_pending == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for JetStream pending count"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_matching_consumer_count(
    runtime: &trellis_test::TrellisTestRuntime,
    subject: &str,
    consumer_name: &str,
    expected: usize,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if matching_named_consumers(runtime, subject, consumer_name)
            .await
            .len()
            == expected
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for JetStream consumer count"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_grouped_pending_count(
    runtime: &trellis_test::TrellisTestRuntime,
    consumer_name: &str,
    expected: usize,
) {
    wait_for_matching_grouped_pending_count(
        runtime,
        SourcePingedEvent::SUBJECT,
        SourcePongedEvent::SUBJECT,
        consumer_name,
        expected,
    )
    .await;
}

async fn wait_for_matching_grouped_pending_count(
    runtime: &trellis_test::TrellisTestRuntime,
    first_subject: &str,
    second_subject: &str,
    consumer_name: &str,
    expected: usize,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let consumers =
            matching_named_grouped_consumers(runtime, first_subject, second_subject, consumer_name)
                .await;
        if consumers.len() == 1 && consumers[0].num_pending == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for grouped JetStream pending count"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_matching_grouped_waiting_count(
    runtime: &trellis_test::TrellisTestRuntime,
    first_subject: &str,
    second_subject: &str,
    consumer_name: &str,
    expected: usize,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let consumers =
            matching_named_grouped_consumers(runtime, first_subject, second_subject, consumer_name)
                .await;
        if consumers.len() == 1 && consumers[0].num_waiting == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for grouped JetStream waiting count"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_matching_grouped_ack_pending_and_waiting_count(
    runtime: &trellis_test::TrellisTestRuntime,
    first_subject: &str,
    second_subject: &str,
    consumer_name: &str,
    expected_ack_pending: usize,
    expected_waiting: usize,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let consumers =
            matching_named_grouped_consumers(runtime, first_subject, second_subject, consumer_name)
                .await;
        if consumers.len() == 1
            && consumers[0].num_ack_pending == expected_ack_pending
            && consumers[0].num_waiting == expected_waiting
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for grouped JetStream ack-pending and waiting counts"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn start_runtime() -> (
    trellis_test::TrellisTestRuntime,
    String,
    trellis_test::TrellisTestAdmin,
) {
    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let admin = runtime.admin();
    (runtime, bootstrap_url, admin)
}

async fn connect_consumer(
    runtime: &trellis_test::TrellisTestRuntime,
    admin: &mut trellis_test::TrellisTestAdmin,
    trellis_url: &str,
    bootstrap_url: &str,
    manifest_json: &str,
    service_name: &str,
) -> ConnectedServiceRuntime<EventConsumerContract> {
    let contract = test_contract(manifest_json);
    let instance_name = runtime.integration_name(service_name);
    let service_key = admin
        .provision_service_instance(bootstrap_url, &contract, Some(&instance_name), None)
        .await
        .expect("provision event consumer service instance");
    trellis_test::connect_service_runtime::<EventConsumerContract>(trellis_url, &service_key)
        .await
        .expect("connect event consumer service runtime")
}

fn test_contract(manifest_json: &str) -> trellis_test::TrellisTestContract {
    let source = trellis_test::TrellisTestContract::from_native_api_json(
        SOURCE_CONTRACT_ID,
        SOURCE_API_SOURCE_JSON,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build event source test contract");
    if manifest_json == SOURCE_API_SOURCE_JSON {
        return source;
    }

    let (id, display_name, description) = match manifest_json {
        MISSING_GROUP_CONSUMER_JSON => (
            "trellis.integration.event-consumers-missing-group-rust@v1",
            "Trellis Rust Event Consumers Missing Group",
            "Uses source events but intentionally declares no durable event consumer group.",
        ),
        AMBIGUOUS_GROUP_CONSUMER_JSON => (
            "trellis.integration.event-consumers-ambiguous-group-rust@v1",
            "Trellis Rust Event Consumers Ambiguous Group",
            "Declares two durable groups for one source event to require an explicit group.",
        ),
        DEPENDENCY_CONSUMER_JSON => (
            "trellis.integration.event-consumers-dependency-rust@v1",
            "Trellis Rust Event Consumers Dependency",
            "Consumes source events through one Trellis-provisioned durable group.",
        ),
        PARALLEL_DEPENDENCY_CONSUMER_JSON => (
            "trellis.integration.event-consumers-parallel-dependency-rust@v1",
            "Trellis Rust Event Consumers Parallel Dependency",
            "Consumes source events through a parallel Trellis-provisioned durable group.",
        ),
        GROUPED_DEPENDENCY_CONSUMER_JSON => (
            "trellis.integration.event-consumers-grouped-dependency-rust@v1",
            "Trellis Rust Event Consumers Grouped Dependency",
            "Consumes two source events through one Trellis-provisioned durable group.",
        ),
        SELF_CONSUMER_JSON => (
            "trellis.integration.event-consumers-self-rust@v1",
            "Trellis Rust Event Consumers Self",
            "Publishes and consumes self-owned events through durable groups.",
        ),
        _ => panic!("unknown event consumer fixture"),
    };
    let mut builder = trellis_rs::contracts::ContractBuilder::authoring(id, id,
    "1.0.0",
    display_name,
    description,
    trellis_rs::contracts::ContractKind::Service,)
    .schema(
        "EventRecord",
        serde_json::json!({"type": "object", "required": ["id", "value"], "properties": {"id": {"type": "string"}, "value": {"type": "string"}}}),
    );
    if manifest_json == SELF_CONSUMER_JSON {
        builder = builder
            .event(
                "Self.Pinged",
                trellis_rs::contracts::event("v1", "events.v1.Self.Pinged", "EventRecord"),
            )
            .event(
                "Self.Ponged",
                trellis_rs::contracts::event("v1", "events.v1.Self.Ponged", "EventRecord"),
            )
            .event_consumer("ingest", event_consumer_group(&[], &["Self.Pinged"], false))
            .event_consumer(
                "paired",
                event_consumer_group(&[], &["Self.Pinged", "Self.Ponged"], false),
            );
        return trellis_test::TrellisTestContract::from_builder_with_referenced_contracts(
            builder,
            &[],
        )
        .expect("build self event consumer test contract");
    }

    let events = if manifest_json == GROUPED_DEPENDENCY_CONSUMER_JSON {
        vec!["Source.Pinged", "Source.Ponged"]
    } else {
        vec!["Source.Pinged"]
    };
    builder = builder.use_ref(
        "source",
        trellis_rs::contracts::use_contract(SOURCE_CONTRACT_ID)
            .with_event_subscribe(events.clone()),
    );
    if manifest_json == AMBIGUOUS_GROUP_CONSUMER_JSON {
        builder = builder
            .event_consumer("primary", event_consumer_group(&events, &[], false))
            .event_consumer("secondary", event_consumer_group(&events, &[], false));
    } else if manifest_json != MISSING_GROUP_CONSUMER_JSON {
        let name = if manifest_json == GROUPED_DEPENDENCY_CONSUMER_JSON {
            "paired"
        } else {
            "ingest"
        };
        builder = builder.event_consumer(
            name,
            event_consumer_group(
                &events,
                &[],
                manifest_json == PARALLEL_DEPENDENCY_CONSUMER_JSON,
            ),
        );
    }
    trellis_test::TrellisTestContract::from_builder_with_referenced_contracts(builder, &[&source])
        .expect("build dependency event consumer test contract")
}

fn event_consumer_group(
    used_events: &[&str],
    self_events: &[&str],
    parallel: bool,
) -> trellis_rs::contracts::ContractEventConsumerGroup {
    trellis_rs::contracts::ContractEventConsumerGroup {
        uses: (!used_events.is_empty())
            .then(|| {
                (
                    "source".to_owned(),
                    used_events
                        .iter()
                        .map(|event| (*event).to_owned())
                        .collect(),
                )
            })
            .into_iter()
            .collect(),
        self_events: self_events
            .iter()
            .map(|event| (*event).to_owned())
            .collect(),
        replay: trellis_rs::contracts::ContractEventConsumerReplay::New,
        ordering: if parallel {
            trellis_rs::contracts::ContractEventConsumerOrdering::Parallel
        } else {
            trellis_rs::contracts::ContractEventConsumerOrdering::Strict
        },
        ack_wait_ms: Some(if parallel { 10_000 } else { 1_000 }),
        max_deliver: Some(2),
        backoff_ms: None,
        docs: None,
    }
}

fn publisher_contract() -> trellis_test::TrellisTestContract {
    let manifest = trellis_rs::contracts::ContractBuilder::authoring(
        "trellis.integration.event-consumers-publisher-rust@v1",
        "trellis.integration.event-consumers-publisher-rust@v1",
        "1.0.0",
        "Trellis Rust Event Consumers Publisher",
        "Publishes source events through a Rust app facade.",
        trellis_rs::contracts::ContractKind::App,
    )
    .use_ref(
        "source",
        trellis_rs::contracts::use_contract("trellis.integration.event-consumers-source-rust@v1")
            .with_event_publish(["Source.Pinged", "Source.Ponged"]),
    );

    let source = test_contract(SOURCE_API_SOURCE_JSON);
    trellis_test::TrellisTestContract::from_builder_with_referenced_contracts(manifest, &[&source])
        .expect("build publisher test contract")
}

async fn wait_for_observed_id(observed: &Arc<Mutex<Option<String>>>, event_id: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if observed.lock().await.as_deref() == Some(event_id) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for event");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_observed(
    observed: &Arc<Mutex<Option<(EventRecord, ServiceEventListenerContext)>>>,
    event_id: &str,
    group: Option<&str>,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some((event, context)) = observed.lock().await.clone() {
            assert_eq!(event.id, event_id);
            assert_eq!(context.group.as_deref(), group);
            assert_eq!(
                context.mode,
                if group.is_some() {
                    ServiceEventListenerMode::Durable
                } else {
                    ServiceEventListenerMode::Ephemeral
                }
            );
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for event");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn matching_consumers(
    runtime: &trellis_test::TrellisTestRuntime,
    subject: &str,
) -> Vec<trellis_test::TrellisJetStreamConsumerInfo> {
    let subject = subject.to_owned();
    runtime
        .list_trellis_jetstream_consumers()
        .await
        .expect("list Trellis JetStream consumers")
        .into_iter()
        .filter(|consumer| {
            consumer
                .filter_subjects
                .iter()
                .any(|filter_subject| filter_subject == &subject)
        })
        .collect()
}

async fn matching_named_consumers(
    runtime: &trellis_test::TrellisTestRuntime,
    subject: &str,
    expected_name: &str,
) -> Vec<trellis_test::TrellisJetStreamConsumerInfo> {
    matching_consumers(runtime, subject)
        .await
        .into_iter()
        .filter(|consumer| consumer_name(consumer) == expected_name)
        .collect()
}

fn bound_name<'a>(
    service: &'a ConnectedServiceRuntime<EventConsumerContract>,
    group: &str,
) -> &'a str {
    &service.resources().event_consumers[group].consumer_name
}

fn consumer_name(consumer: &trellis_test::TrellisJetStreamConsumerInfo) -> &str {
    consumer.durable_name.as_deref().unwrap_or(&consumer.name)
}

async fn matching_grouped_consumers(
    runtime: &trellis_test::TrellisTestRuntime,
    first_subject: &str,
    second_subject: &str,
) -> Vec<trellis_test::TrellisJetStreamConsumerInfo> {
    let first_subject = first_subject.to_owned();
    let second_subject = second_subject.to_owned();
    runtime
        .list_trellis_jetstream_consumers()
        .await
        .expect("list Trellis JetStream consumers")
        .into_iter()
        .filter(|consumer| {
            consumer
                .filter_subjects
                .iter()
                .any(|filter_subject| filter_subject == &first_subject)
                && consumer
                    .filter_subjects
                    .iter()
                    .any(|filter_subject| filter_subject == &second_subject)
        })
        .collect()
}

async fn matching_named_grouped_consumers(
    runtime: &trellis_test::TrellisTestRuntime,
    first_subject: &str,
    second_subject: &str,
    expected_name: &str,
) -> Vec<trellis_test::TrellisJetStreamConsumerInfo> {
    matching_grouped_consumers(runtime, first_subject, second_subject)
        .await
        .into_iter()
        .filter(|consumer| consumer_name(consumer) == expected_name)
        .collect()
}
