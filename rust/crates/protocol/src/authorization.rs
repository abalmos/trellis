//! Pure signed authorization-context and request-proof protocol.
//!
//! Trellis pins an [`AuthorizationTrustRootV1`]. That root signs a
//! generation-numbered issuer manifest containing the authorized issuer keys.
//! An issuer signs short-lived authorization contexts, and the session key bound
//! into a verified context signs each exact request. Verification requires no network,
//! storage, wall clock, or async runtime: trust records, policy, time, and raw
//! request bytes are explicit inputs.
//!
//! Signed objects have a strict top-level shape. Forward-compatible signed data
//! belongs in `extensions`; names in `critical` fail closed unless understood.
//! Signatures cover RFC 8785 canonical JSON for the complete object with only
//! its top-level `signature` member omitted. Every signed integer, recursively
//! including extension values, is restricted to the interoperable JSON safe
//! integer range `-(2^53 - 1)..=(2^53 - 1)`.
//!
//! # Complete local decision
//!
//! The following constructs and verifies the root, current manifest, context,
//! exact permission, and context-bound request proof:
//!
//! ```
//! use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
//! use ed25519_dalek::SigningKey;
//! use serde_json::Map;
//! use trellis_protocol::{
//!     sign_authorization_context_v1, sign_authorization_event_v2, sign_authorization_request_v2,
//!     sign_issuer_manifest_v1,
//!     verify_authorization_context_v1, verify_authorization_event_v2,
//!     verify_authorization_request_v2,
//!     verify_issuer_manifest_v1, ApiSurfaceKindV1, AuthorizationAuthorityKindV1,
//!     AuthorizationAuthorityRefV1, AuthorizationIssuerManifestEntryV1,
//!     AuthorizationParticipantV1,
//!     AuthorizationPrincipalKindV1, AuthorizationPrincipalV1,
//!     AuthorizationTrustRootV1, AuthorizationVerificationPolicyV1, GrantSetV1,
//!     ParticipantKindV1, PermissionActionV1, PermissionAtomV1,
//!     PermissionTargetV1, UnsignedAuthorizationContextV1,
//!     UnsignedAuthorizationIssuerManifestV1, AUTHORIZATION_CONTEXT_FORMAT_V1,
//!     AUTHORIZATION_ISSUER_MANIFEST_FORMAT_V1,
//! };
//! use sha2::{Digest as _, Sha256};
//!
//! let root_key = SigningKey::from_bytes(&[1; 32]);
//! let issuer_key = SigningKey::from_bytes(&[2; 32]);
//! let session_key = SigningKey::from_bytes(&[3; 32]);
//! let encode_key = |key: &SigningKey| URL_SAFE_NO_PAD.encode(key.verifying_key().as_bytes());
//! let key_id = |key: &SigningKey| {
//!     URL_SAFE_NO_PAD.encode(Sha256::digest(key.verifying_key().as_bytes()))
//! };
//! let root = AuthorizationTrustRootV1::new("trellis-test", encode_key(&root_key))?;
//! let manifest = sign_issuer_manifest_v1(
//!     UnsignedAuthorizationIssuerManifestV1 {
//!         format: AUTHORIZATION_ISSUER_MANIFEST_FORMAT_V1.into(),
//!         authority: root.authority().into(),
//!         root_key_id: root.key_id().into(),
//!         generation: 7,
//!         issued_at: 1_000,
//!         not_before: 1_000,
//!         expires_at: 1_500,
//!         issuers: vec![AuthorizationIssuerManifestEntryV1 {
//!             key_id: key_id(&issuer_key),
//!             public_key: encode_key(&issuer_key),
//!         }],
//!         extensions: Map::new(),
//!         critical: vec![],
//!     },
//!     &root_key,
//! )?;
//! let policy = AuthorizationVerificationPolicyV1::new(1_100, 30, 300, 16_384, 16, 16, 7)?;
//! let manifest = verify_issuer_manifest_v1(&root, &manifest, &policy)?;
//! let permission = PermissionAtomV1::new(
//!     PermissionTargetV1::api_surface(
//!         "documents@v1",
//!         ApiSurfaceKindV1::Rpc,
//!         "Documents.Get",
//!     )?,
//!     PermissionActionV1::Call,
//! )?;
//! let context = sign_authorization_context_v1(
//!     UnsignedAuthorizationContextV1 {
//!         format: AUTHORIZATION_CONTEXT_FORMAT_V1.into(),
//!         authority: root.authority().into(),
//!         issuer_key_id: key_id(&issuer_key),
//!         issuer_manifest_generation: 7,
//!         session_id: "ses_example".into(),
//!         session_key: encode_key(&session_key),
//!         principal: AuthorizationPrincipalV1 {
//!             kind: AuthorizationPrincipalKindV1::User,
//!             id: "usr_example".into(),
//!         },
//!         participant: AuthorizationParticipantV1 {
//!             kind: ParticipantKindV1::App,
//!             id: "documents-web".into(),
//!             artifact_digest: URL_SAFE_NO_PAD.encode([4; 32]),
//!             needs_digest: URL_SAFE_NO_PAD.encode([5; 32]),
//!         },
//!         authority_ref: AuthorizationAuthorityRefV1 {
//!             kind: AuthorizationAuthorityKindV1::Identity,
//!             id: "usr_example".into(),
//!             version: 1,
//!         },
//!         deployment_id: None,
//!         instance_id: None,
//!         inbox_prefix: "_INBOX.example".into(),
//!         issued_at: 1_100,
//!         not_before: 1_100,
//!         expires_at: 1_300,
//!         grant_set: GrantSetV1::new(vec![permission.clone()]),
//!         capabilities: vec![],
//!         extensions: Map::new(),
//!         critical: vec![],
//!     },
//!     &issuer_key,
//! )?;
//! let context = verify_authorization_context_v1(&root, &manifest, &context, &policy)?;
//! assert!(context.allows(&permission));
//! let proof = sign_authorization_request_v2(
//!     context.context_digest(),
//!     "rpc.v1.Documents.Get",
//!     Some("_INBOX.example.reply"),
//!     br#"{"id":"doc-1"}"#,
//!     1_100,
//!     "req_example",
//!     &session_key,
//! )?;
//! let request = verify_authorization_request_v2(
//!     &context,
//!     "rpc.v1.Documents.Get",
//!     Some("_INBOX.example.reply"),
//!     br#"{"id":"doc-1"}"#,
//!     1_100,
//!     "req_example",
//!     &proof,
//!     &policy,
//!     &[permission.clone()],
//!     &[],
//! )?;
//! assert_eq!(request.context().principal().id, "usr_example");
//! let event_proof = sign_authorization_event_v2(
//!     context.context_digest(),
//!     "events.v1.Documents.Changed.doc-1",
//!     br#"{"id":"doc-1"}"#,
//!     "evt_example",
//!     "1970-01-01T00:19:10Z",
//!     &session_key,
//! )?;
//! let event = verify_authorization_event_v2(
//!     &context,
//!     "events.v1.Documents.Changed.doc-1",
//!     br#"{"id":"doc-1"}"#,
//!     "evt_example",
//!     "1970-01-01T00:19:10Z",
//!     &event_proof,
//!     &policy,
//!     &[permission],
//!     &[],
//!     None,
//! )?;
//! assert_eq!(event.publisher().participant_id, "documents-web");
//! # Ok::<(), trellis_protocol::ProtocolError>(())
//! ```

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signature, Signer as _, SigningKey, VerifyingKey};
use jsonptr::PointerBuf;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest as _, Sha256};

use crate::{
    canonicalize_json, AuthorizationErrorCodeV1, GrantSetV1, ParticipantKindV1, PermissionAtomV1,
    ProtocolError,
};

/// Pinned trust-root wire format.
pub const AUTHORIZATION_TRUST_ROOT_FORMAT_V1: &str = "trellis.authorization-trust-root.v1";
/// Root-signed issuer-manifest wire format and signature domain.
pub const AUTHORIZATION_ISSUER_MANIFEST_FORMAT_V1: &str =
    "trellis.authorization-issuer-manifest.v1";
/// Issuer-signed authorization-context wire format and signature domain.
pub const AUTHORIZATION_CONTEXT_FORMAT_V1: &str = "trellis.authorization-context.v1";
/// Context-bound request-proof signature domain.
pub const AUTHORIZATION_REQUEST_PROOF_DOMAIN_V2: &str = "trellis.authorization-request-proof.v2";
/// Context-bound event-proof signature domain.
pub const AUTHORIZATION_EVENT_PROOF_DOMAIN_V2: &str = "trellis.authorization-event-proof.v2";

const MAXIMUM_REQUEST_ID_BYTES: usize = 256;
const MAXIMUM_EVENT_ID_BYTES: usize = 256;
const MAXIMUM_SAFE_JSON_INTEGER: i64 = 9_007_199_254_740_991;

fn authorization_error<'a>(
    code: AuthorizationErrorCodeV1,
    tokens: impl IntoIterator<Item = &'a str>,
    message: impl Into<String>,
) -> ProtocolError {
    ProtocolError::Authorization {
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
    code: AuthorizationErrorCodeV1,
) -> Result<[u8; N], ProtocolError> {
    if encoded.contains('=') {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidEncoding,
            path.iter().copied(),
            "padded base64url is not accepted",
        ));
    }
    let decoded = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| {
        authorization_error(
            AuthorizationErrorCodeV1::InvalidEncoding,
            path.iter().copied(),
            "value is not unpadded base64url",
        )
    })?;
    if decoded.len() != N || encode_base64url(&decoded) != encoded {
        return Err(authorization_error(
            code,
            path.iter().copied(),
            format!("value must canonically encode exactly {N} bytes"),
        ));
    }
    decoded.try_into().map_err(|_| {
        authorization_error(
            code,
            path.iter().copied(),
            format!("value must encode exactly {N} bytes"),
        )
    })
}

fn decode_verifying_key(
    encoded: &str,
    path: &[&str],
    code: AuthorizationErrorCodeV1,
) -> Result<VerifyingKey, ProtocolError> {
    VerifyingKey::from_bytes(&decode_base64url::<32>(encoded, path, code)?).map_err(|_| {
        authorization_error(
            code,
            path.iter().copied(),
            "value is not a valid Ed25519 public key",
        )
    })
}

fn derived_key_id(key: &VerifyingKey) -> String {
    encode_base64url(&sha256(key.as_bytes()))
}

fn check_key_id(declared: &str, key: &VerifyingKey, path: &[&str]) -> Result<(), ProtocolError> {
    decode_base64url::<32>(declared, path, AuthorizationErrorCodeV1::InvalidKeyId)?;
    if declared == derived_key_id(key) {
        Ok(())
    } else {
        Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidKeyId,
            path.iter().copied(),
            "declared key id does not match the public key",
        ))
    }
}

fn validate_safe_i64(value: i64, path: &[&str]) -> Result<(), ProtocolError> {
    if (-MAXIMUM_SAFE_JSON_INTEGER..=MAXIMUM_SAFE_JSON_INTEGER).contains(&value) {
        Ok(())
    } else {
        Err(authorization_error(
            AuthorizationErrorCodeV1::UnsafeJsonInteger,
            path.iter().copied(),
            "integer must be within the interoperable JSON safe-integer range",
        ))
    }
}

fn validate_safe_u64(value: u64, path: &[&str]) -> Result<(), ProtocolError> {
    if value <= MAXIMUM_SAFE_JSON_INTEGER as u64 {
        Ok(())
    } else {
        Err(authorization_error(
            AuthorizationErrorCodeV1::UnsafeJsonInteger,
            path.iter().copied(),
            "integer must be within the interoperable JSON safe-integer range",
        ))
    }
}

fn validate_safe_extension_integers(
    value: &Value,
    path: &mut Vec<String>,
) -> Result<(), ProtocolError> {
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
                return Err(ProtocolError::Authorization {
                    code: AuthorizationErrorCodeV1::UnsafeJsonInteger,
                    path: PointerBuf::from_tokens(path.iter().map(String::as_str)),
                    message: "integer must be within the interoperable JSON safe-integer range"
                        .to_owned(),
                });
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                path.push(index.to_string());
                validate_safe_extension_integers(value, path)?;
                path.pop();
            }
        }
        Value::Object(values) => {
            for (name, value) in values {
                path.push(name.clone());
                validate_safe_extension_integers(value, path)?;
                path.pop();
            }
        }
        Value::Null | Value::Bool(_) | Value::String(_) => {}
    }
    Ok(())
}

fn push_length_prefixed(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), ProtocolError> {
    let length = u32::try_from(bytes.len()).map_err(|_| {
        authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            std::iter::empty::<&str>(),
            "signature input component exceeds u32 length",
        )
    })?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(bytes);
    Ok(())
}

fn signed_json_digest(domain: &str, unsigned: &Value) -> Result<[u8; 32], ProtocolError> {
    let canonical = canonicalize_json(unsigned)?;
    let mut input = Vec::with_capacity(domain.len() + canonical.len() + 8);
    push_length_prefixed(&mut input, domain.as_bytes())?;
    push_length_prefixed(&mut input, canonical.as_bytes())?;
    Ok(sha256(&input))
}

fn complete_digest<T: Serialize>(value: &T) -> Result<String, ProtocolError> {
    let canonical = canonicalize_json(&serde_json::to_value(value)?)?;
    Ok(encode_base64url(&sha256(canonical.as_bytes())))
}

fn validate_text(value: &str, path: &[&str]) -> Result<(), ProtocolError> {
    if value.is_empty()
        || value.trim() != value
        || value.chars().any(|character| character.is_ascii_control())
    {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            path.iter().copied(),
            "value must be nonempty protocol-safe text",
        ));
    }
    Ok(())
}

fn validate_inbox_prefix(value: &str) -> Result<(), ProtocolError> {
    validate_text(value, &["inboxPrefix"])?;
    if value.starts_with('.')
        || value.ends_with('.')
        || value.split('.').any(str::is_empty)
        || value.contains('*')
        || value.contains('>')
        || value.chars().any(char::is_whitespace)
    {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["inboxPrefix"],
            "inbox prefix must contain only safe literal NATS tokens",
        ));
    }
    Ok(())
}

fn is_utf16_strictly_sorted(values: &[String]) -> bool {
    values
        .windows(2)
        .all(|pair| pair[0].encode_utf16().cmp(pair[1].encode_utf16()).is_lt())
}

fn validate_extensions(
    extensions: &Map<String, Value>,
    critical: &[String],
) -> Result<(), ProtocolError> {
    for (index, name) in critical.iter().enumerate() {
        validate_text(name, &["critical", &index.to_string()])?;
    }
    if !is_utf16_strictly_sorted(critical) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::NonCanonicalSet,
            ["critical"],
            "critical extension names must be UTF-16 sorted and unique",
        ));
    }
    if let Some((index, _name)) = critical
        .iter()
        .enumerate()
        .find(|(_, name)| !extensions.contains_key(name.as_str()))
    {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["critical", &index.to_string()],
            "critical extension is absent from extensions",
        ));
    }
    if let Some(name) = critical.first() {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::UnknownCriticalExtension,
            ["extensions", name],
            "critical extension is not understood by this protocol version",
        ));
    }
    validate_safe_extension_integers(
        &Value::Object(extensions.clone()),
        &mut vec!["extensions".to_owned()],
    )?;
    Ok(())
}

fn validate_window(
    issued_at: i64,
    not_before: i64,
    expires_at: i64,
    prefix: &[&str],
) -> Result<(), ProtocolError> {
    validate_safe_i64(
        issued_at,
        &prefix
            .iter()
            .copied()
            .chain(["issuedAt"])
            .collect::<Vec<_>>(),
    )?;
    validate_safe_i64(
        not_before,
        &prefix
            .iter()
            .copied()
            .chain(["notBefore"])
            .collect::<Vec<_>>(),
    )?;
    validate_safe_i64(
        expires_at,
        &prefix
            .iter()
            .copied()
            .chain(["expiresAt"])
            .collect::<Vec<_>>(),
    )?;
    if not_before <= issued_at && issued_at <= expires_at && not_before < expires_at {
        Ok(())
    } else {
        Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidValidityWindow,
            prefix.iter().copied().chain(["expiresAt"]),
            "validity must satisfy notBefore <= issuedAt <= expiresAt and be nonempty",
        ))
    }
}

fn verify_signature(
    key: &VerifyingKey,
    digest: &[u8; 32],
    encoded: &str,
    code: AuthorizationErrorCodeV1,
) -> Result<(), ProtocolError> {
    let bytes = decode_base64url::<64>(encoded, &["signature"], code)?;
    key.verify_strict(digest, &Signature::from_bytes(&bytes))
        .map_err(|_| authorization_error(code, ["signature"], "signature verification failed"))
}

/// Explicit security policy supplied to pure authorization verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationVerificationPolicyV1 {
    /// Verification time supplied by the caller.
    pub now_unix_seconds: i64,
    /// Allowed symmetric time skew.
    pub allowed_clock_skew_seconds: u32,
    /// Maximum authorization-context lease duration.
    pub maximum_context_lifetime_seconds: u32,
    /// Maximum canonical signed-context JSON size in UTF-8 bytes.
    pub maximum_context_bytes: usize,
    /// Maximum exact permissions in one context.
    pub maximum_permissions: usize,
    /// Maximum platform capabilities in one context.
    pub maximum_capabilities: usize,
    /// Lowest issuer-manifest generation already accepted by the caller.
    pub minimum_manifest_generation: u64,
}

impl AuthorizationVerificationPolicyV1 {
    /// Construct an explicit verification policy.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Authorization`] if any security limit is zero.
    pub fn new(
        now_unix_seconds: i64,
        allowed_clock_skew_seconds: u32,
        maximum_context_lifetime_seconds: u32,
        maximum_context_bytes: usize,
        maximum_permissions: usize,
        maximum_capabilities: usize,
        minimum_manifest_generation: u64,
    ) -> Result<Self, ProtocolError> {
        let policy = Self {
            now_unix_seconds,
            allowed_clock_skew_seconds,
            maximum_context_lifetime_seconds,
            maximum_context_bytes,
            maximum_permissions,
            maximum_capabilities,
            minimum_manifest_generation,
        };
        validate_policy(&policy)?;
        Ok(policy)
    }
}

fn validate_policy(policy: &AuthorizationVerificationPolicyV1) -> Result<(), ProtocolError> {
    validate_safe_i64(policy.now_unix_seconds, &["nowUnixSeconds"])?;
    validate_safe_u64(
        policy.minimum_manifest_generation,
        &["minimumManifestGeneration"],
    )?;
    if policy.maximum_context_lifetime_seconds == 0
        || policy.maximum_context_bytes == 0
        || policy.maximum_permissions == 0
        || policy.maximum_capabilities == 0
    {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            std::iter::empty::<&str>(),
            "verification limits must be nonzero",
        ));
    }
    Ok(())
}

/// A pinned Ed25519 authorization trust root.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorizationTrustRootV1 {
    format: String,
    authority: String,
    key_id: String,
    public_key: String,
}

impl AuthorizationTrustRootV1 {
    /// Construct a trust root and derive its key id from the public key.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Authorization`] for an invalid authority or key.
    pub fn new(
        authority: impl Into<String>,
        public_key: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        let authority = authority.into();
        let public_key = public_key.into();
        validate_text(&authority, &["authority"])?;
        let key = decode_verifying_key(
            &public_key,
            &["publicKey"],
            AuthorizationErrorCodeV1::InvalidPublicKey,
        )?;
        Ok(Self {
            format: AUTHORIZATION_TRUST_ROOT_FORMAT_V1.to_owned(),
            authority,
            key_id: derived_key_id(&key),
            public_key,
        })
    }

    /// Parse and strictly validate a trust-root JSON value.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Authorization`] for an unknown field, wrong
    /// format, malformed key, or mismatched derived key id.
    pub fn parse(value: &Value) -> Result<Self, ProtocolError> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        struct WireTrustRootV1 {
            format: String,
            authority: String,
            key_id: String,
            public_key: String,
        }

        let wire: WireTrustRootV1 = serde_json::from_value(value.clone()).map_err(|_| {
            authorization_error(
                AuthorizationErrorCodeV1::InvalidFormat,
                std::iter::empty::<&str>(),
                "trust root has an invalid strict object shape",
            )
        })?;
        let root = Self {
            format: wire.format,
            authority: wire.authority,
            key_id: wire.key_id,
            public_key: wire.public_key,
        };
        if root.format != AUTHORIZATION_TRUST_ROOT_FORMAT_V1 {
            return Err(authorization_error(
                AuthorizationErrorCodeV1::InvalidFormat,
                ["format"],
                "unsupported trust-root format",
            ));
        }
        validate_text(&root.authority, &["authority"])?;
        let key = root.verifying_key()?;
        check_key_id(&root.key_id, &key, &["keyId"])?;
        Ok(root)
    }

    /// Return the authority namespace pinned by this root.
    pub fn authority(&self) -> &str {
        &self.authority
    }

    /// Return the root's content-derived key id.
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    /// Return the normalized root value.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Json`] if serialization fails.
    pub fn normalized_value(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::to_value(self)?)
    }

    /// Return canonical RFC 8785 JSON for this root.
    ///
    /// # Errors
    ///
    /// Returns a JSON canonicalization error if serialization fails.
    pub fn canonical_json(&self) -> Result<String, ProtocolError> {
        canonicalize_json(&self.normalized_value()?)
    }

    /// Return the root's canonical SHA-256/base64url digest.
    ///
    /// # Errors
    ///
    /// Returns a JSON canonicalization error if serialization fails.
    pub fn digest(&self) -> Result<String, ProtocolError> {
        complete_digest(self)
    }

    /// Decode the pinned Ed25519 verification key.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Authorization`] if the encoded key is invalid.
    pub fn verifying_key(&self) -> Result<VerifyingKey, ProtocolError> {
        decode_verifying_key(
            &self.public_key,
            &["publicKey"],
            AuthorizationErrorCodeV1::InvalidPublicKey,
        )
    }
}

/// One directly root-authorized issuer key selected by a manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorizationIssuerManifestEntryV1 {
    /// Content-derived issuer key id.
    pub key_id: String,
    /// Unpadded base64url Ed25519 issuer public key.
    pub public_key: String,
}

/// Unsigned root-authorized issuer registry fields.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UnsignedAuthorizationIssuerManifestV1 {
    /// Wire format.
    pub format: String,
    /// Authority namespace.
    pub authority: String,
    /// Content-derived root key id.
    pub root_key_id: String,
    /// Positive monotonic manifest generation.
    pub generation: u64,
    /// Manifest issue time.
    pub issued_at: i64,
    /// Inclusive lower validity bound.
    pub not_before: i64,
    /// Inclusive upper validity bound.
    pub expires_at: i64,
    /// Canonically ordered issuer entries.
    pub issuers: Vec<AuthorizationIssuerManifestEntryV1>,
    /// Signed extension values.
    pub extensions: Map<String, Value>,
    /// Canonical critical extension names.
    pub critical: Vec<String>,
}

/// Root-signed issuer manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SignedAuthorizationIssuerManifestV1 {
    /// Unsigned manifest fields.
    #[serde(flatten)]
    pub unsigned: UnsignedAuthorizationIssuerManifestV1,
    /// Root signature.
    pub signature: String,
}

impl SignedAuthorizationIssuerManifestV1 {
    /// Return the digest of the complete signed canonical manifest.
    ///
    /// # Errors
    ///
    /// Returns a JSON canonicalization error if serialization fails.
    pub fn digest(&self) -> Result<String, ProtocolError> {
        complete_digest(self)
    }
}

/// A manifest whose root signature, generation, and current validity succeeded.
#[derive(Clone, Debug)]
pub struct VerifiedAuthorizationIssuerManifestV1 {
    manifest: SignedAuthorizationIssuerManifestV1,
    authority: String,
    root_key_id: String,
}

impl VerifiedAuthorizationIssuerManifestV1 {
    /// Return the signed manifest.
    pub fn manifest(&self) -> &SignedAuthorizationIssuerManifestV1 {
        &self.manifest
    }

    /// Return the authority namespace verified by the pinned root.
    pub fn authority(&self) -> &str {
        &self.authority
    }

    /// Return the content-derived root key id that signed this manifest.
    pub fn root_key_id(&self) -> &str {
        &self.root_key_id
    }

    /// Return the verified monotonic manifest generation.
    pub fn generation(&self) -> u64 {
        self.manifest.unsigned.generation
    }

    /// Return the complete signed canonical manifest digest.
    ///
    /// # Errors
    ///
    /// Returns a JSON canonicalization error if serialization fails.
    pub fn digest(&self) -> Result<String, ProtocolError> {
        self.manifest.digest()
    }
}

/// Return the domain-separated signing digest for an issuer manifest.
///
/// # Errors
///
/// Returns a JSON canonicalization error if serialization fails.
pub fn issuer_manifest_signing_digest_v1(
    manifest: &UnsignedAuthorizationIssuerManifestV1,
) -> Result<[u8; 32], ProtocolError> {
    validate_manifest_fields(manifest)?;
    signed_json_digest(
        AUTHORIZATION_ISSUER_MANIFEST_FORMAT_V1,
        &serde_json::to_value(manifest)?,
    )
}

/// Sign an issuer manifest with the pinned root key.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] for invalid manifest structure, or
/// a JSON canonicalization error.
pub fn sign_issuer_manifest_v1(
    manifest: UnsignedAuthorizationIssuerManifestV1,
    root_key: &SigningKey,
) -> Result<SignedAuthorizationIssuerManifestV1, ProtocolError> {
    validate_manifest_fields(&manifest)?;
    check_key_id(
        &manifest.root_key_id,
        &root_key.verifying_key(),
        &["rootKeyId"],
    )?;
    let digest = issuer_manifest_signing_digest_v1(&manifest)?;
    Ok(SignedAuthorizationIssuerManifestV1 {
        unsigned: manifest,
        signature: encode_base64url(&root_key.sign(&digest).to_bytes()),
    })
}

/// Strictly parse a signed issuer manifest.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] for an invalid strict object shape,
/// signature encoding, or intrinsic manifest invariant.
pub fn parse_issuer_manifest_v1(
    value: &Value,
) -> Result<SignedAuthorizationIssuerManifestV1, ProtocolError> {
    let manifest: SignedAuthorizationIssuerManifestV1 = serde_json::from_value(value.clone())
        .map_err(|_| {
            authorization_error(
                AuthorizationErrorCodeV1::InvalidFormat,
                std::iter::empty::<&str>(),
                "issuer manifest has an invalid strict object shape",
            )
        })?;
    validate_manifest_fields(&manifest.unsigned)?;
    decode_base64url::<64>(
        &manifest.signature,
        &["signature"],
        AuthorizationErrorCodeV1::InvalidSignature,
    )?;
    Ok(manifest)
}

fn validate_manifest_fields(
    manifest: &UnsignedAuthorizationIssuerManifestV1,
) -> Result<(), ProtocolError> {
    if manifest.format != AUTHORIZATION_ISSUER_MANIFEST_FORMAT_V1 || manifest.generation == 0 {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            [if manifest.generation == 0 {
                "generation"
            } else {
                "format"
            }],
            "manifest format must be supported and generation must be positive",
        ));
    }
    validate_text(&manifest.authority, &["authority"])?;
    decode_base64url::<32>(
        &manifest.root_key_id,
        &["rootKeyId"],
        AuthorizationErrorCodeV1::InvalidKeyId,
    )?;
    validate_safe_u64(manifest.generation, &["generation"])?;
    validate_window(
        manifest.issued_at,
        manifest.not_before,
        manifest.expires_at,
        &[],
    )?;
    for (index, entry) in manifest.issuers.iter().enumerate() {
        let path = index.to_string();
        let key = decode_verifying_key(
            &entry.public_key,
            &["issuers", &path, "publicKey"],
            AuthorizationErrorCodeV1::InvalidPublicKey,
        )?;
        check_key_id(&entry.key_id, &key, &["issuers", &path, "keyId"])?;
    }
    if !manifest.issuers.windows(2).all(|pair| {
        pair[0]
            .key_id
            .encode_utf16()
            .cmp(pair[1].key_id.encode_utf16())
            .is_lt()
    }) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::NonCanonicalSet,
            ["issuers"],
            "issuer entries must be UTF-16 sorted by unique key id",
        ));
    }
    validate_extensions(&manifest.extensions, &manifest.critical)
}

/// Verify a root-signed issuer manifest without external state access.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] for invalid structure, authority,
/// rollback, validity, extensions, or root signature.
pub fn verify_issuer_manifest_v1(
    root: &AuthorizationTrustRootV1,
    manifest: &SignedAuthorizationIssuerManifestV1,
    policy: &AuthorizationVerificationPolicyV1,
) -> Result<VerifiedAuthorizationIssuerManifestV1, ProtocolError> {
    validate_policy(policy)?;
    validate_manifest_fields(&manifest.unsigned)?;
    if manifest.unsigned.authority != root.authority || manifest.unsigned.root_key_id != root.key_id
    {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::WrongAuthority,
            ["authority"],
            "manifest authority or root key id does not match the pinned root",
        ));
    }
    if manifest.unsigned.generation < policy.minimum_manifest_generation {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ManifestRollback,
            ["generation"],
            "manifest generation is below the accepted minimum",
        ));
    }
    let now = i128::from(policy.now_unix_seconds);
    let skew = i128::from(policy.allowed_clock_skew_seconds);
    if now + skew < i128::from(manifest.unsigned.not_before) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ManifestNotYetValid,
            ["notBefore"],
            "issuer manifest is not yet valid",
        ));
    }
    if now - skew > i128::from(manifest.unsigned.expires_at) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ManifestExpired,
            ["expiresAt"],
            "issuer manifest has expired",
        ));
    }
    verify_signature(
        &root.verifying_key()?,
        &issuer_manifest_signing_digest_v1(&manifest.unsigned)?,
        &manifest.signature,
        AuthorizationErrorCodeV1::InvalidSignature,
    )?;
    Ok(VerifiedAuthorizationIssuerManifestV1 {
        authority: manifest.unsigned.authority.clone(),
        root_key_id: manifest.unsigned.root_key_id.clone(),
        manifest: manifest.clone(),
    })
}

/// Stable principal classes represented by an authorization context.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthorizationPrincipalKindV1 {
    /// A Trellis user account.
    User,
    /// A hosted service runtime.
    Service,
    /// A device runtime.
    Device,
}

/// Principal identity bound into an authorization context.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorizationPrincipalV1 {
    /// Principal class.
    pub kind: AuthorizationPrincipalKindV1,
    /// Stable authorization subject id.
    pub id: String,
}

/// Exact participant artifact and accepted-needs evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorizationParticipantV1 {
    /// Participant class.
    pub kind: ParticipantKindV1,
    /// Stable participant id.
    pub id: String,
    /// Digest of the exact participant artifact.
    pub artifact_digest: String,
    /// Digest of accepted participant needs.
    pub needs_digest: String,
}

/// Durable authority record classes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthorizationAuthorityKindV1 {
    /// Delegated identity authority.
    Identity,
    /// Deployment-owned authority.
    Deployment,
}

/// Durable authority record and version used to materialize a context.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorizationAuthorityRefV1 {
    /// Authority class.
    pub kind: AuthorizationAuthorityKindV1,
    /// Durable authority record id.
    pub id: String,
    /// Positive desired-authority version.
    pub version: u64,
}

/// Complete unsigned authorization-context fields.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UnsignedAuthorizationContextV1 {
    /// Wire format.
    pub format: String,
    /// Authority namespace.
    pub authority: String,
    /// Content-derived signing issuer key id.
    pub issuer_key_id: String,
    /// Exact root-signed manifest generation used to issue this context.
    pub issuer_manifest_generation: u64,
    /// Stable session id, distinct from key material.
    pub session_id: String,
    /// Session Ed25519 public key.
    pub session_key: String,
    /// Stable caller principal.
    pub principal: AuthorizationPrincipalV1,
    /// Exact participant evidence.
    pub participant: AuthorizationParticipantV1,
    /// Durable source authority and version.
    pub authority_ref: AuthorizationAuthorityRefV1,
    /// Deployment id for service and applicable device contexts.
    pub deployment_id: Option<String>,
    /// Runtime instance id for service and applicable device contexts.
    pub instance_id: Option<String>,
    /// Caller reply-inbox prefix.
    pub inbox_prefix: String,
    /// Context issue time.
    pub issued_at: i64,
    /// Inclusive lower validity bound.
    pub not_before: i64,
    /// Inclusive upper validity bound.
    pub expires_at: i64,
    /// Exact machine permission authority.
    pub grant_set: GrantSetV1,
    /// Canonical platform capability keys.
    pub capabilities: Vec<String>,
    /// Signed extension values.
    pub extensions: Map<String, Value>,
    /// Canonical critical extension names.
    pub critical: Vec<String>,
}

/// Issuer-signed authorization context.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SignedAuthorizationContextV1 {
    /// Unsigned context fields.
    #[serde(flatten)]
    pub unsigned: UnsignedAuthorizationContextV1,
    /// Issuer signature.
    pub signature: String,
}

impl SignedAuthorizationContextV1 {
    /// Return the digest of complete signed canonical context JSON.
    ///
    /// # Errors
    ///
    /// Returns a JSON canonicalization error if serialization fails.
    pub fn digest(&self) -> Result<String, ProtocolError> {
        complete_digest(self)
    }
}

/// Return the domain-separated signing digest for an authorization context.
///
/// # Errors
///
/// Returns a JSON canonicalization error if serialization fails.
pub fn authorization_context_signing_digest_v1(
    context: &UnsignedAuthorizationContextV1,
) -> Result<[u8; 32], ProtocolError> {
    validate_context_fields(context, None)?;
    signed_json_digest(
        AUTHORIZATION_CONTEXT_FORMAT_V1,
        &serde_json::to_value(context)?,
    )
}

/// Sign a short-lived authorization context with an issuer key.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] for invalid context structure, or a
/// JSON canonicalization error.
pub fn sign_authorization_context_v1(
    context: UnsignedAuthorizationContextV1,
    issuer_key: &SigningKey,
) -> Result<SignedAuthorizationContextV1, ProtocolError> {
    validate_context_fields(&context, None)?;
    decode_verifying_key(
        &context.session_key,
        &["sessionKey"],
        AuthorizationErrorCodeV1::InvalidSessionKey,
    )?;
    check_key_id(
        &context.issuer_key_id,
        &issuer_key.verifying_key(),
        &["issuerKeyId"],
    )?;
    let digest = authorization_context_signing_digest_v1(&context)?;
    Ok(SignedAuthorizationContextV1 {
        unsigned: context,
        signature: encode_base64url(&issuer_key.sign(&digest).to_bytes()),
    })
}

/// Strictly parse an authorization context and reject noncanonical set identity.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] for unknown top-level fields,
/// malformed keys/signatures, or noncanonical set-like arrays.
pub fn parse_authorization_context_v1(
    value: &Value,
) -> Result<SignedAuthorizationContextV1, ProtocolError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    struct WireSignedAuthorizationContextV1 {
        #[serde(flatten)]
        unsigned: UnsignedAuthorizationContextV1,
        signature: String,
    }

    let wire: WireSignedAuthorizationContextV1 =
        serde_json::from_value(value.clone()).map_err(|_| {
            authorization_error(
                AuthorizationErrorCodeV1::InvalidFormat,
                std::iter::empty::<&str>(),
                "authorization context has an invalid strict object shape",
            )
        })?;
    let context = SignedAuthorizationContextV1 {
        unsigned: wire.unsigned,
        signature: wire.signature,
    };
    validate_context_fields(&context.unsigned, Some(value))?;
    decode_verifying_key(
        &context.unsigned.session_key,
        &["sessionKey"],
        AuthorizationErrorCodeV1::InvalidSessionKey,
    )?;
    decode_base64url::<64>(
        &context.signature,
        &["signature"],
        AuthorizationErrorCodeV1::InvalidSignature,
    )?;
    Ok(context)
}

fn validate_context_fields(
    context: &UnsignedAuthorizationContextV1,
    authored: Option<&Value>,
) -> Result<(), ProtocolError> {
    if context.format != AUTHORIZATION_CONTEXT_FORMAT_V1 {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["format"],
            "unsupported authorization-context format",
        ));
    }
    validate_text(&context.authority, &["authority"])?;
    validate_text(&context.session_id, &["sessionId"])?;
    validate_text(&context.principal.id, &["principal", "id"])?;
    validate_text(&context.participant.id, &["participant", "id"])?;
    validate_text(&context.authority_ref.id, &["authorityRef", "id"])?;
    decode_base64url::<32>(
        &context.issuer_key_id,
        &["issuerKeyId"],
        AuthorizationErrorCodeV1::InvalidKeyId,
    )?;
    validate_safe_u64(
        context.issuer_manifest_generation,
        &["issuerManifestGeneration"],
    )?;
    if context.issuer_manifest_generation == 0 {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["issuerManifestGeneration"],
            "issuer manifest generation must be positive",
        ));
    }
    validate_safe_u64(context.authority_ref.version, &["authorityRef", "version"])?;
    if context.authority_ref.version == 0 {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["authorityRef", "version"],
            "authority version must be positive",
        ));
    }
    match context.principal.kind {
        AuthorizationPrincipalKindV1::User => {
            if !matches!(
                context.participant.kind,
                ParticipantKindV1::App | ParticipantKindV1::Agent
            ) || context.authority_ref.kind != AuthorizationAuthorityKindV1::Identity
                || context.deployment_id.is_some()
                || context.instance_id.is_some()
            {
                return Err(authorization_error(
                    AuthorizationErrorCodeV1::InvalidFormat,
                    ["authorityRef", "kind"],
                    "user contexts require identity authority, an app or agent, and no deployment instance",
                ));
            }
        }
        AuthorizationPrincipalKindV1::Service => {
            if context.participant.kind != ParticipantKindV1::Service
                || context.authority_ref.kind != AuthorizationAuthorityKindV1::Deployment
                || context.deployment_id.is_none()
                || context.instance_id.is_none()
            {
                return Err(authorization_error(
                    AuthorizationErrorCodeV1::InvalidFormat,
                    ["participant"],
                    "service contexts require service participant, deploymentId, and instanceId",
                ));
            }
        }
        AuthorizationPrincipalKindV1::Device => {
            if context.participant.kind != ParticipantKindV1::Device
                || context.authority_ref.kind != AuthorizationAuthorityKindV1::Deployment
                || context.deployment_id.is_some() != context.instance_id.is_some()
            {
                return Err(authorization_error(
                    AuthorizationErrorCodeV1::InvalidFormat,
                    ["authorityRef", "kind"],
                    "device contexts require deployment authority and paired deployment/instance ids",
                ));
            }
        }
    }
    if let Some(deployment_id) = &context.deployment_id {
        validate_text(deployment_id, &["deploymentId"])?;
    }
    if let Some(instance_id) = &context.instance_id {
        validate_text(instance_id, &["instanceId"])?;
    }
    for (digest, field) in [
        (&context.participant.artifact_digest, "artifactDigest"),
        (&context.participant.needs_digest, "needsDigest"),
    ] {
        decode_base64url::<32>(
            digest,
            &["participant", field],
            AuthorizationErrorCodeV1::InvalidEncoding,
        )?;
    }
    validate_inbox_prefix(&context.inbox_prefix)?;
    validate_window(
        context.issued_at,
        context.not_before,
        context.expires_at,
        &[],
    )?;
    for (index, capability) in context.capabilities.iter().enumerate() {
        validate_text(capability, &["capabilities", &index.to_string()])?;
    }
    if !is_utf16_strictly_sorted(&context.capabilities) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::NonCanonicalSet,
            ["capabilities"],
            "capabilities must be UTF-16 sorted and unique",
        ));
    }
    if let Some(authored_grant) = authored.and_then(|value| value.get("grantSet")) {
        if serde_json::to_value(&context.grant_set)? != *authored_grant {
            return Err(authorization_error(
                AuthorizationErrorCodeV1::NonCanonicalSet,
                ["grantSet", "permissions"],
                "permission atoms must already be in canonical order without duplicates",
            ));
        }
    }
    validate_extensions(&context.extensions, &context.critical)
}

/// Compute the deterministic refresh time for a signed authorization context.
///
/// Jitter is derived from the canonical context digest and can only move the
/// refresh earlier than the configured safety lead. The same context and policy
/// therefore produce the same schedule in every runtime.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] if the digest is malformed, the
/// signed time window is invalid, or the policy cannot schedule a refresh no
/// earlier than issuance and strictly before expiry.
pub fn authorization_context_refresh_at_v1(
    context_digest: &str,
    issued_at: i64,
    not_before: i64,
    expires_at: i64,
    refresh_lead_seconds: u32,
    refresh_jitter_seconds: u32,
) -> Result<i64, ProtocolError> {
    validate_safe_i64(issued_at, &["issuedAt"])?;
    validate_safe_i64(not_before, &["notBefore"])?;
    validate_safe_i64(expires_at, &["expiresAt"])?;
    if not_before > issued_at || issued_at >= expires_at || refresh_lead_seconds == 0 {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["refreshAt"],
            "authorization context cannot be refreshed within its signed window",
        ));
    }
    let digest = decode_base64url::<32>(
        context_digest,
        &["contextDigest"],
        AuthorizationErrorCodeV1::InvalidEncoding,
    )?;
    let jitter_range = u64::from(refresh_jitter_seconds) + 1;
    let jitter = i64::try_from(
        u64::from_be_bytes(digest[..8].try_into().map_err(|_| {
            authorization_error(
                AuthorizationErrorCodeV1::InvalidEncoding,
                ["contextDigest"],
                "authorization context digest is invalid",
            )
        })?) % jitter_range,
    )
    .map_err(|_| {
        authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["refreshAt"],
            "authorization refresh jitter overflows",
        )
    })?;
    let refresh_at = expires_at
        .checked_sub(i64::from(refresh_lead_seconds))
        .and_then(|value| value.checked_sub(jitter))
        .ok_or_else(|| {
            authorization_error(
                AuthorizationErrorCodeV1::InvalidFormat,
                ["refreshAt"],
                "authorization refresh schedule overflows",
            )
        })?;
    if refresh_at < issued_at || refresh_at >= expires_at {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["refreshAt"],
            "authorization refresh policy has no usable window",
        ));
    }
    Ok(refresh_at)
}

/// Authorization context whose complete trust chain and lease have verified.
#[derive(Clone, Debug)]
pub struct VerifiedAuthorizationContextV1 {
    context: SignedAuthorizationContextV1,
    session_key: VerifyingKey,
    context_digest: String,
}

impl VerifiedAuthorizationContextV1 {
    /// Return the signed authority namespace.
    pub fn authority(&self) -> &str {
        &self.context.unsigned.authority
    }

    /// Return the stable caller principal.
    pub fn principal(&self) -> &AuthorizationPrincipalV1 {
        &self.context.unsigned.principal
    }

    /// Return the exact participant evidence.
    pub fn participant(&self) -> &AuthorizationParticipantV1 {
        &self.context.unsigned.participant
    }

    /// Return the durable authority record and version that produced this context.
    pub fn authority_ref(&self) -> &AuthorizationAuthorityRefV1 {
        &self.context.unsigned.authority_ref
    }

    /// Return the deployment id for a service or applicable device context.
    pub fn deployment_id(&self) -> Option<&str> {
        self.context.unsigned.deployment_id.as_deref()
    }

    /// Return the runtime instance id for a service or applicable device context.
    pub fn instance_id(&self) -> Option<&str> {
        self.context.unsigned.instance_id.as_deref()
    }

    /// Return the stable session id.
    pub fn session_id(&self) -> &str {
        &self.context.unsigned.session_id
    }

    /// Return the context-bound session verification key.
    pub fn session_key(&self) -> &VerifyingKey {
        &self.session_key
    }

    /// Return the permitted reply-inbox prefix.
    pub fn inbox_prefix(&self) -> &str {
        &self.context.unsigned.inbox_prefix
    }

    /// Return exact machine grants.
    pub fn grant_set(&self) -> &GrantSetV1 {
        &self.context.unsigned.grant_set
    }

    /// Return canonical platform capability keys.
    pub fn capabilities(&self) -> &[String] {
        &self.context.unsigned.capabilities
    }

    /// Return the complete signed canonical context digest.
    pub fn context_digest(&self) -> &str {
        &self.context_digest
    }

    /// Return the signed context issue time.
    pub fn issued_at(&self) -> i64 {
        self.context.unsigned.issued_at
    }

    /// Return the signed context lower validity bound.
    pub fn not_before(&self) -> i64 {
        self.context.unsigned.not_before
    }

    /// Return the signed context lease expiry.
    pub fn expires_at(&self) -> i64 {
        self.context.unsigned.expires_at
    }

    /// Return the complete immutable signed context.
    pub fn signed_context(&self) -> &SignedAuthorizationContextV1 {
        &self.context
    }

    /// Test exact permission membership.
    pub fn allows(&self, permission: &PermissionAtomV1) -> bool {
        self.context
            .unsigned
            .grant_set
            .permissions()
            .contains(permission)
    }

    /// Test whether every exact permission is present.
    pub fn allows_all(&self, permissions: &[PermissionAtomV1]) -> bool {
        permissions.iter().all(|permission| self.allows(permission))
    }

    /// Test exact platform capability membership.
    pub fn has_capability(&self, key: &str) -> bool {
        self.context
            .unsigned
            .capabilities
            .binary_search_by(|candidate| candidate.encode_utf16().cmp(key.encode_utf16()))
            .is_ok()
    }

    /// Test whether every exact platform capability is present.
    pub fn has_all_capabilities(&self, keys: &[String]) -> bool {
        keys.iter().all(|key| self.has_capability(key))
    }
}

/// Verify the complete root, manifest, and context trust chain.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] if the issuer is missing, the
/// manifest generation differs, a validity bound is exceeded, collection limits
/// fail, or the issuer signature is invalid.
pub fn verify_authorization_context_v1(
    root: &AuthorizationTrustRootV1,
    verified_manifest: &VerifiedAuthorizationIssuerManifestV1,
    context: &SignedAuthorizationContextV1,
    policy: &AuthorizationVerificationPolicyV1,
) -> Result<VerifiedAuthorizationContextV1, ProtocolError> {
    validate_policy(policy)?;
    validate_context_fields(&context.unsigned, None)?;
    let context_size = canonicalize_json(&serde_json::to_value(context)?)?.len();
    if context_size > policy.maximum_context_bytes {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextTooLarge,
            std::iter::empty::<&str>(),
            "canonical signed context exceeds policy size",
        ));
    }
    if context.unsigned.grant_set.permissions().len() > policy.maximum_permissions {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["grantSet", "permissions"],
            "context exceeds the permission limit",
        ));
    }
    if context.unsigned.capabilities.len() > policy.maximum_capabilities {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["capabilities"],
            "context exceeds the capability limit",
        ));
    }
    if verified_manifest.authority != root.authority || verified_manifest.root_key_id != root.key_id
    {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::WrongAuthority,
            ["authority"],
            "verified manifest is not bound to the supplied pinned root",
        ));
    }
    let manifest = &verified_manifest.manifest.unsigned;
    if context.unsigned.issuer_manifest_generation != manifest.generation {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ManifestGenerationMismatch,
            ["issuerManifestGeneration"],
            "context issuer manifest generation does not match the verified manifest",
        ));
    }
    if verified_manifest.generation() < policy.minimum_manifest_generation {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ManifestRollback,
            ["generation"],
            "verified manifest generation is below the current accepted minimum",
        ));
    }
    if context.unsigned.authority != root.authority {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::WrongAuthority,
            ["authority"],
            "context authority does not match the pinned root",
        ));
    }
    let (issuer_index, entry) = manifest
        .issuers
        .iter()
        .enumerate()
        .find(|(_, entry)| entry.key_id == context.unsigned.issuer_key_id)
        .ok_or_else(|| {
            authorization_error(
                AuthorizationErrorCodeV1::IssuerNotListed,
                ["issuerKeyId"],
                "context issuer is absent from the current manifest",
            )
        })?;
    let issuer_key = decode_verifying_key(
        &entry.public_key,
        &["issuers", &issuer_index.to_string(), "publicKey"],
        AuthorizationErrorCodeV1::InvalidPublicKey,
    )?;
    check_key_id(
        &entry.key_id,
        &issuer_key,
        &["issuers", &issuer_index.to_string(), "keyId"],
    )?;
    let lifetime =
        i128::from(context.unsigned.expires_at) - i128::from(context.unsigned.not_before);
    if lifetime > i128::from(policy.maximum_context_lifetime_seconds) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextLifetimeExceeded,
            ["expiresAt"],
            "context lifetime exceeds policy",
        ));
    }
    if context.unsigned.not_before < manifest.not_before
        || context.unsigned.expires_at > manifest.expires_at
    {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextOutlivesManifest,
            ["expiresAt"],
            "context validity is not contained by the current manifest",
        ));
    }
    let now = i128::from(policy.now_unix_seconds);
    let skew = i128::from(policy.allowed_clock_skew_seconds);
    if now + skew < i128::from(context.unsigned.not_before) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextNotYetValid,
            ["notBefore"],
            "authorization context is not yet valid",
        ));
    }
    if now - skew > i128::from(context.unsigned.expires_at) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextExpired,
            ["expiresAt"],
            "authorization context has expired",
        ));
    }
    verify_signature(
        &issuer_key,
        &authorization_context_signing_digest_v1(&context.unsigned)?,
        &context.signature,
        AuthorizationErrorCodeV1::InvalidSignature,
    )?;
    Ok(VerifiedAuthorizationContextV1 {
        session_key: decode_verifying_key(
            &context.unsigned.session_key,
            &["sessionKey"],
            AuthorizationErrorCodeV1::InvalidSessionKey,
        )?,
        context_digest: context.digest()?,
        context: context.clone(),
    })
}

/// Canonical context-bound request-proof input and digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationRequestProofInputV2 {
    bytes: Vec<u8>,
    digest: [u8; 32],
}

impl AuthorizationRequestProofInputV2 {
    /// Return the exact length-prefixed proof input bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Return the SHA-256 proof digest signed by the session key.
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

/// An unpadded base64url Ed25519 request proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationRequestProofV2(String);

impl AuthorizationRequestProofV2 {
    /// Parse and strictly validate an encoded request proof.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Authorization`] unless the value canonically
    /// encodes exactly one Ed25519 signature.
    pub fn parse(encoded: impl Into<String>) -> Result<Self, ProtocolError> {
        let encoded = encoded.into();
        decode_base64url::<64>(
            &encoded,
            &["proof"],
            AuthorizationErrorCodeV1::InvalidRequestProof,
        )?;
        Ok(Self(encoded))
    }

    /// Return the encoded proof.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Build request-proof v2 input from the exact received request values.
///
/// The payload hash is computed internally from `raw_payload`.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] if a component exceeds the unsigned
/// 32-bit length-prefix range.
pub fn build_authorization_request_proof_input_v2(
    context_digest: &[u8; 32],
    subject: &str,
    reply_subject: Option<&str>,
    raw_payload: &[u8],
    iat: i64,
    request_id: &str,
) -> Result<AuthorizationRequestProofInputV2, ProtocolError> {
    validate_safe_i64(iat, &["iat"])?;
    let payload_hash = sha256(raw_payload);
    let iat = iat.to_string();
    let mut bytes = Vec::new();
    for component in [
        AUTHORIZATION_REQUEST_PROOF_DOMAIN_V2.as_bytes(),
        context_digest,
        subject.as_bytes(),
        reply_subject.unwrap_or("").as_bytes(),
        payload_hash.as_slice(),
        iat.as_bytes(),
        request_id.as_bytes(),
    ] {
        push_length_prefixed(&mut bytes, component)?;
    }
    let digest = sha256(&bytes);
    Ok(AuthorizationRequestProofInputV2 { bytes, digest })
}

/// Sign a context-bound request with the session private key.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] for an invalid context digest or
/// oversized proof component.
pub fn sign_authorization_request_v2(
    context_digest: &str,
    subject: &str,
    reply_subject: Option<&str>,
    raw_payload: &[u8],
    iat: i64,
    request_id: &str,
    session_key: &SigningKey,
) -> Result<AuthorizationRequestProofV2, ProtocolError> {
    validate_safe_i64(iat, &["iat"])?;
    validate_text(request_id, &["request-id"])?;
    validate_text(subject, &["subject"])?;
    if request_id.len() > MAXIMUM_REQUEST_ID_BYTES {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["request-id"],
            "request id exceeds the protocol limit",
        ));
    }
    let context_digest = decode_base64url::<32>(
        context_digest,
        &["authorization-context"],
        AuthorizationErrorCodeV1::InvalidEncoding,
    )?;
    let input = build_authorization_request_proof_input_v2(
        &context_digest,
        subject,
        reply_subject,
        raw_payload,
        iat,
        request_id,
    )?;
    Ok(AuthorizationRequestProofV2(encode_base64url(
        &session_key.sign(input.digest()).to_bytes(),
    )))
}

/// Verified local caller metadata.
#[derive(Clone, Debug)]
pub struct VerifiedAuthorizationRequestV2 {
    context: VerifiedAuthorizationContextV1,
}

impl VerifiedAuthorizationRequestV2 {
    /// Return verified caller context metadata.
    pub fn context(&self) -> &VerifiedAuthorizationContextV1 {
        &self.context
    }
}

/// Verify request freshness, inbox binding, exact authority subsets, and session
/// key possession without storage access.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] for an invalid request id, reply
/// subject, issue time, permission/capability subset, or session-key signature.
#[expect(
    clippy::too_many_arguments,
    reason = "the language-neutral proof API takes each signed request component explicitly"
)]
pub fn verify_authorization_request_v2(
    context: &VerifiedAuthorizationContextV1,
    subject: &str,
    reply_subject: Option<&str>,
    raw_payload: &[u8],
    iat: i64,
    request_id: &str,
    proof: &AuthorizationRequestProofV2,
    policy: &AuthorizationVerificationPolicyV1,
    required_permissions: &[PermissionAtomV1],
    required_capabilities: &[String],
) -> Result<VerifiedAuthorizationRequestV2, ProtocolError> {
    validate_policy(policy)?;
    validate_safe_i64(iat, &["iat"])?;
    validate_text(request_id, &["request-id"])?;
    validate_text(subject, &["subject"])?;
    if request_id.len() > MAXIMUM_REQUEST_ID_BYTES {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["request-id"],
            "request id exceeds the protocol limit",
        ));
    }
    if let Some(reply) = reply_subject {
        let mut required_prefix = context.inbox_prefix().to_owned();
        required_prefix.push('.');
        if !reply.starts_with(&required_prefix) {
            return Err(authorization_error(
                AuthorizationErrorCodeV1::ReplySubjectMismatch,
                ["reply"],
                "reply subject is outside the verified caller inbox prefix",
            ));
        }
    }
    let difference = i128::from(policy.now_unix_seconds) - i128::from(iat);
    if difference.abs() > i128::from(policy.allowed_clock_skew_seconds) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ProofIatOutOfRange,
            ["iat"],
            "request proof issue time is outside policy skew",
        ));
    }
    let now = i128::from(policy.now_unix_seconds);
    let skew = i128::from(policy.allowed_clock_skew_seconds);
    if now + skew < i128::from(context.context.unsigned.not_before) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextNotYetValid,
            ["authorization-context", "notBefore"],
            "authorization context is not yet valid for this request",
        ));
    }
    if now - skew > i128::from(context.context.unsigned.expires_at) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextExpired,
            ["authorization-context", "expiresAt"],
            "authorization context has expired for this request",
        ));
    }
    if !context.allows_all(required_permissions) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::PermissionDenied,
            ["grantSet", "permissions"],
            "verified context does not contain every required exact permission",
        ));
    }
    if !context.has_all_capabilities(required_capabilities) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::CapabilityDenied,
            ["capabilities"],
            "verified context does not contain every required platform capability",
        ));
    }
    let context_digest = decode_base64url::<32>(
        context.context_digest(),
        &["authorization-context"],
        AuthorizationErrorCodeV1::InvalidEncoding,
    )?;
    let input = build_authorization_request_proof_input_v2(
        &context_digest,
        subject,
        reply_subject,
        raw_payload,
        iat,
        request_id,
    )?;
    let proof_bytes = decode_base64url::<64>(
        proof.as_str(),
        &["proof"],
        AuthorizationErrorCodeV1::InvalidRequestProof,
    )?;
    context
        .session_key
        .verify_strict(input.digest(), &Signature::from_bytes(&proof_bytes))
        .map_err(|_| {
            authorization_error(
                AuthorizationErrorCodeV1::InvalidRequestProof,
                ["proof"],
                "context-bound request signature verification failed",
            )
        })?;
    Ok(VerifiedAuthorizationRequestV2 {
        context: context.clone(),
    })
}

/// Canonical context-bound event-proof input and digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationEventProofInputV2 {
    bytes: Vec<u8>,
    digest: [u8; 32],
}

impl AuthorizationEventProofInputV2 {
    /// Return the exact length-prefixed proof input bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Return the SHA-256 digest signed by the session key.
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

/// An unpadded base64url Ed25519 event proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationEventProofV2(String);

impl AuthorizationEventProofV2 {
    /// Parse and strictly validate an encoded event proof.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Authorization`] unless the value canonically
    /// encodes exactly one Ed25519 signature.
    pub fn parse(encoded: impl Into<String>) -> Result<Self, ProtocolError> {
        let encoded = encoded.into();
        decode_base64url::<64>(
            &encoded,
            &["proof"],
            AuthorizationErrorCodeV1::InvalidEventProof,
        )?;
        Ok(Self(encoded))
    }

    /// Return the encoded proof.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Typed publisher projection for Event Log indexing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationEventPublisherV2 {
    /// Principal kind of the verified publisher.
    pub kind: String,
    /// Deployment identity for deployed principals.
    pub deployment_id: Option<String>,
    /// Runtime instance identity for deployed principals.
    pub instance_id: Option<String>,
    /// Participant id bound into the verified context.
    pub participant_id: String,
    /// Participant artifact digest bound into the verified context.
    pub participant_digest: String,
    /// Session id bound into the verified context.
    pub session_id: String,
}

/// Verified event metadata and publisher projection.
#[derive(Clone, Debug)]
pub struct VerifiedAuthorizationEventV2 {
    context: VerifiedAuthorizationContextV1,
    publisher: AuthorizationEventPublisherV2,
}

impl VerifiedAuthorizationEventV2 {
    /// Return verified publisher context metadata.
    pub fn context(&self) -> &VerifiedAuthorizationContextV1 {
        &self.context
    }

    /// Return the typed publisher projection for Event Log indexing.
    pub fn publisher(&self) -> &AuthorizationEventPublisherV2 {
        &self.publisher
    }
}

/// Parse a canonical RFC 3339 UTC event time into Unix seconds.
///
/// The accepted wire form is `YYYY-MM-DDTHH:MM:SS(.fraction)?Z` with an
/// uppercase `T` separator and `Z` UTC terminator.
fn canonical_event_time_seconds(event_time: &str, path: &[&str]) -> Result<i64, ProtocolError> {
    if !event_time.contains('T') || !event_time.ends_with('Z') {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidEventTime,
            path.iter().copied(),
            "event time is not canonical RFC 3339 UTC",
        ));
    }
    let parsed =
        time::OffsetDateTime::parse(event_time, &time::format_description::well_known::Rfc3339)
            .map_err(|_| {
                authorization_error(
                    AuthorizationErrorCodeV1::InvalidEventTime,
                    path.iter().copied(),
                    "event time is not canonical RFC 3339 UTC",
                )
            })?;
    if parsed.offset() != time::UtcOffset::UTC {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidEventTime,
            path.iter().copied(),
            "event time must be expressed in UTC",
        ));
    }
    Ok(parsed.unix_timestamp())
}

/// Build event-proof v2 input from the exact published event values.
///
/// The payload hash is computed internally from `raw_payload`, and the exact
/// `event_time` string is signed without reformatting.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] if a component exceeds the unsigned
/// 32-bit length-prefix range.
pub fn build_authorization_event_proof_input_v2(
    context_digest: &[u8; 32],
    subject: &str,
    raw_payload: &[u8],
    event_id: &str,
    event_time: &str,
) -> Result<AuthorizationEventProofInputV2, ProtocolError> {
    let payload_hash = sha256(raw_payload);
    let mut bytes = Vec::new();
    for component in [
        AUTHORIZATION_EVENT_PROOF_DOMAIN_V2.as_bytes(),
        context_digest,
        subject.as_bytes(),
        payload_hash.as_slice(),
        event_id.as_bytes(),
        event_time.as_bytes(),
    ] {
        push_length_prefixed(&mut bytes, component)?;
    }
    let digest = sha256(&bytes);
    Ok(AuthorizationEventProofInputV2 { bytes, digest })
}

/// Sign a context-bound event with the session private key.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] for an invalid context digest,
/// non-canonical event time, or oversized proof component.
pub fn sign_authorization_event_v2(
    context_digest: &str,
    subject: &str,
    raw_payload: &[u8],
    event_id: &str,
    event_time: &str,
    session_key: &SigningKey,
) -> Result<AuthorizationEventProofV2, ProtocolError> {
    validate_text(event_id, &["event-id"])?;
    validate_text(subject, &["subject"])?;
    if event_id.len() > MAXIMUM_EVENT_ID_BYTES {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["event-id"],
            "event id exceeds the protocol limit",
        ));
    }
    canonical_event_time_seconds(event_time, &["event-time"])?;
    let context_digest = decode_base64url::<32>(
        context_digest,
        &["authorization-context"],
        AuthorizationErrorCodeV1::InvalidEncoding,
    )?;
    let input = build_authorization_event_proof_input_v2(
        &context_digest,
        subject,
        raw_payload,
        event_id,
        event_time,
    )?;
    Ok(AuthorizationEventProofV2(encode_base64url(
        &session_key.sign(input.digest()).to_bytes(),
    )))
}

/// Verify event freshness, context binding, exact authority subsets, and
/// session-key possession without storage access.
///
/// The signed event time is evaluated against the verified context window.
/// Any revocation evidence invalidates every event proof from that context so
/// an old signed event cannot be replayed after authority changes.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] for a non-canonical event time,
/// an event outside the context window, a revoked event, missing
/// permission/capability evidence, or an invalid session-key signature.
#[expect(
    clippy::too_many_arguments,
    reason = "the language-neutral proof API takes each signed event component explicitly"
)]
pub fn verify_authorization_event_v2(
    context: &VerifiedAuthorizationContextV1,
    subject: &str,
    raw_payload: &[u8],
    event_id: &str,
    event_time: &str,
    proof: &AuthorizationEventProofV2,
    policy: &AuthorizationVerificationPolicyV1,
    required_permissions: &[PermissionAtomV1],
    required_capabilities: &[String],
    revoked_at: Option<i64>,
) -> Result<VerifiedAuthorizationEventV2, ProtocolError> {
    validate_policy(policy)?;
    validate_text(event_id, &["event-id"])?;
    validate_text(subject, &["subject"])?;
    if event_id.len() > MAXIMUM_EVENT_ID_BYTES {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["event-id"],
            "event id exceeds the protocol limit",
        ));
    }
    if let Some(revoked_at) = revoked_at {
        validate_safe_i64(revoked_at, &["revoked-at"])?;
    }
    let event_time_seconds = canonical_event_time_seconds(event_time, &["event-time"])?;
    let event_time_unix = i128::from(event_time_seconds);
    // Historical eligibility is the strict signed-context window: no clock skew
    // is applied to notBefore/expiresAt and `expiresAt` itself is exclusive.
    if event_time_unix < i128::from(context.context.unsigned.not_before) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextNotYetValid,
            ["event-time"],
            "event is before the signed context notBefore window",
        ));
    }
    if event_time_unix >= i128::from(context.context.unsigned.expires_at) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextExpired,
            ["event-time"],
            "event is at or after the signed context expiresAt window",
        ));
    }
    if revoked_at.is_some() {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::EventRevoked,
            ["event-time"],
            "event authorization context is revoked",
        ));
    }
    if !context.allows_all(required_permissions) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::PermissionDenied,
            ["grantSet", "permissions"],
            "verified context does not contain every required exact permission",
        ));
    }
    if !context.has_all_capabilities(required_capabilities) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::CapabilityDenied,
            ["capabilities"],
            "verified context does not contain every required platform capability",
        ));
    }
    let context_digest = decode_base64url::<32>(
        context.context_digest(),
        &["authorization-context"],
        AuthorizationErrorCodeV1::InvalidEncoding,
    )?;
    let input = build_authorization_event_proof_input_v2(
        &context_digest,
        subject,
        raw_payload,
        event_id,
        event_time,
    )?;
    let proof_bytes = decode_base64url::<64>(
        proof.as_str(),
        &["proof"],
        AuthorizationErrorCodeV1::InvalidEventProof,
    )?;
    context
        .session_key
        .verify_strict(input.digest(), &Signature::from_bytes(&proof_bytes))
        .map_err(|_| {
            authorization_error(
                AuthorizationErrorCodeV1::InvalidEventProof,
                ["proof"],
                "context-bound event signature verification failed",
            )
        })?;
    let publisher = AuthorizationEventPublisherV2 {
        kind: match context.principal().kind {
            AuthorizationPrincipalKindV1::User => "user",
            AuthorizationPrincipalKindV1::Service => "service",
            AuthorizationPrincipalKindV1::Device => "device",
        }
        .to_owned(),
        deployment_id: context.deployment_id().map(str::to_owned),
        instance_id: context.instance_id().map(str::to_owned),
        participant_id: context.participant().id.clone(),
        participant_digest: context.participant().artifact_digest.clone(),
        session_id: context.session_id().to_owned(),
    };
    Ok(VerifiedAuthorizationEventV2 {
        context: context.clone(),
        publisher,
    })
}

#[cfg(test)]
mod phase_a_tests {
    use super::*;
    use crate::{ApiSurfaceKindV1, PermissionActionV1, PermissionTargetV1};
    use serde_json::json;

    const ROOT_SEED: [u8; 32] = [1; 32];
    const ISSUER_SEED: [u8; 32] = [2; 32];
    const SESSION_SEED: [u8; 32] = [3; 32];

    fn key(seed: [u8; 32]) -> SigningKey {
        SigningKey::from_bytes(&seed)
    }

    fn encoded_key(key: &SigningKey) -> String {
        encode_base64url(key.verifying_key().as_bytes())
    }

    fn permission() -> PermissionAtomV1 {
        PermissionAtomV1::new(
            PermissionTargetV1::api_surface("documents@v1", ApiSurfaceKindV1::Rpc, "Documents.Get")
                .unwrap(),
            PermissionActionV1::Call,
        )
        .unwrap()
    }

    fn policy(now: i64) -> AuthorizationVerificationPolicyV1 {
        AuthorizationVerificationPolicyV1::new(now, 30, 300, 16_384, 16, 16, 7).unwrap()
    }

    fn chain() -> (
        AuthorizationTrustRootV1,
        VerifiedAuthorizationIssuerManifestV1,
        SignedAuthorizationContextV1,
        SigningKey,
    ) {
        let root_key = key(ROOT_SEED);
        let issuer_key = key(ISSUER_SEED);
        let session_key = key(SESSION_SEED);
        let root = AuthorizationTrustRootV1::new("trellis-test", encoded_key(&root_key)).unwrap();
        let manifest = sign_issuer_manifest_v1(
            UnsignedAuthorizationIssuerManifestV1 {
                format: AUTHORIZATION_ISSUER_MANIFEST_FORMAT_V1.to_owned(),
                authority: root.authority().to_owned(),
                root_key_id: root.key_id().to_owned(),
                generation: 7,
                issued_at: 1_000,
                not_before: 1_000,
                expires_at: 1_500,
                issuers: vec![AuthorizationIssuerManifestEntryV1 {
                    key_id: derived_key_id(&issuer_key.verifying_key()),
                    public_key: encoded_key(&issuer_key),
                }],
                extensions: Map::new(),
                critical: vec![],
            },
            &root_key,
        )
        .unwrap();
        let manifest = verify_issuer_manifest_v1(&root, &manifest, &policy(1_100)).unwrap();
        let context = sign_authorization_context_v1(
            UnsignedAuthorizationContextV1 {
                format: AUTHORIZATION_CONTEXT_FORMAT_V1.to_owned(),
                authority: root.authority().to_owned(),
                issuer_key_id: derived_key_id(&issuer_key.verifying_key()),
                issuer_manifest_generation: 7,
                session_id: "ses_test".to_owned(),
                session_key: encoded_key(&session_key),
                principal: AuthorizationPrincipalV1 {
                    kind: AuthorizationPrincipalKindV1::User,
                    id: "usr_test".to_owned(),
                },
                participant: AuthorizationParticipantV1 {
                    kind: ParticipantKindV1::App,
                    id: "documents-web".to_owned(),
                    artifact_digest: encode_base64url(&[4; 32]),
                    needs_digest: encode_base64url(&[5; 32]),
                },
                authority_ref: AuthorizationAuthorityRefV1 {
                    kind: AuthorizationAuthorityKindV1::Identity,
                    id: "usr_test".to_owned(),
                    version: 12,
                },
                deployment_id: None,
                instance_id: None,
                inbox_prefix: "_INBOX.test".to_owned(),
                issued_at: 1_100,
                not_before: 1_100,
                expires_at: 1_300,
                grant_set: GrantSetV1::new(vec![permission()]),
                capabilities: vec!["platform.read".to_owned()],
                extensions: Map::new(),
                critical: vec![],
            },
            &issuer_key,
        )
        .unwrap();
        (root, manifest, context, session_key)
    }

    #[test]
    fn direct_key_manifest_verifies_context_and_request_event_proofs() {
        let (root, manifest, context, session_key) = chain();
        let policy = policy(1_100);
        let verified =
            verify_authorization_context_v1(&root, &manifest, &context, &policy).unwrap();
        let request_proof = sign_authorization_request_v2(
            verified.context_digest(),
            "rpc.v1.Documents.Get",
            Some("_INBOX.test.reply"),
            br#"{"id":"doc-1"}"#,
            1_100,
            "req_test",
            &session_key,
        )
        .unwrap();
        let event_proof = sign_authorization_event_v2(
            verified.context_digest(),
            "events.v1.Documents.Changed.doc-1",
            br#"{"id":"doc-1"}"#,
            "evt_doc_1",
            "1970-01-01T00:19:10Z",
            &session_key,
        )
        .unwrap();
        assert_eq!(verified.context_digest(), &context.digest().unwrap());

        verify_authorization_request_v2(
            &verified,
            "rpc.v1.Documents.Get",
            Some("_INBOX.test.reply"),
            br#"{"id":"doc-1"}"#,
            1_100,
            "req_test",
            &request_proof,
            &policy,
            &[permission()],
            &["platform.read".to_owned()],
        )
        .unwrap();

        verify_authorization_event_v2(
            &verified,
            "events.v1.Documents.Changed.doc-1",
            br#"{"id":"doc-1"}"#,
            "evt_doc_1",
            "1970-01-01T00:19:10Z",
            &event_proof,
            &AuthorizationVerificationPolicyV1::new(1_400, 30, 300, 16_384, 16, 16, 7).unwrap(),
            &[permission()],
            &[],
            None,
        )
        .unwrap();
    }

    #[test]
    fn manifest_and_context_bindings_fail_closed() {
        let (root, manifest, context, _) = chain();
        let mut wrong_generation = context.clone();
        wrong_generation.unsigned.issuer_manifest_generation = 8;
        assert!(matches!(
            verify_authorization_context_v1(&root, &manifest, &wrong_generation, &policy(1_100)),
            Err(ProtocolError::Authorization {
                code: AuthorizationErrorCodeV1::ManifestGenerationMismatch,
                ..
            })
        ));

        let mut wrong_key = context.clone();
        wrong_key.unsigned.issuer_key_id = root.key_id().to_owned();
        assert!(matches!(
            verify_authorization_context_v1(&root, &manifest, &wrong_key, &policy(1_100)),
            Err(ProtocolError::Authorization {
                code: AuthorizationErrorCodeV1::IssuerNotListed,
                ..
            })
        ));

        let mut manifest_value = serde_json::to_value(manifest.manifest()).unwrap();
        manifest_value["issuers"][0]["keyId"] = json!(root.key_id());
        assert!(matches!(
            parse_issuer_manifest_v1(&manifest_value),
            Err(ProtocolError::Authorization {
                code: AuthorizationErrorCodeV1::InvalidKeyId,
                ..
            })
        ));
    }

    #[test]
    fn context_digest_is_the_only_context_identity() {
        let (_, _, context, _) = chain();
        let value = serde_json::to_value(&context).unwrap();
        assert!(value.get("contextId").is_none());
        assert!(value.get("issuerManifestGeneration").is_some());
        assert_eq!(context.digest().unwrap().len(), 43);
    }
}
