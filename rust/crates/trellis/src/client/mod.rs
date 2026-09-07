//! Low-level outbound Trellis runtime primitives for generated Rust code.
//!
//! This module provides connection/auth helpers plus descriptor-driven request
//! and publish operations. It intentionally avoids contract-specific
//! convenience methods so first-party code can move toward generated SDKs and
//! small local wrappers.

mod auth;
mod authorization;
mod connection;
mod descriptor;
mod error;
mod events;
mod http_error;
mod operations;
mod proof;
mod state;
mod subject;
mod transfer;

pub use auth::SessionAuth;
#[cfg(any(test, feature = "runtime-internals"))]
pub use authorization::AuthorizationRegistryBinding;
pub use authorization::{canonical_trellis_origin, AuthorizationProviderCache};
pub use authorization::{
    AuthorizationContextBundle, AuthorizationContextCache, AuthorizationContextPolicy,
    AuthorizationInstallation, AuthorizationNativeTransport, AuthorizationRoutingMaterial,
    AuthorizationRuntimeBinding, AuthorizationRuntimeTransports, AuthorizationVerificationCore,
    AuthorizationVerificationError, EventVerificationInput, RequestVerificationInput,
    VerifiedAuthorizationEvent, VerifiedAuthorizationRequest, VerifiedCaller,
};
#[cfg(feature = "runtime-internals")]
pub use authorization::{RuntimeAuthorizationIoCounters, RuntimeAuthorizationTrust};

pub(crate) use connection::fetch_device_activation;
pub(crate) use connection::DeviceEnrollmentResponse;
pub(crate) use connection::ServiceConnectWithContractOptions;
pub(crate) use connection::TrellisClient;
pub use connection::{
    DeviceConnectOptions, EventMessage, EventReplayPolicy, EventSubscribeOptions,
    EventSubscriptionMode, UserConnectOptions, UserSessionCredentials,
};
pub use descriptor::{EventDescriptor, FeedDescriptor, RpcDescriptor};
pub use error::{
    AuthErrorPayload, AuthenticationError, CallError, DeclaredError, DeclaredErrorPayload,
    NoDeclaredError, ProtocolError, RemoteErrorPayload, RpcErrorPayload,
    SchemaValidationErrorPayload, SchemaValidationIssue, TransportError, TrellisClientError,
    ValidationErrorPayload, ValidationFailure, ValidationIssue,
};
pub use events::{
    dispatch_outbox_once, prepare_event, prepare_event_value, EventStoreError, InboxReceipt,
    InboxStore, MemoryInboxStore, MemoryOutboxStore, OutboxDispatchResult, OutboxEventRecord,
    OutboxStore, PostgresInboxStore, PostgresOutboxStore, PreparedTrellisEvent, SqliteInboxStore,
    SqliteOutboxStore,
};
pub(crate) use http_error::read_bounded_http_body;
pub use http_error::{decode_trellis_http_error, TrellisHttpError};
pub use operations::{
    control_subject, DeclaredOperationUpdates, HasOperationUpdates, NoOperationUpdates,
    OperationDescriptor, OperationEvent, OperationInputBuilder, OperationInvoker, OperationRef,
    OperationRefData, OperationSignalAccepted, OperationSnapshot, OperationState,
    OperationTransferInputBuilder, OperationTransferProgress, OperationTransferReaderInputBuilder,
    OperationTransferStartError, OperationTransport, OperationUpdateEvent, OperationUpdateEvidence,
    StartedOperationTransfer, TransferOperationDescriptor,
};
pub use proof::verify_event_proof;
pub use state::{
    DeleteStateOptions, ExpectedPutRevision, ListStateOptions, MapStateEntry, MapStateListResult,
    MapStateStore, PutStateOptions, StateDeleteResult, StateEntry, StateGetResult,
    StateMigrationRequired, StatePutResult, StateTransport, StateValue, ValueStateStore,
};
pub use subject::SubjectError;
pub use transfer::{
    download_transfer_grant_from_value, DownloadTransferDirection, DownloadTransferGrant, FileInfo,
    TransferCancellation, TransferGrantType, UploadTransferDirection, UploadTransferGrant,
};
mod pagination;
pub use pagination::{PageRequest, PageResponse};

#[cfg(test)]
mod tests;
