use trellis_protocol::ParticipantKind;

use super::authority::exercise_store;
use super::fixtures::{
    evidence_scope, participant_fixture_for, participant_fixture_with_resources,
    session_public_key, test_session_creation, NOW,
};
use crate::platform::auth::application::repository::{AccountRepository, SessionRepository};
use crate::platform::auth::authority::{
    AuthorityEvidenceRepository, AuthorityRepository, ContextRepository,
};
use crate::platform::auth::{
    AuthorityDecision, AuthorityKind, AuthorityState, AuthorityTarget, AuthorizationStateError,
    AuthorizationStateService, DeploymentAuthorityRecord, DeploymentRecord, MaterializationState,
    NewSession, PrincipalKind, PrincipalRecord, PrincipalState, ResourceBindingEvidence,
    ResourceBindingState, ResourceProviderIdentity, RuntimeInstanceRecord, RuntimeInstanceState,
    SessionRecord, SessionRuntimeBinding, SqliteAuthorizationStore,
};

#[tokio::test]
async fn sqlite_deployments_and_instances_are_isolated() -> Result<(), Box<dyn std::error::Error>> {
    exercise_deployment_isolation(SqliteAuthorizationStore::open_in_memory()?).await
}

async fn exercise_deployment_isolation(
    store: SqliteAuthorizationStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture =
        participant_fixture_with_resources(ParticipantKind::Service, "shared.service", true)?;
    store
        .put_participant_binding(fixture.binding.clone())
        .await?;
    let facade = AuthorizationStateService::new(store.clone());
    for (suffix, key_seed) in [("a", 21_u8), ("b", 22_u8)] {
        let principal_id = format!("svc_{suffix}");
        let session_id = format!("ses_{suffix}");
        let deployment_id = format!("dep_{suffix}");
        let instance_id = format!("inst_{suffix}");
        let authority_id = format!("dpa_{suffix}");
        store
            .create_principal(PrincipalRecord {
                principal_id: principal_id.clone(),
                kind: PrincipalKind::Service,
                state: PrincipalState::Active,
                created_at: NOW,
                updated_at: NOW,
                version: 1,
                disabled_at: None,
                revoked_at: None,
            })
            .await?;
        store
            .put_deployment_authority(
                DeploymentAuthorityRecord {
                    authority_id: authority_id.clone(),
                    deployment_id: deployment_id.clone(),
                    participant_id: fixture.binding.participant_id.clone(),
                    participant_kind: ParticipantKind::Service,
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
                },
                None,
            )
            .await?;
        store
            .put_deployment_evidence(DeploymentRecord {
                deployment_id: deployment_id.clone(),
                participant_id: fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKind::Service,
                active: true,
                expires_at: None,
            })
            .await?;
        store
            .put_runtime_instance(RuntimeInstanceRecord {
                instance_id: instance_id.clone(),
                deployment_id: deployment_id.clone(),
                principal_id: format!("svc_{suffix}"),
                state: RuntimeInstanceState::Active,
                created_at: NOW,
                updated_at: NOW,
                version: 1,
            })
            .await?;
        let session = SessionRecord::from_new(NewSession {
            session_id: session_id.clone(),
            principal_id,
            principal_kind: PrincipalKind::Service,
            participant_id: fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKind::Service,
            participant_artifact_digest: fixture.binding.artifact_digest.clone(),
            participant_needs_digest: fixture.binding.needs_digest.clone(),
            session_public_key: session_public_key(key_seed),
            inbox_prefix: format!("_INBOX.{suffix}"),
            created_at: NOW,
            expires_at: None,
        })?;
        store
            .create_session(test_session_creation(
                session,
                None,
                Some(SessionRuntimeBinding {
                    session_id: session_id.clone(),
                    deployment_id: deployment_id.clone(),
                    instance_id,
                }),
            ))
            .await?;
        let mut dependency = fixture.required_dependency.clone();
        dependency.provider_deployment_id = Some(format!("provider_dep_{suffix}"));
        store
            .replace_dependency_evidence(
                evidence_scope(AuthorityKind::Deployment, &authority_id, &fixture.binding),
                vec![dependency],
            )
            .await?;
        store
            .replace_resource_evidence(
                evidence_scope(AuthorityKind::Deployment, &authority_id, &fixture.binding),
                vec![ResourceBindingEvidence {
                    resource_kind: "kv".to_owned(),
                    local_name: "cache".to_owned(),
                    binding_id: format!("binding_{suffix}"),
                    owner_participant_id: fixture.binding.participant_id.clone(),
                    provider_identity: ResourceProviderIdentity::Kv {
                        bucket: format!("storage_{suffix}"),
                    },
                    state: ResourceBindingState::Available,
                    materialized_at: NOW,
                    error: None,
                }],
            )
            .await?;
        facade
            .reconcile_authority(
                &AuthorityTarget::new(AuthorityKind::Deployment, &authority_id)?,
                NOW + 1,
            )
            .await?;
        assert!(facade
            .resolve_issuable_state(&session_id, NOW + 2)
            .await
            .is_ok());
    }

    let first = store
        .get_materialized_authority(AuthorityKind::Deployment, "dpa_a")
        .await?
        .ok_or("missing first deployment materialization")?;
    let second = store
        .get_materialized_authority(AuthorityKind::Deployment, "dpa_b")
        .await?
        .ok_or("missing second deployment materialization")?;
    assert_eq!(
        first.dependencies[0].provider_deployment_id.as_deref(),
        Some("provider_dep_a")
    );
    assert_eq!(
        second.dependencies[0].provider_deployment_id.as_deref(),
        Some("provider_dep_b")
    );
    assert_eq!(
        first.resources[0].provider_identity,
        ResourceProviderIdentity::Kv {
            bucket: "storage_a".to_owned(),
        }
    );
    assert_eq!(
        second.resources[0].provider_identity,
        ResourceProviderIdentity::Kv {
            bucket: "storage_b".to_owned(),
        }
    );
    assert_eq!(
        store
            .put_deployment_evidence(DeploymentRecord {
                deployment_id: "dep_a".to_owned(),
                participant_id: "different.service".to_owned(),
                participant_kind: ParticipantKind::Service,
                active: true,
                expires_at: None,
            })
            .await,
        Err(AuthorizationStateError::InvalidRecord(
            "deployment participant identity cannot change".to_owned()
        ))
    );
    let first_target = AuthorityTarget::new(AuthorityKind::Deployment, "dpa_a")?;
    let before_instance_change = facade.reconcile_authority(&first_target, NOW + 2).await?;
    assert!(!before_instance_change.changed);

    store
        .put_runtime_instance(RuntimeInstanceRecord {
            instance_id: "inst_a".to_owned(),
            deployment_id: "dep_a".to_owned(),
            principal_id: "svc_a".to_owned(),
            state: RuntimeInstanceState::Disabled,
            created_at: NOW,
            updated_at: NOW + 3,
            version: 2,
        })
        .await?;
    assert_eq!(
        facade.resolve_issuable_state("ses_a", NOW + 3).await,
        Err(AuthorizationStateError::InstanceInactive)
    );
    assert!(facade
        .resolve_issuable_state("ses_b", NOW + 3)
        .await
        .is_ok());
    let instance_only = facade.reconcile_authority(&first_target, NOW + 3).await?;
    assert!(!instance_only.changed);
    assert_eq!(
        instance_only.snapshot_token,
        before_instance_change.snapshot_token
    );
    assert_eq!(
        store
            .get_materialized_authority(AuthorityKind::Deployment, "dpa_a")
            .await?
            .ok_or("missing first deployment materialization")?
            .authority
            .state,
        MaterializationState::Available
    );
    Ok(())
}

#[tokio::test]
async fn typed_authority_ids_do_not_collide_in_sqlite() -> Result<(), Box<dyn std::error::Error>> {
    exercise_typed_authority_ids(SqliteAuthorizationStore::open_in_memory()?).await
}

async fn exercise_typed_authority_ids(
    store: SqliteAuthorizationStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = exercise_store(store).await?;
    let fixture = participant_fixture_for(ParticipantKind::Service, "collision.service")?;
    store
        .put_participant_binding(fixture.binding.clone())
        .await?;
    store
        .create_principal(PrincipalRecord {
            principal_id: "svc_collision".to_owned(),
            kind: PrincipalKind::Service,
            state: PrincipalState::Active,
            created_at: NOW,
            updated_at: NOW,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        })
        .await?;
    let session = SessionRecord::from_new(NewSession {
        session_id: "ses_collision".to_owned(),
        principal_id: "svc_collision".to_owned(),
        principal_kind: PrincipalKind::Service,
        participant_id: fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKind::Service,
        participant_artifact_digest: fixture.binding.artifact_digest.clone(),
        participant_needs_digest: fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(23),
        inbox_prefix: "_INBOX.collision".to_owned(),
        created_at: NOW,
        expires_at: None,
    })?;
    store
        .put_deployment_authority(
            DeploymentAuthorityRecord {
                authority_id: "ida_01".to_owned(),
                deployment_id: "dep_collision".to_owned(),
                participant_id: fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKind::Service,
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
            },
            None,
        )
        .await?;
    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_collision".to_owned(),
            participant_id: fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKind::Service,
            active: true,
            expires_at: None,
        })
        .await?;
    store
        .put_runtime_instance(RuntimeInstanceRecord {
            instance_id: "inst_collision".to_owned(),
            deployment_id: "dep_collision".to_owned(),
            principal_id: session.principal_id.clone(),
            state: RuntimeInstanceState::Active,
            created_at: NOW,
            updated_at: NOW,
            version: 1,
        })
        .await?;
    store
        .create_session(test_session_creation(
            session.clone(),
            None,
            Some(SessionRuntimeBinding {
                session_id: session.session_id.clone(),
                deployment_id: "dep_collision".to_owned(),
                instance_id: "inst_collision".to_owned(),
            }),
        ))
        .await?;
    store
        .replace_dependency_evidence(
            evidence_scope(AuthorityKind::Deployment, "ida_01", &fixture.binding),
            vec![fixture.required_dependency],
        )
        .await?;
    AuthorizationStateService::new(store.clone())
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "ida_01")?,
            NOW + 200,
        )
        .await?;
    assert!(store
        .get_materialized_authority(AuthorityKind::Identity, "ida_01")
        .await?
        .is_some());
    assert!(store
        .get_materialized_authority(AuthorityKind::Deployment, "ida_01")
        .await?
        .is_some());
    Ok(())
}
