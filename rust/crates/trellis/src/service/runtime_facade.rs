//! High-level Trellis service runtime facade for generated Rust services.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::task::{Context, Poll};
use std::time::Duration;

use async_nats::header::HeaderMap;
use bytes::Bytes;
use futures_util::future::BoxFuture;
use futures_util::{Stream, StreamExt};
use tokio::task::{AbortHandle, JoinError, JoinHandle};

pub use super::core_bootstrap::CoreBootstrapBinding;
use super::resources::{validate_kv_binding, validate_store_binding, ResourceRuntimeClient};
use super::resources::{KvHandle, KvResourceHandle, StoreHandle, StoreResourceHandle};
use super::runtime::run_multi_subject_service;
use super::transfer::{
    spawn_download_transfer_endpoint, spawn_upload_transfer_endpoint_with_completion,
    spawn_upload_transfer_endpoint_with_progress,
};
use super::{
    bootstrap_service_host, control_subject, BootstrapBindingInfo, DownloadTransferGrantPlan,
    EventPublisher, FeedDescriptor, HandlerResult, JobsResourceBinding, KvResourceBinding,
    OperationDescriptor, OperationTransferProgress, RequestContext, Router, RpcDescriptor,
    ServerError, ServiceOperationProvider, ServiceResourceBindings, StoreResourceBinding,
    StoreResourceClient, UploadTransferCompletion, UploadTransferSession,
};

use crate::client::{
    AuthorizationContextStore, EventMessage, EventReplayPolicy, EventSubscribeOptions,
    EventSubscriptionMode, ServiceConnectWithContractOptions, TrellisClient, TrellisClientError,
};
use crate::jobs::{
    start_worker_host_from_client, JobDescriptor, JobManager, JobProcessError, JobRef, JobsError,
    TrellisJobEventPublisher, TrellisJobMetaSource, WorkerHostHandle, WorkerHostOptions,
};
use crate::service::local_validator::LocalAuthVerifier;

const DURABLE_EVENT_CONSUMER_RETRY_MS: u64 = 100;
static SERVICE_EVENT_HANDLER_ID: AtomicU64 = AtomicU64::new(1);

type SharedDurableEventListeners =
    Arc<StdMutex<BTreeMap<DurableEventListenerKey, SharedDurableEventListener>>>;
type SharedEventHandler = Arc<
    dyn Fn(
            Bytes,
            ServiceEventListenerContext,
        ) -> BoxFuture<'static, Result<(), ServiceRuntimeError>>
        + Send
        + Sync,
>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct DurableEventListenerKey {
    stream: String,
    durable_name: String,
}

struct SharedDurableEventListener {
    expected_subjects: BTreeSet<String>,
    handlers: BTreeMap<String, BTreeMap<u64, SharedEventHandler>>,
    concurrency: u32,
    pull_abort_handles: Vec<AbortHandle>,
}

#[derive(Clone)]
struct ServiceEventListenerRegistration {
    event_listeners: SharedDurableEventListeners,
    key: DurableEventListenerKey,
    subject: String,
    handler_id: u64,
}

struct ServiceEventListenerRegistryCleanup {
    event_listeners: SharedDurableEventListeners,
}

impl ServiceEventListenerRegistryCleanup {
    fn new(event_listeners: SharedDurableEventListeners) -> Self {
        Self { event_listeners }
    }
}

impl Drop for ServiceEventListenerRegistryCleanup {
    fn drop(&mut self) {
        remove_service_event_listeners(&self.event_listeners);
    }
}

/// Default request/connect timeout for service bootstrap and NATS RPC calls.
pub const DEFAULT_TIMEOUT_MS: u64 = 5_000;

/// Default retry delay while service deployment authority is pending.
pub const DEFAULT_RETRY_DELAY_MS: u64 = 1_000;

/// Default authority-pending wait limit. `None` waits until authority is ready.
pub const DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS: Option<u64> = None;

/// Native participant and API evidence emitted by generated Rust participant facades.
///
/// Service and device facades use this as their sole source of exact contract evidence.
pub trait GeneratedServiceContract {
    /// Trellis participant id, for example `example.service@v1`.
    const PARTICIPANT_ID: &'static str;

    /// Content digest for the generated participant artifact.
    const CONTRACT_DIGEST: &'static str;

    /// Digest of the participant needs resolution.
    const PARTICIPANT_NEEDS_DIGEST: &'static str;

    /// Canonical participant artifact JSON presented during service bootstrap.
    const PARTICIPANT_JSON: &'static str;

    /// Canonical owned API artifact JSON presented during service bootstrap.
    const API_JSON: &'static str;

    /// Digest of the owned API artifact.
    const API_DIGEST: &'static str;

    /// Exact referenced API JSON and digest evidence.
    const REFERENCED_API_ARTIFACTS: &'static [(&'static str, &'static str)];
}

/// High-level options for connecting a generated Rust service runtime.
#[derive(Debug, Clone)]
pub struct ServiceConnectOptions<'a> {
    /// Base Trellis runtime URL used for HTTP bootstrap.
    trellis_url: &'a str,
    /// Service instance name reported to the runtime.
    name: &'a str,
    /// Deployment that owns the service instance.
    deployment_id: &'a str,
    /// Base64url-encoded provisioned service identity seed.
    provisioned_identity_seed_base64url: &'a str,
    /// Base64url-encoded service session seed.
    session_key_seed_base64url: Cow<'a, str>,
    /// Request/connect timeout in milliseconds.
    timeout_ms: u64,
    /// Retry delay in milliseconds while bootstrap is pending authority readiness.
    retry_delay_ms: u64,
    /// Optional maximum authority-pending wait time. `None` waits until authority is ready.
    authority_pending_timeout_ms: Option<u64>,
    /// Caller-owned durable context and trust-floor storage.
    authorization_context_store: Arc<dyn AuthorizationContextStore>,
}

impl<'a> ServiceConnectOptions<'a> {
    /// Create service connect options with ergonomic default timeouts.
    pub fn new(
        trellis_url: &'a str,
        name: &'a str,
        deployment_id: &'a str,
        provisioned_identity_seed_base64url: &'a str,
        session_key_seed_base64url: &'a str,
        authorization_context_store: Arc<dyn AuthorizationContextStore>,
    ) -> Self {
        Self {
            trellis_url,
            name,
            deployment_id,
            provisioned_identity_seed_base64url,
            session_key_seed_base64url: Cow::Borrowed(session_key_seed_base64url),
            timeout_ms: DEFAULT_TIMEOUT_MS,
            retry_delay_ms: DEFAULT_RETRY_DELAY_MS,
            authority_pending_timeout_ms: DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS,
            authorization_context_store,
        }
    }

    /// Replace the session seed, allowing each service process start to use a fresh session key.
    pub fn with_session_key_seed(
        mut self,
        session_key_seed_base64url: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.session_key_seed_base64url = session_key_seed_base64url.into();
        self
    }

    /// Set the request/connect timeout in milliseconds.
    pub const fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set the delay between authority-pending bootstrap retries.
    pub const fn with_retry_delay_ms(mut self, retry_delay_ms: u64) -> Self {
        self.retry_delay_ms = retry_delay_ms;
        self
    }

    /// Limit authority-pending bootstrap wait time, or use `None` to wait indefinitely.
    pub const fn with_authority_pending_timeout_ms(mut self, timeout_ms: Option<u64>) -> Self {
        self.authority_pending_timeout_ms = timeout_ms;
        self
    }
}

/// Errors returned by the high-level service runtime facade.
#[derive(Debug, thiserror::Error)]
pub enum ServiceRuntimeError {
    /// Client-side bootstrap, transport, or outbound RPC failure.
    #[error(transparent)]
    Client(#[from] TrellisClientError),

    /// Server-side handler, auth-validation, or runtime-loop failure.
    #[error(transparent)]
    Server(Box<ServerError>),

    /// A service event listener handler failed while processing a concrete event message.
    #[error("event handler failed: {source}")]
    EventHandler {
        /// Handler failure returned by the service implementation.
        source: Box<ServerError>,
        /// Event metadata observed from the delivered message.
        context: Box<ServiceEventListenerContext>,
    },

    /// The service bootstrap response did not include a resource binding.
    #[error("service bootstrap response did not include a binding")]
    MissingBootstrapBinding,

    /// The service bootstrap binding could not be parsed as a core binding.
    #[error("invalid service bootstrap binding: {0}")]
    InvalidBootstrapBinding(#[source] serde_json::Error),

    /// Service-private jobs bindings were missing or invalid.
    #[error(transparent)]
    JobsBinding(#[from] crate::jobs::bindings::JobsBindingError),

    /// A service-private jobs worker host failed.
    #[error(transparent)]
    JobWorker(#[from] crate::jobs::internal::WorkerHostError),

    /// A generated jobs queue was not present in the resolved binding.
    #[error("jobs queue '{queue_type}' was not found in service bootstrap bindings")]
    MissingJobQueue {
        /// Declared queue type absent from the binding.
        queue_type: String,
    },

    /// A durable event listener supplied a caller-owned durable name.
    #[error(
        "durable event consumer names are provisioned by Trellis event consumer bindings; remove caller durable name '{durable_name}'"
    )]
    CallerDurableName {
        /// Caller-provided durable consumer name.
        durable_name: String,
    },

    /// No durable event consumer group was declared for the requested event subject.
    #[error("event subject '{subject}' is not declared in any event consumer group")]
    MissingEventConsumerGroup {
        /// Event subject requested by the listener.
        subject: String,
    },

    /// More than one durable event consumer group matched the requested event subject.
    #[error(
        "event subject '{subject}' is declared in multiple event consumer groups: {}; specify a group",
        groups.join(", ")
    )]
    AmbiguousEventConsumerGroup {
        /// Event subject requested by the listener.
        subject: String,
        /// Matching group names.
        groups: Vec<String>,
    },

    /// The requested event consumer group is not present in the bootstrap binding.
    #[error("event consumer group '{group}' was not found in service bootstrap bindings")]
    EventConsumerGroupNotFound {
        /// Requested event consumer group name.
        group: String,
    },

    /// The requested event consumer group does not include the event subject.
    #[error("event consumer group '{group}' does not include event subject '{subject}'")]
    EventConsumerGroupSubjectMismatch {
        /// Requested event consumer group name.
        group: String,
        /// Event subject requested by the listener.
        subject: String,
    },

    /// A durable listener count must be at least one.
    #[error("event consumer group '{group}' has invalid listener concurrency {concurrency}; expected >= 1")]
    InvalidEventListenerConcurrency {
        /// Event consumer group name.
        group: String,
        /// Invalid requested listener count.
        concurrency: u32,
    },

    /// Strict ordering permits only one pull loop per service instance.
    #[error(
        "event consumer group '{group}' uses strict ordering and requires listener concurrency 1"
    )]
    StrictEventListenerConcurrency {
        /// Strictly ordered event consumer group name.
        group: String,
    },

    /// Registrations sharing one durable consumer must use the same local count.
    #[error(
        "event consumer group '{group}' already uses listener concurrency {existing}; requested {requested}"
    )]
    EventListenerConcurrencyMismatch {
        /// Event consumer group name.
        group: String,
        /// Listener count already registered locally.
        existing: u32,
        /// Conflicting requested listener count.
        requested: u32,
    },
}

impl From<ServerError> for ServiceRuntimeError {
    fn from(source: ServerError) -> Self {
        Self::Server(Box::new(source))
    }
}

/// Options for registering a service event listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceEventListenOptions {
    /// Listener delivery mode. Durable listeners use Trellis-provisioned bindings by default.
    pub mode: ServiceEventListenerMode,
    /// Contract-local event consumer group name. Required when more than one group matches.
    pub group: Option<String>,
    /// Caller-provided durable names are rejected because Trellis owns durable consumers.
    pub durable_name: Option<String>,
    /// Number of local pull loops for a parallel durable consumer group.
    pub concurrency: u32,
}

impl Default for ServiceEventListenOptions {
    fn default() -> Self {
        Self {
            mode: ServiceEventListenerMode::Durable,
            group: None,
            durable_name: None,
            concurrency: 1,
        }
    }
}

/// Runtime context passed to service event listener handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceEventListenerContext {
    /// Listener delivery mode.
    pub mode: ServiceEventListenerMode,
    /// Contract-local event consumer group selected for durable listeners.
    pub group: Option<String>,
    /// Trellis event id from the `Nats-Msg-Id` header, when present.
    pub id: Option<String>,
    /// Trellis event timestamp from the `Trellis-Event-Time` header, when present.
    pub time: Option<String>,
    /// W3C traceparent propagated with the event, when present.
    pub traceparent: Option<String>,
    /// Raw event transport headers delivered with the message.
    pub headers: HeaderMap,
    /// Verified publisher metadata from local event verification, when available.
    pub publisher: Option<ServiceEventPublisherContext>,
}

/// Verified event publisher metadata produced by local event verification.
///
/// The publisher projection is derived from the verified authorization
/// context bound into the event proof: principal kind, deployment/instance
/// identity, participant contract identity, and the active session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceEventPublisherContext {
    /// Publisher participant kind.
    pub kind: String,
    /// Publisher deployment id, when the publisher is deployment-backed.
    pub deployment_id: Option<String>,
    /// Publisher runtime instance id, when known.
    pub instance_id: Option<String>,
    /// Publisher contract id, when known.
    pub contract_id: Option<String>,
    /// Publisher contract digest, when known.
    pub contract_digest: Option<String>,
    /// Retained session lifecycle status used for validation.
    pub session_status: String,
}

/// Event listener delivery mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceEventListenerMode {
    /// Delivery comes from a live NATS subscription without durable JetStream cursor metadata.
    Ephemeral,
    /// Delivery comes from a Trellis-provisioned durable JetStream consumer.
    Durable,
}

/// Handle for a registered service event listener.
///
/// Call [`ServiceEventListenerHandle::abort`] to stop delivery for this handler
/// registration. Durable listeners are removed from the shared listener registry;
/// when the last handler for a durable consumer is removed, the shared pull task
/// is also aborted.
pub struct ServiceEventListenerHandle {
    task: JoinHandle<Result<(), ServiceRuntimeError>>,
    registration: StdMutex<Option<ServiceEventListenerRegistration>>,
}

impl std::fmt::Debug for ServiceEventListenerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceEventListenerHandle")
            .field(
                "has_registration",
                &self
                    .registration
                    .lock()
                    .map(|registration| registration.is_some())
                    .unwrap_or(false),
            )
            .finish_non_exhaustive()
    }
}

impl ServiceEventListenerHandle {
    fn new(
        task: JoinHandle<Result<(), ServiceRuntimeError>>,
        registration: Option<ServiceEventListenerRegistration>,
    ) -> Self {
        Self {
            task,
            registration: StdMutex::new(registration),
        }
    }

    /// Abort this listener and remove its durable handler registration, if any.
    pub fn abort(&self) {
        if let Ok(mut registration) = self.registration.lock() {
            if let Some(registration) = registration.take() {
                remove_service_event_listener_registration(registration);
            }
        }
        self.task.abort();
    }
}

impl Drop for ServiceEventListenerHandle {
    fn drop(&mut self) {
        if let Ok(registration) = self.registration.get_mut() {
            if let Some(registration) = registration.take() {
                remove_service_event_listener_registration(registration);
            }
        }
        self.task.abort();
    }
}

impl Future for ServiceEventListenerHandle {
    type Output = Result<Result<(), ServiceRuntimeError>, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.task).poll(cx)
    }
}

/// Cloneable handle exposed to registered service handlers.
#[derive(Clone)]
pub struct ServiceHandle {
    client: Arc<TrellisClient>,
    service_name: Arc<str>,
    binding: CoreBootstrapBinding,
    resources: ServiceResourceBindings,
    event_listeners: SharedDurableEventListeners,
    auth: LocalAuthVerifier,
}

impl std::fmt::Debug for ServiceHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceHandle")
            .field("service_name", &self.service_name)
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}

impl ServiceHandle {
    /// Return the opaque caller used for generated outbound calls.
    pub fn caller(&self) -> crate::generated::Caller {
        crate::generated::Caller::from_client(Arc::clone(&self.client))
    }

    /// Return the authenticated service session's public key.
    pub fn session_key(&self) -> &str {
        &self.client.auth().session_key
    }

    fn client(&self) -> &Arc<TrellisClient> {
        &self.client
    }

    /// Return the service instance name used during bootstrap.
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Return the parsed core bootstrap binding supplied by service bootstrap.
    pub fn binding(&self) -> &CoreBootstrapBinding {
        &self.binding
    }

    /// Return all typed resource bindings resolved during service bootstrap.
    pub fn resources(&self) -> &ServiceResourceBindings {
        &self.resources
    }

    /// Return one KV/state resource binding by contract-local resource name.
    pub fn kv_binding(&self, name: &str) -> Result<&KvResourceBinding, ServerError> {
        self.resources
            .kv
            .get(name)
            .ok_or_else(|| ServerError::MissingResourceBinding {
                service_name: self.service_name().to_string(),
                resource_kind: "kv".to_string(),
                resource_name: name.to_string(),
            })
    }

    /// Return one object-store resource binding by contract-local resource name.
    pub fn store_binding(&self, name: &str) -> Result<&StoreResourceBinding, ServerError> {
        self.resources
            .store
            .get(name)
            .ok_or_else(|| ServerError::MissingResourceBinding {
                service_name: self.service_name().to_string(),
                resource_kind: "store".to_string(),
                resource_name: name.to_string(),
            })
    }

    /// Return the service-private jobs resource binding.
    pub fn jobs_binding(&self) -> Result<&JobsResourceBinding, ServerError> {
        self.resources
            .jobs
            .as_ref()
            .ok_or_else(|| ServerError::MissingResourceBinding {
                service_name: self.service_name().to_string(),
                resource_kind: "jobs".to_string(),
                resource_name: "jobs".to_string(),
            })
    }

    /// Return an event publisher backed by the connected Trellis client.
    pub fn event_publisher(&self) -> EventPublisher {
        EventPublisher::new(Arc::clone(self.client()))
    }

    /// Submit a typed service-private job for generated participant code.
    #[doc(hidden)]
    pub async fn generated_submit_job<D>(
        &self,
        payload: D::Payload,
    ) -> Result<JobRef<D::Payload, D::Result>, JobsError>
    where
        D: JobDescriptor,
    {
        let binding = self
            .binding
            .jobs_runtime_binding()
            .map_err(|error| JobsError::Message {
                message: error.to_string(),
            })?;
        let key_coordinator = crate::jobs::keys::NatsKeyCoordinator::open_for_service(
            self.client().nats().clone(),
            &binding.jobs.namespace,
        )
        .await
        .map_err(|error| JobsError::Message {
            message: error.to_string(),
        })?;
        let manager = JobManager::new_with_key_coordinator(
            TrellisJobEventPublisher::new(self.client().nats().clone()),
            binding.jobs,
            TrellisJobMetaSource,
            Arc::new(key_coordinator),
        );
        let job = manager
            .create(D::QUEUE_TYPE, payload)
            .await
            .map_err(|error| JobsError::Message {
                message: error.to_string(),
            })?;
        let queue = manager
            .bindings()
            .queues
            .get(D::QUEUE_TYPE)
            .cloned()
            .ok_or_else(|| JobsError::Message {
                message: format!("missing jobs queue binding '{}'", D::QUEUE_TYPE),
            })?;
        let waiter = crate::jobs::runtime_ref::NatsJobWaiter::new(
            self.client().nats().clone(),
            queue,
            Duration::from_secs(30),
        );
        Ok(JobRef::from_runtime(job, waiter, manager))
    }

    /// Start a descriptor-backed event listener.
    pub async fn listen_event<D, F, Fut>(
        &self,
        handler: F,
        options: ServiceEventListenOptions,
    ) -> Result<ServiceEventListenerHandle, ServiceRuntimeError>
    where
        D: crate::client::EventDescriptor + 'static,
        D::Event: Send + 'static,
        F: Fn(D::Event, ServiceEventListenerContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), ServerError>> + Send + 'static,
    {
        listen_event_with_bindings::<D, _, _>(
            self.client(),
            self.auth.clone(),
            self.auth.api_id(),
            &self.resources.event_consumers,
            Arc::clone(&self.event_listeners),
            handler,
            options,
        )
        .await
    }

    /// Start a descriptor-backed event listener with an explicit owning API id.
    ///
    /// Use this form for events imported from another participant contract; the
    /// API id is part of the precompiled event descriptor and is required for
    /// exact publisher-permission verification.
    pub async fn listen_event_with_api_id<D, F, Fut>(
        &self,
        event_api_id: &str,
        handler: F,
        options: ServiceEventListenOptions,
    ) -> Result<ServiceEventListenerHandle, ServiceRuntimeError>
    where
        D: crate::client::EventDescriptor + 'static,
        D::Event: Send + 'static,
        F: Fn(D::Event, ServiceEventListenerContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), ServerError>> + Send + 'static,
    {
        listen_event_with_bindings::<D, _, _>(
            self.client(),
            self.auth.clone(),
            event_api_id,
            &self.resources.event_consumers,
            Arc::clone(&self.event_listeners),
            handler,
            options,
        )
        .await
    }

    /// Open a bound KV resource client by contract-local resource name.
    pub async fn kv_client(&self, name: &str) -> Result<KvHandle, ServerError> {
        let binding = self.kv_binding(name)?;
        validate_kv_binding(self.service_name(), name, binding)?;
        let client = self.client().nats().open_kv(binding).await?;
        Ok(KvResourceHandle::new(name, binding.clone(), client))
    }

    /// Open a bound object-store resource client by contract-local resource name.
    pub async fn store_client(&self, name: &str) -> Result<StoreHandle, ServerError> {
        let binding = self.store_binding(name)?;
        validate_store_binding(self.service_name(), name, binding)?;
        let client = self.client().nats().open_store(binding).await?;
        Ok(StoreResourceHandle::new(
            self.service_name(),
            name,
            binding.clone(),
            client,
        ))
    }

    /// Subscribe and run an upload transfer endpoint backed by the connected NATS client.
    pub async fn spawn_upload_transfer_endpoint_with_progress<C, F>(
        &self,
        session: UploadTransferSession,
        store: C,
        on_progress: F,
    ) -> Result<(), ServerError>
    where
        C: StoreResourceClient,
        F: Fn(OperationTransferProgress) + Send + Sync + 'static,
    {
        spawn_upload_transfer_endpoint_with_progress(
            self.client().nats().clone(),
            session,
            store,
            self.auth.clone(),
            on_progress,
        )
        .await
    }

    /// Subscribe and run an upload transfer endpoint that can be awaited until durable storage.
    pub async fn spawn_upload_transfer_endpoint_with_completion<C>(
        &self,
        session: UploadTransferSession,
        store: C,
    ) -> Result<UploadTransferCompletion, ServerError>
    where
        C: StoreResourceClient,
    {
        spawn_upload_transfer_endpoint_with_completion(
            self.client().nats().clone(),
            session,
            store,
            self.auth.clone(),
        )
        .await
    }

    /// Subscribe and run a download transfer endpoint backed by the connected NATS client.
    pub async fn spawn_download_transfer_endpoint<C>(
        &self,
        plan: DownloadTransferGrantPlan,
        store: C,
    ) -> Result<(), ServerError>
    where
        C: StoreResourceClient,
    {
        spawn_download_transfer_endpoint(
            self.client().nats().clone(),
            plan,
            store,
            self.auth.clone(),
        )
        .await
    }
}

/// Per-request handler context with request metadata and a cloneable service handle.
#[derive(Debug, Clone)]
pub struct ServiceHandlerContext {
    request: RequestContext,
    handle: ServiceHandle,
}

impl ServiceHandlerContext {
    /// Build a handler context from low-level request metadata and a service handle.
    pub fn new(request: RequestContext, handle: ServiceHandle) -> Self {
        Self { request, handle }
    }

    /// Return low-level request metadata, including caller and tracing fields.
    pub fn request(&self) -> &RequestContext {
        &self.request
    }

    /// Return the cloneable service handle for outbound calls and bindings.
    pub fn handle(&self) -> &ServiceHandle {
        &self.handle
    }

    /// Plan a download transfer using this request's authenticated caller and service bindings.
    pub fn plan_download_transfer(
        &self,
        store: &str,
        transfer_id: &str,
        expires_at: &str,
        chunk_bytes: u64,
        info: super::FileTransferInfo,
    ) -> Result<DownloadTransferGrantPlan, ServerError> {
        let session_key =
            self.request
                .session_key
                .as_deref()
                .ok_or_else(|| ServerError::MissingSessionKey {
                    subject: self.request.subject.clone(),
                })?;
        super::plan_download_transfer_grant(super::TransferDownloadGrantArgs {
            service_name: self.handle.service_name(),
            session_key,
            service_session_key: self.handle.session_key(),
            resources: self.handle.resources(),
            store,
            transfer_id,
            expires_at,
            chunk_bytes,
            info,
        })
    }

    /// Consume this context into the low-level request metadata.
    pub fn into_request_context(self) -> RequestContext {
        self.request
    }
}

/// Connected high-level service runtime for one generated service contract.
pub struct ConnectedServiceRuntime<C> {
    client: Arc<TrellisClient>,
    caller: crate::generated::Caller,
    binding: CoreBootstrapBinding,
    resources: ServiceResourceBindings,
    event_listeners: SharedDurableEventListeners,
    auth: LocalAuthVerifier,
    _event_listener_cleanup: ServiceEventListenerRegistryCleanup,
    router: Router,
    service_name: String,
    registered_subjects: BTreeSet<String>,
    job_hosts: Vec<WorkerHostHandle>,
    _contract: PhantomData<C>,
}

impl<C> std::fmt::Debug for ConnectedServiceRuntime<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectedServiceRuntime")
            .field("binding", &self.binding)
            .field("service_name", &self.service_name)
            .field("registered_subjects", &self.registered_subjects)
            .finish_non_exhaustive()
    }
}

impl<C> ConnectedServiceRuntime<C> {
    /// Return the local provider cache for live integration assertions.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn integration_test_authorization_provider(
        &self,
    ) -> crate::client::AuthorizationProviderCache {
        self.client
            .integration_test_authorization_provider()
            .expect("connected service authorization provider is present")
    }

    /// Return the connected NATS client for live reconnect assertions.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn integration_test_nats(&self) -> async_nats::Client {
        self.client.integration_test_nats()
    }

    /// Send one signed raw RPC request through the connected service session.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub async fn integration_test_request_json_value(
        &self,
        subject: &str,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, crate::client::TrellisClientError> {
        self.client.request_json_value(subject, input).await
    }

    /// Build a connected runtime from an injected client and bootstrap binding.
    pub(crate) fn from_parts(
        service_name: impl Into<String>,
        client: Arc<TrellisClient>,
        binding: CoreBootstrapBinding,
        api_id: impl Into<String>,
    ) -> Self {
        let resources = binding.resource_bindings();
        let event_listeners = SharedDurableEventListeners::default();
        let api_id = api_id.into();
        let auth =
            LocalAuthVerifier::new(client.authorization_context_cache().ok(), api_id.clone());
        let caller = crate::generated::Caller::new(Arc::clone(&client));
        let mut router = Router::new();
        router.set_api_id(api_id);
        Self {
            client,
            caller,
            binding,
            resources,
            event_listeners: Arc::clone(&event_listeners),
            auth,
            _event_listener_cleanup: ServiceEventListenerRegistryCleanup::new(event_listeners),
            router,
            service_name: service_name.into(),
            registered_subjects: BTreeSet::new(),
            job_hosts: Vec::new(),
            _contract: PhantomData,
        }
    }

    /// Build a connected runtime from a service client that already completed bootstrap.
    #[cfg(feature = "test-support")]
    pub(crate) fn from_connected_client(
        service_name: impl Into<String>,
        client: Arc<TrellisClient>,
    ) -> Result<Self, ServiceRuntimeError> {
        let binding = parse_bootstrap_binding(client.as_ref())?;
        let service_name = service_name.into();
        Ok(Self::from_parts(
            service_name.clone(),
            client,
            binding,
            service_name,
        ))
    }

    /// Return the internal Trellis client owned by this runtime.
    pub(crate) fn client(&self) -> &Arc<TrellisClient> {
        &self.client
    }

    fn descriptor_subject(&self, subject: &str) -> String {
        crate::client::OperationTransport::descriptor_subject(&self.caller, subject)
    }

    /// Return the opaque caller handle consumed by generated facades.
    #[doc(hidden)]
    pub fn caller(&self) -> &crate::generated::Caller {
        &self.caller
    }

    /// Return the parsed core bootstrap binding supplied by service bootstrap.
    pub fn binding(&self) -> &CoreBootstrapBinding {
        &self.binding
    }

    /// Return all typed resource bindings resolved during service bootstrap.
    pub fn resources(&self) -> &ServiceResourceBindings {
        &self.resources
    }

    /// Return one KV/state resource binding by contract-local resource name.
    pub fn kv_binding(&self, name: &str) -> Result<&KvResourceBinding, ServerError> {
        self.resources
            .kv
            .get(name)
            .ok_or_else(|| ServerError::MissingResourceBinding {
                service_name: self.service_name().to_string(),
                resource_kind: "kv".to_string(),
                resource_name: name.to_string(),
            })
    }

    /// Return one object-store resource binding by contract-local resource name.
    pub fn store_binding(&self, name: &str) -> Result<&StoreResourceBinding, ServerError> {
        self.resources
            .store
            .get(name)
            .ok_or_else(|| ServerError::MissingResourceBinding {
                service_name: self.service_name().to_string(),
                resource_kind: "store".to_string(),
                resource_name: name.to_string(),
            })
    }

    /// Return the service-private jobs resource binding.
    pub fn jobs_binding(&self) -> Result<&JobsResourceBinding, ServerError> {
        self.resources
            .jobs
            .as_ref()
            .ok_or_else(|| ServerError::MissingResourceBinding {
                service_name: self.service_name().to_string(),
                resource_kind: "jobs".to_string(),
                resource_name: "jobs".to_string(),
            })
    }

    /// Return the Jobs-domain transport used by Trellis infrastructure services.
    pub fn jobs_runtime(&self) -> crate::jobs::JobsRuntime {
        crate::jobs::JobsRuntime::from_client(self.client())
    }

    /// Return Jobs-only worker host access for Trellis integration tests.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn test_jobs_worker_runtime(
        &self,
    ) -> Result<crate::jobs::TestJobsWorkerRuntime, ServiceRuntimeError> {
        Ok(crate::jobs::TestJobsWorkerRuntime::new(
            Arc::clone(self.client()),
            self.binding.jobs_runtime_binding()?,
        ))
    }

    /// Return the Event Log domain transport used by Trellis infrastructure.
    pub fn eventlog_runtime(&self) -> super::EventLogRuntime {
        super::EventLogRuntime::from_client(Arc::clone(self.client()))
    }

    /// Submit a typed service-private job for generated participant code.
    #[doc(hidden)]
    pub async fn generated_submit_job<D>(
        &self,
        payload: D::Payload,
    ) -> Result<JobRef<D::Payload, D::Result>, JobsError>
    where
        D: JobDescriptor,
    {
        self.generated_handle()
            .generated_submit_job::<D>(payload)
            .await
    }

    /// Start one generated service-private job worker and retain its lifecycle.
    #[doc(hidden)]
    pub async fn register_generated_job_worker<D, H, Fut, E>(
        &mut self,
        handler: H,
    ) -> Result<(), ServiceRuntimeError>
    where
        D: JobDescriptor + 'static,
        H: Fn(crate::jobs::ActiveJob<D::Payload, D::Result>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<D::Result, E>> + Send + 'static,
        E: ToString + Send + 'static,
    {
        let mut binding = self.binding.jobs_runtime_binding()?;
        binding
            .jobs
            .queues
            .retain(|queue, _| queue == D::QUEUE_TYPE);
        if binding.jobs.queues.is_empty() {
            return Err(ServiceRuntimeError::MissingJobQueue {
                queue_type: D::QUEUE_TYPE.to_string(),
            });
        }
        let host = start_worker_host_from_client(
            self.client(),
            binding,
            ulid::Ulid::new().to_string(),
            |_, _| TrellisJobMetaSource,
            move |active| {
                let handler = handler.clone();
                async move {
                    let active = crate::jobs::internal::typed_active_job::<D>(active)
                        .map_err(|error| JobProcessError::Failed(error.to_string()))?;
                    let result = handler(active)
                        .await
                        .map_err(|error| JobProcessError::Failed(error.to_string()))?;
                    serde_json::to_value(result)
                        .map_err(|error| JobProcessError::Failed(error.to_string()))
                }
            },
            WorkerHostOptions::default(),
        )
        .await?;
        self.job_hosts.push(host);
        Ok(())
    }

    /// Return an event publisher backed by the connected NATS client.
    pub fn event_publisher(&self) -> EventPublisher {
        EventPublisher::new(Arc::clone(self.client()))
    }

    /// Start a descriptor-backed event listener.
    pub async fn listen_event<D, F, Fut>(
        &self,
        handler: F,
        options: ServiceEventListenOptions,
    ) -> Result<ServiceEventListenerHandle, ServiceRuntimeError>
    where
        D: crate::client::EventDescriptor + 'static,
        D::Event: Send + 'static,
        F: Fn(D::Event, ServiceEventListenerContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), ServerError>> + Send + 'static,
    {
        listen_event_with_bindings::<D, _, _>(
            self.client(),
            self.auth.clone(),
            self.auth.api_id(),
            &self.resources.event_consumers,
            Arc::clone(&self.event_listeners),
            handler,
            options,
        )
        .await
    }

    /// Open a bound KV resource client by contract-local resource name.
    pub async fn kv_client(&self, name: &str) -> Result<KvHandle, ServerError> {
        let binding = self.kv_binding(name)?;
        validate_kv_binding(self.service_name(), name, binding)?;
        let client = self.client().nats().open_kv(binding).await?;
        Ok(KvResourceHandle::new(name, binding.clone(), client))
    }

    /// Open a bound object-store resource client by contract-local resource name.
    pub async fn store_client(&self, name: &str) -> Result<StoreHandle, ServerError> {
        let binding = self.store_binding(name)?;
        validate_store_binding(self.service_name(), name, binding)?;
        let client = self.client().nats().open_store(binding).await?;
        Ok(StoreResourceHandle::new(
            self.service_name(),
            name,
            binding.clone(),
            client,
        ))
    }

    /// Return the service instance name used during bootstrap.
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Return the registered NATS subjects, derived from descriptors.
    pub fn registered_subjects(&self) -> Vec<&str> {
        self.registered_subjects
            .iter()
            .map(String::as_str)
            .collect()
    }

    /// Start a descriptor-backed event listener with an explicit owning API id.
    ///
    /// Use this form for events imported from another participant contract; the
    /// API id is part of the precompiled event descriptor and is required for
    /// exact publisher-permission verification.
    pub async fn listen_event_with_api_id<D, F, Fut>(
        &self,
        event_api_id: &str,
        handler: F,
        options: ServiceEventListenOptions,
    ) -> Result<ServiceEventListenerHandle, ServiceRuntimeError>
    where
        D: crate::client::EventDescriptor + 'static,
        D::Event: Send + 'static,
        F: Fn(D::Event, ServiceEventListenerContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), ServerError>> + Send + 'static,
    {
        listen_event_with_bindings::<D, _, _>(
            self.client(),
            self.auth.clone(),
            event_api_id,
            &self.resources.event_consumers,
            Arc::clone(&self.event_listeners),
            handler,
            options,
        )
        .await
    }

    /// Register one descriptor-backed RPC handler and record its subject.
    pub fn register_rpc<D, F, Fut>(&mut self, handler: F)
    where
        D: RpcDescriptor + 'static,
        F: Fn(ServiceHandlerContext, D::Input) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult<D::Output>> + Send + 'static,
    {
        let handle = self.generated_handle();
        self.router.register_rpc::<D, _, _>(move |request, input| {
            handler(ServiceHandlerContext::new(request, handle.clone()), input)
        });
        self.registered_subjects
            .insert(self.descriptor_subject(D::SUBJECT));
    }

    /// Register one descriptor-backed feed handler and record its subject.
    pub fn register_feed<D, F, S>(&mut self, handler: F)
    where
        D: FeedDescriptor + 'static,
        F: Fn(ServiceHandlerContext, D::Input) -> S + Send + Sync + 'static,
        S: Stream<Item = Result<D::Event, ServerError>> + Send + 'static,
    {
        let handle = self.generated_handle();
        self.router.register_feed::<D, _, _>(move |request, input| {
            handler(ServiceHandlerContext::new(request, handle.clone()), input)
        });
        self.registered_subjects
            .insert(self.descriptor_subject(D::SUBJECT));
    }

    /// Register one operation-backed provider and record data/control subjects.
    pub fn register_operation_provider<D, P>(&mut self, provider: P)
    where
        D: OperationDescriptor + 'static,
        P: ServiceOperationProvider<D>,
    {
        self.router.register_operation_provider::<D, _>(provider);
        self.registered_subjects
            .insert(self.descriptor_subject(D::SUBJECT));
        self.registered_subjects
            .insert(control_subject(&self.descriptor_subject(D::SUBJECT)));
    }

    /// Run registered subjects using the default NATS request loop.
    pub async fn run(self) -> Result<(), ServiceRuntimeError> {
        let subjects = self.registered_subjects.into_iter().collect::<Vec<_>>();
        let job_hosts = self.job_hosts;
        let host = bootstrap_service_host(
            &self.service_name,
            self.binding.bootstrap_binding(),
            self.router,
            self.auth,
        );
        let serve = async {
            if subjects.is_empty() {
                std::future::pending::<()>().await;
            }
            let subject_refs = subjects.iter().map(String::as_str).collect::<Vec<_>>();
            run_multi_subject_service(self.client.nats().clone(), &subject_refs, host)
                .await
                .map_err(ServiceRuntimeError::from)
        };
        if job_hosts.is_empty() {
            return serve.await;
        }
        let workers = async {
            futures_util::future::try_join_all(job_hosts.into_iter().map(WorkerHostHandle::join))
                .await
                .map_err(ServiceRuntimeError::JobWorker)?;
            Ok(())
        };
        tokio::try_join!(serve, workers)?;
        Ok(())
    }

    /// Return a cloneable service handle for generated participant code.
    #[doc(hidden)]
    pub fn generated_handle(&self) -> ServiceHandle {
        ServiceHandle {
            client: Arc::clone(&self.client),
            service_name: Arc::from(self.service_name.as_str()),
            binding: self.binding.clone(),
            resources: self.resources.clone(),
            event_listeners: Arc::clone(&self.event_listeners),
            auth: self.auth.clone(),
        }
    }
}

impl<C: GeneratedServiceContract> ConnectedServiceRuntime<C> {
    /// Connect with generated contract constants and parse the returned bootstrap binding.
    pub async fn connect(options: ServiceConnectOptions<'_>) -> Result<Self, ServiceRuntimeError> {
        let client =
            TrellisClient::connect_service_with_contract(ServiceConnectWithContractOptions {
                trellis_url: options.trellis_url,
                participant_id: C::PARTICIPANT_ID,
                participant_digest: C::CONTRACT_DIGEST,
                participant_json: C::PARTICIPANT_JSON,
                api_json: C::API_JSON,
                api_digest: C::API_DIGEST,
                referenced_api_artifacts: C::REFERENCED_API_ARTIFACTS,
                deployment_id: options.deployment_id,
                instance_id: options.name,
                provisioned_identity_seed_base64url: options.provisioned_identity_seed_base64url,
                participant_needs_digest: C::PARTICIPANT_NEEDS_DIGEST,
                session_key_seed_base64url: options.session_key_seed_base64url.as_ref(),
                timeout_ms: options.timeout_ms,
                retry_delay_ms: options.retry_delay_ms,
                authority_pending_timeout_ms: options.authority_pending_timeout_ms,
                authorization_context_store: options.authorization_context_store.clone(),
            })
            .await?;
        let binding = parse_bootstrap_binding(&client)?;
        Ok(Self::from_parts(
            options.name,
            Arc::new(client),
            binding,
            C::PARTICIPANT_ID,
        ))
    }
}

fn parse_bootstrap_binding(
    client: &TrellisClient,
) -> Result<CoreBootstrapBinding, ServiceRuntimeError> {
    client
        .service_bootstrap_binding()
        .cloned()
        .ok_or(ServiceRuntimeError::MissingBootstrapBinding)
}

fn service_event_context_from_message<T>(
    mode: ServiceEventListenerMode,
    group: Option<String>,
    message: &EventMessage<T>,
    publisher: Option<ServiceEventPublisherContext>,
) -> ServiceEventListenerContext {
    service_event_context_from_headers(mode, group, message.headers(), publisher)
}

fn service_event_context_from_headers(
    mode: ServiceEventListenerMode,
    group: Option<String>,
    headers: Option<&HeaderMap>,
    publisher: Option<ServiceEventPublisherContext>,
) -> ServiceEventListenerContext {
    let headers = headers.cloned().unwrap_or_default();
    ServiceEventListenerContext {
        mode,
        group,
        id: headers
            .get("Nats-Msg-Id")
            .map(|value| value.as_str().to_string()),
        time: headers
            .get("Trellis-Event-Time")
            .map(|value| value.as_str().to_string()),
        traceparent: headers
            .get("traceparent")
            .map(|value| value.as_str().to_string()),
        headers,
        publisher,
    }
}

async fn listen_event_with_bindings<D, F, Fut>(
    client: &Arc<TrellisClient>,
    auth: LocalAuthVerifier,
    event_api_id: &str,
    bindings: &BTreeMap<String, super::EventConsumerResourceBinding>,
    event_listeners: SharedDurableEventListeners,
    handler: F,
    options: ServiceEventListenOptions,
) -> Result<ServiceEventListenerHandle, ServiceRuntimeError>
where
    D: crate::client::EventDescriptor + 'static,
    D::Event: Send + 'static,
    F: Fn(D::Event, ServiceEventListenerContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), ServerError>> + Send + 'static,
{
    let event_api_id = event_api_id.to_owned();
    let event_name = D::KEY.to_owned();
    let publish_capabilities = D::PUBLISH_CAPABILITIES
        .iter()
        .map(|capability| (*capability).to_owned())
        .collect::<Vec<_>>();
    if let Some(durable_name) = options.durable_name.as_deref() {
        return Err(ServiceRuntimeError::CallerDurableName {
            durable_name: durable_name.to_string(),
        });
    }

    if options.mode == ServiceEventListenerMode::Ephemeral {
        let mut events = client
            .nats()
            .subscribe(client.descriptor_subject(D::SUBJECT))
            .await
            .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;
        let event_auth = auth.clone();
        let event_api_id = event_api_id.clone();
        let event_name = event_name.clone();
        let publish_capabilities = publish_capabilities.clone();
        return Ok(ServiceEventListenerHandle::new(
            tokio::spawn(async move {
                while let Some(message) = events.next().await {
                    let publisher = match event_auth
                        .verify_event(
                            message.subject.as_ref(),
                            &message.payload,
                            message.headers.as_ref(),
                            &event_api_id,
                            &event_name,
                            &publish_capabilities,
                        )
                        .await
                    {
                        Ok(publisher) => publisher,
                        Err(error) => {
                            tracing::warn!(
                                subject = %message.subject,
                                error = %error.message(),
                                "Event auth validation failed"
                            );
                            continue;
                        }
                    };
                    let context = service_event_context_from_headers(
                        ServiceEventListenerMode::Ephemeral,
                        None,
                        message.headers.as_ref(),
                        Some(publisher),
                    );
                    let event = serde_json::from_slice::<D::Event>(&message.payload)
                        .map_err(TrellisClientError::from)?;
                    if let Err(source) = handler(event, context.clone()).await {
                        return Err(ServiceRuntimeError::EventHandler {
                            source: Box::new(source),
                            context: Box::new(context),
                        });
                    }
                }
                Ok(())
            }),
            None,
        ));
    }

    let subject = client.descriptor_subject(D::SUBJECT);
    let (group, binding) =
        resolve_event_consumer_binding(bindings, &subject, options.group.as_deref(), None)?;
    validate_event_listener_concurrency(&group, binding.ordering, options.concurrency, None)?;
    let key = DurableEventListenerKey {
        stream: binding.stream.clone(),
        durable_name: binding.consumer_name.clone(),
    };
    let context = ServiceEventListenerContext {
        mode: ServiceEventListenerMode::Durable,
        group: Some(group),
        id: None,
        time: None,
        traceparent: None,
        headers: HeaderMap::new(),
        publisher: None,
    };
    let handler = Arc::new(handler);
    let handler_id = SERVICE_EVENT_HANDLER_ID.fetch_add(1, Ordering::Relaxed);
    let handler: SharedEventHandler = Arc::new(move |payload, context| {
        let handler = Arc::clone(&handler);
        let event = serde_json::from_slice::<D::Event>(&payload)
            .map_err(TrellisClientError::from)
            .map_err(ServiceRuntimeError::from);
        Box::pin(async move {
            handler(event?, context.clone()).await.map_err(|source| {
                ServiceRuntimeError::EventHandler {
                    source: Box::new(source),
                    context: Box::new(context),
                }
            })
        })
    });

    let mut listeners = lock_service_event_listeners(&event_listeners);
    if let Some(listener) = listeners.get_mut(&key) {
        validate_event_listener_concurrency(
            context.group.as_deref().expect("durable listener group"),
            binding.ordering,
            options.concurrency,
            Some(listener.concurrency),
        )?;
        listener
            .handlers
            .entry(subject.clone())
            .or_default()
            .insert(handler_id, handler);
        return Ok(ServiceEventListenerHandle::new(
            tokio::spawn(async { futures_util::future::pending().await }),
            Some(ServiceEventListenerRegistration {
                event_listeners: Arc::clone(&event_listeners),
                key,
                subject,
                handler_id,
            }),
        ));
    }

    let subscribe_options = EventSubscribeOptions {
        stream: Some(binding.stream.clone()),
        mode: EventSubscriptionMode::Durable,
        replay: EventReplayPolicy::New,
        durable_name: Some(binding.consumer_name.clone()),
    };
    let pull_abort_handles = (0..options.concurrency)
        .map(|_| {
            tokio::spawn(run_durable_event_pull_loop::<D>(
                Arc::clone(client),
                auth.clone(),
                event_api_id.clone(),
                event_name.clone(),
                publish_capabilities.clone(),
                Arc::clone(&event_listeners),
                key.clone(),
                subscribe_options.clone(),
                context.clone(),
            ))
            .abort_handle()
        })
        .collect();
    listeners.insert(
        key.clone(),
        SharedDurableEventListener {
            expected_subjects: binding.filter_subjects.iter().cloned().collect(),
            handlers: BTreeMap::from([(subject.clone(), BTreeMap::from([(handler_id, handler)]))]),
            concurrency: options.concurrency,
            pull_abort_handles,
        },
    );
    drop(listeners);

    Ok(ServiceEventListenerHandle::new(
        tokio::spawn(async { futures_util::future::pending().await }),
        Some(ServiceEventListenerRegistration {
            event_listeners,
            key,
            subject,
            handler_id,
        }),
    ))
}

fn validate_event_listener_concurrency(
    group: &str,
    ordering: super::EventConsumerOrdering,
    requested: u32,
    existing: Option<u32>,
) -> Result<(), ServiceRuntimeError> {
    if requested == 0 {
        return Err(ServiceRuntimeError::InvalidEventListenerConcurrency {
            group: group.to_string(),
            concurrency: requested,
        });
    }
    if ordering == super::EventConsumerOrdering::Strict && requested > 1 {
        return Err(ServiceRuntimeError::StrictEventListenerConcurrency {
            group: group.to_string(),
        });
    }
    if let Some(existing) = existing.filter(|existing| *existing != requested) {
        return Err(ServiceRuntimeError::EventListenerConcurrencyMismatch {
            group: group.to_string(),
            existing,
            requested,
        });
    }
    Ok(())
}

fn lock_service_event_listeners(
    event_listeners: &SharedDurableEventListeners,
) -> std::sync::MutexGuard<'_, BTreeMap<DurableEventListenerKey, SharedDurableEventListener>> {
    event_listeners
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn remove_service_event_listener_registration(registration: ServiceEventListenerRegistration) {
    let mut listeners = lock_service_event_listeners(&registration.event_listeners);
    let Some(listener) = listeners.get_mut(&registration.key) else {
        return;
    };
    if let Some(handlers) = listener.handlers.get_mut(&registration.subject) {
        handlers.remove(&registration.handler_id);
        if handlers.is_empty() {
            listener.handlers.remove(&registration.subject);
        }
    }
    if listener.handlers.values().all(BTreeMap::is_empty) {
        if let Some(listener) = listeners.remove(&registration.key) {
            for handle in listener.pull_abort_handles {
                handle.abort();
            }
        }
    }
}

fn remove_service_event_listeners(event_listeners: &SharedDurableEventListeners) {
    let listeners = std::mem::take(&mut *lock_service_event_listeners(event_listeners));
    for (_, listener) in listeners {
        for handle in listener.pull_abort_handles {
            handle.abort();
        }
    }
}

#[allow(clippy::too_many_arguments)] // Keeps one event-listener task spawn explicit and local.
async fn run_durable_event_pull_loop<D>(
    client: Arc<TrellisClient>,
    auth: LocalAuthVerifier,
    event_api_id: String,
    event_name: String,
    publish_capabilities: Vec<String>,
    event_listeners: SharedDurableEventListeners,
    key: DurableEventListenerKey,
    subscribe_options: EventSubscribeOptions,
    context: ServiceEventListenerContext,
) where
    D: crate::client::EventDescriptor + 'static,
    D::Event: Send + 'static,
{
    loop {
        if !durable_listener_ready(&event_listeners, &key) {
            tokio::time::sleep(Duration::from_millis(25)).await;
            continue;
        }

        let mut messages = match client
            .subscribe_messages::<D>(subscribe_options.clone())
            .await
        {
            Ok(messages) => messages,
            Err(error) if is_missing_durable_event_consumer_error(&error) => {
                tokio::time::sleep(Duration::from_millis(DURABLE_EVENT_CONSUMER_RETRY_MS)).await;
                continue;
            }
            Err(error) => {
                tracing::warn!(
                    group = ?context.group,
                    error = %error,
                    "Durable event subscription failed; retrying"
                );
                tokio::time::sleep(Duration::from_millis(DURABLE_EVENT_CONSUMER_RETRY_MS)).await;
                continue;
            }
        };

        while durable_listener_ready(&event_listeners, &key) {
            let Some(result) = messages.next().await else {
                break;
            };
            let message = match result {
                Ok(message) => message,
                Err(error) if is_missing_durable_event_consumer_error(&error) => {
                    tokio::time::sleep(Duration::from_millis(DURABLE_EVENT_CONSUMER_RETRY_MS))
                        .await;
                    break;
                }
                Err(error) => {
                    tracing::warn!(
                        group = ?context.group,
                        error = %error,
                        "Durable event pull failed; retrying"
                    );
                    tokio::time::sleep(Duration::from_millis(DURABLE_EVENT_CONSUMER_RETRY_MS))
                        .await;
                    break;
                }
            };
            if !durable_listener_ready(&event_listeners, &key) {
                break;
            }
            let handlers = lock_service_event_listeners(&event_listeners)
                .get(&key)
                .and_then(|listener| listener.handlers.get(message.subject()).cloned())
                .unwrap_or_default();
            let publisher = match auth
                .verify_event(
                    message.subject(),
                    message.payload(),
                    message.headers(),
                    &event_api_id,
                    &event_name,
                    &publish_capabilities,
                )
                .await
            {
                Ok(publisher) => publisher,
                Err(error) => {
                    tracing::warn!(
                        subject = %message.subject(),
                        error = %error.message(),
                        "Event auth validation failed"
                    );
                    match error {
                        super::EventVerificationFailure::Retryable(_) => {
                            let _ = message.nak_after(Duration::from_secs(5)).await;
                        }
                        super::EventVerificationFailure::Rejected(_) => {
                            let _ = message.term().await;
                        }
                    }
                    continue;
                }
            };
            let mut handled = true;
            for handler in handlers.values() {
                let context = service_event_context_from_message(
                    context.mode,
                    context.group.clone(),
                    &message,
                    Some(publisher.clone()),
                );
                if handler(Bytes::copy_from_slice(message.payload()), context)
                    .await
                    .is_err()
                {
                    let _ = message.nak().await;
                    handled = false;
                    break;
                }
            }
            if !handled {
                continue;
            }
            if !durable_listener_ready(&event_listeners, &key) {
                break;
            }
            if let Err(error) = message.ack().await {
                tracing::warn!(
                    group = ?context.group,
                    error = %error,
                    "Durable event acknowledgement failed; retrying"
                );
                break;
            }
        }
    }
}

fn durable_listener_ready(
    event_listeners: &SharedDurableEventListeners,
    key: &DurableEventListenerKey,
) -> bool {
    lock_service_event_listeners(event_listeners)
        .get(key)
        .map(|listener| {
            listener
                .expected_subjects
                .iter()
                .all(|subject| listener.handlers.contains_key(subject))
        })
        .unwrap_or(false)
}

fn resolve_event_consumer_binding(
    bindings: &BTreeMap<String, super::EventConsumerResourceBinding>,
    subject: &str,
    group: Option<&str>,
    durable_name: Option<&str>,
) -> Result<(String, super::EventConsumerResourceBinding), ServiceRuntimeError> {
    if let Some(durable_name) = durable_name {
        return Err(ServiceRuntimeError::CallerDurableName {
            durable_name: durable_name.to_string(),
        });
    }

    if let Some(group) = group {
        let binding =
            bindings
                .get(group)
                .ok_or_else(|| ServiceRuntimeError::EventConsumerGroupNotFound {
                    group: group.to_string(),
                })?;
        if !binding
            .filter_subjects
            .iter()
            .any(|filter_subject| filter_subject == subject)
        {
            return Err(ServiceRuntimeError::EventConsumerGroupSubjectMismatch {
                group: group.to_string(),
                subject: subject.to_string(),
            });
        }
        return Ok((group.to_string(), binding.clone()));
    }

    let matches = bindings
        .iter()
        .filter(|(_, binding)| {
            binding
                .filter_subjects
                .iter()
                .any(|filter_subject| filter_subject == subject)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Err(ServiceRuntimeError::MissingEventConsumerGroup {
            subject: subject.to_string(),
        }),
        [(group, binding)] => Ok(((*group).clone(), (*binding).clone())),
        _ => Err(ServiceRuntimeError::AmbiguousEventConsumerGroup {
            subject: subject.to_string(),
            groups: matches.iter().map(|(group, _)| (*group).clone()).collect(),
        }),
    }
}

fn is_missing_durable_event_consumer_error(error: &TrellisClientError) -> bool {
    let TrellisClientError::NatsRequest(message) = error else {
        return false;
    };

    let message = message.to_ascii_lowercase();
    message.contains("consumer not found")
        || message.contains("consumer does not exist")
        || message.contains("no consumer")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::{
        BootstrapBinding, EventConsumerOrdering, EventConsumerReplay, EventConsumerResourceBinding,
        KvResourceBinding, StoreResourceBinding,
    };
    use std::collections::BTreeMap;

    #[test]
    fn service_connect_waits_indefinitely_for_authority_by_default() {
        let options = ServiceConnectOptions::new(
            "http://localhost:3000",
            "svc",
            "dep_1",
            "identity-seed",
            "session-seed",
            Arc::new(crate::client::MemoryAuthorizationContextStore::default()),
        );

        assert_eq!(options.authority_pending_timeout_ms, None);
    }

    fn binding() -> CoreBootstrapBinding {
        CoreBootstrapBinding::new(
            BootstrapBinding {
                contract_id: "example.service@v1".to_string(),
                digest: "sha256:test".to_string(),
            },
            ServiceResourceBindings {
                event_consumers: BTreeMap::from([(
                    "projection".to_string(),
                    EventConsumerResourceBinding {
                        stream: "trellis".to_string(),
                        consumer_name: "svc-projection".to_string(),
                        filter_subjects: vec!["events.v1.Billing.Paid".to_string()],
                        replay: EventConsumerReplay::New,
                        ordering: EventConsumerOrdering::Strict,
                        ack_wait_ms: 30_000,
                        max_deliver: 5,
                        backoff_ms: vec![1_000, 5_000],
                    },
                )]),
                jobs: None,
                kv: BTreeMap::from([(
                    "drafts".to_string(),
                    KvResourceBinding {
                        bucket: "svc_drafts".to_string(),
                        history: 3,
                        max_value_bytes: Some(4096),
                        ttl_ms: 60_000,
                    },
                )]),
                store: BTreeMap::from([(
                    "evidence".to_string(),
                    StoreResourceBinding {
                        name: "svc_evidence".to_string(),
                        max_object_bytes: Some(8192),
                        max_total_bytes: None,
                        ttl_ms: 0,
                    },
                )]),
            },
        )
    }

    fn event_consumer_binding(subjects: &[&str]) -> EventConsumerResourceBinding {
        EventConsumerResourceBinding {
            stream: "trellis".to_string(),
            consumer_name: "consumer".to_string(),
            filter_subjects: subjects
                .iter()
                .map(|subject| (*subject).to_string())
                .collect(),
            replay: EventConsumerReplay::New,
            ordering: EventConsumerOrdering::Strict,
            ack_wait_ms: 30_000,
            max_deliver: 5,
            backoff_ms: vec![1_000, 5_000],
        }
    }

    #[test]
    fn resolve_event_consumer_binding_infers_unique_group() {
        let bindings = BTreeMap::from([(
            "projection".to_string(),
            event_consumer_binding(&["events.v1.Billing.Paid"]),
        )]);

        let (group, binding) =
            resolve_event_consumer_binding(&bindings, "events.v1.Billing.Paid", None, None)
                .expect("binding resolves");

        assert_eq!(group, "projection");
        assert_eq!(binding.consumer_name, "consumer");
    }

    #[test]
    fn durable_event_listener_concurrency_defaults_to_one() {
        assert_eq!(ServiceEventListenOptions::default().concurrency, 1);
    }

    #[test]
    fn core_bootstrap_maps_parallel_event_consumer_ordering() {
        let mut resources = binding().resource_bindings();
        resources
            .event_consumers
            .get_mut("projection")
            .expect("projection event consumer binding")
            .ordering = EventConsumerOrdering::Parallel;

        assert_eq!(
            resources.event_consumers["projection"].ordering,
            EventConsumerOrdering::Parallel
        );
    }

    #[test]
    fn durable_event_listener_concurrency_enforces_ordering_and_group_agreement() {
        assert!(validate_event_listener_concurrency(
            "projection",
            EventConsumerOrdering::Parallel,
            4,
            Some(4)
        )
        .is_ok());
        assert!(matches!(
            validate_event_listener_concurrency(
                "projection",
                EventConsumerOrdering::Strict,
                2,
                None
            ),
            Err(ServiceRuntimeError::StrictEventListenerConcurrency { group })
                if group == "projection"
        ));
        assert!(matches!(
            validate_event_listener_concurrency(
                "projection",
                EventConsumerOrdering::Parallel,
                2,
                Some(4)
            ),
            Err(ServiceRuntimeError::EventListenerConcurrencyMismatch {
                group,
                existing: 4,
                requested: 2
            }) if group == "projection"
        ));
        assert!(matches!(
            validate_event_listener_concurrency(
                "projection",
                EventConsumerOrdering::Parallel,
                0,
                None
            ),
            Err(ServiceRuntimeError::InvalidEventListenerConcurrency {
                group,
                concurrency: 0
            }) if group == "projection"
        ));
    }

    #[test]
    fn resolve_event_consumer_binding_rejects_invalid_group_selection() {
        let bindings = BTreeMap::from([(
            "projection".to_string(),
            event_consumer_binding(&["events.v1.Billing.Paid"]),
        )]);

        assert!(matches!(
            resolve_event_consumer_binding(
                &bindings,
                "events.v1.Billing.Paid",
                None,
                Some("caller-owned"),
            ),
            Err(ServiceRuntimeError::CallerDurableName { durable_name })
                if durable_name == "caller-owned"
        ));
        assert!(matches!(
            resolve_event_consumer_binding(&bindings, "events.v1.Missing", None, None),
            Err(ServiceRuntimeError::MissingEventConsumerGroup { subject })
                if subject == "events.v1.Missing"
        ));
        assert!(matches!(
            resolve_event_consumer_binding(
                &bindings,
                "events.v1.Billing.Paid",
                Some("missing"),
                None,
            ),
            Err(ServiceRuntimeError::EventConsumerGroupNotFound { group })
                if group == "missing"
        ));
        assert!(matches!(
            resolve_event_consumer_binding(
                &bindings,
                "events.v1.Other",
                Some("projection"),
                None,
            ),
            Err(ServiceRuntimeError::EventConsumerGroupSubjectMismatch { group, subject })
                if group == "projection" && subject == "events.v1.Other"
        ));
    }

    #[test]
    fn resolve_event_consumer_binding_requires_group_for_ambiguous_match() {
        let bindings = BTreeMap::from([
            (
                "first".to_string(),
                event_consumer_binding(&["events.v1.Billing.Paid"]),
            ),
            (
                "second".to_string(),
                event_consumer_binding(&["events.v1.Billing.Paid"]),
            ),
        ]);

        assert!(matches!(
            resolve_event_consumer_binding(
                &bindings,
                "events.v1.Billing.Paid",
                None,
                None,
            ),
            Err(ServiceRuntimeError::AmbiguousEventConsumerGroup { subject, groups })
                if subject == "events.v1.Billing.Paid"
                    && groups == vec!["first".to_string(), "second".to_string()]
        ));
    }

    #[test]
    fn is_missing_durable_event_consumer_error_matches_only_missing_consumer_requests() {
        assert!(is_missing_durable_event_consumer_error(
            &TrellisClientError::NatsRequest("consumer not found".to_string())
        ));
        assert!(is_missing_durable_event_consumer_error(
            &TrellisClientError::NatsRequest("Consumer does not exist".to_string())
        ));
        assert!(is_missing_durable_event_consumer_error(
            &TrellisClientError::NatsRequest("no consumer available".to_string())
        ));
        assert!(!is_missing_durable_event_consumer_error(
            &TrellisClientError::NatsRequest("permissions violation".to_string())
        ));
        assert!(!is_missing_durable_event_consumer_error(
            &TrellisClientError::Timeout
        ));
    }
}
