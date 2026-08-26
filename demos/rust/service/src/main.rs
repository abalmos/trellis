use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::Engine as _;
use bytes::Bytes;
use clap::Parser;
use futures_util::future::BoxFuture;
use futures_util::stream;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use trellis_participant_demo_service::jobs::RefreshSiteSummaryQueueClient;
use trellis_participant_demo_service::owned::Publisher;
use trellis_participant_demo_service::{
    ConnectedService, ServiceConnectOptions, ServiceHandlerContext,
};
use trellis_rs::client::MemoryAuthorizationContextStore;
use trellis_rs::jobs;
use trellis_rs::service::{
    AcceptedOperation, DownloadTransferGrant, FileTransferInfo, InMemoryOperationRuntime, KvHandle,
    OperationDescriptor, OperationFailure, OperationRefData, OperationSnapshot, OperationState,
    OperationTransferProgress, RequestContext, ServerError, ServiceHandle, ServiceOperation,
    ServiceOperationProvider, StoreHandle, StoreObjectInfo, StoreResourceClient,
    TransferUploadGrantArgs, UploadTransferGrant, UploadTransferSession,
};
use trellis_sdk_demo_service::operations as sdk_operations;
use trellis_sdk_demo_service::types::{
    AssignmentsListRequest, AssignmentsListResponse, AssignmentsListResponseEntriesItem,
    AssignmentsListResponseEntriesItemPriority, AuditRecordedEvent, EvidenceDeleteRequest,
    EvidenceDeleteResponse, EvidenceDownloadRequest, EvidenceDownloadResponse,
    EvidenceDownloadResponseTransfer, EvidenceDownloadResponseTransferDirection,
    EvidenceDownloadResponseTransferInfo, EvidenceDownloadResponseTransferType,
    EvidenceListRequest, EvidenceListResponse, EvidenceListResponseEntriesItem,
    EvidenceUploadInput, EvidenceUploadOutput, EvidenceUploadProgress, EvidenceUploadedEvent,
    ReportsGenerateInput, ReportsGenerateOutput, ReportsGenerateProgress, ReportsListRequest,
    ReportsListResponse, ReportsListResponseEntriesItem, ReportsPublishedEvent, SitesGetRequest,
    SitesGetResponse, SitesGetResponseSite, SitesListRequest, SitesListResponse,
    SitesListResponseEntriesItem, SitesRefreshInput, SitesRefreshOutput, SitesRefreshOutputSite,
    SitesRefreshProgress, SitesRefreshedEvent,
};

#[cfg(any())]
use futures_util::Stream;
#[cfg(any())]
use std::pin::Pin;

const SERVICE_NAME: &str = "rust-field-ops-demo";
const FIXED_NOW: &str = "2026-05-02T00:00:00.000Z";
const TRANSFER_EXPIRES_AT: &str = "2099-01-01T00:00:00.000Z";
const TRANSFER_CHUNK_BYTES: u64 = 65_536;
const UPLOADS_STORE: &str = "uploads";
const MAX_UPLOAD_BYTES: i64 = 10 * 1024 * 1024;
const REQUEST_TIMEOUT_MS: u64 = 5_000;

fn now_iso() -> String {
    match time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339) {
        Ok(value) => value,
        Err(_) => FIXED_NOW.to_string(),
    }
}
const OPERATION_WAIT_TIMEOUT_MS: u64 = 60_000;
const OPERATION_WAIT_POLL_MS: u64 = 100;

#[derive(Debug, Parser)]
struct Args {
    /// Print the generated contract identity and exit.
    #[arg(long)]
    contract: bool,

    /// Trellis HTTP base URL for authenticated service bootstrap.
    #[arg(long, env = "TRELLIS_URL")]
    trellis_url: Option<String>,

    /// Base64url service instance seed for authenticated bootstrap.
    #[arg(long, env = "TRELLIS_SEED")]
    seed: Option<String>,

    /// Provisioned service deployment id.
    #[arg(long, env = "TRELLIS_DEPLOYMENT_ID")]
    deployment_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RuntimeMode {
    Authenticated {
        trellis_url: String,
        deployment_id: String,
        seed: String,
    },
    Idle,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Site {
    site_id: String,
    site_name: String,
    open_inspections: i64,
    overdue_inspections: i64,
    latest_status: String,
    last_report_at: String,
}

#[derive(Debug, Clone)]
struct Assignment {
    inspection_id: String,
    site_id: String,
    site_name: String,
    asset_name: String,
    checklist_name: String,
    priority: String,
    scheduled_for: String,
}

#[derive(Debug, Clone)]
struct Evidence {
    evidence_id: String,
    key: String,
    size: i64,
    content_type: Option<String>,
    evidence_type: String,
    file_name: Option<String>,
    uploaded_at: String,
}

#[derive(Debug, Clone)]
struct PendingUpload {
    evidence_id: String,
}

#[derive(Debug, Default)]
struct AppState {
    assignments: Vec<Assignment>,
    evidence: Vec<Evidence>,
    reports: Vec<ReportsListResponseEntriesItem>,
    operations: BTreeMap<String, serde_json::Value>,
    operation_history: BTreeMap<String, Vec<serde_json::Value>>,
    pending_uploads: BTreeMap<String, PendingUpload>,
    next_operation_sequence: u64,
    next_evidence_sequence: u64,
    next_transfer_sequence: u64,
}

type SharedState = Arc<Mutex<AppState>>;

#[derive(Clone)]
struct AppContext {
    state: SharedState,
    store: EvidenceStore,
    site_summaries: SiteSummaryStore,
    publisher: Publisher,
    refresh_jobs: RefreshSiteSummaryQueueClient,
    refresh_operations: ServiceOperation<sdk_operations::SitesRefreshOperation>,
}

type StartOperation<D> = Arc<
    dyn Fn(
            RequestContext,
            <D as OperationDescriptor>::Input,
        ) -> BoxFuture<
            'static,
            Result<
                AcceptedOperation<
                    <D as OperationDescriptor>::Progress,
                    <D as OperationDescriptor>::Output,
                >,
                ServerError,
            >,
        > + Send
        + Sync,
>;
type ReadOperation<D> = Arc<
    dyn Fn(
            RequestContext,
            String,
        ) -> BoxFuture<
            'static,
            Result<
                OperationSnapshot<
                    <D as OperationDescriptor>::Progress,
                    <D as OperationDescriptor>::Output,
                >,
                ServerError,
            >,
        > + Send
        + Sync,
>;

struct DemoOperationProvider<D: OperationDescriptor> {
    start: StartOperation<D>,
    get: ReadOperation<D>,
    wait: ReadOperation<D>,
    cancel: ReadOperation<D>,
    descriptor: PhantomData<fn() -> D>,
}

impl<D> ServiceOperationProvider<D> for DemoOperationProvider<D>
where
    D: OperationDescriptor + 'static,
{
    fn start(
        &self,
        context: RequestContext,
        input: D::Input,
    ) -> BoxFuture<'static, Result<AcceptedOperation<D::Progress, D::Output>, ServerError>> {
        (self.start)(context, input)
    }

    fn get(
        &self,
        context: RequestContext,
        operation_id: String,
    ) -> BoxFuture<'static, Result<OperationSnapshot<D::Progress, D::Output>, ServerError>> {
        (self.get)(context, operation_id)
    }

    fn wait(
        &self,
        context: RequestContext,
        operation_id: String,
    ) -> BoxFuture<'static, Result<OperationSnapshot<D::Progress, D::Output>, ServerError>> {
        (self.wait)(context, operation_id)
    }

    fn cancel(
        &self,
        context: RequestContext,
        operation_id: String,
    ) -> BoxFuture<'static, Result<OperationSnapshot<D::Progress, D::Output>, ServerError>> {
        (self.cancel)(context, operation_id)
    }
}

#[derive(Debug, Clone)]
struct SiteSummaryStore(KvHandle);

impl SiteSummaryStore {
    async fn seed_missing_sample_sites(&self) -> Result<(), ServerError> {
        for site in sample_sites() {
            self.put(&site).await?;
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Site>, ServerError> {
        let mut sites = Vec::new();
        for key in self.0.list().await? {
            if let Some(value) = self.0.get(&key).await? {
                sites.push(serde_json::from_slice(&value)?);
            }
        }
        sites.sort_by(|left: &Site, right: &Site| left.site_name.cmp(&right.site_name));
        Ok(sites)
    }

    async fn get(&self, site_id: &str) -> Result<Option<Site>, ServerError> {
        self.0
            .get(site_id)
            .await?
            .map(|value| serde_json::from_slice(&value))
            .transpose()
            .map_err(ServerError::from)
    }

    async fn put(&self, site: &Site) -> Result<(), ServerError> {
        self.0
            .put(&site.site_id, Bytes::from(serde_json::to_vec(site)?))
            .await
    }
}

#[derive(Clone)]
struct EvidenceStore {
    inner: StoreHandle,
    state: SharedState,
    upload_evidence_id: Option<String>,
    upload_operation_id: Option<String>,
    publisher: Option<Publisher>,
}

impl std::fmt::Debug for EvidenceStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EvidenceStore")
            .finish_non_exhaustive()
    }
}

impl StoreResourceClient for EvidenceStore {
    async fn read_into<W>(
        &self,
        key: &str,
        writer: &mut W,
    ) -> Result<Option<StoreObjectInfo>, ServerError>
    where
        W: AsyncWrite + Unpin + Send,
    {
        StoreResourceClient::read_into(&self.inner, key, writer).await
    }

    async fn write_from<R>(&self, key: &str, reader: &mut R) -> Result<StoreObjectInfo, ServerError>
    where
        R: AsyncRead + Unpin + Send,
    {
        let info = StoreResourceClient::write_from(&self.inner, key, reader).await?;
        let updated = {
            let mut state = self.state.lock().expect("demo state lock");
            let upload_evidence_id = self.upload_evidence_id.clone().or_else(|| {
                state
                    .pending_uploads
                    .remove(key)
                    .map(|pending| pending.evidence_id)
            });
            let updated = if let Some(evidence) = state.evidence.iter_mut().find(|evidence| {
                upload_evidence_id
                    .as_ref()
                    .is_some_and(|evidence_id| evidence_id == &evidence.evidence_id)
                    || (upload_evidence_id.is_none() && evidence.key == key)
            }) {
                evidence.size = i64::try_from(info.size).unwrap_or(i64::MAX);
                evidence.uploaded_at = now_iso();
                if evidence.file_name.is_none() {
                    evidence.file_name = key.rsplit('/').next().map(ToString::to_string);
                }
                Some(evidence.clone())
            } else {
                None
            };
            tracing::info!(
                key,
                bytes = info.size,
                evidence_id = upload_evidence_id.as_deref().unwrap_or("<unknown>"),
                operation_id = self.upload_operation_id.as_deref().unwrap_or("<none>"),
                "evidence bytes stored"
            );
            if let (Some(operation_id), Some(evidence)) =
                (&self.upload_operation_id, updated.as_ref())
            {
                complete_upload_operation(&mut state, operation_id, evidence);
            }
            updated
        };
        if let Some(evidence) = updated.as_ref() {
            publish_evidence_upload_events(self.publisher.as_ref(), evidence).await;
        }
        Ok(info)
    }

    async fn list(&self) -> Result<Vec<String>, ServerError> {
        self.inner.list().await
    }

    async fn delete(&self, key: &str) -> Result<(), ServerError> {
        self.inner.delete(key).await
    }
}

impl EvidenceStore {
    fn for_upload(&self, evidence_id: String, operation_id: String) -> Self {
        Self {
            state: Arc::clone(&self.state),
            inner: self.inner.clone(),
            upload_evidence_id: Some(evidence_id),
            upload_operation_id: Some(operation_id),
            publisher: self.publisher.clone(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let args = Args::parse();
    if args.contract {
        println!(
            "{} {}",
            trellis_participant_demo_service::contract::CONTRACT_ID,
            trellis_participant_demo_service::contract::CONTRACT_DIGEST
        );
        return Ok(());
    }

    match runtime_mode(&args)? {
        RuntimeMode::Authenticated {
            trellis_url,
            deployment_id,
            seed,
        } => {
            tracing::info!(trellis_url = %trellis_url, "starting authenticated Rust demo service");
            run_authenticated_service(&trellis_url, &deployment_id, &seed).await?
        }
        RuntimeMode::Idle => {
            println!(
                "Rust demo service handlers are ready. Pass --trellis-url and --seed for authenticated bootstrap."
            );
        }
    }
    Ok(())
}

fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new(
            "trellis_rust_demo_service=info,trellis=info,trellis_jobs=info",
        )
    });
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .try_init();
}

fn runtime_mode(args: &Args) -> anyhow::Result<RuntimeMode> {
    if args.trellis_url.is_some() || args.deployment_id.is_some() || args.seed.is_some() {
        let trellis_url = args
            .trellis_url
            .clone()
            .ok_or_else(|| anyhow::anyhow!("--trellis-url is required for authenticated mode"))?;
        let seed = args
            .seed
            .clone()
            .ok_or_else(|| anyhow::anyhow!("--seed is required for authenticated mode"))?;
        let deployment_id = args
            .deployment_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("--deployment-id is required for authenticated mode"))?;
        return Ok(RuntimeMode::Authenticated {
            trellis_url,
            deployment_id,
            seed,
        });
    }

    Ok(RuntimeMode::Idle)
}

async fn run_authenticated_service(
    trellis_url: &str,
    deployment_id: &str,
    seed: &str,
) -> anyhow::Result<()> {
    let session_seed =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>());
    let options = ServiceConnectOptions::new(
        trellis_url,
        SERVICE_NAME,
        deployment_id,
        seed,
        &session_seed,
        Arc::new(MemoryAuthorizationContextStore::default()),
    )
    .with_timeout_ms(REQUEST_TIMEOUT_MS);
    let mut service = trellis_participant_demo_service::connect(options).await?;
    let site_summaries = SiteSummaryStore(service.kv().site_summaries().await?);
    site_summaries.seed_missing_sample_sites().await?;
    let store = service.store().uploads().await?;
    let publisher = service.publisher();
    let state = Arc::new(Mutex::new(sample_state()));
    let context = AppContext {
        state: Arc::clone(&state),
        store: EvidenceStore {
            inner: store,
            state,
            upload_evidence_id: None,
            upload_operation_id: None,
            publisher: Some(publisher.clone()),
        },
        site_summaries: site_summaries.clone(),
        publisher,
        refresh_jobs: service.jobs_client().refresh_site_summary(),
        refresh_operations: InMemoryOperationRuntime::new(SERVICE_NAME)
            .operation::<sdk_operations::SitesRefreshOperation>(),
    };
    service
        .jobs()
        .refresh_site_summary()
        .handle({
            let site_summaries = site_summaries.clone();
            move |job| {
                let site_summaries = site_summaries.clone();
                async move {
                    let input = SitesRefreshInput {
                        site_id: job.payload().site_id.clone(),
                    };
                    let output = refresh_site_summary(
                        site_summaries,
                        input,
                        format!("refresh-{}", job.context().request_id),
                    )
                    .await?;
                    serde_json::from_value(serde_json::to_value(output).map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())
                }
            }
        })
        .await?;
    register_demo_runtime_handlers(&mut service, context);
    tracing::info!("starting Rust demo service request loop");
    service.run().await?;
    Ok(())
}

const _: () = ();

#[cfg(any())]
fn activity_live_stream(
    nats: async_nats::Client,
) -> impl Stream<Item = Result<AuditFeedEvent, ServerError>> + Send + 'static {
    stream::unfold(ActivityLiveStreamState::Init(nats), |state| async move {
        let mut event_stream = match state {
            ActivityLiveStreamState::Init(nats) => {
                match subscribe_activity_live_sources(&nats).await {
                    Ok(stream) => {
                        Box::pin(stream) as Pin<Box<dyn Stream<Item = async_nats::Message> + Send>>
                    }
                    Err(error) => return Some((Err(error), ActivityLiveStreamState::Done)),
                }
            }
            ActivityLiveStreamState::Streaming(stream) => stream,
            ActivityLiveStreamState::Done => return None,
        };

        loop {
            let event_message = event_stream.next().await?;
            let subject = event_message.subject.to_string();
            let Some(name) = activity_live_source_name(&subject) else {
                continue;
            };
            let event = match serde_json::from_slice::<serde_json::Value>(&event_message.payload) {
                Ok(event) => event,
                Err(error) => {
                    return Some((Err(ServerError::Json(error)), ActivityLiveStreamState::Done));
                }
            };
            let frame = AuditFeedEvent(json!({ "name": name, "event": event }));
            return Some((Ok(frame), ActivityLiveStreamState::Streaming(event_stream)));
        }
    })
}

#[cfg(any())]
async fn subscribe_activity_live_sources(
    nats: &async_nats::Client,
) -> Result<impl futures_util::Stream<Item = async_nats::Message>, ServerError> {
    let mut subscribers = Vec::with_capacity(ACTIVITY_LIVE_SOURCE_EVENTS.len());
    for (_, subject) in ACTIVITY_LIVE_SOURCE_EVENTS {
        subscribers.push(
            nats.subscribe((*subject).to_string())
                .await
                .map_err(|error| {
                    ServerError::Nats(format!(
                        "failed to subscribe to Audit.Feed source event {subject}: {error}"
                    ))
                })?,
        );
    }
    nats.flush()
        .await
        .map_err(|error| ServerError::Nats(error.to_string()))?;
    Ok(futures_util::stream::select_all(subscribers))
}

#[cfg(any())]
fn activity_live_source_name(subject: &str) -> Option<&'static str> {
    ACTIVITY_LIVE_SOURCE_EVENTS
        .iter()
        .find_map(|(name, event_subject)| (*event_subject == subject).then_some(*name))
}

#[cfg(any())]
fn build_test_app() -> AppContext {
    build_test_app_with_nats(None)
}

#[cfg(any())]
fn build_test_app_with_nats(nats: Option<async_nats::Client>) -> AppContext {
    build_test_app_with_nats_and_resources(nats, demo_resources())
}

#[cfg(any())]
fn build_test_app_with_nats_and_resources(
    nats: Option<async_nats::Client>,
    resources: ServiceResourceBindings,
) -> AppContext {
    build_test_app_with_nats_resources_and_store(nats, resources, None)
}

#[cfg(any())]
fn build_test_app_with_nats_resources_and_store(
    nats: Option<async_nats::Client>,
    resources: ServiceResourceBindings,
    nats_store: Option<NatsStoreResourceClient>,
) -> AppContext {
    build_test_app_with_nats_resources_store_and_jobs(nats, resources, nats_store, None, None, None)
}

#[cfg(any())]
fn build_test_app_with_nats_resources_store_and_jobs(
    nats: Option<async_nats::Client>,
    resources: ServiceResourceBindings,
    nats_store: Option<NatsStoreResourceClient>,
    nats_site_summaries: Option<NatsKvResourceClient>,
    jobs_nats: Option<async_nats::Client>,
    recorded_jobs: Option<Arc<Mutex<Vec<RecordedJobPublish>>>>,
) -> AppContext {
    build_test_app_with_nats_resources_store_jobs_and_validator(
        nats,
        resources,
        nats_store,
        nats_site_summaries,
        jobs_nats,
        "demo-service-session".to_string(),
        recorded_jobs,
        DemoRequestValidator::allow(),
    )
}

#[cfg(any())]
fn build_test_app_with_nats_resources_store_jobs_and_validator(
    nats: Option<async_nats::Client>,
    resources: ServiceResourceBindings,
    nats_store: Option<NatsStoreResourceClient>,
    nats_site_summaries: Option<NatsKvResourceClient>,
    jobs_nats: Option<async_nats::Client>,
    service_session_key: String,
    recorded_jobs: Option<Arc<Mutex<Vec<RecordedJobPublish>>>>,
    transfer_validator: DemoRequestValidator,
) -> AppContext {
    let demo_store = DemoStore {
        objects: Arc::new(Mutex::new(sample_store_objects())),
    };
    let store = nats_store.map_or_else(
        || SelectedEvidenceStore::Demo(demo_store),
        SelectedEvidenceStore::Runtime,
    );
    build_test_app_with_selected_evidence_store_and_jobs(
        nats,
        resources,
        store,
        nats_site_summaries,
        jobs_nats,
        service_session_key,
        recorded_jobs,
        transfer_validator,
    )
}

#[cfg(any())]
fn build_test_app_with_selected_evidence_store(
    nats: Option<async_nats::Client>,
    resources: ServiceResourceBindings,
    inner: SelectedEvidenceStore,
) -> AppContext {
    build_test_app_with_selected_evidence_store_and_jobs(
        nats,
        resources,
        inner,
        None,
        None,
        "demo-service-session".to_string(),
        None,
        DemoRequestValidator::allow(),
    )
}

#[cfg(any())]
fn build_test_app_with_selected_evidence_store_and_jobs(
    nats: Option<async_nats::Client>,
    resources: ServiceResourceBindings,
    inner: SelectedEvidenceStore,
    nats_site_summaries: Option<NatsKvResourceClient>,
    jobs_nats: Option<async_nats::Client>,
    service_session_key: String,
    recorded_jobs: Option<Arc<Mutex<Vec<RecordedJobPublish>>>>,
    transfer_validator: DemoRequestValidator,
) -> AppContext {
    build_app_context_with_store(
        nats,
        resources,
        inner,
        nats_site_summaries,
        jobs_nats,
        service_session_key,
        recorded_jobs,
        transfer_validator,
    )
}

#[cfg(any())]
fn build_app_context(
    nats: Option<async_nats::Client>,
    resources: ServiceResourceBindings,
    nats_store: Option<NatsStoreResourceClient>,
    nats_site_summaries: Option<NatsKvResourceClient>,
    jobs_nats: Option<async_nats::Client>,
    service_session_key: String,
    recorded_jobs: Option<Arc<Mutex<Vec<RecordedJobPublish>>>>,
    transfer_validator: DemoRequestValidator,
) -> AppContext {
    let demo_store = DemoStore {
        objects: Arc::new(Mutex::new(sample_store_objects())),
    };
    let store = nats_store.map_or_else(
        || SelectedEvidenceStore::Demo(demo_store),
        SelectedEvidenceStore::Runtime,
    );
    build_app_context_with_store(
        nats,
        resources,
        store,
        nats_site_summaries,
        jobs_nats,
        service_session_key,
        recorded_jobs,
        transfer_validator,
    )
}

#[cfg(any())]
fn build_app_context_with_store(
    nats: Option<async_nats::Client>,
    resources: ServiceResourceBindings,
    inner: SelectedEvidenceStore,
    nats_site_summaries: Option<NatsKvResourceClient>,
    jobs_nats: Option<async_nats::Client>,
    service_session_key: String,
    recorded_jobs: Option<Arc<Mutex<Vec<RecordedJobPublish>>>>,
    transfer_validator: DemoRequestValidator,
) -> AppContext {
    let state = Arc::new(Mutex::new(sample_state()));
    let publisher = nats.clone().map(EventPublisher::new);
    let store = EvidenceStore {
        inner,
        state: Arc::clone(&state),
        upload_evidence_id: None,
        upload_operation_id: None,
        publisher: publisher.clone(),
    };
    let use_worker_wait = nats_site_summaries.is_some();
    let site_summaries = nats_site_summaries.map_or_else(
        || SiteSummaryStore::Memory(Arc::clone(&state)),
        SiteSummaryStore::Runtime,
    );
    let refresh_operations = InMemoryOperationRuntime::new(SERVICE_NAME)
        .operation::<sdk_operations::SitesRefreshOperation>();
    AppContext {
        state,
        store,
        site_summaries,
        refresh_jobs: refresh_job_manager(&resources, jobs_nats, recorded_jobs),
        refresh_operations,
        refresh_worker_wait: if use_worker_wait {
            refresh_worker_wait_strategy(&resources, nats.clone())
        } else {
            None
        },
        resources,
        nats,
        publisher,
        service_session_key,
        transfer_validator,
    }
}

fn register_demo_runtime_handlers(service: &mut ConnectedService, context: AppContext) {
    let service_handle = service.generated_handle();
    service.handle().rpc().assignments().list({
        let state = Arc::clone(&context.state);
        move |_ctx, input| assignments_list(Arc::clone(&state), input)
    });
    service.handle().rpc().sites().list({
        let site_summaries = context.site_summaries.clone();
        move |_ctx, input| sites_list(site_summaries.clone(), input)
    });
    service.handle().rpc().sites().get({
        let site_summaries = context.site_summaries.clone();
        move |_ctx, input| sites_get(site_summaries.clone(), input)
    });
    service.handle().rpc().evidence().list({
        let state = Arc::clone(&context.state);
        move |_ctx, input| evidence_list(Arc::clone(&state), input)
    });
    service.handle().rpc().evidence().download({
        let context = context.clone();
        move |ctx, input| evidence_download(context.clone(), ctx, input)
    });
    service.handle().rpc().evidence().delete({
        let context = context.clone();
        move |_ctx, input| evidence_delete(context.clone(), input)
    });
    service.handle().rpc().reports().list({
        let state = Arc::clone(&context.state);
        move |_ctx, input| reports_list(Arc::clone(&state), input)
    });

    service
        .handle()
        .operation()
        .sites()
        .refresh(
            DemoOperationProvider::<sdk_operations::SitesRefreshOperation> {
                start: Arc::new({
                    let context = context.clone();
                    move |ctx, input| Box::pin(sites_refresh_start(context.clone(), ctx, input))
                }),
                get: Arc::new({
                    let operations = context.refresh_operations.clone();
                    move |_ctx, id| {
                        let operations = operations.clone();
                        Box::pin(async move { operations.get(id).await })
                    }
                }),
                wait: Arc::new({
                    let operations = context.refresh_operations.clone();
                    move |_ctx, id| {
                        let operations = operations.clone();
                        Box::pin(async move { operations.wait(id).await })
                    }
                }),
                cancel: Arc::new({
                    let operations = context.refresh_operations.clone();
                    move |_ctx, id| {
                        let operations = operations.clone();
                        Box::pin(async move { operations.cancel(id).await })
                    }
                }),
                descriptor: PhantomData,
            },
        );
    service
        .handle()
        .operation()
        .reports()
        .generate(
            DemoOperationProvider::<sdk_operations::ReportsGenerateOperation> {
                start: Arc::new({
                    let context = context.clone();
                    move |ctx, input| Box::pin(reports_generate_start(context.clone(), ctx, input))
                }),
                get: Arc::new({
                    let state = Arc::clone(&context.state);
                    move |_ctx, id| {
                        Box::pin(operation_get::<
                            ReportsGenerateProgress,
                            ReportsGenerateOutput,
                        >(Arc::clone(&state), id))
                    }
                }),
                wait: Arc::new({
                    let state = Arc::clone(&context.state);
                    move |_ctx, id| {
                        Box::pin(operation_wait::<
                            ReportsGenerateProgress,
                            ReportsGenerateOutput,
                        >(Arc::clone(&state), id))
                    }
                }),
                cancel: Arc::new(move |ctx, id| {
                    Box::pin(operation_cancel::<
                        ReportsGenerateProgress,
                        ReportsGenerateOutput,
                    >(ctx, id))
                }),
                descriptor: PhantomData,
            },
        );
    service
        .handle()
        .operation()
        .evidence()
        .upload(
            DemoOperationProvider::<sdk_operations::EvidenceUploadOperation> {
                start: Arc::new({
                    let context = context.clone();
                    let service_handle = service_handle.clone();
                    move |ctx, input| {
                        Box::pin(evidence_upload_start(
                            context.clone(),
                            service_handle.clone(),
                            ctx,
                            input,
                        ))
                    }
                }),
                get: Arc::new({
                    let state = Arc::clone(&context.state);
                    move |_ctx, id| {
                        Box::pin(
                            operation_get::<EvidenceUploadProgress, EvidenceUploadOutput>(
                                Arc::clone(&state),
                                id,
                            ),
                        )
                    }
                }),
                wait: Arc::new({
                    let state = Arc::clone(&context.state);
                    move |_ctx, id| {
                        Box::pin(
                            operation_wait::<EvidenceUploadProgress, EvidenceUploadOutput>(
                                Arc::clone(&state),
                                id,
                            ),
                        )
                    }
                }),
                cancel: Arc::new(move |ctx, id| {
                    Box::pin(operation_cancel::<
                        EvidenceUploadProgress,
                        EvidenceUploadOutput,
                    >(ctx, id))
                }),
                descriptor: PhantomData,
            },
        );
    service
        .handle()
        .feed()
        .audit()
        .feed(|_ctx, _input| stream::empty());
}

#[cfg(any())]
fn refresh_job_manager(
    resources: &ServiceResourceBindings,
    nats: Option<async_nats::Client>,
    recorded: Option<Arc<Mutex<Vec<RecordedJobPublish>>>>,
) -> RefreshJobManager {
    jobs::JobManager::new(
        DemoJobPublisher { nats, recorded },
        refresh_jobs_binding(resources),
        DemoJobMetaSource::new(),
    )
}

#[cfg(any())]
fn refresh_jobs_binding(resources: &ServiceResourceBindings) -> jobs::JobsBinding {
    if let Some(jobs) = &resources.jobs {
        let queues = jobs
            .queues
            .iter()
            .map(|(queue_type, queue)| {
                (
                    queue_type.clone(),
                    jobs::JobsQueueBinding {
                        queue_type: queue.queue_type.clone(),
                        publish_prefix: queue.publish_prefix.clone(),
                        work_subject: queue.work_subject.clone(),
                        consumer_name: queue.consumer_name.clone(),
                        max_deliver: queue.max_deliver.max(0) as u64,
                        backoff_ms: queue
                            .backoff_ms
                            .iter()
                            .map(|value| (*value).max(0) as u64)
                            .collect(),
                        ack_wait_ms: queue.ack_wait_ms.max(0) as u64,
                        default_deadline_ms: queue
                            .default_deadline_ms
                            .map(|value| value.max(0) as u64),
                        progress: queue.progress,
                        logs: queue.logs,
                    },
                )
            })
            .collect();
        return jobs::JobsBinding {
            namespace: jobs.namespace.clone(),
            queues,
        };
    }

    demo_refresh_jobs_binding()
}

#[cfg(any())]
fn refresh_jobs_runtime_binding(
    resources: &ServiceResourceBindings,
) -> Option<jobs::JobsRuntimeBinding> {
    let work_stream = resources.jobs.as_ref()?.work_stream.clone()?;
    let jobs = refresh_jobs_binding(resources);
    if !jobs.queues.contains_key(REFRESH_SITE_SUMMARY_JOB) {
        return None;
    }
    Some(jobs::JobsRuntimeBinding { jobs, work_stream })
}

#[cfg(any())]
fn refresh_worker_wait_strategy(
    resources: &ServiceResourceBindings,
    nats: Option<async_nats::Client>,
) -> Option<jobs::NatsJobWaiter> {
    let nats = nats?;
    let runtime_binding = refresh_jobs_runtime_binding(resources)?;
    let queue = runtime_binding
        .jobs
        .queues
        .get(REFRESH_SITE_SUMMARY_JOB)?
        .clone();
    Some(jobs::NatsJobWaiter::new(
        nats,
        queue,
        Duration::from_millis(REFRESH_JOB_WAIT_TIMEOUT_MS),
    ))
}

#[cfg(any())]
async fn start_refresh_worker_host(
    nats: async_nats::Client,
    binding: jobs::JobsRuntimeBinding,
    site_summaries: SiteSummaryStore,
) -> anyhow::Result<jobs::WorkerHostHandle> {
    let publisher_nats = nats.clone();
    let worker_site_summaries = site_summaries.clone();
    let host = jobs::start_worker_host_from_binding(
        nats,
        binding,
        format!("{SERVICE_NAME}-refresh-worker"),
        move || DemoJobPublisher {
            nats: Some(publisher_nats.clone()),
            recorded: None,
        },
        |_queue_type, _worker_index| DemoJobMetaSource::new(),
        move |active_job| {
            let site_summaries = worker_site_summaries.clone();
            async move { process_refresh_site_summary_job(site_summaries, active_job).await }
        },
        jobs::WorkerHostOptions {
            queue_types: Some(vec![REFRESH_SITE_SUMMARY_JOB.to_string()]),
            ..jobs::WorkerHostOptions::default()
        },
    )
    .await?;
    Ok(host)
}

const _: () = ();

#[cfg(any())]
fn demo_refresh_jobs_binding() -> jobs::JobsBinding {
    jobs::JobsBinding {
        namespace: SERVICE_NAME.to_string(),
        queues: BTreeMap::from([(
            REFRESH_SITE_SUMMARY_JOB.to_string(),
            jobs::JobsQueueBinding {
                queue_type: REFRESH_SITE_SUMMARY_JOB.to_string(),
                publish_prefix: format!("trellis.jobs.{SERVICE_NAME}.{REFRESH_SITE_SUMMARY_JOB}"),
                work_subject: format!("trellis.work.{SERVICE_NAME}.{REFRESH_SITE_SUMMARY_JOB}"),
                consumer_name: format!("{SERVICE_NAME}-{REFRESH_SITE_SUMMARY_JOB}"),
                max_deliver: 1,
                backoff_ms: Vec::new(),
                ack_wait_ms: 30_000,
                default_deadline_ms: None,
                progress: true,
                logs: false,
            },
        )]),
    }
}

async fn assignments_list(
    state: SharedState,
    input: AssignmentsListRequest,
) -> Result<AssignmentsListResponse, ServerError> {
    let state = state.lock().expect("demo state lock");
    let offset = input.offset.unwrap_or(0);
    let count = state.assignments.len() as i64;
    let next_offset =
        (input.limit > 0 && offset + input.limit < count).then_some(offset + input.limit);
    Ok(AssignmentsListResponse {
        entries: state
            .assignments
            .iter()
            .skip(offset as usize)
            .take(input.limit as usize)
            .map(assignment_to_response)
            .collect(),
        count,
        offset,
        limit: input.limit,
        next_offset,
    })
}

async fn sites_list(
    site_summaries: SiteSummaryStore,
    input: SitesListRequest,
) -> Result<SitesListResponse, ServerError> {
    let offset = input.offset.unwrap_or(0);
    let sites = site_summaries.list().await?;
    let count = sites.len() as i64;
    let next_offset =
        (input.limit > 0 && offset + input.limit < count).then_some(offset + input.limit);
    Ok(SitesListResponse {
        entries: sites
            .iter()
            .skip(offset as usize)
            .take(input.limit as usize)
            .map(site_to_list_response)
            .collect(),
        count,
        offset,
        limit: input.limit,
        next_offset,
    })
}

async fn sites_get(
    site_summaries: SiteSummaryStore,
    input: SitesGetRequest,
) -> Result<SitesGetResponse, ServerError> {
    Ok(SitesGetResponse {
        site: site_summaries
            .get(&input.site_id)
            .await?
            .as_ref()
            .map(site_to_get_response),
    })
}

async fn evidence_list(
    state: SharedState,
    input: EvidenceListRequest,
) -> Result<EvidenceListResponse, ServerError> {
    let state = state.lock().expect("demo state lock");
    let offset = input.offset.unwrap_or(0);
    let filtered: Vec<_> = state
        .evidence
        .iter()
        .filter(|evidence| {
            input
                .prefix
                .as_ref()
                .is_none_or(|prefix| evidence.key.starts_with(prefix))
        })
        .collect();
    let count = filtered.len() as i64;
    let next_offset =
        (input.limit > 0 && offset + input.limit < count).then_some(offset + input.limit);
    Ok(EvidenceListResponse {
        entries: filtered
            .into_iter()
            .skip(offset as usize)
            .take(input.limit as usize)
            .map(evidence_to_response)
            .collect(),
        count,
        offset,
        limit: input.limit,
        next_offset,
    })
}

async fn evidence_download(
    context: AppContext,
    ctx: ServiceHandlerContext,
    input: EvidenceDownloadRequest,
) -> Result<EvidenceDownloadResponse, ServerError> {
    let Some((evidence, transfer_id)) = ({
        let mut state = context.state.lock().expect("demo state lock");
        let evidence = state
            .evidence
            .iter()
            .find(|evidence| evidence.key == input.key)
            .cloned();
        evidence.map(|evidence| {
            let transfer_id = allocate_transfer_id(&mut state, "download");
            (evidence, transfer_id)
        })
    }) else {
        return Err(ServerError::TransferObjectMissing {
            store: UPLOADS_STORE.to_string(),
            key: input.key,
        });
    };
    let plan = ctx.plan_download_transfer(
        UPLOADS_STORE,
        &transfer_id,
        TRANSFER_EXPIRES_AT,
        TRANSFER_CHUNK_BYTES,
        FileTransferInfo {
            key: evidence.key,
            size: evidence.size as u64,
            updated_at: evidence.uploaded_at,
            digest: None,
            content_type: evidence.content_type,
            metadata: BTreeMap::new(),
        },
    )?;
    if context.store.read(&plan.grant.info.key).await?.is_none() {
        return Err(ServerError::TransferObjectMissing {
            store: UPLOADS_STORE.to_string(),
            key: plan.grant.info.key.clone(),
        });
    }

    ctx.handle()
        .spawn_download_transfer_endpoint(plan.clone(), context.store.clone())
        .await?;

    Ok(EvidenceDownloadResponse {
        transfer: download_transfer_to_response(plan.grant),
    })
}

async fn evidence_delete(
    context: AppContext,
    input: EvidenceDeleteRequest,
) -> Result<EvidenceDeleteResponse, ServerError> {
    let key = input.key.clone();
    let deleted = {
        let mut state = context.state.lock().expect("demo state lock");
        let before = state.evidence.len();
        state.evidence.retain(|evidence| evidence.key != key);
        before != state.evidence.len()
    };
    context.store.delete(&key).await?;
    publish_activity_event(
        Some(&context.publisher),
        AuditRecordedEvent {
            activity_id: format!("activity-evidence-deleted-{key}"),
            kind: "evidence-deleted".to_string(),
            message: format!("Deleted evidence upload {key}"),
            occurred_at: now_iso(),
            related_site_id: None,
            related_inspection_id: None,
        },
        "Evidence.Delete activity",
    )
    .await;
    Ok(EvidenceDeleteResponse { key, deleted })
}

async fn reports_list(
    state: SharedState,
    input: ReportsListRequest,
) -> Result<ReportsListResponse, ServerError> {
    let state = state.lock().expect("demo state lock");
    let offset = input.offset.unwrap_or(0);
    let count = state.reports.len() as i64;
    let next_offset =
        (input.limit > 0 && offset + input.limit < count).then_some(offset + input.limit);
    Ok(ReportsListResponse {
        entries: state
            .reports
            .iter()
            .skip(offset as usize)
            .take(input.limit as usize)
            .cloned()
            .collect(),
        count,
        offset,
        limit: input.limit,
        next_offset,
    })
}

async fn sites_refresh_start(
    context: AppContext,
    _ctx: RequestContext,
    input: SitesRefreshInput,
) -> Result<AcceptedOperation<SitesRefreshProgress, SitesRefreshOutput>, ServerError> {
    let operation_id = {
        let mut state = context.state.lock().expect("demo state lock");
        allocate_operation_id(&mut state, "op-sites-refresh")
    };
    let mut accepted = context
        .refresh_operations
        .accept(operation_id.clone())
        .await?;
    let queued = context
        .refresh_operations
        .control(operation_id.clone())
        .await?
        .progress(SitesRefreshProgress {
            stage: "queued".to_string(),
            message: format!("Queued summary refresh for {}", input.site_id),
        })
        .await?;
    accepted.snapshot = queued;
    tracing::info!(
        operation_id = %operation_id,
        site_id = %input.site_id,
        "Sites.Refresh accepted"
    );
    let refresh_operations = context.refresh_operations.clone();
    let operation_id_for_failure = operation_id.clone();
    let context_for_task = context.clone();
    tokio::spawn(async move {
        if let Err(error) = run_sites_refresh(context_for_task, operation_id, input).await {
            if let Ok(control) = refresh_operations.control(operation_id_for_failure).await {
                let _ = control
                    .fail(OperationFailure {
                        message: error.to_string(),
                    })
                    .await;
            }
        }
    });
    Ok(accepted)
}

const _: () = ();

async fn run_sites_refresh(
    context: AppContext,
    operation_id: String,
    input: SitesRefreshInput,
) -> Result<(), ServerError> {
    context
        .refresh_operations
        .control(operation_id.clone())
        .await?
        .progress(SitesRefreshProgress {
            stage: "refreshing".to_string(),
            message: format!("Refreshing field status for {}", input.site_id),
        })
        .await?;
    let job = context
        .refresh_jobs
        .submit(
            trellis_participant_demo_service::jobs::SiteRefreshJobPayload {
                site_id: input.site_id,
            },
        )
        .await
        .map_err(job_wait_error)?;
    let terminal = job.wait().await.map_err(job_wait_error)?;
    let result = terminal.result.ok_or_else(|| {
        ServerError::Nats(format!(
            "refresh job '{}' completed without a result",
            terminal.id
        ))
    })?;
    let output: SitesRefreshOutput = serde_json::from_value(serde_json::to_value(result)?)?;
    let output_for_events = output.clone();
    context
        .refresh_operations
        .control(operation_id)
        .await?
        .complete(output)
        .await?;
    publish_sites_refresh_events(&context, &output_for_events).await;
    Ok(())
}

async fn publish_sites_refresh_events(context: &AppContext, output: &SitesRefreshOutput) {
    let publisher = &context.publisher;

    let refreshed = sites_refreshed_event_from_output(output);
    if let Err(error) = publisher.publish_sites_refreshed(&refreshed).await {
        tracing::warn!(error = %error, "failed to publish Sites.Refreshed");
    }

    let occurred_at = now_iso();
    let activity = AuditRecordedEvent {
        activity_id: format!("activity-refresh-{}", output.site.site_id),
        kind: "site-refreshed".to_string(),
        message: format!("Refreshed {}", output.site.site_name),
        occurred_at,
        related_site_id: Some(output.site.site_id.clone()),
        related_inspection_id: None,
    };
    if let Err(error) = publisher.publish_audit_recorded(&activity).await {
        tracing::warn!(error = %error, "failed to publish Audit.Recorded");
    }
}

#[cfg(any())]
fn refresh_output_from_terminal_job(job: &jobs::Job) -> Result<SitesRefreshOutput, ServerError> {
    match job.state {
        jobs::JobState::Completed => {
            serde_json::from_value(job.result.clone().ok_or_else(|| {
                ServerError::Nats(format!("refresh job '{}' missing result", job.id))
            })?)
            .map_err(ServerError::from)
        }
        _ => Err(ServerError::Nats(format!(
            "refresh job '{}' ended in state {:?}: {}",
            job.id,
            job.state,
            job.last_error.as_deref().unwrap_or("no error detail")
        ))),
    }
}

async fn refresh_site_summary(
    site_summaries: SiteSummaryStore,
    input: SitesRefreshInput,
    refresh_id: String,
) -> Result<SitesRefreshOutput, String> {
    tracing::debug!(site_id = %input.site_id, "refreshing site summary store");
    let mut site = site_summaries
        .get(&input.site_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Unknown site '{}'", input.site_id))?;
    site.last_report_at = FIXED_NOW.to_string();
    site_summaries
        .put(&site)
        .await
        .map_err(|error| error.to_string())?;
    tracing::debug!(site_id = %site.site_id, last_report_at = %site.last_report_at, "site summary stored");
    let output = SitesRefreshOutput {
        refresh_id,
        site: site_to_refresh_output(&site),
        status: "completed".to_string(),
    };
    Ok(output)
}

#[cfg(any())]
fn job_manager_error(error: jobs::JobManagerError<String>) -> ServerError {
    ServerError::Nats(format!("refresh job error: {error}"))
}

fn job_wait_error(error: jobs::JobsError) -> ServerError {
    ServerError::Nats(format!("refresh job wait error: {error}"))
}

async fn reports_generate_start(
    context: AppContext,
    _ctx: RequestContext,
    input: ReportsGenerateInput,
) -> Result<AcceptedOperation<ReportsGenerateProgress, ReportsGenerateOutput>, ServerError> {
    let report_id = format!("closeout-{}", input.inspection_id);
    let inspection_id = input.inspection_id.clone();
    let report_comment = input.report_comment.clone();
    let (accepted_operation, assignment) = {
        let mut state = context.state.lock().expect("demo state lock");
        let assignment = state
            .assignments
            .iter()
            .find(|candidate| candidate.inspection_id == input.inspection_id)
            .cloned();
        let site_id = assignment
            .as_ref()
            .map(|assignment| assignment.site_id.clone());
        let site_name = assignment.as_ref().map_or_else(
            || "Unknown site".to_string(),
            |assignment| assignment.site_name.clone(),
        );
        let asset_name = assignment.as_ref().map_or_else(
            || "Unknown asset".to_string(),
            |assignment| assignment.asset_name.clone(),
        );
        let summary = assignment.as_ref().map_or_else(
            || format!("Closeout report for {}.", input.inspection_id),
            |assignment| {
                format!(
                    "{} closeout for {}.",
                    assignment.checklist_name, assignment.site_name
                )
            },
        );
        state.reports.push(ReportsListResponseEntriesItem {
            report_id: report_id.clone(),
            inspection_id: input.inspection_id.clone(),
            site_id,
            site_name,
            asset_name,
            status: "published".to_string(),
            published_at: now_iso(),
            report_comment: report_comment.trim().to_string(),
            summary,
            readiness: "Site context reconciled before closeout.".to_string(),
            evidence_status: "Evidence review completed in the inspection workflow.".to_string(),
        });

        let accepted_operation = accepted(
            &mut state,
            "Reports.Generate",
            ReportsGenerateOutput {
                report_id: report_id.clone(),
                inspection_id: input.inspection_id,
                status: "published".to_string(),
            },
            ReportsGenerateProgress {
                stage: "complete".to_string(),
                message: "Report generated".to_string(),
            },
        );
        (accepted_operation, assignment)
    };

    publish_reports_generate_events(&context, report_id, inspection_id, assignment).await;
    Ok(accepted_operation)
}

async fn publish_reports_generate_events(
    context: &AppContext,
    report_id: String,
    inspection_id: String,
    assignment: Option<Assignment>,
) {
    let publisher = &context.publisher;

    let published = ReportsPublishedEvent {
        report_id,
        inspection_id: inspection_id.clone(),
        site_id: assignment
            .as_ref()
            .map(|assignment| assignment.site_id.clone()),
        published_at: now_iso(),
    };
    if let Err(error) = publisher.publish_reports_published(&published).await {
        tracing::warn!(error = %error, "failed to publish Reports.Published");
    }

    let inspection_label = assignment.as_ref().map_or_else(
        || inspection_id.clone(),
        |assignment| format!("{} / {}", assignment.site_name, assignment.asset_name),
    );
    let activity = AuditRecordedEvent {
        activity_id: format!("activity-closeout-{inspection_id}"),
        kind: "closeout-published".to_string(),
        message: format!("Published closeout status for {inspection_label}"),
        occurred_at: now_iso(),
        related_site_id: assignment.map(|assignment| assignment.site_id),
        related_inspection_id: Some(inspection_id),
    };
    if let Err(error) = publisher.publish_audit_recorded(&activity).await {
        tracing::warn!(error = %error, "failed to publish Audit.Recorded");
    }
}

async fn publish_evidence_upload_events(publisher: Option<&Publisher>, evidence: &Evidence) {
    let Some(publisher) = publisher else {
        return;
    };

    let uploaded = EvidenceUploadedEvent {
        evidence_id: evidence.evidence_id.clone(),
        key: evidence.key.clone(),
        size: evidence.size,
        content_type: evidence.content_type.clone(),
        file_name: evidence.file_name.clone(),
        evidence_type: evidence.evidence_type.clone(),
        uploaded_at: evidence.uploaded_at.clone(),
    };
    if let Err(error) = publisher.publish_evidence_uploaded(&uploaded).await {
        tracing::warn!(error = %error, "failed to publish Evidence.Uploaded");
    }

    publish_activity_event(
        Some(publisher),
        AuditRecordedEvent {
            activity_id: format!("activity-evidence-uploaded-{}", evidence.evidence_id),
            kind: "evidence-uploaded".to_string(),
            message: format!(
                "Uploaded {} evidence from {}",
                evidence.evidence_type, evidence.key
            ),
            occurred_at: now_iso(),
            related_site_id: None,
            related_inspection_id: None,
        },
        "Evidence.Upload activity",
    )
    .await;
}

async fn publish_activity_event(
    publisher: Option<&Publisher>,
    activity: AuditRecordedEvent,
    context: &str,
) {
    let Some(publisher) = publisher else {
        return;
    };

    if let Err(error) = publisher.publish_audit_recorded(&activity).await {
        tracing::warn!(error = %error, context, "failed to publish Audit.Recorded");
    }
}

async fn evidence_upload_start(
    context: AppContext,
    service_handle: ServiceHandle,
    request: RequestContext,
    input: EvidenceUploadInput,
) -> Result<AcceptedOperation<EvidenceUploadProgress, EvidenceUploadOutput>, ServerError> {
    let (accepted, plan, evidence_id, operation_id) = {
        let mut state = context.state.lock().expect("demo state lock");
        let metadata = input.metadata.clone().unwrap_or_default();
        let file_name = metadata
            .get("fileName")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string)
            .or_else(|| input.key.rsplit('/').next().map(ToString::to_string));
        let evidence_id = if let Some(existing) = state
            .evidence
            .iter_mut()
            .find(|evidence| evidence.key == input.key)
        {
            existing.size = 0;
            existing.content_type = input.content_type.clone();
            existing.evidence_type = input.evidence_type;
            existing.file_name = file_name.clone();
            existing.uploaded_at = FIXED_NOW.to_string();
            existing.evidence_id.clone()
        } else {
            let evidence_id = metadata
                .get("evidenceId")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| allocate_evidence_id(&mut state));
            state.evidence.push(Evidence {
                evidence_id: evidence_id.clone(),
                key: input.key.clone(),
                size: 0,
                content_type: input.content_type.clone(),
                evidence_type: input.evidence_type,
                file_name: file_name.clone(),
                uploaded_at: FIXED_NOW.to_string(),
            });
            evidence_id
        };
        state.pending_uploads.insert(
            input.key.clone(),
            PendingUpload {
                evidence_id: evidence_id.clone(),
            },
        );
        let transfer_id = allocate_transfer_id(&mut state, "upload");

        let session_key =
            request
                .session_key
                .as_deref()
                .ok_or_else(|| ServerError::MissingSessionKey {
                    subject: request.subject.clone(),
                })?;
        let plan = trellis_rs::service::plan_upload_transfer_grant(TransferUploadGrantArgs {
            service_name: service_handle.service_name(),
            session_key,
            service_session_key: service_handle.session_key(),
            resources: service_handle.resources(),
            store: UPLOADS_STORE,
            key: &input.key,
            transfer_id: &transfer_id,
            expires_at: TRANSFER_EXPIRES_AT,
            chunk_bytes: TRANSFER_CHUNK_BYTES,
            max_bytes: Some(MAX_UPLOAD_BYTES as u64),
            content_type: input.content_type.as_deref(),
            metadata: metadata
                .into_iter()
                .map(|(key, value)| {
                    let value = value
                        .as_str()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| value.to_string());
                    (key, value)
                })
                .collect(),
        })?;

        let accepted = accepted_with_transfer_state(
            &mut state,
            "Evidence.Upload",
            OperationState::Running,
            None,
            EvidenceUploadProgress {
                stage: "transfer".to_string(),
                message: "Upload transfer grant is ready".to_string(),
            },
            Some(plan.grant.clone()),
        );
        let operation_id = accepted.operation_ref.id.clone();
        tracing::info!(
            operation_id = %operation_id,
            key = %plan.key,
            transfer_subject = %plan.grant.subject,
            caller_session_prefix = %plan.grant.session_key.chars().take(16).collect::<String>(),
            "Evidence.Upload accepted"
        );
        (accepted, plan, evidence_id, operation_id)
    };

    let session = UploadTransferSession::new(plan, FIXED_NOW);
    let store = context.store.for_upload(evidence_id, operation_id.clone());
    let state = Arc::clone(&context.state);
    service_handle
        .spawn_upload_transfer_endpoint_with_progress(session, store, move |progress| {
            let mut state = state.lock().expect("demo state lock");
            progress_upload_transfer_operation(&mut state, &operation_id, progress);
        })
        .await?;
    Ok(accepted)
}

async fn operation_get<TProgress, TOutput>(
    state: SharedState,
    operation_id: String,
) -> Result<OperationSnapshot<TProgress, TOutput>, ServerError>
where
    TProgress: DeserializeOwned,
    TOutput: DeserializeOwned,
{
    let state = state.lock().expect("demo state lock");
    let value = state
        .operations
        .get(&operation_id)
        .cloned()
        .ok_or_else(|| ServerError::OperationNotFound {
            operation_id: operation_id.clone(),
        })?;
    let snapshot: OperationSnapshot<TProgress, TOutput> = serde_json::from_value(value)?;
    tracing::debug!(
        operation_id = %operation_id,
        state = ?snapshot.state,
        revision = snapshot.revision,
        "operation get"
    );
    Ok(snapshot)
}

async fn operation_wait<TProgress, TOutput>(
    state: SharedState,
    operation_id: String,
) -> Result<OperationSnapshot<TProgress, TOutput>, ServerError>
where
    TProgress: DeserializeOwned,
    TOutput: DeserializeOwned,
{
    let deadline = Instant::now() + Duration::from_millis(OPERATION_WAIT_TIMEOUT_MS);
    tracing::debug!(operation_id = %operation_id, "operation wait started");
    loop {
        let snapshot =
            operation_get::<TProgress, TOutput>(Arc::clone(&state), operation_id.clone()).await?;
        if matches!(
            snapshot.state,
            OperationState::Completed | OperationState::Failed | OperationState::Cancelled
        ) || Instant::now() >= deadline
        {
            tracing::debug!(
                operation_id = %operation_id,
                state = ?snapshot.state,
                revision = snapshot.revision,
                "operation wait returning"
            );
            return Ok(snapshot);
        }
        tokio::time::sleep(Duration::from_millis(OPERATION_WAIT_POLL_MS)).await;
    }
}

async fn operation_cancel<TProgress, TOutput>(
    _ctx: RequestContext,
    _operation_id: String,
) -> Result<OperationSnapshot<TProgress, TOutput>, ServerError> {
    Ok(OperationSnapshot {
        revision: 1,
        state: OperationState::Cancelled,
        progress: None,
        transfer: None,
        output: None,
        ..Default::default()
    })
}

fn accepted<TProgress, TOutput>(
    state: &mut AppState,
    operation: &str,
    output: TOutput,
    progress: TProgress,
) -> AcceptedOperation<TProgress, TOutput>
where
    TProgress: Clone + Serialize,
    TOutput: Clone + Serialize,
{
    accepted_with_transfer(state, operation, output, progress, None)
}

fn accepted_with_transfer<TProgress, TOutput>(
    state: &mut AppState,
    operation: &str,
    output: TOutput,
    progress: TProgress,
    transfer: Option<UploadTransferGrant>,
) -> AcceptedOperation<TProgress, TOutput>
where
    TProgress: Clone + Serialize,
    TOutput: Clone + Serialize,
{
    accepted_with_transfer_state(
        state,
        operation,
        OperationState::Completed,
        Some(output),
        progress,
        transfer,
    )
}

fn accepted_with_transfer_state<TProgress, TOutput>(
    state: &mut AppState,
    operation: &str,
    operation_state: OperationState,
    output: Option<TOutput>,
    progress: TProgress,
    transfer: Option<UploadTransferGrant>,
) -> AcceptedOperation<TProgress, TOutput>
where
    TProgress: Clone + Serialize,
    TOutput: Clone + Serialize,
{
    let operation_id = format!("op-{}", operation.replace('.', "-").to_ascii_lowercase());
    let operation_id = unique_operation_id(state, &operation_id);
    let snapshot = OperationSnapshot {
        id: Some(operation_id.clone()),
        service: Some(SERVICE_NAME.to_string()),
        operation: Some(operation.to_string()),
        revision: 1,
        state: operation_state,
        created_at: Some(now_iso()),
        updated_at: Some(now_iso()),
        progress: Some(progress),
        transfer: None,
        output,
        ..Default::default()
    };
    record_operation_snapshot(state, &operation_id, &snapshot);

    AcceptedOperation {
        kind: "accepted".to_string(),
        operation_ref: OperationRefData {
            id: operation_id,
            service: SERVICE_NAME.to_string(),
            operation: operation.to_string(),
        },
        snapshot,
        transfer,
    }
}

fn record_operation_snapshot<TProgress, TOutput>(
    state: &mut AppState,
    operation_id: &str,
    snapshot: &OperationSnapshot<TProgress, TOutput>,
) where
    TProgress: Serialize,
    TOutput: Serialize,
{
    let mut value =
        serde_json::to_value(snapshot).expect("demo operation snapshot should serialize");
    if let serde_json::Value::Object(ref mut object) = value {
        object
            .entry("id".to_string())
            .or_insert_with(|| serde_json::Value::String(operation_id.to_string()));
        object
            .entry("service".to_string())
            .or_insert_with(|| serde_json::Value::String(SERVICE_NAME.to_string()));
        if !object.contains_key("operation") {
            if let Some(operation) = state
                .operations
                .get(operation_id)
                .and_then(|snapshot| snapshot.get("operation"))
                .cloned()
            {
                object.insert("operation".to_string(), operation);
            }
        }
        if !object.contains_key("createdAt") {
            let created_at = state
                .operations
                .get(operation_id)
                .and_then(|snapshot| snapshot.get("createdAt"))
                .cloned()
                .unwrap_or_else(|| serde_json::Value::String(now_iso()));
            object.insert("createdAt".to_string(), created_at);
        }
        object.insert(
            "updatedAt".to_string(),
            serde_json::Value::String(now_iso()),
        );
        if matches!(
            object.get("state").and_then(serde_json::Value::as_str),
            Some("completed" | "failed" | "cancelled")
        ) {
            object
                .entry("completedAt".to_string())
                .or_insert_with(|| serde_json::Value::String(now_iso()));
        }
        if !object.contains_key("progress") {
            if let Some(progress) = state
                .operations
                .get(operation_id)
                .and_then(|snapshot| snapshot.get("progress"))
                .cloned()
            {
                object.insert("progress".to_string(), progress);
            }
        }
        if !object.contains_key("output") {
            if let Some(output) = state
                .operations
                .get(operation_id)
                .and_then(|snapshot| snapshot.get("output"))
                .cloned()
            {
                object.insert("output".to_string(), output);
            }
        }
    }
    state
        .operations
        .insert(operation_id.to_string(), value.clone());
    state
        .operation_history
        .entry(operation_id.to_string())
        .or_default()
        .push(value);
}

fn next_operation_revision(state: &AppState, operation_id: &str) -> u64 {
    state
        .operation_history
        .get(operation_id)
        .and_then(|history| history.last())
        .and_then(|snapshot| snapshot.get("revision"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        + 1
}

fn complete_upload_operation(state: &mut AppState, operation_id: &str, evidence: &Evidence) {
    tracing::info!(
        operation_id,
        evidence_id = %evidence.evidence_id,
        key = %evidence.key,
        size = evidence.size,
        "Evidence.Upload completed"
    );
    let snapshot = OperationSnapshot {
        revision: next_operation_revision(state, operation_id),
        state: OperationState::Completed,
        progress: Some(EvidenceUploadProgress {
            stage: "indexed".to_string(),
            message: format!("Indexed evidence blocks from {}", evidence.key),
        }),
        transfer: None,
        output: Some(EvidenceUploadOutput {
            evidence_id: evidence.evidence_id.clone(),
            key: evidence.key.clone(),
            size: evidence.size,
            content_type: evidence.content_type.clone(),
            file_name: evidence.file_name.clone(),
            disposition: "ready-for-review".to_string(),
        }),
        ..Default::default()
    };
    record_operation_snapshot(state, operation_id, &snapshot);
}

fn progress_upload_transfer_operation(
    state: &mut AppState,
    operation_id: &str,
    transfer: OperationTransferProgress,
) {
    let snapshot: OperationSnapshot<EvidenceUploadProgress, EvidenceUploadOutput> =
        OperationSnapshot {
            revision: next_operation_revision(state, operation_id),
            state: OperationState::Running,
            progress: None,
            transfer: Some(transfer),
            output: None,
            ..Default::default()
        };
    record_operation_snapshot(state, operation_id, &snapshot);
}

fn unique_operation_id(state: &AppState, base: &str) -> String {
    if !state.operations.contains_key(base) {
        return base.to_string();
    }

    for suffix in 2.. {
        let candidate = format!("{base}-{suffix}");
        if !state.operations.contains_key(&candidate) {
            return candidate;
        }
    }

    unreachable!("unbounded suffix search always returns")
}

fn allocate_operation_id(state: &mut AppState, prefix: &str) -> String {
    state.next_operation_sequence += 1;
    format!("{prefix}-{}", state.next_operation_sequence)
}

fn allocate_evidence_id(state: &mut AppState) -> String {
    let evidence_id = format!("ev-{}", state.next_evidence_sequence);
    state.next_evidence_sequence += 1;
    evidence_id
}

fn allocate_transfer_id(state: &mut AppState, prefix: &str) -> String {
    let transfer_id = format!("{prefix}-{}", state.next_transfer_sequence);
    state.next_transfer_sequence += 1;
    transfer_id
}

const _: () = ();

#[cfg(any())]
#[cfg(any())]
fn demo_resources() -> ServiceResourceBindings {
    let mut store = BTreeMap::new();
    store.insert(
        UPLOADS_STORE.to_string(),
        trellis_rs::service::StoreResourceBinding {
            name: "demo-uploads".to_string(),
            max_object_bytes: Some(MAX_UPLOAD_BYTES),
            max_total_bytes: None,
            ttl_ms: 0,
        },
    );
    ServiceResourceBindings {
        store,
        ..ServiceResourceBindings::default()
    }
}

#[cfg(any())]
async fn spawn_upload_transfer(
    context: &AppContext,
    plan: UploadTransferGrantPlan,
    evidence_id: String,
    operation_id: String,
) -> Result<(), ServerError> {
    let Some(nats) = context.nats.clone() else {
        tracing::debug!(operation_id = %operation_id, "upload transfer endpoint skipped without NATS");
        return Ok(());
    };
    tracing::info!(
        operation_id = %operation_id,
        evidence_id = %evidence_id,
        subject = %plan.grant.subject,
        "starting upload transfer endpoint"
    );
    let session = UploadTransferSession::new(plan, FIXED_NOW);
    let store = context.store.for_upload(evidence_id, operation_id.clone());
    let validator = context.transfer_validator.clone();
    let state = Arc::clone(&context.state);
    spawn_upload_transfer_endpoint_with_progress(nats, session, store, validator, move |progress| {
        let mut state = state.lock().expect("demo state lock");
        progress_upload_transfer_operation(&mut state, &operation_id, progress);
    })
    .await
}

#[cfg(any())]
async fn spawn_download_transfer(
    context: &AppContext,
    plan: DownloadTransferGrantPlan,
) -> Result<(), ServerError> {
    let Some(nats) = context.nats.clone() else {
        tracing::debug!(subject = %plan.grant.subject, "download transfer endpoint skipped without NATS");
        return Ok(());
    };
    tracing::info!(subject = %plan.grant.subject, key = %plan.grant.info.key, "starting download transfer endpoint");
    let store = context.store.clone();
    let validator = context.transfer_validator.clone();
    spawn_download_transfer_endpoint(nats, plan, store, validator).await
}

fn download_transfer_to_response(grant: DownloadTransferGrant) -> EvidenceDownloadResponseTransfer {
    EvidenceDownloadResponseTransfer {
        r#type: EvidenceDownloadResponseTransferType::TransferGrant,
        direction: EvidenceDownloadResponseTransferDirection::Receive,
        service: grant.service,
        session_key: grant.session_key,
        transfer_id: grant.transfer_id,
        subject: grant.subject,
        expires_at: grant.expires_at,
        chunk_bytes: grant.chunk_bytes as i64,
        info: EvidenceDownloadResponseTransferInfo {
            key: grant.info.key,
            size: grant.info.size as i64,
            updated_at: grant.info.updated_at,
            digest: grant.info.digest,
            content_type: grant.info.content_type,
            metadata: grant
                .info
                .metadata
                .into_iter()
                .map(|(key, value)| (key, serde_json::Value::String(value)))
                .collect(),
        },
    }
}

fn sample_state() -> AppState {
    AppState {
        assignments: vec![
            Assignment {
                inspection_id: "insp-west-001".to_string(),
                site_id: "site-west-yard".to_string(),
                site_name: "West Yard".to_string(),
                asset_name: "Pump Station 7".to_string(),
                checklist_name: "Leak and vibration check".to_string(),
                priority: "high".to_string(),
                scheduled_for: "2026-04-18T09:00:00.000Z".to_string(),
            },
            Assignment {
                inspection_id: "insp-ridge-002".to_string(),
                site_id: "site-ridge-line".to_string(),
                site_name: "Ridge Line".to_string(),
                asset_name: "Backup Generator 2".to_string(),
                checklist_name: "Run test and battery review".to_string(),
                priority: "medium".to_string(),
                scheduled_for: "2026-04-18T13:30:00.000Z".to_string(),
            },
            Assignment {
                inspection_id: "insp-harbor-003".to_string(),
                site_id: "site-harbor-gate".to_string(),
                site_name: "Harbor Gate".to_string(),
                asset_name: "Security Gate Controller".to_string(),
                checklist_name: "Ingress log verification".to_string(),
                priority: "low".to_string(),
                scheduled_for: "2026-04-19T08:15:00.000Z".to_string(),
            },
        ],
        evidence: vec![Evidence {
            evidence_id: "ev-1001".to_string(),
            key: "site-north/transformer-a/photo.txt".to_string(),
            size: 42,
            content_type: Some("text/plain".to_string()),
            evidence_type: "photo".to_string(),
            file_name: Some("photo.txt".to_string()),
            uploaded_at: "2026-05-01T12:00:00.000Z".to_string(),
        }],
        reports: Vec::new(),
        operations: BTreeMap::new(),
        operation_history: BTreeMap::new(),
        pending_uploads: BTreeMap::new(),
        next_operation_sequence: 1,
        next_evidence_sequence: 1002,
        next_transfer_sequence: 1,
    }
}

fn sample_sites() -> Vec<Site> {
    vec![
        Site {
            site_id: "site-west-yard".to_string(),
            site_name: "West Yard".to_string(),
            open_inspections: 3,
            overdue_inspections: 1,
            latest_status: "attention-needed".to_string(),
            last_report_at: "2026-04-17T18:12:00.000Z".to_string(),
        },
        Site {
            site_id: "site-ridge-line".to_string(),
            site_name: "Ridge Line".to_string(),
            open_inspections: 2,
            overdue_inspections: 0,
            latest_status: "on-track".to_string(),
            last_report_at: "2026-04-17T11:45:00.000Z".to_string(),
        },
        Site {
            site_id: "site-harbor-gate".to_string(),
            site_name: "Harbor Gate".to_string(),
            open_inspections: 1,
            overdue_inspections: 0,
            latest_status: "ready".to_string(),
            last_report_at: "2026-04-16T15:05:00.000Z".to_string(),
        },
    ]
}

const _: () = ();

fn assignment_to_response(assignment: &Assignment) -> AssignmentsListResponseEntriesItem {
    AssignmentsListResponseEntriesItem {
        inspection_id: assignment.inspection_id.clone(),
        site_id: assignment.site_id.clone(),
        site_name: assignment.site_name.clone(),
        asset_name: assignment.asset_name.clone(),
        checklist_name: assignment.checklist_name.clone(),
        priority: match assignment.priority.as_str() {
            "high" => AssignmentsListResponseEntriesItemPriority::High,
            "medium" => AssignmentsListResponseEntriesItemPriority::Medium,
            _ => AssignmentsListResponseEntriesItemPriority::Low,
        },
        scheduled_for: assignment.scheduled_for.clone(),
    }
}

fn site_to_list_response(site: &Site) -> SitesListResponseEntriesItem {
    SitesListResponseEntriesItem {
        site_id: site.site_id.clone(),
        site_name: site.site_name.clone(),
        open_inspections: site.open_inspections,
        overdue_inspections: site.overdue_inspections,
        latest_status: site.latest_status.clone(),
        last_report_at: site.last_report_at.clone(),
    }
}

fn site_to_get_response(site: &Site) -> SitesGetResponseSite {
    SitesGetResponseSite {
        site_id: site.site_id.clone(),
        site_name: site.site_name.clone(),
        open_inspections: site.open_inspections,
        overdue_inspections: site.overdue_inspections,
        latest_status: site.latest_status.clone(),
        last_report_at: site.last_report_at.clone(),
    }
}

fn site_to_refresh_output(site: &Site) -> SitesRefreshOutputSite {
    SitesRefreshOutputSite {
        site_id: site.site_id.clone(),
        site_name: site.site_name.clone(),
        open_inspections: site.open_inspections,
        overdue_inspections: site.overdue_inspections,
        latest_status: site.latest_status.clone(),
        last_report_at: site.last_report_at.clone(),
    }
}

fn evidence_to_response(evidence: &Evidence) -> EvidenceListResponseEntriesItem {
    EvidenceListResponseEntriesItem {
        evidence_id: evidence.evidence_id.clone(),
        key: evidence.key.clone(),
        size: evidence.size,
        content_type: evidence.content_type.clone(),
        evidence_type: evidence.evidence_type.clone(),
        file_name: evidence.file_name.clone(),
        uploaded_at: evidence.uploaded_at.clone(),
    }
}

fn sites_refreshed_event_from_output(output: &SitesRefreshOutput) -> SitesRefreshedEvent {
    SitesRefreshedEvent {
        refresh_id: output.refresh_id.clone(),
        site: trellis_sdk_demo_service::types::SitesRefreshedEventSite {
            site_id: output.site.site_id.clone(),
            site_name: output.site.site_name.clone(),
            open_inspections: output.site.open_inspections,
            overdue_inspections: output.site.overdue_inspections,
            latest_status: output.site.latest_status.clone(),
            last_report_at: output.site.last_report_at.clone(),
        },
        refreshed_at: now_iso(),
    }
}
