use std::{sync::Arc, time::Duration};

use async_nats::jetstream::{self, consumer};
use futures_util::StreamExt;
use trellis_rs::client::{ServiceConnectWithContractOptions, TrellisClient};
use trellis_rs::service::ConnectedServiceRuntime;

const EVENTLOG_DEPLOYMENT: &str = "service-eventlog-live-watch-rust";
const PUBLISHER_DEPLOYMENT: &str = "service-eventlog-live-watch-publisher-rust";
const PUBLISHER_CONTRACT_ID: &str = "trellis.integration.service-eventlog-publisher@v1";
const CLIENT_CONTRACT_ID: &str = "trellis.integration.service-eventlog-client@v1";
const EVENT_SUBJECT: &str = "events.v1.integration.service-eventlog.changed";

const PUBLISHER_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.service-eventlog-publisher@v1",
  "displayName": "Trellis Event Log Live Watch Publisher",
  "description": "Publishes an event for EventLog.Watch integration coverage.",
  "kind": "service",
  "schemas": {
    "Changed": {
      "type": "object",
      "required": ["id"],
      "properties": { "id": { "type": "string" } }
    }
  },
  "events": {
    "Integration.Changed": {
      "version": "v1",
      "subject": "events.v1.integration.service-eventlog.changed",
      "event": { "schema": "Changed" }
    }
  }
}"#;

#[tokio::test]
async fn eventlog_watch_is_ephemeral_and_legacy_watches_expire() {
    let runtime = trellis_test::TrellisTestRuntime::start(Default::default())
        .await
        .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();
    let eventlog_contract = trellis_test::TrellisTestContract::from_manifest_json(
        trellis_service_eventlog::CONTRACT_JSON,
    )
    .expect("build Event Log service contract");
    let publisher_contract =
        trellis_test::TrellisTestContract::from_manifest_json(PUBLISHER_CONTRACT_JSON)
            .expect("build publisher contract");
    let client_contract = eventlog_client_contract().expect("build Event Log client contract");

    let eventlog_key = admin
        .provision_service_instance(
            &bootstrap_url,
            &eventlog_contract,
            Some(EVENTLOG_DEPLOYMENT),
            None,
        )
        .await
        .expect("provision Event Log service instance");
    let publisher_key = admin
        .provision_service_instance(
            &bootstrap_url,
            &publisher_contract,
            Some(PUBLISHER_DEPLOYMENT),
            None,
        )
        .await
        .expect("provision publisher service instance");

    let eventlog_client =
        TrellisClient::connect_service_with_contract(ServiceConnectWithContractOptions {
            trellis_url: runtime.trellis_url(),
            contract_id: trellis_service_eventlog::CONTRACT_ID,
            contract_digest: eventlog_contract.digest(),
            contract_json: trellis_service_eventlog::CONTRACT_JSON,
            session_key_seed_base64url: &eventlog_key.seed,
            timeout_ms: trellis_rs::service::DEFAULT_TIMEOUT_MS,
            retry_delay_ms: trellis_rs::service::DEFAULT_RETRY_DELAY_MS,
            authority_pending_timeout_ms: trellis_rs::service::DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS,
        })
        .await
        .expect("connect Event Log service client");
    let eventlog_nats = eventlog_client.internal_nats().clone();
    let eventlog_jetstream = jetstream::new(eventlog_nats.clone());
    let event_stream = eventlog_jetstream
        .get_stream("trellis")
        .await
        .expect("open Trellis event stream");
    event_stream
        .create_consumer(consumer::pull::Config {
            durable_name: Some("event-log-watch-stale_test-1".to_string()),
            filter_subject: "events.v1.>".to_string(),
            deliver_policy: consumer::DeliverPolicy::New,
            ack_policy: consumer::AckPolicy::Explicit,
            ..Default::default()
        })
        .await
        .expect("seed obsolete EventLog.Watch consumer");

    let previous_db_path = std::env::var_os("TRELLIS_EVENTLOG_DB_PATH");
    std::env::set_var(
        "TRELLIS_EVENTLOG_DB_PATH",
        runtime.workdir().join("service-eventlog-live-watch.sqlite"),
    );
    let service_runtime = ConnectedServiceRuntime::<trellis_service_eventlog::EventLogContract>::from_connected_client(
        trellis_service_eventlog::SERVICE_NAME,
        Arc::new(eventlog_client),
    )
    .expect("build connected Event Log runtime");
    let eventlog_service = trellis_service_eventlog::ConnectedEventLogService::new(service_runtime)
        .expect("build Event Log service");
    if let Some(path) = previous_db_path {
        std::env::set_var("TRELLIS_EVENTLOG_DB_PATH", path);
    } else {
        std::env::remove_var("TRELLIS_EVENTLOG_DB_PATH");
    }
    let eventlog_task = tokio::spawn(async move { eventlog_service.run().await });

    let publisher_client =
        TrellisClient::connect_service_with_contract(ServiceConnectWithContractOptions {
            trellis_url: runtime.trellis_url(),
            contract_id: PUBLISHER_CONTRACT_ID,
            contract_digest: publisher_contract.digest(),
            contract_json: PUBLISHER_CONTRACT_JSON,
            session_key_seed_base64url: &publisher_key.seed,
            timeout_ms: trellis_rs::service::DEFAULT_TIMEOUT_MS,
            retry_delay_ms: trellis_rs::service::DEFAULT_RETRY_DELAY_MS,
            authority_pending_timeout_ms: trellis_rs::service::DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS,
        })
        .await
        .expect("connect publisher service");
    let app_client = admin
        .connect_client(&bootstrap_url, &client_contract)
        .await
        .expect("connect Event Log client");
    let eventlog = trellis_rs::sdk::eventlog::EventlogClient::new(&app_client);
    let mut watch = eventlog
        .feed()
        .event_log()
        .watch()
        .await
        .expect("subscribe to generated EventLog.Watch");
    let ready = tokio::time::timeout(Duration::from_secs(5), watch.next())
        .await
        .expect("EventLog.Watch should become ready")
        .expect("EventLog.Watch should emit ready")
        .expect("EventLog.Watch ready frame should decode");
    assert_eq!(
        ready.0.get("kind").and_then(serde_json::Value::as_str),
        Some("ready")
    );

    let stale_watch = eventlog_jetstream
        .get_stream("trellis")
        .await
        .expect("reopen Trellis event stream")
        .consumer_info("event-log-watch-stale_test-1")
        .await
        .expect("legacy watch should remain during its expiry grace period");
    assert_eq!(
        stale_watch.config.inactive_threshold,
        Duration::from_secs(5 * 60)
    );

    let event_stream = eventlog_jetstream
        .get_stream("trellis")
        .await
        .expect("inspect Trellis event stream");
    let mut consumers = event_stream.consumers();
    let mut found_ephemeral_watch = false;
    while let Some(info) = consumers.next().await {
        let info = info.expect("inspect Event Log consumer");
        if info.config.durable_name.is_none() && info.config.filter_subject == "events.v1.>" {
            assert_eq!(
                info.config
                    .metadata
                    .get("trellis.managed_by")
                    .map(String::as_str),
                Some("platform")
            );
            found_ephemeral_watch = true;
        }
    }
    assert!(found_ephemeral_watch);

    publisher_client
        .internal_nats()
        .publish(EVENT_SUBJECT, br#"{"id":"event-1"}"#.as_slice().into())
        .await
        .expect("publish watched event");
    publisher_client
        .internal_nats()
        .flush()
        .await
        .expect("flush watched event");
    let invalidated = tokio::time::timeout(Duration::from_secs(5), watch.next())
        .await
        .expect("EventLog.Watch should observe event")
        .expect("EventLog.Watch should emit invalidation")
        .expect("EventLog.Watch invalidation should decode");
    assert_eq!(
        invalidated
            .0
            .get("kind")
            .and_then(serde_json::Value::as_str),
        Some("eventQueryInvalidated")
    );

    let metrics_deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let metrics = loop {
        let metrics = eventlog
            .rpc()
            .event_log()
            .metrics(&trellis_rs::sdk::eventlog::types::EventLogMetricsRequest {
                window: Some("15m".to_string()),
            })
            .await
            .expect("query EventLog.Metrics");
        if metrics.summary.total > 0 {
            break metrics;
        }
        assert!(
            tokio::time::Instant::now() < metrics_deadline,
            "EventLog.Metrics should include the projected event"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    assert!(metrics.summary.integrity_exceptions >= 1);
    assert_eq!(
        metrics
            .buckets
            .iter()
            .map(|bucket| bucket.total)
            .sum::<i64>(),
        metrics.summary.total
    );

    drop(watch);
    let expiry_deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let event_stream = eventlog_jetstream
            .get_stream("trellis")
            .await
            .expect("inspect event stream after watch closes");
        let mut consumers = event_stream.consumers();
        let mut watch_exists = false;
        while let Some(info) = consumers.next().await {
            let info = info.expect("inspect Event Log consumer after watch closes");
            watch_exists |=
                info.config.durable_name.is_none() && info.config.filter_subject == "events.v1.>";
        }
        if !watch_exists {
            break;
        }
        assert!(
            tokio::time::Instant::now() < expiry_deadline,
            "ephemeral EventLog.Watch consumer should expire after disconnect"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    eventlog_task.abort();
    let _ = eventlog_task.await;
}

fn eventlog_client_contract(
) -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractManifestBuilder::new(
        CLIENT_CONTRACT_ID,
        "Trellis Event Log Live Watch Client",
        "Subscribes to the generated EventLog.Watch feed.",
        trellis_rs::contracts::ContractKind::App,
    )
    .use_ref(
        "eventlog",
        trellis_rs::contracts::use_contract(trellis_rs::sdk::eventlog::CONTRACT_ID)
            .with_rpc_call(["EventLog.Metrics"])
            .with_feed_subscribe(["EventLog.Watch"]),
    )
    .build()?;

    trellis_test::TrellisTestContract::from_manifest_value(serde_json::to_value(manifest)?)
}
