//! Pure protocol primitives shared by Trellis runtimes and language bindings.

mod api;
mod canonical;
mod error;
mod identifiers;
mod participant;
mod permissions;
mod schema_profile;
mod subjects;

pub use api::{parse_api_v1, ApiArtifactV1, API_FORMAT_V1, API_SCHEMA_V1_JSON};
pub use canonical::{canonicalize_json, digest_json, sha256_base64url};
pub use error::ProtocolError;
pub use participant::{
    parse_participant_v1, ParticipantArtifactV1, ParticipantKindV1, PARTICIPANT_FORMAT_V1,
    PARTICIPANT_SCHEMA_V1_JSON,
};
pub use permissions::{
    ApiSurfaceKindV1, CapabilityDefinitionV1, ConsentMetadataV1, GrantSetV1,
    ParticipantResourceKindV1, PermissionActionV1, PermissionAtomV1, PermissionTargetV1,
    GRANT_SET_FORMAT_V1,
};
pub use subjects::{
    derive_event_subject, derive_event_wildcard_subject, derive_feed_subject,
    derive_operation_subject, derive_rpc_subject, DerivedApiSubjectsV1, DerivedEventSubjectsV1,
};
