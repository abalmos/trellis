use std::collections::BTreeMap;
use std::time::Duration;

use async_nats::header::HeaderMap;
use bytes::Bytes;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use crate::client::connection::signed_headers;
use crate::client::{SessionAuth, TrellisClient, TrellisClientError};

const TRANSFER_SEQUENCE_HEADER: &str = "trellis-transfer-seq";
const TRANSFER_EOF_HEADER: &str = "trellis-transfer-eof";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub key: String,
    pub size: u64,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UploadTransferGrant {
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(rename = "direction", alias = "kind")]
    pub kind: String,
    pub service: String,
    pub session_key: String,
    pub transfer_id: String,
    pub subject: String,
    pub expires_at: String,
    pub chunk_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTransferGrant {
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(rename = "direction", alias = "kind")]
    pub kind: String,
    pub service: String,
    pub session_key: String,
    pub transfer_id: String,
    pub subject: String,
    pub expires_at: String,
    pub chunk_bytes: u64,
    pub info: FileInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "lowercase")]
enum UploadAck {
    Continue,
    Complete { info: FileInfo },
}

fn upload_headers(
    auth: &SessionAuth,
    context_digest: &str,
    subject: &str,
    reply: &str,
    payload: &[u8],
    seq: u64,
    eof: bool,
) -> Result<HeaderMap, TrellisClientError> {
    let mut headers = signed_headers(auth, context_digest, subject, reply, payload)?;
    headers.insert(TRANSFER_SEQUENCE_HEADER, seq.to_string().as_str());
    if eof {
        headers.insert(TRANSFER_EOF_HEADER, "true");
    }
    Ok(headers)
}

fn upload_chunk_size(chunk_bytes: u64) -> usize {
    (chunk_bytes as usize).max(1)
}

pub(crate) async fn put_upload_grant(
    client: &TrellisClient,
    grant: &UploadTransferGrant,
    body: impl AsRef<[u8]>,
) -> Result<FileInfo, TrellisClientError> {
    validate_grant(&grant.session_key, client)?;

    let bytes = body.as_ref();
    if let Some(max_bytes) = grant.max_bytes {
        let attempted_bytes = bytes.len() as u64;
        if attempted_bytes > max_bytes {
            return Err(TrellisClientError::TransferProtocol(format!(
                "upload exceeds max bytes: attempted {attempted_bytes}, max {max_bytes}"
            )));
        }
    }
    let max_chunk = upload_chunk_size(grant.chunk_bytes);
    let context_digest = client.authorization_context_digest()?;
    let mut seq: u64 = 0;

    for chunk in bytes.chunks(max_chunk) {
        let reply = client.nats().new_inbox();
        let headers = upload_headers(
            client.auth(),
            &context_digest,
            &grant.subject,
            &reply,
            chunk,
            seq,
            false,
        )?;
        let request = async_nats::Request::new()
            .inbox(reply)
            .headers(headers)
            .payload(Bytes::copy_from_slice(chunk));
        let response = tokio::time::timeout(
            Duration::from_millis(client.timeout_ms()),
            client.nats().send_request(grant.subject.clone(), request),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

        let ack = parse_upload_ack(response)?;
        if matches!(ack, UploadAck::Complete { .. }) {
            return Err(TrellisClientError::TransferProtocol(
                "upload completed before eof frame".into(),
            ));
        }
        seq += 1;
    }

    let reply = client.nats().new_inbox();
    let headers = upload_headers(
        client.auth(),
        &context_digest,
        &grant.subject,
        &reply,
        &[],
        seq,
        true,
    )?;
    let request = async_nats::Request::new()
        .inbox(reply)
        .headers(headers)
        .payload(Bytes::new());
    let response = tokio::time::timeout(
        Duration::from_millis(client.timeout_ms()),
        client.nats().send_request(grant.subject.clone(), request),
    )
    .await
    .map_err(|_| TrellisClientError::Timeout)?
    .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

    match parse_upload_ack(response)? {
        UploadAck::Continue => Err(TrellisClientError::TransferProtocol(
            "upload finished without completion payload".into(),
        )),
        UploadAck::Complete { info } => Ok(info),
    }
}

pub(crate) async fn get_download_grant(
    client: &TrellisClient,
    grant: &DownloadTransferGrant,
) -> Result<Vec<u8>, TrellisClientError> {
    validate_grant(&grant.session_key, client)?;

    let inbox = client.nats().new_inbox();
    let headers = client.signed_headers(&grant.subject, &inbox, &[])?;
    let mut subscriber = tokio::time::timeout(
        Duration::from_millis(client.timeout_ms()),
        client.nats().subscribe(inbox.clone()),
    )
    .await
    .map_err(|_| TrellisClientError::Timeout)?
    .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

    tokio::time::timeout(
        Duration::from_millis(client.timeout_ms()),
        client.nats().publish_with_reply_and_headers(
            grant.subject.clone(),
            inbox,
            headers,
            Bytes::new(),
        ),
    )
    .await
    .map_err(|_| TrellisClientError::Timeout)?
    .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

    let mut out = Vec::new();
    loop {
        let next = tokio::time::timeout(
            Duration::from_millis(client.timeout_ms()),
            subscriber.next(),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?;

        let message = next.ok_or_else(|| {
            TrellisClientError::TransferProtocol("download stream closed early".into())
        })?;

        if message
            .headers
            .as_ref()
            .and_then(|headers| headers.get("status"))
            .is_some_and(|status| status.as_str() == "error")
        {
            let value: serde_json::Value = serde_json::from_slice(&message.payload)?;
            return Err(TrellisClientError::TransferProtocol(value.to_string()));
        }

        out.extend_from_slice(&message.payload);

        if message
            .headers
            .as_ref()
            .and_then(|headers| headers.get(TRANSFER_EOF_HEADER))
            .is_some_and(|value| value.as_str() == "true")
        {
            return Ok(out);
        }
    }
}

/// Parse a receive transfer grant from generated SDK or raw JSON values.
pub fn download_transfer_grant_from_value(
    value: serde_json::Value,
) -> Result<DownloadTransferGrant, TrellisClientError> {
    Ok(serde_json::from_value(value)?)
}

fn validate_grant(
    expected_session_key: &str,
    client: &TrellisClient,
) -> Result<(), TrellisClientError> {
    if expected_session_key != client.auth().session_key {
        return Err(TrellisClientError::TransferProtocol(
            "transfer grant session key does not match client session".into(),
        ));
    }
    Ok(())
}

fn parse_upload_ack(message: async_nats::Message) -> Result<UploadAck, TrellisClientError> {
    if message
        .headers
        .as_ref()
        .and_then(|headers| headers.get("status"))
        .is_some_and(|status| status.as_str() == "error")
    {
        let value: serde_json::Value = serde_json::from_slice(&message.payload)?;
        return Err(TrellisClientError::TransferProtocol(value.to_string()));
    }

    Ok(serde_json::from_slice(&message.payload)?)
}

#[cfg(test)]
mod tests {
    use crate::client::proof::verify_event_proof_v2;
    use base64::Engine as _;
    use ed25519_dalek::{Signature, Verifier as _, VerifyingKey};
    use trellis_protocol::build_authorization_request_proof_input_v2;

    use super::*;

    fn test_auth() -> SessionAuth {
        SessionAuth::from_seed_base64url("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
            .expect("session auth")
    }

    const TEST_CONTEXT_DIGEST: &str = "byhVYTUxr4iVywgon-utTJesrl5WZVm1MC0PXqCU06c";

    #[test]
    fn upload_chunk_size_never_returns_zero() {
        assert_eq!(upload_chunk_size(0), 1);
        assert_eq!(upload_chunk_size(6), 6);
    }

    #[test]
    fn upload_chunks_match_raw_transfer_sequence() {
        let body = b"hello world";
        let chunks: Vec<&[u8]> = body.chunks(upload_chunk_size(6)).collect();

        assert_eq!(chunks, vec![b"hello ".as_slice(), b"world".as_slice()]);
        assert_eq!(chunks.len() as u64, 2);
    }

    fn verify_request_proof_headers(
        auth: &SessionAuth,
        subject: &str,
        reply: &str,
        payload: &[u8],
        headers: &HeaderMap,
    ) -> bool {
        let context_digest: [u8; 32] = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(TEST_CONTEXT_DIGEST)
            .expect("context digest")
            .try_into()
            .expect("context digest bytes");
        let input = build_authorization_request_proof_input_v2(
            &context_digest,
            subject,
            Some(reply),
            payload,
            headers
                .get("iat")
                .expect("iat")
                .as_str()
                .parse()
                .expect("iat integer"),
            headers.get("request-id").expect("request-id").as_str(),
        )
        .expect("proof input");
        let public_key = VerifyingKey::from_bytes(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(&auth.session_key)
                .expect("session key")
                .try_into()
                .expect("session key bytes"),
        )
        .expect("public key");
        let signature = Signature::from_bytes(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(headers.get("proof").expect("proof").as_str())
                .expect("proof bytes")
                .try_into()
                .expect("proof signature"),
        );
        public_key.verify(input.digest(), &signature).is_ok()
    }

    #[test]
    fn upload_headers_include_session_proof_sequence_and_eof_marker() {
        let auth = test_auth();
        let subject = "transfer.v1.upload.test.tx1";
        let reply = "_INBOX.test.reply";
        let payload = b"hello ";

        let chunk_headers = upload_headers(
            &auth,
            TEST_CONTEXT_DIGEST,
            subject,
            reply,
            payload,
            0,
            false,
        )
        .expect("chunk headers");

        assert_eq!(
            chunk_headers
                .get("session-key")
                .expect("session-key")
                .as_str(),
            auth.session_key
        );
        assert_eq!(
            chunk_headers
                .get("authorization-context")
                .expect("authorization-context")
                .as_str(),
            TEST_CONTEXT_DIGEST
        );
        assert!(verify_request_proof_headers(
            &auth,
            subject,
            reply,
            payload,
            &chunk_headers
        ));
        assert_eq!(
            chunk_headers
                .get(TRANSFER_SEQUENCE_HEADER)
                .expect("sequence")
                .as_str(),
            "0"
        );
        assert!(chunk_headers.get(TRANSFER_EOF_HEADER).is_none());

        let eof_headers = upload_headers(&auth, TEST_CONTEXT_DIGEST, subject, reply, &[], 2, true)
            .expect("eof headers");

        assert_eq!(
            eof_headers
                .get(TRANSFER_SEQUENCE_HEADER)
                .expect("eof sequence")
                .as_str(),
            "2"
        );
        assert_eq!(
            eof_headers
                .get(TRANSFER_EOF_HEADER)
                .expect("eof marker")
                .as_str(),
            "true"
        );
        assert!(verify_request_proof_headers(
            &auth,
            subject,
            reply,
            &[],
            &eof_headers
        ));
    }

    #[test]
    fn event_proof_v2_verifies_with_context_digest() {
        let auth = test_auth();
        let subject = "events.v1.Documents.Changed.doc-1";
        let payload = br#"{"id":"doc-1"}"#;
        let event_id = "evt_doc_1";
        let event_time = "1970-01-01T00:19:10Z";
        let proof = auth
            .create_event_proof_v2(TEST_CONTEXT_DIGEST, subject, payload, event_id, event_time)
            .expect("event proof");
        assert!(verify_event_proof_v2(
            &auth.session_key,
            TEST_CONTEXT_DIGEST,
            subject,
            payload,
            event_id,
            event_time,
            proof.as_str(),
        )
        .expect("event proof verifies"));
        assert!(!verify_event_proof_v2(
            &auth.session_key,
            TEST_CONTEXT_DIGEST,
            subject,
            payload,
            "evt_other",
            event_time,
            proof.as_str(),
        )
        .expect("changed event id rejects"));
    }
}
