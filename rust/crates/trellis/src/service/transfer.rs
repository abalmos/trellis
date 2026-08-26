use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};
use std::task::{Context, Poll};
use std::time::Duration;

use async_nats::header::HeaderMap;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use bytes::Bytes;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt, ReadBuf};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use super::request_loop::encode_error_reply;
use super::{
    OperationTransferProgress, RequestContext, RequestValidator, ServerError,
    ServiceResourceBindings, StoreResourceBinding, StoreResourceClient,
};

const UPLOAD_SUBJECT_PREFIX: &str = "transfer.v2.upload";
const DOWNLOAD_SUBJECT_PREFIX: &str = "transfer.v2.download";
/// Header carrying the zero-based transfer chunk sequence number.
pub const TRANSFER_SEQUENCE_HEADER: &str = "trellis-transfer-seq";
/// Header marking the final transfer frame.
pub const TRANSFER_EOF_HEADER: &str = "trellis-transfer-eof";
/// Header distinguishing signed transfer control payloads from arbitrary data.
pub const TRANSFER_CONTROL_HEADER: &str = "trellis-transfer-control";
/// Largest transfer frame accepted by Trellis runtimes.
pub const MAX_TRANSFER_CHUNK_BYTES: u64 = 1024 * 1024;
const TRANSFER_FRAME_PROOF_DOMAIN: &[u8] = b"trellis.transfer.v2.frame\0";

pub(crate) fn transfer_frame_proof_payload(
    seq: u64,
    control: Option<&str>,
    payload: &[u8],
) -> Bytes {
    let control = control.unwrap_or_default().as_bytes();
    let mut framed = Vec::with_capacity(
        TRANSFER_FRAME_PROOF_DOMAIN.len() + 8 + 4 + control.len() + payload.len(),
    );
    framed.extend_from_slice(TRANSFER_FRAME_PROOF_DOMAIN);
    framed.extend_from_slice(&seq.to_be_bytes());
    framed.extend_from_slice(&(control.len() as u32).to_be_bytes());
    framed.extend_from_slice(control);
    framed.extend_from_slice(payload);
    Bytes::from(framed)
}

/// File metadata carried by receive transfer grants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(FileTransferInfo), "`.")]
pub struct FileTransferInfo {
    /// Object key within the bound store.
    #[doc = concat!("The `", stringify!(key), "` value.")]
    pub key: String,
    /// Object size in bytes.
    #[doc = concat!("The `", stringify!(size), "` value.")]
    pub size: u64,
    /// Last update timestamp encoded as an ISO-8601 string.
    #[doc = concat!("The `", stringify!(updated_at), "` value.")]
    pub updated_at: String,
    /// SHA-256 object digest supplied by the store.
    #[doc = concat!("The `", stringify!(digest), "` value.")]
    pub digest: String,
    /// Optional object content type.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(content_type), "` value.")]
    pub content_type: Option<String>,
    /// Store metadata associated with the object.
    #[doc = concat!("The `", stringify!(metadata), "` value.")]
    pub metadata: BTreeMap<String, String>,
}

/// Inputs for planning a service-owned upload transfer grant.
#[derive(Debug)]
pub struct TransferUploadGrantArgs<'a> {
    /// Service name exposed in the transfer grant.
    #[doc = concat!("The `", stringify!(service_name), "` value.")]
    pub service_name: &'a str,
    /// Caller session key that owns this transfer grant.
    #[doc = concat!("The `", stringify!(session_key), "` value.")]
    pub session_key: &'a str,
    /// Service session key used to scope the NATS transfer subject.
    #[doc = concat!("The `", stringify!(service_session_key), "` value.")]
    pub service_session_key: &'a str,
    /// Resolved service resource bindings from bootstrap.
    #[doc = concat!("The `", stringify!(resources), "` value.")]
    pub resources: &'a ServiceResourceBindings,
    /// Contract-local store alias used by the transfer declaration.
    #[doc = concat!("The `", stringify!(store), "` value.")]
    pub store: &'a str,
    /// Object key that will receive uploaded bytes.
    #[doc = concat!("The `", stringify!(key), "` value.")]
    pub key: &'a str,
    /// Preallocated transfer id supplied by the caller.
    #[doc = concat!("The `", stringify!(transfer_id), "` value.")]
    pub transfer_id: &'a str,
    /// Grant expiration timestamp encoded as an ISO-8601 string.
    #[doc = concat!("The `", stringify!(expires_at), "` value.")]
    pub expires_at: &'a str,
    /// Maximum transfer frame size advertised to clients.
    #[doc = concat!("The `", stringify!(chunk_bytes), "` value.")]
    pub chunk_bytes: u64,
    /// Optional operation-level upload size cap.
    #[doc = concat!("The `", stringify!(max_bytes), "` value.")]
    pub max_bytes: Option<u64>,
    /// Optional content type for the stored object.
    #[doc = concat!("The `", stringify!(content_type), "` value.")]
    pub content_type: Option<&'a str>,
    /// Optional object metadata to store with uploaded bytes.
    #[doc = concat!("The `", stringify!(metadata), "` value.")]
    pub metadata: BTreeMap<String, String>,
}

/// Inputs for planning a service-owned download transfer grant.
#[derive(Debug)]
pub struct TransferDownloadGrantArgs<'a> {
    /// Service name exposed in the transfer grant.
    #[doc = concat!("The `", stringify!(service_name), "` value.")]
    pub service_name: &'a str,
    /// Caller session key that owns this transfer grant.
    #[doc = concat!("The `", stringify!(session_key), "` value.")]
    pub session_key: &'a str,
    /// Service session key used to scope the NATS transfer subject.
    #[doc = concat!("The `", stringify!(service_session_key), "` value.")]
    pub service_session_key: &'a str,
    /// Resolved service resource bindings from bootstrap.
    #[doc = concat!("The `", stringify!(resources), "` value.")]
    pub resources: &'a ServiceResourceBindings,
    /// Contract-local store alias used by the transfer declaration.
    #[doc = concat!("The `", stringify!(store), "` value.")]
    pub store: &'a str,
    /// Preallocated transfer id supplied by the caller.
    #[doc = concat!("The `", stringify!(transfer_id), "` value.")]
    pub transfer_id: &'a str,
    /// Grant expiration timestamp encoded as an ISO-8601 string.
    #[doc = concat!("The `", stringify!(expires_at), "` value.")]
    pub expires_at: &'a str,
    /// Maximum transfer frame size advertised to clients.
    #[doc = concat!("The `", stringify!(chunk_bytes), "` value.")]
    pub chunk_bytes: u64,
    /// Object metadata for the file that will be streamed later.
    #[doc = concat!("The `", stringify!(info), "` value.")]
    pub info: FileTransferInfo,
}

/// Public wire DTO for an upload transfer grant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(UploadTransferGrant), "`.")]
pub struct UploadTransferGrant {
    #[serde(rename = "type")]
    /// Discriminator matching Trellis transfer grant wire DTOs.
    #[doc = concat!("The `", stringify!(type_name), "` value.")]
    pub type_name: String,
    /// Transfer direction, always `send` for upload grants.
    #[doc = concat!("The `", stringify!(direction), "` value.")]
    pub direction: String,
    /// Service name exposed in the transfer grant.
    #[doc = concat!("The `", stringify!(service), "` value.")]
    pub service: String,
    /// Caller session key that owns this transfer grant.
    #[doc = concat!("The `", stringify!(session_key), "` value.")]
    pub session_key: String,
    /// Unique transfer id for the planned session.
    #[doc = concat!("The `", stringify!(transfer_id), "` value.")]
    pub transfer_id: String,
    /// NATS subject that the follow-up upload session should bind.
    #[doc = concat!("The `", stringify!(subject), "` value.")]
    pub subject: String,
    /// Grant expiration timestamp encoded as an ISO-8601 string.
    #[doc = concat!("The `", stringify!(expires_at), "` value.")]
    pub expires_at: String,
    /// Maximum transfer frame size advertised to clients.
    #[serde(deserialize_with = "deserialize_chunk_bytes")]
    #[doc = concat!("The `", stringify!(chunk_bytes), "` value.")]
    pub chunk_bytes: u64,
    /// Effective upload cap after applying the bound store limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(max_bytes), "` value.")]
    pub max_bytes: Option<u64>,
    /// Optional content type for the stored object.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(content_type), "` value.")]
    pub content_type: Option<String>,
    /// Optional object metadata to store with uploaded bytes.
    #[doc = concat!("The `", stringify!(metadata), "` value.")]
    pub metadata: BTreeMap<String, String>,
}

/// Planned upload transfer grant plus binding metadata needed by follow-up streaming.
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(UploadTransferGrantPlan), "`.")]
pub struct UploadTransferGrantPlan {
    /// Public transfer grant that is safe to serialize and return to callers.
    #[doc = concat!("The `", stringify!(grant), "` value.")]
    pub grant: UploadTransferGrant,
    /// Contract-local store alias selected by the transfer declaration.
    #[doc = concat!("The `", stringify!(store_alias), "` value.")]
    pub store_alias: String,
    /// Concrete object-store bucket name resolved from bindings.
    #[doc = concat!("The `", stringify!(store), "` value.")]
    pub store: String,
    /// Object key that will receive uploaded bytes.
    #[doc = concat!("The `", stringify!(key), "` value.")]
    pub key: String,
}

/// Public wire DTO for a download transfer grant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(DownloadTransferGrant), "`.")]
pub struct DownloadTransferGrant {
    #[serde(rename = "type")]
    /// Discriminator matching Trellis transfer grant wire DTOs.
    #[doc = concat!("The `", stringify!(type_name), "` value.")]
    pub type_name: String,
    /// Transfer direction, always `receive` for download grants.
    #[doc = concat!("The `", stringify!(direction), "` value.")]
    pub direction: String,
    /// Service name exposed in the transfer grant.
    #[doc = concat!("The `", stringify!(service), "` value.")]
    pub service: String,
    /// Caller session key that owns this transfer grant.
    #[doc = concat!("The `", stringify!(session_key), "` value.")]
    pub session_key: String,
    /// Unique transfer id for the planned session.
    #[doc = concat!("The `", stringify!(transfer_id), "` value.")]
    pub transfer_id: String,
    /// NATS subject that the follow-up download session should bind.
    #[doc = concat!("The `", stringify!(subject), "` value.")]
    pub subject: String,
    /// Grant expiration timestamp encoded as an ISO-8601 string.
    #[doc = concat!("The `", stringify!(expires_at), "` value.")]
    pub expires_at: String,
    /// Maximum transfer frame size advertised to clients.
    #[serde(deserialize_with = "deserialize_chunk_bytes")]
    #[doc = concat!("The `", stringify!(chunk_bytes), "` value.")]
    pub chunk_bytes: u64,
    /// Object metadata for the file that will be streamed later.
    #[doc = concat!("The `", stringify!(info), "` value.")]
    pub info: FileTransferInfo,
}

/// Planned download transfer grant plus binding metadata needed by follow-up streaming.
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DownloadTransferGrantPlan), "`.")]
pub struct DownloadTransferGrantPlan {
    /// Public transfer grant that is safe to serialize and return to callers.
    #[doc = concat!("The `", stringify!(grant), "` value.")]
    pub grant: DownloadTransferGrant,
    /// Contract-local store alias selected by the transfer declaration.
    #[doc = concat!("The `", stringify!(store_alias), "` value.")]
    pub store_alias: String,
    /// Concrete object-store bucket name resolved from bindings.
    #[doc = concat!("The `", stringify!(store), "` value.")]
    pub store: String,
    /// Effective object size limit from the store binding, when configured.
    #[doc = concat!("The `", stringify!(max_object_bytes), "` value.")]
    pub max_object_bytes: Option<u64>,
}

/// One upload frame decoded from the transfer chunk protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(UploadTransferChunk), "`.")]
pub struct UploadTransferChunk {
    /// Zero-based chunk sequence number from `trellis-transfer-seq`.
    #[doc = concat!("The `", stringify!(seq), "` value.")]
    pub seq: u64,
    /// Raw chunk payload bytes.
    #[doc = concat!("The `", stringify!(payload), "` value.")]
    pub payload: Bytes,
    /// Whether this chunk carries `trellis-transfer-eof: true`.
    #[doc = concat!("The `", stringify!(eof), "` value.")]
    pub eof: bool,
    /// Whether this frame carries an authenticated cancellation control.
    #[doc = concat!("The `", stringify!(cancel), "` value.")]
    pub cancel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "lowercase")]
enum UploadTransferControl {
    Complete { size: u64, digest: String },
    Cancel,
}

/// Service reply payload for an upload chunk request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "lowercase")]
#[doc = concat!("Public Trellis value set `", stringify!(UploadTransferAck), "`.")]
pub enum UploadTransferAck {
    /// More chunks are expected.
    Continue,
    /// EOF was accepted and bytes were stored.
    Complete {
        /// Metadata for the stored object.
        info: FileTransferInfo,
    },
    /// Cancellation was authenticated and the endpoint terminated.
    Cancelled,
}

/// Awaitable provider-side notification that an upload transfer reached durable storage.
#[derive(Debug)]
#[doc = concat!("Public Trellis data type `", stringify!(UploadTransferCompletion), "`.")]
pub struct UploadTransferCompletion {
    receiver: oneshot::Receiver<Result<FileTransferInfo, ServerError>>,
}

impl UploadTransferCompletion {
    /// Wait until the upload endpoint has durably stored the object or observed a transfer error.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(completed), "`.")]
    pub async fn completed(self) -> Result<FileTransferInfo, ServerError> {
        self.receiver.await.map_err(|_| {
            ServerError::Nats("upload transfer completion channel closed".to_string())
        })?
    }
}

/// Store-backed upload transfer executor for a single planned grant.
#[doc = concat!("Public Trellis data type `", stringify!(UploadTransferSession), "`.")]
pub struct UploadTransferSession {
    plan: UploadTransferGrantPlan,
    next_seq: u64,
    transferred_bytes: u64,
    hasher: Sha256,
    pipe: Option<tokio::io::DuplexStream>,
    upload_state: Arc<AtomicU8>,
    upload_task: Option<JoinHandle<Result<super::StoreObjectInfo, ServerError>>>,
    complete: bool,
    updated_at: String,
}

impl std::fmt::Debug for UploadTransferSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UploadTransferSession")
            .field("plan", &self.plan)
            .field("next_seq", &self.next_seq)
            .field("transferred_bytes", &self.transferred_bytes)
            .field("complete", &self.complete)
            .field("updated_at", &self.updated_at)
            .finish_non_exhaustive()
    }
}

const UPLOAD_OPEN: u8 = 0;
const UPLOAD_COMMIT: u8 = 1;
const UPLOAD_ABORT: u8 = 2;

struct UploadPipeReader {
    reader: tokio::io::DuplexStream,
    state: Arc<AtomicU8>,
}

impl AsyncRead for UploadPipeReader {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        let before = buf.filled().len();
        match Pin::new(&mut this.reader).poll_read(cx, buf) {
            Poll::Ready(Ok(()))
                if buf.filled().len() == before
                    && this.state.load(Ordering::Acquire) != UPLOAD_COMMIT =>
            {
                Poll::Ready(Err(std::io::Error::other("upload transfer aborted")))
            }
            result => result,
        }
    }
}

impl UploadTransferSession {
    /// Create an upload transfer session with the timestamp to report on completion.
    #[doc = concat!("Trellis API operation `", stringify!(new), "`.")]
    pub fn new(plan: UploadTransferGrantPlan, updated_at: impl Into<String>) -> Self {
        Self {
            plan,
            next_seq: 0,
            transferred_bytes: 0,
            hasher: Sha256::new(),
            pipe: None,
            upload_state: Arc::new(AtomicU8::new(UPLOAD_OPEN)),
            upload_task: None,
            complete: false,
            updated_at: updated_at.into(),
        }
    }

    /// NATS subject that this planned upload session accepts chunks on.
    #[doc = concat!("Trellis API operation `", stringify!(subject), "`.")]
    pub fn subject(&self) -> &str {
        &self.plan.grant.subject
    }

    /// Caller session key that owns this planned upload session.
    #[doc = concat!("Trellis API operation `", stringify!(session_key), "`.")]
    pub fn session_key(&self) -> &str {
        &self.plan.grant.session_key
    }

    /// Build the operation transfer progress snapshot that would result from this chunk.
    #[doc = concat!("Trellis API operation `", stringify!(progress_for_chunk), "`.")]
    pub fn progress_for_chunk(&self, chunk: &UploadTransferChunk) -> OperationTransferProgress {
        OperationTransferProgress {
            chunk_index: chunk.seq,
            chunk_bytes: chunk.payload.len() as u64,
            transferred_bytes: self
                .transferred_bytes
                .saturating_add(chunk.payload.len() as u64),
        }
    }

    async fn start<C>(&mut self, store: C) -> Result<(), ServerError>
    where
        C: StoreResourceClient,
    {
        let capacity = usize::try_from(self.plan.grant.chunk_bytes)
            .unwrap_or(usize::MAX)
            .max(1);
        let (writer, reader) = tokio::io::duplex(capacity);
        let mut reader = UploadPipeReader {
            reader,
            state: Arc::clone(&self.upload_state),
        };
        let key = self.plan.key.clone();
        self.pipe = Some(writer);
        self.upload_task = Some(tokio::spawn(async move {
            store.write_from(&key, &mut reader).await
        }));
        Ok(())
    }

    /// Accept one ordered upload chunk using an explicit timestamp for expiry checks.
    async fn receive_at(
        &mut self,
        chunk: UploadTransferChunk,
        now_iso: &str,
    ) -> Result<UploadTransferAck, ServerError> {
        if self.complete {
            return Err(ServerError::TransferAlreadyComplete {
                transfer_id: self.plan.grant.transfer_id.clone(),
            });
        }
        enforce_transfer_not_expired(
            &self.plan.grant.transfer_id,
            &self.plan.grant.expires_at,
            now_iso,
        )?;

        if chunk.seq != self.next_seq {
            return Err(ServerError::TransferSequenceOutOfOrder {
                transfer_id: self.plan.grant.transfer_id.clone(),
                expected_seq: self.next_seq,
                actual_seq: chunk.seq,
            });
        }

        let chunk_limit = self.plan.grant.chunk_bytes;
        if !chunk.eof && chunk.payload.len() as u64 > chunk_limit {
            return Err(ServerError::TransferObjectTooLarge {
                service_name: self.plan.grant.service.clone(),
                store: self.plan.store_alias.clone(),
                key: self.plan.key.clone(),
                size: chunk.payload.len() as u64,
                max_bytes: chunk_limit,
            });
        }

        if !chunk.eof {
            let next_size = self
                .transferred_bytes
                .checked_add(chunk.payload.len() as u64)
                .ok_or_else(|| ServerError::Nats("upload transfer size overflow".to_string()))?;
            enforce_upload_max_bytes(&self.plan, next_size)?;
            self.pipe
                .as_mut()
                .ok_or_else(|| ServerError::Nats("upload transfer pipe is not open".to_string()))?
                .write_all(&chunk.payload)
                .await
                .map_err(|error| ServerError::Nats(error.to_string()))?;
            self.hasher.update(&chunk.payload);
            self.transferred_bytes = next_size;
            self.next_seq = self.next_seq.checked_add(1).ok_or_else(|| {
                ServerError::Nats("upload transfer sequence overflow".to_string())
            })?;
            return Ok(UploadTransferAck::Continue);
        }

        let control: UploadTransferControl = serde_json::from_slice(&chunk.payload)?;
        let UploadTransferControl::Complete { size, digest } = control else {
            self.abort().await;
            return Err(ServerError::TransferCancelled {
                transfer_id: self.plan.grant.transfer_id.clone(),
            });
        };
        if size != self.transferred_bytes {
            self.abort().await;
            return Err(ServerError::TransferObjectSizeMismatch {
                store: self.plan.store_alias.clone(),
                key: self.plan.key.clone(),
                expected_size: size,
                actual_size: self.transferred_bytes,
            });
        }
        let actual_digest = format!(
            "SHA-256={}",
            URL_SAFE_NO_PAD.encode(self.hasher.clone().finalize())
        );
        if digest != actual_digest {
            self.abort().await;
            return Err(ServerError::TransferDigestMismatch {
                transfer_id: self.plan.grant.transfer_id.clone(),
                expected_digest: digest,
                actual_digest,
            });
        }

        self.upload_state.store(UPLOAD_COMMIT, Ordering::Release);
        self.pipe.take();
        let stored = self
            .upload_task
            .take()
            .ok_or_else(|| ServerError::Nats("upload transfer task is not running".to_string()))?
            .await
            .map_err(|error| {
                ServerError::Nats(format!("upload transfer task failed: {error}"))
            })??;
        if stored.key != self.plan.key {
            return Err(ServerError::Nats(format!(
                "upload stored key mismatch: expected {}, got {}",
                self.plan.key, stored.key
            )));
        }
        if stored.size != self.transferred_bytes {
            return Err(ServerError::TransferObjectSizeMismatch {
                store: self.plan.store_alias.clone(),
                key: self.plan.key.clone(),
                expected_size: self.transferred_bytes,
                actual_size: stored.size,
            });
        }
        let stored_digest = stored
            .digest
            .ok_or_else(|| ServerError::TransferDigestMismatch {
                transfer_id: self.plan.grant.transfer_id.clone(),
                expected_digest: actual_digest.clone(),
                actual_digest: "missing backend digest".to_string(),
            })?;
        if !transfer_digests_match(&stored_digest, &actual_digest) {
            return Err(ServerError::TransferDigestMismatch {
                transfer_id: self.plan.grant.transfer_id.clone(),
                expected_digest: actual_digest,
                actual_digest: stored_digest,
            });
        }

        let info = FileTransferInfo {
            key: self.plan.key.clone(),
            size: self.transferred_bytes,
            updated_at: self.updated_at.clone(),
            digest: actual_digest,
            content_type: self.plan.grant.content_type.clone(),
            metadata: self.plan.grant.metadata.clone(),
        };
        self.next_seq = self
            .next_seq
            .checked_add(1)
            .ok_or_else(|| ServerError::Nats("upload transfer sequence overflow".to_string()))?;
        self.complete = true;
        Ok(UploadTransferAck::Complete { info })
    }

    async fn abort(&mut self) {
        self.upload_state.store(UPLOAD_ABORT, Ordering::Release);
        self.pipe.take();
        abort_store_task(&mut self.upload_task).await;
    }

    /// Fail if the session has not received an EOF completion frame.
    #[doc = concat!("Trellis API operation `", stringify!(ensure_complete), "`.")]
    pub fn ensure_complete(&self) -> Result<(), ServerError> {
        if self.complete {
            Ok(())
        } else {
            Err(ServerError::TransferMissingEof {
                transfer_id: self.plan.grant.transfer_id.clone(),
            })
        }
    }
}

/// Decode one upload transfer request frame from NATS headers and payload.
#[doc = concat!("Trellis API operation `", stringify!(decode_upload_transfer_chunk), "`.")]
pub fn decode_upload_transfer_chunk(
    headers: Option<&HeaderMap>,
    payload: Bytes,
) -> Result<UploadTransferChunk, ServerError> {
    let seq = parse_transfer_sequence(headers)?;
    if let Some(value) = optional_header(headers, TRANSFER_EOF_HEADER) {
        return Err(ServerError::InvalidTransferHeader {
            header: TRANSFER_EOF_HEADER,
            value: value.to_string(),
        });
    }
    let (eof, cancel) = match optional_header(headers, TRANSFER_CONTROL_HEADER) {
        None => (false, false),
        Some("complete") => (true, false),
        Some("cancel") => (false, true),
        Some(value) => {
            return Err(ServerError::InvalidTransferHeader {
                header: TRANSFER_CONTROL_HEADER,
                value: value.to_string(),
            })
        }
    };

    Ok(UploadTransferChunk {
        seq,
        payload,
        eof,
        cancel,
    })
}

/// Run a NATS upload transfer endpoint and report operation progress for accepted body chunks.
pub async fn run_upload_transfer_endpoint_with_progress<C, V, F>(
    client: async_nats::Client,
    subscriber: impl futures_util::Stream<Item = async_nats::Message>,
    session: UploadTransferSession,
    store: C,
    validator: V,
    on_progress: F,
) -> Result<(), ServerError>
where
    C: StoreResourceClient,
    V: RequestValidator + 'static,
    F: Fn(OperationTransferProgress) + Send + Sync + 'static,
{
    run_upload_transfer_endpoint_inner(
        client,
        subscriber,
        session,
        store,
        validator,
        on_progress,
        None,
    )
    .await
}

async fn run_upload_transfer_endpoint_inner<C, V, F>(
    client: async_nats::Client,
    subscriber: impl futures_util::Stream<Item = async_nats::Message>,
    mut session: UploadTransferSession,
    store: C,
    validator: V,
    on_progress: F,
    mut completion: Option<oneshot::Sender<Result<FileTransferInfo, ServerError>>>,
) -> Result<(), ServerError>
where
    C: StoreResourceClient,
    V: RequestValidator + 'static,
    F: Fn(OperationTransferProgress) + Send + Sync + 'static,
{
    let mut subscriber = Box::pin(subscriber);
    session.start(store).await?;
    let expiry = tokio::time::sleep(transfer_expiry_delay(&session.plan.grant.expires_at)?);
    tokio::pin!(expiry);
    loop {
        let message = tokio::select! {
            _ = &mut expiry => {
                session.abort().await;
                if let Some(sender) = completion.take() {
                    let _ = sender.send(Err(ServerError::TransferExpired {
                        transfer_id: session.plan.grant.transfer_id.clone(),
                        expires_at: session.plan.grant.expires_at.clone(),
                    }));
                }
                return Ok(());
            }
            maybe_message = subscriber.next() => {
                let Some(message) = maybe_message else {
                    break;
                };
                message
            }
        };
        let reply_to = message.reply.as_ref().map(ToString::to_string);
        let subject = session.subject().to_string();
        let session_key = session.session_key().to_string();
        let upload_state = Arc::clone(&session.upload_state);
        enum Outcome {
            Handled(Result<(UploadTransferAck, OperationTransferProgress), ServerError>),
            Cancelled(Option<String>),
            Expired,
            Closed,
        }
        let outcome = {
            let handling = handle_upload_transfer_message(&mut session, &validator, &message);
            tokio::pin!(handling);
            loop {
                tokio::select! {
                _ = &mut expiry, if upload_state.load(Ordering::Acquire) < UPLOAD_COMMIT => {
                    break Outcome::Expired;
                }
                result = &mut handling => break Outcome::Handled(result),
                pending = subscriber.next() => {
                    let Some(pending) = pending else {
                        break Outcome::Closed;
                    };
                    let pending_reply = pending.reply.as_ref().map(ToString::to_string);
                    match decode_authenticated_upload_transfer_message(
                        &subject,
                        &session_key,
                        &validator,
                        &pending,
                    ).await {
                        Ok(chunk) if chunk.cancel => {
                            if upload_state.load(Ordering::Acquire) >= UPLOAD_COMMIT {
                                if let Some(pending_reply) = pending_reply {
                                    publish_error_reply(
                                        &client,
                                        pending_reply,
                                        &ServerError::Nats(
                                            "upload transfer cannot be cancelled after validated EOF"
                                                .to_string(),
                                        ),
                                    ).await?;
                                }
                            } else {
                                break Outcome::Cancelled(pending_reply);
                            }
                        }
                        Ok(_) => {
                            if let Some(pending_reply) = pending_reply {
                                publish_error_reply(
                                    &client,
                                    pending_reply,
                                    &ServerError::Nats(
                                        "upload transfer already has a pending frame".to_string(),
                                    ),
                                ).await?;
                            }
                        }
                        Err(error) => {
                            if let Some(pending_reply) = pending_reply {
                                publish_error_reply(&client, pending_reply, &error).await?;
                            }
                        }
                    }
                }
                }
            }
        };
        match outcome {
            Outcome::Expired => {
                session.abort().await;
                if let Some(sender) = completion.take() {
                    let _ = sender.send(Err(ServerError::TransferExpired {
                        transfer_id: session.plan.grant.transfer_id.clone(),
                        expires_at: session.plan.grant.expires_at.clone(),
                    }));
                }
                return Ok(());
            }
            Outcome::Closed => break,
            Outcome::Cancelled(cancel_reply) => {
                session.abort().await;
                if let Some(sender) = completion.take() {
                    let _ = sender.send(Err(ServerError::TransferCancelled {
                        transfer_id: session.plan.grant.transfer_id.clone(),
                    }));
                }
                if let Some(cancel_reply) = cancel_reply {
                    client
                        .publish(
                            cancel_reply,
                            Bytes::from_static(b"{\"status\":\"cancelled\"}"),
                        )
                        .await
                        .map_err(|error| ServerError::Nats(error.to_string()))?;
                }
                return Ok(());
            }
            Outcome::Handled(result) => match result {
                Ok((ack, progress)) => {
                    match &ack {
                        UploadTransferAck::Continue if progress.chunk_bytes > 0 => {
                            on_progress(progress);
                        }
                        UploadTransferAck::Complete { info } => {
                            if let Some(sender) = completion.take() {
                                let _ = sender.send(Ok(info.clone()));
                            }
                        }
                        UploadTransferAck::Continue | UploadTransferAck::Cancelled => {}
                    }
                    if let Some(reply_to) = reply_to {
                        client
                            .publish(reply_to, Bytes::from(serde_json::to_vec(&ack)?))
                            .await
                            .map_err(|error| ServerError::Nats(error.to_string()))?;
                    }
                    if matches!(
                        ack,
                        UploadTransferAck::Complete { .. } | UploadTransferAck::Cancelled
                    ) {
                        return Ok(());
                    }
                }
                Err(error) => {
                    session.abort().await;
                    if let Some(sender) = completion.take() {
                        let _ = sender.send(Err(transfer_completion_error(&error)));
                    }
                    if let Some(reply_to) = reply_to {
                        if matches!(error, ServerError::TransferCancelled { .. }) {
                            client
                                .publish(
                                    reply_to,
                                    Bytes::from_static(b"{\"status\":\"cancelled\"}"),
                                )
                                .await
                                .map_err(|error| ServerError::Nats(error.to_string()))?;
                        } else {
                            publish_error_reply(&client, reply_to, &error).await?;
                        }
                    }
                    return Ok(());
                }
            },
        }
    }

    session.abort().await;
    if let Some(sender) = completion.take() {
        let _ = sender.send(Err(ServerError::TransferMissingEof {
            transfer_id: session.plan.grant.transfer_id.clone(),
        }));
    }

    Ok(())
}

fn transfer_completion_error(error: &ServerError) -> ServerError {
    match error {
        ServerError::TransferSessionMismatch {
            subject,
            actual_session_key,
        } => ServerError::TransferSessionMismatch {
            subject: subject.clone(),
            actual_session_key: actual_session_key.clone(),
        },
        ServerError::MissingSessionKey { subject } => ServerError::MissingSessionKey {
            subject: subject.clone(),
        },
        ServerError::MissingProof { subject } => ServerError::MissingProof {
            subject: subject.clone(),
        },
        ServerError::RequestDenied {
            subject,
            session_key,
        } => ServerError::RequestDenied {
            subject: subject.clone(),
            session_key: session_key.clone(),
        },
        ServerError::ReplyInboxMismatch {
            subject,
            session_key,
            reply_to,
            expected_prefix,
        } => ServerError::ReplyInboxMismatch {
            subject: subject.clone(),
            session_key: session_key.clone(),
            reply_to: reply_to.clone(),
            expected_prefix: expected_prefix.clone(),
        },
        ServerError::TransferObjectTooLarge {
            service_name,
            store,
            key,
            size,
            max_bytes,
        } => ServerError::TransferObjectTooLarge {
            service_name: service_name.clone(),
            store: store.clone(),
            key: key.clone(),
            size: *size,
            max_bytes: *max_bytes,
        },
        ServerError::TransferSequenceOutOfOrder {
            transfer_id,
            expected_seq,
            actual_seq,
        } => ServerError::TransferSequenceOutOfOrder {
            transfer_id: transfer_id.clone(),
            expected_seq: *expected_seq,
            actual_seq: *actual_seq,
        },
        ServerError::TransferMissingEof { transfer_id } => ServerError::TransferMissingEof {
            transfer_id: transfer_id.clone(),
        },
        ServerError::TransferAlreadyComplete { transfer_id } => {
            ServerError::TransferAlreadyComplete {
                transfer_id: transfer_id.clone(),
            }
        }
        ServerError::InvalidTransferId { value } => ServerError::InvalidTransferId {
            value: value.clone(),
        },
        ServerError::TransferExpired {
            transfer_id,
            expires_at,
        } => ServerError::TransferExpired {
            transfer_id: transfer_id.clone(),
            expires_at: expires_at.clone(),
        },
        ServerError::InvalidTransferExpiry {
            expires_at,
            details,
        } => ServerError::InvalidTransferExpiry {
            expires_at: expires_at.clone(),
            details: details.clone(),
        },
        ServerError::TransferObjectMissing { store, key } => ServerError::TransferObjectMissing {
            store: store.clone(),
            key: key.clone(),
        },
        ServerError::InvalidTransferChunkSize { chunk_bytes } => {
            ServerError::InvalidTransferChunkSize {
                chunk_bytes: *chunk_bytes,
            }
        }
        ServerError::MissingTransferHeader { header } => {
            ServerError::MissingTransferHeader { header }
        }
        ServerError::InvalidTransferHeader { header, value } => {
            ServerError::InvalidTransferHeader {
                header,
                value: value.clone(),
            }
        }
        ServerError::TransferObjectSizeMismatch {
            store,
            key,
            expected_size,
            actual_size,
        } => ServerError::TransferObjectSizeMismatch {
            store: store.clone(),
            key: key.clone(),
            expected_size: *expected_size,
            actual_size: *actual_size,
        },
        ServerError::TransferDigestMismatch {
            transfer_id,
            expected_digest,
            actual_digest,
        } => ServerError::TransferDigestMismatch {
            transfer_id: transfer_id.clone(),
            expected_digest: expected_digest.clone(),
            actual_digest: actual_digest.clone(),
        },
        ServerError::TransferCancelled { transfer_id } => ServerError::TransferCancelled {
            transfer_id: transfer_id.clone(),
        },
        ServerError::StoreCommitIndeterminate { key, message } => {
            ServerError::StoreCommitIndeterminate {
                key: key.clone(),
                message: message.clone(),
            }
        }
        ServerError::StoreCommittedCleanupFailed { key, message } => {
            ServerError::StoreCommittedCleanupFailed {
                key: key.clone(),
                message: message.clone(),
            }
        }
        _ => ServerError::Nats(error.to_string()),
    }
}

/// Run a NATS download transfer endpoint for a single planned grant until its subscriber closes.
pub async fn run_download_transfer_endpoint<C, V>(
    client: async_nats::Client,
    subscriber: impl futures_util::Stream<Item = async_nats::Message>,
    plan: DownloadTransferGrantPlan,
    store: C,
    validator: V,
) -> Result<(), ServerError>
where
    C: StoreResourceClient,
    V: RequestValidator + 'static,
{
    let mut subscriber = Box::pin(subscriber);
    let capacity = usize::try_from(plan.grant.chunk_bytes).map_err(|_| {
        ServerError::InvalidTransferChunkSize {
            chunk_bytes: plan.grant.chunk_bytes,
        }
    })?;
    validate_chunk_bytes(plan.grant.chunk_bytes)?;
    let mut store = Some(store);
    let mut pipe_reader = None;
    let mut store_task = None;
    let mut seq = 0_u64;
    let mut transferred = 0_u64;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; capacity];
    let expiry = tokio::time::sleep(transfer_expiry_delay(&plan.grant.expires_at)?);
    tokio::pin!(expiry);

    loop {
        let message = tokio::select! {
            _ = &mut expiry => {
                abort_store_task(&mut store_task).await;
                return Ok(());
            }
            message = subscriber.next() => {
                let Some(message) = message else {
                    abort_store_task(&mut store_task).await;
                    return Ok(());
                };
                message
            }
        };
        let Some(reply_to) = message.reply.as_ref().map(ToString::to_string) else {
            continue;
        };
        if let Err(error) = validate_download_transfer_message(&plan, &validator, &message).await {
            publish_error_reply(&client, reply_to, &error).await?;
            if matches!(error, ServerError::TransferExpired { .. }) {
                abort_store_task(&mut store_task).await;
                return Ok(());
            }
            continue;
        }
        let frame =
            decode_upload_transfer_chunk(message.headers.as_ref(), message.payload.clone())?;
        if frame.cancel {
            if !matches!(
                serde_json::from_slice(&message.payload),
                Ok(UploadTransferControl::Cancel)
            ) {
                publish_error_reply(
                    &client,
                    reply_to,
                    &ServerError::Nats("invalid download cancellation control".to_string()),
                )
                .await?;
                continue;
            }
            abort_store_task(&mut store_task).await;
            client
                .publish(reply_to, Bytes::from_static(b"{\"status\":\"cancelled\"}"))
                .await
                .map_err(|error| ServerError::Nats(error.to_string()))?;
            return Ok(());
        }
        if frame.seq != seq {
            publish_error_reply(
                &client,
                reply_to,
                &ServerError::TransferSequenceOutOfOrder {
                    transfer_id: plan.grant.transfer_id.clone(),
                    expected_seq: seq,
                    actual_seq: frame.seq,
                },
            )
            .await?;
            continue;
        }
        if frame.eof || !message.payload.is_empty() {
            publish_error_reply(
                &client,
                reply_to,
                &ServerError::Nats("invalid download transfer control".to_string()),
            )
            .await?;
            continue;
        }

        if pipe_reader.is_none() {
            let (mut pipe_writer, reader) = tokio::io::duplex(capacity);
            let key = plan.grant.info.key.clone();
            let store = store.take().ok_or_else(|| {
                ServerError::Nats("download transfer store already started".to_string())
            })?;
            pipe_reader = Some(reader);
            store_task = Some(tokio::spawn(async move {
                store.read_into(&key, &mut pipe_writer).await
            }));
        }

        let count = match loop {
            tokio::select! {
                _ = &mut expiry => {
                    abort_store_task(&mut store_task).await;
                    return Ok(());
                }
                result = pipe_reader.as_mut().expect("download pipe initialized").read(&mut buffer) => break result,
                control = subscriber.next() => {
                    let Some(control) = control else {
                        abort_store_task(&mut store_task).await;
                        return Ok(());
                    };
                    let Some(control_reply) = control.reply.as_ref().map(ToString::to_string) else {
                        continue;
                    };
                    if let Err(error) = validate_download_transfer_message(&plan, &validator, &control).await {
                        publish_error_reply(&client, control_reply, &error).await?;
                        continue;
                    }
                    let frame = decode_upload_transfer_chunk(
                        control.headers.as_ref(),
                        control.payload.clone(),
                    )?;
                    if frame.cancel && matches!(
                        serde_json::from_slice(&control.payload),
                        Ok(UploadTransferControl::Cancel)
                    ) {
                        abort_store_task(&mut store_task).await;
                        client
                            .publish(
                                control_reply,
                                Bytes::from_static(b"{\"status\":\"cancelled\"}"),
                            )
                            .await
                            .map_err(|error| ServerError::Nats(error.to_string()))?;
                        return Ok(());
                    }
                    publish_error_reply(
                        &client,
                        control_reply,
                        &ServerError::Nats("download transfer already has a pending pull".to_string()),
                    )
                    .await?;
                }
            }
        } {
            Ok(count) => count,
            Err(error) => {
                abort_store_task(&mut store_task).await;
                publish_error_reply(&client, reply_to, &ServerError::Nats(error.to_string()))
                    .await?;
                return Ok(());
            }
        };
        if count == 0 {
            let store_info = match store_task
                .take()
                .ok_or_else(|| ServerError::Nats("download transfer task missing".to_string()))?
                .await
            {
                Ok(Ok(Some(info))) => info,
                Ok(Ok(None)) => {
                    publish_error_reply(
                        &client,
                        reply_to,
                        &ServerError::TransferObjectMissing {
                            store: plan.store_alias.clone(),
                            key: plan.grant.info.key.clone(),
                        },
                    )
                    .await?;
                    return Ok(());
                }
                Ok(Err(error)) => {
                    publish_error_reply(&client, reply_to, &error).await?;
                    return Ok(());
                }
                Err(error) => {
                    publish_error_reply(&client, reply_to, &ServerError::Nats(error.to_string()))
                        .await?;
                    return Ok(());
                }
            };
            if store_info.key != plan.grant.info.key {
                publish_error_reply(
                    &client,
                    reply_to,
                    &ServerError::Nats(format!(
                        "download stored key mismatch: expected {}, got {}",
                        plan.grant.info.key, store_info.key
                    )),
                )
                .await?;
                return Ok(());
            }
            if transferred != plan.grant.info.size || store_info.size != transferred {
                publish_error_reply(
                    &client,
                    reply_to,
                    &ServerError::TransferObjectSizeMismatch {
                        store: plan.store_alias.clone(),
                        key: plan.grant.info.key.clone(),
                        expected_size: plan.grant.info.size,
                        actual_size: transferred,
                    },
                )
                .await?;
                return Ok(());
            }
            let digest = format!("SHA-256={}", URL_SAFE_NO_PAD.encode(hasher.finalize()));
            if !transfer_digests_match(&plan.grant.info.digest, &digest) {
                publish_error_reply(
                    &client,
                    reply_to,
                    &ServerError::TransferDigestMismatch {
                        transfer_id: plan.grant.transfer_id.clone(),
                        expected_digest: plan.grant.info.digest.clone(),
                        actual_digest: digest,
                    },
                )
                .await?;
                return Ok(());
            }
            let Some(expected) = store_info.digest.as_ref() else {
                publish_error_reply(
                    &client,
                    reply_to,
                    &ServerError::TransferDigestMismatch {
                        transfer_id: plan.grant.transfer_id.clone(),
                        expected_digest: plan.grant.info.digest.clone(),
                        actual_digest: "missing backend digest".to_string(),
                    },
                )
                .await?;
                return Ok(());
            };
            if !transfer_digests_match(expected, &digest) {
                publish_error_reply(
                    &client,
                    reply_to,
                    &ServerError::TransferDigestMismatch {
                        transfer_id: plan.grant.transfer_id.clone(),
                        expected_digest: expected.clone(),
                        actual_digest: digest,
                    },
                )
                .await?;
                return Ok(());
            }
            publish_download_chunk(&client, &reply_to, seq, Bytes::new(), true).await?;
            return Ok(());
        }

        let next = transferred
            .checked_add(count as u64)
            .ok_or_else(|| ServerError::Nats("download transfer size overflow".to_string()))?;
        if next > plan.grant.info.size || plan.max_object_bytes.is_some_and(|max| next > max) {
            abort_store_task(&mut store_task).await;
            publish_error_reply(
                &client,
                reply_to,
                &ServerError::TransferObjectTooLarge {
                    service_name: plan.grant.service.clone(),
                    store: plan.store_alias.clone(),
                    key: plan.grant.info.key.clone(),
                    size: next,
                    max_bytes: plan.max_object_bytes.unwrap_or(plan.grant.info.size),
                },
            )
            .await?;
            return Ok(());
        }
        publish_download_chunk(
            &client,
            &reply_to,
            seq,
            Bytes::copy_from_slice(&buffer[..count]),
            false,
        )
        .await?;
        hasher.update(&buffer[..count]);
        transferred = next;
        seq = seq
            .checked_add(1)
            .ok_or_else(|| ServerError::Nats("download transfer sequence overflow".to_string()))?;
    }
}

async fn abort_store_task<T>(task: &mut Option<JoinHandle<T>>) {
    if let Some(task) = task.take() {
        task.abort();
        let _ = task.await;
    }
}

/// Subscribe and run an upload transfer endpoint that reports operation progress.
pub async fn spawn_upload_transfer_endpoint_with_progress<C, V, F>(
    client: async_nats::Client,
    session: UploadTransferSession,
    store: C,
    validator: V,
    on_progress: F,
) -> Result<(), ServerError>
where
    C: StoreResourceClient,
    V: RequestValidator + 'static,
    F: Fn(OperationTransferProgress) + Send + Sync + 'static,
{
    let subject = session.subject().to_string();
    tracing::info!(subject = %subject, "subscribing upload transfer endpoint");
    let subscriber = client.subscribe(subject.clone()).await.map_err(|error| {
        ServerError::Nats(format!(
            "failed to subscribe to upload transfer subject '{subject}': {error}"
        ))
    })?;
    tokio::spawn(async move {
        tracing::debug!(subject = %subject, "upload transfer endpoint task started");
        if let Err(error) = run_upload_transfer_endpoint_with_progress(
            client,
            subscriber,
            session,
            store,
            validator,
            on_progress,
        )
        .await
        {
            tracing::error!(subject = %subject, error = %error, "upload transfer endpoint failed");
        }
        tracing::debug!(subject = %subject, "upload transfer endpoint task ended");
    });
    Ok(())
}

/// Subscribe and run an upload transfer endpoint that can be awaited until durable storage.
pub async fn spawn_upload_transfer_endpoint_with_completion<C, V>(
    client: async_nats::Client,
    session: UploadTransferSession,
    store: C,
    validator: V,
) -> Result<UploadTransferCompletion, ServerError>
where
    C: StoreResourceClient,
    V: RequestValidator + 'static,
{
    spawn_upload_transfer_endpoint_with_progress_and_completion(
        client,
        session,
        store,
        validator,
        |_| {},
    )
    .await
}

/// Subscribe and run an upload transfer endpoint with progress and durable completion reporting.
pub async fn spawn_upload_transfer_endpoint_with_progress_and_completion<C, V, F>(
    client: async_nats::Client,
    session: UploadTransferSession,
    store: C,
    validator: V,
    on_progress: F,
) -> Result<UploadTransferCompletion, ServerError>
where
    C: StoreResourceClient,
    V: RequestValidator + 'static,
    F: Fn(OperationTransferProgress) + Send + Sync + 'static,
{
    let subject = session.subject().to_string();
    tracing::info!(subject = %subject, "subscribing upload transfer endpoint");
    let subscriber = client.subscribe(subject.clone()).await.map_err(|error| {
        ServerError::Nats(format!(
            "failed to subscribe to upload transfer subject '{subject}': {error}"
        ))
    })?;
    let (sender, receiver) = oneshot::channel();
    tokio::spawn(async move {
        tracing::debug!(subject = %subject, "upload transfer endpoint task started");
        if let Err(error) = run_upload_transfer_endpoint_inner(
            client,
            subscriber,
            session,
            store,
            validator,
            on_progress,
            Some(sender),
        )
        .await
        {
            tracing::error!(subject = %subject, error = %error, "upload transfer endpoint failed");
        }
        tracing::debug!(subject = %subject, "upload transfer endpoint task ended");
    });
    Ok(UploadTransferCompletion { receiver })
}

/// Subscribe and run one planned download transfer endpoint in the background.
pub async fn spawn_download_transfer_endpoint<C, V>(
    client: async_nats::Client,
    plan: DownloadTransferGrantPlan,
    store: C,
    validator: V,
) -> Result<(), ServerError>
where
    C: StoreResourceClient,
    V: RequestValidator + 'static,
{
    let subject = plan.grant.subject.clone();
    tracing::info!(subject = %subject, "subscribing download transfer endpoint");
    let subscriber = client.subscribe(subject.clone()).await.map_err(|error| {
        ServerError::Nats(format!(
            "failed to subscribe to download transfer subject '{subject}': {error}"
        ))
    })?;
    tokio::spawn(async move {
        tracing::debug!(subject = %subject, "download transfer endpoint task started");
        if let Err(error) =
            run_download_transfer_endpoint(client, subscriber, plan, store, validator).await
        {
            tracing::error!(subject = %subject, error = %error, "download transfer endpoint failed");
        }
        tracing::debug!(subject = %subject, "download transfer endpoint task ended");
    });
    Ok(())
}

async fn handle_upload_transfer_message<V>(
    session: &mut UploadTransferSession,
    validator: &V,
    message: &async_nats::Message,
) -> Result<(UploadTransferAck, OperationTransferProgress), ServerError>
where
    V: RequestValidator,
{
    let chunk = decode_authenticated_upload_transfer_message(
        session.subject(),
        session.session_key(),
        validator,
        message,
    )
    .await?;
    if chunk.cancel {
        session.abort().await;
        return Err(ServerError::TransferCancelled {
            transfer_id: session.plan.grant.transfer_id.clone(),
        });
    }
    let progress = session.progress_for_chunk(&chunk);
    tracing::debug!(
        subject = %session.subject(),
        seq = chunk.seq,
        bytes = chunk.payload.len(),
        eof = chunk.eof,
        "received upload transfer chunk"
    );
    let now = current_time_iso()?;
    let ack = session.receive_at(chunk, &now).await?;
    Ok((ack, progress))
}

async fn decode_authenticated_upload_transfer_message<V>(
    subject: &str,
    session_key: &str,
    validator: &V,
    message: &async_nats::Message,
) -> Result<UploadTransferChunk, ServerError>
where
    V: RequestValidator,
{
    let context = transfer_request_context(message);
    let seq = parse_transfer_sequence(message.headers.as_ref())?;
    let control = optional_header(message.headers.as_ref(), TRANSFER_CONTROL_HEADER);
    let proof_payload = transfer_frame_proof_payload(seq, control, &message.payload);
    validate_transfer_request(subject, &proof_payload, &context, session_key, validator).await?;
    let chunk = decode_upload_transfer_chunk(message.headers.as_ref(), message.payload.clone())?;
    if chunk.cancel {
        if !matches!(
            serde_json::from_slice(&message.payload),
            Ok(UploadTransferControl::Cancel)
        ) {
            return Err(ServerError::Nats(
                "invalid upload cancellation control".to_string(),
            ));
        }
        return Ok(chunk);
    }
    Ok(chunk)
}

async fn validate_download_transfer_message<V>(
    plan: &DownloadTransferGrantPlan,
    validator: &V,
    message: &async_nats::Message,
) -> Result<(), ServerError>
where
    V: RequestValidator,
{
    let context = transfer_request_context(message);
    let seq = parse_transfer_sequence(message.headers.as_ref())?;
    let control = optional_header(message.headers.as_ref(), TRANSFER_CONTROL_HEADER);
    let proof_payload = transfer_frame_proof_payload(seq, control, &message.payload);
    validate_transfer_request(
        &plan.grant.subject,
        &proof_payload,
        &context,
        &plan.grant.session_key,
        validator,
    )
    .await?;
    let now = current_time_iso()?;
    enforce_transfer_not_expired(&plan.grant.transfer_id, &plan.grant.expires_at, &now)
}

fn transfer_request_context(message: &async_nats::Message) -> RequestContext {
    RequestContext {
        subject: message.subject.to_string(),
        session_key: optional_header(message.headers.as_ref(), "session-key")
            .map(ToString::to_string),
        proof: optional_header(message.headers.as_ref(), "proof").map(ToString::to_string),
        authorization_context: optional_header(message.headers.as_ref(), "authorization-context")
            .map(ToString::to_string),
        iat: optional_header(message.headers.as_ref(), "iat").and_then(|value| value.parse().ok()),
        request_id: optional_header(message.headers.as_ref(), "request-id")
            .map(ToString::to_string),
        required_capabilities: None,
        required_permission: None,
        reply_to: message.reply.as_ref().map(ToString::to_string),
        caller: None,
        traceparent: optional_header(message.headers.as_ref(), "traceparent")
            .map(ToString::to_string),
        tracestate: optional_header(message.headers.as_ref(), "tracestate")
            .map(ToString::to_string),
    }
}

async fn validate_transfer_request<V>(
    subject: &str,
    payload: &Bytes,
    context: &RequestContext,
    expected_session_key: &str,
    validator: &V,
) -> Result<(), ServerError>
where
    V: RequestValidator,
{
    let actual_session_key =
        context
            .session_key
            .clone()
            .ok_or_else(|| ServerError::MissingSessionKey {
                subject: subject.to_string(),
            })?;
    if context.proof.as_deref().is_none_or(str::is_empty) {
        return Err(ServerError::MissingProof {
            subject: subject.to_string(),
        });
    }
    if actual_session_key != expected_session_key {
        return Err(ServerError::TransferSessionMismatch {
            subject: subject.to_string(),
            actual_session_key,
        });
    }

    if validator
        .validate_possession(subject, payload, context)
        .await?
        .allowed
    {
        Ok(())
    } else {
        Err(ServerError::RequestDenied {
            subject: subject.to_string(),
            session_key: actual_session_key,
        })
    }
}

async fn publish_download_chunk(
    client: &async_nats::Client,
    reply_to: &str,
    seq: u64,
    payload: Bytes,
    eof: bool,
) -> Result<(), ServerError> {
    let mut headers = HeaderMap::new();
    headers.insert(TRANSFER_SEQUENCE_HEADER, seq.to_string().as_str());
    if eof {
        headers.insert(TRANSFER_EOF_HEADER, "true");
    }
    client
        .publish_with_headers(reply_to.to_string(), headers, payload)
        .await
        .map_err(|error| ServerError::Nats(error.to_string()))
}

async fn publish_error_reply(
    client: &async_nats::Client,
    reply_to: String,
    error: &ServerError,
) -> Result<(), ServerError> {
    let reply = encode_error_reply(reply_to, error);
    let mut headers = HeaderMap::new();
    headers.insert("status", "error");
    client
        .publish_with_headers(reply.reply_to, headers, reply.payload)
        .await
        .map_err(|error| ServerError::Nats(error.to_string()))
}

fn required_header<'a>(
    headers: Option<&'a HeaderMap>,
    header: &'static str,
) -> Result<&'a str, ServerError> {
    optional_header(headers, header).ok_or(ServerError::MissingTransferHeader { header })
}

fn parse_transfer_sequence(headers: Option<&HeaderMap>) -> Result<u64, ServerError> {
    let seq = required_header(headers, TRANSFER_SEQUENCE_HEADER)?;
    seq.parse::<u64>()
        .map_err(|_| ServerError::InvalidTransferHeader {
            header: TRANSFER_SEQUENCE_HEADER,
            value: seq.to_string(),
        })
}

fn optional_header<'a>(headers: Option<&'a HeaderMap>, header: &str) -> Option<&'a str> {
    headers
        .and_then(|headers| headers.get(header))
        .map(async_nats::header::HeaderValue::as_str)
}

/// Build upload transfer grant metadata from resolved service resource bindings.
#[doc = concat!("Trellis API operation `", stringify!(plan_upload_transfer_grant), "`.")]
pub fn plan_upload_transfer_grant(
    args: TransferUploadGrantArgs<'_>,
) -> Result<UploadTransferGrantPlan, ServerError> {
    validate_chunk_bytes(args.chunk_bytes)?;
    let store = store_binding(args.service_name, args.resources, args.store)?;
    let max_bytes = effective_upload_max_bytes(args.max_bytes, store.max_object_bytes);
    validate_transfer_id(args.transfer_id)?;

    Ok(UploadTransferGrantPlan {
        grant: UploadTransferGrant {
            type_name: "TransferGrant".to_string(),
            direction: "send".to_string(),
            service: args.service_name.to_string(),
            session_key: args.session_key.to_string(),
            transfer_id: args.transfer_id.to_string(),
            subject: transfer_subject(
                UPLOAD_SUBJECT_PREFIX,
                args.service_session_key,
                args.transfer_id,
            ),
            expires_at: args.expires_at.to_string(),
            chunk_bytes: args.chunk_bytes,
            max_bytes,
            content_type: args.content_type.map(ToString::to_string),
            metadata: args.metadata,
        },
        store_alias: args.store.to_string(),
        store: store.name.clone(),
        key: args.key.to_string(),
    })
}

/// Build download transfer grant metadata from resolved service resource bindings.
#[doc = concat!("Trellis API operation `", stringify!(plan_download_transfer_grant), "`.")]
pub fn plan_download_transfer_grant(
    args: TransferDownloadGrantArgs<'_>,
) -> Result<DownloadTransferGrantPlan, ServerError> {
    validate_chunk_bytes(args.chunk_bytes)?;
    let store = store_binding(args.service_name, args.resources, args.store)?;
    enforce_max_object_bytes(
        args.service_name,
        args.store,
        &args.info,
        store.max_object_bytes,
    )?;
    validate_transfer_id(args.transfer_id)?;

    Ok(DownloadTransferGrantPlan {
        grant: DownloadTransferGrant {
            type_name: "TransferGrant".to_string(),
            direction: "receive".to_string(),
            service: args.service_name.to_string(),
            session_key: args.session_key.to_string(),
            transfer_id: args.transfer_id.to_string(),
            subject: transfer_subject(
                DOWNLOAD_SUBJECT_PREFIX,
                args.service_session_key,
                args.transfer_id,
            ),
            expires_at: args.expires_at.to_string(),
            chunk_bytes: args.chunk_bytes,
            info: args.info,
        },
        store_alias: args.store.to_string(),
        store: store.name.clone(),
        max_object_bytes: store
            .max_object_bytes
            .and_then(|value| u64::try_from(value).ok()),
    })
}

fn store_binding<'a>(
    service_name: &str,
    resources: &'a ServiceResourceBindings,
    store: &str,
) -> Result<&'a StoreResourceBinding, ServerError> {
    resources
        .store
        .get(store)
        .ok_or_else(|| ServerError::MissingResourceBinding {
            service_name: service_name.to_string(),
            resource_kind: "store".to_string(),
            resource_name: store.to_string(),
        })
}

fn effective_upload_max_bytes(
    requested: Option<u64>,
    store_max_object_bytes: Option<i64>,
) -> Option<u64> {
    match (
        requested,
        store_max_object_bytes.and_then(|value| u64::try_from(value).ok()),
    ) {
        (Some(requested), Some(store_max)) => Some(requested.min(store_max)),
        (Some(requested), None) => Some(requested),
        (None, Some(store_max)) => Some(store_max),
        (None, None) => None,
    }
}

fn enforce_max_object_bytes(
    service_name: &str,
    store: &str,
    info: &FileTransferInfo,
    store_max_object_bytes: Option<i64>,
) -> Result<(), ServerError> {
    let Some(max_bytes) = store_max_object_bytes.and_then(|value| u64::try_from(value).ok()) else {
        return Ok(());
    };

    if info.size > max_bytes {
        return Err(ServerError::TransferObjectTooLarge {
            service_name: service_name.to_string(),
            store: store.to_string(),
            key: info.key.clone(),
            size: info.size,
            max_bytes,
        });
    }

    Ok(())
}

fn enforce_upload_max_bytes(plan: &UploadTransferGrantPlan, size: u64) -> Result<(), ServerError> {
    let Some(max_bytes) = plan.grant.max_bytes else {
        return Ok(());
    };

    if size > max_bytes {
        return Err(ServerError::TransferObjectTooLarge {
            service_name: plan.grant.service.clone(),
            store: plan.store_alias.clone(),
            key: plan.key.clone(),
            size,
            max_bytes,
        });
    }

    Ok(())
}

fn validate_chunk_bytes(chunk_bytes: u64) -> Result<(), ServerError> {
    if !(1..=MAX_TRANSFER_CHUNK_BYTES).contains(&chunk_bytes) {
        return Err(ServerError::InvalidTransferChunkSize { chunk_bytes });
    }
    Ok(())
}

fn deserialize_chunk_bytes<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let chunk_bytes = u64::deserialize(deserializer)?;
    if !(1..=MAX_TRANSFER_CHUNK_BYTES).contains(&chunk_bytes) {
        return Err(serde::de::Error::custom(format!(
            "transfer chunk size must be between 1 and {MAX_TRANSFER_CHUNK_BYTES} bytes"
        )));
    }
    Ok(chunk_bytes)
}

fn current_time_iso() -> Result<String, ServerError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| ServerError::InvalidTransferExpiry {
            expires_at: "now".to_string(),
            details: error.to_string(),
        })
}

fn enforce_transfer_not_expired(
    transfer_id: &str,
    expires_at: &str,
    now_iso: &str,
) -> Result<(), ServerError> {
    let expires_at_time = parse_transfer_time(expires_at)?;
    let now = parse_transfer_time(now_iso)?;
    if now >= expires_at_time {
        return Err(ServerError::TransferExpired {
            transfer_id: transfer_id.to_string(),
            expires_at: expires_at.to_string(),
        });
    }
    Ok(())
}

fn transfer_expiry_delay(expires_at: &str) -> Result<Duration, ServerError> {
    let expires_at_time = parse_transfer_time(expires_at)?;
    let remaining = expires_at_time - OffsetDateTime::now_utc();
    let millis = remaining.whole_milliseconds();
    if millis <= 0 {
        return Ok(Duration::ZERO);
    }
    Ok(Duration::from_millis(millis.min(u64::MAX as i128) as u64))
}

fn parse_transfer_time(value: &str) -> Result<OffsetDateTime, ServerError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|error| ServerError::InvalidTransferExpiry {
        expires_at: value.to_string(),
        details: error.to_string(),
    })
}

fn transfer_subject(prefix: &str, session_key: &str, transfer_id: &str) -> String {
    let session_prefix: String = session_key.chars().take(16).collect();
    format!("{prefix}.{session_prefix}.{transfer_id}")
}

fn validate_transfer_id(transfer_id: &str) -> Result<(), ServerError> {
    let invalid = transfer_id.is_empty()
        || transfer_id
            .chars()
            .any(|ch| matches!(ch, '.' | '*' | '>' | '/') || ch.is_whitespace() || ch.is_control());

    if invalid {
        return Err(ServerError::InvalidTransferId {
            value: transfer_id.to_string(),
        });
    }

    Ok(())
}

fn transfer_digests_match(left: &str, right: &str) -> bool {
    left.trim_end_matches('=') == right.trim_end_matches('=')
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };

    use futures_util::future::BoxFuture;
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

    use crate::service::{RequestValidation, StoreObjectInfo};

    use super::*;

    #[derive(Debug, Clone)]
    struct CountingValidator {
        calls: Arc<AtomicUsize>,
        allowed: bool,
    }

    #[derive(Debug, Clone, Default)]
    struct CommitOnEofStore {
        bytes: Arc<Mutex<Option<Vec<u8>>>>,
        reported_key: Option<String>,
        reported_size: Option<u64>,
        reported_digest: Option<String>,
    }

    impl StoreResourceClient for CommitOnEofStore {
        async fn read_into<W>(
            &self,
            _key: &str,
            _writer: &mut W,
        ) -> Result<Option<StoreObjectInfo>, ServerError>
        where
            W: AsyncWrite + Unpin + Send,
        {
            Ok(None)
        }

        async fn write_from<R>(
            &self,
            key: &str,
            reader: &mut R,
        ) -> Result<StoreObjectInfo, ServerError>
        where
            R: AsyncRead + Unpin + Send,
        {
            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .await
                .map_err(|error| ServerError::Nats(format!("test store upload failed: {error}")))?;
            let size = bytes.len() as u64;
            *self.bytes.lock().expect("test store lock") = Some(bytes);
            Ok(StoreObjectInfo {
                key: self.reported_key.clone().unwrap_or_else(|| key.to_string()),
                size: self.reported_size.unwrap_or(size),
                digest: self.reported_digest.clone().or_else(|| {
                    Some(format!(
                        "SHA-256={}",
                        URL_SAFE_NO_PAD.encode(Sha256::digest(
                            self.bytes
                                .lock()
                                .expect("test store lock")
                                .as_deref()
                                .unwrap_or_default()
                        ))
                    ))
                }),
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

    fn upload_plan(chunk_bytes: u64) -> UploadTransferGrantPlan {
        UploadTransferGrantPlan {
            grant: UploadTransferGrant {
                type_name: "TransferGrant".to_string(),
                direction: "send".to_string(),
                service: "test-service".to_string(),
                session_key: "session".to_string(),
                transfer_id: "transfer-1".to_string(),
                subject: "transfer.v2.upload.service.transfer-1".to_string(),
                expires_at: "2099-01-01T00:00:00Z".to_string(),
                chunk_bytes,
                max_bytes: Some(32),
                content_type: None,
                metadata: BTreeMap::new(),
            },
            store_alias: "objects".to_string(),
            store: "physical_store".to_string(),
            key: "object".to_string(),
        }
    }

    #[tokio::test]
    async fn upload_session_streams_and_commits_only_after_authenticated_completion() {
        let store = CommitOnEofStore::default();
        let mut session = UploadTransferSession::new(upload_plan(4), "2026-08-26T00:00:00Z");
        session.start(store.clone()).await.unwrap();
        session
            .receive_at(
                UploadTransferChunk {
                    seq: 0,
                    payload: Bytes::from_static(b"1234"),
                    eof: false,
                    cancel: false,
                },
                "2026-08-26T00:00:00Z",
            )
            .await
            .unwrap();
        assert!(store.bytes.lock().expect("test store lock").is_none());

        let digest = format!(
            "SHA-256={}",
            URL_SAFE_NO_PAD.encode(Sha256::digest(b"1234"))
        );
        let completion =
            serde_json::to_vec(&UploadTransferControl::Complete { size: 4, digest }).unwrap();
        let ack = session
            .receive_at(
                UploadTransferChunk {
                    seq: 1,
                    payload: Bytes::from(completion),
                    eof: true,
                    cancel: false,
                },
                "2026-08-26T00:00:00Z",
            )
            .await
            .unwrap();

        assert!(matches!(ack, UploadTransferAck::Complete { .. }));
        assert_eq!(
            store.bytes.lock().expect("test store lock").as_deref(),
            Some(b"1234".as_slice())
        );
    }

    #[tokio::test]
    async fn upload_session_abort_never_commits_a_partial_object() {
        let store = CommitOnEofStore::default();
        let mut session = UploadTransferSession::new(upload_plan(4), "2026-08-26T00:00:00Z");
        session.start(store.clone()).await.unwrap();
        session
            .receive_at(
                UploadTransferChunk {
                    seq: 0,
                    payload: Bytes::from_static(b"1234"),
                    eof: false,
                    cancel: false,
                },
                "2026-08-26T00:00:00Z",
            )
            .await
            .unwrap();
        session.abort().await;
        tokio::task::yield_now().await;
        assert!(store.bytes.lock().expect("test store lock").is_none());
    }

    #[test]
    fn cancellation_json_is_data_without_an_explicit_control_header() {
        let mut headers = HeaderMap::new();
        headers.insert(TRANSFER_SEQUENCE_HEADER, "0");
        let chunk = decode_upload_transfer_chunk(
            Some(&headers),
            Bytes::from_static(br#"{"action":"cancel"}"#),
        )
        .expect("decode arbitrary data frame");

        assert!(!chunk.cancel);
        assert!(!chunk.eof);
        assert_eq!(chunk.payload, Bytes::from_static(br#"{"action":"cancel"}"#));
    }

    #[test]
    fn authenticated_framing_matches_shared_transfer_v2_vectors() {
        let vectors: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../conformance/transfer-v2-vectors.json"
        ))
        .unwrap();
        for vector in vectors["vectors"].as_array().unwrap() {
            let payload = URL_SAFE_NO_PAD
                .decode(vector["payloadBase64url"].as_str().unwrap())
                .unwrap();
            let framed = transfer_frame_proof_payload(
                vector["seq"].as_u64().unwrap(),
                vector["control"].as_str(),
                &payload,
            );
            assert_eq!(
                URL_SAFE_NO_PAD.encode(framed),
                vector["framedBase64url"].as_str().unwrap()
            );
        }
    }

    #[tokio::test]
    async fn upload_rejects_backend_final_key_size_and_digest_mismatches() {
        let expected_digest = format!(
            "SHA-256={}",
            URL_SAFE_NO_PAD.encode(Sha256::digest(b"1234"))
        );
        for (store, expected_error) in [
            (
                CommitOnEofStore {
                    reported_size: Some(5),
                    ..Default::default()
                },
                "size",
            ),
            (
                CommitOnEofStore {
                    reported_digest: Some("SHA-256=wrong".to_string()),
                    ..Default::default()
                },
                "digest",
            ),
            (
                CommitOnEofStore {
                    reported_key: Some("wrong-key".to_string()),
                    ..Default::default()
                },
                "key",
            ),
        ] {
            let mut session = UploadTransferSession::new(upload_plan(4), "2026-08-26T00:00:00Z");
            session.start(store).await.unwrap();
            session
                .receive_at(
                    UploadTransferChunk {
                        seq: 0,
                        payload: Bytes::from_static(b"1234"),
                        eof: false,
                        cancel: false,
                    },
                    "2026-08-26T00:00:00Z",
                )
                .await
                .unwrap();
            let completion = serde_json::to_vec(&UploadTransferControl::Complete {
                size: 4,
                digest: expected_digest.clone(),
            })
            .unwrap();
            let error = session
                .receive_at(
                    UploadTransferChunk {
                        seq: 1,
                        payload: Bytes::from(completion),
                        eof: true,
                        cancel: false,
                    },
                    "2026-08-26T00:00:00Z",
                )
                .await
                .unwrap_err();
            assert!(match expected_error {
                "size" => matches!(&error, ServerError::TransferObjectSizeMismatch { .. }),
                "digest" => matches!(&error, ServerError::TransferDigestMismatch { .. }),
                "key" =>
                    matches!(&error, ServerError::Nats(message) if message.contains("key mismatch")),
                _ => unreachable!(),
            });
        }
    }

    impl RequestValidator for CountingValidator {
        fn validate<'a>(
            &'a self,
            _subject: &'a str,
            _payload: &'a Bytes,
            _context: &'a RequestContext,
        ) -> BoxFuture<'a, Result<RequestValidation, ServerError>> {
            Box::pin(async move {
                self.calls.fetch_add(1, Ordering::SeqCst);
                Ok(if self.allowed {
                    RequestValidation::allowed()
                } else {
                    RequestValidation::denied()
                })
            })
        }
    }

    #[tokio::test]
    async fn transfer_validation_rejects_session_mismatch_before_validator() {
        let calls = Arc::new(AtomicUsize::new(0));
        let validator = CountingValidator {
            calls: Arc::clone(&calls),
            allowed: true,
        };
        let context = RequestContext {
            subject: "transfer.v2.upload.session.transfer-1".to_string(),
            session_key: Some("wrong-session".to_string()),
            proof: Some("proof".to_string()),
            authorization_context: None,
            iat: None,
            request_id: None,
            required_capabilities: None,
            required_permission: None,
            reply_to: None,
            caller: None,
            traceparent: None,
            tracestate: None,
        };

        let error = validate_transfer_request(
            "transfer.v2.upload.session.transfer-1",
            &Bytes::new(),
            &context,
            "expected-session",
            &validator,
        )
        .await
        .expect_err("session mismatch");

        assert!(matches!(
            error,
            ServerError::TransferSessionMismatch { actual_session_key, .. }
                if actual_session_key == "wrong-session"
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn transfer_validation_requires_proof_before_session_mismatch() {
        let calls = Arc::new(AtomicUsize::new(0));
        let validator = CountingValidator {
            calls: Arc::clone(&calls),
            allowed: true,
        };
        let context = RequestContext {
            subject: "transfer.v2.upload.session.transfer-1".to_string(),
            session_key: Some("wrong-session".to_string()),
            proof: None,
            authorization_context: None,
            iat: None,
            request_id: None,
            required_capabilities: None,
            required_permission: None,
            reply_to: None,
            caller: None,
            traceparent: None,
            tracestate: None,
        };

        let error = validate_transfer_request(
            "transfer.v2.upload.session.transfer-1",
            &Bytes::new(),
            &context,
            "expected-session",
            &validator,
        )
        .await
        .expect_err("missing proof");

        assert!(matches!(error, ServerError::MissingProof { .. }));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn transfer_validation_maps_denied_validator_to_request_denied() {
        let calls = Arc::new(AtomicUsize::new(0));
        let validator = CountingValidator {
            calls: Arc::clone(&calls),
            allowed: false,
        };
        let context = RequestContext {
            subject: "transfer.v2.download.session.transfer-1".to_string(),
            session_key: Some("expected-session".to_string()),
            proof: Some("proof".to_string()),
            authorization_context: None,
            iat: None,
            request_id: None,
            required_capabilities: None,
            required_permission: None,
            reply_to: None,
            caller: None,
            traceparent: None,
            tracestate: None,
        };

        let error = validate_transfer_request(
            "transfer.v2.download.session.transfer-1",
            &Bytes::new(),
            &context,
            "expected-session",
            &validator,
        )
        .await
        .expect_err("denied");

        assert!(matches!(
            error,
            ServerError::RequestDenied { session_key, .. } if session_key == "expected-session"
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
