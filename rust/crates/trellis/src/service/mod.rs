//! Low-level inbound Trellis runtime primitives for generated Rust code.

mod bindings;
#[doc(hidden)]
mod bootstrap_ports;
#[doc(hidden)]
mod core_bootstrap;
#[doc(hidden)]
mod descriptor;
mod error;
mod eventlog_runtime;
mod operations;
#[doc(hidden)]
mod publisher;
#[doc(hidden)]
mod request_loop;
#[doc(hidden)]
mod request_validator_adapter;
mod resources;
#[doc(hidden)]
mod router;
mod runtime;
mod runtime_facade;
#[doc(hidden)]
mod schema_validation;
#[doc(hidden)]
#[expect(
    clippy::module_inception,
    reason = "the service module preserves its established public layout"
)]
mod service;
mod service_host;
mod transfer;

pub use crate::jobs::{ActiveJob, JobDescriptor, JobRef, JobUpdateDescriptor, JobsError};
pub use bindings::{
    validate_bootstrap_contract_state, BootstrapBinding, BootstrapContractRef,
    EventConsumerOrdering, EventConsumerReplay, EventConsumerResourceBinding,
    JobsQueueResourceBinding, JobsResourceBinding, JobsSchemaRef, KvResourceBinding,
    ServiceResourceBindings, StoreResourceBinding,
};
#[doc(hidden)]
pub use bootstrap_ports::{resolve_bootstrap_binding, BootstrapBindingInfo, CoreBootstrapPort};
#[doc(hidden)]
pub use core_bootstrap::{CoreBootstrapAdapter, CoreBootstrapClientPort};
#[doc(hidden)]
pub use descriptor::{EventDescriptor, FeedDescriptor, RpcDescriptor};
pub use error::{
    DeclaredRpcError, HandlerResult, SchemaValidationIssue, ServerError, ValidationIssue,
};
pub use eventlog_runtime::{EventLogMessageStream, EventLogRuntime};
pub use operations::{
    control_subject, AcceptedOperation, InMemoryOperationRuntime, OperationControl,
    OperationControlRequest, OperationDescriptor, OperationError, OperationFailure,
    OperationFailureLike, OperationLiveEvent, OperationProvider, OperationRefData, OperationSignal,
    OperationSignalAccepted, OperationSnapshot, OperationSnapshotFrame, OperationState,
    OperationTransferProgress, OperationUpdateDescriptor, ServiceOperation,
};
#[doc(hidden)]
pub use publisher::EventPublisher;
#[doc(hidden)]
pub use request_validator_adapter::{
    payload_hash_base64url, AuthRequestValidatorAdapter as DefaultRequestValidator,
    AuthRequestValidatorClientPort as DefaultRequestValidatorClientPort,
};
pub use resources::{
    KvHandle, KvResourceClient, KvResourceEntry, KvResourceHandle, KvResourceOperation,
    StoreHandle, StoreResourceClient, StoreResourceHandle, StoreWaitOptions,
};
#[doc(hidden)]
pub use router::{RequestContext, Router};
#[doc(hidden)]
pub use runtime_facade::{
    ConnectedServiceRuntime, CoreBootstrapBinding, GeneratedServiceContract, ServiceHandle,
};
pub use runtime_facade::{
    ServiceConnectOptions, ServiceEventListenOptions, ServiceEventListenerContext,
    ServiceEventListenerHandle, ServiceEventListenerMode, ServiceHandlerContext,
    ServiceOperationLiveWatch, ServiceOperationProvider, ServiceOperationWatch,
    ServiceRuntimeError, DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS, DEFAULT_RETRY_DELAY_MS,
    DEFAULT_TIMEOUT_MS,
};
#[doc(hidden)]
pub use schema_validation::validate_input_schema;
#[doc(hidden)]
pub use service::{AuthenticatedRouter, RequestValidation, RequestValidator};
pub(crate) use service_host::bootstrap_service_host;
#[cfg(test)]
pub(crate) use service_host::ServiceHost;
pub use transfer::{
    decode_upload_transfer_chunk, plan_download_transfer_chunks, plan_download_transfer_chunks_at,
    plan_download_transfer_grant, plan_upload_transfer_grant, DownloadTransferChunk,
    DownloadTransferGrant, DownloadTransferGrantPlan, FileTransferInfo, TransferDownloadGrantArgs,
    TransferUploadGrantArgs, UploadTransferAck, UploadTransferChunk, UploadTransferCompletion,
    UploadTransferGrant, UploadTransferGrantPlan, UploadTransferSession, TRANSFER_EOF_HEADER,
    TRANSFER_SEQUENCE_HEADER,
};

#[doc(hidden)]
pub mod internal {
    use std::sync::Arc;

    pub use super::request_loop::{
        dispatch_one, encode_error_reply, encode_success_reply, HandlerResponse, InboundRequest,
        OutboundReply, RequestHandler, ResponseStream,
    };

    /// Run a Trellis-owned built-in router through normal request authentication.
    pub async fn run_builtin_authenticated_router(
        nats: async_nats::Client,
        auth: Arc<crate::client::SessionAuth>,
        timeout_ms: u64,
        subjects: &[&str],
        router: super::Router,
    ) -> Result<(), super::ServerError> {
        let client = Arc::new(crate::client::TrellisClient::from_internal_parts(
            nats.clone(),
            auth,
            timeout_ms,
        ));
        let validator = super::request_validator_adapter::AuthRequestValidatorAdapter::new(client);
        let router = super::AuthenticatedRouter::new(router, validator);
        super::runtime::run_multi_subject_service(nats, subjects, router).await
    }
}
