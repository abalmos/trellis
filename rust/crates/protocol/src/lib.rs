//! Canonical protocol primitives shared by Trellis runtimes and language bindings.
//!
//! This crate owns language-neutral values whose identity must agree across
//! implementations. JSON identity uses RFC 8785 canonicalization rather than
//! ordinary [`serde_json`] serialization, and content digests are SHA-256 bytes
//! encoded as unpadded base64url. Set-like protocol arrays are sorted by UTF-16
//! code units, matching JavaScript string ordering even when Unicode scalar-value
//! order differs.
//!
//! Permissions are exact, machine-enforceable [`PermissionAtomV1`] values.
//! [`CapabilityDefinitionV1`] groups atoms for authoring and explanation, while
//! [`GrantSetV1`] is the normalized enforceable set: construction sorts atoms and
//! removes duplicates. [`ConsentMetadataV1`] is presentation text only and never
//! grants authority.
//!
//! # Examples
//!
//! Canonicalize and digest ordinary JSON:
//!
//! ```
//! use serde_json::json;
//! use trellis_protocol::{canonicalize_json, digest_json};
//!
//! let value = json!({"z": 1, "a": 2});
//! assert_eq!(canonicalize_json(&value)?, r#"{"a":2,"z":1}"#);
//! assert_eq!(digest_json(&value)?.len(), 43);
//! # Ok::<(), trellis_protocol::ProtocolError>(())
//! ```
//!
//! Construct one valid RPC permission and a content-addressed grant set:
//!
//! ```
//! use trellis_protocol::{
//!     ApiSurfaceKindV1, GrantSetV1, PermissionActionV1, PermissionAtomV1,
//!     PermissionTargetV1,
//! };
//!
//! let target = PermissionTargetV1::api_surface(
//!     "documents@v1",
//!     ApiSurfaceKindV1::Rpc,
//!     "Documents.Get",
//! )?;
//! let atom = PermissionAtomV1::new(target, PermissionActionV1::Call)?;
//! let grants = GrantSetV1::new(vec![atom.clone(), atom]);
//!
//! assert_eq!(grants.permissions().len(), 1);
//! assert!(grants.canonical_json()?.starts_with('{'));
//! assert_eq!(grants.digest()?.len(), 43);
//! # Ok::<(), trellis_protocol::ProtocolError>(())
//! ```
//!
//! Semantic normalized projections are suitable for identity and comparison.
//! They are not a future cryptographic signature payload: a signature must cover
//! the complete raw canonical object, excluding only the signature member, so
//! unknown fields cannot be dropped. Signature support is not implemented here.

mod canonical;
mod error;
mod permissions;

pub use canonical::{canonicalize_json, digest_json, sha256_base64url};
pub use error::ProtocolError;
pub use permissions::{
    ApiSurfaceKindV1, CapabilityDefinitionV1, ConsentMetadataV1, GrantSetV1,
    ParticipantResourceKindV1, PermissionActionV1, PermissionAtomV1, PermissionTargetV1,
    GRANT_SET_FORMAT_V1,
};
