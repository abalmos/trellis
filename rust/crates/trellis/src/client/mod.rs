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
mod operations;
mod proof;
mod state;
mod subject;
mod transfer;

pub use auth::SessionAuth;
pub use authorization::AuthorizationProviderCache;
#[cfg(any(test, feature = "runtime-internals"))]
pub use authorization::AuthorizationRegistryBinding;
#[cfg(feature = "test-support")]
pub use authorization::IntegrationTestAuthorizationIoCounters;
pub use authorization::{
    AuthorizationClientState, AuthorizationClientTrustState, AuthorizationContextBundle,
    AuthorizationContextCache, AuthorizationContextStore, AuthorizationRoutingMaterial,
    AuthorizationSessionBinding, AuthorizationTrustBundle, AuthorizationTrustPolicy,
    AuthorizationVerificationCore, AuthorizationVerificationError, FileAuthorizationContextStore,
    MemoryAuthorizationContextStore, VerifiedAuthorizationEvent, VerifiedAuthorizationRequest,
    VerifiedCaller,
};
#[cfg(feature = "runtime-internals")]
pub use authorization::{RuntimeAuthorizationIoCounters, RuntimeAuthorizationTrust};

#[cfg(test)]
pub(crate) use authorization::inject_own_verified_for_test;
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub use connection::connect_captured_user_admission;
pub(crate) use connection::fetch_device_activation;
pub(crate) use connection::ServiceBootstrapResponse;
pub(crate) use connection::ServiceConnectWithContractOptions;
pub(crate) use connection::TrellisClient;
#[cfg(feature = "test-support")]
pub(crate) use connection::{
    fetch_device_activation_with_test_proof, DeviceBootstrapProofOverrides,
};
pub use connection::{
    DeviceConnectOptions, EventMessage, EventReplayPolicy, EventSubscribeOptions,
    EventSubscriptionMode, UserConnectOptions,
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
pub use operations::{
    control_subject, OperationDescriptor, OperationEvent, OperationInputBuilder, OperationInvoker,
    OperationRef, OperationRefData, OperationSignalAccepted, OperationSnapshot, OperationState,
    OperationTransferInputBuilder, OperationTransferProgress, OperationTransferStartError,
    OperationTransport, OperationUpdateDescriptor, OperationUpdateEvent, StartedOperationTransfer,
    TransferOperationDescriptor,
};
pub use proof::verify_event_proof;
pub use state::{
    DeleteStateOptions, ExpectedPutRevision, ListStateOptions, MapStateEntry, MapStateListResult,
    MapStateStore, PutStateOptions, StateDeleteResult, StateEntry, StateGetResult,
    StateMigrationRequired, StatePutResult, StateTransport, StateValue, ValueStateStore,
};
pub use subject::SubjectError;
pub use transfer::{
    download_transfer_grant_from_value, DownloadTransferGrant, FileInfo, UploadTransferGrant,
};
pub use trellis_contracts::{PageRequest, PageResponse};

#[cfg(test)]
mod tests;
