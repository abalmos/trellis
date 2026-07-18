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
//!
//! # API artifacts
//!
//! [`ApiArtifactV1`] represents one versioned public Trellis API. Its identity is
//! exactly `lineage@vN`; each RPC, operation, event, or feed also has its own
//! surface version used to derive subjects such as `rpc.v1.Documents.Get`.
//! Parsing validates embedded Draft 2020-12 JSON Schemas against the Trellis
//! profile and resolves all artifact-local references.
//!
//! [`ApiArtifactV1::normalized_value`] preserves supported runtime fields,
//! including human documentation. [`ApiArtifactV1::digest`] instead hashes a
//! semantic projection: schemas, exports, errors, surfaces, capabilities, and
//! other machine behavior affect identity; display names, descriptions, docs,
//! consent wording, and surface docs do not.
//!
//! Event subject parameters retain authored JSON Pointer order. The derived
//! subscription pattern appends one `*` token per parameter in that same order.
//!
//! ```
//! use serde_json::json;
//! use trellis_protocol::{parse_api_v1, API_SCHEMA_V1_JSON};
//!
//! let raw = json!({
//!     "format": "trellis.api.v1",
//!     "id": "documents@v1",
//!     "displayName": "Documents",
//!     "description": "Document APIs.",
//!     "schemas": { "Input": true, "Output": true },
//!     "rpc": {
//!         "Documents.Get": {
//!             "version": "v1",
//!             "input": { "schema": "Input" },
//!             "output": { "schema": "Output" }
//!         }
//!     }
//! });
//! assert!(API_SCHEMA_V1_JSON.contains("trellis.api.v1"));
//! let api = parse_api_v1(&raw)?;
//! assert_eq!(api.id(), "documents@v1");
//! assert_eq!(api.normalized_value()?["displayName"], "Documents");
//! assert_eq!(api.digest()?.len(), 43);
//! assert_eq!(api.derived_subjects()?.rpc["Documents.Get"], "rpc.v1.Documents.Get");
//! # Ok::<(), trellis_protocol::ProtocolError>(())
//! ```

mod api;
mod canonical;
mod error;
mod identifiers;
mod permissions;
mod schema_profile;
mod subjects;

pub use api::{parse_api_v1, ApiArtifactV1, API_FORMAT_V1, API_SCHEMA_V1_JSON};
pub use canonical::{canonicalize_json, digest_json, sha256_base64url};
pub use error::ProtocolError;
pub use permissions::{
    ApiSurfaceKindV1, CapabilityDefinitionV1, ConsentMetadataV1, GrantSetV1,
    ParticipantResourceKindV1, PermissionActionV1, PermissionAtomV1, PermissionTargetV1,
    GRANT_SET_FORMAT_V1,
};
pub use subjects::{
    derive_event_subject, derive_event_wildcard_subject, derive_feed_subject,
    derive_operation_subject, derive_rpc_subject, DerivedApiSubjectsV1, DerivedEventSubjectsV1,
};
