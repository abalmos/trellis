use std::{sync::Arc, time::Duration};

use async_nats::jetstream::{self, consumer};
use futures_util::StreamExt;
use serde_json::json;
use trellis_rs::client::{ServiceConnectWithContractOptions, TrellisClient};
use trellis_rs::jobs::keys::NatsKeyCoordinator;
use trellis_rs::jobs::{
    start_worker_host_from_client, JobManager, JobProcessError, JobsRuntimeBinding,
    NatsJobEventPublisher, TrellisJobEventPublisher, TrellisJobMetaSource, WorkerActiveJob,
    WorkerHostOptions,
};
use trellis_rs::sdk::core::types::TrellisBindingsGetResponseBinding;
use trellis_rs::sdk::jobs::types::{
    JobsCancelRequest, JobsInspectRequest, JobsQueryRequest, JobsQueryResponseEntriesItem,
    JobsWatchInput,
};
use trellis_rs::service::ConnectedServiceRuntime;

const PROBE_SERVICE_CONTRACT_ID: &str =
    "trellis.integration.service-jobs-live-admin.probe-service@v1";
const ADMIN_CLIENT_CONTRACT_ID: &str = "trellis.integration.service-jobs-live-admin.client@v1";
const JOBS_SERVICE_DEPLOYMENT: &str = "service-jobs-live-admin-jobs-rust";
const PROBE_SERVICE_DEPLOYMENT: &str = "service-jobs-live-admin-probe-rust";
const PROBE_SERVICE_NAME: &str = "service-jobs-live-admin-probe-rust";
const JOB_TYPE: &str = "holdOpen";
const MARKER: &str = "service-jobs-live-admin-marker-rust";

const PROBE_SERVICE_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.service-jobs-live-admin.probe-service@v1",
  "displayName": "Trellis Service Jobs Live Admin Probe Service",
  "description": "Creates service-local jobs for Rust Jobs admin service coverage.",
  "kind": "service",
  "schemas": {
    "HoldPayload": {
      "type": "object",
      "required": ["marker"],
      "properties": { "marker": { "type": "string" } }
    },
    "HoldResult": {
      "type": "object",
      "required": ["cancelled"],
      "properties": { "cancelled": { "type": "boolean" } }
    }
  },
  "jobs": {
    "holdOpen": {
      "payload": { "schema": "HoldPayload" },
      "result": { "schema": "HoldResult" }
    }
  }
}"#;

#[tokio::test]
async fn rust_service_jobs_hosts_generated_admin_rpcs() {
    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions {
            disable_jobs_admin: true,
            ..trellis_test::TrellisTestRuntimeOptions::default()
        })
        .await
        .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();
    let jobs_contract =
        trellis_test::TrellisTestContract::from_manifest_json(trellis_service_jobs::CONTRACT_JSON)
            .expect("build Jobs admin service contract");
    let probe_contract =
        trellis_test::TrellisTestContract::from_manifest_json(PROBE_SERVICE_CONTRACT_JSON)
            .expect("build probe service contract");
    let admin_client_contract =
        jobs_admin_client_contract().expect("build Jobs admin client contract");

    let jobs_service_key = admin
        .provision_service_instance(
            &bootstrap_url,
            &jobs_contract,
            Some(JOBS_SERVICE_DEPLOYMENT),
            None,
        )
        .await
        .expect("provision Rust Jobs admin service instance");
    let probe_service_key = admin
        .provision_service_instance(
            &bootstrap_url,
            &probe_contract,
            Some(PROBE_SERVICE_DEPLOYMENT),
            None,
        )
        .await
        .expect("provision probe service instance");

    let previous_db_path = std::env::var_os("TRELLIS_JOBS_DB_PATH");
    std::env::set_var(
        "TRELLIS_JOBS_DB_PATH",
        runtime.workdir().join("service-jobs-live-admin.sqlite"),
    );
    let jobs_client =
        TrellisClient::connect_service_with_contract(ServiceConnectWithContractOptions {
            trellis_url: runtime.trellis_url(),
            contract_id: trellis_service_jobs::CONTRACT_ID,
            contract_digest: jobs_contract.digest(),
            contract_json: trellis_service_jobs::CONTRACT_JSON,
            session_key_seed_base64url: &jobs_service_key.seed,
            timeout_ms: trellis_rs::service::DEFAULT_TIMEOUT_MS,
            retry_delay_ms: trellis_rs::service::DEFAULT_RETRY_DELAY_MS,
            authority_pending_timeout_ms: trellis_rs::service::DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS,
        })
        .await
        .expect("connect Rust Jobs admin service client");
    let jobs_nats = jobs_client.internal_nats().clone();
    let jobs_jetstream = jetstream::new(jobs_nats.clone());
    let jobs_stream = jobs_jetstream
        .get_stream("JOBS")
        .await
        .expect("open Jobs lifecycle stream");
    jobs_stream
        .create_consumer(consumer::pull::Config {
            durable_name: Some("jobs-watch-stale_test-1".to_string()),
            filter_subject: "trellis.jobs.>".to_string(),
            ack_policy: consumer::AckPolicy::Explicit,
            ..Default::default()
        })
        .await
        .expect("seed obsolete Jobs.Watch consumer");
    let jobs_runtime =
        ConnectedServiceRuntime::<trellis_service_jobs::JobsContract>::from_connected_client(
            trellis_service_jobs::SERVICE_NAME,
            Arc::new(jobs_client),
        )
        .expect("build connected Rust Jobs admin service runtime");
    let jobs_service = trellis_service_jobs::ConnectedJobsService::new(jobs_runtime)
        .expect("build Rust Jobs admin service");
    if let Some(path) = previous_db_path {
        std::env::set_var("TRELLIS_JOBS_DB_PATH", path);
    } else {
        std::env::remove_var("TRELLIS_JOBS_DB_PATH");
    }
    let jobs_service_task = tokio::spawn(async move {
        jobs_service
            .run_with_mode(trellis_service_jobs::JobsServiceMode::Owner)
            .await
    });

    let probe_client =
        TrellisClient::connect_service_with_contract(ServiceConnectWithContractOptions {
            trellis_url: runtime.trellis_url(),
            contract_id: PROBE_SERVICE_CONTRACT_ID,
            contract_digest: probe_contract.digest(),
            contract_json: PROBE_SERVICE_CONTRACT_JSON,
            session_key_seed_base64url: &probe_service_key.seed,
            timeout_ms: trellis_rs::service::DEFAULT_TIMEOUT_MS,
            retry_delay_ms: trellis_rs::service::DEFAULT_RETRY_DELAY_MS,
            authority_pending_timeout_ms: trellis_rs::service::DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS,
        })
        .await
        .expect("connect probe service");
    let probe_service = ConnectedServiceRuntime::<()>::from_connected_client(
        PROBE_SERVICE_NAME,
        Arc::new(probe_client),
    )
    .expect("build connected probe service runtime");
    let nats = probe_service.client().internal_nats().clone();
    let trellis_binding: &TrellisBindingsGetResponseBinding = probe_service.binding().as_ref();
    let jobs_runtime_binding = JobsRuntimeBinding::try_from(trellis_binding)
        .expect("parse probe service jobs runtime binding");
    let publisher = NatsJobEventPublisher::new(nats.clone());
    let key_coordinator = NatsKeyCoordinator::open_for_service(
        nats.clone(),
        jobs_runtime_binding.jobs.namespace.as_str(),
    )
    .await
    .expect("open probe key coordinator");
    let manager = JobManager::new_with_key_coordinator(
        publisher,
        jobs_runtime_binding.jobs.clone(),
        TrellisJobMetaSource,
        Arc::new(key_coordinator),
    );
    let started = Arc::new(tokio::sync::Notify::new());
    let worker_started = Arc::clone(&started);
    let worker_host = start_worker_host_from_client(
        &*probe_service.client(),
        jobs_runtime_binding,
        PROBE_SERVICE_NAME.to_string(),
        |_, _| TrellisJobMetaSource,
        move |active_job: WorkerActiveJob<TrellisJobEventPublisher, TrellisJobMetaSource>| {
            let worker_started = Arc::clone(&worker_started);
            async move {
                worker_started.notify_one();
                while !active_job.is_cancelled() {
                    active_job
                        .heartbeat()
                        .await
                        .map_err(|error| JobProcessError::failed(error.to_string()))?;
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
                Ok(json!({ "cancelled": true }))
            }
        },
        WorkerHostOptions::default(),
    )
    .await
    .expect("start probe worker host");

    let admin_client = admin
        .connect_client(&bootstrap_url, &admin_client_contract)
        .await
        .expect("connect Jobs admin client");
    let jobs_admin = trellis_rs::sdk::jobs::JobsClient::new(&admin_client);

    let job = manager
        .create(JOB_TYPE, json!({ "marker": MARKER }))
        .await
        .expect("create service-local holdOpen job");
    tokio::time::timeout(Duration::from_secs(15), started.notified())
        .await
        .expect("worker should start the holdOpen job");

    let listed_job = wait_for_listed_job(&jobs_admin, &job.service, &job.job_type, &job.id).await;
    assert_eq!(listed_job.service, job.service);
    assert_eq!(listed_job.r#type, job.job_type);
    let stale_watch = jobs_jetstream
        .get_stream("JOBS")
        .await
        .expect("reopen Jobs lifecycle stream")
        .consumer_info("jobs-watch-stale_test-1")
        .await
        .expect("obsolete Jobs.Watch consumer should remain during its expiry grace period");
    assert_eq!(
        stale_watch.config.inactive_threshold,
        Duration::from_secs(5 * 60)
    );
    assert_eq!(
        stale_watch
            .config
            .metadata
            .get("trellis.managed_by")
            .map(String::as_str),
        Some("platform")
    );

    let detail = jobs_admin
        .rpc()
        .jobs()
        .inspect(&JobsInspectRequest { id: job.id.clone() })
        .await
        .expect("call generated Jobs.Inspect");
    assert_eq!(detail.job.id, job.id);
    assert_eq!(detail.job.service, job.service);
    assert_eq!(detail.job.r#type, job.job_type);
    assert_eq!(detail.job.payload, json!({ "marker": MARKER }));
    assert!(detail
        .timeline
        .iter()
        .any(|event| event.r#type == "started"));

    let mut watch = jobs_admin
        .feed()
        .jobs()
        .watch(&JobsWatchInput {
            include_initial: Some(false),
            job_id: Some(job.id.clone()),
            query: None,
        })
        .await
        .expect("subscribe to generated Jobs.Watch");
    let ready = tokio::time::timeout(Duration::from_secs(5), watch.next())
        .await
        .expect("Jobs.Watch should become ready")
        .expect("Jobs.Watch should emit ready")
        .expect("Jobs.Watch ready frame should decode");
    assert_eq!(ready.0["kind"], "ready");
    let jobs_stream = jobs_jetstream
        .get_stream("JOBS")
        .await
        .expect("inspect Jobs lifecycle stream");
    let mut consumers = jobs_stream.consumers();
    let watch_filter = format!("trellis.jobs.*.*.{}.>", job.id);
    let mut found_ephemeral_watch = false;
    while let Some(info) = consumers.next().await {
        let info = info.expect("inspect Jobs consumer");
        if info.config.filter_subject == watch_filter {
            assert!(info.config.durable_name.is_none());
            assert_eq!(
                info.config
                    .metadata
                    .get("trellis.managed_by")
                    .map(String::as_str),
                Some("platform")
            );
            found_ephemeral_watch = true;
        }
        if info
            .config
            .durable_name
            .as_deref()
            .is_some_and(|name| name.starts_with("jobs-watch-"))
        {
            assert_eq!(info.config.inactive_threshold, Duration::from_secs(5 * 60));
        }
    }
    assert!(found_ephemeral_watch);

    let cancelled = jobs_admin
        .rpc()
        .jobs()
        .cancel(&JobsCancelRequest {
            id: job.id.clone(),
            reason: None,
        })
        .await
        .expect("call generated Jobs.Cancel");
    assert_eq!(cancelled.job.id, job.id);

    let changed = tokio::time::timeout(Duration::from_secs(5), watch.next())
        .await
        .expect("Jobs.Watch should observe cancellation")
        .expect("Jobs.Watch should emit cancellation invalidation")
        .expect("Jobs.Watch cancellation frame should decode");
    assert_eq!(changed.0["kind"], "jobInspectChanged");
    assert_eq!(changed.0["id"], job.id);
    drop(watch);
    let expiry_deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let jobs_stream = jobs_jetstream
            .get_stream("JOBS")
            .await
            .expect("inspect Jobs lifecycle stream after watch closes");
        let mut consumers = jobs_stream.consumers();
        let mut watch_exists = false;
        while let Some(info) = consumers.next().await {
            let info = info.expect("inspect Jobs consumer after watch closes");
            watch_exists |=
                info.config.durable_name.is_none() && info.config.filter_subject == watch_filter;
        }
        if !watch_exists {
            break;
        }
        assert!(
            tokio::time::Instant::now() < expiry_deadline,
            "ephemeral Jobs.Watch consumer should expire after disconnect"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let terminal = wait_for_job_state(&jobs_admin, &job.id, "cancelled").await;
    assert_eq!(terminal.state, "cancelled");

    worker_host.stop().await.expect("stop probe worker host");

    jobs_service_task.abort();
    let _ = jobs_service_task.await;
}

fn jobs_admin_client_contract(
) -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractManifestBuilder::new(
        ADMIN_CLIENT_CONTRACT_ID,
        "Trellis Service Jobs Live Admin Client",
        "Uses the generated Jobs admin SDK surface.",
        trellis_rs::contracts::ContractKind::App,
    )
    .use_ref(
        "jobs",
        trellis_rs::contracts::use_contract(trellis_rs::sdk::jobs::CONTRACT_ID)
            .with_rpc_call(["Jobs.Query", "Jobs.Inspect", "Jobs.Cancel"])
            .with_feed_subscribe(["Jobs.Watch"]),
    )
    .build()?;

    trellis_test::TrellisTestContract::from_manifest_value(serde_json::to_value(manifest)?)
}

async fn wait_for_listed_job(
    jobs_admin: &trellis_rs::sdk::jobs::JobsClient<'_>,
    service: &str,
    job_type: &str,
    job_id: &str,
) -> JobsQueryResponseEntriesItem {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        let page = jobs_admin
            .rpc()
            .jobs()
            .query(&JobsQueryRequest {
                service: Some(service.to_string()),
                r#type: Some(job_type.to_string()),
                state: None,
                search: None,
                queue_key: None,
                trigger: None,
                runtime_band: None,
                group_by: None,
                sort: None,
                window: None,
                offset: None,
                limit: 20,
            })
            .await
            .expect("call generated Jobs.Query");
        if let Some(entry) = page.entries.into_iter().find(|entry| entry.id == job_id) {
            return entry;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "Jobs.Query did not return job before timeout"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_job_state(
    jobs_admin: &trellis_rs::sdk::jobs::JobsClient<'_>,
    job_id: &str,
    state: &str,
) -> trellis_rs::sdk::jobs::types::JobsInspectResponseJob {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        let current = jobs_admin
            .rpc()
            .jobs()
            .inspect(&JobsInspectRequest {
                id: job_id.to_string(),
            })
            .await
            .expect("call generated Jobs.Inspect while polling state");
        if current.job.state == state {
            return current.job;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "Jobs.Inspect did not reach {state} before timeout"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
