use std::{
    fmt,
    future::{pending, Future},
    io::Cursor,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use async_nats::jetstream::kv::Operation;
use async_nats::jetstream::object_store::{GetErrorKind, PutErrorKind};
use bytes::Bytes;
use futures_util::{pin_mut, Stream, StreamExt, TryStreamExt};
use time::OffsetDateTime;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

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

/// Contract-safe metadata for one object-store object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreObjectInfo {
    /// Logical object key.
    pub key: String,
    /// Object size in bytes.
    pub size: u64,
    /// Backend-verified digest when available.
    pub digest: Option<String>,
    /// Last modification time when available.
    pub modified_at: Option<OffsetDateTime>,
}

/// Operations required by a high-level bound object-store resource handle.
pub trait StoreResourceClient: Clone + fmt::Debug + Send + Sync + 'static {
    /// Stream `key` into `writer`, or return `None` when the object is absent.
    ///
    /// This method does not flush or shut down the caller-owned writer.
    fn read_into<W>(
        &self,
        key: &str,
        writer: &mut W,
    ) -> impl Future<Output = Result<Option<StoreObjectInfo>, ServerError>> + Send
    where
        W: AsyncWrite + Unpin + Send;

    /// Stream an object from `reader` into the store.
    fn write_from<R>(
        &self,
        key: &str,
        reader: &mut R,
    ) -> impl Future<Output = Result<StoreObjectInfo, ServerError>> + Send
    where
        R: AsyncRead + Unpin + Send;

    /// Read all bytes for `key`, or `None` when the object is absent.
    fn read(&self, key: &str) -> impl Future<Output = Result<Option<Bytes>, ServerError>> + Send {
        async move {
            let mut writer = Cursor::new(Vec::new());
            Ok(self
                .read_into(key, &mut writer)
                .await?
                .map(|_| Bytes::from(writer.into_inner())))
        }
    }

    /// Persist a complete in-memory object at `key`.
    fn write(
        &self,
        key: &str,
        value: Bytes,
    ) -> impl Future<Output = Result<(), ServerError>> + Send {
        async move {
            let mut reader = Cursor::new(value);
            self.write_from(key, &mut reader).await?;
            Ok(())
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UploadReadFailure {
    TooLarge {
        attempted_bytes: u64,
        max_bytes: u64,
    },
    SizeMismatch {
        expected_bytes: u64,
        actual_bytes: u64,
    },
    Cancelled,
}

struct GuardedUploadReader<R, F> {
    reader: R,
    cancel: Pin<Box<F>>,
    expected_size: Option<u64>,
    max_size: Option<u64>,
    read: u64,
    validated_eof: bool,
    failure: Option<UploadReadFailure>,
}

impl<R, F> GuardedUploadReader<R, F> {
    fn new(reader: R, expected_size: Option<u64>, max_size: Option<u64>, cancel: F) -> Self {
        Self {
            reader,
            cancel: Box::pin(cancel),
            expected_size,
            max_size,
            read: 0,
            validated_eof: false,
            failure: None,
        }
    }

    fn limit(&self) -> Option<u64> {
        match (self.expected_size, self.max_size) {
            (Some(expected), Some(max)) => Some(expected.min(max)),
            (Some(expected), None) => Some(expected),
            (None, max) => max,
        }
    }

    fn crossing_failure(&self) -> UploadReadFailure {
        if let Some(expected_bytes) = self.expected_size {
            UploadReadFailure::SizeMismatch {
                expected_bytes,
                actual_bytes: expected_bytes.saturating_add(1),
            }
        } else {
            let max_bytes = self.max_size.expect("a crossing requires a size limit");
            UploadReadFailure::TooLarge {
                attempted_bytes: max_bytes.saturating_add(1),
                max_bytes,
            }
        }
    }
}

impl<R, F> AsyncRead for GuardedUploadReader<R, F>
where
    R: AsyncRead + Unpin,
    F: Future<Output = ()>,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        if this.validated_eof {
            return Poll::Ready(Ok(()));
        }
        if this.failure.is_some() {
            return Poll::Ready(Err(std::io::Error::other("guarded upload terminated")));
        }
        if this.cancel.as_mut().poll(cx).is_ready() {
            this.failure = Some(UploadReadFailure::Cancelled);
            return Poll::Ready(Err(std::io::Error::other("store upload cancelled")));
        }

        let limit = this.limit();
        if limit.is_some_and(|limit| this.read == limit) {
            let mut extra = [0_u8; 1];
            let mut probe = ReadBuf::new(&mut extra);
            return match Pin::new(&mut this.reader).poll_read(cx, &mut probe) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
                Poll::Ready(Ok(())) if probe.filled().is_empty() => {
                    this.validated_eof = true;
                    Poll::Ready(Ok(()))
                }
                Poll::Ready(Ok(())) => {
                    this.failure = Some(this.crossing_failure());
                    Poll::Ready(Err(std::io::Error::other(
                        "store upload size limit exceeded",
                    )))
                }
            };
        }

        let remaining = limit
            .map(|limit| usize::try_from(limit - this.read).unwrap_or(usize::MAX))
            .unwrap_or(buf.remaining())
            .min(buf.remaining());
        let unfilled = buf.initialize_unfilled_to(remaining);
        let mut bounded = ReadBuf::new(unfilled);
        match Pin::new(&mut this.reader).poll_read(cx, &mut bounded) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Ready(Ok(())) => {
                let count = bounded.filled().len();
                buf.advance(count);
                let Some(actual) = this
                    .read
                    .checked_add(u64::try_from(count).unwrap_or(u64::MAX))
                else {
                    return Poll::Ready(Err(std::io::Error::other("store upload size overflow")));
                };
                this.read = actual;
                if count == 0 {
                    if let Some(expected_bytes) = this.expected_size {
                        if actual != expected_bytes {
                            this.failure = Some(UploadReadFailure::SizeMismatch {
                                expected_bytes,
                                actual_bytes: actual,
                            });
                            return Poll::Ready(Err(std::io::Error::other(
                                "store upload size mismatch",
                            )));
                        }
                    }
                    this.validated_eof = true;
                }
                Poll::Ready(Ok(()))
            }
        }
    }
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

    /// Stream an object from `reader` while enforcing the bound maximum and exact expected size.
    pub async fn write_from<R>(
        &self,
        key: &str,
        reader: R,
        expected_size: Option<u64>,
    ) -> Result<StoreObjectInfo, ServerError>
    where
        R: AsyncRead + Unpin + Send,
    {
        self.write_from_with_cancel(key, reader, expected_size, pending())
            .await
    }

    /// Stream an object from `reader`, aborting before validated EOF when `cancel` resolves.
    ///
    /// Once validated source EOF has been observed, backend metadata commit is awaited and
    /// cancellation can no longer promise rollback.
    pub async fn write_from_with_cancel<R, F>(
        &self,
        key: &str,
        reader: R,
        expected_size: Option<u64>,
        cancel: F,
    ) -> Result<StoreObjectInfo, ServerError>
    where
        R: AsyncRead + Unpin + Send,
        F: Future<Output = ()> + Send,
    {
        let max_size = self
            .binding
            .max_object_bytes
            .and_then(|value| value.try_into().ok());
        if let (Some(expected_bytes), Some(max_bytes)) = (expected_size, max_size) {
            if expected_bytes > max_bytes {
                return Err(ServerError::StoreObjectTooLarge {
                    attempted_bytes: expected_bytes,
                    max_bytes,
                });
            }
        }

        let mut guarded = GuardedUploadReader::new(reader, expected_size, max_size, cancel);
        let result = self.client.write_from(key, &mut guarded).await;
        match guarded.failure {
            Some(UploadReadFailure::TooLarge {
                attempted_bytes,
                max_bytes,
            }) => Err(ServerError::StoreObjectTooLarge {
                attempted_bytes,
                max_bytes,
            }),
            Some(UploadReadFailure::SizeMismatch {
                expected_bytes,
                actual_bytes,
            }) => Err(ServerError::StoreObjectSizeMismatch {
                expected_bytes,
                actual_bytes,
            }),
            Some(UploadReadFailure::Cancelled) => Err(ServerError::StoreWriteCancelled),
            None => result,
        }
    }

    /// Stream an object into `writer`; the writer is neither flushed nor shut down.
    pub async fn read_into<W>(
        &self,
        key: &str,
        writer: W,
    ) -> Result<Option<StoreObjectInfo>, ServerError>
    where
        W: AsyncWrite + Unpin + Send,
    {
        self.read_into_with_cancel(key, writer, pending()).await
    }

    /// Stream an object into `writer`, returning cancellation after any already-written prefix.
    pub async fn read_into_with_cancel<W, F>(
        &self,
        key: &str,
        mut writer: W,
        cancel: F,
    ) -> Result<Option<StoreObjectInfo>, ServerError>
    where
        W: AsyncWrite + Unpin + Send,
        F: Future<Output = ()> + Send,
    {
        tokio::pin!(cancel);
        tokio::select! {
            biased;
            () = &mut cancel => Err(ServerError::StoreReadCancelled),
            result = self.client.read_into(key, &mut writer) => result,
        }
    }

    /// Read all bytes for `key`, or `None` when the object is absent.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(read), "`.")]
    pub async fn read(&self, key: &str) -> Result<Option<Bytes>, ServerError> {
        let mut writer = Cursor::new(Vec::new());
        Ok(self
            .read_into(key, &mut writer)
            .await?
            .map(|_| Bytes::from(writer.into_inner())))
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
        let expected_size = u64::try_from(value.len()).map_err(|_| {
            ServerError::Nats("store object length does not fit in u64".to_string())
        })?;
        let mut reader = Cursor::new(value);
        self.write_from(key, &mut reader, Some(expected_size))
            .await?;
        Ok(())
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
    async fn read_into<W>(
        &self,
        key: &str,
        writer: &mut W,
    ) -> Result<Option<StoreObjectInfo>, ServerError>
    where
        W: AsyncWrite + Unpin + Send,
    {
        StoreResourceHandle::read_into(self, key, writer).await
    }

    async fn write_from<R>(&self, key: &str, reader: &mut R) -> Result<StoreObjectInfo, ServerError>
    where
        R: AsyncRead + Unpin + Send,
    {
        StoreResourceHandle::write_from(self, key, reader, None).await
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
    async fn read_into<W>(
        &self,
        key: &str,
        writer: &mut W,
    ) -> Result<Option<StoreObjectInfo>, ServerError>
    where
        W: AsyncWrite + Unpin + Send,
    {
        let mut object = match self.store.get(key).await {
            Ok(object) => object,
            Err(error) if error.kind() == GetErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(nats_error(error)),
        };
        let info = store_object_info(object.info())?;
        tokio::io::copy(&mut object, writer)
            .await
            .map_err(nats_error)?;
        Ok(Some(info))
    }

    async fn write_from<R>(&self, key: &str, reader: &mut R) -> Result<StoreObjectInfo, ServerError>
    where
        R: AsyncRead + Unpin + Send,
    {
        let mut reader = reader;
        match self.store.put(key, &mut reader).await {
            Ok(info) => store_object_info(&info),
            Err(error) if error.kind() == PutErrorKind::PublishMetadata => {
                Err(ServerError::StoreCommitIndeterminate {
                    key: key.to_string(),
                    message: error.to_string(),
                })
            }
            Err(error) if error.kind() == PutErrorKind::PurgeOldChunks => {
                Err(ServerError::StoreCommittedCleanupFailed {
                    key: key.to_string(),
                    message: error.to_string(),
                })
            }
            Err(error) => Err(nats_error(error)),
        }
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

fn store_object_info(
    info: &async_nats::jetstream::object_store::ObjectInfo,
) -> Result<StoreObjectInfo, ServerError> {
    Ok(StoreObjectInfo {
        key: info.name.clone(),
        size: u64::try_from(info.size)
            .map_err(|_| ServerError::Nats("store object size does not fit in u64".to_string()))?,
        digest: info.digest.clone(),
        modified_at: info.modified,
    })
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

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    #[derive(Debug, Clone, Default)]
    struct RecordingStore {
        writes: Arc<AtomicUsize>,
        bytes: Arc<Mutex<Vec<u8>>>,
    }

    impl StoreResourceClient for RecordingStore {
        async fn read_into<W>(
            &self,
            key: &str,
            writer: &mut W,
        ) -> Result<Option<StoreObjectInfo>, ServerError>
        where
            W: AsyncWrite + Unpin + Send,
        {
            let bytes = self.bytes.lock().expect("recording store lock").clone();
            writer.write_all(&bytes).await.map_err(nats_error)?;
            Ok(Some(StoreObjectInfo {
                key: key.to_string(),
                size: bytes.len() as u64,
                digest: None,
                modified_at: None,
            }))
        }

        async fn write_from<R>(
            &self,
            key: &str,
            reader: &mut R,
        ) -> Result<StoreObjectInfo, ServerError>
        where
            R: AsyncRead + Unpin + Send,
        {
            self.writes.fetch_add(1, Ordering::SeqCst);
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await.map_err(nats_error)?;
            let size = bytes.len() as u64;
            *self.bytes.lock().expect("recording store lock") = bytes;
            Ok(StoreObjectInfo {
                key: key.to_string(),
                size,
                digest: None,
                modified_at: None,
            })
        }

        async fn list(&self) -> Result<Vec<String>, ServerError> {
            Ok(Vec::new())
        }

        async fn delete(&self, _key: &str) -> Result<(), ServerError> {
            Ok(())
        }
    }

    fn handle(max_object_bytes: Option<i64>) -> StoreResourceHandle<RecordingStore> {
        StoreResourceHandle::new(
            "test-service",
            "objects",
            StoreResourceBinding {
                name: "test_store".to_string(),
                max_object_bytes,
                max_total_bytes: None,
                ttl_ms: 0,
            },
            RecordingStore::default(),
        )
    }

    #[tokio::test]
    async fn store_write_from_rejects_known_oversize_before_backend_io() {
        let handle = handle(Some(4));
        let writes = Arc::clone(&handle.client.writes);
        let mut reader = Cursor::new(b"12345".to_vec());

        let error = handle
            .write_from("key", &mut reader, Some(5))
            .await
            .expect_err("known oversize must fail");

        assert!(matches!(
            error,
            ServerError::StoreObjectTooLarge {
                attempted_bytes: 5,
                max_bytes: 4
            }
        ));
        assert_eq!(writes.load(Ordering::SeqCst), 0);
        assert_eq!(reader.position(), 0);
    }

    #[tokio::test]
    async fn store_write_from_rejects_unknown_oversize_without_clean_eof() {
        let handle = handle(Some(4));
        let error = handle
            .write_from("key", Cursor::new(b"12345".to_vec()), None)
            .await
            .expect_err("unknown oversize must fail");

        assert!(matches!(
            error,
            ServerError::StoreObjectTooLarge {
                attempted_bytes: 5,
                max_bytes: 4
            }
        ));
        assert!(handle
            .client
            .bytes
            .lock()
            .expect("recording store lock")
            .is_empty());
    }

    #[tokio::test]
    async fn store_write_from_requires_exact_expected_size() {
        for (bytes, actual) in [(b"123".as_slice(), 3), (b"12345".as_slice(), 5)] {
            let handle = handle(None);
            let error = handle
                .write_from("key", Cursor::new(bytes.to_vec()), Some(4))
                .await
                .expect_err("size mismatch must fail");
            assert!(matches!(
                error,
                ServerError::StoreObjectSizeMismatch {
                    expected_bytes: 4,
                    actual_bytes
                } if actual_bytes == actual
            ));
        }
    }

    #[tokio::test]
    async fn store_trait_forwarding_keeps_bound_maximum() {
        let handle = handle(Some(4));
        let mut reader = Cursor::new(b"12345".to_vec());
        let error = StoreResourceClient::write_from(&handle, "key", &mut reader)
            .await
            .expect_err("trait forwarding must enforce binding");
        assert!(matches!(error, ServerError::StoreObjectTooLarge { .. }));
    }

    #[tokio::test]
    async fn store_streaming_and_whole_buffer_paths_round_trip() {
        let handle = handle(Some(1024));
        let info = handle
            .write_from("key", Cursor::new(b"streamed".to_vec()), Some(8))
            .await
            .expect("stream upload");
        assert_eq!(info.size, 8);

        let mut writer = Cursor::new(Vec::new());
        let read_info = handle
            .read_into("key", &mut writer)
            .await
            .expect("stream download")
            .expect("object exists");
        assert_eq!(read_info.size, 8);
        assert_eq!(writer.into_inner(), b"streamed");

        handle
            .write("key", Bytes::from_static(b"buffered"))
            .await
            .unwrap();
        assert_eq!(
            handle.read("key").await.unwrap().unwrap(),
            b"buffered".as_slice()
        );
    }

    #[tokio::test]
    async fn store_write_from_cancellation_aborts_before_eof() {
        let handle = handle(None);
        let (mut writer, reader) = tokio::io::duplex(8);
        writer.write_all(b"prefix").await.unwrap();
        let error = handle
            .write_from_with_cancel("key", reader, None, async {})
            .await
            .expect_err("cancellation must abort upload");
        assert!(matches!(error, ServerError::StoreWriteCancelled));
    }
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
