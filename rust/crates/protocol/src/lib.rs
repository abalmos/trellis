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
//! # Participant artifacts
//!
//! [`ParticipantArtifactV1`] describes a service, app, device, or agent. An
//! `implements` entry says the participant provides a pinned API; `uses.required`
//! and `uses.optional` select pinned API surfaces needed for mandatory and
//! optional behavior. Participants may additionally declare local schemas,
//! private state, Jobs queues, durable event consumers, KV buckets, object
//! stores, and provider operation-transfer mappings.
//!
//! Parsing validates the artifact itself, including aliases, local schema
//! references, resource declarations, and normalized ordering. It does not prove
//! that a pinned API exists or that selected surfaces occur in it; contextual
//! cross-artifact resolution is a separate, later boundary.
//!
//! Human display names, descriptions, docs, and resource-purpose text remain in
//! normalized values but do not affect participant identity. Kind, schemas, API
//! pins and selections, local resources, consumers, state, queues, and transfer
//! mappings do affect the digest.
//!
//! ```
//! use serde_json::json;
//! use trellis_protocol::{parse_participant_v1, ParticipantKindV1};
//!
//! let raw = json!({
//!     "format": "trellis.participant.v1",
//!     "id": "documents-worker",
//!     "displayName": "Documents Worker",
//!     "description": "Processes documents.",
//!     "kind": "service",
//!     "uses": {
//!         "required": { "billing": {
//!             "api": "billing@v1",
//!             "apiDigest": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
//!             "operations": { "control": { "Billing.Refund": ["approve"] } }
//!         }},
//!         "optional": { "health": {
//!             "api": "health@v1",
//!             "apiDigest": "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE",
//!             "feeds": { "subscribe": ["Health.Watch"] }
//!         }}
//!     },
//!     "resources": { "store": { "uploads": { "purpose": "Incoming files." } } }
//! });
//! let participant = parse_participant_v1(&raw)?;
//! assert_eq!(participant.kind(), ParticipantKindV1::Service);
//! assert_eq!(participant.normalized_value()?["id"], "documents-worker");
//! assert_eq!(participant.digest()?.len(), 43);
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
//! They are not a cryptographic signature payload: a signature must cover
//! the complete raw canonical object, excluding only the signature member, so
//! unknown fields cannot be dropped. The [`authorization`] module applies that
//! rule to its deliberately strict signed security objects.
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
//! use trellis_protocol::{lint_api_v1_authoring, parse_api_v1, API_AUTHORING_SCHEMA_V1_JSON};
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
//! assert!(API_AUTHORING_SCHEMA_V1_JSON.contains("trellis.api.v1"));
//! lint_api_v1_authoring(&raw)?;
//! let api = parse_api_v1(&raw)?;
//! assert_eq!(api.id(), "documents@v1");
//! assert_eq!(api.normalized_value()?["displayName"], "Documents");
//! assert_eq!(api.digest()?.len(), 43);
//! assert_eq!(api.derived_subjects()?.rpc["Documents.Get"], "rpc.v1.Documents.Get");
//! # Ok::<(), trellis_protocol::ProtocolError>(())
//! ```
//!
//! Runtime parsers ignore unknown object members and project only semantics this
//! version understands. Strict authoring lint reports those members before
//! tolerant parsing. New required semantics need a negotiated feature or a new
//! format rather than an ignored critical extension member.
//!
//! # Compatibility
//!
//! [`compare_api_replacement_v1`] is directional: it asks whether clients
//! accepted against an old provider remain supported by a replacement provider.
//! Additive optional object fields are compatible; removing a surface is not.
//!
//! ```
//! use serde_json::json;
//! use trellis_protocol::{compare_api_replacement_v1, parse_api_v1};
//!
//! let old = parse_api_v1(&json!({
//!     "format": "trellis.api.v1", "id": "documents@v1",
//!     "displayName": "Documents", "description": "Documents."
//! }))?;
//! let additive = parse_api_v1(&json!({
//!     "format": "trellis.api.v1", "id": "documents@v1",
//!     "displayName": "Documents", "description": "Documents.",
//!     "schemas": { "Input": true, "Output": true },
//!     "rpc": { "Documents.Get": {
//!         "version": "v1", "input": { "schema": "Input" },
//!         "output": { "schema": "Output" }
//!     }}
//! }))?;
//! assert!(compare_api_replacement_v1(&old, &additive)?.compatible);
//!
//! let wrong_lineage = parse_api_v1(&json!({
//!     "format": "trellis.api.v1", "id": "archive@v1",
//!     "displayName": "Archive", "description": "Archive."
//! }))?;
//! assert!(!compare_api_replacement_v1(&old, &wrong_lineage)?.compatible);
//! # Ok::<(), trellis_protocol::ProtocolError>(())
//! ```
//!
//! # Contextual resolution
//!
//! [`resolve_participant_v1`] proves a participant's exact API pins and selected
//! surfaces, then derives separate required and optional machine needs and
//! owner-reviewable authority evidence.
//!
//! ```
//! use std::collections::BTreeMap;
//! use serde_json::json;
//! use trellis_protocol::{parse_api_v1, parse_participant_v1, resolve_participant_v1};
//!
//! let documents = parse_api_v1(&json!({
//!     "format": "trellis.api.v1", "id": "documents@v1",
//!     "displayName": "Documents", "description": "Documents."
//! }))?;
//! let billing = parse_api_v1(&json!({
//!     "format": "trellis.api.v1", "id": "billing@v1",
//!     "displayName": "Billing", "description": "Billing.",
//!     "schemas": { "Input": true, "Output": true },
//!     "rpc": {
//!         "Billing.Get": { "version": "v1", "input": { "schema": "Input" }, "output": { "schema": "Output" } },
//!         "Billing.Update": { "version": "v1", "input": { "schema": "Input" }, "output": { "schema": "Output" } }
//!     },
//!     "capabilities": { "billing.read": { "allows": [{
//!         "target": { "kind": "apiSurface", "api": "billing@v1", "surface": "rpc", "name": "Billing.Get" },
//!         "action": "call"
//!     }] } },
//!     "consent": { "billing.read": {
//!         "title": "Read billing", "description": "Reads billing data.",
//!         "consequence": "Billing data is shared."
//!     } }
//! }))?;
//! let health = parse_api_v1(&json!({
//!     "format": "trellis.api.v1", "id": "health@v1",
//!     "displayName": "Health", "description": "Health.",
//!     "schemas": { "Input": true, "Event": true },
//!     "feeds": { "Health.Watch": {
//!         "version": "v1", "input": { "schema": "Input" },
//!         "event": { "schema": "Event" }
//!     } }
//! }))?;
//!
//! let participant = parse_participant_v1(&json!({
//!     "format": "trellis.participant.v1", "id": "documents-worker",
//!     "displayName": "Documents Worker", "description": "Processes documents.",
//!     "kind": "service",
//!     "implements": { "documents": { "api": "documents@v1", "apiDigest": documents.digest()? } },
//!     "uses": {
//!         "required": { "billing": {
//!             "api": "billing@v1", "apiDigest": billing.digest()?,
//!             "rpc": { "call": ["Billing.Update", "Billing.Get"] }
//!         } },
//!         "optional": { "health": {
//!             "api": "health@v1", "apiDigest": health.digest()?,
//!             "feeds": { "subscribe": ["Health.Watch"] }
//!         } }
//!     },
//!     "resources": { "store": { "uploads": { "purpose": "Incoming files." } } }
//! }))?;
//!
//! let mut apis = BTreeMap::new();
//! apis.insert(documents.id().to_owned(), documents);
//! apis.insert(billing.id().to_owned(), billing);
//! apis.insert(health.id().to_owned(), health);
//! let resolved = resolve_participant_v1(&participant, &apis)?;
//!
//! assert_eq!(resolved.needs().required().grant_set().permissions().len(), 5);
//! assert_eq!(resolved.needs().optional().apis().len(), 1);
//! assert_eq!(resolved.proposal().required().capabilities().len(), 1);
//! assert_eq!(resolved.proposal().required().uncovered_permissions().len(), 4);
//! assert_eq!(resolved.needs().provided_apis().len(), 1);
//! assert_eq!(resolved.needs().digest()?.len(), 43);
//! assert_eq!(resolved.proposal().fingerprint()?.len(), 43);
//! # Ok::<(), trellis_protocol::ProtocolError>(())
//! ```

mod api;
pub mod authorization;
mod canonical;
mod error;
mod identifiers;
mod participant;
mod permissions;
mod resolution;
mod schema_profile;
mod session_proof;
mod subjects;

pub use api::{
    compare_api_replacement_v1, lint_api_v1_authoring, parse_api_v1, ApiArtifactV1,
    ApiCompatibilityIssueCodeV1, ApiCompatibilityIssueV1, ApiCompatibilityReportV1,
    StateDefinitionV1, StateKindV1, API_AUTHORING_SCHEMA_V1_JSON, API_FORMAT_V1,
};
pub use authorization::{
    authorization_context_refresh_at_v1, authorization_context_signing_digest_v1,
    build_authorization_event_proof_input_v2, build_authorization_request_proof_input_v2,
    issuer_manifest_signing_digest_v1, parse_authorization_context_v1, parse_issuer_manifest_v1,
    sign_authorization_context_v1, sign_authorization_event_v2, sign_authorization_request_v2,
    sign_issuer_manifest_v1, verify_authorization_context_v1, verify_authorization_event_v2,
    verify_authorization_request_v2, verify_issuer_manifest_v1, AuthorizationAuthorityKindV1,
    AuthorizationAuthorityRefV1, AuthorizationEventProofInputV2, AuthorizationEventProofV2,
    AuthorizationEventPublisherV2, AuthorizationIssuerManifestEntryV1, AuthorizationParticipantV1,
    AuthorizationPrincipalKindV1, AuthorizationPrincipalV1, AuthorizationRequestProofInputV2,
    AuthorizationRequestProofV2, AuthorizationTrustRootV1, AuthorizationVerificationPolicyV1,
    SignedAuthorizationContextV1, SignedAuthorizationIssuerManifestV1,
    UnsignedAuthorizationContextV1, UnsignedAuthorizationIssuerManifestV1,
    VerifiedAuthorizationContextV1, VerifiedAuthorizationEventV2,
    VerifiedAuthorizationIssuerManifestV1, VerifiedAuthorizationRequestV2,
    AUTHORIZATION_CONTEXT_FORMAT_V1, AUTHORIZATION_EVENT_PROOF_DOMAIN_V2,
    AUTHORIZATION_ISSUER_MANIFEST_FORMAT_V1, AUTHORIZATION_REQUEST_PROOF_DOMAIN_V2,
    AUTHORIZATION_TRUST_ROOT_FORMAT_V1,
};
pub use canonical::{canonicalize_json, digest_json, sha256_base64url};
pub use error::{
    AuthorizationErrorCodeV1, ProtocolError, ResolutionErrorCodeV1, SessionProofErrorCodeV1,
};
pub use participant::{
    lint_participant_v1_authoring, parse_participant_v1, ParticipantArtifactV1, ParticipantKindV1,
    PARTICIPANT_AUTHORING_SCHEMA_V1_JSON, PARTICIPANT_FORMAT_V1,
};
pub use permissions::{
    ApiSurfaceKindV1, CapabilityDefinitionV1, ConsentMetadataV1, GrantSetV1,
    ParticipantResourceKindV1, PermissionActionV1, PermissionAtomV1, PermissionTargetV1,
    GRANT_SET_FORMAT_V1,
};
pub use resolution::{
    resolve_participant_v1, AuthorityCapabilityEvidenceV1, AuthorityProposalSectionV1,
    AuthorityProposalV1, ParticipantNeedsSectionV1, ParticipantNeedsV1, ParticipantResourceNeedsV1,
    ProvidedApiNeedV1, ResolvedImplementedApiV1, ResolvedParticipantV1, ResolvedProvidedApiV1,
    ResolvedProvidedOperationV1, ResolvedUsedApiV1, AUTHORITY_PROPOSAL_FORMAT_V1,
    PARTICIPANT_NEEDS_FORMAT_V1,
};
pub use session_proof::{
    parse_session_proof_v1, session_proof_request_digest_v1, session_proof_signing_digest_v1,
    sign_session_proof_v1, verify_session_proof_v1, SessionProofInputV1, SessionProofPolicyV1,
    SessionProofPurposeV1, SessionProofV1, SESSION_PROOF_FORMAT_V1,
};
pub use subjects::{
    derive_event_subject, derive_event_wildcard_subject, derive_feed_subject,
    derive_operation_subject, derive_rpc_subject, DerivedApiSubjectsV1, DerivedEventSubjectsV1,
};
