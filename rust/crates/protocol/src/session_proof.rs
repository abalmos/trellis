use std::fmt;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use jsonptr::PointerBuf;
use nkeys::KeyPairType;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};

use crate::{canonicalize_json, ProtocolError, SessionProofErrorCodeV1};

/// Strict wire format for Milestone 8 bootstrap, session-control, and NATS connect proofs.
pub const SESSION_PROOF_FORMAT_V1: &str = "trellis.session-proof.v1";

const MAXIMUM_SAFE_JSON_INTEGER: i64 = 9_007_199_254_740_991;
const MAXIMUM_REQUEST_ID_BYTES: usize = 256;
const MAXIMUM_TEXT_BYTES: usize = 16 * 1024;
const MAXIMUM_PROOF_WINDOW_MS: i64 = 5 * 60 * 1_000;

/// One fixed signature domain within [`SESSION_PROOF_FORMAT_V1`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionProofPurposeV1 {
    /// Start a user app or agent browser-auth request.
    UserAuthRequest,
    /// Reconnect an existing user app or agent session through client bootstrap.
    ClientBootstrap,
    /// Bootstrap a provisioned service instance.
    ServiceBootstrap,
    /// Bootstrap a provisioned or activated device.
    DeviceBootstrap,
    /// Authenticate one NATS connection or reconnection.
    NatsConnect,
    /// Revoke or end the signing session itself before ordinary request auth is available.
    SessionSelfControl,
}

impl SessionProofPurposeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::UserAuthRequest => "userAuthRequest",
            Self::ClientBootstrap => "clientBootstrap",
            Self::ServiceBootstrap => "serviceBootstrap",
            Self::DeviceBootstrap => "deviceBootstrap",
            Self::NatsConnect => "natsConnect",
            Self::SessionSelfControl => "sessionSelfControl",
        }
    }
}

impl fmt::Display for SessionProofPurposeV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Validated purpose-specific input to one session proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionProofInputV1 {
    purpose: SessionProofPurposeV1,
    request_id: String,
    issued_at: i64,
    signer_key_id: String,
    transcript_fields: Vec<Vec<u8>>,
    nkey_binding: Option<NkeyBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum NkeyBinding {
    Signer(String),
    PublicKey {
        nkey: String,
        public_key: VerifyingKey,
    },
}

impl SessionProofInputV1 {
    /// Build a user browser-auth initiation proof input.
    ///
    /// `request_digest` is the output of [`session_proof_request_digest_v1`].
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::SessionProof`] when a field is noncanonical,
    /// unsafe, empty, malformed, or the NKey does not encode `session_public_key`.
    #[allow(clippy::too_many_arguments)]
    pub fn user_auth_request(
        request_id: impl Into<String>,
        issued_at: i64,
        session_public_key: impl Into<String>,
        session_nkey: impl Into<String>,
        participant_id: impl Into<String>,
        participant_digest: impl Into<String>,
        redirect_target: impl Into<String>,
        request_digest: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        let request_id = request_id.into();
        let session_public_key = session_public_key.into();
        let session_nkey = session_nkey.into();
        let participant_id = participant_id.into();
        let participant_digest = participant_digest.into();
        let redirect_target = redirect_target.into();
        let request_digest = request_digest.into();
        let key = decode_public_key(&session_public_key, &["sessionPublicKey"])?;
        let session_nkey_bytes = validate_nkey_binding(&session_nkey, &key, &["sessionNkey"])?;
        let signer_key_id = derived_key_id(&key);

        Self::new(
            SessionProofPurposeV1::UserAuthRequest,
            request_id,
            issued_at,
            signer_key_id,
            vec![
                key.as_bytes().to_vec(),
                session_nkey_bytes.to_vec(),
                text(&participant_id, &["participantId"])?,
                digest(&participant_digest, &["participantDigest"])?,
                text(&redirect_target, &["redirectTarget"])?,
                digest(&request_digest, &["requestDigest"])?,
            ],
            Some(NkeyBinding::PublicKey {
                nkey: session_nkey,
                public_key: key,
            }),
        )
    }

    /// Build an existing client-session bootstrap proof input.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::SessionProof`] when a field is noncanonical,
    /// unsafe, empty, or malformed.
    #[allow(clippy::too_many_arguments)]
    pub fn client_bootstrap(
        request_id: impl Into<String>,
        issued_at: i64,
        session_id: impl Into<String>,
        session_key_id: impl Into<String>,
        session_public_key: impl Into<String>,
        session_nkey: impl Into<String>,
        expected_participant_digest: Option<String>,
        expected_needs_digest: Option<String>,
        request_digest: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        let request_id = request_id.into();
        let session_id = session_id.into();
        let session_key_id = session_key_id.into();
        let session_public_key = session_public_key.into();
        let session_nkey = session_nkey.into();
        let request_digest = request_digest.into();
        let session_key = decode_public_key(&session_public_key, &["sessionPublicKey"])?;
        if session_key_id != derived_key_id(&session_key) {
            return Err(proof_error(
                SessionProofErrorCodeV1::InvalidKeyId,
                ["sessionKeyId"],
                "session key id does not match the session public key",
            ));
        }
        let session_nkey_bytes =
            validate_nkey_binding(&session_nkey, &session_key, &["sessionNkey"])?;

        Self::new(
            SessionProofPurposeV1::ClientBootstrap,
            request_id,
            issued_at,
            session_key_id.clone(),
            vec![
                text(&session_id, &["sessionId"])?,
                digest(&session_key_id, &["sessionKeyId"])?,
                session_nkey_bytes.to_vec(),
                optional_digest(
                    expected_participant_digest.as_deref(),
                    &["expectedParticipantDigest"],
                )?,
                optional_digest(expected_needs_digest.as_deref(), &["expectedNeedsDigest"])?,
                digest(&request_digest, &["requestDigest"])?,
            ],
            Some(NkeyBinding::Signer(session_nkey)),
        )
    }

    /// Build a provisioned service-instance bootstrap proof input.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::SessionProof`] when a field is noncanonical,
    /// unsafe, empty, malformed, or the NKey does not encode `new_session_public_key`.
    #[allow(clippy::too_many_arguments)]
    pub fn service_bootstrap(
        request_id: impl Into<String>,
        issued_at: i64,
        deployment_id: impl Into<String>,
        instance_id: impl Into<String>,
        provisioned_identity_key_id: impl Into<String>,
        new_session_public_key: impl Into<String>,
        new_session_nkey: impl Into<String>,
        participant_id: impl Into<String>,
        participant_digest: impl Into<String>,
        request_digest: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        let request_id = request_id.into();
        let deployment_id = deployment_id.into();
        let instance_id = instance_id.into();
        let provisioned_identity_key_id = provisioned_identity_key_id.into();
        let new_session_public_key = new_session_public_key.into();
        let new_session_nkey = new_session_nkey.into();
        let participant_id = participant_id.into();
        let participant_digest = participant_digest.into();
        let request_digest = request_digest.into();
        validate_key_id(&provisioned_identity_key_id, &["provisionedIdentityKeyId"])?;
        let session_key = decode_public_key(&new_session_public_key, &["newSessionPublicKey"])?;
        let session_nkey_bytes =
            validate_nkey_binding(&new_session_nkey, &session_key, &["newSessionNkey"])?;

        Self::new(
            SessionProofPurposeV1::ServiceBootstrap,
            request_id,
            issued_at,
            provisioned_identity_key_id.clone(),
            vec![
                text(&deployment_id, &["deploymentId"])?,
                text(&instance_id, &["instanceId"])?,
                digest(&provisioned_identity_key_id, &["provisionedIdentityKeyId"])?,
                session_key.as_bytes().to_vec(),
                session_nkey_bytes.to_vec(),
                text(&participant_id, &["participantId"])?,
                digest(&participant_digest, &["participantDigest"])?,
                digest(&request_digest, &["requestDigest"])?,
            ],
            Some(NkeyBinding::PublicKey {
                nkey: new_session_nkey,
                public_key: session_key,
            }),
        )
    }

    /// Build a provisioned or activated device bootstrap proof input.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::SessionProof`] when a field is noncanonical,
    /// unsafe, empty, malformed, or the NKey does not encode `new_session_public_key`.
    #[allow(clippy::too_many_arguments)]
    pub fn device_bootstrap(
        request_id: impl Into<String>,
        issued_at: i64,
        deployment_id: impl Into<String>,
        instance_id: impl Into<String>,
        device_identity_key_id: impl Into<String>,
        new_session_public_key: impl Into<String>,
        new_session_nkey: impl Into<String>,
        participant_id: impl Into<String>,
        participant_digest: impl Into<String>,
        challenge_digest: Option<String>,
        request_digest: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        let request_id = request_id.into();
        let deployment_id = deployment_id.into();
        let instance_id = instance_id.into();
        let device_identity_key_id = device_identity_key_id.into();
        let new_session_public_key = new_session_public_key.into();
        let new_session_nkey = new_session_nkey.into();
        let participant_id = participant_id.into();
        let participant_digest = participant_digest.into();
        let request_digest = request_digest.into();
        validate_key_id(&device_identity_key_id, &["deviceIdentityKeyId"])?;
        let session_key = decode_public_key(&new_session_public_key, &["newSessionPublicKey"])?;
        let session_nkey_bytes =
            validate_nkey_binding(&new_session_nkey, &session_key, &["newSessionNkey"])?;

        Self::new(
            SessionProofPurposeV1::DeviceBootstrap,
            request_id,
            issued_at,
            device_identity_key_id.clone(),
            vec![
                text(&deployment_id, &["deploymentId"])?,
                text(&instance_id, &["instanceId"])?,
                digest(&device_identity_key_id, &["deviceIdentityKeyId"])?,
                session_key.as_bytes().to_vec(),
                session_nkey_bytes.to_vec(),
                text(&participant_id, &["participantId"])?,
                digest(&participant_digest, &["participantDigest"])?,
                optional_digest(challenge_digest.as_deref(), &["challengeDigest"])?,
                digest(&request_digest, &["requestDigest"])?,
            ],
            Some(NkeyBinding::PublicKey {
                nkey: new_session_nkey,
                public_key: session_key,
            }),
        )
    }

    /// Build a nonce-bound NATS connect proof input.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::SessionProof`] when a field is noncanonical,
    /// unsafe, empty, or malformed.
    #[allow(clippy::too_many_arguments)]
    pub fn nats_connect(
        request_id: impl Into<String>,
        issued_at: i64,
        session_id: impl Into<String>,
        session_key_id: impl Into<String>,
        session_public_key: impl Into<String>,
        session_nkey: impl Into<String>,
        participant_digest: impl Into<String>,
        nonce: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        let request_id = request_id.into();
        let session_id = session_id.into();
        let session_key_id = session_key_id.into();
        let session_public_key = session_public_key.into();
        let session_nkey = session_nkey.into();
        let participant_digest = participant_digest.into();
        let nonce = nonce.into();
        let session_key = decode_public_key(&session_public_key, &["sessionPublicKey"])?;
        if session_key_id != derived_key_id(&session_key) {
            return Err(proof_error(
                SessionProofErrorCodeV1::InvalidKeyId,
                ["sessionKeyId"],
                "session key id does not match the session public key",
            ));
        }
        let session_nkey_bytes =
            validate_nkey_binding(&session_nkey, &session_key, &["sessionNkey"])?;

        Self::new(
            SessionProofPurposeV1::NatsConnect,
            request_id,
            issued_at,
            session_key_id.clone(),
            vec![
                text(&session_id, &["sessionId"])?,
                digest(&session_key_id, &["sessionKeyId"])?,
                session_nkey_bytes.to_vec(),
                digest(&participant_digest, &["participantDigest"])?,
                text(&nonce, &["nonce"])?,
            ],
            Some(NkeyBinding::Signer(session_nkey)),
        )
    }

    /// Build a pre-context session self-control proof input.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::SessionProof`] when a field is noncanonical,
    /// unsafe, empty, or malformed.
    pub fn session_self_control(
        request_id: impl Into<String>,
        issued_at: i64,
        session_id: impl Into<String>,
        session_key_id: impl Into<String>,
        request_digest: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        let request_id = request_id.into();
        let session_id = session_id.into();
        let session_key_id = session_key_id.into();
        let request_digest = request_digest.into();
        validate_key_id(&session_key_id, &["sessionKeyId"])?;

        Self::new(
            SessionProofPurposeV1::SessionSelfControl,
            request_id,
            issued_at,
            session_key_id.clone(),
            vec![
                text(&session_id, &["sessionId"])?,
                digest(&session_key_id, &["sessionKeyId"])?,
                digest(&request_digest, &["requestDigest"])?,
            ],
            None,
        )
    }

    fn new(
        purpose: SessionProofPurposeV1,
        request_id: String,
        issued_at: i64,
        signer_key_id: String,
        transcript_fields: Vec<Vec<u8>>,
        nkey_binding: Option<NkeyBinding>,
    ) -> Result<Self, ProtocolError> {
        validate_request_id(&request_id)?;
        validate_safe_integer(issued_at, &["issuedAt"])?;
        Ok(Self {
            purpose,
            request_id,
            issued_at,
            signer_key_id,
            transcript_fields,
            nkey_binding,
        })
    }

    /// Return the fixed signature purpose.
    #[must_use]
    pub fn purpose(&self) -> SessionProofPurposeV1 {
        self.purpose
    }

    /// Return the caller-generated replay identifier.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Return the claimed Unix issue time in milliseconds.
    #[must_use]
    pub fn issued_at(&self) -> i64 {
        self.issued_at
    }

    /// Return the expected signing-key identifier.
    #[must_use]
    pub fn signer_key_id(&self) -> &str {
        &self.signer_key_id
    }

    fn digest(&self) -> Result<[u8; 32], ProtocolError> {
        let mut transcript = Vec::new();
        push_length_prefixed(&mut transcript, SESSION_PROOF_FORMAT_V1.as_bytes())?;
        push_length_prefixed(&mut transcript, self.purpose.as_str().as_bytes())?;
        push_length_prefixed(&mut transcript, self.request_id.as_bytes())?;
        push_length_prefixed(&mut transcript, self.issued_at.to_string().as_bytes())?;
        for field in &self.transcript_fields {
            push_length_prefixed(&mut transcript, field)?;
        }
        Ok(sha256(&transcript))
    }

    fn validate_signer(&self, key: &VerifyingKey) -> Result<(), ProtocolError> {
        if self.signer_key_id != derived_key_id(key) {
            return Err(proof_error(
                SessionProofErrorCodeV1::InvalidKeyId,
                ["signerKeyId"],
                "signer key id does not match the verification key",
            ));
        }
        match &self.nkey_binding {
            Some(NkeyBinding::Signer(nkey)) => {
                validate_nkey_binding(nkey, key, &["sessionNkey"]).map(|_| ())
            }
            Some(NkeyBinding::PublicKey { nkey, public_key }) => {
                validate_nkey_binding(nkey, public_key, &["sessionNkey"]).map(|_| ())
            }
            None => Ok(()),
        }
    }
}

/// One strict session-proof signature envelope.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionProofV1 {
    format: String,
    signature: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WireSessionProofV1 {
    format: String,
    signature: String,
}

impl<'de> Deserialize<'de> for SessionProofV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = WireSessionProofV1::deserialize(deserializer)?;
        parse_wire_proof(wire).map_err(de::Error::custom)
    }
}

impl SessionProofV1 {
    /// Return the proof format identifier.
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Return the canonical unpadded base64url Ed25519 signature.
    #[must_use]
    pub fn signature(&self) -> &str {
        &self.signature
    }
}

/// Freshness limits applied while verifying session proofs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionProofPolicyV1 {
    maximum_age_ms: i64,
    maximum_future_skew_ms: i64,
}

impl SessionProofPolicyV1 {
    /// Construct bounded proof freshness policy.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::SessionProof`] when either limit is negative,
    /// exceeds five minutes, or their sum overflows.
    pub fn new(maximum_age_ms: i64, maximum_future_skew_ms: i64) -> Result<Self, ProtocolError> {
        if !(0..=MAXIMUM_PROOF_WINDOW_MS).contains(&maximum_age_ms)
            || !(0..=MAXIMUM_PROOF_WINDOW_MS).contains(&maximum_future_skew_ms)
        {
            return Err(proof_error(
                SessionProofErrorCodeV1::InvalidFormat,
                std::iter::empty::<&str>(),
                "proof age and future skew must be between zero and five minutes",
            ));
        }
        maximum_age_ms
            .checked_add(maximum_future_skew_ms)
            .ok_or_else(|| {
                proof_error(
                    SessionProofErrorCodeV1::InvalidFormat,
                    std::iter::empty::<&str>(),
                    "proof replay window overflows",
                )
            })?;
        Ok(Self {
            maximum_age_ms,
            maximum_future_skew_ms,
        })
    }

    /// Return the maximum accepted proof age in milliseconds.
    #[must_use]
    pub fn maximum_age_ms(self) -> i64 {
        self.maximum_age_ms
    }

    /// Return the maximum accepted future clock skew in milliseconds.
    #[must_use]
    pub fn maximum_future_skew_ms(self) -> i64 {
        self.maximum_future_skew_ms
    }

    /// Return the minimum replay-record retention after first admission.
    #[must_use]
    pub fn replay_retention_ms(self) -> i64 {
        self.maximum_age_ms + self.maximum_future_skew_ms
    }
}

impl Default for SessionProofPolicyV1 {
    fn default() -> Self {
        Self {
            maximum_age_ms: 30_000,
            maximum_future_skew_ms: 30_000,
        }
    }
}

/// Stable replay identity returned by successful proof verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionProofReplayKeyV1 {
    purpose: SessionProofPurposeV1,
    signer_key_id: String,
    request_id: String,
    transcript_digest: String,
}

impl SessionProofReplayKeyV1 {
    /// Return the proof purpose that scopes the request ID.
    #[must_use]
    pub fn purpose(&self) -> SessionProofPurposeV1 {
        self.purpose
    }

    /// Return the verified signer key identifier that scopes the request ID.
    #[must_use]
    pub fn signer_key_id(&self) -> &str {
        &self.signer_key_id
    }

    /// Return the caller-generated request ID.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Return the exact verified transcript digest used for changed-content detection.
    #[must_use]
    pub fn transcript_digest(&self) -> &str {
        &self.transcript_digest
    }
}

/// A cryptographically verified session proof and its replay identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedSessionProofV1 {
    replay_key: SessionProofReplayKeyV1,
}

impl VerifiedSessionProofV1 {
    /// Return the replay identity that must be admitted atomically by runtime storage.
    #[must_use]
    pub fn replay_key(&self) -> &SessionProofReplayKeyV1 {
        &self.replay_key
    }
}

/// Parse and structurally validate one strict session-proof envelope.
///
/// # Errors
///
/// Returns [`ProtocolError::SessionProof`] when the value has an unknown member,
/// wrong format, or noncanonical signature encoding.
pub fn parse_session_proof_v1(value: &Value) -> Result<SessionProofV1, ProtocolError> {
    let wire: WireSessionProofV1 = serde_json::from_value(value.clone()).map_err(|error| {
        proof_error(
            SessionProofErrorCodeV1::InvalidFormat,
            std::iter::empty::<&str>(),
            error.to_string(),
        )
    })?;
    parse_wire_proof(wire)
}

/// Compute the canonical request digest used by HTTP session proofs.
///
/// The input must contain a top-level `proof` object with the exact format and a
/// `signature` member. Only that signature member is removed before Trellis JSON
/// canonicalization and SHA-256 hashing.
///
/// # Errors
///
/// Returns [`ProtocolError::SessionProof`] when the request/proof shape is wrong,
/// the proof format differs, or canonicalization fails.
pub fn session_proof_request_digest_v1(request: &Value) -> Result<String, ProtocolError> {
    let mut unsigned = request.clone();
    let object = unsigned.as_object_mut().ok_or_else(|| {
        proof_error(
            SessionProofErrorCodeV1::InvalidFormat,
            std::iter::empty::<&str>(),
            "proof-bearing request must be an object",
        )
    })?;
    let proof = object
        .get_mut("proof")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| {
            proof_error(
                SessionProofErrorCodeV1::InvalidFormat,
                ["proof"],
                "proof must be an object",
            )
        })?;
    if proof.get("format").and_then(Value::as_str) != Some(SESSION_PROOF_FORMAT_V1) {
        return Err(proof_error(
            SessionProofErrorCodeV1::InvalidFormat,
            ["proof", "format"],
            format!("format must equal '{SESSION_PROOF_FORMAT_V1}'"),
        ));
    }
    if proof.remove("signature").is_none() {
        return Err(proof_error(
            SessionProofErrorCodeV1::InvalidFormat,
            ["proof", "signature"],
            "signature member is required",
        ));
    }
    validate_safe_json_integers(&unsigned, &mut Vec::new())?;
    let canonical = canonicalize_json(&unsigned)?;
    Ok(encode_base64url(&sha256(canonical.as_bytes())))
}

/// Compute the canonical SHA-256 signing digest for one validated proof input.
///
/// This is the cross-language boundary used by WebCrypto clients and WASM
/// bindings. Signatures are Ed25519 signatures over the decoded 32 digest bytes.
///
/// # Errors
///
/// Returns [`ProtocolError::SessionProof`] if a length-prefixed transcript
/// component cannot be encoded.
pub fn session_proof_signing_digest_v1(
    input: &SessionProofInputV1,
) -> Result<String, ProtocolError> {
    Ok(encode_base64url(&input.digest()?))
}

/// Sign one validated purpose-specific session-proof input.
///
/// # Errors
///
/// Returns [`ProtocolError::SessionProof`] when the supplied signing key does not
/// match the input's signer identity or a transcript component cannot be encoded.
pub fn sign_session_proof_v1(
    input: &SessionProofInputV1,
    signing_key: &SigningKey,
) -> Result<SessionProofV1, ProtocolError> {
    input.validate_signer(&signing_key.verifying_key())?;
    let digest = input.digest()?;
    Ok(SessionProofV1 {
        format: SESSION_PROOF_FORMAT_V1.to_owned(),
        signature: encode_base64url(&signing_key.sign(&digest).to_bytes()),
    })
}

/// Verify one session proof against its expected signer and freshness policy.
///
/// Runtime callers must still atomically admit the returned replay key. Pure
/// protocol verification does not own replay storage.
///
/// # Errors
///
/// Returns [`ProtocolError::SessionProof`] when signer identity, proof format,
/// freshness, transcript encoding, or signature verification fails.
pub fn verify_session_proof_v1(
    input: &SessionProofInputV1,
    proof: &SessionProofV1,
    expected_signer_public_key: &str,
    now_ms: i64,
    policy: SessionProofPolicyV1,
) -> Result<VerifiedSessionProofV1, ProtocolError> {
    validate_safe_integer(now_ms, &["now"])?;
    if proof.format != SESSION_PROOF_FORMAT_V1 {
        return Err(proof_error(
            SessionProofErrorCodeV1::InvalidFormat,
            ["proof", "format"],
            format!("format must equal '{SESSION_PROOF_FORMAT_V1}'"),
        ));
    }
    let oldest = now_ms.checked_sub(policy.maximum_age_ms).ok_or_else(|| {
        proof_error(
            SessionProofErrorCodeV1::ProofIatOutOfRange,
            ["issuedAt"],
            "proof age calculation underflowed",
        )
    })?;
    let newest = now_ms
        .checked_add(policy.maximum_future_skew_ms)
        .ok_or_else(|| {
            proof_error(
                SessionProofErrorCodeV1::ProofIatOutOfRange,
                ["issuedAt"],
                "proof future-skew calculation overflowed",
            )
        })?;
    if !(oldest..=newest).contains(&input.issued_at) {
        return Err(proof_error(
            SessionProofErrorCodeV1::ProofIatOutOfRange,
            ["issuedAt"],
            "proof issue time is outside the accepted policy window",
        ));
    }

    let key = decode_public_key(expected_signer_public_key, &["signerPublicKey"])?;
    input.validate_signer(&key)?;
    let digest = input.digest()?;
    let signature = decode_base64url::<64>(
        &proof.signature,
        &["proof", "signature"],
        SessionProofErrorCodeV1::InvalidSignature,
    )?;
    key.verify_strict(&digest, &Signature::from_bytes(&signature))
        .map_err(|_| {
            proof_error(
                SessionProofErrorCodeV1::InvalidSignature,
                ["proof", "signature"],
                "signature verification failed",
            )
        })?;

    Ok(VerifiedSessionProofV1 {
        replay_key: SessionProofReplayKeyV1 {
            purpose: input.purpose,
            signer_key_id: input.signer_key_id.clone(),
            request_id: input.request_id.clone(),
            transcript_digest: encode_base64url(&digest),
        },
    })
}

fn parse_wire_proof(wire: WireSessionProofV1) -> Result<SessionProofV1, ProtocolError> {
    if wire.format != SESSION_PROOF_FORMAT_V1 {
        return Err(proof_error(
            SessionProofErrorCodeV1::InvalidFormat,
            ["format"],
            format!("format must equal '{SESSION_PROOF_FORMAT_V1}'"),
        ));
    }
    decode_base64url::<64>(
        &wire.signature,
        &["signature"],
        SessionProofErrorCodeV1::InvalidSignature,
    )?;
    Ok(SessionProofV1 {
        format: wire.format,
        signature: wire.signature,
    })
}

fn proof_error<'a>(
    code: SessionProofErrorCodeV1,
    tokens: impl IntoIterator<Item = &'a str>,
    message: impl Into<String>,
) -> ProtocolError {
    ProtocolError::SessionProof {
        code,
        path: PointerBuf::from_tokens(tokens),
        message: message.into(),
    }
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn encode_base64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn decode_base64url<const N: usize>(
    encoded: &str,
    path: &[&str],
    code: SessionProofErrorCodeV1,
) -> Result<[u8; N], ProtocolError> {
    if encoded.contains('=') {
        return Err(proof_error(
            SessionProofErrorCodeV1::InvalidEncoding,
            path.iter().copied(),
            "padded base64url is not accepted",
        ));
    }
    let decoded = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| {
        proof_error(
            SessionProofErrorCodeV1::InvalidEncoding,
            path.iter().copied(),
            "value is not unpadded base64url",
        )
    })?;
    if decoded.len() != N || encode_base64url(&decoded) != encoded {
        return Err(proof_error(
            code,
            path.iter().copied(),
            format!("value must canonically encode exactly {N} bytes"),
        ));
    }
    decoded.try_into().map_err(|_| {
        proof_error(
            code,
            path.iter().copied(),
            format!("value must encode exactly {N} bytes"),
        )
    })
}

fn decode_public_key(encoded: &str, path: &[&str]) -> Result<VerifyingKey, ProtocolError> {
    let key = VerifyingKey::from_bytes(&decode_base64url::<32>(
        encoded,
        path,
        SessionProofErrorCodeV1::InvalidPublicKey,
    )?)
    .map_err(|_| {
        proof_error(
            SessionProofErrorCodeV1::InvalidPublicKey,
            path.iter().copied(),
            "value is not a valid Ed25519 public key",
        )
    })?;
    if key.is_weak() {
        return Err(proof_error(
            SessionProofErrorCodeV1::InvalidPublicKey,
            path.iter().copied(),
            "weak Ed25519 public keys are not accepted",
        ));
    }
    Ok(key)
}

fn derived_key_id(key: &VerifyingKey) -> String {
    encode_base64url(&sha256(key.as_bytes()))
}

fn validate_key_id(value: &str, path: &[&str]) -> Result<(), ProtocolError> {
    decode_base64url::<32>(value, path, SessionProofErrorCodeV1::InvalidKeyId).map(|_| ())
}

fn validate_nkey(value: &str, path: &[&str]) -> Result<[u8; 32], ProtocolError> {
    let (kind, bytes) = nkeys::from_public_key(value).map_err(|_| {
        proof_error(
            SessionProofErrorCodeV1::InvalidNatsKey,
            path.iter().copied(),
            "value is not a canonical NATS public key",
        )
    })?;
    if KeyPairType::from(kind) != KeyPairType::User {
        return Err(proof_error(
            SessionProofErrorCodeV1::InvalidNatsKey,
            path.iter().copied(),
            "value must be a NATS User NKey",
        ));
    }
    Ok(bytes)
}

fn validate_nkey_binding(
    nkey: &str,
    key: &VerifyingKey,
    path: &[&str],
) -> Result<[u8; 32], ProtocolError> {
    let nkey_bytes = validate_nkey(nkey, path)?;
    if nkey_bytes == *key.as_bytes() {
        Ok(nkey_bytes)
    } else {
        Err(proof_error(
            SessionProofErrorCodeV1::InvalidNatsKey,
            path.iter().copied(),
            "NATS User NKey does not encode the session public key",
        ))
    }
}

fn validate_safe_json_integers(value: &Value, path: &mut Vec<String>) -> Result<(), ProtocolError> {
    match value {
        Value::Number(number) => {
            let unsafe_integer = if let Some(value) = number.as_i64() {
                value.unsigned_abs() > MAXIMUM_SAFE_JSON_INTEGER as u64
            } else if let Some(value) = number.as_u64() {
                value > MAXIMUM_SAFE_JSON_INTEGER as u64
            } else {
                number.as_f64().is_some_and(|value| {
                    value.fract() == 0.0 && value.abs() > MAXIMUM_SAFE_JSON_INTEGER as f64
                })
            };
            if unsafe_integer {
                return Err(ProtocolError::SessionProof {
                    code: SessionProofErrorCodeV1::UnsafeJsonInteger,
                    path: PointerBuf::from_tokens(path.iter().map(String::as_str)),
                    message: "integer must be within the interoperable JSON safe-integer range"
                        .to_owned(),
                });
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                path.push(index.to_string());
                validate_safe_json_integers(value, path)?;
                path.pop();
            }
        }
        Value::Object(values) => {
            for (name, value) in values {
                path.push(name.clone());
                validate_safe_json_integers(value, path)?;
                path.pop();
            }
        }
        Value::Null | Value::Bool(_) | Value::String(_) => {}
    }
    Ok(())
}

fn validate_safe_integer(value: i64, path: &[&str]) -> Result<(), ProtocolError> {
    if (-MAXIMUM_SAFE_JSON_INTEGER..=MAXIMUM_SAFE_JSON_INTEGER).contains(&value) {
        Ok(())
    } else {
        Err(proof_error(
            SessionProofErrorCodeV1::UnsafeJsonInteger,
            path.iter().copied(),
            "integer must be within the interoperable JSON safe-integer range",
        ))
    }
}

fn validate_request_id(value: &str) -> Result<(), ProtocolError> {
    validate_text(value, &["requestId"])?;
    if value.len() <= MAXIMUM_REQUEST_ID_BYTES {
        Ok(())
    } else {
        Err(proof_error(
            SessionProofErrorCodeV1::InvalidFormat,
            ["requestId"],
            "request ID exceeds 256 UTF-8 bytes",
        ))
    }
}

fn validate_text(value: &str, path: &[&str]) -> Result<(), ProtocolError> {
    if value.is_empty()
        || value.len() > MAXIMUM_TEXT_BYTES
        || value.trim() != value
        || value.chars().any(|character| character.is_ascii_control())
    {
        return Err(proof_error(
            SessionProofErrorCodeV1::InvalidFormat,
            path.iter().copied(),
            "value must be bounded, nonempty protocol-safe text",
        ));
    }
    Ok(())
}

fn text(value: &str, path: &[&str]) -> Result<Vec<u8>, ProtocolError> {
    validate_text(value, path)?;
    Ok(value.as_bytes().to_vec())
}

fn digest(value: &str, path: &[&str]) -> Result<Vec<u8>, ProtocolError> {
    Ok(decode_base64url::<32>(value, path, SessionProofErrorCodeV1::InvalidEncoding)?.to_vec())
}

fn optional_digest(value: Option<&str>, path: &[&str]) -> Result<Vec<u8>, ProtocolError> {
    value.map_or_else(|| Ok(Vec::new()), |value| digest(value, path))
}

fn push_length_prefixed(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), ProtocolError> {
    let length = u32::try_from(bytes.len()).map_err(|_| {
        proof_error(
            SessionProofErrorCodeV1::InvalidFormat,
            std::iter::empty::<&str>(),
            "signature input component exceeds u32 length",
        )
    })?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const SEED: [u8; 32] = [7; 32];
    const DIGEST: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    fn key() -> SigningKey {
        SigningKey::from_bytes(&SEED)
    }

    fn nkey() -> String {
        nkeys::KeyPair::new_from_raw(KeyPairType::User, SEED)
            .expect("NKey from seed")
            .public_key()
    }

    #[test]
    fn signs_and_verifies_nonce_bound_nats_connect() -> Result<(), ProtocolError> {
        let key = key();
        let key_id = derived_key_id(&key.verifying_key());
        let input = SessionProofInputV1::nats_connect(
            "req_connect_1",
            1_735_689_600_000,
            "ses_1",
            key_id,
            encode_base64url(key.verifying_key().as_bytes()),
            nkey(),
            DIGEST,
            "server-nonce",
        )?;
        let proof = sign_session_proof_v1(&input, &key)?;
        let verified = verify_session_proof_v1(
            &input,
            &proof,
            &encode_base64url(key.verifying_key().as_bytes()),
            1_735_689_600_000,
            SessionProofPolicyV1::default(),
        )?;
        assert_eq!(
            verified.replay_key().purpose(),
            SessionProofPurposeV1::NatsConnect
        );

        let changed = SessionProofInputV1::nats_connect(
            "req_connect_1",
            1_735_689_600_000,
            "ses_1",
            derived_key_id(&key.verifying_key()),
            encode_base64url(key.verifying_key().as_bytes()),
            nkey(),
            DIGEST,
            "another-nonce",
        )?;
        assert!(matches!(
            verify_session_proof_v1(
                &changed,
                &proof,
                &encode_base64url(key.verifying_key().as_bytes()),
                1_735_689_600_000,
                SessionProofPolicyV1::default(),
            ),
            Err(ProtocolError::SessionProof {
                code: SessionProofErrorCodeV1::InvalidSignature,
                ..
            })
        ));
        Ok(())
    }

    #[test]
    fn canonical_request_digest_removes_only_signature() -> Result<(), ProtocolError> {
        let first = json!({
            "requestId": "req_1",
            "proof": {"format": SESSION_PROOF_FORMAT_V1, "signature": "first"},
            "nullable": null
        });
        let second = json!({
            "nullable": null,
            "proof": {"signature": "second", "format": SESSION_PROOF_FORMAT_V1},
            "requestId": "req_1"
        });
        assert_eq!(
            session_proof_request_digest_v1(&first)?,
            session_proof_request_digest_v1(&second)?
        );
        Ok(())
    }

    #[test]
    fn rejects_unsafe_request_integers_and_weak_keys() {
        let unsafe_request = json!({
            "proof": {
                "format": SESSION_PROOF_FORMAT_V1,
                "signature": encode_base64url(&[0; 64])
            },
            "nested": {"counter": 9_007_199_254_740_992_u64}
        });
        assert!(matches!(
            session_proof_request_digest_v1(&unsafe_request),
            Err(ProtocolError::SessionProof {
                code: SessionProofErrorCodeV1::UnsafeJsonInteger,
                ..
            })
        ));
        assert!(matches!(
            decode_public_key(&encode_base64url(&[0; 32]), &["sessionPublicKey"]),
            Err(ProtocolError::SessionProof {
                code: SessionProofErrorCodeV1::InvalidPublicKey,
                ..
            })
        ));
    }

    #[test]
    fn rejects_wrong_nkey_and_out_of_window_proof() -> Result<(), ProtocolError> {
        let key = key();
        let other_nkey = nkeys::KeyPair::new_user().public_key();
        assert!(matches!(
            SessionProofInputV1::nats_connect(
                "req_connect_1",
                1_735_689_600_000,
                "ses_1",
                derived_key_id(&key.verifying_key()),
                encode_base64url(key.verifying_key().as_bytes()),
                other_nkey,
                DIGEST,
                "server-nonce",
            )
            .and_then(|input| sign_session_proof_v1(&input, &key)),
            Err(ProtocolError::SessionProof {
                code: SessionProofErrorCodeV1::InvalidNatsKey,
                ..
            })
        ));

        let input = SessionProofInputV1::session_self_control(
            "req_logout_1",
            1_735_689_600_000,
            "ses_1",
            derived_key_id(&key.verifying_key()),
            DIGEST,
        )?;
        let proof = sign_session_proof_v1(&input, &key)?;
        assert!(matches!(
            verify_session_proof_v1(
                &input,
                &proof,
                &encode_base64url(key.verifying_key().as_bytes()),
                1_735_689_700_000,
                SessionProofPolicyV1::default(),
            ),
            Err(ProtocolError::SessionProof {
                code: SessionProofErrorCodeV1::ProofIatOutOfRange,
                ..
            })
        ));
        Ok(())
    }

    fn vector_field<'a>(value: &'a Value, name: &str) -> &'a str {
        value[name]
            .as_str()
            .unwrap_or_else(|| panic!("missing vector field {name}"))
    }

    fn vector_time(value: &Value) -> i64 {
        value["issuedAt"].as_i64().expect("vector issuedAt")
    }

    fn vector_input(case: &Value) -> Result<SessionProofInputV1, ProtocolError> {
        let value = case.get("request").unwrap_or(&case["input"]);
        let request_digest = case["requestDigest"].as_str();
        match vector_field(case, "purpose") {
            "userAuthRequest" => SessionProofInputV1::user_auth_request(
                vector_field(value, "requestId"),
                vector_time(value),
                vector_field(value, "sessionPublicKey"),
                vector_field(value, "sessionNkey"),
                vector_field(value, "participantId"),
                vector_field(value, "participantDigest"),
                vector_field(value, "redirectTarget"),
                request_digest.expect("user request digest"),
            ),
            "clientBootstrap" => SessionProofInputV1::client_bootstrap(
                vector_field(value, "requestId"),
                vector_time(value),
                vector_field(value, "sessionId"),
                vector_field(value, "sessionKeyId"),
                vector_field(case, "signerPublicKey"),
                vector_field(value, "sessionNkey"),
                value["expectedParticipantDigest"]
                    .as_str()
                    .map(str::to_owned),
                value["expectedNeedsDigest"].as_str().map(str::to_owned),
                request_digest.expect("client request digest"),
            ),
            "serviceBootstrap" => SessionProofInputV1::service_bootstrap(
                vector_field(value, "requestId"),
                vector_time(value),
                vector_field(value, "deploymentId"),
                vector_field(value, "instanceId"),
                vector_field(value, "provisionedIdentityKeyId"),
                vector_field(value, "newSessionPublicKey"),
                vector_field(value, "newSessionNkey"),
                vector_field(value, "participantId"),
                vector_field(value, "participantDigest"),
                request_digest.expect("service request digest"),
            ),
            "deviceBootstrap" => SessionProofInputV1::device_bootstrap(
                vector_field(value, "requestId"),
                vector_time(value),
                vector_field(value, "deploymentId"),
                vector_field(value, "instanceId"),
                vector_field(value, "deviceIdentityKeyId"),
                vector_field(value, "newSessionPublicKey"),
                vector_field(value, "newSessionNkey"),
                vector_field(value, "participantId"),
                vector_field(value, "participantDigest"),
                value["challengeDigest"].as_str().map(str::to_owned),
                request_digest.expect("device request digest"),
            ),
            "natsConnect" => SessionProofInputV1::nats_connect(
                vector_field(value, "requestId"),
                vector_time(value),
                vector_field(value, "sessionId"),
                vector_field(value, "sessionKeyId"),
                vector_field(case, "signerPublicKey"),
                vector_field(value, "sessionNkey"),
                vector_field(value, "participantDigest"),
                vector_field(value, "nonce"),
            ),
            "sessionSelfControl" => SessionProofInputV1::session_self_control(
                vector_field(value, "requestId"),
                vector_time(value),
                vector_field(value, "sessionId"),
                vector_field(value, "sessionKeyId"),
                request_digest.expect("self-control request digest"),
            ),
            purpose => panic!("unknown vector purpose {purpose}"),
        }
    }

    #[test]
    fn shared_vectors_match_rust() -> Result<(), ProtocolError> {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../conformance/session-proof/vectors.json"
        ))?;
        let seed = decode_base64url::<32>(
            vector_field(&fixture, "identitySeed"),
            &["identitySeed"],
            SessionProofErrorCodeV1::InvalidEncoding,
        )?;
        let signing_key = SigningKey::from_bytes(&seed);
        for case in fixture["cases"].as_array().expect("vector cases") {
            let value = case.get("request").unwrap_or(&case["input"]);
            if let Some(expected) = case["requestDigest"].as_str() {
                assert_eq!(session_proof_request_digest_v1(value)?, expected);
            }
            let input = vector_input(case)?;
            let proof = parse_session_proof_v1(&value["proof"])?;
            assert_eq!(sign_session_proof_v1(&input, &signing_key)?, proof);
            let verified = verify_session_proof_v1(
                &input,
                &proof,
                vector_field(case, "signerPublicKey"),
                input.issued_at(),
                SessionProofPolicyV1::default(),
            )?;
            assert_eq!(
                verified.replay_key().transcript_digest(),
                vector_field(case, "transcriptDigest")
            );
            assert_eq!(proof.signature(), vector_field(case, "signature"));
        }
        Ok(())
    }

    #[test]
    fn shared_invalid_vectors_cover_failures() -> Result<(), ProtocolError> {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../conformance/session-proof/vectors.json"
        ))?;
        let cases = fixture["cases"].as_array().expect("vector cases");
        let find = |name: &str| {
            cases
                .iter()
                .find(|case| case["name"] == name)
                .unwrap_or_else(|| panic!("missing base vector {name}"))
        };
        let public_key = vector_field(&fixture, "identityPublicKey");
        let seed = decode_base64url::<32>(
            vector_field(&fixture, "identitySeed"),
            &["identitySeed"],
            SessionProofErrorCodeV1::InvalidEncoding,
        )?;
        let signing_key = SigningKey::from_bytes(&seed);

        for invalid in fixture["invalidCases"].as_array().expect("invalid cases") {
            let base = find(vector_field(invalid, "base"));
            let value = base.get("request").unwrap_or(&base["input"]);
            let proof = parse_session_proof_v1(&value["proof"])?;
            let mutation = vector_field(invalid, "mutation");
            let error = match mutation {
                "devicePurpose" => {
                    let request = &base["request"];
                    let input = SessionProofInputV1::device_bootstrap(
                        vector_field(request, "requestId"),
                        vector_time(request),
                        vector_field(request, "deploymentId"),
                        vector_field(request, "instanceId"),
                        vector_field(request, "provisionedIdentityKeyId"),
                        vector_field(request, "newSessionPublicKey"),
                        vector_field(request, "newSessionNkey"),
                        vector_field(request, "participantId"),
                        vector_field(request, "participantDigest"),
                        None,
                        vector_field(base, "requestDigest"),
                    )?;
                    verify_session_proof_v1(
                        &input,
                        &proof,
                        public_key,
                        input.issued_at(),
                        SessionProofPolicyV1::default(),
                    )
                    .expect_err("wrong purpose must fail")
                }
                "identityAsNewSession" => {
                    let request = &base["request"];
                    let input = SessionProofInputV1::service_bootstrap(
                        vector_field(request, "requestId"),
                        vector_time(request),
                        vector_field(request, "deploymentId"),
                        vector_field(request, "instanceId"),
                        vector_field(request, "provisionedIdentityKeyId"),
                        vector_field(&fixture, "identityPublicKey"),
                        vector_field(&fixture, "identityNkey"),
                        vector_field(request, "participantId"),
                        vector_field(request, "participantDigest"),
                        vector_field(base, "requestDigest"),
                    )?;
                    verify_session_proof_v1(
                        &input,
                        &proof,
                        public_key,
                        input.issued_at(),
                        SessionProofPolicyV1::default(),
                    )
                    .expect_err("modified session must fail")
                }
                "participantDigest" | "deploymentId" | "instanceId" | "redirectTarget"
                | "requestId" => {
                    let mut changed = base.clone();
                    let request = changed["request"].as_object_mut().expect("request object");
                    let field = match mutation {
                        "participantDigest" => "participantDigest",
                        "deploymentId" => "deploymentId",
                        "instanceId" => "instanceId",
                        "redirectTarget" => "redirectTarget",
                        "requestId" => "requestId",
                        _ => unreachable!(),
                    };
                    request.insert(
                        field.to_owned(),
                        Value::String(if mutation == "participantDigest" {
                            "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE".to_owned()
                        } else {
                            format!("changed-{}", vector_field(value, field))
                        }),
                    );
                    let input = vector_input(&changed)?;
                    verify_session_proof_v1(
                        &input,
                        &proof,
                        public_key,
                        input.issued_at(),
                        SessionProofPolicyV1::default(),
                    )
                    .expect_err("modified field must fail")
                }
                "paddedPublicKey" => {
                    let request = &base["request"];
                    SessionProofInputV1::user_auth_request(
                        vector_field(request, "requestId"),
                        vector_time(request),
                        format!("{}=", vector_field(request, "sessionPublicKey")),
                        vector_field(request, "sessionNkey"),
                        vector_field(request, "participantId"),
                        vector_field(request, "participantDigest"),
                        vector_field(request, "redirectTarget"),
                        vector_field(base, "requestDigest"),
                    )
                    .expect_err("padded public key must fail")
                }
                "sessionPublicKey" => SessionProofInputV1::nats_connect(
                    vector_field(value, "requestId"),
                    vector_time(value),
                    vector_field(value, "sessionId"),
                    vector_field(value, "sessionKeyId"),
                    vector_field(&fixture, "sessionPublicKey"),
                    vector_field(&fixture, "sessionNkey"),
                    vector_field(value, "participantDigest"),
                    vector_field(value, "nonce"),
                )
                .expect_err("mismatched session public key must fail"),
                "signature" => {
                    let input = vector_input(base)?;
                    let bad = parse_session_proof_v1(&json!({
                        "format": SESSION_PROOF_FORMAT_V1,
                        "signature": encode_base64url(&[0; 64])
                    }))?;
                    verify_session_proof_v1(
                        &input,
                        &bad,
                        public_key,
                        input.issued_at(),
                        SessionProofPolicyV1::default(),
                    )
                    .expect_err("bad signature must fail")
                }
                "expiredNow" | "futureNow" => {
                    let input = vector_input(base)?;
                    let now = if mutation == "expiredNow" {
                        input.issued_at() + 30_001
                    } else {
                        input.issued_at() - 30_001
                    };
                    verify_session_proof_v1(
                        &input,
                        &proof,
                        public_key,
                        now,
                        SessionProofPolicyV1::default(),
                    )
                    .expect_err("out-of-window proof must fail")
                }
                "admitTwice" => {
                    let input = vector_input(base)?;
                    let first = verify_session_proof_v1(
                        &input,
                        &proof,
                        public_key,
                        input.issued_at(),
                        SessionProofPolicyV1::default(),
                    )?;
                    let second = verify_session_proof_v1(
                        &input,
                        &proof,
                        public_key,
                        input.issued_at(),
                        SessionProofPolicyV1::default(),
                    )?;
                    assert_eq!(first.replay_key(), second.replay_key());
                    continue;
                }
                "sameIdDifferentNonce" => {
                    let original = vector_input(base)?;
                    let changed = SessionProofInputV1::nats_connect(
                        original.request_id(),
                        original.issued_at(),
                        "ses_example_01",
                        original.signer_key_id(),
                        public_key,
                        vector_field(&fixture, "identityNkey"),
                        DIGEST,
                        "NATS-SERVER-NONCE-02",
                    )?;
                    let changed_proof = sign_session_proof_v1(&changed, &signing_key)?;
                    let first = verify_session_proof_v1(
                        &original,
                        &proof,
                        public_key,
                        original.issued_at(),
                        SessionProofPolicyV1::default(),
                    )?;
                    let second = verify_session_proof_v1(
                        &changed,
                        &changed_proof,
                        public_key,
                        changed.issued_at(),
                        SessionProofPolicyV1::default(),
                    )?;
                    assert_eq!(first.replay_key().purpose(), second.replay_key().purpose());
                    assert_eq!(
                        first.replay_key().signer_key_id(),
                        second.replay_key().signer_key_id()
                    );
                    assert_eq!(
                        first.replay_key().request_id(),
                        second.replay_key().request_id()
                    );
                    assert_ne!(
                        first.replay_key().transcript_digest(),
                        second.replay_key().transcript_digest()
                    );
                    continue;
                }
                "unknownProofField" => {
                    let mut unknown = value["proof"].clone();
                    unknown["unknown"] = Value::Bool(true);
                    parse_session_proof_v1(&unknown).expect_err("unknown proof field must fail")
                }
                other => panic!("unknown invalid-vector mutation {other}"),
            };
            let code = match error {
                ProtocolError::SessionProof { code, .. } => format!("{code:?}"),
                other => panic!("unexpected error {other}"),
            };
            assert_eq!(code, vector_field(invalid, "expected"));
        }
        Ok(())
    }
}
