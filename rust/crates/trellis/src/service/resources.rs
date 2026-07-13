use std::{fmt, future::Future, io::Cursor, pin::Pin, task::Poll, time::Duration};

use async_nats::jetstream::kv::Operation;
use async_nats::jetstream::object_store::GetErrorKind;
use bytes::Bytes;
use futures_util::{pin_mut, Stream, StreamExt, TryStreamExt};
use time::OffsetDateTime;
use tokio::io::AsyncReadExt;

use super::{KvResourceBinding, ServerError, StoreResourceBinding};

pub(crate) trait ResourceRuntimeClient {
    /// KV client type returned for a bound KV resource.
    type Kv: KvResourceClient;
    /// Object-store client type returned for a bound store resource.
    type Store: StoreResourceClient;

    /// Open the concrete KV bucket described by `binding`.
    fn open_kv(
        &self,
        binding: &KvResourceBinding,
    ) -> impl Future<Output = Result<Self::Kv, ServerError>> + Send;

    /// Open the concrete object-store bucket described by `binding`.
    fn open_store(
        &self,
        binding: &StoreResourceBinding,
    ) -> impl Future<Output = Result<Self::Store, ServerError>> + Send;
}

/// Operations required by a high-level bound KV resource handle.
pub trait KvResourceClient: Clone + fmt::Debug + Send + Sync + 'static {
    /// Watch stream type returned by this client.
    type Watch: Stream<Item = Result<KvResourceEntry, ServerError>> + Send + Unpin + 'static;

    /// Read the latest bytes for `key`, or `None` when the key is absent.
    fn get(&self, key: &str) -> impl Future<Output = Result<Option<Bytes>, ServerError>> + Send;

    /// Read the latest entry metadata for `key`, including delete markers.
    fn get_entry(
        &self,
        key: &str,
    ) -> impl Future<Output = Result<Option<KvResourceEntry>, ServerError>> + Send;

    /// Persist `value` at `key`.
    fn put(&self, key: &str, value: Bytes) -> impl Future<Output = Result<(), ServerError>> + Send;

    /// Persist `value` at `key` only if `key` is still at `revision`.
    fn update_revision(
        &self,
        key: &str,
        value: Bytes,
        revision: u64,
    ) -> impl Future<Output = Result<u64, ServerError>> + Send;

    /// List active keys in this bucket.
    fn list(&self) -> impl Future<Output = Result<Vec<String>, ServerError>> + Send;

    /// Delete `key` from this bucket.
    fn delete(&self, key: &str) -> impl Future<Output = Result<(), ServerError>> + Send;

    /// Delete `key` only if `key` is still at `revision`.
    fn delete_revision(
        &self,
        key: &str,
        revision: u64,
    ) -> impl Future<Output = Result<(), ServerError>> + Send;

    /// Watch updates and deletes for one key.
    fn watch(&self, key: &str) -> impl Future<Output = Result<Self::Watch, ServerError>> + Send;
}

/// Operation that produced a KV entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc = concat!("Public Trellis value set `", stringify!(KvResourceOperation), "`.")]
pub enum KvResourceOperation {
    /// Value bytes were written for the key.
    Update,
    /// The key was deleted or purged.
    Delete,
}

/// Latest KV entry metadata and bytes for a service-bound key.
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(KvResourceEntry), "`.")]
pub struct KvResourceEntry {
    /// Key for this entry.
    #[doc = concat!("The `", stringify!(key), "` value.")]
    pub key: String,
    /// Raw value bytes for this revision.
    #[doc = concat!("The `", stringify!(value), "` value.")]
    pub value: Bytes,
    /// Monotonic bucket revision for this entry.
    #[doc = concat!("The `", stringify!(revision), "` value.")]
    pub revision: u64,
    /// Timestamp assigned by the KV backend.
    #[doc = concat!("The `", stringify!(timestamp), "` value.")]
    pub timestamp: OffsetDateTime,
    /// Operation that produced this entry.
    #[doc = concat!("The `", stringify!(operation), "` value.")]
    pub operation: KvResourceOperation,
}

/// Operations required by a high-level bound object-store resource handle.
pub trait StoreResourceClient: Clone + fmt::Debug + Send + Sync + 'static {
    /// Read all bytes for `key`, or `None` when the object is absent.
    fn read(&self, key: &str) -> impl Future<Output = Result<Option<Bytes>, ServerError>> + Send;

    /// Persist `value` at `key`.
    fn write(
        &self,
        key: &str,
        value: Bytes,
    ) -> impl Future<Output = Result<(), ServerError>> + Send;

    /// List active object names in this store.
    fn list(&self) -> impl Future<Output = Result<Vec<String>, ServerError>> + Send;

    /// Delete `key` from this store.
    fn delete(&self, key: &str) -> impl Future<Output = Result<(), ServerError>> + Send;
}

/// High-level handle for one service-owned KV resource alias.
#[derive(Debug, Clone)]
pub struct KvResourceHandle<C> {
    resource_name: String,
    binding: KvResourceBinding,
    client: C,
}

impl<C> KvResourceHandle<C>
where
    C: KvResourceClient,
{
    /// Create a KV resource handle from a validated binding and opened client.
    #[doc = concat!("Trellis API operation `", stringify!(new), "`.")]
    pub fn new(resource_name: impl Into<String>, binding: KvResourceBinding, client: C) -> Self {
        Self {
            resource_name: resource_name.into(),
            binding,
            client,
        }
    }

    /// Contract-local resource alias used to open this handle.
    #[doc = concat!("Trellis API operation `", stringify!(resource_name), "`.")]
    pub fn resource_name(&self) -> &str {
        &self.resource_name
    }

    /// Concrete resource binding resolved during bootstrap.
    #[doc = concat!("Trellis API operation `", stringify!(binding), "`.")]
    pub fn binding(&self) -> &KvResourceBinding {
        &self.binding
    }

    /// Read the latest bytes for `key`, or `None` when the key is absent.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(get), "`.")]
    pub async fn get(&self, key: &str) -> Result<Option<Bytes>, ServerError> {
        self.client.get(key).await
    }

    /// Read the latest entry metadata for `key`, including delete markers.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(get_entry), "`.")]
    pub async fn get_entry(&self, key: &str) -> Result<Option<KvResourceEntry>, ServerError> {
        self.client.get_entry(key).await
    }

    /// Persist `value` at `key`.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(put), "`.")]
    pub async fn put(&self, key: &str, value: impl Into<Bytes>) -> Result<(), ServerError> {
        self.client.put(key, value.into()).await
    }

    /// Persist `value` at `key` only if `key` is still at `revision`.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(update_revision), "`.")]
    pub async fn update_revision(
        &self,
        key: &str,
        value: impl Into<Bytes>,
        revision: u64,
    ) -> Result<u64, ServerError> {
        self.client
            .update_revision(key, value.into(), revision)
            .await
    }

    /// List active keys in this bucket.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(list), "`.")]
    pub async fn list(&self) -> Result<Vec<String>, ServerError> {
        self.client.list().await
    }

    /// Delete `key` from this bucket.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(delete), "`.")]
    pub async fn delete(&self, key: &str) -> Result<(), ServerError> {
        self.client.delete(key).await
    }

    /// Delete `key` only if `key` is still at `revision`.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(delete_revision), "`.")]
    pub async fn delete_revision(&self, key: &str, revision: u64) -> Result<(), ServerError> {
        self.client.delete_revision(key, revision).await
    }

    /// Watch updates and deletes for one key.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(watch), "`.")]
    pub async fn watch(&self, key: &str) -> Result<C::Watch, ServerError> {
        self.client.watch(key).await
    }
}

/// High-level handle for one service-owned object-store resource alias.
#[derive(Debug, Clone)]
pub struct StoreResourceHandle<C> {
    service_name: String,
    resource_name: String,
    binding: StoreResourceBinding,
    client: C,
}

/// Options for waiting until an object appears in a bound object store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(StoreWaitOptions), "`.")]
pub struct StoreWaitOptions {
    /// Maximum time to wait before returning [`ServerError::StoreWaitTimeout`].
    #[doc = concat!("The `", stringify!(timeout), "` value.")]
    pub timeout: Option<Duration>,
    /// Delay between object existence checks. Defaults to 250ms.
    #[doc = concat!("The `", stringify!(poll_interval), "` value.")]
    pub poll_interval: Duration,
}

impl Default for StoreWaitOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            poll_interval: Duration::from_millis(250),
        }
    }
}

impl<C> StoreResourceHandle<C>
where
    C: StoreResourceClient,
{
    /// Create a store resource handle from a validated binding and opened client.
    #[doc = concat!("Trellis API operation `", stringify!(new), "`.")]
    pub fn new(
        service_name: impl Into<String>,
        resource_name: impl Into<String>,
        binding: StoreResourceBinding,
        client: C,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            resource_name: resource_name.into(),
            binding,
            client,
        }
    }

    /// Contract-local resource alias used to open this handle.
    #[doc = concat!("Trellis API operation `", stringify!(resource_name), "`.")]
    pub fn resource_name(&self) -> &str {
        &self.resource_name
    }

    /// Concrete resource binding resolved during bootstrap.
    #[doc = concat!("Trellis API operation `", stringify!(binding), "`.")]
    pub fn binding(&self) -> &StoreResourceBinding {
        &self.binding
    }

    /// Read all bytes for `key`, or `None` when the object is absent.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(read), "`.")]
    pub async fn read(&self, key: &str) -> Result<Option<Bytes>, ServerError> {
        self.client.read(key).await
    }

    /// Wait until `key` appears in this store, then return its bytes.
    ///
    /// The handle checks immediately, then polls according to `options`. When
    /// `options.timeout` elapses before the object appears, this returns
    /// [`ServerError::StoreWaitTimeout`].
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(wait_for), "`.")]
    pub async fn wait_for(
        &self,
        key: &str,
        options: StoreWaitOptions,
    ) -> Result<Bytes, ServerError> {
        self.wait_for_with_cancel(key, options, std::future::pending::<()>())
            .await
    }

    /// Wait until `key` appears in this store, or until `cancel` resolves.
    ///
    /// This has the same timeout behavior as [`StoreResourceHandle::wait_for`].
    /// If `cancel` resolves first, this returns
    /// [`ServerError::StoreWaitCanceled`].
    pub async fn wait_for_with_cancel<F>(
        &self,
        key: &str,
        options: StoreWaitOptions,
        cancel: F,
    ) -> Result<Bytes, ServerError>
    where
        F: Future<Output = ()> + Send,
    {
        pin_mut!(cancel);
        let started = tokio::time::Instant::now();
        let deadline = options.timeout.map(|timeout| started + timeout);
        loop {
            let read = self.read(key);
            if let (Some(deadline), Some(timeout_duration)) = (deadline, options.timeout) {
                let timeout = tokio::time::sleep_until(deadline);
                tokio::pin!(timeout);
                tokio::select! {
                    biased;
                    () = &mut cancel => {
                        return Err(self.store_wait_canceled_error(key));
                    }
                    result = read => {
                        if let Some(bytes) = result? {
                            return Ok(bytes);
                        }
                    }
                    () = &mut timeout => {
                        return Err(self.store_wait_timeout_error(key, timeout_duration));
                    }
                }
            } else {
                tokio::select! {
                    biased;
                    () = &mut cancel => {
                        return Err(self.store_wait_canceled_error(key));
                    }
                    result = read => {
                        if let Some(bytes) = result? {
                            return Ok(bytes);
                        }
                    }
                }
            }

            let poll_interval = options.poll_interval.max(Duration::from_millis(1));
            let delay = if let (Some(deadline), Some(timeout)) = (deadline, options.timeout) {
                let now = tokio::time::Instant::now();
                if now >= deadline {
                    return Err(self.store_wait_timeout_error(key, timeout));
                }
                poll_interval.min(deadline - now)
            } else {
                poll_interval
            };

            tokio::select! {
                biased;
                () = &mut cancel => {
                    return Err(self.store_wait_canceled_error(key));
                }
                () = tokio::time::sleep(delay) => {}
            }
        }
    }

    fn store_wait_timeout_error(&self, key: &str, timeout: Duration) -> ServerError {
        ServerError::StoreWaitTimeout {
            service_name: self.service_name.clone(),
            store: self.resource_name.clone(),
            key: key.to_string(),
            timeout_ms: timeout.as_millis(),
        }
    }

    fn store_wait_canceled_error(&self, key: &str) -> ServerError {
        ServerError::StoreWaitCanceled {
            service_name: self.service_name.clone(),
            store: self.resource_name.clone(),
            key: key.to_string(),
        }
    }

    /// Persist `value` at `key`.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(write), "`.")]
    pub async fn write(&self, key: &str, value: impl Into<Bytes>) -> Result<(), ServerError> {
        let value = value.into();
        if let Some(max_bytes) = self
            .binding
            .max_object_bytes
            .and_then(|value| u64::try_from(value).ok())
        {
            if value.len() as u64 > max_bytes {
                return Err(ServerError::TransferObjectTooLarge {
                    service_name: self.service_name.clone(),
                    store: self.resource_name.clone(),
                    key: key.to_string(),
                    size: value.len() as u64,
                    max_bytes,
                });
            }
        }

        self.client.write(key, value).await
    }

    /// List active object names in this store.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(list), "`.")]
    pub async fn list(&self) -> Result<Vec<String>, ServerError> {
        self.client.list().await
    }

    /// Delete `key` from this store.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(delete), "`.")]
    pub async fn delete(&self, key: &str) -> Result<(), ServerError> {
        self.client.delete(key).await
    }
}

impl<C> StoreResourceClient for StoreResourceHandle<C>
where
    C: StoreResourceClient,
{
    async fn read(&self, key: &str) -> Result<Option<Bytes>, ServerError> {
        self.client.read(key).await
    }

    async fn write(&self, key: &str, value: Bytes) -> Result<(), ServerError> {
        self.client.write(key, value).await
    }

    async fn list(&self) -> Result<Vec<String>, ServerError> {
        self.client.list().await
    }

    async fn delete(&self, key: &str) -> Result<(), ServerError> {
        self.client.delete(key).await
    }
}

/// Concrete KV client used by connected service resources.
#[derive(Debug, Clone)]
#[doc = concat!("Public Trellis data type `", stringify!(BoundKvResourceClient), "`.")]
pub struct BoundKvResourceClient {
    store: async_nats::jetstream::kv::Store,
}

/// Watch stream for connected KV resources.
#[doc = concat!("Public Trellis data type `", stringify!(BoundKvWatch), "`.")]
pub struct BoundKvWatch {
    inner: async_nats::jetstream::kv::Watch,
}

impl fmt::Debug for BoundKvWatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundKvWatch")
            .finish_non_exhaustive()
    }
}

impl Stream for BoundKvWatch {
    type Item = Result<KvResourceEntry, ServerError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner)
            .poll_next(cx)
            .map(|entry| entry.map(|entry| entry.map(kv_entry_from_nats).map_err(nats_error)))
    }
}

impl KvResourceClient for BoundKvResourceClient {
    type Watch = BoundKvWatch;

    async fn get(&self, key: &str) -> Result<Option<Bytes>, ServerError> {
        self.store.get(key.to_string()).await.map_err(nats_error)
    }

    async fn get_entry(&self, key: &str) -> Result<Option<KvResourceEntry>, ServerError> {
        self.store
            .entry(key.to_string())
            .await
            .map(|entry| entry.map(kv_entry_from_nats))
            .map_err(nats_error)
    }

    async fn put(&self, key: &str, value: Bytes) -> Result<(), ServerError> {
        self.store
            .put(key, value)
            .await
            .map(|_| ())
            .map_err(nats_error)
    }

    async fn update_revision(
        &self,
        key: &str,
        value: Bytes,
        revision: u64,
    ) -> Result<u64, ServerError> {
        match self.store.update(key, value, revision).await {
            Ok(revision) => Ok(revision),
            Err(error) if is_revision_mismatch(&error) => {
                Err(self.kv_revision_mismatch(key, revision).await)
            }
            Err(error) => Err(nats_error(error)),
        }
    }

    async fn list(&self) -> Result<Vec<String>, ServerError> {
        let keys = self.store.keys().await.map_err(nats_error)?;
        keys.try_collect().await.map_err(nats_error)
    }

    async fn delete(&self, key: &str) -> Result<(), ServerError> {
        self.store.delete(key).await.map_err(nats_error)
    }

    async fn delete_revision(&self, key: &str, revision: u64) -> Result<(), ServerError> {
        match self.store.delete_expect_revision(key, Some(revision)).await {
            Ok(()) => Ok(()),
            Err(error) if is_revision_mismatch(&error) => {
                Err(self.kv_revision_mismatch(key, revision).await)
            }
            Err(error) => Err(nats_error(error)),
        }
    }

    async fn watch(&self, key: &str) -> Result<Self::Watch, ServerError> {
        self.store
            .watch(key)
            .await
            .map(|inner| BoundKvWatch { inner })
            .map_err(nats_error)
    }
}

impl BoundKvResourceClient {
    async fn kv_revision_mismatch(&self, key: &str, expected: u64) -> ServerError {
        let actual = self
            .store
            .entry(key.to_string())
            .await
            .ok()
            .flatten()
            .map(|entry| entry.revision);
        ServerError::KvRevisionMismatch {
            key: key.to_string(),
            expected,
            actual,
        }
    }
}

fn is_revision_mismatch(error: &impl fmt::Display) -> bool {
    let message = error.to_string().to_ascii_lowercase();
    message.contains("wrong last sequence")
        || message.contains("wrong last revision")
        || message.contains("revision mismatch")
        || message.contains("sequence mismatch")
}

fn kv_entry_from_nats(entry: async_nats::jetstream::kv::Entry) -> KvResourceEntry {
    KvResourceEntry {
        key: entry.key,
        value: entry.value,
        revision: entry.revision,
        timestamp: entry.created,
        operation: match entry.operation {
            Operation::Put => KvResourceOperation::Update,
            Operation::Delete | Operation::Purge => KvResourceOperation::Delete,
        },
    }
}

/// Concrete object-store client used by connected service resources.
#[derive(Clone)]
#[doc = concat!("Public Trellis data type `", stringify!(BoundStoreResourceClient), "`.")]
pub struct BoundStoreResourceClient {
    store: async_nats::jetstream::object_store::ObjectStore,
}

/// Connected handle for one contract-declared KV resource.
pub type KvHandle = KvResourceHandle<BoundKvResourceClient>;

/// Connected handle for one contract-declared object-store resource.
pub type StoreHandle = StoreResourceHandle<BoundStoreResourceClient>;

impl fmt::Debug for BoundStoreResourceClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundStoreResourceClient")
            .finish_non_exhaustive()
    }
}

impl StoreResourceClient for BoundStoreResourceClient {
    async fn read(&self, key: &str) -> Result<Option<Bytes>, ServerError> {
        let mut object = match self.store.get(key).await {
            Ok(object) => object,
            Err(error) if error.kind() == GetErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(nats_error(error)),
        };
        let mut bytes = Vec::new();
        object.read_to_end(&mut bytes).await.map_err(nats_error)?;
        Ok(Some(bytes.into()))
    }

    async fn write(&self, key: &str, value: Bytes) -> Result<(), ServerError> {
        let mut reader = Cursor::new(value);
        self.store
            .put(key, &mut reader)
            .await
            .map(|_| ())
            .map_err(nats_error)
    }

    async fn list(&self) -> Result<Vec<String>, ServerError> {
        let objects = self.store.list().await.map_err(nats_error)?;
        objects
            .map(|object| object.map(|info| info.name).map_err(nats_error))
            .try_collect()
            .await
    }

    async fn delete(&self, key: &str) -> Result<(), ServerError> {
        self.store.delete(key).await.map_err(nats_error)
    }
}

impl ResourceRuntimeClient for async_nats::Client {
    type Kv = BoundKvResourceClient;
    type Store = BoundStoreResourceClient;

    async fn open_kv(&self, binding: &KvResourceBinding) -> Result<Self::Kv, ServerError> {
        let context = async_nats::jetstream::new(self.clone());
        let store = context
            .get_key_value(binding.bucket.clone())
            .await
            .map_err(nats_error)?;
        Ok(BoundKvResourceClient { store })
    }

    async fn open_store(&self, binding: &StoreResourceBinding) -> Result<Self::Store, ServerError> {
        let context = async_nats::jetstream::new(self.clone());
        let store = context
            .get_object_store(&binding.name)
            .await
            .map_err(nats_error)?;
        Ok(BoundStoreResourceClient { store })
    }
}

#[doc = concat!("Trellis API operation `", stringify!(validate_kv_binding), "`.")]
pub fn validate_kv_binding(
    service_name: &str,
    resource_name: &str,
    binding: &KvResourceBinding,
) -> Result<(), ServerError> {
    if binding.bucket.is_empty() {
        return Err(invalid_binding(
            service_name,
            "kv",
            resource_name,
            "bucket name is empty",
        ));
    }
    if !is_valid_nats_resource_name(&binding.bucket) {
        return Err(invalid_binding(
            service_name,
            "kv",
            resource_name,
            "bucket name must contain only ASCII letters, digits, underscores, and hyphens",
        ));
    }
    if binding.history < 1 {
        return Err(invalid_binding(
            service_name,
            "kv",
            resource_name,
            "history must be greater than zero",
        ));
    }
    if matches!(binding.max_value_bytes, Some(max_bytes) if max_bytes < 0) {
        return Err(invalid_binding(
            service_name,
            "kv",
            resource_name,
            "max_value_bytes must not be negative",
        ));
    }
    if binding.ttl_ms < 0 {
        return Err(invalid_binding(
            service_name,
            "kv",
            resource_name,
            "ttl_ms must not be negative",
        ));
    }
    Ok(())
}

#[doc = concat!("Trellis API operation `", stringify!(validate_store_binding), "`.")]
pub fn validate_store_binding(
    service_name: &str,
    resource_name: &str,
    binding: &StoreResourceBinding,
) -> Result<(), ServerError> {
    if binding.name.is_empty() {
        return Err(invalid_binding(
            service_name,
            "store",
            resource_name,
            "store name is empty",
        ));
    }
    if !is_valid_nats_resource_name(&binding.name) {
        return Err(invalid_binding(
            service_name,
            "store",
            resource_name,
            "store name must contain only ASCII letters, digits, underscores, and hyphens",
        ));
    }
    if matches!(binding.max_object_bytes, Some(max_bytes) if max_bytes < 0) {
        return Err(invalid_binding(
            service_name,
            "store",
            resource_name,
            "max_object_bytes must not be negative",
        ));
    }
    if matches!(binding.max_total_bytes, Some(max_bytes) if max_bytes < 0) {
        return Err(invalid_binding(
            service_name,
            "store",
            resource_name,
            "max_total_bytes must not be negative",
        ));
    }
    if binding.ttl_ms < 0 {
        return Err(invalid_binding(
            service_name,
            "store",
            resource_name,
            "ttl_ms must not be negative",
        ));
    }
    Ok(())
}

fn invalid_binding(
    service_name: &str,
    resource_kind: &str,
    resource_name: &str,
    reason: &str,
) -> ServerError {
    ServerError::InvalidResourceBinding {
        service_name: service_name.to_string(),
        resource_kind: resource_kind.to_string(),
        resource_name: resource_name.to_string(),
        reason: reason.to_string(),
    }
}

fn is_valid_nats_resource_name(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn nats_error(error: impl fmt::Display) -> ServerError {
    ServerError::Nats(error.to_string())
}
