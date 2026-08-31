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
    AuthErrorPayload, AuthorizationContextBundle, AuthorizationContextStore,
    AuthorizationInstallation, CallError, DeclaredError, DeclaredErrorPayload,
    DeclaredOperationUpdates, DeviceConnectOptions, DownloadTransferGrant, EventDescriptor,
    FeedDescriptor, FileInfo, MapStateStore, NoDeclaredError, NoOperationUpdates,
    OperationDescriptor, OperationInvoker, OperationRef, OperationTransferStartError,
    RemoteErrorPayload, RpcDescriptor, StartedOperationTransfer, TransferCancellation,
    TransferOperationDescriptor, TrellisClientError, UserAuthorizationContext, UserConnectOptions,
    UserSessionCredentials, ValueStateStore,
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

    /// Return the connected NATS client for live transport-boundary tests.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    #[must_use]
    pub fn integration_test_nats(&self) -> async_nats::Client {
        self.client.integration_test_nats()
    }

    /// Return the active authorization context digest for live integration synchronization.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn integration_test_authorization_context_digest(
        &self,
    ) -> Result<String, crate::client::TrellisClientError> {
        self.client.integration_test_authorization_context_digest()
    }

    /// Refresh and return the installed native runtime binding for live tests.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub async fn integration_test_refresh_authorization_context(
        &self,
    ) -> Result<crate::client::AuthorizationRuntimeBinding, crate::client::TrellisClientError> {
        self.client
            .integration_test_refresh_authorization_context()
            .await
    }

    /// Close the installed native connection for persisted reconnect tests.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub async fn integration_test_close_native_connection(
        &self,
    ) -> Result<(), crate::client::TrellisClientError> {
        self.client.integration_test_close_native_connection().await
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
    pub async fn connect_device<C>(
        options: crate::client::DeviceConnectOptions<'_, C>,
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

    /// Stream a transfer grant into a caller-owned asynchronous writer.
    #[doc(hidden)]
    pub async fn download_transfer_into<W>(
        &self,
        grant: &crate::client::DownloadTransferGrant,
        writer: &mut W,
    ) -> Result<crate::client::FileInfo, crate::client::TrellisClientError>
    where
        W: tokio::io::AsyncWrite + Unpin + Send + ?Sized,
    {
        self.client.download_transfer_into(grant, writer).await
    }

    /// Stream a transfer grant into a writer with authenticated cancellation.
    #[doc(hidden)]
    pub async fn download_transfer_into_with_cancel<W>(
        &self,
        grant: &crate::client::DownloadTransferGrant,
        writer: &mut W,
        cancellation: &crate::client::TransferCancellation,
    ) -> Result<crate::client::FileInfo, crate::client::TrellisClientError>
    where
        W: tokio::io::AsyncWrite + Unpin + Send + ?Sized,
    {
        self.client
            .download_transfer_into_with_cancel(grant, writer, cancellation)
            .await
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

/// Dynamic contract marker reserved for runtime-authored integration fixtures.
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub struct DynamicDeviceContract;

/// Build dynamic-evidence device options for Trellis integration fixtures.
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub fn test_device_connect_options<'a>(
    trellis_url: &'a str,
    deployment_id: &'a str,
    instance_id: &'a str,
    contract: crate::client::DeviceContractEvidence<'a>,
    public_identity_key: &'a str,
    identity_seed_base64url: &'a str,
    authorization_context_store: Arc<dyn AuthorizationContextStore>,
) -> DeviceConnectOptions<'a, DynamicDeviceContract> {
    crate::client::test_device_connect_options(
        trellis_url,
        deployment_id,
        instance_id,
        contract,
        public_identity_key,
        identity_seed_base64url,
        authorization_context_store,
    )
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

    fn put_upload_transfer_from<'a, R>(
        &'a self,
        grant: crate::client::UploadTransferGrant,
        reader: &'a mut R,
        expected_size: Option<u64>,
    ) -> impl std::future::Future<
        Output = Result<crate::client::FileInfo, crate::client::TrellisClientError>,
    > + Send
           + 'a
    where
        R: tokio::io::AsyncRead + Unpin + Send + ?Sized + 'a,
    {
        crate::client::OperationTransport::put_upload_transfer_from(
            self.client.as_ref(),
            grant,
            reader,
            expected_size,
        )
    }

    fn put_upload_transfer_from_with_cancel<'a, R>(
        &'a self,
        grant: crate::client::UploadTransferGrant,
        reader: &'a mut R,
        expected_size: Option<u64>,
        cancellation: &'a crate::client::TransferCancellation,
    ) -> impl std::future::Future<
        Output = Result<crate::client::FileInfo, crate::client::TrellisClientError>,
    > + Send
           + 'a
    where
        R: tokio::io::AsyncRead + Unpin + Send + ?Sized + 'a,
    {
        crate::client::OperationTransport::put_upload_transfer_from_with_cancel(
            self.client.as_ref(),
            grant,
            reader,
            expected_size,
            cancellation,
        )
    }
}

/// Connect an ad hoc generated service runtime for Trellis integration tests.
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub async fn test_connect_service_runtime<C>(
    options: crate::client::ServiceConnectWithContractOptions<'_>,
) -> Result<crate::service::ConnectedServiceRuntime<C>, crate::service::ServiceRuntimeError> {
    let participant_id = options.participant_id;
    let client = crate::client::TrellisClient::connect_service_with_contract(options).await?;
    crate::service::ConnectedServiceRuntime::from_connected_client(participant_id, Arc::new(client))
}
