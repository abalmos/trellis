//! Rust-owned authorization state and deterministic authority materialization.
//!
//! This module is the platform-internal ownership boundary for principals,
//! provider identities, sessions, exact participant bindings, desired identity
//! and deployment authority, runtime evidence, materialized authority, and the
//! unsigned state consumed by later authorization-context issuance.
//!
//! Desired authority records an accepted decision. Materialized authority is a
//! separate fail-closed projection of that decision against authority-scoped
//! participant, dependency, resource, and deployment evidence. Session,
//! instance, and activation eligibility is checked only during issuance. Exact
//! permissions remain [`trellis_protocol::GrantSetV1`] values; platform
//! capabilities never expand those permissions.
//!
//! Context signing and public auth/bootstrap routes are intentionally outside
//! this module.

mod domain;
mod materializer;
mod reconciliation;
mod repository;
mod service;
mod sqlite;

#[cfg(test)]
mod tests;

pub use domain::{
    AuthorityDecision, AuthorityEvidenceScope, AuthorityKind, AuthorityState, AuthorityTarget,
    AuthorizationStateError, AuthorizationTransition, AuthorizationTransitionKind,
    AuthorizationTransitionOutboxRecord, DelegationEvidence, DependencyEvidence, DependencyState,
    DeploymentAuthorityRecord, DeploymentRecord, DesiredAuthorityRecord, DeviceDelegationRecord,
    DeviceDelegationState, DeviceEvidence, DeviceRecord, DeviceState, IdentityAuthorityRecord,
    IssuableAuthorizationState, MaterializationState, MaterializedAuthorityRecord, NewSession,
    ParticipantBindingRecord, ParticipantBindingState, PrincipalAuthorizationChange, PrincipalKind,
    PrincipalRecord, PrincipalState, ProviderIdentityLink, ResourceBindingEvidence,
    ResourceBindingState, RuntimeEvidence, RuntimeInstanceRecord, RuntimeInstanceState,
    ServiceEvidence, SessionRecord, SessionRuntimeBinding, SessionState, MAX_PROTOCOL_INTEGER,
};
pub(crate) use reconciliation::authorization_reconciliation_channel;
pub use reconciliation::{AuthorizationReconciliationHandle, ReconciliationCause};
pub use repository::{
    AuthorityMaterializationSnapshot, AuthorityReconciliationOutcome, AuthoritySnapshotToken,
    AuthoritySubjectRecord, AuthorizationMaterializationRepository, DeploymentAuthorityRepository,
    EvidenceRepository, IdentityAuthorityRepository, InMemoryAuthorizationStore, IssuanceSnapshot,
    MaterializationReplacement, ParticipantBindingRepository, PrincipalRepository,
    ProviderIdentityRepository, SessionRepository,
};
pub use service::AuthorizationStateService;
pub use sqlite::SqliteAuthorizationStore;
