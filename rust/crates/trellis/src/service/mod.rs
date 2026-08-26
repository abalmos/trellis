//! Service hosting, generated routing, resources, events, operations, and Jobs.
//!
//! Generated service facades use [`crate::service::ConnectedServiceRuntime`] to own bootstrap,
//! authenticated routing, and lifecycle. Handler registration is descriptor
//! driven; [`crate::service::ServiceHandlerContext`] carries the request and resolved service
//! handles. Runtime-provided [`crate::service::KvHandle`] and
//! [`crate::service::StoreHandle`] values are the
//! supported resource boundary. Jobs are private execution, while operations
//! are caller-visible workflows with progress, updates, cancellation, and named
//! signals.

#[doc(hidden)]
mod authenticated_router;
mod bindings;
#[doc(hidden)]
mod bootstrap_ports;
#[doc(hidden)]
mod core_bootstrap;
#[doc(hidden)]
mod descriptor;
mod error;
mod eventlog_runtime;
mod local_validator;
mod operations;
#[doc(hidden)]
mod publisher;
#[doc(hidden)]
mod request_loop;
mod resources;
#[doc(hidden)]
mod router;
mod runtime;
mod runtime_facade;
#[doc(hidden)]
mod schema_validation;
mod service_host;
mod transfer;

pub use crate::jobs::{ActiveJob, JobDescriptor, JobRef, JobUpdateDescriptor, JobsError};
#[doc(hidden)]
pub use authenticated_router::{AuthenticatedRouter, RequestValidation, RequestValidator};
pub use bindings::{
    validate_bootstrap_contract_state, BootstrapBinding, BootstrapContractRef,
    EventConsumerOrdering, EventConsumerReplay, EventConsumerResourceBinding,
    JobsQueueResourceBinding, JobsResourceBinding, JobsSchemaRef, KvResourceBinding,
    ServiceResourceBindings, StoreResourceBinding,
};
#[doc(hidden)]
pub use bootstrap_ports::BootstrapBindingInfo;
#[doc(hidden)]
pub use descriptor::{EventDescriptor, FeedDescriptor, RpcDescriptor};
pub use error::{
    DeclaredRpcError, HandlerResult, SchemaValidationIssue, ServerError, ValidationIssue,
};
pub use eventlog_runtime::{EventLogMessageStream, EventLogRuntime};
#[doc(hidden)]
pub use local_validator::{
    payload_hash_base64url, EventVerificationFailure, LocalAuthVerifier, VerifiedCaller,
};
pub use operations::{
    control_subject, AcceptedOperation, InMemoryOperationRuntime, OperationControl,
    OperationControlRequest, OperationDescriptor, OperationError, OperationFailure,
    OperationFailureLike, OperationLiveEvent, OperationLiveWatch, OperationRefData,
    OperationSignal, OperationSignalAccepted, OperationSnapshot, OperationSnapshotFrame,
    OperationState, OperationTransferProgress, ServiceOperation, ServiceOperationProvider,
};
#[doc(hidden)]
pub use publisher::EventPublisher;
pub use resources::{
    KvHandle, KvResourceClient, KvResourceEntry, KvResourceHandle, KvResourceOperation,
    StoreHandle, StoreObjectInfo, StoreResourceClient, StoreResourceHandle, StoreWaitOptions,
};
#[doc(hidden)]
pub use router::{RequestContext, RoutePermission, Router};
#[doc(hidden)]
pub use runtime_facade::{
    ConnectedServiceRuntime, CoreBootstrapBinding, GeneratedServiceContract, ServiceHandle,
};
pub use runtime_facade::{
    ServiceConnectOptions, ServiceEventListenOptions, ServiceEventListenerContext,
    ServiceEventListenerHandle, ServiceEventListenerMode, ServiceEventPublisherContext,
    ServiceHandlerContext, ServiceRuntimeError, DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS,
    DEFAULT_RETRY_DELAY_MS, DEFAULT_TIMEOUT_MS,
};
#[doc(hidden)]
pub use schema_validation::validate_input_schema;
pub(crate) use service_host::bootstrap_service_host;
#[cfg(test)]
pub(crate) use service_host::ServiceHost;
pub use transfer::{
    decode_upload_transfer_chunk, plan_download_transfer_grant, plan_upload_transfer_grant,
    DownloadTransferGrant, DownloadTransferGrantPlan, FileTransferInfo, TransferDownloadGrantArgs,
    TransferUploadGrantArgs, UploadTransferAck, UploadTransferChunk, UploadTransferCompletion,
    UploadTransferGrant, UploadTransferGrantPlan, UploadTransferSession, TRANSFER_EOF_HEADER,
    TRANSFER_SEQUENCE_HEADER,
};

#[doc(hidden)]
pub mod internal {
    pub use super::request_loop::{
        dispatch_one, encode_error_reply, encode_success_reply, HandlerResponse, InboundRequest,
        OutboundReply, RequestHandler, ResponseStream,
    };

    /// Run a Trellis-owned built-in router through the supplied local request
    /// verifier. Built-in runtimes without a verifier deny all requests
    /// fail-closed; the runtime Auth-side verifier supplies verification in
    /// platform mode.
    #[cfg(feature = "runtime-internals")]
    pub async fn run_builtin_authenticated_router<V>(
        nats: async_nats::Client,
        api_id: &str,
        subjects: &[&str],
        mut router: super::Router,
        validator: V,
    ) -> Result<(), super::ServerError>
    where
        V: super::RequestValidator + Send + Sync + 'static,
    {
        router.set_api_id(api_id);
        let router = super::AuthenticatedRouter::new(router, validator);
        super::runtime::run_multi_subject_service(nats, subjects, router).await
    }
}
