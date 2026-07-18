//! Pure signed authorization-context and request-proof protocol.
//!
//! Trellis pins an [`AuthorizationTrustRootV1`]. That root signs issuer
//! certificates and a generation-numbered issuer manifest. An active issuer
//! signs short-lived authorization contexts, and the session key bound into a
//! verified context signs each exact request. Verification requires no network,
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
//! The following constructs and verifies the root, issuer certificate, current
//! manifest, context, exact permission, and context-bound request proof:
//!
//! ```
//! use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
//! use ed25519_dalek::SigningKey;
//! use serde_json::Map;
//! use trellis_protocol::{
//!     sign_authorization_context_v1, sign_authorization_request_v2,
//!     sign_issuer_certificate_v1, sign_issuer_manifest_v1,
//!     verify_authorization_context_v1, verify_authorization_request_v2,
//!     verify_issuer_manifest_v1, ApiSurfaceKindV1, AuthorizationAuthorityKindV1,
//!     AuthorizationAuthorityRefV1, AuthorizationIssuerManifestEntryV1,
//!     AuthorizationIssuerStatusV1, AuthorizationParticipantV1,
//!     AuthorizationPrincipalKindV1, AuthorizationPrincipalV1,
//!     AuthorizationTrustRootV1, AuthorizationVerificationPolicyV1, GrantSetV1,
//!     ParticipantKindV1, PermissionActionV1, PermissionAtomV1,
//!     PermissionTargetV1, UnsignedAuthorizationContextV1,
//!     UnsignedAuthorizationIssuerCertificateV1,
//!     UnsignedAuthorizationIssuerManifestV1, AUTHORIZATION_CONTEXT_FORMAT_V1,
//!     AUTHORIZATION_ISSUER_CERTIFICATE_FORMAT_V1,
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
//! let certificate = sign_issuer_certificate_v1(
//!     UnsignedAuthorizationIssuerCertificateV1 {
//!         format: AUTHORIZATION_ISSUER_CERTIFICATE_FORMAT_V1.into(),
//!         authority: root.authority().into(),
//!         root_key_id: root.key_id().into(),
//!         serial: "isc_example".into(),
//!         key_id: key_id(&issuer_key),
//!         public_key: encode_key(&issuer_key),
//!         issued_at: 1_000,
//!         not_before: 1_000,
//!         expires_at: 2_000,
//!         usages: vec!["authorizationContext".into()],
//!         extensions: Map::new(),
//!         critical: vec![],
//!     },
//!     &root_key,
//! )?;
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
//!             key_id: certificate.unsigned.key_id.clone(),
//!             certificate_digest: certificate.digest()?,
//!             status: AuthorizationIssuerStatusV1::Active,
//!             revoked_at: None,
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
//!         context_id: "ctx_example".into(),
//!         issuer_key_id: certificate.unsigned.key_id.clone(),
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
//! let context = verify_authorization_context_v1(
//!     &root, &manifest, &certificate, &context, &policy,
//! )?;
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
//!     &[permission],
//!     &[],
//! )?;
//! assert_eq!(request.context().principal().id, "usr_example");
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
/// Root-signed issuer-certificate wire format and signature domain.
pub const AUTHORIZATION_ISSUER_CERTIFICATE_FORMAT_V1: &str =
    "trellis.authorization-issuer-certificate.v1";
/// Root-signed issuer-manifest wire format and signature domain.
pub const AUTHORIZATION_ISSUER_MANIFEST_FORMAT_V1: &str =
    "trellis.authorization-issuer-manifest.v1";
/// Issuer-signed authorization-context wire format and signature domain.
pub const AUTHORIZATION_CONTEXT_FORMAT_V1: &str = "trellis.authorization-context.v1";
/// Context-bound request-proof signature domain.
pub const AUTHORIZATION_REQUEST_PROOF_DOMAIN_V2: &str = "trellis.authorization-request-proof.v2";

const AUTHORIZATION_CONTEXT_USAGE: &str = "authorizationContext";
const MAXIMUM_REQUEST_ID_BYTES: usize = 256;
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
    /// Maximum decoded canonical context bytes.
    pub maximum_context_token_bytes: usize,
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
        maximum_context_token_bytes: usize,
        maximum_permissions: usize,
        maximum_capabilities: usize,
        minimum_manifest_generation: u64,
    ) -> Result<Self, ProtocolError> {
        let policy = Self {
            now_unix_seconds,
            allowed_clock_skew_seconds,
            maximum_context_lifetime_seconds,
            maximum_context_token_bytes,
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
        || policy.maximum_context_token_bytes == 0
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

/// Context-verification policy name retained for API clarity.
pub type AuthorizationContextVerificationPolicyV1 = AuthorizationVerificationPolicyV1;

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

/// Unsigned root-authorized issuer certificate fields.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UnsignedAuthorizationIssuerCertificateV1 {
    /// Wire format; must equal [`AUTHORIZATION_ISSUER_CERTIFICATE_FORMAT_V1`].
    pub format: String,
    /// Authority namespace.
    pub authority: String,
    /// Content-derived root key id.
    pub root_key_id: String,
    /// Nonempty certificate serial.
    pub serial: String,
    /// Content-derived issuer key id.
    pub key_id: String,
    /// Unpadded base64url Ed25519 issuer public key.
    pub public_key: String,
    /// Certificate issue time.
    pub issued_at: i64,
    /// Inclusive lower validity bound.
    pub not_before: i64,
    /// Inclusive upper validity bound.
    pub expires_at: i64,
    /// Canonical certificate usage set.
    pub usages: Vec<String>,
    /// Signed noncritical or critical extension values.
    pub extensions: Map<String, Value>,
    /// Canonical names of security-critical extensions.
    pub critical: Vec<String>,
}

/// Root-signed issuer certificate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SignedAuthorizationIssuerCertificateV1 {
    /// Unsigned certificate fields.
    #[serde(flatten)]
    pub unsigned: UnsignedAuthorizationIssuerCertificateV1,
    /// Root Ed25519 signature.
    pub signature: String,
}

impl SignedAuthorizationIssuerCertificateV1 {
    /// Return the digest of the complete signed canonical certificate.
    ///
    /// # Errors
    ///
    /// Returns a JSON canonicalization error if serialization fails.
    pub fn digest(&self) -> Result<String, ProtocolError> {
        complete_digest(self)
    }
}

/// Return the domain-separated signing digest for an issuer certificate.
///
/// # Errors
///
/// Returns a JSON canonicalization error if serialization fails.
pub fn issuer_certificate_signing_digest_v1(
    certificate: &UnsignedAuthorizationIssuerCertificateV1,
) -> Result<[u8; 32], ProtocolError> {
    validate_certificate_fields(certificate)?;
    signed_json_digest(
        AUTHORIZATION_ISSUER_CERTIFICATE_FORMAT_V1,
        &serde_json::to_value(certificate)?,
    )
}

/// Sign an issuer certificate with the pinned root key.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] when certificate structure is
/// invalid, or a JSON canonicalization error.
pub fn sign_issuer_certificate_v1(
    certificate: UnsignedAuthorizationIssuerCertificateV1,
    root_key: &SigningKey,
) -> Result<SignedAuthorizationIssuerCertificateV1, ProtocolError> {
    validate_certificate_fields(&certificate)?;
    check_key_id(
        &certificate.root_key_id,
        &root_key.verifying_key(),
        &["rootKeyId"],
    )?;
    let digest = issuer_certificate_signing_digest_v1(&certificate)?;
    Ok(SignedAuthorizationIssuerCertificateV1 {
        unsigned: certificate,
        signature: encode_base64url(&root_key.sign(&digest).to_bytes()),
    })
}

/// Strictly parse a signed issuer certificate.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] for an invalid strict object shape
/// or intrinsic certificate invariant.
pub fn parse_issuer_certificate_v1(
    value: &Value,
) -> Result<SignedAuthorizationIssuerCertificateV1, ProtocolError> {
    let certificate: SignedAuthorizationIssuerCertificateV1 = serde_json::from_value(value.clone())
        .map_err(|_| {
            authorization_error(
                AuthorizationErrorCodeV1::InvalidFormat,
                std::iter::empty::<&str>(),
                "issuer certificate has an invalid strict object shape",
            )
        })?;
    validate_certificate_fields(&certificate.unsigned)?;
    decode_base64url::<64>(
        &certificate.signature,
        &["signature"],
        AuthorizationErrorCodeV1::InvalidSignature,
    )?;
    Ok(certificate)
}

fn validate_certificate_fields(
    certificate: &UnsignedAuthorizationIssuerCertificateV1,
) -> Result<(), ProtocolError> {
    if certificate.format != AUTHORIZATION_ISSUER_CERTIFICATE_FORMAT_V1 {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            ["format"],
            "unsupported issuer-certificate format",
        ));
    }
    validate_text(&certificate.authority, &["authority"])?;
    validate_text(&certificate.serial, &["serial"])?;
    decode_base64url::<32>(
        &certificate.root_key_id,
        &["rootKeyId"],
        AuthorizationErrorCodeV1::InvalidKeyId,
    )?;
    validate_window(
        certificate.issued_at,
        certificate.not_before,
        certificate.expires_at,
        &[],
    )?;
    for (index, usage) in certificate.usages.iter().enumerate() {
        validate_text(usage, &["usages", &index.to_string()])?;
    }
    if !is_utf16_strictly_sorted(&certificate.usages) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::NonCanonicalSet,
            ["usages"],
            "certificate usages must be UTF-16 sorted and unique",
        ));
    }
    if !certificate
        .usages
        .iter()
        .any(|usage| usage == AUTHORIZATION_CONTEXT_USAGE)
    {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::CertificateUsageMissing,
            ["usages"],
            "certificate does not authorize authorization contexts",
        ));
    }
    let key = decode_verifying_key(
        &certificate.public_key,
        &["publicKey"],
        AuthorizationErrorCodeV1::InvalidPublicKey,
    )?;
    check_key_id(&certificate.key_id, &key, &["keyId"])?;
    validate_extensions(&certificate.extensions, &certificate.critical)
}

/// Verify a root-signed issuer certificate at the supplied policy time.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] if authority, key ids, validity,
/// usage, extensions, or the strict root signature fail verification.
pub fn verify_issuer_certificate_v1(
    root: &AuthorizationTrustRootV1,
    certificate: &SignedAuthorizationIssuerCertificateV1,
    policy: &AuthorizationVerificationPolicyV1,
) -> Result<(), ProtocolError> {
    validate_policy(policy)?;
    validate_certificate_fields(&certificate.unsigned)?;
    if certificate.unsigned.authority != root.authority
        || certificate.unsigned.root_key_id != root.key_id
    {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::WrongAuthority,
            ["authority"],
            "certificate authority or root key id does not match the pinned root",
        ));
    }
    let now = i128::from(policy.now_unix_seconds);
    let skew = i128::from(policy.allowed_clock_skew_seconds);
    if now + skew < i128::from(certificate.unsigned.not_before) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::CertificateNotYetValid,
            ["notBefore"],
            "issuer certificate is not yet valid",
        ));
    }
    if now - skew > i128::from(certificate.unsigned.expires_at) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::CertificateExpired,
            ["expiresAt"],
            "issuer certificate has expired",
        ));
    }
    verify_signature(
        &root.verifying_key()?,
        &issuer_certificate_signing_digest_v1(&certificate.unsigned)?,
        &certificate.signature,
        AuthorizationErrorCodeV1::InvalidSignature,
    )
}

/// Issuer state in a root-signed manifest.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthorizationIssuerStatusV1 {
    /// The issuer may validate current contexts.
    Active,
    /// The issuer is explicitly untrusted.
    Revoked,
}

/// One issuer certificate selected by a manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorizationIssuerManifestEntryV1 {
    /// Content-derived issuer key id.
    pub key_id: String,
    /// Digest of the exact complete signed certificate.
    pub certificate_digest: String,
    /// Current issuer state.
    pub status: AuthorizationIssuerStatusV1,
    /// Revocation time, present only for revoked issuers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<i64>,
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

    /// Return the exact certificate digest for an active issuer.
    pub fn active_certificate_digest(&self, key_id: &str) -> Option<&str> {
        self.manifest
            .unsigned
            .issuers
            .iter()
            .find(|entry| {
                entry.key_id == key_id && entry.status == AuthorizationIssuerStatusV1::Active
            })
            .map(|entry| entry.certificate_digest.as_str())
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
        decode_base64url::<32>(
            &entry.key_id,
            &["issuers", &index.to_string(), "keyId"],
            AuthorizationErrorCodeV1::InvalidKeyId,
        )?;
        decode_base64url::<32>(
            &entry.certificate_digest,
            &["issuers", &index.to_string(), "certificateDigest"],
            AuthorizationErrorCodeV1::InvalidEncoding,
        )?;
        match (entry.status, entry.revoked_at) {
            (AuthorizationIssuerStatusV1::Active, None)
            | (AuthorizationIssuerStatusV1::Revoked, Some(_)) => {}
            _ => {
                return Err(authorization_error(
                    AuthorizationErrorCodeV1::InvalidFormat,
                    ["issuers", &index.to_string(), "revokedAt"],
                    "active issuers omit revokedAt and revoked issuers require it",
                ));
            }
        }
        if let Some(revoked_at) = entry.revoked_at {
            validate_safe_i64(revoked_at, &["issuers", &index.to_string(), "revokedAt"])?;
        }
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
    /// Unique context lease id.
    pub context_id: String,
    /// Content-derived signing issuer key id.
    pub issuer_key_id: String,
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
    validate_text(&context.context_id, &["contextId"])?;
    validate_text(&context.session_id, &["sessionId"])?;
    validate_text(&context.principal.id, &["principal", "id"])?;
    validate_text(&context.participant.id, &["participant", "id"])?;
    validate_text(&context.authority_ref.id, &["authorityRef", "id"])?;
    decode_base64url::<32>(
        &context.issuer_key_id,
        &["issuerKeyId"],
        AuthorizationErrorCodeV1::InvalidKeyId,
    )?;
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
    decode_verifying_key(
        &context.session_key,
        &["sessionKey"],
        AuthorizationErrorCodeV1::InvalidSessionKey,
    )?;
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

/// Encode complete signed canonical context JSON as an unpadded base64url token.
///
/// # Errors
///
/// Returns a JSON canonicalization error if serialization fails.
pub fn encode_authorization_context_token_v1(
    context: &SignedAuthorizationContextV1,
) -> Result<String, ProtocolError> {
    let canonical = canonicalize_json(&serde_json::to_value(context)?)?;
    Ok(encode_base64url(canonical.as_bytes()))
}

/// Decode and parse a context token under an explicit byte limit.
///
/// Size is checked before JSON parsing.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] for noncanonical base64url, excess
/// size, malformed JSON, or an invalid signed context shape.
pub fn parse_authorization_context_token_v1(
    token: &str,
    policy: &AuthorizationVerificationPolicyV1,
) -> Result<SignedAuthorizationContextV1, ProtocolError> {
    validate_policy(policy)?;
    if token.contains('=') {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidEncoding,
            std::iter::empty::<&str>(),
            "padded context tokens are not accepted",
        ));
    }
    let maximum_encoded_bytes = policy
        .maximum_context_token_bytes
        .checked_add(2)
        .and_then(|length| length.checked_div(3))
        .and_then(|groups| groups.checked_mul(4))
        .ok_or_else(|| {
            authorization_error(
                AuthorizationErrorCodeV1::InvalidFormat,
                std::iter::empty::<&str>(),
                "context token size policy overflows base64 length calculation",
            )
        })?;
    if token.len() > maximum_encoded_bytes {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextTokenTooLarge,
            std::iter::empty::<&str>(),
            "encoded context exceeds policy size",
        ));
    }
    let bytes = URL_SAFE_NO_PAD.decode(token).map_err(|_| {
        authorization_error(
            AuthorizationErrorCodeV1::InvalidEncoding,
            std::iter::empty::<&str>(),
            "context token is not unpadded base64url",
        )
    })?;
    if bytes.len() > policy.maximum_context_token_bytes {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextTokenTooLarge,
            std::iter::empty::<&str>(),
            "decoded context exceeds policy size",
        ));
    }
    if encode_base64url(&bytes) != token {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidEncoding,
            std::iter::empty::<&str>(),
            "context token has a noncanonical base64url encoding",
        ));
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| {
        authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            std::iter::empty::<&str>(),
            "context token does not contain valid JSON",
        )
    })?;
    let context = parse_authorization_context_v1(&value)?;
    let canonical = canonicalize_json(&serde_json::to_value(&context)?)?;
    if canonical.as_bytes() != bytes {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::InvalidFormat,
            std::iter::empty::<&str>(),
            "context token JSON is not in canonical RFC 8785 form",
        ));
    }
    Ok(context)
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

    /// Return the context id used in replay keys.
    pub fn context_id(&self) -> &str {
        &self.context.unsigned.context_id
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

    /// Return the context lease expiry used by replay caches.
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

/// Verify the complete root, manifest, certificate, and context trust chain.
///
/// # Errors
///
/// Returns [`ProtocolError::Authorization`] if the issuer is missing or revoked,
/// certificate digest or authority differs, a validity bound is exceeded,
/// collection limits fail, or the issuer signature is invalid.
pub fn verify_authorization_context_v1(
    root: &AuthorizationTrustRootV1,
    verified_manifest: &VerifiedAuthorizationIssuerManifestV1,
    issuer_certificate: &SignedAuthorizationIssuerCertificateV1,
    context: &SignedAuthorizationContextV1,
    policy: &AuthorizationContextVerificationPolicyV1,
) -> Result<VerifiedAuthorizationContextV1, ProtocolError> {
    validate_policy(policy)?;
    validate_context_fields(&context.unsigned, None)?;
    let context_size = canonicalize_json(&serde_json::to_value(context)?)?.len();
    if context_size > policy.maximum_context_token_bytes {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextTokenTooLarge,
            std::iter::empty::<&str>(),
            "canonical signed context exceeds policy size",
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
    if verified_manifest.generation() < policy.minimum_manifest_generation {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ManifestRollback,
            ["generation"],
            "verified manifest generation is below the current accepted minimum",
        ));
    }
    verify_issuer_certificate_v1(root, issuer_certificate, policy)?;
    if context.unsigned.authority != root.authority {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::WrongAuthority,
            ["authority"],
            "context authority does not match the pinned root",
        ));
    }
    if context.unsigned.issuer_key_id != issuer_certificate.unsigned.key_id {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::IssuerNotListed,
            ["issuerKeyId"],
            "context issuer does not match the supplied certificate",
        ));
    }
    let entry = verified_manifest
        .manifest
        .unsigned
        .issuers
        .iter()
        .find(|entry| entry.key_id == context.unsigned.issuer_key_id)
        .ok_or_else(|| {
            authorization_error(
                AuthorizationErrorCodeV1::IssuerNotListed,
                ["issuerKeyId"],
                "context issuer is absent from the current manifest",
            )
        })?;
    if entry.status == AuthorizationIssuerStatusV1::Revoked {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::IssuerRevoked,
            ["issuerKeyId"],
            "context issuer is revoked",
        ));
    }
    if entry.certificate_digest != issuer_certificate.digest()? {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::CertificateDigestMismatch,
            ["issuerKeyId"],
            "manifest certificate digest does not match the supplied certificate",
        ));
    }
    let lifetime =
        i128::from(context.unsigned.expires_at) - i128::from(context.unsigned.not_before);
    if lifetime > i128::from(policy.maximum_context_lifetime_seconds) {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextLifetimeExceeded,
            ["expiresAt"],
            "context lifetime exceeds policy",
        ));
    }
    if context.unsigned.not_before < issuer_certificate.unsigned.not_before
        || context.unsigned.expires_at > issuer_certificate.unsigned.expires_at
    {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextOutlivesCertificate,
            ["expiresAt"],
            "context validity is not contained by its issuer certificate",
        ));
    }
    let manifest = &verified_manifest.manifest.unsigned;
    if context.unsigned.not_before < manifest.not_before
        || context.unsigned.expires_at > manifest.expires_at
    {
        return Err(authorization_error(
            AuthorizationErrorCodeV1::ContextOutlivesManifest,
            ["expiresAt"],
            "context validity is not contained by the current manifest",
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
    let issuer_key = decode_verifying_key(
        &issuer_certificate.unsigned.public_key,
        &["publicKey"],
        AuthorizationErrorCodeV1::InvalidPublicKey,
    )?;
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

/// Replay-cache identity returned after pure request verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationReplayKeyV2 {
    /// Verified authorization-context id.
    pub context_id: String,
    /// Caller-authored request id.
    pub request_id: String,
}

/// Verified local caller metadata and replay-cache material.
#[derive(Clone, Debug)]
pub struct VerifiedAuthorizationRequestV2 {
    context: VerifiedAuthorizationContextV1,
    replay_key: AuthorizationReplayKeyV2,
    replay_expires_at: i64,
}

impl VerifiedAuthorizationRequestV2 {
    /// Return verified caller context metadata.
    pub fn context(&self) -> &VerifiedAuthorizationContextV1 {
        &self.context
    }

    /// Return the replay-cache identity `(contextId, requestId)`.
    pub fn replay_key(&self) -> &AuthorizationReplayKeyV2 {
        &self.replay_key
    }

    /// Return the context expiry suitable for the replay-cache entry.
    pub fn replay_expires_at(&self) -> i64 {
        self.replay_expires_at
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
    let replay_expires_at = context
        .expires_at()
        .checked_add(i64::from(policy.allowed_clock_skew_seconds))
        .ok_or_else(|| {
            authorization_error(
                AuthorizationErrorCodeV1::InvalidValidityWindow,
                ["authorization-context", "expiresAt"],
                "context replay expiry overflows Unix seconds",
            )
        })?;
    Ok(VerifiedAuthorizationRequestV2 {
        context: context.clone(),
        replay_key: AuthorizationReplayKeyV2 {
            context_id: context.context_id().to_owned(),
            request_id: request_id.to_owned(),
        },
        replay_expires_at,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{ApiSurfaceKindV1, PermissionActionV1, PermissionTargetV1};

    fn fixture_policy(now: i64) -> AuthorizationVerificationPolicyV1 {
        AuthorizationVerificationPolicyV1::new(now, 30, 300, 16_384, 16, 16, 7).unwrap()
    }

    fn permission() -> PermissionAtomV1 {
        PermissionAtomV1::new(
            PermissionTargetV1::api_surface("documents@v1", ApiSurfaceKindV1::Rpc, "Documents.Get")
                .unwrap(),
            PermissionActionV1::Call,
        )
        .unwrap()
    }

    fn assert_authorization_error(
        error: ProtocolError,
        expected_code: AuthorizationErrorCodeV1,
        expected_path: &str,
    ) {
        match error {
            ProtocolError::Authorization { code, path, .. } => {
                assert_eq!(code, expected_code);
                assert_eq!(path.as_str(), expected_path);
            }
            error => panic!("unexpected error: {error}"),
        }
    }

    #[derive(Clone)]
    struct FixtureArtifacts {
        root: Value,
        certificate: Value,
        manifest: Value,
        context: Value,
    }

    fn fixture_signing_key(encoded: &str) -> SigningKey {
        SigningKey::from_bytes(
            &decode_base64url::<32>(encoded, &[], AuthorizationErrorCodeV1::InvalidEncoding)
                .unwrap(),
        )
    }

    fn fixture_policy_from(
        defaults: &Value,
        case: &Value,
    ) -> Result<AuthorizationVerificationPolicyV1, ProtocolError> {
        let base = &defaults["policy"];
        let mut policy = AuthorizationVerificationPolicyV1::new(
            base["nowUnixSeconds"].as_i64().unwrap(),
            u32::try_from(base["allowedClockSkewSeconds"].as_u64().unwrap()).unwrap(),
            u32::try_from(base["maximumContextLifetimeSeconds"].as_u64().unwrap()).unwrap(),
            usize::try_from(base["maximumContextTokenBytes"].as_u64().unwrap()).unwrap(),
            usize::try_from(base["maximumPermissions"].as_u64().unwrap()).unwrap(),
            usize::try_from(base["maximumCapabilities"].as_u64().unwrap()).unwrap(),
            base["minimumManifestGeneration"].as_u64().unwrap(),
        )?;
        if let Some(overrides) = case["inputs"]["policy"].as_object() {
            if case["inputs"]["mutatePolicyAfterConstruction"].as_bool() == Some(true) {
                if let Some(value) = overrides.get("nowUnixSeconds") {
                    policy.now_unix_seconds = value.as_i64().unwrap();
                }
                if let Some(value) = overrides.get("minimumManifestGeneration") {
                    policy.minimum_manifest_generation = value.as_u64().unwrap();
                }
                return Ok(policy);
            }
        }
        let mut policy = base.clone();
        if let Some(overrides) = case["inputs"]["policy"].as_object() {
            policy.as_object_mut().unwrap().extend(overrides.clone());
        }
        AuthorizationVerificationPolicyV1::new(
            policy["nowUnixSeconds"].as_i64().unwrap(),
            u32::try_from(policy["allowedClockSkewSeconds"].as_u64().unwrap()).unwrap(),
            u32::try_from(policy["maximumContextLifetimeSeconds"].as_u64().unwrap()).unwrap(),
            usize::try_from(policy["maximumContextTokenBytes"].as_u64().unwrap()).unwrap(),
            usize::try_from(policy["maximumPermissions"].as_u64().unwrap()).unwrap(),
            usize::try_from(policy["maximumCapabilities"].as_u64().unwrap()).unwrap(),
            policy["minimumManifestGeneration"].as_u64().unwrap(),
        )
    }

    fn decode_pointer_token(token: &str) -> String {
        token.replace("~1", "/").replace("~0", "~")
    }

    fn apply_json_patch(document: &mut Value, patch: &Value) {
        let operation = patch["op"].as_str().unwrap();
        let path = patch["path"].as_str().unwrap();
        let (parent_path, token) = path.rsplit_once('/').unwrap();
        let token = decode_pointer_token(token);
        let parent = if parent_path.is_empty() {
            document
        } else {
            document.pointer_mut(parent_path).unwrap()
        };
        match (operation, parent) {
            ("add", Value::Object(object)) => {
                object.insert(token, patch["value"].clone());
            }
            ("replace", Value::Object(object)) => {
                assert!(
                    object.contains_key(&token),
                    "fixture replace path does not exist: {path}"
                );
                object.insert(token, patch["value"].clone());
            }
            ("add", Value::Array(array)) => {
                if token == "-" {
                    array.push(patch["value"].clone());
                } else {
                    array.insert(token.parse().unwrap(), patch["value"].clone());
                }
            }
            ("replace", Value::Array(array)) => {
                array[token.parse::<usize>().unwrap()] = patch["value"].clone();
            }
            ("remove", Value::Object(object)) => {
                object.remove(&token).unwrap();
            }
            ("remove", Value::Array(array)) => {
                array.remove(token.parse().unwrap());
            }
            _ => panic!("unsupported fixture patch operation {operation} at {path}"),
        }
    }

    fn fixture_artifacts(fixture: &Value, case: &Value) -> FixtureArtifacts {
        let chain = &fixture["completeChain"];
        let mut artifacts = FixtureArtifacts {
            root: serde_json::from_str(chain["rootCanonicalJson"].as_str().unwrap()).unwrap(),
            certificate: serde_json::from_str(chain["certificateCanonicalJson"].as_str().unwrap())
                .unwrap(),
            manifest: serde_json::from_str(chain["manifestCanonicalJson"].as_str().unwrap())
                .unwrap(),
            context: serde_json::from_str(chain["contextCanonicalJson"].as_str().unwrap()).unwrap(),
        };
        for patch in case["mutations"].as_array().into_iter().flatten() {
            let target = patch["target"].as_str().unwrap();
            let document = match target {
                "root" => &mut artifacts.root,
                "certificate" => &mut artifacts.certificate,
                "manifest" => &mut artifacts.manifest,
                "context" => &mut artifacts.context,
                _ => panic!("unknown fixture mutation target {target}"),
            };
            apply_json_patch(document, patch);
        }
        artifacts
    }

    fn unsigned_value(value: &Value) -> Value {
        let mut value = value.clone();
        value.as_object_mut().unwrap().remove("signature");
        value
    }

    fn prepare_fixture_artifacts(
        fixture: &Value,
        case: &Value,
    ) -> Result<FixtureArtifacts, ProtocolError> {
        let mut artifacts = fixture_artifacts(fixture, case);
        let chain = &fixture["completeChain"];
        let root_key = fixture_signing_key(chain["rootSeed"].as_str().unwrap());
        let issuer_key = fixture_signing_key(
            case["inputs"]["issuerSeed"]
                .as_str()
                .unwrap_or_else(|| chain["issuerSeed"].as_str().unwrap()),
        );
        let resign = case["resign"].as_array();

        if resign.is_some_and(|values| values.iter().any(|value| value == "certificate")) {
            let unsigned: UnsignedAuthorizationIssuerCertificateV1 =
                serde_json::from_value(unsigned_value(&artifacts.certificate))?;
            artifacts.certificate =
                serde_json::to_value(sign_issuer_certificate_v1(unsigned, &root_key)?)?;
        }

        if case["inputs"]["bindCertificate"].as_bool() == Some(true) {
            let certificate = parse_issuer_certificate_v1(&artifacts.certificate)?;
            let index = case["inputs"]["certificateEntryIndex"]
                .as_u64()
                .unwrap_or(0) as usize;
            artifacts.manifest["issuers"][index]["keyId"] =
                Value::String(certificate.unsigned.key_id.clone());
            artifacts.manifest["issuers"][index]["certificateDigest"] =
                Value::String(certificate.digest()?);
            if case["inputs"]["sortIssuers"].as_bool() == Some(true) {
                artifacts.manifest["issuers"]
                    .as_array_mut()
                    .unwrap()
                    .sort_by(|left, right| {
                        left["keyId"]
                            .as_str()
                            .unwrap()
                            .encode_utf16()
                            .cmp(right["keyId"].as_str().unwrap().encode_utf16())
                    });
            }
        }

        if resign.is_some_and(|values| values.iter().any(|value| value == "manifest")) {
            let unsigned: UnsignedAuthorizationIssuerManifestV1 =
                serde_json::from_value(unsigned_value(&artifacts.manifest))?;
            artifacts.manifest =
                serde_json::to_value(sign_issuer_manifest_v1(unsigned, &root_key)?)?;
        }

        if case["inputs"]["bindContextIssuer"].as_bool() == Some(true) {
            artifacts.context["issuerKeyId"] = artifacts.certificate["keyId"].clone();
        }
        if resign.is_some_and(|values| values.iter().any(|value| value == "context")) {
            let unsigned: UnsignedAuthorizationContextV1 =
                serde_json::from_value(unsigned_value(&artifacts.context))?;
            artifacts.context =
                serde_json::to_value(sign_authorization_context_v1(unsigned, &issuer_key)?)?;
        }
        for patch in case["postMutations"].as_array().into_iter().flatten() {
            let target = patch["target"].as_str().unwrap();
            let document = match target {
                "root" => &mut artifacts.root,
                "certificate" => &mut artifacts.certificate,
                "manifest" => &mut artifacts.manifest,
                "context" => &mut artifacts.context,
                _ => panic!("unknown fixture post-mutation target {target}"),
            };
            apply_json_patch(document, patch);
        }
        Ok(artifacts)
    }

    fn verified_manifest_from_fixture(
        artifacts: &FixtureArtifacts,
        policy: &AuthorizationVerificationPolicyV1,
    ) -> Result<
        (
            AuthorizationTrustRootV1,
            SignedAuthorizationIssuerCertificateV1,
            VerifiedAuthorizationIssuerManifestV1,
        ),
        ProtocolError,
    > {
        let root = AuthorizationTrustRootV1::parse(&artifacts.root)?;
        let certificate = parse_issuer_certificate_v1(&artifacts.certificate)?;
        let manifest = parse_issuer_manifest_v1(&artifacts.manifest)?;
        let manifest = verify_issuer_manifest_v1(&root, &manifest, policy)?;
        Ok((root, certificate, manifest))
    }

    fn verified_context_from_fixture(
        artifacts: &FixtureArtifacts,
        policy: &AuthorizationVerificationPolicyV1,
    ) -> Result<VerifiedAuthorizationContextV1, ProtocolError> {
        let (root, certificate, manifest) = verified_manifest_from_fixture(artifacts, policy)?;
        let context = parse_authorization_context_v1(&artifacts.context)?;
        verify_authorization_context_v1(&root, &manifest, &certificate, &context, policy)
    }

    fn verified_context_projection(
        context: &VerifiedAuthorizationContextV1,
    ) -> Result<Value, ProtocolError> {
        Ok(serde_json::json!({
            "authority": context.authority(),
            "authorityRef": context.authority_ref(),
            "principal": context.principal(),
            "participant": context.participant(),
            "deploymentId": context.deployment_id(),
            "instanceId": context.instance_id(),
            "contextId": context.context_id(),
            "issuerKeyId": context.signed_context().unsigned.issuer_key_id,
            "sessionId": context.session_id(),
            "sessionKey": encode_base64url(context.session_key().as_bytes()),
            "inboxPrefix": context.inbox_prefix(),
            "issuedAt": context.issued_at(),
            "notBefore": context.not_before(),
            "expiresAt": context.expires_at(),
            "grantSet": context.grant_set(),
            "grantDigest": context.grant_set().digest()?,
            "capabilities": context.capabilities(),
            "extensions": context.signed_context().unsigned.extensions,
            "contextDigest": context.context_digest(),
        }))
    }

    fn fixture_request_value<'a>(fixture: &'a Value, case: &'a Value, kind: &str) -> &'a Value {
        if case["inputs"][kind].is_object() {
            &case["inputs"][kind]
        } else {
            &fixture["defaults"]["request"]
        }
    }

    fn fixture_permissions(values: &Value) -> Result<Vec<PermissionAtomV1>, ProtocolError> {
        values
            .as_array()
            .into_iter()
            .flatten()
            .map(|value| Ok(serde_json::from_value(value.clone())?))
            .collect()
    }

    fn execute_fixture_case(fixture: &Value, case: &Value) -> Result<Value, ProtocolError> {
        let artifacts = prepare_fixture_artifacts(fixture, case)?;
        let policy = fixture_policy_from(&fixture["defaults"], case)?;
        match case["operation"].as_str().unwrap() {
            "constructTrustRoot" => {
                let root = AuthorizationTrustRootV1::parse(&artifacts.root)?;
                Ok(serde_json::json!({
                    "keyId": root.key_id(),
                    "canonicalJson": root.canonical_json()?,
                    "digest": root.digest()?,
                }))
            }
            "parseCertificate" => {
                let certificate = parse_issuer_certificate_v1(&artifacts.certificate)?;
                Ok(serde_json::json!({ "digest": certificate.digest()? }))
            }
            "signCertificate" => {
                let unsigned: UnsignedAuthorizationIssuerCertificateV1 =
                    serde_json::from_value(unsigned_value(&artifacts.certificate))?;
                let key =
                    fixture_signing_key(fixture["completeChain"]["rootSeed"].as_str().unwrap());
                let signed = sign_issuer_certificate_v1(unsigned, &key)?;
                Ok(serde_json::json!({ "digest": signed.digest()? }))
            }
            "verifyCertificate" => {
                let root = AuthorizationTrustRootV1::parse(&artifacts.root)?;
                let certificate = parse_issuer_certificate_v1(&artifacts.certificate)?;
                verify_issuer_certificate_v1(&root, &certificate, &policy)?;
                Ok(serde_json::json!({ "digest": certificate.digest()? }))
            }
            "parseManifest" => {
                let manifest = parse_issuer_manifest_v1(&artifacts.manifest)?;
                Ok(serde_json::json!({ "digest": manifest.digest()? }))
            }
            "signManifest" => {
                let unsigned: UnsignedAuthorizationIssuerManifestV1 =
                    serde_json::from_value(unsigned_value(&artifacts.manifest))?;
                let key =
                    fixture_signing_key(fixture["completeChain"]["rootSeed"].as_str().unwrap());
                let signed = sign_issuer_manifest_v1(unsigned, &key)?;
                Ok(serde_json::json!({ "digest": signed.digest()? }))
            }
            "verifyManifest" => {
                let root = AuthorizationTrustRootV1::parse(&artifacts.root)?;
                let manifest = parse_issuer_manifest_v1(&artifacts.manifest)?;
                let verified = verify_issuer_manifest_v1(&root, &manifest, &policy)?;
                let active = verified
                    .manifest()
                    .unsigned
                    .issuers
                    .iter()
                    .filter(|entry| entry.status == AuthorizationIssuerStatusV1::Active)
                    .map(|entry| entry.key_id.clone())
                    .collect::<Vec<_>>();
                Ok(serde_json::json!({
                    "authority": verified.authority(),
                    "rootKeyId": verified.root_key_id(),
                    "generation": verified.generation(),
                    "digest": verified.digest()?,
                    "activeIssuerKeyIds": active,
                }))
            }
            "parseContext" => {
                let context = parse_authorization_context_v1(&artifacts.context)?;
                Ok(serde_json::json!({ "digest": context.digest()? }))
            }
            "signContext" => {
                let unsigned: UnsignedAuthorizationContextV1 =
                    serde_json::from_value(unsigned_value(&artifacts.context))?;
                let seed = case["inputs"]["issuerSeed"]
                    .as_str()
                    .unwrap_or_else(|| fixture["completeChain"]["issuerSeed"].as_str().unwrap());
                let signed = sign_authorization_context_v1(unsigned, &fixture_signing_key(seed))?;
                Ok(serde_json::json!({ "digest": signed.digest()? }))
            }
            "verifyContext" => {
                let context = verified_context_from_fixture(&artifacts, &policy)?;
                verified_context_projection(&context)
            }
            "verifyContextAtTimes" => {
                let times = case["inputs"]["times"].as_array().unwrap();
                for time in times {
                    let mut time_policy = policy.clone();
                    time_policy.now_unix_seconds = time.as_i64().unwrap();
                    verified_context_from_fixture(&artifacts, &time_policy)?;
                }
                Ok(serde_json::json!({ "acceptedTimes": times }))
            }
            "contextToken" => {
                let context = parse_authorization_context_v1(&artifacts.context)?;
                let token = encode_authorization_context_token_v1(&context)?;
                let parsed = parse_authorization_context_token_v1(&token, &policy)?;
                Ok(serde_json::json!({ "token": token, "digest": parsed.digest()? }))
            }
            "verifyStaleManifest" => {
                let mut initial_policy = policy.clone();
                initial_policy.minimum_manifest_generation = case["inputs"]
                    ["initialMinimumManifestGeneration"]
                    .as_u64()
                    .unwrap();
                let (root, certificate, manifest) =
                    verified_manifest_from_fixture(&artifacts, &initial_policy)?;
                let manifest = if case["inputs"]["clone"].as_bool() == Some(true) {
                    manifest.clone()
                } else {
                    manifest
                };
                let context = parse_authorization_context_v1(&artifacts.context)?;
                let verified = verify_authorization_context_v1(
                    &root,
                    &manifest,
                    &certificate,
                    &context,
                    &policy,
                )?;
                verified_context_projection(&verified)
            }
            "buildRequestProof" => {
                let request = fixture_request_value(fixture, case, "signedRequest");
                let context_digest = decode_base64url::<32>(
                    fixture["completeChain"]["contextDigest"].as_str().unwrap(),
                    &[],
                    AuthorizationErrorCodeV1::InvalidEncoding,
                )?;
                let input = build_authorization_request_proof_input_v2(
                    &context_digest,
                    request["subject"].as_str().unwrap(),
                    request["reply"].as_str(),
                    request["payload"].as_str().unwrap().as_bytes(),
                    request["iat"].as_i64().unwrap(),
                    request["requestId"].as_str().unwrap(),
                )?;
                Ok(serde_json::json!({
                    "digest": encode_base64url(input.digest()),
                }))
            }
            "verifyRequest" => {
                let context = verified_context_from_fixture(&artifacts, &policy)?;
                let signed_request = fixture_request_value(fixture, case, "signedRequest");
                let verification_request =
                    fixture_request_value(fixture, case, "verificationRequest");
                let proof = if case["inputs"]["usePinnedProof"].as_bool() == Some(true) {
                    AuthorizationRequestProofV2::parse(
                        fixture["completeChain"]["requestProof"].as_str().unwrap(),
                    )?
                } else {
                    let seed = signed_request["sessionSeed"].as_str().unwrap_or_else(|| {
                        fixture["completeChain"]["sessionSeed"].as_str().unwrap()
                    });
                    sign_authorization_request_v2(
                        context.context_digest(),
                        signed_request["subject"].as_str().unwrap(),
                        signed_request["reply"].as_str(),
                        signed_request["payload"].as_str().unwrap().as_bytes(),
                        signed_request["iat"].as_i64().unwrap(),
                        signed_request["requestId"].as_str().unwrap(),
                        &fixture_signing_key(seed),
                    )?
                };
                let permissions = fixture_permissions(&case["inputs"]["requiredPermissions"])?;
                let capabilities = case["inputs"]["requiredCapabilities"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|value| value.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>();
                let verified = verify_authorization_request_v2(
                    &context,
                    verification_request["subject"].as_str().unwrap(),
                    verification_request["reply"].as_str(),
                    verification_request["payload"].as_str().unwrap().as_bytes(),
                    verification_request["iat"].as_i64().unwrap(),
                    verification_request["requestId"].as_str().unwrap(),
                    &proof,
                    &policy,
                    &permissions,
                    &capabilities,
                )?;
                let mut projection = verified_context_projection(verified.context())?;
                projection["replayKey"] = serde_json::to_value(serde_json::json!({
                    "contextId": verified.replay_key().context_id,
                    "requestId": verified.replay_key().request_id,
                }))?;
                projection["replayExpiresAt"] = Value::from(verified.replay_expires_at());
                Ok(projection)
            }
            operation => panic!("unknown conformance operation {operation}"),
        }
    }

    fn assert_json_subset(expected: &Value, actual: &Value, location: &str) {
        match expected {
            Value::Object(expected) => {
                let actual = actual
                    .as_object()
                    .unwrap_or_else(|| panic!("{location}: expected object, got {actual}"));
                for (key, value) in expected {
                    assert_json_subset(
                        value,
                        actual
                            .get(key)
                            .unwrap_or_else(|| panic!("{location}: missing output key {key}")),
                        &format!("{location}/{key}"),
                    );
                }
            }
            _ => assert_eq!(expected, actual, "{location}"),
        }
    }

    fn assert_fixture_case(fixture: &Value, case: &Value) {
        let name = case["name"].as_str().unwrap();
        let result = execute_fixture_case(fixture, case);
        if case["expected"]["valid"].as_bool().unwrap() {
            let output = result.unwrap_or_else(|error| panic!("{name}: {error}"));
            if !case["expected"]["output"].is_null() {
                assert_json_subset(&case["expected"]["output"], &output, name);
            }
        } else {
            let error = match result {
                Ok(output) => panic!("{name}: unexpectedly succeeded with {output}"),
                Err(error) => error,
            };
            match error {
                ProtocolError::Authorization {
                    code,
                    path,
                    message,
                } => {
                    assert_eq!(format!("{code:?}"), case["expected"]["code"], "{name}");
                    assert_eq!(path.as_str(), case["expected"]["path"], "{name}");
                    for secret in ["rootSeed", "issuerSeed", "sessionSeed", "requestProof"] {
                        if let Some(secret) = fixture["completeChain"][secret].as_str() {
                            assert!(!message.contains(secret), "{name}: error leaked {secret}");
                        }
                    }
                    if let Some(payload) = fixture["defaults"]["request"]["payload"].as_str() {
                        assert!(!message.contains(payload), "{name}: error leaked payload");
                    }
                }
                error => panic!("{name}: unexpected error {error}"),
            }
        }
    }

    fn chain() -> (
        AuthorizationTrustRootV1,
        SignedAuthorizationIssuerCertificateV1,
        VerifiedAuthorizationIssuerManifestV1,
        SignedAuthorizationContextV1,
        SigningKey,
    ) {
        let root_key = SigningKey::from_bytes(&[1; 32]);
        let issuer_key = SigningKey::from_bytes(&[2; 32]);
        let session_key = SigningKey::from_bytes(&[3; 32]);
        let root_public = encode_base64url(root_key.verifying_key().as_bytes());
        let issuer_public = encode_base64url(issuer_key.verifying_key().as_bytes());
        let session_public = encode_base64url(session_key.verifying_key().as_bytes());
        let root = AuthorizationTrustRootV1::new("trellis-test", root_public).unwrap();
        let certificate = sign_issuer_certificate_v1(
            UnsignedAuthorizationIssuerCertificateV1 {
                format: AUTHORIZATION_ISSUER_CERTIFICATE_FORMAT_V1.to_owned(),
                authority: root.authority().to_owned(),
                root_key_id: root.key_id().to_owned(),
                serial: "isc_test".to_owned(),
                key_id: derived_key_id(&issuer_key.verifying_key()),
                public_key: issuer_public,
                issued_at: 1_000,
                not_before: 1_000,
                expires_at: 2_000,
                usages: vec![AUTHORIZATION_CONTEXT_USAGE.to_owned()],
                extensions: Map::new(),
                critical: Vec::new(),
            },
            &root_key,
        )
        .unwrap();
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
                    key_id: certificate.unsigned.key_id.clone(),
                    certificate_digest: certificate.digest().unwrap(),
                    status: AuthorizationIssuerStatusV1::Active,
                    revoked_at: None,
                }],
                extensions: Map::new(),
                critical: Vec::new(),
            },
            &root_key,
        )
        .unwrap();
        let verified_manifest =
            verify_issuer_manifest_v1(&root, &manifest, &fixture_policy(1_100)).unwrap();
        let context = sign_authorization_context_v1(
            UnsignedAuthorizationContextV1 {
                format: AUTHORIZATION_CONTEXT_FORMAT_V1.to_owned(),
                authority: root.authority().to_owned(),
                context_id: "ctx_test".to_owned(),
                issuer_key_id: certificate.unsigned.key_id.clone(),
                session_id: "ses_test".to_owned(),
                session_key: session_public,
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
                critical: Vec::new(),
            },
            &issuer_key,
        )
        .unwrap();
        (root, certificate, verified_manifest, context, session_key)
    }

    #[test]
    fn verifies_complete_chain_and_context_bound_request() {
        let (root, certificate, manifest, context, session_key) = chain();
        let policy = fixture_policy(1_100);
        let verified =
            verify_authorization_context_v1(&root, &manifest, &certificate, &context, &policy)
                .unwrap();
        assert_eq!(
            parse_issuer_certificate_v1(&serde_json::to_value(&certificate).unwrap()).unwrap(),
            certificate
        );
        assert_eq!(
            parse_authorization_context_v1(&serde_json::to_value(&context).unwrap()).unwrap(),
            context
        );
        let token = encode_authorization_context_token_v1(&context).unwrap();
        assert_eq!(
            parse_authorization_context_token_v1(&token, &policy).unwrap(),
            context
        );
        assert!(verified.allows(&permission()));
        assert!(verified.has_capability("platform.read"));

        let proof = sign_authorization_request_v2(
            verified.context_digest(),
            "rpc.v1.Documents.Get",
            Some("_INBOX.test.reply"),
            br#"{"id":"doc-1"}"#,
            1_100,
            "req_test",
            &session_key,
        )
        .unwrap();
        let request = verify_authorization_request_v2(
            &verified,
            "rpc.v1.Documents.Get",
            Some("_INBOX.test.reply"),
            br#"{"id":"doc-1"}"#,
            1_100,
            "req_test",
            &proof,
            &policy,
            &[permission()],
            &["platform.read".to_owned()],
        )
        .unwrap();
        assert_eq!(request.replay_key().context_id, "ctx_test");
        assert_eq!(request.replay_expires_at(), 1_330);
    }

    #[test]
    fn rejects_tampered_request_components_and_authority_subsets() {
        let (root, certificate, manifest, context, session_key) = chain();
        let policy = fixture_policy(1_100);
        let verified =
            verify_authorization_context_v1(&root, &manifest, &certificate, &context, &policy)
                .unwrap();
        let proof = sign_authorization_request_v2(
            verified.context_digest(),
            "rpc.v1.Documents.Get",
            None,
            b"payload",
            1_100,
            "req_test",
            &session_key,
        )
        .unwrap();
        let changed = verify_authorization_request_v2(
            &verified,
            "rpc.v1.Documents.Other",
            None,
            b"payload",
            1_100,
            "req_test",
            &proof,
            &policy,
            &[],
            &[],
        )
        .unwrap_err();
        assert!(matches!(
            changed,
            ProtocolError::Authorization {
                code: AuthorizationErrorCodeV1::InvalidRequestProof,
                ..
            }
        ));

        let missing = PermissionAtomV1::new(
            PermissionTargetV1::api_surface(
                "documents@v1",
                ApiSurfaceKindV1::Rpc,
                "Documents.Delete",
            )
            .unwrap(),
            PermissionActionV1::Call,
        )
        .unwrap();
        assert!(matches!(
            verify_authorization_request_v2(
                &verified,
                "rpc.v1.Documents.Get",
                None,
                b"payload",
                1_100,
                "req_test",
                &proof,
                &policy,
                &[missing],
                &[],
            ),
            Err(ProtocolError::Authorization {
                code: AuthorizationErrorCodeV1::PermissionDenied,
                ..
            })
        ));
    }

    #[test]
    fn escapes_critical_extension_error_paths() {
        let mut extensions = Map::new();
        extensions.insert("future/~policy".to_owned(), Value::Bool(true));
        let error = validate_extensions(&extensions, &["future/~policy".to_owned()]).unwrap_err();
        match error {
            ProtocolError::Authorization { path, .. } => {
                assert_eq!(path.as_str(), "/extensions/future~1~0policy");
            }
            _ => panic!("unexpected error"),
        }
    }

    #[test]
    fn strict_signed_objects_reject_unknown_top_level_fields() {
        let (_, _, _, context, _) = chain();
        let mut value = serde_json::to_value(context).unwrap();
        value["unexpected"] = Value::Bool(true);
        assert!(matches!(
            parse_authorization_context_v1(&value),
            Err(ProtocolError::Authorization {
                code: AuthorizationErrorCodeV1::InvalidFormat,
                ..
            })
        ));
    }

    #[test]
    fn rejects_noncanonical_keys_signatures_sets_and_extensions() {
        assert_authorization_error(
            AuthorizationTrustRootV1::new("trellis-test", "AQ==").unwrap_err(),
            AuthorizationErrorCodeV1::InvalidEncoding,
            "/publicKey",
        );
        assert_authorization_error(
            AuthorizationTrustRootV1::new("trellis-test", "AQ").unwrap_err(),
            AuthorizationErrorCodeV1::InvalidPublicKey,
            "/publicKey",
        );

        let (root, certificate, _, context, _) = chain();
        let mut root_value = root.normalized_value().unwrap();
        root_value["keyId"] = Value::String(encode_base64url(&[0; 32]));
        assert_authorization_error(
            AuthorizationTrustRootV1::parse(&root_value).unwrap_err(),
            AuthorizationErrorCodeV1::InvalidKeyId,
            "/keyId",
        );

        let mut certificate_value = serde_json::to_value(&certificate).unwrap();
        certificate_value["signature"] = Value::String("AQ".to_owned());
        assert_authorization_error(
            parse_issuer_certificate_v1(&certificate_value).unwrap_err(),
            AuthorizationErrorCodeV1::InvalidSignature,
            "/signature",
        );
        let root_key = SigningKey::from_bytes(&[1; 32]);
        let mut malformed_issuer = certificate.unsigned.clone();
        malformed_issuer.public_key = "AQ".into();
        assert_authorization_error(
            sign_issuer_certificate_v1(malformed_issuer, &root_key).unwrap_err(),
            AuthorizationErrorCodeV1::InvalidPublicKey,
            "/publicKey",
        );

        let issuer_key = SigningKey::from_bytes(&[2; 32]);
        let mut duplicate_capabilities = context.unsigned.clone();
        duplicate_capabilities.capabilities = vec!["platform.read".into(), "platform.read".into()];
        assert_authorization_error(
            sign_authorization_context_v1(duplicate_capabilities, &issuer_key).unwrap_err(),
            AuthorizationErrorCodeV1::NonCanonicalSet,
            "/capabilities",
        );
        let mut malformed_session = context.unsigned.clone();
        malformed_session.session_key = "AQ".into();
        assert_authorization_error(
            sign_authorization_context_v1(malformed_session, &issuer_key).unwrap_err(),
            AuthorizationErrorCodeV1::InvalidSessionKey,
            "/sessionKey",
        );
        let mut wrong_user_authority = context.unsigned.clone();
        wrong_user_authority.authority_ref.kind = AuthorizationAuthorityKindV1::Deployment;
        assert_authorization_error(
            sign_authorization_context_v1(wrong_user_authority, &issuer_key).unwrap_err(),
            AuthorizationErrorCodeV1::InvalidFormat,
            "/authorityRef/kind",
        );

        let mut utf16_capabilities = context.unsigned.clone();
        utf16_capabilities.capabilities = vec!["cap.\u{10000}".into(), "cap.\u{e000}".into()];
        assert!(sign_authorization_context_v1(utf16_capabilities, &issuer_key).is_ok());

        let second_permission = PermissionAtomV1::new(
            PermissionTargetV1::api_surface(
                "documents@v1",
                ApiSurfaceKindV1::Rpc,
                "Documents.List",
            )
            .unwrap(),
            PermissionActionV1::Call,
        )
        .unwrap();
        let ordered = sign_authorization_context_v1(
            UnsignedAuthorizationContextV1 {
                grant_set: GrantSetV1::new(vec![second_permission, permission()]),
                ..context.unsigned.clone()
            },
            &issuer_key,
        )
        .unwrap();
        let mut unordered = serde_json::to_value(ordered).unwrap();
        unordered["grantSet"]["permissions"]
            .as_array_mut()
            .unwrap()
            .reverse();
        assert_authorization_error(
            parse_authorization_context_v1(&unordered).unwrap_err(),
            AuthorizationErrorCodeV1::NonCanonicalSet,
            "/grantSet/permissions",
        );

        let mut critical = context.unsigned.clone();
        critical.critical = vec!["future/~policy".into()];
        assert_authorization_error(
            sign_authorization_context_v1(critical.clone(), &issuer_key).unwrap_err(),
            AuthorizationErrorCodeV1::InvalidFormat,
            "/critical/0",
        );
        critical
            .extensions
            .insert("future/~policy".into(), Value::Bool(true));
        assert_authorization_error(
            sign_authorization_context_v1(critical, &issuer_key).unwrap_err(),
            AuthorizationErrorCodeV1::UnknownCriticalExtension,
            "/extensions/future~1~0policy",
        );

        let mut extension = context.unsigned.clone();
        extension
            .extensions
            .insert("trace".into(), serde_json::json!({"v": 1}));
        let extension = sign_authorization_context_v1(extension, &issuer_key).unwrap();
        assert_eq!(
            serde_json::to_value(extension).unwrap()["extensions"]["trace"],
            serde_json::json!({"v": 1})
        );
    }

    #[test]
    fn enforces_certificate_manifest_and_context_validity() {
        let (root, certificate, manifest, context, _) = chain();
        let root_key = SigningKey::from_bytes(&[1; 32]);
        let issuer_key = SigningKey::from_bytes(&[2; 32]);

        let mut invalid_certificate = certificate.unsigned.clone();
        invalid_certificate.expires_at = 999;
        assert_authorization_error(
            sign_issuer_certificate_v1(invalid_certificate, &root_key).unwrap_err(),
            AuthorizationErrorCodeV1::InvalidValidityWindow,
            "/expiresAt",
        );
        let mut missing_usage = certificate.unsigned.clone();
        missing_usage.usages.clear();
        assert_authorization_error(
            sign_issuer_certificate_v1(missing_usage, &root_key).unwrap_err(),
            AuthorizationErrorCodeV1::CertificateUsageMissing,
            "/usages",
        );
        let wrong_root = SigningKey::from_bytes(&[9; 32]);
        let wrong_signature = SignedAuthorizationIssuerCertificateV1 {
            unsigned: certificate.unsigned.clone(),
            signature: encode_base64url(
                &wrong_root
                    .sign(&issuer_certificate_signing_digest_v1(&certificate.unsigned).unwrap())
                    .to_bytes(),
            ),
        };
        assert_authorization_error(
            verify_issuer_certificate_v1(&root, &wrong_signature, &fixture_policy(1_100))
                .unwrap_err(),
            AuthorizationErrorCodeV1::InvalidSignature,
            "/signature",
        );
        let wrong_authority = sign_issuer_certificate_v1(
            UnsignedAuthorizationIssuerCertificateV1 {
                authority: "trellis-other".into(),
                ..certificate.unsigned.clone()
            },
            &root_key,
        )
        .unwrap();
        assert_authorization_error(
            verify_issuer_certificate_v1(&root, &wrong_authority, &fixture_policy(1_100))
                .unwrap_err(),
            AuthorizationErrorCodeV1::WrongAuthority,
            "/authority",
        );

        let mut zero_generation = manifest.manifest().unsigned.clone();
        zero_generation.generation = 0;
        assert_authorization_error(
            sign_issuer_manifest_v1(zero_generation, &root_key).unwrap_err(),
            AuthorizationErrorCodeV1::InvalidFormat,
            "/generation",
        );
        let rollback_policy =
            AuthorizationVerificationPolicyV1::new(1_100, 30, 300, 16_384, 16, 16, 8).unwrap();
        assert_authorization_error(
            verify_issuer_manifest_v1(&root, manifest.manifest(), &rollback_policy).unwrap_err(),
            AuthorizationErrorCodeV1::ManifestRollback,
            "/generation",
        );
        let mut invalid_manifest_signature = manifest.manifest().clone();
        invalid_manifest_signature.signature = encode_base64url(&[0; 64]);
        assert_authorization_error(
            verify_issuer_manifest_v1(&root, &invalid_manifest_signature, &fixture_policy(1_100))
                .unwrap_err(),
            AuthorizationErrorCodeV1::InvalidSignature,
            "/signature",
        );
        let other_root_key = SigningKey::from_bytes(&[9; 32]);
        let other_root = AuthorizationTrustRootV1::new(
            "trellis-other",
            encode_base64url(other_root_key.verifying_key().as_bytes()),
        )
        .unwrap();
        let other_manifest = sign_issuer_manifest_v1(
            UnsignedAuthorizationIssuerManifestV1 {
                authority: other_root.authority().into(),
                root_key_id: other_root.key_id().into(),
                ..manifest.manifest().unsigned.clone()
            },
            &other_root_key,
        )
        .unwrap();
        let other_manifest =
            verify_issuer_manifest_v1(&other_root, &other_manifest, &fixture_policy(1_100))
                .unwrap();
        assert_authorization_error(
            verify_authorization_context_v1(
                &root,
                &other_manifest,
                &certificate,
                &context,
                &fixture_policy(1_100),
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::WrongAuthority,
            "/authority",
        );
        let mut duplicate = manifest.manifest().unsigned.clone();
        duplicate.issuers.push(duplicate.issuers[0].clone());
        assert_authorization_error(
            sign_issuer_manifest_v1(duplicate, &root_key).unwrap_err(),
            AuthorizationErrorCodeV1::NonCanonicalSet,
            "/issuers",
        );
        let mut invalid_status = manifest.manifest().unsigned.clone();
        invalid_status.issuers[0].revoked_at = Some(1_050);
        assert_authorization_error(
            sign_issuer_manifest_v1(invalid_status, &root_key).unwrap_err(),
            AuthorizationErrorCodeV1::InvalidFormat,
            "/issuers/0/revokedAt",
        );
        let mut missing_revoked_at = manifest.manifest().unsigned.clone();
        missing_revoked_at.issuers[0].status = AuthorizationIssuerStatusV1::Revoked;
        assert_authorization_error(
            sign_issuer_manifest_v1(missing_revoked_at, &root_key).unwrap_err(),
            AuthorizationErrorCodeV1::InvalidFormat,
            "/issuers/0/revokedAt",
        );

        let omitted = sign_issuer_manifest_v1(
            UnsignedAuthorizationIssuerManifestV1 {
                issuers: vec![],
                ..manifest.manifest().unsigned.clone()
            },
            &root_key,
        )
        .unwrap();
        let omitted = verify_issuer_manifest_v1(&root, &omitted, &fixture_policy(1_100)).unwrap();
        assert_authorization_error(
            verify_authorization_context_v1(
                &root,
                &omitted,
                &certificate,
                &context,
                &fixture_policy(1_100),
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::IssuerNotListed,
            "/issuerKeyId",
        );

        let mut revoked = manifest.manifest().unsigned.clone();
        revoked.issuers[0].status = AuthorizationIssuerStatusV1::Revoked;
        revoked.issuers[0].revoked_at = Some(1_050);
        let revoked = sign_issuer_manifest_v1(revoked, &root_key).unwrap();
        let revoked = verify_issuer_manifest_v1(&root, &revoked, &fixture_policy(1_100)).unwrap();
        assert_authorization_error(
            verify_authorization_context_v1(
                &root,
                &revoked,
                &certificate,
                &context,
                &fixture_policy(1_100),
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::IssuerRevoked,
            "/issuerKeyId",
        );

        let mut wrong_digest = manifest.manifest().unsigned.clone();
        wrong_digest.issuers[0].certificate_digest = encode_base64url(&[8; 32]);
        let wrong_digest = sign_issuer_manifest_v1(wrong_digest, &root_key).unwrap();
        let wrong_digest =
            verify_issuer_manifest_v1(&root, &wrong_digest, &fixture_policy(1_100)).unwrap();
        assert_authorization_error(
            verify_authorization_context_v1(
                &root,
                &wrong_digest,
                &certificate,
                &context,
                &fixture_policy(1_100),
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::CertificateDigestMismatch,
            "/issuerKeyId",
        );

        for (not_before, expires_at, now, code, path) in [
            (
                1_100,
                1_401,
                1_100,
                AuthorizationErrorCodeV1::ContextLifetimeExceeded,
                "/expiresAt",
            ),
            (
                999,
                1_200,
                1_100,
                AuthorizationErrorCodeV1::ContextOutlivesCertificate,
                "/expiresAt",
            ),
            (
                1_250,
                1_501,
                1_100,
                AuthorizationErrorCodeV1::ContextOutlivesManifest,
                "/expiresAt",
            ),
            (
                1_200,
                1_300,
                1_100,
                AuthorizationErrorCodeV1::ContextNotYetValid,
                "/notBefore",
            ),
            (
                1_000,
                1_050,
                1_100,
                AuthorizationErrorCodeV1::ContextExpired,
                "/expiresAt",
            ),
        ] {
            let candidate = sign_authorization_context_v1(
                UnsignedAuthorizationContextV1 {
                    issued_at: not_before,
                    not_before,
                    expires_at,
                    ..context.unsigned.clone()
                },
                &issuer_key,
            )
            .unwrap();
            assert_authorization_error(
                verify_authorization_context_v1(
                    &root,
                    &manifest,
                    &certificate,
                    &candidate,
                    &fixture_policy(now),
                )
                .unwrap_err(),
                code,
                path,
            );
        }

        let invalid_window = UnsignedAuthorizationContextV1 {
            not_before: 1_200,
            issued_at: 1_100,
            ..context.unsigned.clone()
        };
        assert_authorization_error(
            sign_authorization_context_v1(invalid_window, &issuer_key).unwrap_err(),
            AuthorizationErrorCodeV1::InvalidValidityWindow,
            "/expiresAt",
        );
        let wrong_context_authority = sign_authorization_context_v1(
            UnsignedAuthorizationContextV1 {
                authority: "trellis-other".into(),
                ..context.unsigned.clone()
            },
            &issuer_key,
        )
        .unwrap();
        assert_authorization_error(
            verify_authorization_context_v1(
                &root,
                &manifest,
                &certificate,
                &wrong_context_authority,
                &fixture_policy(1_100),
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::WrongAuthority,
            "/authority",
        );
        let mut invalid_context_signature = context.clone();
        invalid_context_signature.signature = encode_base64url(&[0; 64]);
        assert_authorization_error(
            verify_authorization_context_v1(
                &root,
                &manifest,
                &certificate,
                &invalid_context_signature,
                &fixture_policy(1_100),
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::InvalidSignature,
            "/signature",
        );

        for now in [1_070, 1_330] {
            assert!(verify_authorization_context_v1(
                &root,
                &manifest,
                &certificate,
                &context,
                &fixture_policy(now),
            )
            .is_ok());
        }
    }

    #[test]
    fn verifies_service_device_rotation_and_token_limits() {
        let (root, certificate, manifest, context, _) = chain();
        let root_key = SigningKey::from_bytes(&[1; 32]);
        let issuer_key = SigningKey::from_bytes(&[2; 32]);
        for (principal_kind, participant_kind, principal_id) in [
            (
                AuthorizationPrincipalKindV1::Service,
                ParticipantKindV1::Service,
                "svc_documents",
            ),
            (
                AuthorizationPrincipalKindV1::Device,
                ParticipantKindV1::Device,
                "dev_scanner",
            ),
        ] {
            let candidate = sign_authorization_context_v1(
                UnsignedAuthorizationContextV1 {
                    principal: AuthorizationPrincipalV1 {
                        kind: principal_kind,
                        id: principal_id.into(),
                    },
                    participant: AuthorizationParticipantV1 {
                        kind: participant_kind,
                        id: principal_id.into(),
                        ..context.unsigned.participant.clone()
                    },
                    authority_ref: AuthorizationAuthorityRefV1 {
                        kind: AuthorizationAuthorityKindV1::Deployment,
                        id: "dep_documents".into(),
                        version: 3,
                    },
                    deployment_id: Some("dep_documents".into()),
                    instance_id: Some("ins_01".into()),
                    ..context.unsigned.clone()
                },
                &issuer_key,
            )
            .unwrap();
            assert!(verify_authorization_context_v1(
                &root,
                &manifest,
                &certificate,
                &candidate,
                &fixture_policy(1_100),
            )
            .is_ok());
        }

        let newer_key = SigningKey::from_bytes(&[6; 32]);
        let newer_certificate = sign_issuer_certificate_v1(
            UnsignedAuthorizationIssuerCertificateV1 {
                key_id: derived_key_id(&newer_key.verifying_key()),
                public_key: encode_base64url(newer_key.verifying_key().as_bytes()),
                serial: "isc_newer".into(),
                ..certificate.unsigned.clone()
            },
            &root_key,
        )
        .unwrap();
        let mut issuers = vec![
            manifest.manifest().unsigned.issuers[0].clone(),
            AuthorizationIssuerManifestEntryV1 {
                key_id: newer_certificate.unsigned.key_id.clone(),
                certificate_digest: newer_certificate.digest().unwrap(),
                status: AuthorizationIssuerStatusV1::Active,
                revoked_at: None,
            },
        ];
        let mut reversed_issuers = issuers.clone();
        reversed_issuers
            .sort_by(|left, right| right.key_id.encode_utf16().cmp(left.key_id.encode_utf16()));
        assert_authorization_error(
            sign_issuer_manifest_v1(
                UnsignedAuthorizationIssuerManifestV1 {
                    generation: 8,
                    issuers: reversed_issuers,
                    ..manifest.manifest().unsigned.clone()
                },
                &root_key,
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::NonCanonicalSet,
            "/issuers",
        );
        issuers.sort_by(|left, right| left.key_id.encode_utf16().cmp(right.key_id.encode_utf16()));
        let overlap = sign_issuer_manifest_v1(
            UnsignedAuthorizationIssuerManifestV1 {
                generation: 8,
                issuers,
                ..manifest.manifest().unsigned.clone()
            },
            &root_key,
        )
        .unwrap();
        let overlap_policy =
            AuthorizationVerificationPolicyV1::new(1_100, 30, 300, 16_384, 16, 16, 8).unwrap();
        let overlap = verify_issuer_manifest_v1(&root, &overlap, &overlap_policy).unwrap();
        assert!(overlap
            .active_certificate_digest(&certificate.unsigned.key_id)
            .is_some());
        assert!(overlap
            .active_certificate_digest(&newer_certificate.unsigned.key_id)
            .is_some());
        let newer_context = sign_authorization_context_v1(
            UnsignedAuthorizationContextV1 {
                issuer_key_id: newer_certificate.unsigned.key_id.clone(),
                ..context.unsigned.clone()
            },
            &newer_key,
        )
        .unwrap();
        assert_authorization_error(
            verify_authorization_context_v1(
                &root,
                &manifest,
                &newer_certificate,
                &newer_context,
                &fixture_policy(1_100),
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::IssuerNotListed,
            "/issuerKeyId",
        );
        assert_authorization_error(
            verify_authorization_context_v1(
                &root,
                &manifest,
                &certificate,
                &newer_context,
                &fixture_policy(1_100),
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::IssuerNotListed,
            "/issuerKeyId",
        );
        assert!(verify_authorization_context_v1(
            &root,
            &overlap,
            &newer_certificate,
            &newer_context,
            &overlap_policy,
        )
        .is_ok());

        let empty_authority = sign_authorization_context_v1(
            UnsignedAuthorizationContextV1 {
                grant_set: GrantSetV1::new(vec![]),
                capabilities: vec![],
                ..context.unsigned.clone()
            },
            &issuer_key,
        )
        .unwrap();
        let empty_authority = verify_authorization_context_v1(
            &root,
            &manifest,
            &certificate,
            &empty_authority,
            &fixture_policy(1_100),
        )
        .unwrap();
        assert!(empty_authority.grant_set().permissions().is_empty());
        assert!(empty_authority.capabilities().is_empty());

        let token = encode_authorization_context_token_v1(&context).unwrap();
        let tiny_policy =
            AuthorizationVerificationPolicyV1::new(1_100, 30, 300, 1, 16, 16, 7).unwrap();
        assert_authorization_error(
            parse_authorization_context_token_v1(&token, &tiny_policy).unwrap_err(),
            AuthorizationErrorCodeV1::ContextTokenTooLarge,
            "",
        );
        assert_authorization_error(
            verify_authorization_context_v1(&root, &manifest, &certificate, &context, &tiny_policy)
                .unwrap_err(),
            AuthorizationErrorCodeV1::ContextTokenTooLarge,
            "",
        );
        let canonical = canonicalize_json(&serde_json::to_value(&context).unwrap()).unwrap();
        let noncanonical = encode_base64url(format!(" {canonical}").as_bytes());
        assert_authorization_error(
            parse_authorization_context_token_v1(&noncanonical, &fixture_policy(1_100))
                .unwrap_err(),
            AuthorizationErrorCodeV1::InvalidFormat,
            "",
        );
        assert_authorization_error(
            parse_authorization_context_token_v1(&format!("{token}="), &fixture_policy(1_100))
                .unwrap_err(),
            AuthorizationErrorCodeV1::InvalidEncoding,
            "",
        );
    }

    #[test]
    fn binds_every_request_component_and_denies_missing_capabilities() {
        let (root, certificate, manifest, context, session_key) = chain();
        let policy = fixture_policy(1_100);
        let verified =
            verify_authorization_context_v1(&root, &manifest, &certificate, &context, &policy)
                .unwrap();
        let proof = sign_authorization_request_v2(
            verified.context_digest(),
            "rpc.v1.Documents.Get",
            Some("_INBOX.test.reply"),
            b"payload",
            1_100,
            "req_test",
            &session_key,
        )
        .unwrap();
        let other_session_key = SigningKey::from_bytes(&[7; 32]);
        let other_key_proof = sign_authorization_request_v2(
            verified.context_digest(),
            "rpc.v1.Documents.Get",
            Some("_INBOX.test.reply"),
            b"payload",
            1_100,
            "req_test",
            &other_session_key,
        )
        .unwrap();
        assert_authorization_error(
            verify_authorization_request_v2(
                &verified,
                "rpc.v1.Documents.Get",
                Some("_INBOX.test.reply"),
                b"payload",
                1_100,
                "req_test",
                &other_key_proof,
                &policy,
                &[],
                &[],
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::InvalidRequestProof,
            "/proof",
        );
        let issuer_key = SigningKey::from_bytes(&[2; 32]);
        let swapped_context = sign_authorization_context_v1(
            UnsignedAuthorizationContextV1 {
                context_id: "ctx_swapped".into(),
                ..context.unsigned.clone()
            },
            &issuer_key,
        )
        .unwrap();
        let swapped_context = verify_authorization_context_v1(
            &root,
            &manifest,
            &certificate,
            &swapped_context,
            &policy,
        )
        .unwrap();
        assert_authorization_error(
            verify_authorization_request_v2(
                &swapped_context,
                "rpc.v1.Documents.Get",
                Some("_INBOX.test.reply"),
                b"payload",
                1_100,
                "req_test",
                &proof,
                &policy,
                &[],
                &[],
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::InvalidRequestProof,
            "/proof",
        );
        for (subject, reply, payload, iat, request_id) in [
            (
                "rpc.v1.Documents.Other",
                Some("_INBOX.test.reply"),
                b"payload".as_slice(),
                1_100,
                "req_test",
            ),
            (
                "rpc.v1.Documents.Get",
                Some("_INBOX.test.other"),
                b"payload".as_slice(),
                1_100,
                "req_test",
            ),
            (
                "rpc.v1.Documents.Get",
                Some("_INBOX.test.reply"),
                b"changed".as_slice(),
                1_100,
                "req_test",
            ),
            (
                "rpc.v1.Documents.Get",
                Some("_INBOX.test.reply"),
                b"payload".as_slice(),
                1_101,
                "req_test",
            ),
            (
                "rpc.v1.Documents.Get",
                Some("_INBOX.test.reply"),
                b"payload".as_slice(),
                1_100,
                "req_other",
            ),
        ] {
            assert_authorization_error(
                verify_authorization_request_v2(
                    &verified,
                    subject,
                    reply,
                    payload,
                    iat,
                    request_id,
                    &proof,
                    &policy,
                    &[],
                    &[],
                )
                .unwrap_err(),
                AuthorizationErrorCodeV1::InvalidRequestProof,
                "/proof",
            );
        }
        assert_authorization_error(
            verify_authorization_request_v2(
                &verified,
                "rpc.v1.Documents.Get",
                Some("OTHER.reply"),
                b"payload",
                1_100,
                "req_test",
                &proof,
                &policy,
                &[],
                &[],
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::ReplySubjectMismatch,
            "/reply",
        );
        assert_authorization_error(
            verify_authorization_request_v2(
                &verified,
                "rpc.v1.Documents.Get",
                Some("_INBOX.test.reply"),
                b"payload",
                1_200,
                "req_test",
                &proof,
                &policy,
                &[],
                &[],
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::ProofIatOutOfRange,
            "/iat",
        );
        assert_authorization_error(
            verify_authorization_request_v2(
                &verified,
                "rpc.v1.Documents.Get",
                Some("_INBOX.test.reply"),
                b"payload",
                1_100,
                "req_test",
                &proof,
                &policy,
                &[],
                &["platform.write".into()],
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::CapabilityDenied,
            "/capabilities",
        );
        assert_authorization_error(
            verify_authorization_request_v2(
                &verified,
                "rpc.v1.Documents.Get",
                Some("_INBOX.test.reply"),
                b"payload",
                1_100,
                "",
                &proof,
                &policy,
                &[],
                &[],
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::InvalidFormat,
            "/request-id",
        );
        assert!(sign_authorization_request_v2(
            verified.context_digest(),
            "rpc.v1.Documents.Get",
            None,
            b"payload",
            1_100,
            "req_no_reply",
            &session_key,
        )
        .is_ok());

        let late_policy = fixture_policy(1_400);
        let late_proof = sign_authorization_request_v2(
            verified.context_digest(),
            "rpc.v1.Documents.Get",
            None,
            b"payload",
            1_400,
            "req_late",
            &session_key,
        )
        .unwrap();
        assert_authorization_error(
            verify_authorization_request_v2(
                &verified,
                "rpc.v1.Documents.Get",
                None,
                b"payload",
                1_400,
                "req_late",
                &late_proof,
                &late_policy,
                &[],
                &[],
            )
            .unwrap_err(),
            AuthorizationErrorCodeV1::ContextExpired,
            "/authorization-context/expiresAt",
        );
    }

    #[test]
    fn matches_language_neutral_conformance_vector() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../conformance/authorization-context/vectors.json");
        let fixture: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let expected = &fixture["completeChain"];
        let (root, certificate, manifest, context, session_key) = chain();
        let policy = fixture_policy(1_100);
        let verified =
            verify_authorization_context_v1(&root, &manifest, &certificate, &context, &policy)
                .unwrap();
        assert_eq!(root.key_id(), expected["rootKeyId"]);
        assert_eq!(
            root.canonical_json().unwrap(),
            expected["rootCanonicalJson"]
        );
        assert_eq!(root.digest().unwrap(), expected["rootDigest"]);
        assert_eq!(certificate.unsigned.key_id, expected["issuerKeyId"]);
        assert_eq!(
            canonicalize_json(&serde_json::to_value(&certificate.unsigned).unwrap()).unwrap(),
            expected["certificateUnsignedCanonicalJson"]
        );
        assert_eq!(
            encode_base64url(&issuer_certificate_signing_digest_v1(&certificate.unsigned).unwrap()),
            expected["certificateSigningDigest"]
        );
        assert_eq!(certificate.signature, expected["certificateSignature"]);
        assert_eq!(
            canonicalize_json(&serde_json::to_value(&certificate).unwrap()).unwrap(),
            expected["certificateCanonicalJson"]
        );
        assert_eq!(certificate.digest().unwrap(), expected["certificateDigest"]);
        assert_eq!(
            canonicalize_json(&serde_json::to_value(&manifest.manifest().unsigned).unwrap())
                .unwrap(),
            expected["manifestUnsignedCanonicalJson"]
        );
        assert_eq!(
            encode_base64url(
                &issuer_manifest_signing_digest_v1(&manifest.manifest().unsigned).unwrap()
            ),
            expected["manifestSigningDigest"]
        );
        assert_eq!(manifest.manifest().signature, expected["manifestSignature"]);
        assert_eq!(
            canonicalize_json(&serde_json::to_value(manifest.manifest()).unwrap()).unwrap(),
            expected["manifestCanonicalJson"]
        );
        assert_eq!(
            manifest.manifest().digest().unwrap(),
            expected["manifestDigest"]
        );
        assert_eq!(
            canonicalize_json(&serde_json::to_value(&context.unsigned).unwrap()).unwrap(),
            expected["contextUnsignedCanonicalJson"]
        );
        assert_eq!(
            encode_base64url(&authorization_context_signing_digest_v1(&context.unsigned).unwrap()),
            expected["contextSigningDigest"]
        );
        assert_eq!(context.signature, expected["contextSignature"]);
        assert_eq!(
            canonicalize_json(&serde_json::to_value(&context).unwrap()).unwrap(),
            expected["contextCanonicalJson"]
        );
        assert_eq!(verified.context_digest(), expected["contextDigest"]);
        assert_eq!(
            encode_authorization_context_token_v1(&context).unwrap(),
            expected["contextToken"]
        );
        let proof = sign_authorization_request_v2(
            verified.context_digest(),
            "rpc.v1.Documents.Get",
            Some("_INBOX.test.reply"),
            br#"{"id":"doc-1"}"#,
            1_100,
            "req_test",
            &session_key,
        )
        .unwrap();
        let input = build_authorization_request_proof_input_v2(
            &decode_base64url::<32>(
                verified.context_digest(),
                &["authorization-context"],
                AuthorizationErrorCodeV1::InvalidEncoding,
            )
            .unwrap(),
            "rpc.v1.Documents.Get",
            Some("_INBOX.test.reply"),
            br#"{"id":"doc-1"}"#,
            1_100,
            "req_test",
        )
        .unwrap();
        let hex = input
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(hex, expected["requestProofInputHex"]);
        assert_eq!(
            encode_base64url(input.digest()),
            expected["requestProofDigest"]
        );
        assert_eq!(proof.as_str(), expected["requestProof"]);
        let cases = fixture["cases"].as_array().unwrap();
        let mut names = BTreeSet::new();
        for case in cases {
            assert!(
                names.insert(case["name"].as_str().unwrap()),
                "duplicate conformance case {}",
                case["name"]
            );
            assert_fixture_case(&fixture, case);
        }
        assert!(cases.iter().any(|case| case["expected"]["valid"] == true));
        assert!(cases.iter().any(|case| case["expected"]["valid"] == false));
    }
}
