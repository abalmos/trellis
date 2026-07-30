use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use bytes::Bytes;
use futures_util::future::BoxFuture;
use futures_util::{Stream, StreamExt};
use tokio::sync::oneshot;

use serde_json::Value;

use super::error::ValidationIssue;
use super::request_loop::{HandlerResponse, ResponseStream};
use super::schema_validation::validate_input_schema;
use super::{
    control_subject, AcceptedOperation, FeedDescriptor, HandlerResult, OperationControlRequest,
    OperationDescriptor, OperationLiveEvent, OperationProvider, OperationSignalAccepted,
    OperationSnapshot, OperationSnapshotFrame, RpcDescriptor, ServerError,
};

/// Request metadata forwarded to mounted RPC handlers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequestContext {
    /// NATS subject that received the request.
    pub subject: String,
    /// Runtime session key from the authenticated request headers.
    pub session_key: Option<String>,
    /// Proof signature from the authenticated request headers.
    pub proof: Option<String>,
    /// Proof issued-at timestamp from the authenticated request headers.
    pub iat: Option<i64>,
    /// Unique request id from the authenticated request headers.
    pub request_id: Option<String>,
    /// Capability requirements for this exact routed request.
    pub required_capabilities: Option<Vec<String>>,
    /// NATS reply inbox used for request/reply responses.
    pub reply_to: Option<String>,
    /// Validated caller metadata returned by `Auth.Requests.Validate`.
    pub caller: Option<Value>,
    /// W3C trace context header propagated by the caller, if present.
    pub traceparent: Option<String>,
    /// W3C trace state header propagated by the caller, if present.
    pub tracestate: Option<String>,
}

type BoxedHandler = Box<
    dyn Fn(RequestContext, Bytes) -> BoxFuture<'static, Result<HandlerResponse, ServerError>>
        + Send
        + Sync,
>;

struct Route {
    handler: BoxedHandler,
    capabilities: RouteCapabilities,
}

#[derive(Debug, Clone)]
enum RouteCapabilities {
    Static(Vec<String>),
    OperationControl {
        observe: Vec<String>,
        cancel: Vec<String>,
        control: Vec<String>,
    },
}

impl RouteCapabilities {
    fn required_for_payload(&self, payload: &[u8]) -> Option<Vec<String>> {
        let capabilities = match self {
            Self::Static(capabilities) => capabilities,
            Self::OperationControl {
                observe,
                cancel,
                control,
            } => match serde_json::from_slice::<OperationControlRequest>(payload) {
                Ok(request) => match request.action.as_str() {
                    "get" | "wait" | "watch" => observe,
                    "cancel" => cancel,
                    "signal" => control,
                    _ => return Some(Vec::new()),
                },
                Err(_) => return Some(Vec::new()),
            },
        };

        Some(capabilities.to_vec())
    }
}

type OperationWatch<TProgress, TOutput> =
    Pin<Box<dyn Stream<Item = Result<OperationSnapshot<TProgress, TOutput>, ServerError>> + Send>>;
type OperationLiveWatch<TProgress, TUpdate, TOutput> = Pin<
    Box<
        dyn Stream<Item = Result<OperationLiveEvent<TProgress, TUpdate, TOutput>, ServerError>>
            + Send,
    >,
>;
const FEED_CANCEL_TOMBSTONE_TTL: Duration = Duration::from_secs(30);
const MAX_FEED_CANCEL_TOMBSTONES: usize = 1_024;

enum FeedCancellationState {
    Active(oneshot::Sender<()>),
    Cancelled(Instant),
}

type FeedCancellations = Arc<Mutex<HashMap<(String, String), FeedCancellationState>>>;

struct FeedCancellation {
    receiver: oneshot::Receiver<()>,
    key: (String, String),
    cancellations: FeedCancellations,
}

impl Future for FeedCancellation {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver).poll(context).map(|_| ())
    }
}

impl Drop for FeedCancellation {
    fn drop(&mut self) {
        self.cancellations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&self.key);
    }
}

/// An in-memory subject router for descriptor-backed RPC handlers.
#[derive(Default)]
pub struct Router {
    handlers: HashMap<String, Route>,
    feed_cancellations: FeedCancellations,
    #[cfg(feature = "integration-test-scoping")]
    integration_test_scope: Option<crate::integration_test_scoping::IntegrationTestScope>,
}

impl Router {
    /// Create an empty router.
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "integration-test-scoping")]
    pub(crate) fn set_integration_test_scope(
        &mut self,
        scope: Option<crate::integration_test_scoping::IntegrationTestScope>,
    ) {
        self.integration_test_scope = scope;
    }

    fn descriptor_subject(&self, subject: &str) -> String {
        #[cfg(feature = "integration-test-scoping")]
        {
            crate::integration_test_scoping::resolve_descriptor_subject(
                self.integration_test_scope.as_ref(),
                subject,
            )
            .expect("generated contract descriptor subjects are valid")
            .into_owned()
        }
        #[cfg(not(feature = "integration-test-scoping"))]
        subject.to_string()
    }

    fn descriptor_capabilities(&self, capabilities: &[&str]) -> Vec<String> {
        #[cfg(feature = "integration-test-scoping")]
        if let Some(scope) = &self.integration_test_scope {
            return capabilities
                .iter()
                .map(|capability| scope.capability(capability))
                .collect();
        }
        capabilities
            .iter()
            .map(|capability| (*capability).to_string())
            .collect()
    }

    /// Register one descriptor-backed handler.
    pub fn register_rpc<D, F, Fut>(&mut self, handler: F)
    where
        D: RpcDescriptor + 'static,
        F: Fn(RequestContext, D::Input) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult<D::Output>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        let capabilities = self.descriptor_capabilities(D::CALLER_CAPABILITIES);
        self.handlers.insert(
            self.descriptor_subject(D::SUBJECT),
            Route {
                capabilities: RouteCapabilities::Static(capabilities),
                handler: Box::new(
                move |ctx, payload| -> BoxFuture<'static, Result<HandlerResponse, ServerError>> {
                    let handler = Arc::clone(&handler);
                    Box::pin(async move {
                        let input = parse_validated_input::<D::Input>(&payload, D::INPUT_SCHEMA_JSON)?;
                        let output = handler(ctx, input).await?;
                        let output = serde_json::to_value(output)?;
                        validate_provider_value(D::KEY, D::OUTPUT_SCHEMA_JSON, &output)?;
                        Ok(HandlerResponse::Frames(vec![Bytes::from(serde_json::to_vec(
                            &output,
                        )?)]))
                    })
                },
            ),
            },
        );
    }

    /// Register one descriptor-backed feed handler.
    pub fn register_feed<D, F, S>(&mut self, handler: F)
    where
        D: FeedDescriptor + 'static,
        F: Fn(RequestContext, D::Input) -> S + Send + Sync + 'static,
        S: Stream<Item = Result<D::Event, ServerError>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        let cancellations = Arc::clone(&self.feed_cancellations);
        let subject = self.descriptor_subject(D::SUBJECT);
        let handler_subject = subject.clone();
        let capabilities = self.descriptor_capabilities(D::SUBSCRIBE_CAPABILITIES);
        self.handlers.insert(
            subject,
            Route {
                capabilities: RouteCapabilities::Static(capabilities),
                handler: Box::new(
                move |ctx, payload| -> BoxFuture<'static, Result<HandlerResponse, ServerError>> {
                    let handler = Arc::clone(&handler);
                    let cancellations = Arc::clone(&cancellations);
                    let handler_subject = handler_subject.clone();
                    Box::pin(async move {
                        let input = parse_validated_input::<D::Input>(&payload, D::INPUT_SCHEMA_JSON)?;
                        let reply_to = ctx.reply_to.clone().ok_or_else(|| {
                            ServerError::Nats("feed request is missing a reply inbox".to_string())
                        })?;
                        let key = (handler_subject.clone(), reply_to);
                        let (cancel, receiver) = oneshot::channel();
                        let mut states = cancellations
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        let now = Instant::now();
                        states.retain(|_, state| {
                            !matches!(state, FeedCancellationState::Cancelled(at) if now.duration_since(*at) >= FEED_CANCEL_TOMBSTONE_TTL)
                        });
                        match states.remove(&key) {
                            Some(FeedCancellationState::Cancelled(_)) => {
                                let _ = cancel.send(());
                            }
                            Some(FeedCancellationState::Active(previous)) => {
                                let _ = previous.send(());
                                states.insert(key.clone(), FeedCancellationState::Active(cancel));
                            }
                            None => {
                                states.insert(key.clone(), FeedCancellationState::Active(cancel));
                            }
                        }
                        drop(states);
                        let cancellation = FeedCancellation {
                            receiver,
                            key,
                            cancellations,
                        };
                         Ok(HandlerResponse::FeedStream(feed_response_stream(
                            handler(ctx, input).take_until(cancellation),
                            D::KEY,
                            D::EVENT_SCHEMA_JSON,
                         )))
                    })
                },
            ),
            },
        );
    }

    /// Register one operation-backed handler pair.
    pub fn register_operation<
        D,
        FStart,
        FutStart,
        FGet,
        FutGet,
        FWait,
        FutWait,
        FCancel,
        FutCancel,
    >(
        &mut self,
        start: FStart,
        get: FGet,
        wait: FWait,
        cancel: FCancel,
    ) where
        D: OperationDescriptor + 'static,
        FStart: Fn(RequestContext, D::Input) -> FutStart + Send + Sync + 'static,
        FutStart: Future<Output = Result<AcceptedOperation<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FGet: Fn(RequestContext, String) -> FutGet + Send + Sync + 'static,
        FutGet: Future<Output = Result<OperationSnapshot<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FWait: Fn(RequestContext, String) -> FutWait + Send + Sync + 'static,
        FutWait: Future<Output = Result<OperationSnapshot<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FCancel: Fn(RequestContext, String) -> FutCancel + Send + Sync + 'static,
        FutCancel: Future<Output = Result<OperationSnapshot<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
    {
        let watch = {
            let wait = Arc::new(wait);
            move |ctx, operation_id| {
                let wait = Arc::clone(&wait);
                Box::pin(futures_util::stream::once(async move {
                    wait(ctx, operation_id).await
                })) as OperationWatch<D::Progress, D::Output>
            }
        };

        self.register_operation_with_watch::<D, _, _, _, _, _, _, _>(start, get, watch, cancel);
    }

    /// Register one operation-backed handler pair with a watch snapshot stream.
    pub fn register_operation_with_watch<
        D,
        FStart,
        FutStart,
        FGet,
        FutGet,
        FWatch,
        FCancel,
        FutCancel,
    >(
        &mut self,
        start: FStart,
        get: FGet,
        watch: FWatch,
        cancel: FCancel,
    ) where
        D: OperationDescriptor + 'static,
        FStart: Fn(RequestContext, D::Input) -> FutStart + Send + Sync + 'static,
        FutStart: Future<Output = Result<AcceptedOperation<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FGet: Fn(RequestContext, String) -> FutGet + Send + Sync + 'static,
        FutGet: Future<Output = Result<OperationSnapshot<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FWatch: Fn(RequestContext, String) -> OperationWatch<D::Progress, D::Output>
            + Send
            + Sync
            + 'static,
        FCancel: Fn(RequestContext, String) -> FutCancel + Send + Sync + 'static,
        FutCancel: Future<Output = Result<OperationSnapshot<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
    {
        let start = Arc::new(start);
        let get = Arc::new(get);
        let watch = Arc::new(watch);
        let cancel = Arc::new(cancel);
        let subject = self.descriptor_subject(D::SUBJECT);

        self.register_operation_with_watch_and_signal::<D, _, _, _, _, _, _, _, _, _>(
            move |ctx, input| {
                let start = Arc::clone(&start);
                async move { start(ctx, input).await }
            },
            move |ctx, operation_id| {
                let get = Arc::clone(&get);
                async move { get(ctx, operation_id).await }
            },
            move |ctx, operation_id| watch(ctx, operation_id),
            move |ctx, operation_id| {
                let cancel = Arc::clone(&cancel);
                async move { cancel(ctx, operation_id).await }
            },
            move |_ctx, _operation_id, _signal, _input| {
                let subject = subject.clone();
                async move {
                    Err(ServerError::InvalidOperationControlAction {
                        subject,
                        action: "signal".to_string(),
                    })
                }
            },
        );
    }

    /// Register one operation-backed handler with watch and signal control support.
    pub fn register_operation_with_watch_and_signal<
        D,
        FStart,
        FutStart,
        FGet,
        FutGet,
        FWatch,
        FCancel,
        FutCancel,
        FSignal,
        FutSignal,
    >(
        &mut self,
        start: FStart,
        get: FGet,
        watch: FWatch,
        cancel: FCancel,
        signal: FSignal,
    ) where
        D: OperationDescriptor + 'static,
        FStart: Fn(RequestContext, D::Input) -> FutStart + Send + Sync + 'static,
        FutStart: Future<Output = Result<AcceptedOperation<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FGet: Fn(RequestContext, String) -> FutGet + Send + Sync + 'static,
        FutGet: Future<Output = Result<OperationSnapshot<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FWatch: Fn(RequestContext, String) -> OperationWatch<D::Progress, D::Output>
            + Send
            + Sync
            + 'static,
        FCancel: Fn(RequestContext, String) -> FutCancel + Send + Sync + 'static,
        FutCancel: Future<Output = Result<OperationSnapshot<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FSignal:
            Fn(RequestContext, String, String, Option<Value>) -> FutSignal + Send + Sync + 'static,
        FutSignal: Future<Output = Result<OperationSignalAccepted<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
    {
        self.register_operation_with_live_watch_and_signal::<D, Value, _, _, _, _, _, _, _, _, _>(
            start,
            get,
            move |context, operation_id| {
                Box::pin(
                    watch(context, operation_id)
                        .map(|snapshot| snapshot.map(OperationLiveEvent::Snapshot)),
                ) as OperationLiveWatch<D::Progress, Value, D::Output>
            },
            cancel,
            signal,
            None,
        );
    }

    /// Register an operation handler whose watch stream includes declared live updates.
    pub fn register_operation_with_updates_and_signal<
        D,
        FStart,
        FutStart,
        FGet,
        FutGet,
        FWatch,
        FCancel,
        FutCancel,
        FSignal,
        FutSignal,
    >(
        &mut self,
        start: FStart,
        get: FGet,
        watch: FWatch,
        cancel: FCancel,
        signal: FSignal,
    ) where
        D: super::OperationUpdateDescriptor + 'static,
        D::Update: serde::Serialize + Send + 'static,
        FStart: Fn(RequestContext, D::Input) -> FutStart + Send + Sync + 'static,
        FutStart: Future<Output = Result<AcceptedOperation<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FGet: Fn(RequestContext, String) -> FutGet + Send + Sync + 'static,
        FutGet: Future<Output = Result<OperationSnapshot<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FWatch: Fn(RequestContext, String) -> OperationLiveWatch<D::Progress, D::Update, D::Output>
            + Send
            + Sync
            + 'static,
        FCancel: Fn(RequestContext, String) -> FutCancel + Send + Sync + 'static,
        FutCancel: Future<Output = Result<OperationSnapshot<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FSignal:
            Fn(RequestContext, String, String, Option<Value>) -> FutSignal + Send + Sync + 'static,
        FutSignal: Future<Output = Result<OperationSignalAccepted<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
    {
        self.register_operation_with_live_watch_and_signal::<D, D::Update, _, _, _, _, _, _, _, _, _>(
            start,
            get,
            watch,
            cancel,
            signal,
            Some(D::UPDATE_SCHEMA_JSON),
        );
    }

    fn register_operation_with_live_watch_and_signal<
        D,
        TUpdate,
        FStart,
        FutStart,
        FGet,
        FutGet,
        FWatch,
        FCancel,
        FutCancel,
        FSignal,
        FutSignal,
    >(
        &mut self,
        start: FStart,
        get: FGet,
        watch: FWatch,
        cancel: FCancel,
        signal: FSignal,
        update_schema_json: Option<&'static str>,
    ) where
        D: OperationDescriptor + 'static,
        TUpdate: serde::Serialize + Send + 'static,
        FStart: Fn(RequestContext, D::Input) -> FutStart + Send + Sync + 'static,
        FutStart: Future<Output = Result<AcceptedOperation<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FGet: Fn(RequestContext, String) -> FutGet + Send + Sync + 'static,
        FutGet: Future<Output = Result<OperationSnapshot<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FWatch: Fn(RequestContext, String) -> OperationLiveWatch<D::Progress, TUpdate, D::Output>
            + Send
            + Sync
            + 'static,
        FCancel: Fn(RequestContext, String) -> FutCancel + Send + Sync + 'static,
        FutCancel: Future<Output = Result<OperationSnapshot<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
        FSignal:
            Fn(RequestContext, String, String, Option<Value>) -> FutSignal + Send + Sync + 'static,
        FutSignal: Future<Output = Result<OperationSignalAccepted<D::Progress, D::Output>, ServerError>>
            + Send
            + 'static,
    {
        let start = Arc::new(start);
        let get = Arc::new(get);
        let watch = Arc::new(watch);
        let cancel = Arc::new(cancel);
        let signal = Arc::new(signal);
        let subject = self.descriptor_subject(D::SUBJECT);
        let handler_subject = subject.clone();
        let caller_capabilities = self.descriptor_capabilities(D::CALLER_CAPABILITIES);
        let observe_capabilities = self.descriptor_capabilities(D::OBSERVE_CAPABILITIES);
        let cancel_capabilities = self.descriptor_capabilities(D::CANCEL_CAPABILITIES);
        let control_capabilities = self.descriptor_capabilities(D::CONTROL_CAPABILITIES);

        self.handlers.insert(
            subject.clone(),
            Route {
                capabilities: RouteCapabilities::Static(caller_capabilities),
                handler: Box::new(
                move |ctx, payload| -> BoxFuture<'static, Result<HandlerResponse, ServerError>> {
                    let start = Arc::clone(&start);
                    Box::pin(async move {
                        let input = parse_validated_input::<D::Input>(&payload, D::INPUT_SCHEMA_JSON)?;
                        let output = start(ctx, input).await?;
                        validate_operation_snapshot::<D>(&output.snapshot)?;
                        Ok(HandlerResponse::Frames(vec![Bytes::from(
                            serde_json::to_vec(&output)?,
                        )]))
                    })
                },
            ),
            },
        );

        self.handlers.insert(
            control_subject(&subject),
            Route {
                capabilities: RouteCapabilities::OperationControl {
                    observe: observe_capabilities,
                    cancel: cancel_capabilities,
                    control: control_capabilities,
                },
                handler: Box::new(
                move |ctx, payload| -> BoxFuture<'static, Result<HandlerResponse, ServerError>> {
                    let get = Arc::clone(&get);
                    let watch = Arc::clone(&watch);
                    let cancel = Arc::clone(&cancel);
                    let signal = Arc::clone(&signal);
                    let subject = handler_subject.clone();
                    let request = serde_json::from_slice::<OperationControlRequest>(&payload)
                        .map_err(ServerError::Json);
                    Box::pin(async move {
                        let request = request?;
                        tracing::debug!(
                            subject = %subject,
                            action = %request.action,
                            operation_id = %request.operation_id,
                            "operation control request"
                        );
                        let frames = match request.action.as_str() {
                            "get" => HandlerResponse::Frames(vec![snapshot_frame::<D>(
                                get(ctx, request.operation_id).await?,
                            )?]),
                            "wait" => {
                                let mut snapshots = watch(ctx, request.operation_id);
                                let mut terminal = None;
                                while let Some(event) = snapshots.next().await {
                                    let event = event?;
                                    if let OperationLiveEvent::Snapshot(snapshot) = event {
                                        if snapshot.state.is_terminal() {
                                            terminal = Some(snapshot);
                                            break;
                                        }
                                    }
                                }
                                let snapshot = terminal.ok_or_else(|| {
                                    ServerError::Nats(
                                        "operation wait ended without terminal snapshot"
                                            .to_string(),
                                    )
                                })?;
                                HandlerResponse::Frames(vec![snapshot_frame::<D>(snapshot)?])
                            }
                            "watch" => {
                                let include_updates = request.include_updates.unwrap_or(false);
                                if include_updates && update_schema_json.is_none() {
                                    return Err(ServerError::InvalidOperationControlAction {
                                        subject: subject.clone(),
                                        action: "watch:updates".to_string(),
                                    });
                                }
                                HandlerResponse::Stream(watch_response_stream::<D, TUpdate>(
                                    watch(ctx, request.operation_id),
                                    include_updates,
                                    update_schema_json,
                                ))
                            }
                            "cancel" if D::CANCELABLE => {
                                HandlerResponse::Frames(vec![snapshot_frame::<D>(
                                    cancel(ctx, request.operation_id).await?,
                                )?])
                            }
                            "signal" => {
                                let signal_name = request.signal.ok_or_else(|| {
                                    ServerError::InvalidOperationControlAction {
                                        subject: subject.clone(),
                                        action: "signal".to_string(),
                                    }
                                })?;
                                let signal_schemas: serde_json::Value =
                                    serde_json::from_str(D::SIGNAL_INPUT_SCHEMAS_JSON)
                                        .map_err(|e| ServerError::Nats(
                                            format!("failed to parse signal schemas: {e}")
                                        ))?;
                                let signal_schema = signal_schemas
                                    .get(&signal_name)
                                    .ok_or_else(|| ServerError::InvalidOperationControlAction {
                                        subject: subject.clone(),
                                        action: format!("signal:{signal_name}"),
                                    })?;
                                let signal_value = request.input.as_ref().unwrap_or(&serde_json::Value::Null);
                                let signal_schema_str = serde_json::to_string(signal_schema)
                                    .map_err(|e| ServerError::Nats(
                                        format!("failed to serialize signal schema: {e}")
                                    ))?;
                                validate_input_schema(&signal_schema_str, signal_value)?;
                                HandlerResponse::Frames(vec![signal_frame::<D>(
                                    signal(ctx, request.operation_id, signal_name, request.input)
                                        .await?,
                                )?])
                            }
                            action => {
                                return Err(ServerError::InvalidOperationControlAction {
                                    subject,
                                    action: action.to_string(),
                                })
                            }
                        };
                        Ok(frames)
                    })
                },
            ),
            },
        );
    }

    /// Register one operation-backed provider.
    pub fn register_operation_provider<D, P>(&mut self, provider: P)
    where
        D: OperationDescriptor + 'static,
        P: OperationProvider<D>,
    {
        let provider = Arc::new(provider);
        self.register_operation::<D, _, _, _, _, _, _, _, _>(
            {
                let provider = Arc::clone(&provider);
                move |context, input| provider.start(context, input)
            },
            {
                let provider = Arc::clone(&provider);
                move |context, operation_id| provider.get(context, operation_id)
            },
            {
                let provider = Arc::clone(&provider);
                move |context, operation_id| provider.wait(context, operation_id)
            },
            move |context, operation_id| provider.cancel(context, operation_id),
        );
    }

    /// Dispatch one request to the registered handler for its subject.
    pub async fn handle_request(
        &self,
        subject: &str,
        payload: Bytes,
        context: RequestContext,
    ) -> Result<Bytes, ServerError> {
        let mut frames = self
            .handle_request_frames(subject, payload, context)
            .await?;
        let first = frames.drain(..).next().ok_or_else(|| {
            ServerError::Nats(format!("handler for '{subject}' returned no response"))
        })?;
        Ok(first)
    }

    /// Return declared capabilities required for the routed request payload.
    pub fn required_capabilities(
        &self,
        subject: &str,
        payload: &[u8],
    ) -> Result<Option<Vec<String>>, ServerError> {
        let route = self
            .handlers
            .get(subject)
            .ok_or_else(|| ServerError::MissingHandler(subject.to_string()))?;
        Ok(route.capabilities.required_for_payload(payload))
    }

    /// Dispatch one request to the registered handler for its subject.
    pub async fn handle_request_frames(
        &self,
        subject: &str,
        payload: Bytes,
        context: RequestContext,
    ) -> Result<Vec<Bytes>, ServerError> {
        match self
            .handle_request_response(subject, payload, context)
            .await?
        {
            HandlerResponse::Frames(frames) => Ok(frames),
            HandlerResponse::Error(payload) => Ok(vec![payload]),
            HandlerResponse::Stream(mut stream) => {
                let mut frames = Vec::new();
                while let Some(frame) = stream.next().await {
                    frames.push(frame?);
                }
                Ok(frames)
            }
            HandlerResponse::FeedStream(mut stream) => {
                let mut frames = Vec::new();
                while let Some(frame) = stream.next().await {
                    frames.push(frame?);
                }
                Ok(frames)
            }
        }
    }

    /// Dispatch one request to the registered handler for its subject.
    pub async fn handle_request_response(
        &self,
        subject: &str,
        payload: Bytes,
        context: RequestContext,
    ) -> Result<HandlerResponse, ServerError> {
        let route = self
            .handlers
            .get(subject)
            .ok_or_else(|| ServerError::MissingHandler(subject.to_string()))?;
        if let Some(reply_to) = feed_cancel_reply_to(&payload)
            .filter(|reply_to| context.reply_to.as_deref() == Some(reply_to.as_str()))
        {
            let key = (subject.to_string(), reply_to);
            let mut states = self
                .feed_cancellations
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let now = Instant::now();
            states.retain(|_, state| {
                !matches!(state, FeedCancellationState::Cancelled(at) if now.duration_since(*at) >= FEED_CANCEL_TOMBSTONE_TTL)
            });
            match states.remove(&key) {
                Some(FeedCancellationState::Active(cancel)) => {
                    let _ = cancel.send(());
                }
                Some(FeedCancellationState::Cancelled(_)) | None => {
                    let tombstones = states
                        .values()
                        .filter(|state| matches!(state, FeedCancellationState::Cancelled(_)))
                        .count();
                    if tombstones < MAX_FEED_CANCEL_TOMBSTONES {
                        states.insert(key, FeedCancellationState::Cancelled(now));
                    }
                }
            }
            return Ok(HandlerResponse::Frames(Vec::new()));
        }
        (route.handler)(context, payload).await
    }
}

fn feed_cancel_reply_to(payload: &[u8]) -> Option<String> {
    let value = serde_json::from_slice::<Value>(payload).ok()?;
    let object = value.as_object()?;
    if object.len() != 1 {
        return None;
    }
    object
        .get("_trellisFeedCancel")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

/// Parse bytes into a valid JSON value, validate against JSON Schema, then
/// deserialize into the target type.
///
/// JSON Schema validation failures become `ServerError::Validation` or
/// `ServerError::SchemaValidation` before handler dispatch.
/// Serde deserialization failures after successful validation are internal errors.
fn parse_validated_input<T>(payload: &[u8], schema_json: &str) -> Result<T, ServerError>
where
    T: serde::de::DeserializeOwned,
{
    let value: serde_json::Value =
        serde_json::from_slice(payload).map_err(|error| ServerError::Validation {
            issues: vec![ValidationIssue {
                path: String::new(),
                message: format!("Invalid JSON: {error}"),
            }],
        })?;

    validate_input_schema(schema_json, &value)?;

    serde_json::from_value::<T>(value).map_err(|error| {
        ServerError::Nats(format!(
            "validated payload failed Rust type decoding: {error}"
        ))
    })
}

fn feed_response_stream<TEvent>(
    events: impl Stream<Item = Result<TEvent, ServerError>> + Send + 'static,
    key: &'static str,
    schema_json: &'static str,
) -> ResponseStream
where
    TEvent: serde::Serialize + 'static,
{
    Box::pin(events.map(move |event| {
        event.and_then(|event| {
            let event = serde_json::to_value(event)?;
            validate_provider_value(key, schema_json, &event)?;
            Ok(Bytes::from(serde_json::to_vec(&event)?))
        })
    }))
}

fn validate_provider_value(
    key: &str,
    schema_json: &str,
    value: &serde_json::Value,
) -> Result<(), ServerError> {
    validate_input_schema(schema_json, value).map_err(|error| {
        tracing::error!(surface = key, %error, "provider emitted an invalid contract payload");
        ServerError::Nats(format!(
            "provider output for `{key}` violated its contract schema"
        ))
    })
}

fn watch_response_stream<D, TUpdate>(
    events: OperationLiveWatch<D::Progress, TUpdate, D::Output>,
    include_updates: bool,
    update_schema_json: Option<&'static str>,
) -> ResponseStream
where
    D: OperationDescriptor,
    TUpdate: serde::Serialize + 'static,
{
    let mut sent_initial_snapshot = false;
    Box::pin(
        events
            .map(move |event| match event {
                Ok(OperationLiveEvent::Snapshot(snapshot)) => {
                    let index = usize::from(sent_initial_snapshot);
                    sent_initial_snapshot = true;
                    Some(operation_watch_frame::<D>(index, snapshot))
                }
                Ok(OperationLiveEvent::Update(update)) if include_updates => {
                    Some(operation_update_frame(update, update_schema_json))
                }
                Ok(OperationLiveEvent::Update(_)) => None,
                Err(error) => Some(Err(error)),
            })
            .filter_map(futures_util::future::ready),
    )
}

fn operation_update_frame<TUpdate>(
    update: crate::client::OperationUpdateEvent<TUpdate>,
    update_schema_json: Option<&str>,
) -> Result<Bytes, ServerError>
where
    TUpdate: serde::Serialize,
{
    let update_value = serde_json::to_value(update.update)?;
    let schema_json = update_schema_json.ok_or_else(|| {
        ServerError::Nats("operation update stream is missing its declared schema".to_string())
    })?;
    validate_input_schema(schema_json, &update_value)?;
    Ok(Bytes::from(serde_json::to_vec(&serde_json::json!({
        "kind": "event",
        "sequence": update.sequence,
        "event": {
            "type": "update",
            "operationId": update.operation_id,
            "sequence": update.sequence,
            "timestamp": update.timestamp,
            "update": update_value,
        }
    }))?))
}

fn snapshot_frame<D>(
    snapshot: OperationSnapshot<D::Progress, D::Output>,
) -> Result<Bytes, ServerError>
where
    D: OperationDescriptor,
{
    validate_operation_snapshot::<D>(&snapshot)?;
    Ok(Bytes::from(serde_json::to_vec(&OperationSnapshotFrame {
        kind: "snapshot".to_string(),
        snapshot,
    })?))
}

fn signal_frame<D>(
    accepted: OperationSignalAccepted<D::Progress, D::Output>,
) -> Result<Bytes, ServerError>
where
    D: OperationDescriptor,
{
    validate_operation_snapshot::<D>(&accepted.snapshot)?;
    Ok(Bytes::from(serde_json::to_vec(&accepted)?))
}

fn operation_watch_frame<D>(
    index: usize,
    snapshot: OperationSnapshot<D::Progress, D::Output>,
) -> Result<Bytes, ServerError>
where
    D: OperationDescriptor,
{
    if index == 0 {
        return snapshot_frame::<D>(snapshot);
    }

    validate_operation_snapshot::<D>(&snapshot)?;

    let event_type = match snapshot.state {
        super::OperationState::Pending => "accepted",
        super::OperationState::Running if snapshot.transfer.is_some() => "transfer",
        super::OperationState::Running if snapshot.progress.is_some() => "progress",
        super::OperationState::Running => "started",
        super::OperationState::Completed => "completed",
        super::OperationState::Failed => "failed",
        super::OperationState::Cancelled => "cancelled",
    };

    let mut event = serde_json::json!({
        "type": event_type,
        "snapshot": snapshot,
    });
    if let Some(progress) = event
        .get("snapshot")
        .and_then(|value| value.get("progress"))
        .cloned()
    {
        event["progress"] = progress;
    }
    if let Some(transfer) = event
        .get("snapshot")
        .and_then(|value| value.get("transfer"))
        .cloned()
    {
        event["transfer"] = transfer;
    }

    Ok(Bytes::from(serde_json::to_vec(&serde_json::json!({
        "kind": "event",
        "sequence": index,
        "event": event,
    }))?))
}

fn validate_operation_snapshot<D>(
    snapshot: &OperationSnapshot<D::Progress, D::Output>,
) -> Result<(), ServerError>
where
    D: OperationDescriptor,
{
    if let (Some(schema), Some(progress)) = (D::PROGRESS_SCHEMA_JSON, &snapshot.progress) {
        validate_provider_value(D::KEY, schema, &serde_json::to_value(progress)?)?;
    }
    if let Some(output) = &snapshot.output {
        validate_provider_value(
            D::KEY,
            D::OUTPUT_SCHEMA_JSON,
            &serde_json::to_value(output)?,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures_util::{stream, StreamExt};

    use super::*;

    struct TestFeed;

    impl FeedDescriptor for TestFeed {
        type Input = Value;
        type Event = Value;

        const KEY: &'static str = "Test.Live";
        const SUBJECT: &'static str = "feeds.v1.Test.Live";
        const INPUT_SCHEMA_JSON: &'static str = r#"{"type":"object"}"#;
        const EVENT_SCHEMA_JSON: &'static str = r#"{"type":"object"}"#;
    }

    #[tokio::test]
    async fn feed_cancel_frame_stops_the_active_response_stream() {
        let mut router = Router::new();
        router.register_feed::<TestFeed, _, _>(|_, _| {
            stream::pending::<Result<Value, ServerError>>()
        });
        let reply_to = "_INBOX.test.feed";
        let context = || RequestContext {
            subject: TestFeed::SUBJECT.to_string(),
            reply_to: Some(reply_to.to_string()),
            ..Default::default()
        };
        let response = router
            .handle_request_response(TestFeed::SUBJECT, Bytes::from_static(b"{}"), context())
            .await
            .expect("open feed");
        let HandlerResponse::FeedStream(mut stream) = response else {
            panic!("feed registration should return a response stream");
        };

        router
            .handle_request_response(
                TestFeed::SUBJECT,
                Bytes::from(
                    serde_json::to_vec(&serde_json::json!({
                        "_trellisFeedCancel": reply_to,
                    }))
                    .expect("serialize cancellation"),
                ),
                context(),
            )
            .await
            .expect("cancel feed");

        assert!(
            tokio::time::timeout(Duration::from_millis(100), stream.next())
                .await
                .expect("feed should stop promptly")
                .is_none()
        );
    }

    #[tokio::test]
    async fn feed_cancel_frame_stops_a_request_that_registers_after_it() {
        let mut router = Router::new();
        router.register_feed::<TestFeed, _, _>(|_, _| {
            stream::pending::<Result<Value, ServerError>>()
        });
        let reply_to = "_INBOX.test.pending-feed";
        let context = || RequestContext {
            subject: TestFeed::SUBJECT.to_string(),
            reply_to: Some(reply_to.to_string()),
            ..Default::default()
        };
        {
            let mut states = router
                .feed_cancellations
                .lock()
                .expect("lock cancellation states");
            for index in 0..MAX_FEED_CANCEL_TOMBSTONES {
                states.insert(
                    (
                        TestFeed::SUBJECT.to_string(),
                        format!("_INBOX.expired.{index}"),
                    ),
                    FeedCancellationState::Cancelled(Instant::now() - FEED_CANCEL_TOMBSTONE_TTL),
                );
            }
        }

        router
            .handle_request_response(
                TestFeed::SUBJECT,
                Bytes::from(
                    serde_json::to_vec(&serde_json::json!({
                        "_trellisFeedCancel": reply_to,
                    }))
                    .expect("serialize cancellation"),
                ),
                context(),
            )
            .await
            .expect("record early cancellation");
        let response = router
            .handle_request_response(TestFeed::SUBJECT, Bytes::from_static(b"{}"), context())
            .await
            .expect("open cancelled feed");
        let HandlerResponse::FeedStream(mut stream) = response else {
            panic!("feed registration should return a response stream");
        };

        assert!(
            tokio::time::timeout(Duration::from_millis(100), stream.next())
                .await
                .expect("early cancellation should stop feed promptly")
                .is_none()
        );
    }
}
