use trellis_protocol::ParticipantKind;

use super::fixtures::{
    evidence_scope, participant_fixture_with_resources, session_public_key, test_session_creation,
    NOW,
};
use crate::platform::auth::application::repository::{AccountRepository, SessionRepository};
use crate::platform::auth::authority::{AuthorityEvidenceRepository, AuthorityRepository};
use crate::platform::auth::{
    AuthorityDecision, AuthorityKind, AuthorityState, AuthorityTarget, AuthorizationStateError,
    AuthorizationStateService, DesiredAuthorityRecord, IdentityAuthorityRecord, NewSession,
    PrincipalKind, PrincipalRecord, PrincipalState, ResourceBindingEvidence, ResourceBindingState,
    ResourceProviderIdentity, SessionRecord, SqliteAuthorizationStore,
};

pub(super) async fn exercise_resources(
    store: SqliteAuthorizationStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = participant_fixture_with_resources(ParticipantKind::App, "resource.app", true)?;
    store
        .put_participant_binding(fixture.binding.clone())
        .await?;
    store
        .create_principal(PrincipalRecord {
            principal_id: "usr_resources".to_owned(),
            kind: PrincipalKind::User,
            state: PrincipalState::Active,
            created_at: NOW,
            updated_at: NOW,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        })
        .await?;
    let authority = IdentityAuthorityRecord {
        authority_id: "ida_resources".to_owned(),
        principal_id: "usr_resources".to_owned(),
        participant_id: fixture.binding.participant_id.clone(),
        participant_artifact_digest: fixture.binding.artifact_digest.clone(),
        accepted_needs_digest: fixture.binding.needs_digest.clone(),
        desired_grant_set: fixture.all_grants.clone(),
        desired_capabilities: Vec::new(),
        state: AuthorityState::Accepted,
        version: 1,
        created_at: NOW,
        updated_at: NOW,
        expires_at: None,
        decision: Some(AuthorityDecision {
            decided_at: NOW,
            decided_by: "usr_admin".to_owned(),
            reason: None,
        }),
    };
    let session = SessionRecord::from_new(NewSession {
        session_id: "ses_resources".to_owned(),
        principal_id: "usr_resources".to_owned(),
        principal_kind: PrincipalKind::User,
        participant_id: fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKind::App,
        participant_artifact_digest: fixture.binding.artifact_digest.clone(),
        participant_needs_digest: fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(11),
        inbox_prefix: "_INBOX.resources".to_owned(),
        created_at: NOW,
        expires_at: None,
    })?;
    store
        .create_session(test_session_creation(
            session.clone(),
            Some(DesiredAuthorityRecord::Identity(authority)),
            None,
        ))
        .await?;
    store
        .replace_dependency_evidence(
            evidence_scope(AuthorityKind::Identity, "ida_resources", &fixture.binding),
            vec![fixture.required_dependency.clone()],
        )
        .await?;
    let facade = AuthorizationStateService::new(store.clone());
    let target = AuthorityTarget::new(AuthorityKind::Identity, "ida_resources")?;
    facade.reconcile_authority(&target, NOW + 1).await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&session.session_id, NOW + 1)
            .await,
        Err(AuthorizationStateError::MaterializationStale)
    );

    let required_resource = ResourceBindingEvidence {
        resource_kind: "kv".to_owned(),
        local_name: "cache".to_owned(),
        binding_id: "binding_cache".to_owned(),
        owner_participant_id: fixture.binding.participant_id.clone(),
        provider_identity: ResourceProviderIdentity::Kv {
            bucket: "storage_kv_cache".to_owned(),
        },
        state: ResourceBindingState::Available,
        materialized_at: NOW,
        error: None,
    };
    store
        .replace_resource_evidence(
            evidence_scope(AuthorityKind::Identity, "ida_resources", &fixture.binding),
            vec![required_resource.clone()],
        )
        .await?;
    facade.reconcile_authority(&target, NOW + 2).await?;
    let required_only = facade
        .resolve_issuable_state(&session.session_id, NOW + 2)
        .await?;
    assert_eq!(required_only.grant_set, fixture.required_grants);

    store
        .replace_resource_evidence(
            evidence_scope(AuthorityKind::Identity, "ida_resources", &fixture.binding),
            vec![
                required_resource,
                ResourceBindingEvidence {
                    resource_kind: "store".to_owned(),
                    local_name: "attachments".to_owned(),
                    binding_id: "binding_attachments".to_owned(),
                    owner_participant_id: fixture.binding.participant_id.clone(),
                    provider_identity: ResourceProviderIdentity::Store {
                        bucket: "storage_attachments".to_owned(),
                    },
                    state: ResourceBindingState::Available,
                    materialized_at: NOW,
                    error: None,
                },
            ],
        )
        .await?;
    store
        .replace_dependency_evidence(
            evidence_scope(AuthorityKind::Identity, "ida_resources", &fixture.binding),
            vec![
                fixture.required_dependency.clone(),
                fixture.optional_dependency.clone(),
            ],
        )
        .await?;
    facade.reconcile_authority(&target, NOW + 3).await?;
    let complete = facade
        .resolve_issuable_state(&session.session_id, NOW + 3)
        .await?;
    assert_eq!(complete.grant_set, fixture.all_grants);
    Ok(())
}
