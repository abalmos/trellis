//! File-backed authorization trust loading and active issuer verification.

use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use ed25519_dalek::SigningKey;
use sha2::{Digest as _, Sha256};
use thiserror::Error;
use trellis_protocol::{
    canonicalize_json, parse_issuer_manifest_v1, verify_issuer_manifest_v1,
    AuthorizationTrustRootV1, AuthorizationVerificationPolicyV1,
    SignedAuthorizationIssuerManifestV1, VerifiedAuthorizationIssuerManifestV1,
};

use crate::config::AuthorizationConfig;

const MAX_TRUST_FILE_BYTES: u64 = 1_048_576;

/// Fully verified authorization trust material and active online issuer.
pub(crate) struct VerifiedTrustMaterial {
    pub(crate) root: AuthorizationTrustRootV1,
    pub(crate) manifest: SignedAuthorizationIssuerManifestV1,
    pub(crate) verified_manifest: VerifiedAuthorizationIssuerManifestV1,
    pub(crate) issuer_signing_key: SigningKey,
    pub(crate) policy: AuthorizationVerificationPolicyV1,
}

/// Public trust material sufficient for local request and event verification.
pub(crate) struct VerifiedValidatorTrustMaterial {
    pub(crate) root: AuthorizationTrustRootV1,
    pub(crate) manifest: SignedAuthorizationIssuerManifestV1,
    pub(crate) verified_manifest: VerifiedAuthorizationIssuerManifestV1,
    pub(crate) policy: AuthorizationVerificationPolicyV1,
}

impl std::fmt::Debug for VerifiedTrustMaterial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VerifiedTrustMaterial")
            .field("rootKeyId", &self.root.key_id())
            .field("manifestGeneration", &self.verified_manifest.generation())
            .field(
                "issuerKeyId",
                &base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .encode(self.issuer_signing_key.verifying_key().to_bytes()),
            )
            .finish_non_exhaustive()
    }
}

impl VerifiedTrustMaterial {
    /// Load and verify configured trust material at one injected wall-clock time.
    pub(crate) fn load(
        config: &AuthorizationConfig,
        now_unix_seconds: i64,
    ) -> Result<Self, TrustMaterialError> {
        let VerifiedValidatorTrustMaterial {
            root,
            manifest,
            verified_manifest,
            policy,
        } = VerifiedValidatorTrustMaterial::load(config, now_unix_seconds)?;

        let mut seed_bytes = read_signing_seed(&config.issuer_signing_seed_file)?;
        let issuer_signing_key = SigningKey::from_bytes(&seed_bytes);
        seed_bytes.fill(0);
        let issuer_public_key = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(issuer_signing_key.verifying_key().to_bytes());
        let issuer_key_id = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
            Sha256::digest(issuer_signing_key.verifying_key().to_bytes()),
        );
        let matching_entries = manifest
            .unsigned
            .issuers
            .iter()
            .filter(|issuer| {
                issuer.key_id == issuer_key_id && issuer.public_key == issuer_public_key
            })
            .count();
        if matching_entries != 1 {
            if manifest
                .unsigned
                .issuers
                .iter()
                .any(|issuer| issuer.key_id == issuer_key_id)
            {
                return Err(TrustMaterialError::IssuerSeedMismatch);
            }
            return Err(TrustMaterialError::IssuerNotListed);
        }
        Ok(Self {
            root,
            manifest,
            verified_manifest,
            issuer_signing_key,
            policy,
        })
    }
}

impl VerifiedValidatorTrustMaterial {
    /// Load and verify public trust material without opening the issuer seed.
    pub(crate) fn load(
        config: &AuthorizationConfig,
        now_unix_seconds: i64,
    ) -> Result<Self, TrustMaterialError> {
        let policy_at = |now| {
            AuthorizationVerificationPolicyV1::new(
                now,
                u32::try_from(config.allowed_clock_skew_seconds)
                    .map_err(|_| TrustMaterialError::InvalidPolicy)?,
                u32::try_from(config.context_lifetime_seconds)
                    .map_err(|_| TrustMaterialError::InvalidPolicy)?,
                config.maximum_context_bytes,
                config.maximum_permissions,
                config.maximum_capabilities,
                1,
            )
            .map_err(|_| TrustMaterialError::InvalidPolicy)
        };
        let policy = policy_at(now_unix_seconds)?;

        let root_value = read_canonical_json(&config.trust_root_file, "trust root")?;
        let root = AuthorizationTrustRootV1::parse(&root_value)
            .map_err(|_| TrustMaterialError::InvalidTrustRoot)?;
        let manifest_value = read_canonical_json(&config.issuer_manifest_file, "issuer manifest")?;
        let manifest = parse_issuer_manifest_v1(&manifest_value)
            .map_err(|_| TrustMaterialError::InvalidManifest)?;
        let verified_manifest = verify_issuer_manifest_v1(&root, &manifest, &policy)
            .map_err(|_| TrustMaterialError::InvalidManifest)?;
        if manifest.unsigned.not_before > now_unix_seconds {
            return Err(TrustMaterialError::InvalidManifest);
        }

        Ok(Self {
            root,
            manifest,
            verified_manifest,
            policy,
        })
    }
}

/// Safe file-backed trust loading failure.
#[derive(Debug, Error)]
pub(crate) enum TrustMaterialError {
    #[error("authorization trust file could not be read: {path}")]
    Read { path: PathBuf },
    #[error("authorization trust file is too large: {path}")]
    FileTooLarge { path: PathBuf },
    #[error("authorization trust file is not canonical JSON: {path}")]
    NoncanonicalJson { path: PathBuf },
    #[error("authorization trust root is invalid")]
    InvalidTrustRoot,
    #[error("authorization issuer manifest is invalid")]
    InvalidManifest,
    #[error("authorization issuer seed file is invalid")]
    InvalidIssuerSeed,
    #[error("authorization issuer seed is not listed in the manifest")]
    IssuerNotListed,
    #[error("authorization issuer seed does not match its manifest entry")]
    IssuerSeedMismatch,
    #[error("authorization verification policy is invalid")]
    InvalidPolicy,
}

fn read_canonical_json(path: &Path, label: &str) -> Result<serde_json::Value, TrustMaterialError> {
    let metadata = fs::metadata(path).map_err(|_| TrustMaterialError::Read {
        path: path.to_path_buf(),
    })?;
    if metadata.len() > MAX_TRUST_FILE_BYTES {
        return Err(TrustMaterialError::FileTooLarge {
            path: path.to_path_buf(),
        });
    }
    let text = fs::read_to_string(path).map_err(|_| TrustMaterialError::Read {
        path: path.to_path_buf(),
    })?;
    let value = serde_json::from_str::<serde_json::Value>(&text).map_err(|_| {
        TrustMaterialError::NoncanonicalJson {
            path: path.to_path_buf(),
        }
    })?;
    let canonical =
        canonicalize_json(&value).map_err(|_| TrustMaterialError::NoncanonicalJson {
            path: path.to_path_buf(),
        })?;
    if text.trim() != canonical {
        return Err(TrustMaterialError::NoncanonicalJson {
            path: path.to_path_buf(),
        });
    }
    let _ = label;
    Ok(value)
}

fn read_signing_seed(path: &Path) -> Result<[u8; 32], TrustMaterialError> {
    warn_if_secret_permissions_are_open(path);
    let text = fs::read_to_string(path).map_err(|_| TrustMaterialError::Read {
        path: path.to_path_buf(),
    })?;
    let encoded = text.trim();
    let mut decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| TrustMaterialError::InvalidIssuerSeed)?;
    if base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&decoded) != encoded {
        decoded.fill(0);
        return Err(TrustMaterialError::InvalidIssuerSeed);
    }
    let result = <[u8; 32]>::try_from(decoded.as_slice()).map_err(|_| {
        decoded.fill(0);
        TrustMaterialError::InvalidIssuerSeed
    })?;
    decoded.fill(0);
    Ok(result)
}

#[cfg(unix)]
fn warn_if_secret_permissions_are_open(path: &Path) {
    use std::os::unix::fs::PermissionsExt as _;

    if let Ok(metadata) = fs::metadata(path) {
        if metadata.permissions().mode() & 0o077 != 0 {
            tracing::warn!(path = %path.display(), "authorization issuer seed permissions allow group or other access");
        }
    }
}

#[cfg(not(unix))]
fn warn_if_secret_permissions_are_open(_path: &Path) {}
