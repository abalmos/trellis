//! Runtime ABI used by Trellis-generated Rust crates.
//!
//! Application code should use generated owner SDK and participant facade APIs
//! rather than implementing these traits directly.

use std::sync::Arc;

use futures_util::stream::BoxStream;

/// Generated-code ABI version supported by this runtime.
pub const ABI_VERSION: u32 = 1;

/// Fail compilation when generated source targets a different runtime ABI.
pub const fn assert_abi(version: u32) {
    assert!(version == ABI_VERSION, "generated Trellis ABI mismatch");
}

pub use crate::client::{
    AuthErrorPayload, AuthorizationContextBundle, AuthorizationContextStore, CallError,
    DeclaredError, DeclaredErrorPayload, DeviceConnectOptions, DownloadTransferGrant,
    EventDescriptor, FeedDescriptor, MapStateStore, NoDeclaredError, OperationDescriptor,
    OperationInvoker, OperationRef, OperationTransferStartError, OperationUpdateDescriptor,
    RemoteErrorPayload, RpcDescriptor, StartedOperationTransfer, TransferOperationDescriptor,
    TrellisClientError, UserConnectOptions, ValueStateStore,
};

/// Opaque authenticated caller handle used by generated crates.
#[derive(Clone)]
pub struct Caller {
    client: Arc<crate::client::TrellisClient>,
}

impl Caller {
    /// Return the signed authorization context currently bound to this connection.
    pub fn authorization_context(
        &self,
    ) -> Result<Option<crate::client::AuthorizationContextBundle>, crate::client::TrellisClientError>
    {
        self.client.authorization_context()
    }

    /// Refresh and verify the current authorization context immediately.
    pub async fn refresh_authorization_context(
        &self,
    ) -> Result<crate::client::AuthorizationContextBundle, crate::client::TrellisClientError> {
        self.client.refresh_authorization_context().await
    }

    /// Wrap an authenticated runtime client for generated-code integration.
    #[doc(hidden)]
    pub(crate) fn from_client(client: Arc<crate::client::TrellisClient>) -> Self {
        Self::new(client)
    }

    pub(crate) fn new(client: Arc<crate::client::TrellisClient>) -> Self {
        Self { client }
    }

    /// Resolve a descriptor subject through this connection's integration-test scope.
    #[cfg(all(feature = "integration-test-scoping", feature = "test-support"))]
    #[doc(hidden)]
    pub fn integration_test_descriptor_subject(&self, subject: &str) -> String {
        self.client.descriptor_subject(subject)
    }

    /// Resolve a capability through this connection's integration-test scope.
    #[cfg(all(feature = "integration-test-scoping", feature = "test-support"))]
    #[doc(hidden)]
    pub fn integration_test_descriptor_capability(&self, capability: &str) -> String {
        self.client.integration_test_scope().map_or_else(
            || capability.to_string(),
            |scope| scope.capability(capability),
        )
    }

    /// Connect a user-authenticated generated participant.
    #[doc(hidden)]
    pub async fn connect_user(
        options: crate::client::UserConnectOptions<'_>,
    ) -> Result<Self, crate::client::TrellisClientError> {
        Ok(Self::new(Arc::new(
            crate::client::TrellisClient::connect_user(options).await?,
        )))
    }

    /// Connect an activated-device generated participant.
    #[doc(hidden)]
    pub async fn connect_device(
        options: crate::client::DeviceConnectOptions<'_>,
    ) -> Result<Self, crate::client::TrellisClientError> {
        Ok(Self::new(Arc::new(
            crate::client::TrellisClient::connect_device(options).await?,
        )))
    }

    /// Call one generated RPC descriptor without declared-error decoding.
    #[doc(hidden)]
    pub async fn call<D>(
        &self,
        input: &D::Input,
    ) -> Result<D::Output, crate::client::TrellisClientError>
    where
        D: crate::client::RpcDescriptor,
    {
        self.client.call::<D>(input).await
    }

    pub(crate) async fn request_json_value(
        &self,
        subject: &str,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, crate::client::TrellisClientError> {
        self.client.request_json_value(subject, input).await
    }

    /// Send deliberately malformed wire input from Trellis integration tests.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub async fn test_request_json_value(
        &self,
        subject: &str,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, crate::client::TrellisClientError> {
        self.client.request_json_value(subject, input).await
    }

    /// Download a transfer grant for a generated participant facade.
    #[doc(hidden)]
    pub async fn download_transfer(
        &self,
        grant: &crate::client::DownloadTransferGrant,
    ) -> Result<Vec<u8>, crate::client::TrellisClientError> {
        self.client.download_transfer(grant).await
    }

    /// Flush the authenticated connection from Trellis integration tests.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub async fn test_flush(&self) -> Result<(), crate::client::TrellisClientError> {
        self.client.flush().await
    }

    /// Call one generated RPC descriptor.
    #[doc(hidden)]
    pub async fn call_typed<D, E>(
        &self,
        input: &D::Input,
    ) -> Result<D::Output, crate::client::CallError<E>>
    where
        D: crate::client::RpcDescriptor,
        E: crate::client::DeclaredError,
    {
        self.client.call_typed::<D, E>(input).await
    }

    /// Publish one generated event descriptor.
    #[doc(hidden)]
    pub async fn publish<D>(
        &self,
        event: &D::Event,
    ) -> Result<(), crate::client::TrellisClientError>
    where
        D: crate::client::EventDescriptor,
        D::Event: Send + 'static,
    {
        self.client.publish::<D>(event).await
    }

    /// Subscribe to one generated event descriptor.
    #[doc(hidden)]
    pub async fn subscribe<D>(
        &self,
    ) -> Result<
        BoxStream<'static, Result<D::Event, crate::client::TrellisClientError>>,
        crate::client::TrellisClientError,
    >
    where
        D: crate::client::EventDescriptor,
        D::Event: Send + 'static,
    {
        self.client.subscribe::<D>().await
    }

    /// Open one generated feed descriptor.
    #[doc(hidden)]
    pub async fn feed<D>(
        &self,
        input: &D::Input,
    ) -> Result<
        BoxStream<'static, Result<D::Event, crate::client::TrellisClientError>>,
        crate::client::TrellisClientError,
    >
    where
        D: crate::client::FeedDescriptor,
        D::Event: Send + 'static,
    {
        self.client.feed::<D>(input).await
    }

    /// Build one generated operation invocation.
    #[doc(hidden)]
    pub fn operation<D>(&self) -> crate::client::OperationInvoker<'_, Self, D>
    where
        D: crate::client::OperationDescriptor,
    {
        crate::client::OperationInvoker::new(self)
    }
}

impl crate::client::StateTransport for Caller {
    fn request_state_json<'a>(
        &'a self,
        subject: &'static str,
        body: serde_json::Value,
    ) -> impl std::future::Future<
        Output = Result<serde_json::Value, crate::client::TrellisClientError>,
    > + Send
           + 'a {
        crate::client::StateTransport::request_state_json(self.client.as_ref(), subject, body)
    }
}

impl crate::client::OperationTransport for Caller {
    fn descriptor_subject(&self, subject: &str) -> String {
        self.client.descriptor_subject(subject)
    }

    fn request_json_value<'a>(
        &'a self,
        subject: String,
        body: serde_json::Value,
    ) -> impl std::future::Future<
        Output = Result<serde_json::Value, crate::client::TrellisClientError>,
    > + Send
           + 'a {
        crate::client::OperationTransport::request_json_value(self.client.as_ref(), subject, body)
    }

    fn watch_json_value<'a>(
        &'a self,
        subject: String,
        body: serde_json::Value,
    ) -> impl std::future::Future<
        Output = Result<
            BoxStream<'a, Result<serde_json::Value, crate::client::TrellisClientError>>,
            crate::client::TrellisClientError,
        >,
    > + Send
           + 'a {
        crate::client::OperationTransport::watch_json_value(self.client.as_ref(), subject, body)
    }

    fn put_upload_transfer<'a>(
        &'a self,
        grant: crate::client::UploadTransferGrant,
        body: Vec<u8>,
    ) -> impl std::future::Future<
        Output = Result<crate::client::FileInfo, crate::client::TrellisClientError>,
    > + Send
           + 'a {
        crate::client::OperationTransport::put_upload_transfer(self.client.as_ref(), grant, body)
    }
}

/// Connect an ad hoc generated service runtime for Trellis integration tests.
#[cfg(feature = "test-support")]
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub async fn test_connect_service_runtime<C>(
    trellis_url: &str,
    contract_id: &str,
    contract_digest: &str,
    contract_json: &str,
    deployment_id: &str,
    instance_id: &str,
    identity_seed: &str,
    participant_needs_digest: &str,
    session_seed: &str,
    #[cfg(feature = "integration-test-scoping")] integration_test_scope: Option<
        crate::integration_test_scoping::IntegrationTestScope,
    >,
) -> Result<crate::service::ConnectedServiceRuntime<C>, crate::service::ServiceRuntimeError>
where
    C: crate::service::GeneratedServiceContract,
{
    let client = crate::client::TrellisClient::connect_service_with_contract(
        crate::client::ServiceConnectWithContractOptions {
            trellis_url,
            contract_id,
            contract_digest,
            contract_json,
            deployment_id,
            instance_id,
            provisioned_identity_seed_base64url: identity_seed,
            participant_needs_digest,
            session_key_seed_base64url: session_seed,
            timeout_ms: 30_000,
            retry_delay_ms: crate::service::DEFAULT_RETRY_DELAY_MS,
            authority_pending_timeout_ms: crate::service::DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS,
            authorization_context_store: Arc::new(
                crate::client::MemoryAuthorizationContextStore::default(),
            ),
            #[cfg(feature = "integration-test-scoping")]
            integration_test_scope,
        },
    )
    .await?;
    crate::service::ConnectedServiceRuntime::from_connected_client(contract_id, Arc::new(client))
}
