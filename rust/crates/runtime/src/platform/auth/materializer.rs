use std::collections::BTreeSet;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};
use trellis_protocol::{
    AuthorizationAuthorityKindV1, AuthorizationPrincipalKindV1, GrantSetV1, ParticipantKindV1,
    ParticipantResourceKindV1, PermissionAtomV1,
};

use super::domain::canonical_capabilities;
use super::repository::{
    AuthorityMaterializationSnapshot, AuthoritySubjectRecord, MaterializationReplacement,
};
use super::{
    AuthorityKind, AuthorizationStateError, AuthorizationTransition, AuthorizationTransitionKind,
    DependencyEvidence, DependencyState, DesiredAuthorityRecord, MaterializationState,
    MaterializedAuthorityRecord, PrincipalKind, PrincipalState, ResourceBindingEvidence,
    ResourceBindingState,
};

pub(crate) fn materialize_authority(
    snapshot: &AuthorityMaterializationSnapshot,
    now: i64,
) -> Option<MaterializationReplacement> {
    let authority = snapshot.authority.as_ref()?;
    Some(match materialize_available(snapshot, authority, now) {
        Ok(replacement) => replacement,
        Err(error) => unavailable(snapshot, authority, &error, now),
    })
}

fn materialize_available(
    snapshot: &AuthorityMaterializationSnapshot,
    authority: &DesiredAuthorityRecord,
    now: i64,
) -> Result<MaterializationReplacement, AuthorizationStateError> {
    validate_authority(authority, now)?;
    let expiry = validate_subject(snapshot, authority, now)?;
    let binding = snapshot
        .participant
        .as_ref()
        .ok_or(AuthorizationStateError::ParticipantMissing)?;
    if binding.participant_id != authority.participant_id()
        || binding.artifact_digest != authority.participant_artifact_digest()
    {
        return Err(AuthorizationStateError::ParticipantDigestMismatch);
    }
    if binding.needs_digest != authority.accepted_needs_digest() {
        return Err(AuthorizationStateError::NeedsDigestMismatch);
    }
    if snapshot
        .dependency_scope
        .as_ref()
        .is_some_and(|scope| !scope_matches(scope, authority))
        || snapshot
            .resource_scope
            .as_ref()
            .is_some_and(|scope| !scope_matches(scope, authority))
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "authority evidence scope does not match desired authority".to_owned(),
        ));
    }
    let participant = binding.resolve()?;

    let mut requested = participant
        .needs()
        .required()
        .grant_set()
        .permissions()
        .to_vec();
    for dependency in participant.required_apis() {
        require_dependency(dependency, true, &snapshot.dependencies)?;
    }
    require_resource_needs(
        participant.participant_id(),
        participant.needs().required().resources(),
        &snapshot.resources,
    )?;
    for dependency in participant.optional_apis() {
        if dependency_available(dependency, false, &snapshot.dependencies) {
            requested.extend_from_slice(dependency.grant_set().permissions());
        }
    }
    requested.extend(
        participant
            .needs()
            .optional()
            .grant_set()
            .permissions()
            .iter()
            .filter(|permission| {
                optional_resource_available(
                    participant.participant_id(),
                    permission,
                    &snapshot.resources,
                )
            })
            .cloned(),
    );
    let accepted = authority.grant_set().permissions();
    let effective = GrantSetV1::new(
        requested
            .into_iter()
            .filter(|permission| accepted.contains(permission))
            .collect(),
    );

    let mut requested_capabilities = BTreeSet::new();
    requested_capabilities.extend(
        participant
            .proposal()
            .required()
            .capabilities()
            .iter()
            .map(|value| value.name().to_owned()),
    );
    for capability in participant.proposal().optional().capabilities() {
        if participant.optional_apis().iter().any(|dependency| {
            dependency.api() == capability.api()
                && dependency.api_digest() == capability.api_digest()
                && dependency_available(dependency, false, &snapshot.dependencies)
        }) {
            requested_capabilities.insert(capability.name().to_owned());
        }
    }
    let accepted_capabilities = authority.capabilities().iter().collect::<BTreeSet<_>>();
    let capabilities = canonical_capabilities(
        requested_capabilities
            .into_iter()
            .filter(|value| accepted_capabilities.contains(value))
            .collect::<Vec<_>>(),
    )?;

    Ok(MaterializationReplacement {
        authority: materialization_header(
            authority,
            participant.participant_kind(),
            effective,
            capabilities,
            MaterializationState::Available,
            None,
            expiry,
            now,
        ),
        dependencies: snapshot.dependencies.clone(),
        resources: snapshot.resources.clone(),
    })
}

fn scope_matches(
    scope: &super::AuthorityEvidenceScope,
    authority: &DesiredAuthorityRecord,
) -> bool {
    scope.target == authority.target()
        && scope.participant_id == authority.participant_id()
        && scope.participant_artifact_digest == authority.participant_artifact_digest()
        && scope.participant_needs_digest == authority.accepted_needs_digest()
}

fn unavailable(
    snapshot: &AuthorityMaterializationSnapshot,
    authority: &DesiredAuthorityRecord,
    error: &AuthorizationStateError,
    now: i64,
) -> MaterializationReplacement {
    let state = if matches!(error, AuthorizationStateError::InvalidRecord(_)) {
        MaterializationState::Error
    } else {
        MaterializationState::Unavailable
    };
    let participant_kind = match authority {
        DesiredAuthorityRecord::Identity(_) => snapshot
            .participant
            .as_ref()
            .map_or(ParticipantKindV1::App, |binding| binding.participant_kind),
        DesiredAuthorityRecord::Deployment(record) => record.participant_kind,
    };
    let expires_at = match (authority, snapshot.subject.as_ref()) {
        (
            DesiredAuthorityRecord::Deployment(record),
            Some(AuthoritySubjectRecord::Deployment(deployment)),
        ) if deployment.deployment_id == record.deployment_id
            && deployment.participant_id == record.participant_id
            && deployment.participant_kind == record.participant_kind =>
        {
            min_expiry(record.expires_at, deployment.expires_at)
        }
        _ => authority.expires_at(),
    };
    MaterializationReplacement {
        authority: materialization_header(
            authority,
            participant_kind,
            GrantSetV1::new(Vec::new()),
            Vec::new(),
            state,
            Some(error_category(error).to_owned()),
            expires_at,
            now,
        ),
        dependencies: snapshot.dependencies.clone(),
        resources: snapshot.resources.clone(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the header mirrors one durable row"
)]
fn materialization_header(
    authority: &DesiredAuthorityRecord,
    participant_kind: ParticipantKindV1,
    effective_grant_set: GrantSetV1,
    effective_capabilities: Vec<String>,
    state: MaterializationState,
    error: Option<String>,
    expires_at: Option<i64>,
    now: i64,
) -> MaterializedAuthorityRecord {
    let target = authority.target();
    MaterializedAuthorityRecord {
        materialization_id: format!(
            "mat:{}:{}",
            match target.kind {
                AuthorityKind::Identity => "identity",
                AuthorityKind::Deployment => "deployment",
            },
            target.authority_id
        ),
        authority_kind: target.kind,
        authority_id: target.authority_id,
        authority_version: authority.version(),
        materialization_version: 1,
        subject_id: authority.subject_id().to_owned(),
        participant_id: authority.participant_id().to_owned(),
        participant_kind,
        participant_artifact_digest: authority.participant_artifact_digest().to_owned(),
        participant_needs_digest: authority.accepted_needs_digest().to_owned(),
        effective_grant_set,
        effective_capabilities,
        state,
        reconciled_at: Some(now),
        error,
        expires_at,
    }
}

fn validate_authority(
    authority: &DesiredAuthorityRecord,
    now: i64,
) -> Result<(), AuthorizationStateError> {
    match authority.state() {
        super::AuthorityState::Accepted => {}
        super::AuthorityState::Pending => return Err(AuthorizationStateError::AuthorityPending),
        super::AuthorityState::Rejected => return Err(AuthorizationStateError::AuthorityRejected),
        super::AuthorityState::Revoked => return Err(AuthorizationStateError::AuthorityRevoked),
        super::AuthorityState::Stale => return Err(AuthorizationStateError::AuthorityStale),
    }
    if authority.expires_at().is_some_and(|expiry| now >= expiry) {
        return Err(AuthorizationStateError::AuthorityExpired);
    }
    Ok(())
}

fn validate_subject(
    snapshot: &AuthorityMaterializationSnapshot,
    authority: &DesiredAuthorityRecord,
    now: i64,
) -> Result<Option<i64>, AuthorizationStateError> {
    match (authority, snapshot.subject.as_ref()) {
        (
            DesiredAuthorityRecord::Identity(record),
            Some(AuthoritySubjectRecord::Identity(principal)),
        ) => {
            if principal.principal_id != record.principal_id
                || principal.kind != PrincipalKind::User
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "identity authority principal does not match".to_owned(),
                ));
            }
            if principal.state != PrincipalState::Active {
                return Err(AuthorizationStateError::PrincipalInactive);
            }
            Ok(record.expires_at)
        }
        (
            DesiredAuthorityRecord::Deployment(record),
            Some(AuthoritySubjectRecord::Deployment(deployment)),
        ) => {
            if deployment.deployment_id != record.deployment_id
                || deployment.participant_id != record.participant_id
                || deployment.participant_kind != record.participant_kind
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "deployment authority subject does not match".to_owned(),
                ));
            }
            if !deployment.active || deployment.expires_at.is_some_and(|expiry| now >= expiry) {
                return Err(AuthorizationStateError::DeploymentInactive);
            }
            Ok(min_expiry(record.expires_at, deployment.expires_at))
        }
        (DesiredAuthorityRecord::Identity(_), None) => {
            Err(AuthorizationStateError::PrincipalMissing)
        }
        (DesiredAuthorityRecord::Deployment(_), None) => {
            Err(AuthorizationStateError::DeploymentInactive)
        }
        _ => Err(AuthorizationStateError::InvalidRecord(
            "authority subject kind does not match authority kind".to_owned(),
        )),
    }
}

fn require_dependency(
    dependency: &trellis_protocol::ResolvedUsedApiV1,
    required: bool,
    evidence: &[DependencyEvidence],
) -> Result<(), AuthorizationStateError> {
    if dependency_available(dependency, required, evidence) {
        Ok(())
    } else {
        Err(AuthorizationStateError::RequiredDependencyUnavailable(
            dependency.alias().to_owned(),
        ))
    }
}

fn dependency_available(
    dependency: &trellis_protocol::ResolvedUsedApiV1,
    required: bool,
    evidence: &[DependencyEvidence],
) -> bool {
    evidence.iter().any(|item| {
        item.alias == dependency.alias()
            && item.required == required
            && item.api_id == dependency.api()
            && item.api_digest == dependency.api_digest()
            && item.state == DependencyState::Available
    })
}

fn require_resource_needs(
    participant_id: &str,
    resources: &trellis_protocol::ParticipantResourceNeedsV1,
    evidence: &[ResourceBindingEvidence],
) -> Result<(), AuthorizationStateError> {
    for (resource_kind, entries) in [
        ("state", resources.state()),
        ("jobQueue", resources.job_queues()),
        ("eventConsumer", resources.event_consumers()),
        ("kv", resources.kv()),
        ("store", resources.stores()),
    ] {
        for local_name in entries.keys() {
            let available = evidence.iter().any(|item| {
                item.owner_participant_id == participant_id
                    && item.resource_kind == resource_kind
                    && item.local_name == *local_name
                    && item.state == ResourceBindingState::Available
            });
            if !available {
                return Err(AuthorizationStateError::RequiredResourceUnavailable(
                    format!("{resource_kind}.{local_name}"),
                ));
            }
        }
    }
    Ok(())
}

fn optional_resource_available(
    participant_id: &str,
    permission: &PermissionAtomV1,
    evidence: &[ResourceBindingEvidence],
) -> bool {
    let Some((owner, kind, local_name)) = permission.target().as_participant_resource() else {
        return false;
    };
    let resource_kind = match kind {
        ParticipantResourceKindV1::State => "state",
        ParticipantResourceKindV1::JobQueue => "jobQueue",
        ParticipantResourceKindV1::EventConsumer => "eventConsumer",
        ParticipantResourceKindV1::Kv => "kv",
        ParticipantResourceKindV1::Store => "store",
    };
    owner == participant_id
        && evidence.iter().any(|item| {
            item.owner_participant_id == participant_id
                && item.resource_kind == resource_kind
                && item.local_name == local_name
                && item.state == ResourceBindingState::Available
        })
}

pub(crate) fn transition_for_change(
    previous: Option<&MaterializationReplacement>,
    current: Option<&MaterializationReplacement>,
    now: i64,
) -> Result<Option<AuthorizationTransition>, AuthorizationStateError> {
    let (authority_kind, authority_id, authority_version, materialization_version, state, error) =
        match current {
            Some(current) => (
                current.authority.authority_kind,
                current.authority.authority_id.clone(),
                current.authority.authority_version,
                current.authority.materialization_version,
                current.authority.state,
                current.authority.error.clone(),
            ),
            None => {
                let Some(previous) = previous else {
                    return Ok(None);
                };
                let materialization_version = previous
                    .authority
                    .materialization_version
                    .checked_add(1)
                    .filter(|version| *version <= super::MAX_PROTOCOL_INTEGER)
                    .ok_or_else(|| {
                        AuthorizationStateError::InvalidRecord(
                            "materializationVersion exceeds protocol-safe range".to_owned(),
                        )
                    })?;
                (
                    previous.authority.authority_kind,
                    previous.authority.authority_id.clone(),
                    previous.authority.authority_version,
                    materialization_version,
                    MaterializationState::Unavailable,
                    Some("authority_removed".to_owned()),
                )
            }
        };
    let kind = match (
        previous.map(|value| value.authority.state),
        current.map(|value| value.authority.state),
    ) {
        (
            Some(MaterializationState::Unavailable | MaterializationState::Error),
            Some(MaterializationState::Available),
        ) => AuthorizationTransitionKind::MaterializedRestored,
        (_, None | Some(MaterializationState::Unavailable | MaterializationState::Error)) => {
            AuthorizationTransitionKind::MaterializedUnavailable
        }
        _ => AuthorizationTransitionKind::MaterializedChanged,
    };
    let event_id =
        transition_event_id(authority_kind, &authority_id, materialization_version, kind)?;
    Ok(Some(AuthorizationTransition {
        event_id,
        kind,
        authority_kind,
        authority_id,
        authority_version,
        materialization_version,
        state,
        error,
        created_at: now,
    }))
}

fn transition_event_id(
    authority_kind: AuthorityKind,
    authority_id: &str,
    materialization_version: u64,
    transition_kind: AuthorizationTransitionKind,
) -> Result<String, AuthorizationStateError> {
    let identity = serde_json::to_vec(&(
        authority_kind,
        authority_id,
        materialization_version,
        transition_kind,
    ))
    .map_err(|error| {
        AuthorizationStateError::Storage(format!(
            "cannot encode authorization transition identity: {error}"
        ))
    })?;
    Ok(URL_SAFE_NO_PAD.encode(Sha256::digest(identity)))
}

pub(crate) fn protocol_principal_kind(kind: PrincipalKind) -> AuthorizationPrincipalKindV1 {
    match kind {
        PrincipalKind::User => AuthorizationPrincipalKindV1::User,
        PrincipalKind::Service => AuthorizationPrincipalKindV1::Service,
        PrincipalKind::Device => AuthorizationPrincipalKindV1::Device,
    }
}

pub(crate) fn protocol_authority_kind(kind: AuthorityKind) -> AuthorizationAuthorityKindV1 {
    match kind {
        AuthorityKind::Identity => AuthorizationAuthorityKindV1::Identity,
        AuthorityKind::Deployment => AuthorizationAuthorityKindV1::Deployment,
    }
}

fn min_expiry(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn error_category(error: &AuthorizationStateError) -> &'static str {
    match error {
        AuthorizationStateError::SessionMissing => "session_missing",
        AuthorizationStateError::SessionExpired => "session_expired",
        AuthorizationStateError::SessionRevoked => "session_revoked",
        AuthorizationStateError::PrincipalMissing => "principal_missing",
        AuthorizationStateError::PrincipalInactive => "principal_inactive",
        AuthorizationStateError::ParticipantMissing => "participant_missing",
        AuthorizationStateError::ParticipantDigestMismatch => "participant_digest_mismatch",
        AuthorizationStateError::NeedsDigestMismatch => "needs_digest_mismatch",
        AuthorizationStateError::AuthorityMissing => "authority_missing",
        AuthorizationStateError::AuthorityPending => "authority_pending",
        AuthorizationStateError::AuthorityRejected => "authority_rejected",
        AuthorizationStateError::AuthorityRevoked => "authority_revoked",
        AuthorizationStateError::AuthorityStale => "authority_stale",
        AuthorizationStateError::AuthorityExpired => "authority_expired",
        AuthorizationStateError::DeploymentInactive => "deployment_inactive",
        AuthorizationStateError::InstanceInactive => "instance_inactive",
        AuthorizationStateError::DeviceInactive => "device_inactive",
        AuthorizationStateError::ActivationMissing => "activation_missing",
        AuthorizationStateError::DelegationExpired => "delegation_expired",
        AuthorizationStateError::RequiredDependencyUnavailable(_) => {
            "required_dependency_unavailable"
        }
        AuthorizationStateError::RequiredResourceUnavailable(_) => "required_resource_unavailable",
        AuthorizationStateError::MaterializationStale => "materialization_stale",
        AuthorizationStateError::StorageConflict => "storage_conflict",
        AuthorizationStateError::InvalidRecord(_) => "invalid_record",
        AuthorizationStateError::Storage(_) => "storage_error",
    }
}
