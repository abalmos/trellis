//! Canonical protocol primitives shared by Trellis runtimes and language bindings.
//!
//! This crate owns language-neutral values whose identity must agree across
//! implementations. JSON identity uses RFC 8785 canonicalization rather than
//! ordinary [`serde_json`] serialization, and content digests are SHA-256 bytes
//! encoded as unpadded base64url. Set-like protocol arrays are sorted by UTF-16
//! code units, matching JavaScript string ordering even when Unicode scalar-value
//! order differs.
//!
//! Permissions are exact, machine-enforceable [`PermissionAtom`] values.
//! [`CapabilityDefinition`] groups atoms for authoring and explanation, while
//! [`GrantSet`] is the normalized enforceable set: construction sorts atoms and
//! removes duplicates. [`ConsentMetadata`] is presentation text only and never
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
//! [`ParticipantArtifact`] describes a service, app, device, or agent. An
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
//! use trellis_protocol::{parse_participant, ParticipantKind};
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
//! let participant = parse_participant(&raw)?;
//! assert_eq!(participant.kind(), ParticipantKind::Service);
//! assert_eq!(participant.normalized_value()?["id"], "documents-worker");
//! assert_eq!(participant.digest()?.len(), 43);
//! # Ok::<(), trellis_protocol::ProtocolError>(())
//! ```
//!
//! Construct one valid RPC permission and a content-addressed grant set:
//!
//! ```
//! use trellis_protocol::{
//!     ApiSurfaceKind, GrantSet, PermissionAction, PermissionAtom,
//!     PermissionTarget,
//! };
//!
//! let target = PermissionTarget::api_surface(
//!     "documents@v1",
//!     ApiSurfaceKind::Rpc,
//!     "Documents.Get",
//! )?;
//! let atom = PermissionAtom::new(target, PermissionAction::Call)?;
//! let grants = GrantSet::new(vec![atom.clone(), atom]);
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
//! [`ApiArtifact`] represents one versioned public Trellis API. Its identity is
//! exactly `lineage@vN`; each RPC, operation, event, or feed also has its own
//! surface version used to derive subjects such as `rpc.v1.Documents.Get`.
//! Parsing validates embedded Draft 2020-12 JSON Schemas against the Trellis
//! profile and resolves all artifact-local references.
//!
//! [`ApiArtifact::normalized_value`] preserves supported runtime fields,
//! including human documentation. [`ApiArtifact::digest`] instead hashes a
//! semantic projection: schemas, exports, errors, surfaces, capabilities, and
//! other machine behavior affect identity; display names, descriptions, docs,
//! consent wording, and surface docs do not.
//!
//! Event subject parameters retain authored JSON Pointer order. The derived
//! subscription pattern appends one `*` token per parameter in that same order.
//!
//! ```
//! use serde_json::json;
//! use trellis_protocol::{lint_api_authoring, parse_api, API_AUTHORING_SCHEMA_V1_JSON};
//!
//! let raw = json!({
//!     "format": "trellis.api.v1",
//!     "id": "documents@v1",
//!     "version": "1.0.0",
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
//! lint_api_authoring(&raw)?;
//! let api = parse_api(&raw)?;
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
//! [`compare_api_replacement`] is directional: it asks whether clients
//! accepted against an old provider remain supported by a replacement provider.
//! Additive optional object fields are compatible; removing a surface is not.
//!
//! ```
//! use serde_json::json;
//! use trellis_protocol::{compare_api_replacement, parse_api};
//!
//! let old = parse_api(&json!({
//!     "format": "trellis.api.v1", "id": "documents@v1", "version": "1.0.0",
//!     "displayName": "Documents", "description": "Documents."
//! }))?;
//! let additive = parse_api(&json!({
//!     "format": "trellis.api.v1", "id": "documents@v1", "version": "1.1.0",
//!     "displayName": "Documents", "description": "Documents.",
//!     "schemas": { "Input": true, "Output": true },
//!     "rpc": { "Documents.Get": {
//!         "version": "v1", "input": { "schema": "Input" },
//!         "output": { "schema": "Output" }
//!     }}
//! }))?;
//! assert!(compare_api_replacement(&old, &additive)?.compatible);
//!
//! let wrong_lineage = parse_api(&json!({
//!     "format": "trellis.api.v1", "id": "archive@v1", "version": "1.0.0",
//!     "displayName": "Archive", "description": "Archive."
//! }))?;
//! assert!(!compare_api_replacement(&old, &wrong_lineage)?.compatible);
//! # Ok::<(), trellis_protocol::ProtocolError>(())
//! ```
//!
//! # Contextual resolution
//!
//! [`resolve_participant`] proves a participant's exact API pins and selected
//! surfaces, then derives separate required and optional machine needs and
//! owner-reviewable authority evidence.
//!
//! ```
//! use std::collections::BTreeMap;
//! use serde_json::json;
//! use trellis_protocol::{parse_api, parse_participant, resolve_participant};
//!
//! let documents = parse_api(&json!({
//!     "format": "trellis.api.v1", "id": "documents@v1", "version": "1.0.0",
//!     "displayName": "Documents", "description": "Documents."
//! }))?;
//! let billing = parse_api(&json!({
//!     "format": "trellis.api.v1", "id": "billing@v1", "version": "1.0.0",
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
//! let health = parse_api(&json!({
//!     "format": "trellis.api.v1", "id": "health@v1", "version": "1.0.0",
//!     "displayName": "Health", "description": "Health.",
//!     "schemas": { "Input": true, "Event": true },
//!     "feeds": { "Health.Watch": {
//!         "version": "v1", "input": { "schema": "Input" },
//!         "event": { "schema": "Event" }
//!     } }
//! }))?;
//!
//! let participant = parse_participant(&json!({
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
//! let resolved = resolve_participant(&participant, &apis)?;
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
    compare_api_replacement, lint_api_authoring, parse_api, ApiArtifact, ApiCompatibilityIssue,
    ApiCompatibilityIssueCode, ApiCompatibilityReport, StateDefinition, StateKind,
    API_AUTHORING_SCHEMA_V1_JSON, API_FORMAT_V1,
};
pub use authorization::{
    authorization_context_refresh_at, authorization_context_signing_digest,
    build_authorization_event_proof_input, build_authorization_request_proof_input,
    issuer_manifest_signing_digest, parse_authorization_context, parse_issuer_manifest,
    sign_authorization_context, sign_authorization_event, sign_authorization_request,
    sign_issuer_manifest, verify_authorization_context, verify_authorization_event,
    verify_authorization_request, verify_issuer_manifest, AuthorizationAuthorityKind,
    AuthorizationAuthorityRef, AuthorizationEventProof, AuthorizationEventProofInput,
    AuthorizationEventPublisher, AuthorizationEventVerificationInput,
    AuthorizationIssuerManifestEntry, AuthorizationParticipant, AuthorizationPrincipal,
    AuthorizationPrincipalKind, AuthorizationRequestProof, AuthorizationRequestProofInput,
    AuthorizationRequestVerificationInput, AuthorizationTrustRoot, AuthorizationVerificationPolicy,
    SignedAuthorizationContext, SignedAuthorizationIssuerManifest, UnsignedAuthorizationContext,
    UnsignedAuthorizationIssuerManifest, VerifiedAuthorizationContext,
    VerifiedAuthorizationEventProof, VerifiedAuthorizationIssuerManifest,
    VerifiedAuthorizationRequestProof, AUTHORIZATION_CONTEXT_FORMAT_V1,
    AUTHORIZATION_EVENT_PROOF_DOMAIN_V1, AUTHORIZATION_ISSUER_MANIFEST_FORMAT_V1,
    AUTHORIZATION_REQUEST_PROOF_DOMAIN_V1, AUTHORIZATION_TRUST_ROOT_FORMAT_V1,
};
pub use canonical::{canonicalize_json, digest_json, sha256_base64url};
pub use error::{
    AuthorizationErrorCode, ProtocolError, ResolutionErrorCode, SessionProofErrorCode,
};
pub use identifiers::validate_api_id;
pub use participant::{
    lint_participant_authoring, parse_participant, ParticipantArtifact, ParticipantKind,
    PARTICIPANT_AUTHORING_SCHEMA_V1_JSON, PARTICIPANT_FORMAT_V1,
};
pub use permissions::{
    ApiSurfaceKind, CapabilityDefinition, ConsentMetadata, GrantSet, ParticipantResourceKind,
    PermissionAction, PermissionAtom, PermissionTarget, GRANT_SET_FORMAT_V1,
};
pub use resolution::{
    resolve_participant, AuthorityCapabilityEvidence, AuthorityProposal, AuthorityProposalSection,
    ParticipantNeeds, ParticipantNeedsSection, ParticipantResourceNeeds, ProvidedApiNeed,
    ResolvedImplementedApi, ResolvedParticipant, ResolvedProvidedApi, ResolvedProvidedOperation,
    ResolvedUsedApi, AUTHORITY_PROPOSAL_FORMAT_V1, PARTICIPANT_NEEDS_FORMAT_V1,
};
pub use session_proof::{
    parse_session_proof, session_proof_request_digest, session_proof_signing_digest,
    sign_session_proof, verify_session_proof, AuthorizationContextRefreshSessionProofInput,
    DeviceBootstrapSessionProofInput, ServiceBootstrapSessionProofInput, SessionProof,
    SessionProofInput, SessionProofPolicy, SessionProofPurpose, UserAuthBindSessionProofInput,
    UserAuthRequestSessionProofInput, SESSION_PROOF_FORMAT_V1,
};
pub use subjects::{
    derive_event_subject, derive_event_wildcard_subject, derive_feed_subject,
    derive_operation_subject, derive_rpc_subject, DerivedApiSubjects, DerivedEventSubjects,
};
