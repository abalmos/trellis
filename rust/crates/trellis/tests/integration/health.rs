use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use async_nats::ConnectOptions;
use futures_util::StreamExt;
use serde_json::{json, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use trellis_rs::client::{verify_event_proof, PreparedTrellisEvent};
use trellis_rs::sdk::health::client::HealthClient;
use trellis_rs::sdk::health::types::{
    HealthInspectRequest, HealthMetricsRequest, HealthQueryRequest, HealthQueryResponse,
    HealthWatchEvent, HealthWatchInput,
};
use ulid::Ulid;

use crate::support::assertions::assert_case_registered;

const CASE_ID: &str = "health.projection-lifecycle-and-recovery";
const SERVICE_ID: &str = "trellis.integration.health-service@v1";
const SERVICE_NAME: &str = "health-fixture-service";
const SERVICE_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.health-service@v1",
  "displayName": "Trellis Integration Health Service",
  "description": "Publishes runtime health samples for projection coverage.",
  "kind": "service"
}"#;
const OBSERVER_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.health-observer@v1",
  "displayName": "Trellis Integration Health Observer",
  "description": "Reads the Trellis health projection.",
  "kind": "app",
  "uses": {
    "required": {
      "health": {
        "contract": "trellis.health@v1",
        "rpc": { "call": ["Health.Query", "Health.Inspect", "Health.Metrics"] },
        "feeds": { "subscribe": ["Health.Watch"] }
      }
    }
  }
}"#;

struct HealthFixtureContract;

impl trellis_rs::service::GeneratedServiceContract for HealthFixtureContract {
    const CONTRACT_ID: &'static str = SERVICE_ID;
    const CONTRACT_DIGEST: &'static str = "runtime";
    const CONTRACT_JSON: &'static str = SERVICE_CONTRACT_JSON;
}

struct HealthRuntimeProcess {
    child: Child,
    config_path: PathBuf,
}

impl HealthRuntimeProcess {
    fn start(runtime: &trellis_test::TrellisTestRuntime) -> Self {
        let config_path = runtime.workdir().join("health-runtime.toml");
        let health_db = runtime.workdir().join("health.sqlite");
        let session_seed = runtime
            .workdir()
            .join(&runtime.manifest().paths.session_seed);
        let nats_dir = runtime.workdir().join("nats/creds");
        let config = format!(
            r#"instance_name = "health-integration"
event_session_seed_file = "{}"

[http]
port = 0

[nats]
servers = "{}"

[nats.runtime]
auth_creds_path = "{}"
trellis_creds_path = "{}"
system_creds_path = "{}"
sentinel_creds_path = "{}"

[health]
history_retention_days = 30
transport_retention_hours = 24
transport_max_bytes = 16777216

[health.storage]
kind = "sqlite"
path = "{}"
journal_mode = "wal"
busy_timeout_ms = 5000
single_writer = true

[leases]
bucket = "trellis_runtime_leases"
replicas = 1
ttl_ms = 15000
renew_ms = 5000
"#,
            toml_path(&session_seed),
            runtime.nats_url(),
            toml_path(&nats_dir.join("auth-auth.creds")),
            toml_path(&nats_dir.join("trellis-auth.creds")),
            toml_path(&nats_dir.join("system.creds")),
            toml_path(&nats_dir.join("sentinel.creds")),
            toml_path(&health_db),
        );
        std::fs::write(&config_path, config).expect("write health runtime config");
        let rust_dir = rust_dir();
        let stdout = File::create(runtime.workdir().join("health-runtime.stdout.log"))
            .expect("create health runtime stdout log");
        let stderr = File::create(runtime.workdir().join("health-runtime.stderr.log"))
            .expect("create health runtime stderr log");
        let child = health_command(rust_dir, &config_path)
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .expect("start Rust health runtime");
        Self { child, config_path }
    }

    fn restart(&mut self) {
        self.stop();
        self.child = health_command(rust_dir(), &self.config_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("restart Rust health runtime");
    }

    fn stop(&mut self) {
        if self
            .child
            .try_wait()
            .expect("inspect health runtime")
            .is_none()
        {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

impl Drop for HealthRuntimeProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

#[tokio::test]
async fn health_projection_lifecycle_and_recovery() {
    assert_case_registered(
        "health.service-publishes-authorized-sample",
        "health",
        "health",
    );
    assert_case_registered(CASE_ID, "health", "health");
    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();
    let service_contract =
        trellis_test::TrellisTestContract::from_manifest_json(SERVICE_CONTRACT_JSON)
            .expect("build health service contract");
    let observer_contract =
        trellis_test::TrellisTestContract::from_manifest_json(OBSERVER_CONTRACT_JSON)
            .expect("build health observer contract");
    let service_key = admin
        .provision_service_instance(&bootstrap_url, &service_contract, Some(SERVICE_NAME), None)
        .await
        .expect("provision health service");
    let observer = admin
        .connect_client(&bootstrap_url, &observer_contract)
        .await
        .expect("connect health observer");

    let mut health_runtime = HealthRuntimeProcess::start(&runtime);
    let raw_nats = ConnectOptions::new()
        .credentials_file(runtime.workdir().join("nats/creds/trellis-auth.creds"))
        .await
        .expect("load Trellis NATS creds")
        .connect(runtime.nats_url())
        .await
        .expect("connect Trellis NATS observer");
    let mut heartbeats = raw_nats
        .subscribe("health.v1.heartbeat.>".to_string())
        .await
        .expect("subscribe to heartbeat transport");
    let mut transitions = raw_nats
        .subscribe("events.v1.Health.StatusChanged".to_string())
        .await
        .expect("subscribe to health transitions");
    raw_nats
        .flush()
        .await
        .expect("flush observer subscriptions");

    let service_runtime = trellis_test::connect_service_runtime::<HealthFixtureContract>(
        runtime.trellis_url(),
        SERVICE_ID,
        service_contract.digest(),
        SERVICE_CONTRACT_JSON,
        &service_key.seed,
    )
    .await
    .expect("connect Rust health service");
    let first_heartbeat = tokio::time::timeout(Duration::from_secs(20), heartbeats.next())
        .await
        .expect("receive automatic heartbeat before timeout")
        .expect("heartbeat transport remains open");
    let mut sample: Value =
        serde_json::from_slice(&first_heartbeat.payload).expect("decode automatic heartbeat");
    assert_eq!(sample["participant"]["contractId"], SERVICE_ID);
    assert_eq!(sample["participant"]["runtime"], "rust");

    let health = HealthClient::new(crate::generated_caller(&observer));
    let initial = wait_for_query(&health, SERVICE_ID, "initial healthy projection", |entry| {
        entry.online_instances == 1
    })
    .await;
    assert_eq!(initial.entries[0].effective_status, "healthy");
    let initial_revision = initial.projection.revision;

    let mut watch = health
        .feed()
        .health()
        .watch(&HealthWatchInput {
            contract_ids: Some(vec![SERVICE_ID.to_string()]),
            deployment_ids: None,
            instance_ids: None,
            participant_kinds: Some(vec![crate::wire("service")]),
        })
        .await
        .expect("subscribe to Health.Watch");
    let ready = tokio::time::timeout(Duration::from_secs(5), watch.next())
        .await
        .expect("receive Health.Watch ready frame")
        .expect("health watch remains open")
        .expect("decode health watch ready frame");
    assert!(matches!(ready, HealthWatchEvent::Ready { .. }));

    update_sample(&mut sample, "healthy", 1_000);
    service_runtime
        .event_publisher()
        .publish_prepared(&PreparedTrellisEvent::new(
            first_heartbeat.subject.to_string(),
            sample.to_string().into(),
        ))
        .await
        .expect("publish short-deadline heartbeat");
    let invalidated = tokio::time::timeout(Duration::from_secs(5), watch.next())
        .await
        .expect("receive Health.Watch invalidation")
        .expect("health watch remains open")
        .expect("decode health invalidation");
    assert!(matches!(
        invalidated,
        HealthWatchEvent::HealthInvalidated { .. }
    ));

    let offline = wait_for_query(
        &health,
        SERVICE_ID,
        "offline deadline projection",
        |entry| entry.offline_instances == 1,
    )
    .await;
    assert_eq!(offline.entries[0].effective_status, "offline");
    assert!(offline.projection.revision > initial_revision);
    let transition = tokio::time::timeout(Duration::from_secs(5), transitions.next())
        .await
        .expect("receive status transition before timeout")
        .expect("status transition subscription remains open");
    assert_signed_transition(&transition);

    let inspect = health
        .rpc()
        .health()
        .inspect(&HealthInspectRequest {
            contract_id: SERVICE_ID.to_string(),
            history_limit: Some(20),
            history_since: None,
            instance_id: None,
            participant_kind: crate::wire("service"),
        })
        .await
        .expect("inspect projected health");
    assert!(inspect.history.iter().any(|interval| {
        interval.effective_status == "offline" && interval.reason == "deadline-expired"
    }));
    let now = OffsetDateTime::now_utc();
    let metrics = health
        .rpc()
        .health()
        .metrics(&HealthMetricsRequest {
            check_names: None,
            contract_id: SERVICE_ID.to_string(),
            end: (now + time::Duration::seconds(1))
                .format(&Rfc3339)
                .expect("format metrics end"),
            instance_ids: None,
            participant_kind: crate::wire("service"),
            start: (now - time::Duration::minutes(5))
                .format(&Rfc3339)
                .expect("format metrics start"),
            step_ms: 300_000,
        })
        .await
        .expect("query health metrics");
    assert!(metrics.summary.sample_count >= 2);
    assert!(metrics.summary.transitions >= 1);

    health_runtime.stop();
    update_sample(&mut sample, "degraded", 30_000);
    service_runtime
        .event_publisher()
        .publish_prepared(&PreparedTrellisEvent::new(
            first_heartbeat.subject.to_string(),
            sample.to_string().into(),
        ))
        .await
        .expect("publish heartbeat while projector is stopped");
    health_runtime.restart();
    let recovered = wait_for_query(
        &health,
        SERVICE_ID,
        "degraded projection after projector restart",
        |entry| entry.effective_status == "degraded",
    )
    .await;
    assert_eq!(recovered.entries[0].effective_status, "degraded");
    assert!(!recovered.projection.gap_detected);
}

fn assert_signed_transition(transition: &async_nats::Message) {
    let headers = transition.headers.as_ref().expect("transition headers");
    let event_id = headers
        .get("Nats-Msg-Id")
        .expect("transition event id")
        .as_str();
    let event_time = headers
        .get("Trellis-Event-Time")
        .expect("transition event time")
        .as_str();
    let session_key = headers
        .get("session-key")
        .expect("transition session key")
        .as_str();
    let proof = headers.get("proof").expect("transition proof").as_str();
    assert!(verify_event_proof(
        session_key,
        transition.subject.as_str(),
        &transition.payload,
        event_id,
        event_time,
        proof,
    )
    .expect("verify Health.StatusChanged proof"));
    let payload: Value =
        serde_json::from_slice(&transition.payload).expect("decode status transition");
    assert_eq!(payload["status"], "offline");
}

async fn wait_for_query(
    health: &HealthClient<'_>,
    contract_id: &str,
    expected: &str,
    predicate: impl Fn(&trellis_rs::sdk::health::types::HealthQueryResponseEntriesItem) -> bool,
) -> HealthQueryResponse {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    let mut last_observation = "no query response".to_string();
    loop {
        match health
            .rpc()
            .health()
            .query(&HealthQueryRequest {
                contract_ids: Some(vec![contract_id.to_string()]),
                deployment_ids: None,
                limit: Some(20),
                offset: Some(0),
                participant_kinds: Some(vec![crate::wire("service")]),
                search: None,
                statuses: None,
            })
            .await
        {
            Ok(response) => {
                if response.entries.first().is_some_and(&predicate) {
                    return response;
                }
                last_observation = format!("response: {:?}", response.entries.first());
            }
            Err(error) => last_observation = format!("query error: {error}"),
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "health query did not reach {expected}; last observation: {last_observation}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn update_sample(sample: &mut Value, status: &str, publish_interval_ms: i64) {
    sample["sample"]["id"] = json!(Ulid::new().to_string());
    sample["sample"]["time"] = json!(OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("format sample time"));
    sample["participant"]["publishIntervalMs"] = json!(publish_interval_ms);
    sample["reportedStatus"] = json!(status);
}

fn rust_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("trellis-rs crate should live under rust/crates/trellis")
}

fn health_command(rust_dir: &Path, config_path: &Path) -> Command {
    let mut command = if let Some(binary) = std::env::var_os("TRELLIS_TEST_SERVER_BIN") {
        Command::new(binary)
    } else {
        let mut command = Command::new("cargo");
        command.args([
            "run",
            "--quiet",
            "--manifest-path",
            rust_dir
                .join("Cargo.toml")
                .to_str()
                .expect("UTF-8 Cargo path"),
            "-p",
            "trellis-runtime",
            "--bin",
            "trellis-server",
            "--",
        ]);
        command
    };
    command
        .args([
            "health",
            "--config",
            config_path.to_str().expect("UTF-8 health config path"),
        ])
        .current_dir(rust_dir)
        .stdin(Stdio::null());
    command
}

fn toml_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}
