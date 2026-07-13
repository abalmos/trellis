use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use trellis_rs::sdk::jobs::types::{
    JobsCancelRequest, JobsListServicesRequest, JobsListServicesResponseEntriesItem,
};

use crate::support::assertions::assert_service_case_registered;
use crate::support::jobs_admin::start_rust_jobs_admin;

const CASE_ID: &str = "control-plane.jobs-admin-lists-and-cancels-job";
const SERVICE_CONTRACT_ID: &str =
    "trellis.integration.control-plane.jobs-admin-lists-and-cancels-job.service@v1";
const ADMIN_CLIENT_CONTRACT_ID: &str =
    "trellis.integration.control-plane.jobs-admin-lists-and-cancels-job.client@v1";
const JOB_TYPE: &str = "holdOpen";
const MARKER: &str = "jobs-admin-probe-marker-rust";

const SERVICE_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.control-plane.jobs-admin-lists-and-cancels-job.service@v1",
  "displayName": "Trellis Control-Plane Jobs Admin Probe Service",
  "description": "Creates a long-running service-local job for Jobs admin integration coverage.",
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct HoldPayload {
    marker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct HoldResult {
    cancelled: bool,
}

struct JobsAdminProbeContract;

impl trellis_rs::service::GeneratedServiceContract for JobsAdminProbeContract {
    const CONTRACT_ID: &'static str = SERVICE_CONTRACT_ID;
    const CONTRACT_DIGEST: &'static str = "runtime";
    const CONTRACT_JSON: &'static str = SERVICE_CONTRACT_JSON;
}

struct HoldOpenJob;

impl trellis_rs::jobs::JobDescriptor for HoldOpenJob {
    type Payload = HoldPayload;
    type Result = HoldResult;
    const QUEUE_TYPE: &'static str = JOB_TYPE;
    const PAYLOAD_SCHEMA_JSON: &'static str =
        r#"{"type":"object","required":["marker"],"properties":{"marker":{"type":"string"}}}"#;
    const RESULT_SCHEMA_JSON: Option<&'static str> = Some(
        r#"{"type":"object","required":["cancelled"],"properties":{"cancelled":{"type":"boolean"}}}"#,
    );
}

#[tokio::test]
async fn control_plane_jobs_admin_lists_and_cancels_job() {
    assert_service_case_registered(CASE_ID, "control-plane", "control_plane_jobs_admin");

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
    let _jobs_admin_process = start_rust_jobs_admin(&runtime, &mut admin, &bootstrap_url).await;
    let service_contract =
        trellis_test::TrellisTestContract::from_manifest_json(SERVICE_CONTRACT_JSON)
            .expect("build jobs admin probe service contract");
    let admin_client_contract =
        jobs_admin_client_contract().expect("build jobs admin probe client contract");
    let service_key = admin
        .provision_service_instance(&bootstrap_url, &service_contract, None, None)
        .await
        .expect("provision live jobs admin probe service instance");

    let mut service = trellis_test::connect_service_runtime::<JobsAdminProbeContract>(
        runtime.trellis_url(),
        SERVICE_CONTRACT_ID,
        service_contract.digest(),
        SERVICE_CONTRACT_JSON,
        &service_key.seed,
    )
    .await
    .expect("connect live Rust jobs admin probe service runtime");
    let started = Arc::new(tokio::sync::Notify::new());
    let worker_started = Arc::clone(&started);

    service
        .register_generated_job_worker::<HoldOpenJob, _, _, String>(move |active_job| {
            let worker_started = Arc::clone(&worker_started);
            async move {
                worker_started.notify_one();
                while !active_job.is_cancelled() {
                    active_job
                        .heartbeat()
                        .await
                        .map_err(|error| error.to_string())?;
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
                Ok(HoldResult { cancelled: true })
            }
        })
        .await
        .expect("start jobs admin probe worker host");

    let admin_client = admin
        .connect_client(&bootstrap_url, &admin_client_contract)
        .await
        .expect("connect live Rust jobs admin probe client");
    let jobs_admin = trellis_rs::sdk::jobs::JobsClient::new(crate::generated_caller(&admin_client));

    let job = service
        .generated_submit_job::<HoldOpenJob>(HoldPayload {
            marker: MARKER.to_string(),
        })
        .await
        .expect("create service-local holdOpen job");
    let identity = job.identity().clone();
    tokio::time::timeout(Duration::from_secs(15), started.notified())
        .await
        .expect("worker should start the holdOpen job");

    let listed_service =
        wait_for_listed_service(&jobs_admin, &identity.service, &identity.job_type).await;
    assert!(
        listed_service
            .workers
            .iter()
            .any(|worker| worker.job_type == identity.job_type),
        "expected Jobs.ListServices to include a worker for {}",
        identity.job_type
    );

    let cancelled = jobs_admin
        .rpc()
        .jobs()
        .cancel(&JobsCancelRequest {
            id: identity.id.clone(),
            reason: None,
        })
        .await
        .expect("call generated Jobs.Cancel");
    assert_eq!(cancelled.job.id, identity.id);

    assert_eq!(cancelled.job.state, "cancelled");
    let local_terminal = job
        .wait()
        .await
        .expect("service-local wait observes terminal cancelled");
    assert_eq!(local_terminal.state, trellis_rs::jobs::JobState::Cancelled);

    drop(service);
}

fn jobs_admin_client_contract(
) -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractManifestBuilder::new(
        ADMIN_CLIENT_CONTRACT_ID,
        "Trellis Control-Plane Jobs Admin Probe Client",
        "Uses the generated Jobs admin SDK surface.",
        trellis_rs::contracts::ContractKind::App,
    )
    .use_ref(
        "jobs",
        trellis_rs::contracts::use_contract(trellis_rs::sdk::jobs::CONTRACT_ID)
            .with_rpc_call(["Jobs.Cancel", "Jobs.ListServices"]),
    )
    .build()?;

    trellis_test::TrellisTestContract::from_manifest_value(serde_json::to_value(manifest)?)
}

async fn wait_for_listed_service(
    jobs_admin: &trellis_rs::sdk::jobs::JobsClient<'_>,
    service: &str,
    job_type: &str,
) -> JobsListServicesResponseEntriesItem {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        let page = match jobs_admin
            .rpc()
            .jobs()
            .list_services(&JobsListServicesRequest {
                offset: None,
                limit: 20,
            })
            .await
        {
            Ok(page) => page,
            Err(error)
                if is_retryable_jobs_admin_error(&error)
                    && tokio::time::Instant::now() < deadline =>
            {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            Err(error) => panic!("call generated Jobs.ListServices: {error}"),
        };
        if let Some(entry) = page.entries.iter().find(|entry| {
            entry.name == service
                && entry
                    .workers
                    .iter()
                    .any(|worker| worker.job_type == job_type)
        }) {
            return entry.clone();
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "Jobs.ListServices did not return service worker before timeout; expected {service}/{job_type}, got {:?}",
            page.entries,
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn is_retryable_jobs_admin_error<E: std::fmt::Debug>(
    error: &trellis_rs::client::CallError<E>,
) -> bool {
    match error {
        trellis_rs::client::CallError::Transport(error) => {
            let message = error.to_string();
            message.contains("no responders") || message.contains("NoResponders")
        }
        trellis_rs::client::CallError::Timeout => true,
        _ => false,
    }
}
