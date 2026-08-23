use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::{format_description::well_known::Rfc3339, Duration as TimeDuration, OffsetDateTime};
use tokio::task::JoinHandle;
use trellis_rs::jobs::keys::NatsKeyCoordinator;
use trellis_rs::jobs::manager::{JobNotEnqueuedReason, JobSubmitOutcome};
use trellis_rs::jobs::{
    publish_worker_heartbeat, runtime_ref::NatsJobWaiter, JobDescriptor, JobLogLevel, JobManager,
    JobProcessError, JobState, JobUpdateDescriptor, NatsJobEventPublisher, TrellisJobMetaSource,
    WorkerActiveJob, WorkerHeartbeat, WorkerHostOptions,
};
use trellis_rs::sdk::jobs::types::{
    JobsListServicesRequest, JobsListServicesResponse, JobsQueryRequest,
};
use trellis_rs::service::ServerError;

use crate::support::assertions::assert_runtime_case_registered;

const JOBS_SERVICE_ID: &str = "trellis.integration.jobs-service@v1";
const JOBS_CLIENT_ID: &str = "trellis.integration.jobs-client@v1";
const JOBS_ADMIN_CLIENT_ID: &str = "trellis.integration.jobs-admin-client@v1";

const JOBS_SERVICE_API_SOURCE_JSON: &str = r#"{
  "format": "trellis.api.v1",
  "id": "trellis.integration.jobs-service@v1",
  "displayName": "Trellis Integration Jobs Service",
  "description": "Exercises service-local jobs behind a client-visible RPC.",
    "schemas": {
    "WorkflowInput": {
      "type": "object",
      "required": ["documentId"],
      "properties": { "documentId": { "type": "string" } }
    },
    "WorkflowOutput": {
      "type": "object",
      "required": ["documentId", "jobId", "processedBy", "requestId", "traceId"],
      "properties": {
        "documentId": { "type": "string" },
        "jobId": { "type": "string" },
        "processedBy": { "type": "string" },
        "requestId": { "type": "string" },
        "traceId": { "type": "string" }
      }
    },
    "JobPayload": {
      "type": "object",
      "required": ["documentId"],
      "properties": { "documentId": { "type": "string" } }
    },
    "JobUpdate": {
      "type": "object",
      "required": ["processed"],
      "properties": { "processed": { "type": "number" } }
    },
    "LongJobPayload": {
      "type": "object",
      "required": ["documentId"],
      "properties": { "documentId": { "type": "string" } }
    },
    "FailingJobPayload": {
      "type": "object",
      "required": ["documentId"],
      "properties": { "documentId": { "type": "string" } }
    },
    "KeyedWorkflowInput": {
      "type": "object",
      "required": ["documentId", "groupKey", "sequence"],
      "properties": {
        "documentId": { "type": "string" },
        "groupKey": { "type": "string" },
        "sequence": { "type": "number" }
      }
    },
    "KeyedWorkflowOutput": {
      "type": "object",
      "required": ["documentId", "groupKey", "sequence", "jobId", "processedBy", "requestId", "traceId"],
      "properties": {
        "documentId": { "type": "string" },
        "groupKey": { "type": "string" },
        "sequence": { "type": "number" },
        "jobId": { "type": "string" },
        "processedBy": { "type": "string" },
        "requestId": { "type": "string" },
        "traceId": { "type": "string" }
      }
    },
    "JobResult": {
      "type": "object",
      "required": ["documentId", "processedBy", "requestId", "traceId"],
      "properties": {
        "documentId": { "type": "string" },
        "processedBy": { "type": "string" },
        "requestId": { "type": "string" },
        "traceId": { "type": "string" }
      }
    },
    "KeyedJobPayload": {
      "type": "object",
      "required": ["documentId", "groupKey", "sequence"],
      "properties": {
        "documentId": { "type": "string" },
        "groupKey": { "type": "string" },
        "sequence": { "type": "number" }
      }
    },
    "KeyedJobResult": {
      "type": "object",
      "required": ["documentId", "groupKey", "sequence", "processedBy", "requestId", "traceId"],
      "properties": {
        "documentId": { "type": "string" },
        "groupKey": { "type": "string" },
        "sequence": { "type": "number" },
        "processedBy": { "type": "string" },
        "requestId": { "type": "string" },
        "traceId": { "type": "string" }
      }
    }
  },
  "rpc": {
    "Documents.Process": {
      "version": "v1",
      "input": { "schema": "WorkflowInput" },
      "output": { "schema": "WorkflowOutput" },
      "errors": []
    },
    "Documents.KeyedProcess": {
      "version": "v1",
      "input": { "schema": "KeyedWorkflowInput" },
      "output": { "schema": "KeyedWorkflowOutput" },
      "errors": []
    },
    "Documents.SubmitLongProcess": {
      "version": "v1",
      "input": { "schema": "WorkflowInput" },
      "output": { "schema": "WorkflowOutput" },
      "errors": []
    }
  }
}"#;

struct JobsFixtureContract;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct JobPayload {
    #[serde(rename = "documentId")]
    document_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct JobResult {
    document_id: String,
    processed_by: String,
    request_id: String,
    trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct JobUpdatePayload {
    processed: u64,
}

struct ProcessDocumentJob;

impl JobDescriptor for ProcessDocumentJob {
    type Payload = JobPayload;
    type Result = JobResult;

    const QUEUE_TYPE: &'static str = "processDocument";
    const PAYLOAD_SCHEMA_JSON: &'static str = r#"{"type":"object","required":["documentId"],"properties":{"documentId":{"type":"string"}}}"#;
    const RESULT_SCHEMA_JSON: Option<&'static str> = None;
}

impl JobUpdateDescriptor for ProcessDocumentJob {
    type Update = JobUpdatePayload;

    const UPDATE_SCHEMA: &'static str = "JobUpdate";
    const UPDATE_SCHEMA_JSON: &'static str = r#"{"type":"object","required":["processed"],"properties":{"processed":{"type":"number"}}}"#;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct KeyedJobPayload {
    document_id: String,
    group_key: String,
    sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct KeyedJobResult {
    document_id: String,
    group_key: String,
    sequence: u64,
    processed_by: String,
    request_id: String,
    trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct WorkflowInput {
    #[serde(rename = "documentId")]
    document_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct WorkflowOutput {
    document_id: String,
    job_id: String,
    processed_by: String,
    request_id: String,
    trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct KeyedWorkflowInput {
    document_id: String,
    group_key: String,
    sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct KeyedWorkflowOutput {
    document_id: String,
    group_key: String,
    sequence: u64,
    job_id: String,
    processed_by: String,
    request_id: String,
    trace_id: String,
}

struct DocumentsProcessRpc;

impl trellis_rs::client::RpcDescriptor for DocumentsProcessRpc {
    type Input = WorkflowInput;
    type Output = WorkflowOutput;

    const KEY: &'static str = "Documents.Process";
    const SUBJECT: &'static str = "rpc.v1.Documents.Process";
    const CALLER_CAPABILITIES: &'static [&'static str] = &[];
    const ERRORS: &'static [&'static str] = &[];
    const INPUT_SCHEMA_JSON: &'static str = r#"{"type":"object","required":["documentId"],"properties":{"documentId":{"type":"string"}}}"#;
    const OUTPUT_SCHEMA_JSON: &'static str = r#"{"type":"object","required":["documentId","jobId","processedBy","requestId","traceId"],"properties":{"documentId":{"type":"string"},"jobId":{"type":"string"},"processedBy":{"type":"string"},"requestId":{"type":"string"},"traceId":{"type":"string"}}}"#;
}

struct DocumentsKeyedProcessRpc;

impl trellis_rs::client::RpcDescriptor for DocumentsKeyedProcessRpc {
    type Input = KeyedWorkflowInput;
    type Output = KeyedWorkflowOutput;

    const KEY: &'static str = "Documents.KeyedProcess";
    const SUBJECT: &'static str = "rpc.v1.Documents.KeyedProcess";
    const CALLER_CAPABILITIES: &'static [&'static str] = &[];
    const ERRORS: &'static [&'static str] = &[];
    const INPUT_SCHEMA_JSON: &'static str = r#"{"type":"object","required":["documentId","groupKey","sequence"],"properties":{"documentId":{"type":"string"},"groupKey":{"type":"string"},"sequence":{"type":"number"}}}"#;
    const OUTPUT_SCHEMA_JSON: &'static str = r#"{"type":"object","required":["documentId","groupKey","sequence","jobId","processedBy","requestId","traceId"],"properties":{"documentId":{"type":"string"},"groupKey":{"type":"string"},"sequence":{"type":"number"},"jobId":{"type":"string"},"processedBy":{"type":"string"},"requestId":{"type":"string"},"traceId":{"type":"string"}}}"#;
}

struct DocumentsSubmitLongProcessRpc;

impl trellis_rs::client::RpcDescriptor for DocumentsSubmitLongProcessRpc {
    type Input = WorkflowInput;
    type Output = WorkflowOutput;

    const KEY: &'static str = "Documents.SubmitLongProcess";
    const SUBJECT: &'static str = "rpc.v1.Documents.SubmitLongProcess";
    const CALLER_CAPABILITIES: &'static [&'static str] = &[];
    const ERRORS: &'static [&'static str] = &[];
    const INPUT_SCHEMA_JSON: &'static str = r#"{"type":"object","required":["documentId"],"properties":{"documentId":{"type":"string"}}}"#;
    const OUTPUT_SCHEMA_JSON: &'static str = r#"{"type":"object","required":["documentId","jobId","processedBy","requestId","traceId"],"properties":{"documentId":{"type":"string"},"jobId":{"type":"string"},"processedBy":{"type":"string"},"requestId":{"type":"string"},"traceId":{"type":"string"}}}"#;
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

struct JobsFixture {
    _runtime: trellis_test::TrellisTestRuntime,
    _admin: trellis_test::TrellisTestAdmin,
    bootstrap_url: String,
    worker_host: trellis_rs::jobs::WorkerHostHandle,
    service_task: AbortOnDrop<Result<(), trellis_rs::service::ServiceRuntimeError>>,
    service_handle: trellis_rs::service::ServiceHandle,
    client: Arc<trellis_rs::generated::Caller>,
    manager: JobManager<NatsJobEventPublisher, TrellisJobMetaSource>,
    keyed_waiter: NatsJobWaiter,
    coalesce_waiter: NatsJobWaiter,
    replace_waiter: NatsJobWaiter,
    cancel_waiter: NatsJobWaiter,
    update_waiter: NatsJobWaiter,
    update_release: Arc<tokio::sync::Notify>,
    jobs_worker_runtime: trellis_rs::jobs::TestJobsWorkerRuntime,
    keyed_run_state: Arc<KeyedJobRunState>,
    failing_attempts: Arc<tokio::sync::Mutex<Vec<u64>>>,
}

#[derive(Debug, Default)]
struct KeyedJobRunState {
    started: tokio::sync::Mutex<Vec<u64>>,
    completed: tokio::sync::Mutex<Vec<u64>>,
    first_started: tokio::sync::Notify,
    release_first: tokio::sync::Notify,
    released: std::sync::atomic::AtomicBool,
    second_started_before_release: std::sync::atomic::AtomicBool,
}

async fn setup_jobs_fixture() -> JobsFixture {
    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    let api: Value =
        serde_json::from_str(JOBS_SERVICE_API_SOURCE_JSON).expect("parse jobs service API JSON");
    let api = trellis_rs::contracts::ApiBuilder::new(api)
        .build()
        .expect("build jobs service API");
    let mut participant = serde_json::json!({
        "format": "trellis.participant.v1",
        "id": JOBS_SERVICE_ID,
        "displayName": "Trellis Integration Jobs Service",
        "description": "Exercises service-local jobs behind a client-visible RPC.",
        "kind": "service",
        "schemas": api.normalized_value().expect("normalize jobs API")["schemas"],
        "implements": {"self": {"api": JOBS_SERVICE_ID, "apiDigest": api.digest().expect("digest jobs API")}},
        "jobQueues": {
            "processDocument": {"payload": {"schema": "JobPayload"}, "update": {"schema": "JobUpdate"}, "result": {"schema": "JobResult"}, "progress": true, "logs": true},
            "longProcessDocument": {"payload": {"schema": "LongJobPayload"}, "result": {"schema": "JobResult"}},
            "failingProcessDocument": {"payload": {"schema": "FailingJobPayload"}, "result": {"schema": "JobResult"}, "maxDeliver": 2, "backoffMs": [0]},
            "keyedProcessDocument": {"payload": {"schema": "KeyedJobPayload"}, "result": {"schema": "KeyedJobResult"}, "keyConcurrency": {"key": ["/groupKey"], "maxActive": 1, "heartbeatIntervalMs": 1000, "heartbeatTtlMs": 10000, "stalePolicy": "fail-stale"}, "queue": {"maxQueuedPerKey": 1, "whenFull": "reject"}},
            "coalesceKeyedProcessDocument": {"payload": {"schema": "KeyedJobPayload"}, "result": {"schema": "KeyedJobResult"}, "keyConcurrency": {"key": ["/groupKey"], "maxActive": 1, "heartbeatIntervalMs": 1000, "heartbeatTtlMs": 10000, "stalePolicy": "fail-stale"}, "queue": {"maxQueuedPerKey": 1, "whenFull": "coalesce"}},
            "replaceKeyedProcessDocument": {"payload": {"schema": "KeyedJobPayload"}, "result": {"schema": "KeyedJobResult"}, "keyConcurrency": {"key": ["/groupKey"], "maxActive": 1, "heartbeatIntervalMs": 1000, "heartbeatTtlMs": 10000, "stalePolicy": "fail-stale"}, "queue": {"maxQueuedPerKey": 1, "whenFull": "replace-oldest"}},
            "cancelKeyedProcessDocument": {"payload": {"schema": "KeyedJobPayload"}, "result": {"schema": "KeyedJobResult"}, "keyConcurrency": {"key": ["/groupKey"], "maxActive": 1, "heartbeatIntervalMs": 1000, "heartbeatTtlMs": 10000, "stalePolicy": "fail-stale"}, "queue": {"maxQueuedPerKey": 1, "whenFull": "reject"}}
        }
    });
    participant["schemas"] = api.normalized_value().expect("normalize jobs API")["schemas"].clone();
    let service_contract = trellis_test::TrellisTestContract::from_artifacts(
        trellis_rs::contracts::ContractBuilder::from_native(
            api.normalized_value().expect("normalize jobs API"),
            participant,
        )
        .build()
        .expect("build jobs service artifacts"),
    )
    .expect("build jobs service test contract");
    let client_contract =
        jobs_client_contract(&service_contract).expect("build jobs client test contract");

    let service_key = admin
        .provision_service_instance(&bootstrap_url, &service_contract, None, None)
        .await
        .expect("provision live jobs service instance");

    let mut service = trellis_test::connect_service_runtime::<JobsFixtureContract>(
        runtime.trellis_url(),
        &service_key,
    )
    .await
    .expect("connect live Rust jobs service runtime");

    let nats = async_nats::ConnectOptions::new()
        .credentials_file(runtime.workdir().join("nats/creds/trellis-auth.creds"))
        .await
        .expect("load jobs test credentials")
        .connect(runtime.nats_url())
        .await
        .expect("connect jobs test transport");
    let jobs_worker_runtime = service
        .test_jobs_worker_runtime()
        .expect("build jobs worker runtime");
    let jobs_runtime = jobs_worker_runtime.binding().clone();
    let queue_binding = jobs_runtime
        .jobs
        .queues
        .get("processDocument")
        .expect("processDocument queue binding")
        .clone();
    let publisher = NatsJobEventPublisher::new(nats.clone());
    let key_coordinator =
        NatsKeyCoordinator::open_for_service(nats.clone(), jobs_runtime.jobs.namespace.as_str())
            .await
            .expect("open keyed jobs coordinator");
    let manager = JobManager::new_with_key_coordinator(
        publisher,
        jobs_runtime.jobs.clone(),
        TrellisJobMetaSource,
        Arc::new(key_coordinator),
    );
    let waiter = NatsJobWaiter::new(nats.clone(), queue_binding, Duration::from_secs(5));
    let keyed_queue_binding = jobs_runtime
        .jobs
        .queues
        .get("keyedProcessDocument")
        .expect("keyedProcessDocument queue binding")
        .clone();
    let long_queue_binding = jobs_runtime
        .jobs
        .queues
        .get("longProcessDocument")
        .expect("longProcessDocument queue binding")
        .clone();
    let keyed_waiter =
        NatsJobWaiter::new(nats.clone(), keyed_queue_binding, Duration::from_secs(5));
    let coalesce_waiter = NatsJobWaiter::new(
        nats.clone(),
        jobs_runtime.jobs.queues["coalesceKeyedProcessDocument"].clone(),
        Duration::from_secs(5),
    );
    let replace_waiter = NatsJobWaiter::new(
        nats.clone(),
        jobs_runtime.jobs.queues["replaceKeyedProcessDocument"].clone(),
        Duration::from_secs(5),
    );
    let cancel_waiter = NatsJobWaiter::new(
        nats.clone(),
        jobs_runtime.jobs.queues["cancelKeyedProcessDocument"].clone(),
        Duration::from_secs(5),
    );
    let long_waiter = NatsJobWaiter::new(nats.clone(), long_queue_binding, Duration::from_secs(5));
    let keyed_run_state = Arc::new(KeyedJobRunState::default());
    let long_started = Arc::new(tokio::sync::Notify::new());
    let update_release = Arc::new(tokio::sync::Notify::new());
    let failing_attempts = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let process_manager = manager.clone();
    let update_waiter = waiter.clone();
    let fixture_keyed_waiter = keyed_waiter.clone();
    service.register_rpc::<DocumentsProcessRpc, _, _>(move |_context, input| {
        let manager = process_manager.clone();
        let waiter = waiter.clone();
        async move {
            let job = manager
                .create(
                    "processDocument",
                    JobPayload {
                        document_id: input.document_id.clone(),
                    },
                )
                .await
                .map_err(|error| ServerError::Nats(error.to_string()))?;
            let terminal: trellis_rs::jobs::Job = waiter
                .wait_for_terminal(job)
                .await
                .map_err(|error| ServerError::Nats(error.to_string()))?;
            let result_value = terminal
                .result
                .ok_or_else(|| ServerError::Nats("job completed without result".to_string()))?;
            let job_result: JobResult = serde_json::from_value(result_value)
                .map_err(|error| ServerError::Nats(format!("decode job result: {error}")))?;
            Ok(WorkflowOutput {
                document_id: input.document_id,
                job_id: terminal.id,
                processed_by: job_result.processed_by,
                request_id: terminal.context.request_id,
                trace_id: terminal.context.trace_id,
            })
        }
    });

    let keyed_manager = manager.clone();
    service.register_rpc::<DocumentsKeyedProcessRpc, _, _>(move |_context, input| {
        let manager = keyed_manager.clone();
        let waiter = keyed_waiter.clone();
        async move {
            let job = manager
                .create(
                    "keyedProcessDocument",
                    KeyedJobPayload {
                        document_id: input.document_id.clone(),
                        group_key: input.group_key.clone(),
                        sequence: input.sequence,
                    },
                )
                .await
                .map_err(|error| ServerError::Nats(error.to_string()))?;
            let terminal: trellis_rs::jobs::Job = waiter
                .wait_for_terminal(job)
                .await
                .map_err(|error| ServerError::Nats(error.to_string()))?;
            let result_value = terminal
                .result
                .ok_or_else(|| ServerError::Nats("job completed without result".to_string()))?;
            let job_result: KeyedJobResult = serde_json::from_value(result_value)
                .map_err(|error| ServerError::Nats(format!("decode keyed job result: {error}")))?;
            Ok(KeyedWorkflowOutput {
                document_id: input.document_id,
                group_key: input.group_key,
                sequence: input.sequence,
                job_id: terminal.id,
                processed_by: job_result.processed_by,
                request_id: terminal.context.request_id,
                trace_id: terminal.context.trace_id,
            })
        }
    });

    let long_manager = manager.clone();
    let long_started_for_rpc = Arc::clone(&long_started);
    service.register_rpc::<DocumentsSubmitLongProcessRpc, _, _>(move |_context, input| {
        let manager = long_manager.clone();
        let waiter = long_waiter.clone();
        let started = Arc::clone(&long_started_for_rpc);
        async move {
            let job = manager
                .create(
                    "longProcessDocument",
                    JobPayload {
                        document_id: input.document_id.clone(),
                    },
                )
                .await
                .map_err(|error| ServerError::Nats(error.to_string()))?;
            started.notified().await;
            manager
                .cancel(&job)
                .await
                .map_err(|error| ServerError::Nats(error.to_string()))?;
            let terminal: trellis_rs::jobs::Job = waiter
                .wait_for_terminal(job.clone())
                .await
                .map_err(|error| ServerError::Nats(error.to_string()))?;
            Ok(WorkflowOutput {
                document_id: input.document_id,
                job_id: job.id,
                processed_by: job_state_name(terminal.state),
                request_id: terminal.context.request_id,
                trace_id: terminal.context.trace_id,
            })
        }
    });

    let service_handle = service.generated_handle();
    let service_task = AbortOnDrop::new(tokio::spawn(async move { service.run().await }));

    let worker_keyed_run_state = Arc::clone(&keyed_run_state);
    let worker_long_started = Arc::clone(&long_started);
    let worker_failing_attempts = Arc::clone(&failing_attempts);
    let worker_update_release = Arc::clone(&update_release);
    let worker_host = jobs_worker_runtime
        .start(
            "jobs-fixture-service".to_string(),
            |_, _| TrellisJobMetaSource,
            move |active_job: WorkerActiveJob<_, _>| {
                let keyed_run_state = Arc::clone(&worker_keyed_run_state);
                let long_started = Arc::clone(&worker_long_started);
                let failing_attempts = Arc::clone(&worker_failing_attempts);
                let update_release = Arc::clone(&worker_update_release);
                async move {
                    if active_job.job().job_type == "keyedProcessDocument"
                        || active_job.job().job_type.ends_with("KeyedProcessDocument")
                    {
                        let payload: KeyedJobPayload =
                            serde_json::from_value(active_job.job().payload.clone())
                                .map_err(|error| JobProcessError::failed(error.to_string()))?;
                        {
                            let mut started = keyed_run_state.started.lock().await;
                            started.push(payload.sequence);
                        }
                        if payload.sequence % 10 == 1 {
                            keyed_run_state.first_started.notify_one();
                            keyed_run_state.release_first.notified().await;
                        } else if payload.sequence == 99 {
                            keyed_run_state.first_started.notify_one();
                            while !active_job.is_cancelled() {
                                tokio::time::sleep(Duration::from_millis(25)).await;
                            }
                            return Err(JobProcessError::failed("worker stopped".to_string()));
                        } else if !keyed_run_state
                            .released
                            .load(std::sync::atomic::Ordering::SeqCst)
                        {
                            keyed_run_state
                                .second_started_before_release
                                .store(true, std::sync::atomic::Ordering::SeqCst);
                        }
                        {
                            let mut completed = keyed_run_state.completed.lock().await;
                            completed.push(payload.sequence);
                        }
                        let result = serde_json::to_value(KeyedJobResult {
                            document_id: payload.document_id,
                            group_key: payload.group_key,
                            sequence: payload.sequence,
                            processed_by: "rust-service-keyed-job".to_string(),
                            request_id: active_job.context().request_id.clone(),
                            trace_id: active_job.context().trace_id.clone(),
                        })
                        .map_err(|error| JobProcessError::failed(error.to_string()))?;
                        return Ok(result);
                    }

                    if active_job.job().job_type == "longProcessDocument" {
                        let payload: JobPayload =
                            serde_json::from_value(active_job.job().payload.clone())
                                .map_err(|error| JobProcessError::failed(error.to_string()))?;
                        long_started.notify_one();
                        while !active_job.is_cancelled() {
                            active_job
                                .heartbeat()
                                .await
                                .map_err(|error| JobProcessError::failed(error.to_string()))?;
                            tokio::time::sleep(Duration::from_millis(25)).await;
                        }
                        let result = serde_json::to_value(JobResult {
                            document_id: payload.document_id,
                            processed_by: "rust-service-long-job".to_string(),
                            request_id: active_job.context().request_id.clone(),
                            trace_id: active_job.context().trace_id.clone(),
                        })
                        .map_err(|error| JobProcessError::failed(error.to_string()))?;
                        return Ok(result);
                    }

                    if active_job.job().job_type == "failingProcessDocument" {
                        failing_attempts.lock().await.push(active_job.job().tries);
                        return Err(JobProcessError::retryable("retry requested".to_string()));
                    }

                    let payload: JobPayload =
                        serde_json::from_value(active_job.job().payload.clone())
                            .map_err(|error| JobProcessError::failed(error.to_string()))?;
                    if payload.document_id == "doc-update" {
                        update_release.notified().await;
                    }
                    active_job
                        .emit_update::<ProcessDocumentJob>(JobUpdatePayload { processed: 1 })
                        .await
                        .map_err(|error| JobProcessError::failed(error.to_string()))?;
                    active_job
                        .update_progress(1, 1, Some(format!("processed {}", payload.document_id)))
                        .await
                        .map_err(|error| JobProcessError::failed(error.to_string()))?;
                    active_job
                        .log(
                            JobLogLevel::Info,
                            format!("processed {}", payload.document_id),
                        )
                        .await
                        .map_err(|error| JobProcessError::failed(error.to_string()))?;
                    let result = serde_json::to_value(JobResult {
                        document_id: payload.document_id,
                        processed_by: "rust-service-job".to_string(),
                        request_id: active_job.context().request_id.clone(),
                        trace_id: active_job.context().trace_id.clone(),
                    })
                    .map_err(|error| JobProcessError::failed(error.to_string()))?;
                    Ok(result)
                }
            },
            WorkerHostOptions {
                queue_concurrency: BTreeMap::from([
                    ("keyedProcessDocument".to_string(), 2),
                    ("coalesceKeyedProcessDocument".to_string(), 2),
                    ("replaceKeyedProcessDocument".to_string(), 2),
                    ("cancelKeyedProcessDocument".to_string(), 2),
                ]),
                ..WorkerHostOptions::default()
            },
        )
        .await
        .expect("start jobs worker host");

    let client = Arc::new(
        admin
            .connect_client(&bootstrap_url, &client_contract)
            .await
            .expect("connect live Rust jobs client"),
    );

    JobsFixture {
        _runtime: runtime,
        _admin: admin,
        bootstrap_url,
        worker_host,
        service_task,
        service_handle,
        client,
        manager,
        keyed_waiter: fixture_keyed_waiter,
        coalesce_waiter,
        replace_waiter,
        cancel_waiter,
        update_waiter,
        update_release,
        jobs_worker_runtime,
        keyed_run_state,
        failing_attempts,
    }
}

#[tokio::test]
async fn jobs_live_updates_are_typed_and_stop_at_terminal_state() {
    assert_runtime_case_registered(
        "jobs.live-updates-are-typed-and-stop-at-terminal-state",
        "jobs",
        "jobs",
    );
    let fixture = setup_jobs_fixture().await;
    let job = fixture
        .manager
        .create(
            "processDocument",
            JobPayload {
                document_id: "doc-update".to_string(),
            },
        )
        .await
        .expect("create live-update job");
    let mut updates = fixture
        .update_waiter
        .updates::<ProcessDocumentJob>(&job.id)
        .await
        .expect("subscribe to live job updates");
    fixture.update_release.notify_one();
    let update = tokio::time::timeout(Duration::from_secs(5), updates.next())
        .await
        .expect("receive update before timeout")
        .expect("update stream item")
        .expect("valid typed update");
    assert_eq!(update.job_id, job.id);
    assert_eq!(update.attempt, 1);
    assert_eq!(update.sequence, 1);
    assert_eq!(update.update.processed, 1);
    assert!(tokio::time::timeout(Duration::from_secs(5), updates.next())
        .await
        .expect("observe terminal stream closure")
        .is_none());
    fixture.stop().await;
}

impl JobsFixture {
    async fn stop(self) {
        self.worker_host
            .stop()
            .await
            .expect("stop jobs worker host");
        self.service_task.abort_and_wait().await;
    }
}

#[tokio::test]
async fn jobs_service_creates_local_job_from_client_rpc() {
    assert_runtime_case_registered(
        "jobs.service-creates-local-job-from-client-rpc",
        "jobs",
        "jobs",
    );

    let fixture = setup_jobs_fixture().await;
    let output = call_documents_process_with_retry(&fixture.client, "doc-1").await;

    fixture
        .worker_host
        .stop()
        .await
        .expect("stop jobs worker host");
    fixture.service_task.abort_and_wait().await;

    assert_eq!(output.document_id, "doc-1");
    assert!(!output.job_id.is_empty());
}

#[tokio::test]
async fn jobs_keyed_jobs_serialize_same_key() {
    assert_runtime_case_registered("jobs.keyed-jobs-serialize-same-key", "jobs", "jobs");

    let fixture = setup_jobs_fixture().await;
    let first = {
        let client = fixture.client.clone();
        tokio::spawn(async move {
            call_documents_keyed_process_with_retry(&client, "doc-keyed-1", "same-key", 1).await
        })
    };
    fixture.keyed_run_state.first_started.notified().await;
    let second = {
        let client = fixture.client.clone();
        tokio::spawn(async move {
            call_documents_keyed_process_with_retry(&client, "doc-keyed-2", "same-key", 2).await
        })
    };
    fixture
        .keyed_run_state
        .released
        .store(true, std::sync::atomic::Ordering::SeqCst);
    fixture.keyed_run_state.release_first.notify_waiters();

    let first_output = first.await.expect("first keyed workflow joins");
    let second_output = second.await.expect("second keyed workflow joins");

    assert_eq!(first_output.sequence, 1);
    assert_eq!(second_output.sequence, 2);
    assert_eq!(first_output.group_key, "same-key");
    assert_eq!(second_output.group_key, "same-key");
    assert!(!fixture
        .keyed_run_state
        .second_started_before_release
        .load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(*fixture.keyed_run_state.started.lock().await, vec![1, 2]);
    assert_eq!(*fixture.keyed_run_state.completed.lock().await, vec![1, 2]);

    fixture.stop().await;
}

#[tokio::test]
async fn jobs_keyed_active_redelivery_after_restart() {
    assert_runtime_case_registered("jobs.keyed-active-redelivery-after-restart", "jobs", "jobs");

    let fixture = setup_jobs_fixture().await;
    let job = fixture
        .manager
        .create(
            "keyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-keyed-restart".to_string(),
                group_key: "restart".to_string(),
                sequence: 99,
            },
        )
        .await
        .expect("create keyed restart job");
    fixture.keyed_run_state.first_started.notified().await;
    fixture
        .worker_host
        .stop()
        .await
        .expect("stop first jobs worker host");

    let replacement_started = Arc::new(tokio::sync::Notify::new());
    let replacement_signal = Arc::clone(&replacement_started);
    let replacement = fixture
        .jobs_worker_runtime
        .start(
            "jobs-fixture-service-replacement".to_string(),
            |_, _| TrellisJobMetaSource,
            move |active_job: WorkerActiveJob<_, _>| {
                let replacement_signal = Arc::clone(&replacement_signal);
                async move {
                    let payload: KeyedJobPayload =
                        serde_json::from_value(active_job.job().payload.clone())
                            .map_err(|error| JobProcessError::failed(error.to_string()))?;
                    replacement_signal.notify_one();
                    serde_json::to_value(KeyedJobResult {
                        document_id: payload.document_id,
                        group_key: payload.group_key,
                        sequence: payload.sequence,
                        processed_by: "rust-replacement-worker".to_string(),
                        request_id: active_job.context().request_id.clone(),
                        trace_id: active_job.context().trace_id.clone(),
                    })
                    .map_err(|error| JobProcessError::failed(error.to_string()))
                }
            },
            WorkerHostOptions::default(),
        )
        .await
        .expect("start replacement jobs worker host");

    tokio::time::timeout(Duration::from_secs(45), replacement_started.notified())
        .await
        .expect("keyed job redelivered to replacement worker");
    let terminal = fixture
        .keyed_waiter
        .wait_for_terminal(job)
        .await
        .expect("wait for redelivered keyed job");
    assert_eq!(terminal.state, JobState::Completed);

    replacement
        .stop()
        .await
        .expect("stop replacement jobs worker host");
    fixture.service_task.abort_and_wait().await;
}

#[tokio::test]
async fn jobs_keyed_jobs_reject_queue_full() {
    assert_runtime_case_registered("jobs.keyed-jobs-reject-queue-full", "jobs", "jobs");

    let fixture = setup_jobs_fixture().await;
    let group_key = "same-key-full";
    let first = fixture
        .manager
        .create(
            "keyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-keyed-full-1".to_string(),
                group_key: group_key.to_string(),
                sequence: 1,
            },
        )
        .await
        .expect("create first keyed job");
    fixture.keyed_run_state.first_started.notified().await;

    let second = match fixture
        .manager
        .submit(
            "keyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-keyed-full-2".to_string(),
                group_key: group_key.to_string(),
                sequence: 2,
            },
        )
        .await
        .expect("submit second keyed job")
    {
        JobSubmitOutcome::Accepted { job, .. } => job,
        other => panic!("expected second keyed job accepted, got {other:?}"),
    };

    let third = fixture
        .manager
        .submit(
            "keyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-keyed-full-3".to_string(),
                group_key: group_key.to_string(),
                sequence: 3,
            },
        )
        .await
        .expect("submit third keyed job");
    match third {
        JobSubmitOutcome::Rejected(error) => {
            assert_eq!(error.reason, JobNotEnqueuedReason::ActiveLimit);
            assert_eq!(error.key, group_key);
            assert_eq!(error.active, 1);
            assert_eq!(error.queued, 1);
            assert_eq!(error.limit, 1);
        }
        other => panic!("expected third keyed job rejected, got {other:?}"),
    }

    assert_eq!(*fixture.keyed_run_state.started.lock().await, vec![1]);
    assert!(!fixture
        .keyed_run_state
        .second_started_before_release
        .load(std::sync::atomic::Ordering::SeqCst));
    fixture
        .keyed_run_state
        .released
        .store(true, std::sync::atomic::Ordering::SeqCst);
    fixture.keyed_run_state.release_first.notify_waiters();

    let first_terminal = fixture
        .keyed_waiter
        .wait_for_terminal(first)
        .await
        .expect("first keyed job completes");
    let second_terminal = fixture
        .keyed_waiter
        .wait_for_terminal(second)
        .await
        .expect("second keyed job completes");
    assert_eq!(first_terminal.state, JobState::Completed);
    assert_eq!(second_terminal.state, JobState::Completed);
    assert_eq!(*fixture.keyed_run_state.completed.lock().await, vec![1, 2]);

    fixture.stop().await;
}

#[tokio::test]
async fn jobs_keyed_jobs_queue_policies_live() {
    assert_runtime_case_registered("jobs.keyed-jobs-queue-policies-live", "jobs", "jobs");

    let fixture = setup_jobs_fixture().await;

    let coalesce_key = "coalesce-key";
    let coalesce_first = fixture
        .manager
        .create(
            "coalesceKeyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-coalesce-1".to_string(),
                group_key: coalesce_key.to_string(),
                sequence: 11,
            },
        )
        .await
        .expect("create active coalesce job");
    fixture.keyed_run_state.first_started.notified().await;
    let coalesce_second = match fixture
        .manager
        .submit(
            "coalesceKeyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-coalesce-2".to_string(),
                group_key: coalesce_key.to_string(),
                sequence: 12,
            },
        )
        .await
        .expect("submit queued coalesce job")
    {
        JobSubmitOutcome::Accepted { job, .. } => job,
        other => panic!("expected queued coalesce job accepted, got {other:?}"),
    };
    match fixture
        .manager
        .submit(
            "coalesceKeyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-coalesce-3".to_string(),
                group_key: coalesce_key.to_string(),
                sequence: 13,
            },
        )
        .await
        .expect("submit full coalesce job")
    {
        JobSubmitOutcome::Coalesced {
            existing_job_id, ..
        } => assert_eq!(existing_job_id, coalesce_second.id),
        other => panic!("expected coalesced job, got {other:?}"),
    }
    fixture
        .keyed_run_state
        .released
        .store(true, std::sync::atomic::Ordering::SeqCst);
    fixture.keyed_run_state.release_first.notify_waiters();
    assert_eq!(
        fixture
            .coalesce_waiter
            .wait_for_terminal(coalesce_first)
            .await
            .expect("active coalesce job completes")
            .state,
        JobState::Completed
    );
    assert_eq!(
        fixture
            .coalesce_waiter
            .wait_for_terminal(coalesce_second)
            .await
            .expect("queued coalesce job completes")
            .state,
        JobState::Completed
    );
    assert_eq!(
        *fixture.keyed_run_state.completed.lock().await,
        vec![11, 12]
    );

    fixture.keyed_run_state.started.lock().await.clear();
    fixture.keyed_run_state.completed.lock().await.clear();
    fixture
        .keyed_run_state
        .released
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let replace_key = "replace-key";
    let replace_first = fixture
        .manager
        .create(
            "replaceKeyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-replace-1".to_string(),
                group_key: replace_key.to_string(),
                sequence: 21,
            },
        )
        .await
        .expect("create active replace job");
    fixture.keyed_run_state.first_started.notified().await;
    let replace_second = match fixture
        .manager
        .submit(
            "replaceKeyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-replace-2".to_string(),
                group_key: replace_key.to_string(),
                sequence: 22,
            },
        )
        .await
        .expect("submit queued replace job")
    {
        JobSubmitOutcome::Accepted { job, .. } => job,
        other => panic!("expected queued replace job accepted, got {other:?}"),
    };
    let replace_third = match fixture
        .manager
        .submit(
            "replaceKeyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-replace-3".to_string(),
                group_key: replace_key.to_string(),
                sequence: 23,
            },
        )
        .await
        .expect("submit replacement job")
    {
        JobSubmitOutcome::Replaced {
            replaced_job_id,
            job,
            ..
        } => {
            assert_eq!(replaced_job_id, replace_second.id);
            job
        }
        other => panic!("expected replaced job, got {other:?}"),
    };
    assert_eq!(
        fixture
            .replace_waiter
            .wait_for_terminal(replace_second)
            .await
            .expect("replaced queued job becomes terminal")
            .state,
        JobState::Skipped
    );
    fixture
        .keyed_run_state
        .released
        .store(true, std::sync::atomic::Ordering::SeqCst);
    fixture.keyed_run_state.release_first.notify_waiters();
    assert_eq!(
        fixture
            .replace_waiter
            .wait_for_terminal(replace_first)
            .await
            .expect("active replace job completes")
            .state,
        JobState::Completed
    );
    assert_eq!(
        fixture
            .replace_waiter
            .wait_for_terminal(replace_third)
            .await
            .expect("replacement job completes")
            .state,
        JobState::Completed
    );
    assert_eq!(
        *fixture.keyed_run_state.completed.lock().await,
        vec![21, 23]
    );

    fixture.keyed_run_state.started.lock().await.clear();
    fixture.keyed_run_state.completed.lock().await.clear();
    fixture
        .keyed_run_state
        .released
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let cancel_key = "cancel-key";
    let cancel_first = fixture
        .manager
        .create(
            "cancelKeyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-cancel-1".to_string(),
                group_key: cancel_key.to_string(),
                sequence: 31,
            },
        )
        .await
        .expect("create active cancel job");
    fixture.keyed_run_state.first_started.notified().await;
    let cancel_second = match fixture
        .manager
        .submit(
            "cancelKeyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-cancel-2".to_string(),
                group_key: cancel_key.to_string(),
                sequence: 32,
            },
        )
        .await
        .expect("submit queued cancel job")
    {
        JobSubmitOutcome::Accepted { job, .. } => job,
        other => panic!("expected queued cancel job accepted, got {other:?}"),
    };
    fixture
        .manager
        .cancel(&cancel_second)
        .await
        .expect("cancel queued keyed job");
    assert_eq!(
        fixture
            .cancel_waiter
            .wait_for_terminal(cancel_second)
            .await
            .expect("cancelled queued job becomes terminal")
            .state,
        JobState::Cancelled
    );
    let cancel_third = match fixture
        .manager
        .submit(
            "cancelKeyedProcessDocument",
            KeyedJobPayload {
                document_id: "doc-cancel-3".to_string(),
                group_key: cancel_key.to_string(),
                sequence: 33,
            },
        )
        .await
        .expect("submit after queued cancellation")
    {
        JobSubmitOutcome::Accepted { job, .. } => job,
        other => panic!("expected post-cancel job accepted, got {other:?}"),
    };
    fixture
        .keyed_run_state
        .released
        .store(true, std::sync::atomic::Ordering::SeqCst);
    fixture.keyed_run_state.release_first.notify_waiters();
    assert_eq!(
        fixture
            .cancel_waiter
            .wait_for_terminal(cancel_first)
            .await
            .expect("active cancel-policy job completes")
            .state,
        JobState::Completed
    );
    assert_eq!(
        fixture
            .cancel_waiter
            .wait_for_terminal(cancel_third)
            .await
            .expect("post-cancel job completes")
            .state,
        JobState::Completed
    );
    assert_eq!(
        *fixture.keyed_run_state.completed.lock().await,
        vec![31, 33]
    );

    fixture.stop().await;
}

#[tokio::test]
async fn jobs_terminal_local_job_edges_and_admin_rpcs() {
    assert_runtime_case_registered(
        "jobs.terminal-local-job-edges-and-admin-rpcs",
        "jobs",
        "jobs",
    );

    let mut fixture = setup_jobs_fixture().await;
    let job = fixture
        .service_handle
        .generated_submit_job::<ProcessDocumentJob>(JobPayload {
            document_id: "doc-terminal-admin".to_string(),
        })
        .await
        .expect("submit typed service-local job");
    let job_id = job.identity().id.clone();
    let terminal = job.wait().await.expect("wait for service-local job");
    assert_eq!(terminal.state, JobState::Completed);
    assert_eq!(
        terminal.result.expect("completed job result").document_id,
        "doc-terminal-admin"
    );
    let snapshot = job.get().await.expect("get terminal service-local job");
    assert_eq!(snapshot.state, JobState::Completed);
    assert_eq!(
        snapshot
            .progress
            .as_ref()
            .and_then(|progress| progress.current),
        Some(1)
    );
    assert!(snapshot
        .logs
        .iter()
        .any(|entry| entry.message == "processed doc-terminal-admin"));
    assert_eq!(
        job.cancel()
            .await
            .expect("cancel terminal service-local job")
            .state,
        JobState::Completed
    );

    let admin_contract = jobs_admin_client_contract().expect("build Jobs admin contract");
    let admin_participant_id = admin_contract.id().to_owned();
    fixture
        ._admin
        .put_test_login_portal(
            &fixture.bootstrap_url,
            "jobs-admin-mutating",
            &admin_participant_id,
            vec!["local".to_string()],
        )
        .await
        .expect("create Jobs admin test portal");
    let (admin_client, _) = fixture
        ._admin
        .connect_new_trusted_local_user_reconnectable(
            &fixture.bootstrap_url,
            &admin_contract,
            "jobs-admin-mutating",
            "jobs-admin-mutating",
            "JobsAdminMutatingPassword-1",
            vec![
                "trellis.jobs::admin.read".to_string(),
                "trellis.jobs::admin.mutate".to_string(),
            ],
        )
        .await
        .expect("connect mutating Jobs admin client");
    let admin_caller = crate::generated_caller(&admin_client);
    let jobs_admin = trellis_rs::sdk::jobs::JobsClient::new(admin_caller);
    let listed = wait_for_admin_job(
        &jobs_admin,
        &job_id,
        trellis_rs::sdk::jobs::types::JobsQueryResponseEntriesItemState::Completed,
    )
    .await;
    assert_eq!(listed.id, job_id);
    let inspected = jobs_admin
        .rpc()
        .jobs()
        .inspect(&trellis_rs::sdk::jobs::types::JobsInspectRequest { id: job_id.clone() })
        .await
        .expect("inspect terminal job through generated Jobs RPC");
    assert_eq!(
        inspected.job.state,
        trellis_rs::sdk::jobs::types::JobsInspectResponseJobState::Completed
    );
    let cancelled = jobs_admin
        .rpc()
        .jobs()
        .cancel(&trellis_rs::sdk::jobs::types::JobsCancelRequest {
            id: job_id,
            reason: Some("terminal idempotency check".to_string()),
        })
        .await
        .expect("cancel terminal job through generated Jobs RPC");
    assert_eq!(
        cancelled.job.state,
        trellis_rs::sdk::jobs::types::JobsCancelResponseJobState::Completed
    );

    fixture.stop().await;
}

#[tokio::test]
async fn jobs_submitted_job_can_be_cancelled() {
    assert_runtime_case_registered("jobs.submitted-job-can-be-cancelled", "jobs", "jobs");

    let fixture = setup_jobs_fixture().await;
    let output =
        call_documents_submit_long_process_with_retry(&fixture.client, "doc-long-cancel").await;

    assert_eq!(output.document_id, "doc-long-cancel");
    assert_eq!(output.processed_by, "cancelled");

    fixture.stop().await;
}

#[tokio::test]
async fn jobs_failed_job_retries_then_dead() {
    assert_runtime_case_registered("jobs.failed-job-retries-then-dead", "jobs", "jobs");

    let mut fixture = setup_jobs_fixture().await;
    let admin_contract = jobs_admin_client_contract().expect("build jobs admin client contract");
    let admin_client = fixture
        ._admin
        .connect_client(&fixture.bootstrap_url, &admin_contract)
        .await
        .expect("connect live Rust jobs admin client");
    let jobs_admin = trellis_rs::sdk::jobs::JobsClient::new(crate::generated_caller(&admin_client));
    let job = fixture
        .manager
        .create(
            "failingProcessDocument",
            JobPayload {
                document_id: "doc-retry-dead".to_string(),
            },
        )
        .await
        .expect("create failing job");
    let terminal = wait_for_admin_dlq_job(&jobs_admin, &job.id).await;
    let attempts = fixture.failing_attempts.lock().await.clone();

    assert_eq!(terminal.state, "dead");
    assert_eq!(terminal.max_tries, 2);
    assert!(attempts.len() > 1, "handler should be retried");

    fixture.stop().await;
}

#[tokio::test]
async fn jobs_job_progress_and_log_are_published() {
    assert_runtime_case_registered("jobs.job-progress-and-log-are-published", "jobs", "jobs");

    let fixture = setup_jobs_fixture().await;
    let output = call_documents_process_with_retry(&fixture.client, "doc-1").await;

    fixture
        .worker_host
        .stop()
        .await
        .expect("stop jobs worker host");
    fixture.service_task.abort_and_wait().await;

    assert_eq!(output.processed_by, "rust-service-job");
}

#[tokio::test]
async fn jobs_job_wait_returns_typed_result() {
    assert_runtime_case_registered("jobs.job-wait-returns-typed-result", "jobs", "jobs");

    let fixture = setup_jobs_fixture().await;
    let output = call_documents_process_with_retry(&fixture.client, "doc-1").await;

    fixture
        .worker_host
        .stop()
        .await
        .expect("stop jobs worker host");
    fixture.service_task.abort_and_wait().await;

    assert_eq!(output.document_id, "doc-1");
    assert_eq!(output.processed_by, "rust-service-job");
}

#[tokio::test]
async fn jobs_job_context_propagates_request_and_trace() {
    assert_runtime_case_registered(
        "jobs.job-context-propagates-request-and-trace",
        "jobs",
        "jobs",
    );

    let fixture = setup_jobs_fixture().await;
    let output = call_documents_process_with_retry(&fixture.client, "doc-1").await;

    fixture
        .worker_host
        .stop()
        .await
        .expect("stop jobs worker host");
    fixture.service_task.abort_and_wait().await;

    assert!(!output.request_id.is_empty());
    assert_eq!(output.trace_id.len(), 32);
}

#[tokio::test]
async fn jobs_admin_list_services_filters_stale_worker_heartbeats() {
    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();
    let admin_contract = jobs_admin_client_contract().expect("build jobs admin client contract");
    let admin_client = admin
        .connect_client(&bootstrap_url, &admin_contract)
        .await
        .expect("connect live Rust jobs admin client");
    let jobs_admin = trellis_rs::sdk::jobs::JobsClient::new(crate::generated_caller(&admin_client));
    let nc = connect_trellis_nats(runtime.nats_url(), runtime.workdir()).await;

    let fresh_service = "jobs-fixture-service-rust-admin-a";
    let second_fresh_service = "jobs-fixture-service-rust-admin-b";
    let job_type = "processDocument";
    let fresh_instance = "fresh-worker-rust-admin-a";
    let second_fresh_instance = "fresh-worker-rust-admin-b";
    let stale_instance = "stale-worker-rust-admin";

    publish_worker_heartbeat(
        nc.clone(),
        &WorkerHeartbeat {
            service: fresh_service.to_string(),
            job_type: job_type.to_string(),
            instance_id: fresh_instance.to_string(),
            concurrency: None,
            version: None,
            timestamp: timestamp_seconds_ago(0),
        },
    )
    .await
    .expect("publish fresh worker heartbeat");
    publish_worker_heartbeat(
        nc.clone(),
        &WorkerHeartbeat {
            service: second_fresh_service.to_string(),
            job_type: job_type.to_string(),
            instance_id: second_fresh_instance.to_string(),
            concurrency: None,
            version: None,
            timestamp: timestamp_seconds_ago(0),
        },
    )
    .await
    .expect("publish second fresh worker heartbeat");
    publish_worker_heartbeat(
        nc.clone(),
        &WorkerHeartbeat {
            service: fresh_service.to_string(),
            job_type: job_type.to_string(),
            instance_id: stale_instance.to_string(),
            concurrency: None,
            version: None,
            timestamp: timestamp_seconds_ago(300),
        },
    )
    .await
    .expect("publish stale worker heartbeat");
    nc.flush().await.expect("flush worker heartbeats");

    let page = wait_for_admin_services(&jobs_admin, fresh_service, fresh_instance).await;
    let fresh_entry = page
        .entries
        .iter()
        .find(|entry| entry.name == fresh_service)
        .expect("fresh service entry");
    let worker_instances = fresh_entry
        .workers
        .iter()
        .map(|worker| worker.instance_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(worker_instances, vec![fresh_instance]);
    assert_eq!(fresh_entry.workers[0].service, fresh_service);
    assert_eq!(fresh_entry.workers[0].job_type, job_type);

    let first_page = jobs_admin
        .rpc()
        .jobs()
        .list_services(&JobsListServicesRequest {
            offset: None,
            limit: 1,
        })
        .await
        .expect("call first paged Jobs.ListServices");
    assert!(first_page.count >= 2);
    assert_eq!(first_page.next_offset, Some(1));
    let second_page = jobs_admin
        .rpc()
        .jobs()
        .list_services(&JobsListServicesRequest {
            offset: first_page.next_offset,
            limit: 1,
        })
        .await
        .expect("call second paged Jobs.ListServices");
    assert_eq!(second_page.offset, 1);
    assert_ne!(first_page.entries[0].name, second_page.entries[0].name);
}

async fn call_documents_process_with_retry(
    client: &trellis_rs::generated::Caller,
    document_id: &str,
) -> WorkflowOutput {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        match client
            .call::<DocumentsProcessRpc>(&WorkflowInput {
                document_id: document_id.to_string(),
            })
            .await
        {
            Ok(output) => return output,
            Err(error)
                if is_retryable_service_startup_error(&error) && Instant::now() < deadline =>
            {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(error) => panic!("call live Documents.Process RPC: {error}"),
        }
    }
}

async fn call_documents_keyed_process_with_retry(
    client: &trellis_rs::generated::Caller,
    document_id: &str,
    group_key: &str,
    sequence: u64,
) -> KeyedWorkflowOutput {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        match client
            .call::<DocumentsKeyedProcessRpc>(&KeyedWorkflowInput {
                document_id: document_id.to_string(),
                group_key: group_key.to_string(),
                sequence,
            })
            .await
        {
            Ok(output) => return output,
            Err(error)
                if is_retryable_service_startup_error(&error) && Instant::now() < deadline =>
            {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(error) => panic!("call live Documents.KeyedProcess RPC: {error}"),
        }
    }
}

async fn call_documents_submit_long_process_with_retry(
    client: &trellis_rs::generated::Caller,
    document_id: &str,
) -> WorkflowOutput {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        match client
            .call::<DocumentsSubmitLongProcessRpc>(&WorkflowInput {
                document_id: document_id.to_string(),
            })
            .await
        {
            Ok(output) => return output,
            Err(error)
                if is_retryable_service_startup_error(&error) && Instant::now() < deadline =>
            {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(error) => panic!("call live Documents.SubmitLongProcess RPC: {error}"),
        }
    }
}

fn job_state_name(state: JobState) -> String {
    match state {
        JobState::Pending => "pending",
        JobState::Active => "active",
        JobState::Retry => "retry",
        JobState::Completed => "completed",
        JobState::Failed => "failed",
        JobState::Cancelled => "cancelled",
        JobState::Expired => "expired",
        JobState::Skipped => "skipped",
        JobState::Stale => "stale",
        JobState::Dead => "dead",
        JobState::Dismissed => "dismissed",
    }
    .to_string()
}

fn is_retryable_service_startup_error(error: &trellis_rs::generated::TrellisClientError) -> bool {
    match error {
        trellis_rs::generated::TrellisClientError::NatsRequest(message) => {
            message.contains("no responders") || message.contains("NoResponders")
        }
        trellis_rs::generated::TrellisClientError::Timeout => true,
        _ => false,
    }
}

async fn connect_trellis_nats(nats_url: &str, workdir: &Path) -> async_nats::Client {
    async_nats::ConnectOptions::new()
        .credentials_file(
            workdir
                .join("nats")
                .join("creds")
                .join("trellis-auth.creds"),
        )
        .await
        .expect("load Trellis NATS credentials")
        .connect(nats_url)
        .await
        .expect("connect to Trellis test NATS")
}

async fn wait_for_admin_services(
    jobs_admin: &trellis_rs::sdk::jobs::JobsClient<'_>,
    service: &str,
    instance_id: &str,
) -> JobsListServicesResponse {
    let deadline = Instant::now() + Duration::from_secs(60);
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
            Err(error) if is_retryable_jobs_admin_error(&error) && Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            Err(error) => panic!("call generated Jobs.ListServices: {error}"),
        };
        if page.entries.iter().any(|entry| {
            entry.name == service
                && entry
                    .workers
                    .iter()
                    .any(|worker| worker.instance_id == instance_id)
        }) {
            return page;
        }
        assert!(
            Instant::now() < deadline,
            "Jobs.ListServices did not return fresh worker before timeout"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_admin_dlq_job(
    jobs_admin: &trellis_rs::sdk::jobs::JobsClient<'_>,
    job_id: &str,
) -> trellis_rs::sdk::jobs::types::JobsQueryResponseEntriesItem {
    wait_for_admin_job(
        jobs_admin,
        job_id,
        trellis_rs::sdk::jobs::types::JobsQueryResponseEntriesItemState::Dead,
    )
    .await
}

async fn wait_for_admin_job(
    jobs_admin: &trellis_rs::sdk::jobs::JobsClient<'_>,
    job_id: &str,
    state: trellis_rs::sdk::jobs::types::JobsQueryResponseEntriesItemState,
) -> trellis_rs::sdk::jobs::types::JobsQueryResponseEntriesItem {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        let page = match jobs_admin
            .rpc()
            .jobs()
            .query(&JobsQueryRequest {
                group_by: None,
                queue_key: None,
                runtime_band: None,
                search: None,
                service: None,
                sort: None,
                state: Some(vec![crate::wire(state.as_str())]),
                trigger: None,
                r#type: None,
                offset: None,
                limit: 20,
                window: None,
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
            Err(error) => panic!("call generated Jobs.Query: {error}"),
        };
        if let Some(job) = page.entries.into_iter().find(|entry| entry.id == job_id) {
            return job;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "Jobs.Query did not return {state:?} job {job_id} before timeout"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn timestamp_seconds_ago(seconds: i64) -> String {
    (OffsetDateTime::now_utc() - TimeDuration::seconds(seconds))
        .format(&Rfc3339)
        .expect("format worker heartbeat timestamp")
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

fn jobs_admin_client_contract(
) -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractBuilder::authoring(
        JOBS_ADMIN_CLIENT_ID,
        "Trellis Integration Jobs Admin Client",
        "Uses generated Jobs admin RPCs for live acceptance coverage.",
        trellis_rs::contracts::ContractKind::App,
    )
    .use_ref(
        "jobs",
        trellis_rs::contracts::use_contract(trellis_rs::sdk::jobs::API_ID).with_rpc_call([
            "Jobs.Cancel",
            "Jobs.DismissDLQ",
            "Jobs.GetKey",
            "Jobs.Inspect",
            "Jobs.ListDLQ",
            "Jobs.ListServices",
            "Jobs.Metrics",
            "Jobs.Query",
            "Jobs.ReplayDLQ",
            "Jobs.Retry",
        ]),
    );

    let jobs_api = trellis_test::TrellisTestContract::from_native_api_json(
        trellis_rs::sdk::jobs::API_JSON,
        trellis_rs::contracts::ContractKind::Service,
    )?;
    trellis_test::TrellisTestContract::from_builder_with_referenced_contracts(
        manifest,
        &[&jobs_api],
    )
}

fn jobs_client_contract(
    service_contract: &trellis_test::TrellisTestContract,
) -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractBuilder::authoring(
        JOBS_CLIENT_ID,
        "Trellis Integration Jobs Client",
        "App/client participant for the jobs integration fixture.",
        trellis_rs::contracts::ContractKind::App,
    )
    .use_ref(
        "jobsService",
        trellis_rs::contracts::use_contract(JOBS_SERVICE_ID).with_rpc_call([
            "Documents.Process",
            "Documents.KeyedProcess",
            "Documents.SubmitLongProcess",
        ]),
    );

    trellis_test::TrellisTestContract::from_builder_with_referenced_contracts(
        manifest,
        &[service_contract],
    )
}
