use ed25519_dalek::{Signature, Signer, SigningKey};
use nkeys::{KeyPair, KeyPairType};
use trellis_protocol::{sign_session_proof_v1, SessionProofInputV1, SessionProofV1};

use crate::client::proof::{
    base64url_decode, base64url_encode, build_event_proof_input, build_proof_input, sha256,
};
use crate::client::TrellisClientError;

/// Session-scoped signing material used for Trellis auth and RPC proofs.
#[doc = concat!("Public Trellis data type `", stringify!(SessionAuth), "`.")]
pub struct SessionAuth {
    /// Public session key in base64url form.
    #[doc = concat!("The `", stringify!(session_key), "` value.")]
    pub session_key: String,
    signing_key: SigningKey,
}

impl SessionAuth {
    /// Construct a session authenticator from a base64url-encoded Ed25519 seed.
    #[doc = concat!("Trellis API operation `", stringify!(from_seed_base64url), "`.")]
    pub fn from_seed_base64url(seed_b64url: &str) -> Result<Self, TrellisClientError> {
        let seed = base64url_decode(seed_b64url)?;
        if seed.len() != 32 {
            return Err(TrellisClientError::InvalidSeedLen(seed.len()));
        }
        let mut seed32 = [0u8; 32];
        seed32.copy_from_slice(&seed);
        let signing_key = SigningKey::from_bytes(&seed32);
        let public = signing_key.verifying_key().to_bytes();
        let session_key = base64url_encode(&public);
        Ok(Self {
            session_key,
            signing_key,
        })
    }

    /// Sign a domain-separated string value with `SHA-256(prefix:value)`.
    #[doc = concat!("Trellis API operation `", stringify!(sign_sha256_domain), "`.")]
    pub fn sign_sha256_domain(&self, prefix: &str, value: &str) -> String {
        let digest = sha256(format!("{prefix}:{value}").as_bytes());
        let signature: Signature = self.signing_key.sign(&digest);
        base64url_encode(&signature.to_bytes())
    }

    #[doc = concat!("Trellis API operation `", stringify!(sign_sha256_bytes), "`.")]
    pub fn sign_sha256_bytes(&self, bytes: &[u8]) -> String {
        let digest = sha256(bytes);
        let signature: Signature = self.signing_key.sign(&digest);
        base64url_encode(&signature.to_bytes())
    }

    pub(crate) fn key_id(&self) -> String {
        base64url_encode(&sha256(self.signing_key.verifying_key().as_bytes()))
    }

    pub(crate) fn nkey_pair(&self) -> Result<KeyPair, TrellisClientError> {
        KeyPair::new_from_raw(KeyPairType::User, self.signing_key.to_bytes())
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))
    }

    /// Sign one canonical protocol-owned session proof input.
    pub fn sign_session_proof(
        &self,
        input: &SessionProofInputV1,
    ) -> Result<SessionProofV1, TrellisClientError> {
        sign_session_proof_v1(input, &self.signing_key)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))
    }

    /// Return the session public key encoded as a NATS User NKey.
    pub fn session_nkey(&self) -> Result<String, TrellisClientError> {
        Ok(self.nkey_pair()?.public_key())
    }

    /// Return the inbox prefix derived from the session key.
    #[doc = concat!("Trellis API operation `", stringify!(inbox_prefix), "`.")]
    pub fn inbox_prefix(&self) -> String {
        format!(
            "_INBOX.{}",
            &self.session_key[..16.min(self.session_key.len())]
        )
    }

    /// Create the `proof` header for a signed RPC request payload.
    #[doc = concat!("Trellis API operation `", stringify!(create_proof), "`.")]
    pub fn create_proof(
        &self,
        subject: &str,
        payload: &[u8],
        iat: i64,
        request_id: &str,
    ) -> String {
        let payload_hash = sha256(payload);
        let input = build_proof_input(&self.session_key, subject, &payload_hash, iat, request_id);
        let digest = sha256(&input);
        let signature: Signature = self.signing_key.sign(&digest);
        base64url_encode(&signature.to_bytes())
    }

    /// Create the `proof` header for a signed event payload.
    #[doc = concat!("Trellis API operation `", stringify!(create_event_proof), "`.")]
    pub fn create_event_proof(
        &self,
        subject: &str,
        payload: &[u8],
        event_id: &str,
        event_time: &str,
    ) -> String {
        let payload_hash = sha256(payload);
        let input = build_event_proof_input(
            &self.session_key,
            subject,
            &payload_hash,
            event_id,
            event_time,
        );
        let digest = sha256(&input);
        let signature: Signature = self.signing_key.sign(&digest);
        base64url_encode(&signature.to_bytes())
    }
}
