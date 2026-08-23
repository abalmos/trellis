use std::fmt;
use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

use crate::client::{fetch_device_activation, DeviceConnectOptions, TrellisClientError};

type HmacSha256 = Hmac<Sha256>;

const DEVICE_CONFIRMATION_DOMAIN: &str = "trellis-device-confirm/v1";
const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

struct DeviceActivationAttempt {
    session_key_seed_base64url: String,
    session_auth: crate::client::SessionAuth,
}

impl DeviceActivationAttempt {
    fn new() -> Result<Self, DeviceActivationError> {
        let (session_key_seed_base64url, _) = crate::auth::generate_session_keypair();
        Self::from_seed(session_key_seed_base64url)
    }

    fn from_seed(session_key_seed_base64url: String) -> Result<Self, DeviceActivationError> {
        let session_auth =
            crate::client::SessionAuth::from_seed_base64url(&session_key_seed_base64url)?;
        Ok(Self {
            session_key_seed_base64url,
            session_auth,
        })
    }
}

/// Inputs retained across proof-bound device activation attempts.
pub struct DeviceActivationOptions<'a, C> {
    connect: DeviceConnectOptions<'a, C>,
    activation_key_base64url: &'a str,
    nonce: String,
}

impl<'a, C> DeviceActivationOptions<'a, C> {
    /// Create activation options for an exact provisioned device and participant.
    pub fn new(connect: DeviceConnectOptions<'a, C>, activation_key_base64url: &'a str) -> Self {
        Self {
            nonce: format!("{}:{}", connect.public_identity_key(), ulid::Ulid::new()),
            connect,
            activation_key_base64url,
        }
    }

    /// Replace the generated nonce, primarily for durable local activation state.
    pub fn with_nonce(mut self, nonce: impl Into<String>) -> Self {
        self.nonce = nonce.into();
        self
    }

    /// Attach ready evidence to the exact options that produced it.
    ///
    /// This consumes both values and returns [`DeviceActivationError::SessionOriginMismatch`]
    /// before connection when device identity, deployment, instance, contract, activation, or
    /// successful session-key evidence differs from the originating activation attempt.
    pub fn into_connect_options(
        self,
        session: DeviceActivationSession,
    ) -> Result<DeviceConnectOptions<'a, C>, DeviceActivationError> {
        if self.connect.activation_origin_digest(
            self.activation_key_base64url,
            &self.nonce,
            &session.session_key_seed_base64url,
        ) != session.origin_digest
        {
            return Err(DeviceActivationError::SessionOriginMismatch);
        }
        Ok(self
            .connect
            .activation_bootstrap(*session.bootstrap, session.session_key_seed_base64url))
    }
}

/// Current result of a device activation bootstrap attempt.
#[derive(Debug)]
pub enum DeviceActivationStatus {
    /// User or administrative activation work remains pending.
    Pending(DeviceActivationPending),
    /// Activation is complete; a subsequent pure device connection may proceed.
    Ready(DeviceActivationSession),
}

/// Durable local evidence needed to resume one pending activation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceActivationPending {
    /// Server-owned activation review identifier.
    pub review_id: String,
    /// Portal URL for the activating user.
    pub activation_url: String,
    /// Device-local challenge nonce retained across attempts.
    pub nonce: String,
    /// Human confirmation code derived from the device-local activation key.
    pub confirmation_code: String,
    /// Server timestamp after which this review is no longer valid.
    pub expires_at: i64,
    /// Server time observed with the pending review.
    pub server_now: i64,
    /// Server-suggested polling delay.
    pub retry_after_ms: u64,
}

/// Proof that `/bootstrap/device` reached ready state.
///
/// This value retains the successful attempt's private session seed and bootstrap credentials
/// until [`DeviceActivationOptions::into_connect_options`] consumes it. Its [`Debug`] output is
/// deliberately redacted and never exposes either secret.
pub struct DeviceActivationSession {
    /// Server time observed in the ready bootstrap response.
    pub server_now: i64,
    pub(crate) bootstrap: Box<crate::client::ServiceBootstrapResponse>,
    session_key_seed_base64url: String,
    origin_digest: [u8; 32],
}

impl fmt::Debug for DeviceActivationSession {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeviceActivationSession")
            .field("server_now", &self.server_now)
            .finish_non_exhaustive()
    }
}

/// Typed device activation failure.
#[derive(Debug, thiserror::Error)]
pub enum DeviceActivationError {
    /// Device or deployment was disabled.
    #[error("device activation is disabled")]
    Disabled,
    /// The pending activation review expired.
    #[error("device activation expired")]
    Expired,
    /// Device activation was rejected.
    #[error("device activation was rejected")]
    Rejected,
    /// The bounded activation wait elapsed.
    #[error("timed out waiting for device activation")]
    TimedOut,
    /// Runtime returned a state not valid for device activation.
    #[error("unexpected device activation state '{0}'")]
    UnexpectedState(String),
    /// Proof construction, HTTP, or response decoding failed.
    #[error(transparent)]
    Client(#[from] TrellisClientError),
    /// Device-local activation evidence was malformed.
    #[error("invalid device activation evidence: {0}")]
    InvalidEvidence(String),
    /// Ready evidence was paired with different device activation options.
    #[error("device activation session does not match its originating options")]
    SessionOriginMismatch,
}

/// Derive the eight-character confirmation code for one activation nonce.
pub fn derive_device_confirmation_code(
    activation_key_base64url: &str,
    public_identity_key: &str,
    nonce: &str,
) -> Result<String, DeviceActivationError> {
    let activation_key = URL_SAFE_NO_PAD
        .decode(activation_key_base64url)
        .map_err(|error| DeviceActivationError::InvalidEvidence(error.to_string()))?;
    let mut mac = HmacSha256::new_from_slice(&activation_key)
        .map_err(|error| DeviceActivationError::InvalidEvidence(error.to_string()))?;
    mac.update(DEVICE_CONFIRMATION_DOMAIN.as_bytes());
    mac.update(public_identity_key.as_bytes());
    mac.update(nonce.as_bytes());
    let bytes = mac.finalize().into_bytes();
    let mut value = 0u64;
    for byte in &bytes[..5] {
        value = (value << 8) | u64::from(*byte);
    }
    Ok((0..8)
        .rev()
        .map(|shift| CROCKFORD_ALPHABET[((value >> (shift * 5)) & 31) as usize] as char)
        .collect())
}

/// Submit one fresh proof-bound `/bootstrap/device` activation request.
pub async fn check_device_activation<C>(
    options: &DeviceActivationOptions<'_, C>,
) -> Result<DeviceActivationStatus, DeviceActivationError> {
    check_device_activation_with_attempt(options, DeviceActivationAttempt::new()?).await
}

async fn check_device_activation_with_attempt<C>(
    options: &DeviceActivationOptions<'_, C>,
    attempt: DeviceActivationAttempt,
) -> Result<DeviceActivationStatus, DeviceActivationError> {
    let confirmation_code = derive_device_confirmation_code(
        options.activation_key_base64url,
        options.connect.public_identity_key(),
        &options.nonce,
    )?;
    let challenge_digest = URL_SAFE_NO_PAD.encode(Sha256::digest(options.nonce.as_bytes()));
    let response = match fetch_device_activation(
        &options.connect,
        &attempt.session_auth,
        &challenge_digest,
        &confirmation_code,
    )
    .await
    {
        Ok(response) => response,
        Err(TrellisClientError::BootstrapHttp { status: 403, .. }) => {
            return Err(DeviceActivationError::Disabled)
        }
        Err(error) => return Err(error.into()),
    };
    activation_status(
        options,
        confirmation_code,
        response,
        attempt.session_key_seed_base64url,
    )
}

fn activation_status<C>(
    options: &DeviceActivationOptions<'_, C>,
    confirmation_code: String,
    response: crate::client::ServiceBootstrapResponse,
    session_key_seed_base64url: String,
) -> Result<DeviceActivationStatus, DeviceActivationError> {
    match response.state.as_str() {
        "ready" => Ok(DeviceActivationStatus::Ready(DeviceActivationSession {
            server_now: response.server_now,
            bootstrap: Box::new(response),
            origin_digest: options.connect.activation_origin_digest(
                options.activation_key_base64url,
                &options.nonce,
                &session_key_seed_base64url,
            ),
            session_key_seed_base64url,
        })),
        "activation_pending" => {
            let activation = response.activation.ok_or_else(|| {
                DeviceActivationError::UnexpectedState("activation_pending without review".into())
            })?;
            if activation.state != "pending" {
                return Err(DeviceActivationError::UnexpectedState(activation.state));
            }
            Ok(DeviceActivationStatus::Pending(DeviceActivationPending {
                review_id: activation.review_id,
                activation_url: activation.activation_url,
                nonce: options.nonce.clone(),
                confirmation_code,
                expires_at: activation.expires_at,
                server_now: response.server_now,
                retry_after_ms: activation.retry_after_ms,
            }))
        }
        "disabled" => Err(DeviceActivationError::Disabled),
        "authority_rejected" => Err(DeviceActivationError::Rejected),
        state => Err(DeviceActivationError::UnexpectedState(state.to_owned())),
    }
}

/// Submit activation with controlled proof properties for live integration tests.
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub async fn check_device_activation_with_test_proof<C>(
    options: &DeviceActivationOptions<'_, C>,
    issued_at_ms: Option<i64>,
    corrupt_signature: bool,
) -> Result<DeviceActivationStatus, DeviceActivationError> {
    let attempt = DeviceActivationAttempt::new()?;
    let confirmation_code = derive_device_confirmation_code(
        options.activation_key_base64url,
        options.connect.public_identity_key(),
        &options.nonce,
    )?;
    let challenge_digest = URL_SAFE_NO_PAD.encode(Sha256::digest(options.nonce.as_bytes()));
    let response = crate::client::fetch_device_activation_with_test_proof(
        &options.connect,
        &attempt.session_auth,
        &challenge_digest,
        &confirmation_code,
        crate::client::DeviceBootstrapProofOverrides {
            issued_at_ms,
            corrupt_signature,
        },
    )
    .await?;
    activation_status(
        options,
        confirmation_code,
        response,
        attempt.session_key_seed_base64url,
    )
}

/// Poll current device activation with fresh request identities and proofs.
pub async fn wait_for_device_activation<C>(
    options: &DeviceActivationOptions<'_, C>,
    pending: &DeviceActivationPending,
    timeout: Duration,
) -> Result<DeviceActivationSession, DeviceActivationError> {
    if options.nonce != pending.nonce {
        return Err(DeviceActivationError::InvalidEvidence(
            "pending nonce does not match activation options".into(),
        ));
    }
    let started_at = tokio::time::Instant::now();
    let deadline = started_at + timeout;
    let review_lifetime_ms = pending.expires_at.saturating_sub(pending.server_now);
    let review_deadline =
        started_at + Duration::from_millis(u64::try_from(review_lifetime_ms).unwrap_or_default());
    loop {
        let now = tokio::time::Instant::now();
        if now >= review_deadline {
            return Err(DeviceActivationError::Expired);
        }
        let remaining = deadline.saturating_duration_since(now);
        if remaining.is_zero() {
            return Err(DeviceActivationError::TimedOut);
        }
        tokio::time::sleep(
            Duration::from_millis(pending.retry_after_ms.max(1))
                .min(remaining)
                .min(review_deadline.saturating_duration_since(now)),
        )
        .await;
        match check_device_activation(options).await {
            Err(DeviceActivationError::Disabled) => return Err(DeviceActivationError::Disabled),
            Err(DeviceActivationError::Rejected) => return Err(DeviceActivationError::Rejected),
            Err(error) => return Err(error),
            Ok(DeviceActivationStatus::Ready(session)) => return Ok(session),
            Ok(DeviceActivationStatus::Pending(current))
                if current.review_id == pending.review_id => {}
            Ok(DeviceActivationStatus::Pending(_)) => return Err(DeviceActivationError::Expired),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine as _;

    use super::{
        activation_status, derive_device_confirmation_code, DeviceActivationAttempt,
        DeviceActivationError, DeviceActivationOptions, DeviceActivationStatus,
    };
    use crate::client::{DeviceConnectOptions, MemoryAuthorizationContextStore};

    struct TestContract;

    impl crate::service::GeneratedServiceContract for TestContract {
        const PARTICIPANT_ID: &'static str = "test.device@v1";
        const CONTRACT_DIGEST: &'static str = "contract-digest";
        const PARTICIPANT_NEEDS_DIGEST: &'static str = "needs-digest";
        const PARTICIPANT_JSON: &'static str = "participant-json";
        const API_JSON: &'static str = "api-json";
        const API_DIGEST: &'static str = "api-digest";
        const REFERENCED_API_ARTIFACTS: &'static [(&'static str, &'static str)] = &[];
    }

    fn activation_options(
        instance_id: &'static str,
    ) -> DeviceActivationOptions<'static, TestContract> {
        DeviceActivationOptions::new(
            DeviceConnectOptions::<TestContract>::new(
                "http://trellis.test",
                "deployment",
                instance_id,
                "public-identity-key",
                "identity-seed",
                Arc::new(MemoryAuthorizationContextStore::default()),
            ),
            "activation-key",
        )
        .with_nonce("nonce")
    }

    fn ready_session(
        options: &DeviceActivationOptions<'_, TestContract>,
        session_seed: String,
    ) -> super::DeviceActivationSession {
        match activation_status(
            options,
            "confirmation-code".into(),
            serde_json::from_value(serde_json::json!({
                "serverNow": 1,
                "state": "ready"
            }))
            .expect("ready response"),
            session_seed,
        )
        .expect("ready status")
        {
            DeviceActivationStatus::Ready(session) => session,
            DeviceActivationStatus::Pending(_) => panic!("expected ready session"),
        }
    }

    #[test]
    fn confirmation_code_is_deterministic_and_crockford_encoded() {
        let code = derive_device_confirmation_code(
            "z89beQNUvhI08xF7ceiwvCD_kUF_RtBGcvDFsyiErgA",
            "PJOPafbG8Sq47Ra0sOSYmG2pJQj5FRgPrlwynA5Dq0I",
            "nonce",
        )
        .expect("confirmation code");
        assert_eq!(code, "HMHF9MKZ");
        assert_eq!(code.len(), 8);
        assert!(code
            .bytes()
            .all(|byte| b"0123456789ABCDEFGHJKMNPQRSTVWXYZ".contains(&byte)));
        assert_eq!(
            code,
            derive_device_confirmation_code(
                "z89beQNUvhI08xF7ceiwvCD_kUF_RtBGcvDFsyiErgA",
                "PJOPafbG8Sq47Ra0sOSYmG2pJQj5FRgPrlwynA5Dq0I",
                "nonce",
            )
            .expect("same confirmation code")
        );
    }

    #[test]
    fn activation_session_requires_its_exact_origin() {
        let successful_seed = URL_SAFE_NO_PAD.encode([3_u8; 32]);
        let matching = activation_options("instance-a");
        let matching_session = ready_session(&matching, successful_seed.clone());
        assert!(matching.into_connect_options(matching_session).is_ok());

        let origin = activation_options("instance-a");
        let unrelated = activation_options("instance-b");
        assert!(matches!(
            unrelated.into_connect_options(ready_session(&origin, successful_seed)),
            Err(DeviceActivationError::SessionOriginMismatch)
        ));
    }

    #[test]
    fn pending_attempts_rotate_keys_and_ready_handoff_keeps_successful_seed() {
        let options = activation_options("instance");
        let first = DeviceActivationAttempt::from_seed(URL_SAFE_NO_PAD.encode([1_u8; 32]))
            .expect("first attempt");
        let second = DeviceActivationAttempt::from_seed(URL_SAFE_NO_PAD.encode([2_u8; 32]))
            .expect("second attempt");
        let first_public = first.session_auth.session_key.clone();
        let second_public = second.session_auth.session_key.clone();
        assert_ne!(first_public, second_public);

        for attempt in [first, second] {
            let status = activation_status(
                &options,
                "confirmation-code".into(),
                serde_json::from_value(serde_json::json!({
                    "serverNow": 1,
                    "state": "activation_pending",
                    "activation": {
                        "state": "pending",
                        "activationUrl": "http://trellis.test/activate",
                        "reviewId": "review",
                        "expiresAt": 1000,
                        "retryAfterMs": 1
                    }
                }))
                .expect("pending response"),
                attempt.session_key_seed_base64url,
            )
            .expect("pending status");
            assert!(matches!(status, DeviceActivationStatus::Pending(_)));
        }

        let successful_seed = URL_SAFE_NO_PAD.encode([3_u8; 32]);
        let session = ready_session(&options, successful_seed.clone());
        let connect = options
            .into_connect_options(session)
            .expect("ready handoff");
        assert_eq!(
            connect.activation_session_seed(),
            Some(successful_seed.as_str())
        );
    }

    #[test]
    fn production_attempts_generate_distinct_session_keys() {
        let first = DeviceActivationAttempt::new().expect("first production attempt");
        let second = DeviceActivationAttempt::new().expect("second production attempt");
        assert_ne!(
            first.session_auth.session_key,
            second.session_auth.session_key
        );
    }

    #[test]
    fn activation_session_debug_is_redacted_through_status() {
        let options = activation_options("instance");
        let private_seed = URL_SAFE_NO_PAD.encode([9_u8; 32]);
        let debug = format!(
            "{:?}",
            DeviceActivationStatus::Ready(ready_session(&options, private_seed.clone()))
        );
        assert!(debug.contains("DeviceActivationSession"));
        assert!(debug.contains("server_now"));
        assert!(!debug.contains(&private_seed));
        assert!(!debug.contains("bootstrap"));
    }
}
