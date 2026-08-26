use async_nats::header::HeaderMap;
use async_nats::jetstream::{self, consumer, AckKind};
use async_nats::ConnectOptions;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use bytes::Bytes;
use futures_util::stream::{self, BoxStream};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use trellis_protocol::{
    parse_participant, resolve_participant, session_proof_request_digest,
    DeviceBootstrapSessionProofInput, ServiceBootstrapSessionProofInput, SessionProofInput,
};

use super::events::{EVENT_ID_HEADER, EVENT_TIME_HEADER};
use crate::client::operations::OperationTransport;
use crate::client::proof::{new_request_id, now_iat_seconds};
use crate::client::transfer::{get_download_grant, DownloadTransferGrant};
use crate::client::transfer::{put_upload_grant, FileInfo, UploadTransferGrant};
use crate::client::{
    prepare_event, AuthorizationContextBundle, AuthorizationContextCache,
    AuthorizationContextStore, AuthorizationProviderCache, AuthorizationRoutingMaterial, CallError,
    EventDescriptor, FeedDescriptor, PreparedTrellisEvent, RpcDescriptor, RpcErrorPayload,
    SessionAuth, TrellisClientError,
};
use crate::service::{BootstrapBinding, CoreBootstrapBinding, ServiceResourceBindings};

const HEALTH_HEARTBEAT_SUBJECT_PREFIX: &str = "health.v1.heartbeat";
const HEALTH_HEARTBEAT_INTERVAL_MS: u64 = 30_000;
const DEFAULT_EVENT_STREAM: &str = "trellis";
static FEED_INBOX_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

struct FeedCancelGuard {
    runtime: tokio::runtime::Handle,
    nats: async_nats::Client,
    auth: Arc<SessionAuth>,
    context_digest: String,
    subject: String,
    reply: String,
    payload: Bytes,
}

impl Drop for FeedCancelGuard {
    fn drop(&mut self) {
        let nats = self.nats.clone();
        let subject = self.subject.clone();
        let reply = self.reply.clone();
        let payload = self.payload.clone();
        let context_digest = self.context_digest.clone();
        let headers = signed_headers(&self.auth, &context_digest, &subject, &reply, &payload);
        self.runtime.spawn(async move {
            let headers = match headers {
                Ok(headers) => headers,
                Err(error) => {
                    tracing::warn!(%error, "feed cancel signing failed");
                    return;
                }
            };
            let _ = nats
                .publish_with_reply_and_headers(subject, reply, headers, payload)
                .await;
        });
    }
}

pub(crate) fn signed_headers(
    auth: &SessionAuth,
    context_digest: &str,
    subject: &str,
    reply: &str,
    payload: &[u8],
) -> Result<HeaderMap, TrellisClientError> {
    let iat = now_iat_seconds() as i64;
    let request_id = new_request_id();
    let proof =
        auth.create_request_proof(context_digest, subject, reply, payload, iat, &request_id)?;
    let mut headers = HeaderMap::new();
    headers.insert("session-key", auth.session_key.as_str());
    headers.insert("authorization-context", context_digest);
    headers.insert("proof", proof.as_str());
    headers.insert("iat", iat.to_string().as_str());
    headers.insert("request-id", request_id.as_str());
    Ok(headers)
}

/// Connection options for a Trellis service that presents native artifacts during bootstrap.
#[cfg_attr(feature = "test-support", doc(hidden))]
pub struct ServiceConnectWithContractOptions<'a> {
    pub trellis_url: &'a str,
    pub participant_id: &'a str,
    pub participant_digest: &'a str,
    pub participant_json: &'a str,
    pub api_json: &'a str,
    pub api_digest: &'a str,
    pub referenced_api_artifacts: &'a [(&'a str, &'a str)],
    pub deployment_id: &'a str,
    pub instance_id: &'a str,
    pub provisioned_identity_seed_base64url: &'a str,
    pub participant_needs_digest: &'a str,
    pub session_key_seed_base64url: &'a str,
    pub timeout_ms: u64,
    pub retry_delay_ms: u64,
    /// Optional maximum authority-pending wait time. `None` waits until authority is ready.
    pub authority_pending_timeout_ms: Option<u64>,
    pub authorization_context_store: Arc<dyn AuthorizationContextStore>,
}

#[doc(hidden)]
pub struct DeviceContractEvidence<'a> {
    participant_id: &'a str,
    participant_digest: &'a str,
    participant_needs_digest: &'a str,
    participant_json: &'a str,
    api_json: &'a str,
    api_digest: &'a str,
    referenced_api_artifacts: Vec<(&'a str, &'a str)>,
}

#[cfg(feature = "test-support")]
impl<'a> DeviceContractEvidence<'a> {
    /// Build dynamic contract evidence for Trellis integration fixtures.
    pub fn for_test(
        participant_id: &'a str,
        participant_digest: &'a str,
        participant_needs_digest: &'a str,
        participant_json: &'a str,
        api_json: &'a str,
        api_digest: &'a str,
        referenced_api_artifacts: &[(&'a str, &'a str)],
    ) -> Self {
        Self {
            participant_id,
            participant_digest,
            participant_needs_digest,
            participant_json,
            api_json,
            api_digest,
            referenced_api_artifacts: referenced_api_artifacts.to_vec(),
        }
    }
}

/// Runtime and device-identity options for an activated device principal.
///
/// The type parameter `C` supplies the exact generated participant and API evidence through
/// [`crate::service::GeneratedServiceContract`]; callers do not provide or duplicate that
/// evidence.
pub struct DeviceConnectOptions<'a, C> {
    trellis_url: &'a str,
    deployment_id: &'a str,
    instance_id: &'a str,
    contract: DeviceContractEvidence<'a>,
    public_identity_key: &'a str,
    identity_seed_base64url: &'a str,
    timeout_ms: u64,
    authorization_context_store: Arc<dyn AuthorizationContextStore>,
    activation_bootstrap: Option<DeviceReadyBootstrap>,
    contract_type: std::marker::PhantomData<C>,
}

impl<'a, C: crate::service::GeneratedServiceContract> DeviceConnectOptions<'a, C> {
    /// Create activated-device connection options using the exact generated evidence from `C`.
    ///
    /// Runtime bootstrap generates fresh session keys internally; this constructor accepts only
    /// runtime location, provisioned device identity, and authorization-context storage inputs.
    pub fn new(
        trellis_url: &'a str,
        deployment_id: &'a str,
        instance_id: &'a str,
        public_identity_key: &'a str,
        identity_seed_base64url: &'a str,
        authorization_context_store: Arc<dyn AuthorizationContextStore>,
    ) -> Self {
        Self {
            trellis_url,
            deployment_id,
            instance_id,
            contract: DeviceContractEvidence {
                participant_id: C::PARTICIPANT_ID,
                participant_digest: C::CONTRACT_DIGEST,
                participant_needs_digest: C::PARTICIPANT_NEEDS_DIGEST,
                participant_json: C::PARTICIPANT_JSON,
                api_json: C::API_JSON,
                api_digest: C::API_DIGEST,
                referenced_api_artifacts: C::REFERENCED_API_ARTIFACTS.to_vec(),
            },
            public_identity_key,
            identity_seed_base64url,
            timeout_ms: crate::service::DEFAULT_TIMEOUT_MS,
            authorization_context_store,
            activation_bootstrap: None,
            contract_type: std::marker::PhantomData,
        }
    }
}

impl<C> DeviceConnectOptions<'_, C> {
    /// Set the request/connect timeout in milliseconds.
    pub const fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Return the device public identity key bound to these options.
    pub fn public_identity_key(&self) -> &str {
        self.public_identity_key
    }

    pub(crate) fn activation_origin_digest(
        &self,
        activation_key_base64url: &str,
        nonce: &str,
        session_key_seed_base64url: &str,
    ) -> [u8; 32] {
        let mut digest = Sha256::new();
        for value in [
            self.trellis_url,
            self.deployment_id,
            self.instance_id,
            self.contract.participant_id,
            self.contract.participant_digest,
            self.contract.participant_needs_digest,
            self.contract.participant_json,
            self.contract.api_json,
            self.contract.api_digest,
            self.public_identity_key,
            self.identity_seed_base64url,
            activation_key_base64url,
            nonce,
            session_key_seed_base64url,
        ] {
            digest.update(value.len().to_be_bytes());
            digest.update(value.as_bytes());
        }
        for (json, artifact_digest) in &self.contract.referenced_api_artifacts {
            for value in [*json, *artifact_digest] {
                digest.update(value.len().to_be_bytes());
                digest.update(value.as_bytes());
            }
        }
        digest.finalize().into()
    }

    pub(crate) fn activation_bootstrap(
        mut self,
        bootstrap: ServiceBootstrapResponse,
        session_key_seed_base64url: String,
    ) -> Self {
        self.activation_bootstrap = Some(DeviceReadyBootstrap {
            response: bootstrap,
            session_key_seed_base64url,
        });
        self
    }

    #[cfg(test)]
    pub(crate) fn activation_session_seed(&self) -> Option<&str> {
        self.activation_bootstrap
            .as_ref()
            .map(|ready| ready.session_key_seed_base64url.as_str())
    }
}

#[cfg(feature = "test-support")]
pub(crate) fn test_device_connect_options<'a>(
    trellis_url: &'a str,
    deployment_id: &'a str,
    instance_id: &'a str,
    contract: DeviceContractEvidence<'a>,
    public_identity_key: &'a str,
    identity_seed_base64url: &'a str,
    authorization_context_store: Arc<dyn AuthorizationContextStore>,
) -> DeviceConnectOptions<'a, crate::generated::DynamicDeviceContract> {
    DeviceConnectOptions {
        trellis_url,
        deployment_id,
        instance_id,
        contract,
        public_identity_key,
        identity_seed_base64url,
        timeout_ms: crate::service::DEFAULT_TIMEOUT_MS,
        authorization_context_store,
        activation_bootstrap: None,
        contract_type: std::marker::PhantomData,
    }
}

struct DeviceReadyBootstrap {
    response: ServiceBootstrapResponse,
    session_key_seed_base64url: String,
}

/// Whether an event subscription uses a durable or ephemeral JetStream consumer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EventSubscriptionMode {
    /// Reuse a named durable consumer and retain delivery state across reconnects.
    Durable,
    /// Create an unnamed consumer that ends when the subscription is dropped.
    #[default]
    Ephemeral,
}

/// Initial delivery position for a descriptor-backed event subscription.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EventReplayPolicy {
    /// Deliver all retained events visible to the consumer.
    All,
    /// Deliver only events published after the consumer is created.
    #[default]
    New,
}

/// Options for descriptor-backed event subscriptions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventSubscribeOptions {
    /// JetStream stream that owns the event consumer. Defaults to the Trellis event stream.
    pub stream: Option<String>,
    /// Durable or ephemeral consumer mode.
    pub mode: EventSubscriptionMode,
    /// Initial delivery position for a newly created consumer.
    pub replay: EventReplayPolicy,
    /// Optional durable name. Ignored for ephemeral subscriptions.
    pub durable_name: Option<String>,
}

/// One descriptor-backed event message with explicit JetStream acknowledgement controls.
#[derive(Debug)]
pub struct EventMessage<T> {
    message: jetstream::Message,
    _event: PhantomData<fn() -> T>,
}

impl<T> EventMessage<T> {
    /// Return the raw NATS headers delivered with the event message, if present.
    pub fn headers(&self) -> Option<&HeaderMap> {
        self.message.headers.as_ref()
    }

    /// Return the Trellis event id from the `Nats-Msg-Id` header, when present.
    pub fn event_id(&self) -> Option<&str> {
        self.headers()
            .and_then(|headers| headers.get(EVENT_ID_HEADER))
            .map(|value| value.as_str())
    }

    /// Return the Trellis event timestamp from the `Trellis-Event-Time` header, when present.
    pub fn event_time(&self) -> Option<&str> {
        self.headers()
            .and_then(|headers| headers.get(EVENT_TIME_HEADER))
            .map(|value| value.as_str())
    }

    /// Return the raw JSON payload bytes.
    pub fn payload(&self) -> &[u8] {
        &self.message.payload
    }

    /// Return the concrete NATS subject that delivered this event message.
    pub fn subject(&self) -> &str {
        self.message.subject.as_ref()
    }

    /// Decode the message payload as the descriptor's typed event payload.
    pub fn decode(&self) -> Result<T, TrellisClientError>
    where
        T: for<'de> Deserialize<'de>,
    {
        Ok(serde_json::from_slice(&self.message.payload)?)
    }

    /// Acknowledge successful handling of the message.
    pub async fn ack(&self) -> Result<(), TrellisClientError> {
        self.message
            .ack()
            .await
            .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))
    }

    /// Negatively acknowledge the message so JetStream may redeliver it.
    pub async fn nak(&self) -> Result<(), TrellisClientError> {
        self.message
            .ack_with(AckKind::Nak(None))
            .await
            .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))
    }

    pub(crate) async fn nak_after(&self, delay: Duration) -> Result<(), TrellisClientError> {
        self.message
            .ack_with(AckKind::Nak(Some(delay)))
            .await
            .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))
    }

    /// Terminate the message without successful acknowledgement or redelivery.
    pub async fn term(&self) -> Result<(), TrellisClientError> {
        self.message
            .ack_with(AckKind::Term)
            .await
            .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServiceBootstrapRequest {
    request_id: String,
    issued_at: i64,
    deployment_id: String,
    instance_id: String,
    provisioned_identity_key_id: String,
    new_session_public_key: String,
    new_session_nkey: String,
    participant_id: String,
    participant_artifact_digest: String,
    participant_needs_digest: String,
    participant_artifact: Value,
    referenced_api_artifacts: Vec<Value>,
    proof: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServiceBootstrapResponse {
    pub(crate) server_now: i64,
    #[serde(skip)]
    server_clock_offset_ms: i64,
    pub(crate) state: String,
    session: Option<ServiceBootstrapSession>,
    authorization: Option<Value>,
    nats: Option<ServiceBootstrapNats>,
    authorization_context: Option<AuthorizationContextBundle>,
    pub(crate) activation: Option<DeviceBootstrapActivation>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeviceBootstrapActivation {
    pub(crate) state: String,
    pub(crate) activation_url: String,
    pub(crate) review_id: String,
    pub(crate) expires_at: i64,
    pub(crate) retry_after_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceBootstrapSession {
    session_id: String,
    inbox_prefix: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceBootstrapNats {
    jwt: String,
    jwt_expires_at: i64,
    servers: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceBootstrapAuthorization {
    participant_id: String,
    participant_artifact_digest: String,
    resource_runtime: ServiceResourceBindings,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NatsConnectToken {
    format: &'static str,
    context_digest: String,
}

#[derive(Debug)]
struct ServiceBootstrapFetchOptions<'a> {
    trellis_url: &'a str,
    timeout_ms: u64,
    retry_delay_ms: Option<u64>,
    authority_pending_timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceBootstrapRequest {
    request_id: String,
    issued_at: i64,
    deployment_id: String,
    instance_id: String,
    device_identity_key_id: String,
    principal_id: Option<String>,
    identity_public_key: Option<String>,
    provisioning_secret: Option<String>,
    expected_secret_version: Option<u64>,
    new_session_public_key: String,
    new_session_nkey: String,
    participant_id: String,
    participant_artifact_digest: String,
    participant_needs_digest: String,
    participant_artifact: Value,
    referenced_api_artifacts: Vec<Value>,
    challenge_digest: Option<String>,
    confirmation_code: Option<String>,
    proof: Value,
}

pub(crate) struct DeviceActivationEvidence<'a> {
    pub(crate) challenge_digest: &'a str,
    pub(crate) confirmation_code: &'a str,
}

#[derive(Default)]
pub(crate) struct DeviceBootstrapProofOverrides {
    pub(crate) issued_at_ms: Option<i64>,
    pub(crate) corrupt_signature: bool,
}

#[derive(Clone, Debug)]
struct HealthHeartbeatConfig {
    session_key: String,
    service_name: String,
    kind: HealthHeartbeatServiceKind,
    deployment_id: String,
    instance_id: String,
    contract_id: String,
    contract_digest: String,
    started_at: String,
    publish_interval_ms: u64,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
enum HealthHeartbeatServiceKind {
    Service,
    Device,
}

async fn fetch_service_bootstrap_with_contract(
    session_auth: &SessionAuth,
    opts: &ServiceConnectWithContractOptions<'_>,
    participant_artifact: Value,
    api_artifact: Value,
    referenced_api_artifacts: Vec<Value>,
) -> Result<ServiceBootstrapResponse, TrellisClientError> {
    let identity_auth = SessionAuth::from_seed_base64url(opts.provisioned_identity_seed_base64url)?;
    let session_nkey = session_auth.nkey_pair()?.public_key();
    let request = ServiceBootstrapRequest {
        request_id: String::new(),
        issued_at: 0,
        deployment_id: opts.deployment_id.to_owned(),
        instance_id: opts.instance_id.to_owned(),
        provisioned_identity_key_id: identity_auth.key_id(),
        new_session_public_key: session_auth.session_key.clone(),
        new_session_nkey: session_nkey.clone(),
        participant_id: opts.participant_id.to_owned(),
        participant_artifact_digest: opts.participant_digest.to_owned(),
        participant_needs_digest: opts.participant_needs_digest.to_owned(),
        participant_artifact,
        referenced_api_artifacts: std::iter::once(api_artifact)
            .chain(referenced_api_artifacts)
            .collect(),
        proof: serde_json::json!({
            "format": "trellis.session-proof.v1",
            "signature": ""
        }),
    };
    fetch_service_bootstrap_inner(
        request,
        &identity_auth,
        &session_nkey,
        &ServiceBootstrapFetchOptions {
            trellis_url: opts.trellis_url,
            timeout_ms: opts.timeout_ms,
            retry_delay_ms: Some(opts.retry_delay_ms),
            authority_pending_timeout_ms: opts.authority_pending_timeout_ms,
        },
    )
    .await
}

async fn fetch_service_bootstrap_inner(
    mut request: ServiceBootstrapRequest,
    identity_auth: &SessionAuth,
    session_nkey: &str,
    opts: &ServiceBootstrapFetchOptions<'_>,
) -> Result<ServiceBootstrapResponse, TrellisClientError> {
    let mut url = reqwest::Url::parse(opts.trellis_url)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    url.set_path("/bootstrap/service");
    url.set_query(None);
    url.set_fragment(None);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(opts.timeout_ms))
        .build()
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    let authority_pending_deadline = opts.authority_pending_timeout_ms.map(|timeout_ms| {
        tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms)
    });
    loop {
        request.request_id = new_request_id();
        request.issued_at = now_iat_seconds()
            .checked_mul(1_000)
            .and_then(|value| i64::try_from(value).ok())
            .ok_or_else(|| TrellisClientError::Bootstrap("bootstrap timestamp overflow".into()))?;
        request.proof = serde_json::json!({
            "format": "trellis.session-proof.v1",
            "signature": ""
        });
        let request_digest = session_proof_request_digest(&serde_json::to_value(&request)?)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let proof_input = SessionProofInput::service_bootstrap(ServiceBootstrapSessionProofInput {
            request_id: request.request_id.clone(),
            issued_at: request.issued_at,
            deployment_id: request.deployment_id.clone(),
            instance_id: request.instance_id.clone(),
            provisioned_identity_key_id: request.provisioned_identity_key_id.clone(),
            new_session_public_key: request.new_session_public_key.clone(),
            new_session_nkey: session_nkey.to_owned(),
            participant_id: request.participant_id.clone(),
            participant_digest: request.participant_artifact_digest.clone(),
            request_digest,
        })
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        request.proof = serde_json::to_value(identity_auth.sign_session_proof(&proof_input)?)?;
        let request_started_at = now_context_millis()?;
        let response = client
            .post(url.clone())
            .json(&request)
            .send()
            .await
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let response_received_at = now_context_millis()?;
        if !status.is_success() {
            if status == reqwest::StatusCode::CONFLICT && body == r#"{"error":{"code":"conflict"}}"# {
                let delay =
                    std::time::Duration::from_millis(opts.retry_delay_ms.unwrap_or(1).max(1));
                if let Some(deadline) = authority_pending_deadline {
                    let now = tokio::time::Instant::now();
                    if now >= deadline {
                        return Err(TrellisClientError::Bootstrap(
                            "timed out waiting for service deployment authority".into(),
                        ));
                    }
                    tokio::time::sleep(delay.min(deadline.saturating_duration_since(now))).await;
                } else {
                    tokio::time::sleep(delay).await;
                }
                continue;
            }
            return Err(TrellisClientError::BootstrapHttp {
                status: status.as_u16(),
                body,
            });
        }

        let mut response: ServiceBootstrapResponse = serde_json::from_str(&body)?;
        let midpoint = request_started_at
            .checked_add(response_received_at)
            .and_then(|sum| sum.checked_div(2))
            .ok_or_else(|| TrellisClientError::Bootstrap("bootstrap time overflow".into()))?;
        response.server_clock_offset_ms = response
            .server_now
            .checked_sub(midpoint)
            .ok_or_else(|| TrellisClientError::Bootstrap("bootstrap time overflow".into()))?;
        if response.state == "ready" {
            return Ok(response);
        }
        if response.state != "authority_pending" {
            return Err(TrellisClientError::Bootstrap(format!(
                "unexpected service bootstrap state '{}'",
                response.state
            )));
        }
        let delay = std::time::Duration::from_millis(opts.retry_delay_ms.unwrap_or(1).max(1));
        if let Some(deadline) = authority_pending_deadline {
            let now = tokio::time::Instant::now();
            if now >= deadline {
                return Err(TrellisClientError::Bootstrap(
                    "timed out waiting for service deployment authority".into(),
                ));
            }
            tokio::time::sleep(delay.min(deadline.saturating_duration_since(now))).await;
        } else {
            tokio::time::sleep(delay).await;
        }
    }
}

async fn fetch_device_bootstrap<C>(
    identity_auth: &SessionAuth,
    session_auth: &SessionAuth,
    opts: &DeviceConnectOptions<'_, C>,
    activation: Option<DeviceActivationEvidence<'_>>,
    proof_overrides: DeviceBootstrapProofOverrides,
) -> Result<ServiceBootstrapResponse, TrellisClientError> {
    let mut url = reqwest::Url::parse(opts.trellis_url)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    url.set_path("/bootstrap/device");
    url.set_query(None);
    url.set_fragment(None);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(opts.timeout_ms))
        .build()
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    let request_id = new_request_id();
    let issued_at = match proof_overrides.issued_at_ms {
        Some(issued_at) => issued_at,
        None => i64::try_from(now_iat_seconds())
            .ok()
            .and_then(|value| value.checked_mul(1_000))
            .ok_or_else(|| TrellisClientError::Bootstrap("device timestamp overflow".into()))?,
    };
    let session_nkey = session_auth.nkey_pair()?.public_key();
    let participant_artifact: Value = serde_json::from_str(opts.contract.participant_json)?;
    let parsed_participant = trellis_protocol::parse_participant(&participant_artifact)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    if parsed_participant.id() != opts.contract.participant_id
        || parsed_participant
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
            != opts.contract.participant_digest
    {
        return Err(TrellisClientError::Bootstrap(
            "participant artifact identity mismatch".into(),
        ));
    }
    let api_artifact: Value = serde_json::from_str(opts.contract.api_json)?;
    let parsed_api = trellis_protocol::parse_api(&api_artifact)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    if parsed_api
        .digest()
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
        != opts.contract.api_digest
    {
        return Err(TrellisClientError::Bootstrap(
            "owned API artifact digest mismatch".into(),
        ));
    }
    let mut participant_apis = std::collections::BTreeMap::new();
    participant_apis.insert(parsed_api.id().to_owned(), parsed_api.clone());
    let referenced_api_artifacts = opts
        .contract
        .referenced_api_artifacts
        .iter()
        .map(|(json, digest)| {
            let artifact = serde_json::from_str(json)?;
            let parsed = trellis_protocol::parse_api(&artifact)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
            if parsed
                .digest()
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
                != *digest
            {
                return Err(TrellisClientError::Bootstrap(
                    "referenced API artifact digest mismatch".into(),
                ));
            }
            participant_apis.insert(parsed.id().to_owned(), parsed);
            Ok(artifact)
        })
        .collect::<Result<Vec<_>, TrellisClientError>>()?;
    let resolved = trellis_protocol::resolve_participant(&parsed_participant, &participant_apis)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    if resolved
        .needs()
        .digest()
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
        != opts.contract.participant_needs_digest
    {
        return Err(TrellisClientError::Bootstrap(
            "participant needs digest mismatch".into(),
        ));
    }
    let mut request = serde_json::to_value(DeviceBootstrapRequest {
        request_id: request_id.clone(),
        issued_at,
        deployment_id: opts.deployment_id.to_owned(),
        instance_id: opts.instance_id.to_owned(),
        device_identity_key_id: identity_auth.key_id(),
        principal_id: None,
        identity_public_key: Some(identity_auth.session_key.clone()),
        provisioning_secret: None,
        expected_secret_version: None,
        new_session_public_key: session_auth.session_key.clone(),
        new_session_nkey: session_nkey.clone(),
        participant_id: opts.contract.participant_id.to_owned(),
        participant_artifact_digest: opts.contract.participant_digest.to_owned(),
        participant_needs_digest: opts.contract.participant_needs_digest.to_owned(),
        participant_artifact,
        referenced_api_artifacts: std::iter::once(api_artifact)
            .chain(referenced_api_artifacts)
            .collect(),
        challenge_digest: activation
            .as_ref()
            .map(|activation| activation.challenge_digest.to_owned()),
        confirmation_code: activation
            .as_ref()
            .map(|activation| activation.confirmation_code.to_owned()),
        proof: serde_json::json!({
            "format": trellis_protocol::SESSION_PROOF_FORMAT_V1,
            "signature": "",
        }),
    })?;
    let request_digest = trellis_protocol::session_proof_request_digest(&request)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    let input = SessionProofInput::device_bootstrap(DeviceBootstrapSessionProofInput {
        request_id,
        issued_at,
        deployment_id: opts.deployment_id.to_owned(),
        instance_id: opts.instance_id.to_owned(),
        device_identity_key_id: identity_auth.key_id(),
        new_session_public_key: session_auth.session_key.clone(),
        new_session_nkey: session_nkey,
        participant_id: opts.contract.participant_id.to_owned(),
        participant_digest: opts.contract.participant_digest.to_owned(),
        challenge_digest: activation
            .as_ref()
            .map(|activation| activation.challenge_digest.to_owned()),
        request_digest,
    })
    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    request["proof"] = serde_json::to_value(identity_auth.sign_session_proof(&input)?)?;
    if proof_overrides.corrupt_signature {
        request["proof"]["signature"] = Value::String("invalid".to_owned());
    }
    let request_started_at = now_context_millis()?;
    let response = client
        .post(url)
        .json(&request)
        .send()
        .await
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    let response_received_at = now_context_millis()?;
    if !status.is_success() {
        return Err(TrellisClientError::BootstrapHttp {
            status: status.as_u16(),
            body,
        });
    }
    let mut response: ServiceBootstrapResponse = serde_json::from_str(&body)?;
    let midpoint = request_started_at
        .checked_add(response_received_at)
        .and_then(|sum| sum.checked_div(2))
        .ok_or_else(|| TrellisClientError::Bootstrap("device bootstrap time overflow".into()))?;
    response.server_clock_offset_ms = response
        .server_now
        .checked_sub(midpoint)
        .ok_or_else(|| TrellisClientError::Bootstrap("device bootstrap time overflow".into()))?;
    Ok(response)
}

pub(crate) async fn fetch_device_activation<C>(
    opts: &DeviceConnectOptions<'_, C>,
    session_auth: &SessionAuth,
    challenge_digest: &str,
    confirmation_code: &str,
) -> Result<ServiceBootstrapResponse, TrellisClientError> {
    let identity_auth = SessionAuth::from_seed_base64url(opts.identity_seed_base64url)?;
    if identity_auth.session_key != opts.public_identity_key {
        return Err(TrellisClientError::Bootstrap(
            "device public identity key does not match identity seed".into(),
        ));
    }
    fetch_device_bootstrap(
        &identity_auth,
        session_auth,
        opts,
        Some(DeviceActivationEvidence {
            challenge_digest,
            confirmation_code,
        }),
        DeviceBootstrapProofOverrides::default(),
    )
    .await
}

#[cfg(feature = "test-support")]
pub(crate) async fn fetch_device_activation_with_test_proof<C>(
    opts: &DeviceConnectOptions<'_, C>,
    session_auth: &SessionAuth,
    challenge_digest: &str,
    confirmation_code: &str,
    proof_overrides: DeviceBootstrapProofOverrides,
) -> Result<ServiceBootstrapResponse, TrellisClientError> {
    let identity_auth = SessionAuth::from_seed_base64url(opts.identity_seed_base64url)?;
    fetch_device_bootstrap(
        &identity_auth,
        session_auth,
        opts,
        Some(DeviceActivationEvidence {
            challenge_digest,
            confirmation_code,
        }),
        proof_overrides,
    )
    .await
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn now_context_seconds() -> Result<i64, TrellisClientError> {
    i64::try_from(now_iat_seconds())
        .map_err(|_| TrellisClientError::Bootstrap("context time overflow".into()))
}

fn now_context_millis() -> Result<i64, TrellisClientError> {
    i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
            .as_millis(),
    )
    .map_err(|_| TrellisClientError::Bootstrap("context time overflow".into()))
}

fn jwt_expiry(jwt: &str) -> Result<i64, TrellisClientError> {
    let payload = jwt
        .split('.')
        .nth(1)
        .ok_or_else(|| TrellisClientError::Bootstrap("bootstrap JWT has no payload".into()))?;
    let payload = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    serde_json::from_slice::<Value>(&payload)?["exp"]
        .as_i64()
        .filter(|expires_at| *expires_at > 0)
        .ok_or_else(|| TrellisClientError::Bootstrap("bootstrap JWT has no expiry".into()))
}

fn health_subject_token(value: &str) -> String {
    URL_SAFE_NO_PAD.encode(value.as_bytes())
}

fn health_heartbeat_subject(config: &HealthHeartbeatConfig) -> String {
    let kind = match config.kind {
        HealthHeartbeatServiceKind::Service => "service",
        HealthHeartbeatServiceKind::Device => "device",
    };
    format!(
        "{HEALTH_HEARTBEAT_SUBJECT_PREFIX}.{kind}.{}.{}.{}.{}.{}",
        health_subject_token(&config.contract_id),
        health_subject_token(&config.contract_digest),
        health_subject_token(&config.deployment_id),
        health_subject_token(&config.instance_id),
        config.session_key,
    )
}

fn build_health_heartbeat(config: &HealthHeartbeatConfig) -> Value {
    serde_json::json!({
        "sample": {
            "id": ulid::Ulid::new().to_string(),
            "time": now_rfc3339(),
        },
        "participant": {
            "name": config.service_name,
            "kind": config.kind,
            "instanceId": config.instance_id,
            "contractId": config.contract_id,
            "contractDigest": config.contract_digest,
            "startedAt": config.started_at,
            "publishIntervalMs": config.publish_interval_ms,
            "runtime": "rust",
        },
        "reportedStatus": "healthy",
        "checks": [{
            "name": "nats",
            "status": "ok",
            "latencyMs": 0.0,
        }],
    })
}

fn signed_event_headers(
    auth: &SessionAuth,
    context_digest: &str,
    event: &PreparedTrellisEvent,
) -> Result<HeaderMap, TrellisClientError> {
    let mut headers = event.publish_headers();
    headers.insert("session-key", auth.session_key.as_str());
    headers.insert("authorization-context", context_digest);
    headers.insert(
        "proof",
        auth.create_event_proof(
            context_digest,
            event.subject(),
            event.payload(),
            event.event_id(),
            event.event_time(),
        )?
        .as_str(),
    );
    Ok(headers)
}

async fn publish_prepared_event(
    nats: &async_nats::Client,
    auth: &SessionAuth,
    context_digest: &str,
    timeout_ms: u64,
    event: &PreparedTrellisEvent,
) -> Result<(), TrellisClientError> {
    let jetstream = jetstream::new(nats.clone());
    let headers = signed_event_headers(auth, context_digest, event)?;
    let publish = async {
        jetstream
            .publish_with_headers(event.subject().to_string(), headers, event.payload_bytes())
            .await
    };
    let ack = timeout(std::time::Duration::from_millis(timeout_ms), publish)
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;
    timeout(std::time::Duration::from_millis(timeout_ms), ack)
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;
    Ok(())
}

async fn publish_health_heartbeat(
    nats: &async_nats::Client,
    timeout_ms: u64,
    config: &HealthHeartbeatConfig,
) -> Result<(), TrellisClientError> {
    let payload = Bytes::from(serde_json::to_vec(&build_health_heartbeat(config))?);
    let jetstream = jetstream::new(nats.clone());
    let publish = jetstream.publish(health_heartbeat_subject(config), payload);
    let ack = timeout(std::time::Duration::from_millis(timeout_ms), publish)
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;
    timeout(std::time::Duration::from_millis(timeout_ms), ack)
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;
    Ok(())
}

fn spawn_health_heartbeat_task(
    nats: async_nats::Client,
    timeout_ms: u64,
    config: HealthHeartbeatConfig,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_millis(config.publish_interval_ms));
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(error) = publish_health_heartbeat(&nats, timeout_ms, &config).await {
                tracing::warn!(%error, "failed to publish health heartbeat");
            }
        }
    })
}

fn spawn_authorization_context_refresh_task(
    contexts: std::sync::Arc<AuthorizationContextCache>,
    auth: Arc<SessionAuth>,
    nats: async_nats::Client,
) -> JoinHandle<()> {
    crate::client::authorization::spawn_authorization_context_refresh_task(contexts, auth, nats)
}

/// Connected provider-cache handle returned by attach.
struct AuthorizationProviderHandle {
    provider: AuthorizationProviderCache,
    stop: tokio::sync::watch::Sender<()>,
    task: JoinHandle<()>,
}

/// Attach the connected NATS authorization registry to this client's provider
/// cache and wait for its complete snapshot before returning.
async fn attach_authorization_provider(
    nats: async_nats::Client,
    authorization_contexts: Arc<AuthorizationContextCache>,
) -> Result<AuthorizationProviderHandle, TrellisClientError> {
    let registry_binding = match authorization_contexts.bundle() {
        Ok(bundle) => bundle.trust.authorization_registry.clone(),
        Err(error) => return Err(error),
    };
    let provider = match AuthorizationProviderCache::attach(
        nats.clone(),
        &registry_binding,
        authorization_contexts.clone(),
    )
    .await
    {
        Ok(provider) => provider,
        Err(error) => {
            let _ = nats.drain().await;
            return Err(error);
        }
    };
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(());
    let watcher = provider.clone();
    let task_stop = stop_tx.clone();
    let task_nats = nats.clone();
    let task = tokio::spawn(async move {
        if let Err(error) = watcher.run(stop_rx).await {
            tracing::warn!(%error, "authorization provider watch stopped");
            let _ = task_stop.send(());
            let _ = task_nats.drain().await;
        }
    });
    let ready = match tokio::time::timeout(
        std::time::Duration::from_secs(30),
        provider.wait_ready(stop_tx.subscribe()),
    )
    .await
    {
        Ok(ready) => ready,
        Err(_) => {
            let _ = stop_tx.send(());
            task.abort();
            let _ = nats.drain().await;
            return Err(TrellisClientError::Bootstrap(
                "authorization provider did not become ready".into(),
            ));
        }
    };
    if let Err(error) = ready {
        let _ = stop_tx.send(());
        task.abort();
        let _ = nats.drain().await;
        return Err(error);
    }
    Ok(AuthorizationProviderHandle {
        provider,
        stop: stop_tx,
        task,
    })
}

async fn connect_bootstrapped_service(
    auth: SessionAuth,
    opts: &ServiceConnectWithContractOptions<'_>,
    bootstrap: ServiceBootstrapResponse,
) -> Result<TrellisClient, TrellisClientError> {
    let session_key_seed_base64url = opts.session_key_seed_base64url;
    let participant_id = opts.participant_id;
    let participant_digest = opts.participant_digest;
    let deployment_id = opts.deployment_id;
    let instance_id = opts.instance_id;
    let timeout_ms = opts.timeout_ms;
    let session = bootstrap
        .session
        .ok_or_else(|| TrellisClientError::Bootstrap("missing bootstrap session".into()))?;
    let authorization = bootstrap
        .authorization
        .ok_or_else(|| TrellisClientError::Bootstrap("missing bootstrap authorization".into()))?;
    let authorization_identity =
        serde_json::from_value::<ServiceBootstrapAuthorization>(authorization.clone())?;
    if authorization_identity.participant_id != participant_id
        || authorization_identity.participant_artifact_digest != participant_digest
    {
        return Err(TrellisClientError::Bootstrap(
            "bootstrap authorization participant mismatch".into(),
        ));
    }
    let nats_credential = bootstrap
        .nats
        .ok_or_else(|| TrellisClientError::Bootstrap("missing NATS bootstrap credential".into()))?;
    let authorization_context = bootstrap.authorization_context.ok_or_else(|| {
        TrellisClientError::Bootstrap("missing authorization context bundle".into())
    })?;
    let authorization_contexts = AuthorizationContextCache::new(
        opts.trellis_url,
        format!("service:{}:{}", opts.deployment_id, opts.instance_id),
        opts.authorization_context_store.clone(),
    )?;
    authorization_contexts.set_server_clock_offset_ms(bootstrap.server_clock_offset_ms);
    authorization_contexts
        .install(
            authorization_context,
            AuthorizationRoutingMaterial {
                bootstrap_jwt: nats_credential.jwt.clone(),
                bootstrap_jwt_expires_at: nats_credential.jwt_expires_at,
            },
            bootstrap.server_now.div_euclid(1_000),
        )
        .await?;
    let authorization_contexts = Arc::new(authorization_contexts);
    let (context_session_id, _, context_participant_digest, _, _) =
        authorization_contexts.refresh_evidence()?;
    if context_session_id != session.session_id || context_participant_digest != participant_digest
    {
        return Err(TrellisClientError::Bootstrap(
            "authorization context binding mismatch".into(),
        ));
    }
    if nats_credential.servers.is_empty() {
        return Err(TrellisClientError::Bootstrap(
            "native NATS transport has no servers".into(),
        ));
    }
    let inbox_prefix = session.inbox_prefix;
    let callback_auth = std::sync::Arc::new(SessionAuth::from_seed_base64url(
        session_key_seed_base64url,
    )?);
    let key_pair = std::sync::Arc::new(callback_auth.nkey_pair()?);
    let session_nkey = key_pair.public_key();
    let callback_authorization_contexts = authorization_contexts.clone();
    let reauth = Arc::new(AtomicBool::new(false));
    let health_session_key = auth.session_key.clone();

    let nats = ConnectOptions::with_auth_callback(move |nonce| {
        let auth = callback_auth.clone();
        let key_pair = key_pair.clone();
        let session_nkey = session_nkey.clone();
        let authorization_contexts = callback_authorization_contexts.clone();
        let reauth = reauth.clone();
        async move {
            if reauth.swap(true, Ordering::AcqRel)
                || authorization_contexts.routing_jwt().is_err()
                || authorization_contexts.context_digest().is_err()
            {
                authorization_contexts
                    .refresh(&auth)
                    .await
                    .map_err(async_nats::AuthError::new)?;
            }
            let deny_all_jwt = authorization_contexts
                .routing_jwt()
                .map_err(async_nats::AuthError::new)?;
            let context_digest = authorization_contexts
                .context_digest()
                .map_err(async_nats::AuthError::new)?;
            let nonce_signature = key_pair.sign(&nonce).map_err(async_nats::AuthError::new)?;
            let mut credentials = async_nats::Auth::new();
            credentials.nkey = Some(session_nkey);
            credentials.jwt = Some(deny_all_jwt);
            credentials.signature = Some(nonce_signature.clone());
            credentials.token = Some(
                serde_json::to_string(&NatsConnectToken {
                    format: "trellis.nats-connect-token.v1",
                    context_digest,
                })
                .map_err(async_nats::AuthError::new)?,
            );
            Ok(credentials)
        }
    })
    .connection_timeout(std::time::Duration::from_millis(timeout_ms))
    .custom_inbox_prefix(inbox_prefix.clone())
    .connect(nats_credential.servers)
    .await
    .map_err(|error| {
        TrellisClientError::NatsConnect(format!(
            "service runtime connect failed for participant '{participant_id}' digest '{participant_digest}': {error}"
        ))
    })?;
    let provider =
        attach_authorization_provider(nats.clone(), authorization_contexts.clone()).await?;

    let health_heartbeat_config = HealthHeartbeatConfig {
        session_key: health_session_key,
        service_name: participant_id.to_string(),
        kind: HealthHeartbeatServiceKind::Service,
        deployment_id: deployment_id.to_owned(),
        instance_id: instance_id.to_owned(),
        contract_id: participant_id.to_string(),
        contract_digest: participant_digest.to_string(),
        started_at: now_rfc3339(),
        publish_interval_ms: HEALTH_HEARTBEAT_INTERVAL_MS,
    };
    if let Err(error) = publish_health_heartbeat(&nats, timeout_ms, &health_heartbeat_config).await
    {
        tracing::warn!(%error, "failed to publish initial health heartbeat");
    }
    let health_heartbeat_task = Some(spawn_health_heartbeat_task(
        nats.clone(),
        timeout_ms,
        health_heartbeat_config,
    ));

    let auth = Arc::new(auth);
    let authorization_context_refresh_task = Some(spawn_authorization_context_refresh_task(
        authorization_contexts.clone(),
        auth.clone(),
        nats.clone(),
    ));
    Ok(TrellisClient {
        nats,
        auth,
        inbox_prefix,
        timeout_ms,
        service_bootstrap_binding: Some(CoreBootstrapBinding::new(
            BootstrapBinding {
                contract_id: authorization_identity.participant_id,
                digest: authorization_identity.participant_artifact_digest,
            },
            authorization_identity.resource_runtime,
        )),
        health_heartbeat_task,
        authorization_contexts: Some(authorization_contexts),
        authorization_context_refresh_task,
        authorization_provider: Some(provider.provider),
        _authorization_provider_stop: Some(provider.stop),
        _authorization_provider_task: Some(provider.task),
    })
}

/// Connection options for a user/session-key principal.
pub struct UserConnectOptions<'a> {
    trellis_url: &'a str,
    servers: &'a str,
    inbox_prefix: &'a str,
    timeout_ms: u64,
    credentials: UserSessionCredentials<'a>,
    authorization: UserAuthorizationContext,
    refresh_before_connect: bool,
}

/// Secret session credentials and participant identity for a user connection.
pub struct UserSessionCredentials<'a> {
    /// Bootstrap JWT used to establish or refresh NATS routing authorization.
    pub bootstrap_jwt: &'a str,
    /// Session identifier bound into the authorization context.
    pub session_id: &'a str,
    /// Base64url-encoded Ed25519 session key seed.
    pub session_key_seed_base64url: &'a str,
    /// Exact participant artifact digest authorized for the session.
    pub participant_digest: &'a str,
}

/// Authorization context and persistent cache identity for a user connection.
pub struct UserAuthorizationContext {
    /// Signed authorization context supplied by session bootstrap.
    pub bundle: AuthorizationContextBundle,
    /// Stable identity used to bind the cached context to this installation or device.
    pub binding: String,
    /// Store used to persist authorization context and routing material.
    pub store: Arc<dyn AuthorizationContextStore>,
}

impl<'a> UserConnectOptions<'a> {
    /// Create user-authenticated connection options.
    pub fn new(
        trellis_url: &'a str,
        servers: &'a str,
        inbox_prefix: &'a str,
        timeout_ms: u64,
        credentials: UserSessionCredentials<'a>,
        authorization: UserAuthorizationContext,
    ) -> Self {
        Self {
            trellis_url,
            servers,
            inbox_prefix,
            timeout_ms,
            credentials,
            authorization,
            refresh_before_connect: false,
        }
    }

    /// Refresh proof-bound authorization material before the first NATS CONNECT.
    pub fn with_refresh_before_connect(mut self) -> Self {
        self.refresh_before_connect = true;
        self
    }
}

/// Attempt one NATS admission with the exact supplied routing material, without refresh.
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub async fn connect_captured_user_admission(
    opts: UserConnectOptions<'_>,
    context_digest: &str,
) -> Result<async_nats::Client, TrellisClientError> {
    let auth = SessionAuth::from_seed_base64url(opts.credentials.session_key_seed_base64url)?;
    let key_pair = std::sync::Arc::new(auth.nkey_pair()?);
    let session_nkey = key_pair.public_key();
    let bootstrap_jwt = opts.credentials.bootstrap_jwt.to_owned();
    let context_digest = context_digest.to_owned();
    ConnectOptions::with_auth_callback(move |nonce| {
        let key_pair = key_pair.clone();
        let session_nkey = session_nkey.clone();
        let bootstrap_jwt = bootstrap_jwt.clone();
        let context_digest = context_digest.clone();
        async move {
            let mut credentials = async_nats::Auth::new();
            credentials.nkey = Some(session_nkey);
            credentials.jwt = Some(bootstrap_jwt);
            credentials.signature =
                Some(key_pair.sign(&nonce).map_err(async_nats::AuthError::new)?);
            credentials.token = Some(
                serde_json::to_string(&NatsConnectToken {
                    format: "trellis.nats-connect-token.v1",
                    context_digest,
                })
                .map_err(async_nats::AuthError::new)?,
            );
            Ok(credentials)
        }
    })
    .custom_inbox_prefix(opts.inbox_prefix)
    .connection_timeout(std::time::Duration::from_millis(opts.timeout_ms))
    .connect(opts.servers)
    .await
    .map_err(|error| TrellisClientError::NatsConnect(error.to_string()))
}

/// Internal authenticated Trellis transport.
pub(crate) struct TrellisClient {
    nats: async_nats::Client,
    auth: Arc<SessionAuth>,
    inbox_prefix: String,
    timeout_ms: u64,
    service_bootstrap_binding: Option<CoreBootstrapBinding>,
    health_heartbeat_task: Option<JoinHandle<()>>,
    authorization_contexts: Option<Arc<AuthorizationContextCache>>,
    authorization_context_refresh_task: Option<JoinHandle<()>>,
    authorization_provider: Option<AuthorizationProviderCache>,
    // RAII/join lifetime: dropping the stop sender ends the provider watch task;
    // the join handle keeps the task addressable.
    _authorization_provider_stop: Option<tokio::sync::watch::Sender<()>>,
    _authorization_provider_task: Option<JoinHandle<()>>,
}

impl TrellisClient {
    pub(crate) fn descriptor_subject(&self, subject: &str) -> String {
        subject.to_string()
    }

    pub(crate) fn nats(&self) -> &async_nats::Client {
        &self.nats
    }

    /// Return the connected NATS client for live transport-boundary tests.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    #[must_use]
    pub fn integration_test_nats(&self) -> async_nats::Client {
        self.nats.clone()
    }

    /// Return the session auth helper used by this client.
    pub fn auth(&self) -> &SessionAuth {
        &self.auth
    }

    /// Return the request timeout configured for this client.
    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    /// Return the resource binding supplied by service HTTP bootstrap, if this is a service client.
    pub fn service_bootstrap_binding(&self) -> Option<&CoreBootstrapBinding> {
        self.service_bootstrap_binding.as_ref()
    }

    /// Connect using service bootstrap, presenting native participant and API artifacts.
    pub async fn connect_service_with_contract(
        opts: ServiceConnectWithContractOptions<'_>,
    ) -> Result<Self, TrellisClientError> {
        let auth = SessionAuth::from_seed_base64url(opts.session_key_seed_base64url)?;
        let participant = serde_json::from_str(opts.participant_json)?;
        let parsed_participant = parse_participant(&participant)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if parsed_participant.id() != opts.participant_id
            || parsed_participant
                .digest()
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
                != opts.participant_digest
        {
            return Err(TrellisClientError::Bootstrap(
                "participant artifact identity mismatch".into(),
            ));
        }
        let mut participant_apis = std::collections::BTreeMap::new();
        let api = serde_json::from_str(opts.api_json)?;
        let referenced = opts
            .referenced_api_artifacts
            .iter()
            .map(|(json, digest)| {
                let artifact = serde_json::from_str(json)?;
                let parsed = trellis_protocol::parse_api(&artifact)
                    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
                let actual_digest = parsed
                    .digest()
                    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
                if actual_digest != *digest {
                    return Err(TrellisClientError::Bootstrap(
                        "referenced API artifact digest mismatch".into(),
                    ));
                }
                participant_apis.insert(parsed.id().to_owned(), parsed);
                Ok(artifact)
            })
            .collect::<Result<Vec<_>, TrellisClientError>>()?;
        let parsed_api = trellis_protocol::parse_api(&api)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if parsed_api
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
            != opts.api_digest
        {
            return Err(TrellisClientError::Bootstrap(
                "owned API artifact digest mismatch".into(),
            ));
        }
        participant_apis.insert(parsed_api.id().to_owned(), parsed_api);
        let resolved = resolve_participant(&parsed_participant, &participant_apis)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if resolved
            .needs()
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
            != opts.participant_needs_digest
        {
            return Err(TrellisClientError::Bootstrap(
                "participant needs digest mismatch".into(),
            ));
        }
        let bootstrap_result =
            fetch_service_bootstrap_with_contract(&auth, &opts, participant, api, referenced)
                .await?;
        connect_bootstrapped_service(auth, &opts, bootstrap_result).await
    }

    /// Connect an activated device using refreshed auth-owned connect info.
    pub async fn connect_device<C>(
        mut opts: DeviceConnectOptions<'_, C>,
    ) -> Result<Self, TrellisClientError> {
        let identity_auth = SessionAuth::from_seed_base64url(opts.identity_seed_base64url)?;
        if identity_auth.session_key != opts.public_identity_key {
            return Err(TrellisClientError::Bootstrap(
                "device public identity key does not match identity seed".into(),
            ));
        }
        let (response, session_key_seed_base64url) = match opts.activation_bootstrap.take() {
            Some(ready) => (ready.response, ready.session_key_seed_base64url),
            None => {
                let (session_key_seed_base64url, _) = crate::auth::generate_session_keypair();
                let session_auth = SessionAuth::from_seed_base64url(&session_key_seed_base64url)?;
                let response = fetch_device_bootstrap(
                    &identity_auth,
                    &session_auth,
                    &opts,
                    None,
                    DeviceBootstrapProofOverrides::default(),
                )
                .await?;
                (response, session_key_seed_base64url)
            }
        };
        let session_auth = SessionAuth::from_seed_base64url(&session_key_seed_base64url)?;
        if response.state != "ready" {
            return Err(TrellisClientError::Bootstrap(format!(
                "unexpected device bootstrap state '{}'",
                response.state
            )));
        }
        let session = response.session.ok_or_else(|| {
            TrellisClientError::Bootstrap("missing device bootstrap session".into())
        })?;
        let nats_credential = response.nats.ok_or_else(|| {
            TrellisClientError::Bootstrap("missing device NATS credential".into())
        })?;
        let authorization_context = response.authorization_context.ok_or_else(|| {
            TrellisClientError::Bootstrap("missing device authorization context bundle".into())
        })?;
        let authorization: ServiceBootstrapAuthorization =
            serde_json::from_value(response.authorization.ok_or_else(|| {
                TrellisClientError::Bootstrap("missing device authorization evidence".into())
            })?)?;
        if authorization.participant_id != opts.contract.participant_id
            || authorization.participant_artifact_digest != opts.contract.participant_digest
        {
            return Err(TrellisClientError::Bootstrap(
                "device authorization participant mismatch".into(),
            ));
        }
        let servers = nats_credential.servers.join(",");
        let mut connected = Self::connect_user(UserConnectOptions::new(
            opts.trellis_url,
            &servers,
            &session.inbox_prefix,
            opts.timeout_ms,
            UserSessionCredentials {
                bootstrap_jwt: &nats_credential.jwt,
                session_id: &session.session_id,
                session_key_seed_base64url: &session_key_seed_base64url,
                participant_digest: opts.contract.participant_digest,
            },
            UserAuthorizationContext {
                bundle: authorization_context,
                binding: format!("device:{}", opts.public_identity_key),
                store: opts.authorization_context_store,
            },
        ))
        .await?;

        let health_heartbeat_config = HealthHeartbeatConfig {
            session_key: session_auth.session_key,
            service_name: opts.contract.participant_id.to_owned(),
            kind: HealthHeartbeatServiceKind::Device,
            deployment_id: opts.deployment_id.to_owned(),
            instance_id: opts.instance_id.to_owned(),
            contract_id: opts.contract.participant_id.to_owned(),
            contract_digest: opts.contract.participant_digest.to_owned(),
            started_at: now_rfc3339(),
            publish_interval_ms: HEALTH_HEARTBEAT_INTERVAL_MS,
        };
        if let Err(error) =
            publish_health_heartbeat(&connected.nats, opts.timeout_ms, &health_heartbeat_config)
                .await
        {
            tracing::warn!(%error, "failed to publish initial health heartbeat");
        }
        connected.health_heartbeat_task = Some(spawn_health_heartbeat_task(
            connected.nats.clone(),
            opts.timeout_ms,
            health_heartbeat_config,
        ));
        Ok(connected)
    }

    /// Connect using reconnect-safe session-key runtime auth for one contract digest.
    pub async fn connect_user(opts: UserConnectOptions<'_>) -> Result<Self, TrellisClientError> {
        let auth = SessionAuth::from_seed_base64url(opts.credentials.session_key_seed_base64url)?;
        let inbox_prefix = opts.inbox_prefix;
        let callback_auth = std::sync::Arc::new(SessionAuth::from_seed_base64url(
            opts.credentials.session_key_seed_base64url,
        )?);
        let key_pair = std::sync::Arc::new(callback_auth.nkey_pair()?);
        let session_nkey = key_pair.public_key();
        let bootstrap_jwt = opts.credentials.bootstrap_jwt.to_owned();
        let authorization_contexts = AuthorizationContextCache::new(
            opts.trellis_url,
            opts.authorization.binding,
            opts.authorization.store,
        )?;
        let authorization_contexts = Arc::new(authorization_contexts);
        let now = now_context_seconds()?;
        if !authorization_contexts.restore(now).await?
            && !authorization_contexts
                .install_recoverable(
                    opts.authorization.bundle,
                    AuthorizationRoutingMaterial {
                        bootstrap_jwt: bootstrap_jwt.clone(),
                        bootstrap_jwt_expires_at: jwt_expiry(&bootstrap_jwt)?,
                    },
                    now,
                )
                .await?
        {
            authorization_contexts.refresh(&auth).await?;
        }
        let (context_session_id, _, context_participant_digest, _, _) =
            authorization_contexts.refresh_evidence()?;
        if context_session_id != opts.credentials.session_id
            || context_participant_digest != opts.credentials.participant_digest
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization context binding mismatch".into(),
            ));
        }
        let callback_authorization_contexts = authorization_contexts.clone();
        let reauth = Arc::new(AtomicBool::new(opts.refresh_before_connect));

        let nats = ConnectOptions::with_auth_callback(move |nonce| {
            let auth = callback_auth.clone();
            let key_pair = key_pair.clone();
            let session_nkey = session_nkey.clone();
            let authorization_contexts = callback_authorization_contexts.clone();
            let reauth = reauth.clone();
            async move {
                if reauth.swap(true, Ordering::AcqRel)
                    || authorization_contexts.routing_jwt().is_err()
                    || authorization_contexts.context_digest().is_err()
                {
                    authorization_contexts
                        .refresh(&auth)
                        .await
                        .map_err(async_nats::AuthError::new)?;
                }
                let bootstrap_jwt = authorization_contexts
                    .routing_jwt()
                    .map_err(async_nats::AuthError::new)?;
                let context_digest = authorization_contexts
                    .context_digest()
                    .map_err(async_nats::AuthError::new)?;
                let mut credentials = async_nats::Auth::new();
                credentials.nkey = Some(session_nkey);
                credentials.jwt = Some(bootstrap_jwt);
                credentials.signature =
                    Some(key_pair.sign(&nonce).map_err(async_nats::AuthError::new)?);
                credentials.token = Some(
                    serde_json::to_string(&NatsConnectToken {
                        format: "trellis.nats-connect-token.v1",
                        context_digest,
                    })
                    .map_err(async_nats::AuthError::new)?,
                );
                Ok(credentials)
            }
        })
        .custom_inbox_prefix(inbox_prefix)
        .connect(opts.servers)
        .await
        .map_err(|error| TrellisClientError::NatsConnect(error.to_string()))?;

        let provider =
            attach_authorization_provider(nats.clone(), authorization_contexts.clone()).await?;
        let auth = Arc::new(auth);
        let authorization_context_refresh_task = Some(spawn_authorization_context_refresh_task(
            authorization_contexts.clone(),
            auth.clone(),
            nats.clone(),
        ));
        Ok(Self {
            nats,
            auth,
            inbox_prefix: opts.inbox_prefix.to_owned(),
            timeout_ms: opts.timeout_ms,
            service_bootstrap_binding: None,
            health_heartbeat_task: None,
            authorization_contexts: Some(authorization_contexts),
            authorization_context_refresh_task,
            authorization_provider: Some(provider.provider),
            _authorization_provider_stop: Some(provider.stop),
            _authorization_provider_task: Some(provider.task),
        })
    }

    /// Return the signed authorization context used by this connection.
    pub fn authorization_context(
        &self,
    ) -> Result<Option<AuthorizationContextBundle>, TrellisClientError> {
        self.authorization_contexts
            .as_ref()
            .map(|cache| cache.bundle())
            .transpose()
    }

    /// Return the in-process provider cache used by local verification.
    pub(crate) fn authorization_context_cache(
        &self,
    ) -> Result<AuthorizationProviderCache, TrellisClientError> {
        self.authorization_provider.clone().ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization provider unavailable".into())
        })
    }

    /// Return the local provider cache for live I/O and readiness assertions.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn integration_test_authorization_provider(
        &self,
    ) -> Result<AuthorizationProviderCache, TrellisClientError> {
        self.authorization_context_cache()
    }

    /// Return the active authorization context digest for live integration synchronization.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn integration_test_authorization_context_digest(
        &self,
    ) -> Result<String, TrellisClientError> {
        self.authorization_context_digest()
    }

    /// Refresh and verify the current authorization context immediately.
    pub async fn refresh_authorization_context(
        &self,
    ) -> Result<AuthorizationContextBundle, TrellisClientError> {
        let contexts = self.authorization_contexts.as_ref().ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization context unavailable".into())
        })?;
        contexts.refresh(&self.auth).await?;
        contexts.bundle()
    }

    async fn request(
        &self,
        subject: &str,
        payload: Bytes,
    ) -> Result<async_nats::Message, TrellisClientError> {
        // Create the exact reply inbox before signing so the proof binds the
        // reply subject the response arrives on.
        let reply = self.nats.new_inbox();
        let headers = self.signed_headers(subject, &reply, &payload)?;
        let request = async_nats::Request::new()
            .inbox(reply)
            .headers(headers)
            .payload(payload);

        let future = self.nats.send_request(subject.to_string(), request);
        let message = timeout(std::time::Duration::from_millis(self.timeout_ms), future)
            .await
            .map_err(|_| TrellisClientError::Timeout)?
            .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;
        Ok(message)
    }

    pub(crate) fn signed_headers(
        &self,
        subject: &str,
        reply: &str,
        payload: &[u8],
    ) -> Result<HeaderMap, TrellisClientError> {
        let context_digest = self.authorization_context_digest()?;
        signed_headers(&self.auth, &context_digest, subject, reply, payload)
    }

    pub(crate) fn authorization_context_digest(&self) -> Result<String, TrellisClientError> {
        let contexts = self.authorization_contexts.as_ref().ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization context unavailable".into())
        })?;
        contexts.context_digest()
    }

    async fn request_json(&self, subject: &str, body: Value) -> Result<Value, TrellisClientError> {
        let payload = Bytes::from(serde_json::to_vec(&body)?);
        let message = self.request(subject, payload).await?;

        decode_json_message(message)
    }

    /// Call a raw subject with a JSON value payload.
    pub async fn request_json_value(
        &self,
        subject: &str,
        body: &Value,
    ) -> Result<Value, TrellisClientError> {
        self.request_json(subject, body.clone()).await
    }

    /// Call one descriptor-backed RPC.
    pub async fn call<D>(&self, input: &D::Input) -> Result<D::Output, TrellisClientError>
    where
        D: RpcDescriptor,
    {
        let value = serde_json::to_value(input)?;
        let response = self
            .request_json(&self.descriptor_subject(D::SUBJECT), value)
            .await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Call one descriptor-backed RPC and decode contract-declared errors.
    pub async fn call_typed<D, E>(&self, input: &D::Input) -> Result<D::Output, CallError<E>>
    where
        D: RpcDescriptor,
        E: crate::client::DeclaredError,
    {
        let input = serde_json::to_value(input).map_err(|error| {
            CallError::Protocol(crate::client::ProtocolError::new(error.to_string()))
        })?;
        validate_caller_input::<E>(D::INPUT_SCHEMA_JSON, &input)?;
        let output = self
            .request_json(&self.descriptor_subject(D::SUBJECT), input)
            .await
            .map_err(CallError::from_client)?;
        crate::service::validate_input_schema(D::OUTPUT_SCHEMA_JSON, &output).map_err(|error| {
            CallError::Protocol(crate::client::ProtocolError::new(format!(
                "remote response violated `{}` output schema: {error}",
                D::KEY
            )))
        })?;
        serde_json::from_value(output.clone()).map_err(|error| {
            CallError::Protocol(crate::client::ProtocolError::new(format!(
                "{error}; output={output}"
            )))
        })
    }

    /// Publish one descriptor-backed event.
    pub async fn publish<D>(&self, event: &D::Event) -> Result<(), TrellisClientError>
    where
        D: EventDescriptor,
    {
        let prepared = prepare_event::<D>(event)?.with_subject(self.descriptor_subject(D::SUBJECT));
        self.publish_prepared(&prepared).await
    }

    /// Publish an event that was already prepared, preserving its subject, payload, and message id.
    pub async fn publish_prepared(
        &self,
        event: &PreparedTrellisEvent,
    ) -> Result<(), TrellisClientError> {
        let event = event
            .clone()
            .with_subject(self.descriptor_subject(event.subject()));
        let context_digest = self.authorization_context_digest()?;
        publish_prepared_event(
            &self.nats,
            &self.auth,
            &context_digest,
            self.timeout_ms,
            &event,
        )
        .await
    }

    /// Subscribe to one descriptor-backed event subject from the default JetStream event stream.
    pub async fn subscribe<D>(
        &self,
    ) -> Result<BoxStream<'static, Result<D::Event, TrellisClientError>>, TrellisClientError>
    where
        D: EventDescriptor,
        D::Event: Send + 'static,
    {
        self.subscribe_with_options::<D>(EventSubscribeOptions::default())
            .await
    }

    /// Subscribe to one descriptor-backed event subject with explicit subscription options.
    pub async fn subscribe_with_options<D>(
        &self,
        options: EventSubscribeOptions,
    ) -> Result<BoxStream<'static, Result<D::Event, TrellisClientError>>, TrellisClientError>
    where
        D: EventDescriptor,
        D::Event: Send + 'static,
    {
        if options.mode == EventSubscriptionMode::Ephemeral {
            return self.subscribe_live::<D>().await;
        }

        let messages = self.subscribe_messages::<D>(options).await?;
        let stream = stream::try_unfold(messages, |mut messages| async move {
            match messages.next().await {
                Some(Ok(event_message)) => {
                    let event = event_message.decode()?;
                    event_message.ack().await?;
                    Ok(Some((event, messages)))
                }
                Some(Err(error)) => Err(error),
                None => Ok(None),
            }
        });

        Ok(Box::pin(stream) as BoxStream<'static, Result<D::Event, TrellisClientError>>)
    }

    async fn subscribe_live<D>(
        &self,
    ) -> Result<BoxStream<'static, Result<D::Event, TrellisClientError>>, TrellisClientError>
    where
        D: EventDescriptor,
        D::Event: Send + 'static,
    {
        let subscriber = timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            self.nats
                .subscribe(self.descriptor_subject(D::SUBSCRIBE_SUBJECT)),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

        let stream = stream::try_unfold(subscriber, |mut subscriber| async move {
            match subscriber.next().await {
                Some(message) => {
                    let value: Value = serde_json::from_slice(&message.payload)?;
                    crate::service::validate_input_schema(D::EVENT_SCHEMA_JSON, &value).map_err(
                        |error| {
                            TrellisClientError::EventSubscriptionProtocol(format!(
                                "event `{}` violated its contract schema: {error}",
                                D::KEY
                            ))
                        },
                    )?;
                    let event: D::Event = serde_json::from_value(value)?;
                    Ok(Some((event, subscriber)))
                }
                None => Ok(None),
            }
        });

        Ok(Box::pin(stream) as BoxStream<'static, Result<D::Event, TrellisClientError>>)
    }

    /// Subscribe to descriptor-backed event messages with explicit ack/nak/term control.
    pub async fn subscribe_messages<D>(
        &self,
        options: EventSubscribeOptions,
    ) -> Result<
        BoxStream<'static, Result<EventMessage<D::Event>, TrellisClientError>>,
        TrellisClientError,
    >
    where
        D: EventDescriptor,
        D::Event: Send + 'static,
    {
        let jetstream = jetstream::new(self.nats.clone());
        let stream_name = options.stream.as_deref().unwrap_or(DEFAULT_EVENT_STREAM);
        let event_stream = timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            jetstream.get_stream_no_info(stream_name),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

        if options.mode == EventSubscriptionMode::Durable && options.durable_name.is_none() {
            return Err(TrellisClientError::EventSubscriptionProtocol(
                "durable event subscriptions require a pre-provisioned durable name".to_string(),
            ));
        }

        let config = event_consumer_config(&options, self.descriptor_subject(D::SUBSCRIBE_SUBJECT));
        let durable_name = config.durable_name.clone();
        let consumer = match durable_name.as_deref() {
            Some(name) => timeout(
                std::time::Duration::from_millis(self.timeout_ms),
                event_stream.get_consumer(name),
            )
            .await
            .map_err(|_| TrellisClientError::Timeout)?
            .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?,
            None => timeout(
                std::time::Duration::from_millis(self.timeout_ms),
                event_stream.create_consumer(config),
            )
            .await
            .map_err(|_| TrellisClientError::Timeout)?
            .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?,
        };

        let messages = timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            consumer.messages(),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

        let stream = stream::try_unfold(messages, |mut messages| async move {
            match messages.next().await {
                Some(Ok(message)) => {
                    let event_message = EventMessage {
                        message,
                        _event: PhantomData,
                    };
                    Ok(Some((event_message, messages)))
                }
                Some(Err(error)) => Err(TrellisClientError::NatsRequest(error.to_string())),
                None => Ok(None),
            }
        });

        Ok(Box::pin(stream)
            as BoxStream<
                'static,
                Result<EventMessage<D::Event>, TrellisClientError>,
            >)
    }

    /// Subscribe to one descriptor-backed feed and decode event payloads.
    pub async fn feed<D>(
        &self,
        input: &D::Input,
    ) -> Result<BoxStream<'static, Result<D::Event, TrellisClientError>>, TrellisClientError>
    where
        D: FeedDescriptor,
        D::Event: Send + 'static,
    {
        let input = serde_json::to_value(input)?;
        crate::service::validate_input_schema(D::INPUT_SCHEMA_JSON, &input).map_err(|error| {
            TrellisClientError::FeedProtocol(format!(
                "feed `{}` input violated its contract schema: {error}",
                D::KEY
            ))
        })?;
        let payload = Bytes::from(serde_json::to_vec(&input)?);
        let subject = self.descriptor_subject(D::SUBJECT);
        let context_digest = self.authorization_context_digest()?;
        let inbox = format!(
            "{}.{}",
            self.inbox_prefix,
            FEED_INBOX_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let headers = signed_headers(&self.auth, &context_digest, &subject, &inbox, &payload)?;
        let cancel_payload = Bytes::from(serde_json::to_vec(&serde_json::json!({
            "_trellisFeedCancel": inbox.clone(),
        }))?);
        let cancel = FeedCancelGuard {
            runtime: tokio::runtime::Handle::current(),
            nats: self.nats.clone(),
            auth: Arc::clone(&self.auth),
            context_digest,
            subject: subject.clone(),
            reply: inbox.clone(),
            payload: cancel_payload,
        };
        let mut subscriber = timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            self.nats.subscribe(inbox.clone()),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

        timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            self.nats
                .publish_with_reply_and_headers(subject, inbox, headers, payload),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

        let first = timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            subscriber.next(),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .ok_or(TrellisClientError::Timeout)?;

        let first_event = decode_feed_message::<D>(first)?;
        let stream = stream::try_unfold(
            (subscriber, first_event, cancel),
            |(mut subscriber, first_event, cancel)| async move {
                if let Some(event) = first_event {
                    return Ok(Some((event, (subscriber, None, cancel))));
                }

                match subscriber.next().await {
                    Some(message) => {
                        let event = decode_feed_message::<D>(message)?.ok_or_else(|| {
                            TrellisClientError::NatsRequest(
                                "feed emitted duplicate ready acknowledgement".to_string(),
                            )
                        })?;
                        Ok(Some((event, (subscriber, None, cancel))))
                    }
                    None => Ok(None),
                }
            },
        );

        Ok(Box::pin(stream) as BoxStream<'static, Result<D::Event, TrellisClientError>>)
    }

    /// Download the bytes exposed by a receive transfer grant.
    pub async fn download_transfer(
        &self,
        grant: &DownloadTransferGrant,
    ) -> Result<Vec<u8>, TrellisClientError> {
        get_download_grant(self, grant).await
    }

    /// Stream the bytes exposed by a receive transfer grant into `writer`.
    pub async fn download_transfer_into<W>(
        &self,
        grant: &DownloadTransferGrant,
        writer: &mut W,
    ) -> Result<FileInfo, TrellisClientError>
    where
        W: tokio::io::AsyncWrite + Unpin + Send + ?Sized,
    {
        crate::client::transfer::get_download_grant_into(self, grant, writer).await
    }

    /// Stream a receive transfer into `writer` with authenticated cancellation.
    pub async fn download_transfer_into_with_cancel<W>(
        &self,
        grant: &DownloadTransferGrant,
        writer: &mut W,
        cancellation: &crate::client::TransferCancellation,
    ) -> Result<FileInfo, TrellisClientError>
    where
        W: tokio::io::AsyncWrite + Unpin + Send + ?Sized,
    {
        crate::client::transfer::get_download_grant_into_with_cancel(
            self,
            grant,
            writer,
            Some(cancellation),
        )
        .await
    }
}

impl Drop for TrellisClient {
    fn drop(&mut self) {
        if let Some(task) = self.health_heartbeat_task.take() {
            task.abort();
        }
        if let Some(task) = self.authorization_context_refresh_task.take() {
            task.abort();
        }
        if let Some(stop) = self._authorization_provider_stop.take() {
            let _ = stop.send(());
        }
        if let Some(task) = self._authorization_provider_task.take() {
            task.abort();
        }
    }
}

impl OperationTransport for TrellisClient {
    fn descriptor_subject(&self, subject: &str) -> String {
        self.descriptor_subject(subject)
    }

    async fn request_json_value(
        &self,
        subject: String,
        body: Value,
    ) -> Result<Value, TrellisClientError> {
        TrellisClient::request_json_value(self, &subject, &body).await
    }

    async fn watch_json_value<'a>(
        &'a self,
        subject: String,
        body: Value,
    ) -> Result<BoxStream<'a, Result<Value, TrellisClientError>>, TrellisClientError> {
        let payload = Bytes::from(serde_json::to_vec(&body)?);
        let inbox = self.nats.new_inbox();
        let headers = self.signed_headers(&subject, &inbox, &payload)?;
        let mut subscriber = timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            self.nats.subscribe(inbox.clone()),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

        timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            self.nats
                .publish_with_reply_and_headers(subject, inbox, headers, payload),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .map_err(|error| TrellisClientError::NatsRequest(error.to_string()))?;

        let first = timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            subscriber.next(),
        )
        .await
        .map_err(|_| TrellisClientError::Timeout)?
        .ok_or_else(|| TrellisClientError::NatsRequest("operation watch closed".to_owned()))?;
        let first = decode_watch_message(first)?;
        let first_terminal = is_terminal_event(&first);

        let stream = stream::once(async move { Ok(first) }).chain(stream::try_unfold(
            (subscriber, first_terminal),
            |(mut subscriber, done)| async move {
                if done {
                    return Ok(None);
                }

                match subscriber.next().await {
                    Some(message) => {
                        let event = decode_watch_message(message)?;
                        let terminal = is_terminal_event(&event);
                        Ok(Some((event, (subscriber, terminal))))
                    }
                    None => Ok(None),
                }
            },
        ));

        Ok(Box::pin(stream) as BoxStream<'a, Result<Value, TrellisClientError>>)
    }

    async fn put_upload_transfer(
        &self,
        grant: UploadTransferGrant,
        body: Vec<u8>,
    ) -> Result<FileInfo, TrellisClientError> {
        put_upload_grant(self, &grant, body).await
    }

    async fn put_upload_transfer_from<'a, R>(
        &'a self,
        grant: UploadTransferGrant,
        reader: &'a mut R,
        expected_size: Option<u64>,
    ) -> Result<FileInfo, TrellisClientError>
    where
        R: tokio::io::AsyncRead + Unpin + Send + ?Sized + 'a,
    {
        crate::client::transfer::put_upload_grant_from(self, &grant, reader, expected_size).await
    }

    async fn put_upload_transfer_from_with_cancel<'a, R>(
        &'a self,
        grant: UploadTransferGrant,
        reader: &'a mut R,
        expected_size: Option<u64>,
        cancellation: &'a crate::client::TransferCancellation,
    ) -> Result<FileInfo, TrellisClientError>
    where
        R: tokio::io::AsyncRead + Unpin + Send + ?Sized + 'a,
    {
        crate::client::transfer::put_upload_grant_from_with_cancel(
            self,
            &grant,
            reader,
            expected_size,
            Some(cancellation),
        )
        .await
    }
}

fn validate_caller_input<E>(schema_json: &str, value: &Value) -> Result<(), CallError<E>>
where
    E: crate::client::DeclaredError,
{
    match crate::service::validate_input_schema(schema_json, value) {
        Ok(()) => Ok(()),
        Err(crate::service::ServerError::Validation { issues }) => Err(CallError::Validation(
            Box::new(crate::client::ValidationFailure::Validation(
                crate::client::ValidationErrorPayload {
                    id: "local".to_string(),
                    error_type: "ValidationError".to_string(),
                    message: "Input validation failed".to_string(),
                    issues: (*issues)
                        .into_iter()
                        .map(|issue| crate::client::ValidationIssue {
                            path: issue.path,
                            message: issue.message,
                        })
                        .collect(),
                    context: None,
                    trace_id: None,
                },
            )),
        )),
        Err(crate::service::ServerError::SchemaValidation { issues }) => Err(
            CallError::Validation(Box::new(crate::client::ValidationFailure::Schema(
                crate::client::SchemaValidationErrorPayload {
                    id: "local".to_string(),
                    error_type: "SchemaValidationError".to_string(),
                    message: "Input validation failed".to_string(),
                    issues: (*issues)
                        .into_iter()
                        .map(|issue| crate::client::SchemaValidationIssue {
                            path: issue.path,
                            schema_path: issue.schema_path,
                            keyword: issue.keyword,
                            code: issue.code,
                            message: issue.message,
                            label: issue.label,
                            note: issue.note,
                            i18n_key: issue.i18n_key,
                            severity: issue.severity,
                            params: issue.params,
                        })
                        .collect(),
                    context: None,
                    trace_id: None,
                },
            ))),
        ),
        Err(error) => Err(CallError::Protocol(crate::client::ProtocolError::new(
            error.to_string(),
        ))),
    }
}

fn decode_json_message(message: async_nats::Message) -> Result<Value, TrellisClientError> {
    if let Some(headers) = &message.headers {
        if headers
            .get("status")
            .is_some_and(|status| status.as_str() == "error")
        {
            return Err(TrellisClientError::RpcError(
                RpcErrorPayload::from_json_slice(&message.payload)?,
            ));
        }
    }

    Ok(serde_json::from_slice(&message.payload)?)
}

fn decode_watch_message(message: async_nats::Message) -> Result<Value, TrellisClientError> {
    decode_json_message(message)
}

fn decode_feed_message<D>(
    message: async_nats::Message,
) -> Result<Option<D::Event>, TrellisClientError>
where
    D: FeedDescriptor,
{
    if message.status == Some(async_nats::StatusCode::NO_RESPONDERS) {
        return Err(TrellisClientError::NatsRequest(
            "no responders for feed request".to_string(),
        ));
    }
    decode_feed_frame::<D>(message.headers.as_ref(), &message.payload)
}

fn decode_feed_frame<D>(
    headers: Option<&HeaderMap>,
    payload: &[u8],
) -> Result<Option<D::Event>, TrellisClientError>
where
    D: FeedDescriptor,
{
    if let Some(headers) = headers {
        if headers
            .get("status")
            .is_some_and(|status| status.as_str() == "error")
        {
            return Err(TrellisClientError::RpcError(
                RpcErrorPayload::from_json_slice(payload)?,
            ));
        }
        if headers
            .get("feed-status")
            .is_some_and(|status| status.as_str() == "ready")
        {
            return Ok(None);
        }
    }

    let value: Value = serde_json::from_slice(payload)?;
    crate::service::validate_input_schema(D::EVENT_SCHEMA_JSON, &value).map_err(|error| {
        TrellisClientError::FeedProtocol(format!(
            "feed `{}` emitted an invalid event: {error}",
            D::KEY
        ))
    })?;
    Ok(Some(serde_json::from_value(value)?))
}

fn is_terminal_event(event: &Value) -> bool {
    matches!(
        event.get("type").and_then(Value::as_str),
        Some("completed" | "failed" | "cancelled")
    )
}

fn event_consumer_config(
    options: &EventSubscribeOptions,
    filter_subject: String,
) -> consumer::pull::Config {
    consumer::pull::Config {
        durable_name: match options.mode {
            EventSubscriptionMode::Durable => options.durable_name.clone(),
            EventSubscriptionMode::Ephemeral => None,
        },
        deliver_policy: match options.replay {
            EventReplayPolicy::All => consumer::DeliverPolicy::All,
            EventReplayPolicy::New => consumer::DeliverPolicy::New,
        },
        ack_policy: consumer::AckPolicy::Explicit,
        filter_subject,
        ..Default::default()
    }
}
