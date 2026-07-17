//! Pure protocol primitives shared by Trellis runtimes and language bindings.
//!
//! Runtime parsers ignore unknown object members and expose normalized semantic
//! projections containing only fields understood by this version. Those
//! projections are intentionally unsuitable for future cryptographic
//! signatures: signatures must cover the complete raw canonical object except
//! for the signature member itself. New required semantics need a negotiated
//! feature or a new format rather than an ignored critical extension member.

mod api;
mod canonical;
mod error;
mod identifiers;
mod participant;
mod permissions;
mod resolution;
mod schema_profile;
mod subjects;

pub use api::{
    compare_api_replacement_v1, lint_api_v1_authoring, parse_api_v1, ApiArtifactV1,
    ApiCompatibilityIssueCodeV1, ApiCompatibilityIssueV1, ApiCompatibilityReportV1,
    API_AUTHORING_SCHEMA_V1_JSON, API_FORMAT_V1,
};
pub use canonical::{canonicalize_json, digest_json, sha256_base64url};
pub use error::{ProtocolError, ResolutionErrorCodeV1};
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
pub use subjects::{
    derive_event_subject, derive_event_wildcard_subject, derive_feed_subject,
    derive_operation_subject, derive_rpc_subject, DerivedApiSubjectsV1, DerivedEventSubjectsV1,
};
