use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::SigningKey;
use serde_json::{json, Value};
use trellis_protocol::{
    authorization_context_signing_digest_v1, parse_api_v1, parse_participant_v1,
    resolve_participant_v1, GrantSetV1, ParticipantKindV1, UnsignedAuthorizationContextV1,
    AUTHORIZATION_CONTEXT_FORMAT_V1,
};

use super::*;

const NOW: i64 = 1_800_000_000_000;

#[tokio::test]
async fn in_memory_authorization_state_conformance() -> Result<(), Box<dyn std::error::Error>> {
    exercise_store(InMemoryAuthorizationStore::default())
        .await
        .map(|_| ())
}

#[tokio::test]
async fn sqlite_authorization_state_conformance() -> Result<(), Box<dyn std::error::Error>> {
    let store = exercise_store(SqliteAuthorizationStore::open_in_memory()?).await?;
    let mut authority = store
        .get_identity_authority("usr_01", "example.app")
        .await?
        .ok_or("missing authority before startup repair")?;
    authority.state = AuthorityState::Accepted;
    authority.version = 3;
    authority.updated_at = NOW + 110;
    authority.decision = Some(AuthorityDecision {
        decided_at: NOW + 110,
        decided_by: "usr_admin".to_owned(),
        reason: Some("authority restored".to_owned()),
    });
    store.put_identity_authority(authority, Some(2)).await?;
    AuthorizationStateService::new(store.clone())
        .reconcile_all(NOW + 120)
        .await?;
    let repaired = store
        .get_materialized_authority(AuthorityKind::Identity, "ida_01")
        .await?
        .ok_or("missing startup-repaired materialization")?;
    assert_eq!(repaired.authority.authority_version, 3);
    assert_eq!(repaired.authority.state, MaterializationState::Unavailable);
    assert!(repaired
        .authority
        .effective_grant_set
        .permissions()
        .is_empty());
    Ok(())
}

#[tokio::test]
async fn in_memory_service_and_device_issuable_state() -> Result<(), Box<dyn std::error::Error>> {
    exercise_deployed_principals(InMemoryAuthorizationStore::default()).await
}

#[tokio::test]
async fn sqlite_service_and_device_issuable_state() -> Result<(), Box<dyn std::error::Error>> {
    exercise_deployed_principals(SqliteAuthorizationStore::open_in_memory()?).await
}

#[tokio::test]
async fn in_memory_runtime_evidence_entities_are_isolated() -> Result<(), Box<dyn std::error::Error>>
{
    exercise_runtime_evidence_entities(InMemoryAuthorizationStore::default()).await
}

#[tokio::test]
async fn sqlite_runtime_evidence_entities_are_isolated() -> Result<(), Box<dyn std::error::Error>> {
    exercise_runtime_evidence_entities(SqliteAuthorizationStore::open_in_memory()?).await
}

#[tokio::test]
async fn in_memory_required_and_optional_resource_materialization(
) -> Result<(), Box<dyn std::error::Error>> {
    exercise_resources(InMemoryAuthorizationStore::default()).await
}

#[tokio::test]
async fn sqlite_required_and_optional_resource_materialization(
) -> Result<(), Box<dyn std::error::Error>> {
    exercise_resources(SqliteAuthorizationStore::open_in_memory()?).await
}

#[tokio::test]
async fn reconciliation_worker_contains_expected_denials_and_stops(
) -> Result<(), Box<dyn std::error::Error>> {
    let service = AuthorizationStateService::new(InMemoryAuthorizationStore::default());
    let (handle, worker) = super::reconciliation::authorization_reconciliation_channel(service, 4);
    let stop = crate::shutdown::StopHandle::new();
    let task_stop = stop.clone();
    let join = tokio::spawn(worker.run(task_stop));

    handle
        .reconcile(
            AuthorityTarget::new(AuthorityKind::Identity, "ida_missing")?,
            ReconciliationCause::StartupRepair,
        )
        .await?;
    tokio::task::yield_now().await;
    assert!(!join.is_finished());

    stop.stop();
    join.await??;
    Ok(())
}

#[test]
fn session_rejects_malformed_public_keys_and_kind_mismatches() {
    let base = NewSession {
        session_id: "ses_invalid".to_owned(),
        principal_id: "usr_invalid".to_owned(),
        principal_kind: PrincipalKind::User,
        participant_id: "example.app".to_owned(),
        participant_kind: ParticipantKindV1::App,
        participant_artifact_digest: digest(1),
        participant_needs_digest: digest(2),
        session_public_key: "not-a-key".to_owned(),
        inbox_prefix: "_INBOX.invalid".to_owned(),
        created_at: NOW,
        expires_at: None,
    };
    assert!(matches!(
        SessionRecord::from_new(base.clone()),
        Err(AuthorizationStateError::InvalidRecord(_))
    ));
    assert!(matches!(
        SessionRecord::from_new(NewSession {
            session_public_key: URL_SAFE_NO_PAD.encode([0_u8; 32]),
            ..base.clone()
        }),
        Err(AuthorizationStateError::InvalidRecord(_))
    ));
    assert!(matches!(
        SessionRecord::from_new(NewSession {
            session_public_key: session_public_key(1),
            participant_kind: ParticipantKindV1::Service,
            ..base
        }),
        Err(AuthorizationStateError::InvalidRecord(_))
    ));
}

#[tokio::test]
async fn sqlite_schema_is_idempotent_and_enforces_foreign_keys(
) -> Result<(), Box<dyn std::error::Error>> {
    let store = SqliteAuthorizationStore::open_in_memory()?;
    let session = SessionRecord::from_new(NewSession {
        session_id: "ses_missing".to_owned(),
        principal_id: "usr_missing".to_owned(),
        principal_kind: PrincipalKind::User,
        participant_id: "example.app".to_owned(),
        participant_kind: ParticipantKindV1::App,
        participant_artifact_digest: digest(1),
        participant_needs_digest: digest(2),
        session_public_key: session_public_key(8),
        inbox_prefix: "_INBOX.missing".to_owned(),
        created_at: NOW,
        expires_at: None,
    })?;
    assert_eq!(
        store.create_session(session).await,
        Err(AuthorizationStateError::PrincipalMissing)
    );
    Ok(())
}

async fn exercise_store<S>(store: S) -> Result<S, Box<dyn std::error::Error>>
where
    S: PrincipalRepository
        + ProviderIdentityRepository
        + SessionRepository
        + ParticipantBindingRepository
        + IdentityAuthorityRepository
        + DeploymentAuthorityRepository
        + EvidenceRepository
        + AuthorizationMaterializationRepository
        + Clone,
{
    let fixture = participant_fixture()?;
    store
        .put_participant_binding(fixture.binding.clone())
        .await?;
    let principal = PrincipalRecord {
        principal_id: "usr_01".to_owned(),
        kind: PrincipalKind::User,
        state: PrincipalState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
        disabled_at: None,
        revoked_at: None,
    };
    store.create_principal(principal.clone()).await?;
    assert_eq!(
        store.create_principal(principal.clone()).await,
        Err(AuthorizationStateError::StorageConflict)
    );

    let link = ProviderIdentityLink {
        provider: "oidc.example".to_owned(),
        provider_subject: "provider-subject".to_owned(),
        principal_id: principal.principal_id.clone(),
        linked_at: NOW,
        last_seen_at: NOW,
    };
    store.link_provider_identity(link.clone()).await?;
    assert_eq!(
        store
            .get_provider_identity(&link.provider, &link.provider_subject)
            .await?,
        Some(link.clone())
    );
    assert_eq!(
        store.link_provider_identity(link).await,
        Err(AuthorizationStateError::StorageConflict)
    );

    let session = SessionRecord::from_new(NewSession {
        session_id: "ses_01".to_owned(),
        principal_id: principal.principal_id.clone(),
        principal_kind: PrincipalKind::User,
        participant_id: fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKindV1::App,
        participant_artifact_digest: fixture.binding.artifact_digest.clone(),
        participant_needs_digest: fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(7),
        inbox_prefix: "_INBOX.session-01".to_owned(),
        created_at: NOW,
        expires_at: None,
    })?;
    assert_ne!(session.session_key_id, session.session_public_key);
    store.create_session(session.clone()).await?;
    store.touch_session(&session.session_id, NOW + 10).await?;
    let touched = store
        .get_session(&session.session_id)
        .await?
        .ok_or("session missing after touch")?;
    assert_eq!(touched.version, 1);
    assert_eq!(touched.last_seen_at, NOW + 10);

    let authority = IdentityAuthorityRecord {
        authority_id: "ida_01".to_owned(),
        principal_id: principal.principal_id.clone(),
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
    store
        .put_identity_authority(authority.clone(), None)
        .await?;
    assert_eq!(
        store.put_identity_authority(authority.clone(), None).await,
        Err(AuthorizationStateError::StorageConflict)
    );
    let mut metadata_only = authority.clone();
    metadata_only.version = 2;
    metadata_only.updated_at = NOW + 1;
    metadata_only.decision = Some(AuthorityDecision {
        decided_at: NOW,
        decided_by: "usr_admin".to_owned(),
        reason: Some("clarified decision text".to_owned()),
    });
    let metadata_only = store.put_identity_authority(metadata_only, Some(1)).await?;
    assert_eq!(metadata_only.version, 1);

    store
        .replace_dependency_evidence(
            evidence_scope(
                AuthorityKind::Identity,
                &authority.authority_id,
                &fixture.binding,
            ),
            vec![fixture.required_dependency.clone()],
        )
        .await?;
    let service = AuthorizationStateService::new(store.clone());
    let target = AuthorityTarget::new(AuthorityKind::Identity, &authority.authority_id)?;
    service.reconcile_authority(&target, NOW + 20).await?;
    let initial = service
        .resolve_issuable_state(&session.session_id, NOW + 20)
        .await?;
    assert_eq!(initial.principal.id, principal.principal_id);
    assert_eq!(
        initial.participant.artifact_digest,
        fixture.binding.artifact_digest
    );
    assert_eq!(
        initial.participant.needs_digest,
        fixture.binding.needs_digest
    );
    assert_eq!(initial.authority_ref.id, authority.authority_id);
    assert_eq!(initial.authority_ref.version, 1);
    assert_eq!(initial.grant_set, fixture.required_grants);
    assert_eq!(initial.materialization_version, 1);
    assert_eq!(initial.session_expires_at, None);
    assert_eq!(initial.effective_authority_expires_at, None);
    assert_issuable_context_valid(&initial)?;
    assert_eq!(store.list_transition_outbox(10).await?.len(), 1);

    let outcome = service.reconcile_authority(&target, NOW + 30).await?;
    assert!(!outcome.changed);
    let unchanged = service
        .resolve_issuable_state(&session.session_id, NOW + 30)
        .await?;
    assert_eq!(unchanged.materialization_version, 1);
    assert_eq!(store.list_transition_outbox(10).await?.len(), 1);

    store
        .replace_dependency_evidence(
            evidence_scope(
                AuthorityKind::Identity,
                &authority.authority_id,
                &fixture.binding,
            ),
            vec![
                fixture.required_dependency.clone(),
                fixture.optional_dependency.clone(),
            ],
        )
        .await?;
    service.reconcile_authority(&target, NOW + 40).await?;
    let expanded = service
        .resolve_issuable_state(&session.session_id, NOW + 40)
        .await?;
    assert_eq!(expanded.grant_set, fixture.all_grants);
    assert_eq!(expanded.materialization_version, 2);
    assert_eq!(store.list_transition_outbox(10).await?.len(), 2);

    store
        .replace_dependency_evidence(
            evidence_scope(
                AuthorityKind::Identity,
                &authority.authority_id,
                &fixture.binding,
            ),
            vec![fixture.optional_dependency.clone()],
        )
        .await?;
    service.reconcile_authority(&target, NOW + 50).await?;
    assert_eq!(
        service
            .resolve_issuable_state(&session.session_id, NOW + 50)
            .await,
        Err(AuthorizationStateError::MaterializationStale)
    );
    let failed = store
        .get_materialized_authority(AuthorityKind::Identity, &authority.authority_id)
        .await?
        .ok_or("missing failed materialization")?;
    assert_eq!(failed.authority.state, MaterializationState::Unavailable);
    assert!(failed
        .authority
        .effective_grant_set
        .permissions()
        .is_empty());
    assert_eq!(failed.authority.materialization_version, 3);
    assert_eq!(
        store
            .list_transition_outbox(10)
            .await?
            .last()
            .map(|value| value.transition.kind),
        Some(AuthorizationTransitionKind::MaterializedUnavailable)
    );

    let mut revoked_authority = authority.clone();
    revoked_authority.state = AuthorityState::Revoked;
    revoked_authority.version = 2;
    revoked_authority.updated_at = NOW + 55;
    revoked_authority.decision = Some(AuthorityDecision {
        decided_at: NOW + 55,
        decided_by: "usr_admin".to_owned(),
        reason: Some("authority revoked".to_owned()),
    });
    store
        .put_identity_authority(revoked_authority, Some(1))
        .await?;
    service.reconcile_authority(&target, NOW + 56).await?;
    assert_eq!(
        service
            .resolve_issuable_state(&session.session_id, NOW + 56)
            .await,
        Err(AuthorizationStateError::AuthorityRevoked)
    );
    let revoked_materialization = store
        .get_materialized_authority(AuthorityKind::Identity, &authority.authority_id)
        .await?
        .ok_or("missing revoked materialization")?;
    assert_eq!(revoked_materialization.authority.authority_version, 2);
    assert_eq!(revoked_materialization.authority.materialization_version, 4);
    assert!(revoked_materialization
        .authority
        .effective_grant_set
        .permissions()
        .is_empty());
    assert_eq!(store.list_transition_outbox(10).await?.len(), 4);

    let revoked = store
        .revoke_session(&session.session_id, 1, NOW + 60)
        .await?;
    assert_eq!(revoked.version, 2);
    assert_eq!(revoked.state, SessionState::Revoked);
    assert_eq!(
        store.touch_session(&session.session_id, NOW + 70).await,
        Err(AuthorizationStateError::SessionRevoked)
    );
    assert_eq!(
        store
            .revoke_session(&session.session_id, 1, NOW + 80,)
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );

    let disabled = store
        .update_principal_authorization_state(
            &principal.principal_id,
            1,
            PrincipalAuthorizationChange {
                state: PrincipalState::Disabled,
                changed_at: NOW + 90,
            },
        )
        .await?;
    assert_eq!(disabled.version, 2);
    assert_eq!(disabled.disabled_at, Some(NOW + 90));
    assert_eq!(
        store
            .update_principal_authorization_state(
                &principal.principal_id,
                1,
                PrincipalAuthorizationChange {
                    state: PrincipalState::Revoked,
                    changed_at: NOW + 100,
                },
            )
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );
    Ok(store)
}

async fn exercise_deployed_principals<S>(store: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: PrincipalRepository
        + SessionRepository
        + ParticipantBindingRepository
        + IdentityAuthorityRepository
        + DeploymentAuthorityRepository
        + EvidenceRepository
        + AuthorizationMaterializationRepository
        + Clone,
{
    let service_fixture = participant_fixture_for(ParticipantKindV1::Service, "example.service")?;
    store
        .put_participant_binding(service_fixture.binding.clone())
        .await?;
    let service_principal = PrincipalRecord {
        principal_id: "svc_01".to_owned(),
        kind: PrincipalKind::Service,
        state: PrincipalState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
        disabled_at: None,
        revoked_at: None,
    };
    store.create_principal(service_principal.clone()).await?;
    let service_session = SessionRecord::from_new(NewSession {
        session_id: "ses_service".to_owned(),
        principal_id: service_principal.principal_id.clone(),
        principal_kind: PrincipalKind::Service,
        participant_id: service_fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKindV1::Service,
        participant_artifact_digest: service_fixture.binding.artifact_digest.clone(),
        participant_needs_digest: service_fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(9),
        inbox_prefix: "_INBOX.service".to_owned(),
        created_at: NOW,
        expires_at: Some(NOW + 1_000),
    })?;
    store.create_session(service_session.clone()).await?;
    store
        .put_deployment_authority(
            DeploymentAuthorityRecord {
                authority_id: "dpa_service".to_owned(),
                deployment_id: "dep_service".to_owned(),
                participant_id: service_fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKindV1::Service,
                participant_artifact_digest: service_fixture.binding.artifact_digest.clone(),
                accepted_needs_digest: service_fixture.binding.needs_digest.clone(),
                desired_grant_set: service_fixture.all_grants.clone(),
                desired_capabilities: Vec::new(),
                state: AuthorityState::Accepted,
                version: 1,
                created_at: NOW,
                updated_at: NOW,
                expires_at: Some(NOW + 900),
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
            deployment_id: "dep_service".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKindV1::Service,
            active: true,
            expires_at: Some(NOW + 800),
        })
        .await?;
    store
        .put_runtime_instance(RuntimeInstanceRecord {
            instance_id: "inst_service".to_owned(),
            deployment_id: "dep_service".to_owned(),
            principal_id: service_principal.principal_id.clone(),
            state: RuntimeInstanceState::Active,
        })
        .await?;
    store
        .put_session_runtime_binding(SessionRuntimeBinding {
            session_id: service_session.session_id.clone(),
            deployment_id: "dep_service".to_owned(),
            instance_id: "inst_service".to_owned(),
        })
        .await?;
    store
        .replace_dependency_evidence(
            evidence_scope(
                AuthorityKind::Deployment,
                "dpa_service",
                &service_fixture.binding,
            ),
            vec![service_fixture.required_dependency.clone()],
        )
        .await?;
    let facade = AuthorizationStateService::new(store.clone());
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_service")?,
            NOW + 10,
        )
        .await?;
    let service_state = facade
        .resolve_issuable_state(&service_session.session_id, NOW + 10)
        .await?;
    assert_eq!(service_state.deployment_id.as_deref(), Some("dep_service"));
    assert_eq!(service_state.instance_id.as_deref(), Some("inst_service"));
    assert_eq!(
        service_state.effective_authority_expires_at,
        Some(NOW + 800)
    );
    assert_issuable_context_valid(&service_state)?;
    assert_eq!(
        facade
            .resolve_issuable_state(&service_session.session_id, NOW + 800)
            .await,
        Err(AuthorizationStateError::DeploymentInactive)
    );

    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_service".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKindV1::Service,
            active: true,
            expires_at: None,
        })
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&service_session.session_id, NOW + 20)
            .await,
        Err(AuthorizationStateError::MaterializationStale)
    );
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_service")?,
            NOW + 20,
        )
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&service_session.session_id, NOW + 20)
            .await?
            .effective_authority_expires_at,
        Some(NOW + 900)
    );

    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_service".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKindV1::Service,
            active: true,
            expires_at: Some(NOW + 800),
        })
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&service_session.session_id, NOW + 30)
            .await,
        Err(AuthorizationStateError::MaterializationStale)
    );
    let mut service_authority = store
        .get_deployment_authority("dep_service", &service_fixture.binding.participant_id)
        .await?
        .ok_or("missing service authority")?;
    service_authority.expires_at = None;
    service_authority.updated_at = NOW + 30;
    service_authority.version = 2;
    store
        .put_deployment_authority(service_authority, Some(1))
        .await?;
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_service")?,
            NOW + 30,
        )
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&service_session.session_id, NOW + 30)
            .await?
            .effective_authority_expires_at,
        Some(NOW + 800)
    );

    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_service".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKindV1::Service,
            active: false,
            expires_at: Some(NOW + 700),
        })
        .await?;
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_service")?,
            NOW + 40,
        )
        .await?;
    let unavailable = store
        .get_materialized_authority(AuthorityKind::Deployment, "dpa_service")
        .await?
        .ok_or("missing unavailable service materialization")?;
    assert_eq!(
        unavailable.authority.state,
        MaterializationState::Unavailable
    );
    assert_eq!(unavailable.authority.expires_at, Some(NOW + 700));
    let unavailable_version = unavailable.authority.materialization_version;
    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_service".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKindV1::Service,
            active: false,
            expires_at: Some(NOW + 600),
        })
        .await?;
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_service")?,
            NOW + 50,
        )
        .await?;
    let unavailable = store
        .get_materialized_authority(AuthorityKind::Deployment, "dpa_service")
        .await?
        .ok_or("missing updated unavailable service materialization")?;
    assert_eq!(unavailable.authority.expires_at, Some(NOW + 600));
    assert_eq!(
        unavailable.authority.materialization_version,
        unavailable_version + 1
    );

    let device_fixture = participant_fixture_for(ParticipantKindV1::Device, "example.device")?;
    store
        .put_participant_binding(device_fixture.binding.clone())
        .await?;
    let device_principal = PrincipalRecord {
        principal_id: "dev_01".to_owned(),
        kind: PrincipalKind::Device,
        state: PrincipalState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
        disabled_at: None,
        revoked_at: None,
    };
    store.create_principal(device_principal.clone()).await?;
    let device_session = SessionRecord::from_new(NewSession {
        session_id: "ses_device".to_owned(),
        principal_id: device_principal.principal_id.clone(),
        principal_kind: PrincipalKind::Device,
        participant_id: device_fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKindV1::Device,
        participant_artifact_digest: device_fixture.binding.artifact_digest.clone(),
        participant_needs_digest: device_fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(10),
        inbox_prefix: "_INBOX.device".to_owned(),
        created_at: NOW,
        expires_at: None,
    })?;
    store.create_session(device_session.clone()).await?;
    store
        .put_deployment_authority(
            DeploymentAuthorityRecord {
                authority_id: "dpa_device".to_owned(),
                deployment_id: "dep_device".to_owned(),
                participant_id: device_fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKindV1::Device,
                participant_artifact_digest: device_fixture.binding.artifact_digest.clone(),
                accepted_needs_digest: device_fixture.binding.needs_digest.clone(),
                desired_grant_set: device_fixture.all_grants.clone(),
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
            deployment_id: "dep_device".to_owned(),
            participant_id: device_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKindV1::Device,
            active: true,
            expires_at: Some(NOW + 750),
        })
        .await?;
    store
        .put_runtime_instance(RuntimeInstanceRecord {
            instance_id: "inst_device".to_owned(),
            deployment_id: "dep_device".to_owned(),
            principal_id: device_principal.principal_id.clone(),
            state: RuntimeInstanceState::Active,
        })
        .await?;
    store
        .put_device(DeviceRecord {
            principal_id: device_principal.principal_id.clone(),
            deployment_id: "dep_device".to_owned(),
            state: DeviceState::Active,
        })
        .await?;
    store
        .put_device_delegation(DeviceDelegationRecord {
            principal_id: device_principal.principal_id.clone(),
            deployment_id: "dep_device".to_owned(),
            required: true,
            state: DeviceDelegationState::Active,
            expires_at: Some(NOW + 700),
        })
        .await?;
    store
        .put_session_runtime_binding(SessionRuntimeBinding {
            session_id: device_session.session_id.clone(),
            deployment_id: "dep_device".to_owned(),
            instance_id: "inst_device".to_owned(),
        })
        .await?;
    store
        .replace_dependency_evidence(
            evidence_scope(
                AuthorityKind::Deployment,
                "dpa_device",
                &device_fixture.binding,
            ),
            vec![device_fixture.required_dependency.clone()],
        )
        .await?;
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_device")?,
            NOW + 10,
        )
        .await?;
    let device_state = facade
        .resolve_issuable_state(&device_session.session_id, NOW + 10)
        .await?;
    assert_eq!(device_state.deployment_id.as_deref(), Some("dep_device"));
    assert_eq!(device_state.instance_id.as_deref(), Some("inst_device"));
    assert_eq!(device_state.delegation_expires_at, Some(NOW + 700));
    assert_issuable_context_valid(&device_state)?;

    store
        .put_device(DeviceRecord {
            principal_id: device_principal.principal_id.clone(),
            deployment_id: "dep_device".to_owned(),
            state: DeviceState::Disabled,
        })
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&device_session.session_id, NOW + 20)
            .await,
        Err(AuthorizationStateError::DeviceInactive)
    );
    store
        .put_device(DeviceRecord {
            principal_id: device_principal.principal_id.clone(),
            deployment_id: "dep_device".to_owned(),
            state: DeviceState::Active,
        })
        .await?;

    store
        .put_device_delegation(DeviceDelegationRecord {
            principal_id: device_principal.principal_id.clone(),
            deployment_id: "dep_device".to_owned(),
            required: true,
            state: DeviceDelegationState::Missing,
            expires_at: Some(NOW + 700),
        })
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&device_session.session_id, NOW + 20)
            .await,
        Err(AuthorizationStateError::ActivationMissing)
    );
    Ok(())
}

async fn exercise_runtime_evidence_entities<S>(store: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: PrincipalRepository
        + SessionRepository
        + ParticipantBindingRepository
        + EvidenceRepository
        + Clone,
{
    let fixture = participant_fixture_for(ParticipantKindV1::Device, "shared.device")?;
    store
        .put_participant_binding(fixture.binding.clone())
        .await?;
    for principal_id in ["dev_shared", "dev_other"] {
        store
            .create_principal(PrincipalRecord {
                principal_id: principal_id.to_owned(),
                kind: PrincipalKind::Device,
                state: PrincipalState::Active,
                created_at: NOW,
                updated_at: NOW,
                version: 1,
                disabled_at: None,
                revoked_at: None,
            })
            .await?;
    }
    for (session_id, seed) in [("ses_shared_a", 51_u8), ("ses_shared_b", 52_u8)] {
        store
            .create_session(SessionRecord::from_new(NewSession {
                session_id: session_id.to_owned(),
                principal_id: "dev_shared".to_owned(),
                principal_kind: PrincipalKind::Device,
                participant_id: fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKindV1::Device,
                participant_artifact_digest: fixture.binding.artifact_digest.clone(),
                participant_needs_digest: fixture.binding.needs_digest.clone(),
                session_public_key: session_public_key(seed),
                inbox_prefix: format!("_INBOX.{session_id}"),
                created_at: NOW,
                expires_at: None,
            })?)
            .await?;
    }
    for deployment_id in ["dep_shared_a", "dep_shared_b"] {
        store
            .put_deployment_evidence(DeploymentRecord {
                deployment_id: deployment_id.to_owned(),
                participant_id: fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKindV1::Device,
                active: true,
                expires_at: None,
            })
            .await?;
        store
            .put_device(DeviceRecord {
                principal_id: "dev_shared".to_owned(),
                deployment_id: deployment_id.to_owned(),
                state: DeviceState::Active,
            })
            .await?;
    }

    let instance = RuntimeInstanceRecord {
        instance_id: "inst_shared".to_owned(),
        deployment_id: "dep_shared_a".to_owned(),
        principal_id: "dev_shared".to_owned(),
        state: RuntimeInstanceState::Active,
    };
    store.put_runtime_instance(instance.clone()).await?;
    assert_eq!(
        store.get_runtime_instance(&instance.instance_id).await?,
        Some(instance.clone())
    );

    let device = DeviceRecord {
        principal_id: "dev_shared".to_owned(),
        deployment_id: "dep_shared_a".to_owned(),
        state: DeviceState::Active,
    };
    assert_eq!(
        store
            .get_device(&device.principal_id, &device.deployment_id)
            .await?,
        Some(device)
    );

    let delegation_a = DeviceDelegationRecord {
        principal_id: "dev_shared".to_owned(),
        deployment_id: "dep_shared_a".to_owned(),
        required: true,
        state: DeviceDelegationState::Active,
        expires_at: Some(NOW + 500),
    };
    store.put_device_delegation(delegation_a.clone()).await?;
    assert_eq!(
        store
            .get_device_delegation("dev_shared", "dep_shared_a")
            .await?,
        Some(delegation_a.clone())
    );

    for session_id in ["ses_shared_a", "ses_shared_b"] {
        let binding = SessionRuntimeBinding {
            session_id: session_id.to_owned(),
            deployment_id: "dep_shared_a".to_owned(),
            instance_id: "inst_shared".to_owned(),
        };
        store.put_session_runtime_binding(binding.clone()).await?;
        assert_eq!(
            store.get_session_runtime_binding(session_id).await?,
            Some(binding)
        );
    }
    let expected = RuntimeEvidence::Device(DeviceEvidence {
        deployment_id: "dep_shared_a".to_owned(),
        instance_id: "inst_shared".to_owned(),
        device_active: true,
        instance_active: true,
        delegation: Some(DelegationEvidence {
            active: true,
            required: true,
            expires_at: Some(NOW + 500),
        }),
    });
    assert_eq!(
        store.get_runtime_evidence("ses_shared_a").await?,
        Some(expected.clone())
    );
    assert_eq!(
        store.get_runtime_evidence("ses_shared_b").await?,
        Some(expected)
    );

    let mut disabled_instance = instance.clone();
    disabled_instance.state = RuntimeInstanceState::Disabled;
    store
        .put_runtime_instance(disabled_instance.clone())
        .await?;
    for session_id in ["ses_shared_a", "ses_shared_b"] {
        let RuntimeEvidence::Device(evidence) = store
            .get_runtime_evidence(session_id)
            .await?
            .ok_or("missing shared runtime evidence")?
        else {
            return Err("unexpected runtime evidence kind".into());
        };
        assert!(!evidence.instance_active);
    }
    store.put_runtime_instance(instance.clone()).await?;

    let mut reassigned_deployment = instance.clone();
    reassigned_deployment.deployment_id = "dep_shared_b".to_owned();
    assert_eq!(
        store.put_runtime_instance(reassigned_deployment).await,
        Err(AuthorizationStateError::InvalidRecord(
            "runtime instance identity cannot change".to_owned()
        ))
    );
    let mut reassigned_principal = instance.clone();
    reassigned_principal.principal_id = "dev_other".to_owned();
    assert_eq!(
        store.put_runtime_instance(reassigned_principal).await,
        Err(AuthorizationStateError::InvalidRecord(
            "runtime instance identity cannot change".to_owned()
        ))
    );

    let delegation_b = DeviceDelegationRecord {
        principal_id: "dev_shared".to_owned(),
        deployment_id: "dep_shared_b".to_owned(),
        required: true,
        state: DeviceDelegationState::Revoked,
        expires_at: None,
    };
    store.put_device_delegation(delegation_b.clone()).await?;
    assert_eq!(
        store
            .get_device_delegation("dev_shared", "dep_shared_a")
            .await?,
        Some(delegation_a.clone())
    );
    assert_eq!(
        store
            .get_device_delegation("dev_shared", "dep_shared_b")
            .await?,
        Some(delegation_b)
    );

    assert!(serde_json::from_value::<SessionRuntimeBinding>(json!({
        "sessionId": "ses_shared_a",
        "deploymentId": "dep_shared_a",
        "instanceId": null,
    }))
    .is_err());

    store.remove_session_runtime_binding("ses_shared_b").await?;
    assert_eq!(store.get_runtime_evidence("ses_shared_b").await?, None);
    assert_eq!(
        store.get_runtime_instance("inst_shared").await?,
        Some(instance)
    );
    assert_eq!(
        store.get_device("dev_shared", "dep_shared_a").await?,
        Some(DeviceRecord {
            principal_id: "dev_shared".to_owned(),
            deployment_id: "dep_shared_a".to_owned(),
            state: DeviceState::Active,
        })
    );
    assert_eq!(
        store
            .get_device_delegation("dev_shared", "dep_shared_a")
            .await?,
        Some(delegation_a)
    );
    Ok(())
}

async fn exercise_resources<S>(store: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: PrincipalRepository
        + SessionRepository
        + ParticipantBindingRepository
        + IdentityAuthorityRepository
        + DeploymentAuthorityRepository
        + EvidenceRepository
        + AuthorizationMaterializationRepository
        + Clone,
{
    let fixture = participant_fixture_with_resources(ParticipantKindV1::App, "resource.app", true)?;
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
    let session = SessionRecord::from_new(NewSession {
        session_id: "ses_resources".to_owned(),
        principal_id: "usr_resources".to_owned(),
        principal_kind: PrincipalKind::User,
        participant_id: fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKindV1::App,
        participant_artifact_digest: fixture.binding.artifact_digest.clone(),
        participant_needs_digest: fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(11),
        inbox_prefix: "_INBOX.resources".to_owned(),
        created_at: NOW,
        expires_at: None,
    })?;
    store.create_session(session.clone()).await?;
    store
        .put_identity_authority(
            IdentityAuthorityRecord {
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
            },
            None,
        )
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
        provider_identity: "storage:kv-cache".to_owned(),
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
                    provider_identity: "storage:attachments".to_owned(),
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

#[tokio::test]
async fn in_memory_deployments_and_instances_are_isolated() -> Result<(), Box<dyn std::error::Error>>
{
    exercise_deployment_isolation(InMemoryAuthorizationStore::default()).await
}

#[tokio::test]
async fn sqlite_deployments_and_instances_are_isolated() -> Result<(), Box<dyn std::error::Error>> {
    exercise_deployment_isolation(SqliteAuthorizationStore::open_in_memory()?).await
}

async fn exercise_deployment_isolation<S>(store: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: PrincipalRepository
        + SessionRepository
        + ParticipantBindingRepository
        + DeploymentAuthorityRepository
        + EvidenceRepository
        + AuthorizationMaterializationRepository
        + Clone,
{
    let fixture =
        participant_fixture_with_resources(ParticipantKindV1::Service, "shared.service", true)?;
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
            .create_session(SessionRecord::from_new(NewSession {
                session_id: session_id.clone(),
                principal_id,
                principal_kind: PrincipalKind::Service,
                participant_id: fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKindV1::Service,
                participant_artifact_digest: fixture.binding.artifact_digest.clone(),
                participant_needs_digest: fixture.binding.needs_digest.clone(),
                session_public_key: session_public_key(key_seed),
                inbox_prefix: format!("_INBOX.{suffix}"),
                created_at: NOW,
                expires_at: None,
            })?)
            .await?;
        store
            .put_deployment_authority(
                DeploymentAuthorityRecord {
                    authority_id: authority_id.clone(),
                    deployment_id: deployment_id.clone(),
                    participant_id: fixture.binding.participant_id.clone(),
                    participant_kind: ParticipantKindV1::Service,
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
                participant_kind: ParticipantKindV1::Service,
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
            })
            .await?;
        store
            .put_session_runtime_binding(SessionRuntimeBinding {
                session_id: session_id.clone(),
                deployment_id: deployment_id.clone(),
                instance_id,
            })
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
                    provider_identity: format!("storage_{suffix}"),
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
    assert_eq!(first.resources[0].provider_identity, "storage_a");
    assert_eq!(second.resources[0].provider_identity, "storage_b");
    assert_eq!(
        store
            .put_deployment_evidence(DeploymentRecord {
                deployment_id: "dep_a".to_owned(),
                participant_id: "different.service".to_owned(),
                participant_kind: ParticipantKindV1::Service,
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
async fn typed_authority_ids_do_not_collide_in_memory() -> Result<(), Box<dyn std::error::Error>> {
    exercise_typed_authority_ids(InMemoryAuthorizationStore::default()).await
}

#[tokio::test]
async fn typed_authority_ids_do_not_collide_in_sqlite() -> Result<(), Box<dyn std::error::Error>> {
    exercise_typed_authority_ids(SqliteAuthorizationStore::open_in_memory()?).await
}

async fn exercise_typed_authority_ids<S>(store: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: PrincipalRepository
        + ProviderIdentityRepository
        + SessionRepository
        + ParticipantBindingRepository
        + IdentityAuthorityRepository
        + DeploymentAuthorityRepository
        + EvidenceRepository
        + AuthorizationMaterializationRepository
        + Clone,
{
    let store = exercise_store(store).await?;
    let fixture = participant_fixture_for(ParticipantKindV1::Service, "collision.service")?;
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
        participant_kind: ParticipantKindV1::Service,
        participant_artifact_digest: fixture.binding.artifact_digest.clone(),
        participant_needs_digest: fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(23),
        inbox_prefix: "_INBOX.collision".to_owned(),
        created_at: NOW,
        expires_at: None,
    })?;
    store.create_session(session.clone()).await?;
    store
        .put_deployment_authority(
            DeploymentAuthorityRecord {
                authority_id: "ida_01".to_owned(),
                deployment_id: "dep_collision".to_owned(),
                participant_id: fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKindV1::Service,
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
            participant_kind: ParticipantKindV1::Service,
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
        })
        .await?;
    store
        .put_session_runtime_binding(SessionRuntimeBinding {
            session_id: session.session_id.clone(),
            deployment_id: "dep_collision".to_owned(),
            instance_id: "inst_collision".to_owned(),
        })
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

#[tokio::test]
async fn sqlite_transition_outbox_survives_failed_delivery_and_restart(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("authorization.sqlite");
    let store = exercise_store(SqliteAuthorizationStore::open_path(&path)?).await?;
    let pending = store.list_transition_outbox(100).await?;
    assert!(!pending.is_empty());
    let first = pending[0].clone();
    drop(store);

    let restarted = SqliteAuthorizationStore::open_path(&path)?;
    assert_eq!(restarted.list_transition_outbox(100).await?, pending);
    restarted.acknowledge_transition(&first.event_id).await?;
    drop(restarted);

    let drained = SqliteAuthorizationStore::open_path(&path)?;
    assert!(!drained
        .list_transition_outbox(100)
        .await?
        .iter()
        .any(|record| record.event_id == first.event_id));
    Ok(())
}

#[test]
fn protocol_integer_boundaries_are_enforced() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = participant_fixture()?;
    let mut authority = IdentityAuthorityRecord {
        authority_id: "ida_boundary".to_owned(),
        principal_id: "usr_boundary".to_owned(),
        participant_id: fixture.binding.participant_id,
        participant_artifact_digest: fixture.binding.artifact_digest,
        accepted_needs_digest: fixture.binding.needs_digest,
        desired_grant_set: fixture.all_grants,
        desired_capabilities: Vec::new(),
        state: AuthorityState::Accepted,
        version: MAX_PROTOCOL_INTEGER,
        created_at: MAX_PROTOCOL_INTEGER as i64,
        updated_at: MAX_PROTOCOL_INTEGER as i64,
        expires_at: Some(MAX_PROTOCOL_INTEGER as i64),
        decision: Some(AuthorityDecision {
            decided_at: MAX_PROTOCOL_INTEGER as i64,
            decided_by: "usr_admin".to_owned(),
            reason: None,
        }),
    };
    super::repository::validate_identity_authority(&mut authority)?;
    authority.version = MAX_PROTOCOL_INTEGER + 1;
    assert!(matches!(
        super::repository::validate_identity_authority(&mut authority),
        Err(AuthorizationStateError::InvalidRecord(_))
    ));
    let too_large = (MAX_PROTOCOL_INTEGER + 1) as i64;
    assert!(matches!(
        SessionRecord::from_new(NewSession {
            session_id: "ses_boundary".to_owned(),
            principal_id: "usr_boundary".to_owned(),
            principal_kind: PrincipalKind::User,
            participant_id: "example.app".to_owned(),
            participant_kind: ParticipantKindV1::App,
            participant_artifact_digest: digest(1),
            participant_needs_digest: digest(2),
            session_public_key: session_public_key(31),
            inbox_prefix: "_INBOX.boundary".to_owned(),
            created_at: too_large,
            expires_at: None,
        }),
        Err(AuthorizationStateError::InvalidRecord(_))
    ));
    Ok(())
}

#[tokio::test]
async fn invalid_session_relationship_errors_match_backends(
) -> Result<(), Box<dyn std::error::Error>> {
    let memory = invalid_session_relationship_errors(InMemoryAuthorizationStore::default()).await?;
    let sqlite =
        invalid_session_relationship_errors(SqliteAuthorizationStore::open_in_memory()?).await?;
    assert_eq!(memory, sqlite);
    assert_eq!(
        memory,
        vec![
            AuthorizationStateError::InvalidRecord(
                "session principal kind does not match principal".to_owned()
            ),
            AuthorizationStateError::InvalidRecord(
                "session participant kind does not match participant binding".to_owned()
            ),
            AuthorizationStateError::NeedsDigestMismatch,
        ]
    );
    Ok(())
}

async fn invalid_session_relationship_errors<S>(
    store: S,
) -> Result<Vec<AuthorizationStateError>, Box<dyn std::error::Error>>
where
    S: PrincipalRepository + SessionRepository + ParticipantBindingRepository,
{
    let fixture = participant_fixture()?;
    store
        .put_participant_binding(fixture.binding.clone())
        .await?;
    store
        .create_principal(PrincipalRecord {
            principal_id: "svc_invalid_relationship".to_owned(),
            kind: PrincipalKind::Service,
            state: PrincipalState::Active,
            created_at: NOW,
            updated_at: NOW,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        })
        .await?;
    let base = SessionRecord::from_new(NewSession {
        session_id: "ses_invalid_relationship".to_owned(),
        principal_id: "svc_invalid_relationship".to_owned(),
        principal_kind: PrincipalKind::User,
        participant_id: fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKindV1::App,
        participant_artifact_digest: fixture.binding.artifact_digest.clone(),
        participant_needs_digest: fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(32),
        inbox_prefix: "_INBOX.invalid-relationship".to_owned(),
        created_at: NOW,
        expires_at: None,
    })?;
    let mut errors = Vec::new();
    errors.push(
        store
            .create_session(base.clone())
            .await
            .expect_err("kind mismatch"),
    );

    store
        .create_principal(PrincipalRecord {
            principal_id: "usr_invalid_relationship".to_owned(),
            kind: PrincipalKind::User,
            state: PrincipalState::Active,
            created_at: NOW,
            updated_at: NOW,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        })
        .await?;
    let mut participant_kind = base.clone();
    participant_kind.session_id = "ses_invalid_participant_kind".to_owned();
    participant_kind.principal_id = "usr_invalid_relationship".to_owned();
    participant_kind.participant_kind = ParticipantKindV1::Agent;
    participant_kind.session_public_key = session_public_key(33);
    participant_kind.session_key_id = digest_key(&participant_kind.session_public_key)?;
    errors.push(
        store
            .create_session(participant_kind)
            .await
            .expect_err("participant kind mismatch"),
    );
    let mut needs = base;
    needs.session_id = "ses_invalid_needs".to_owned();
    needs.principal_id = "usr_invalid_relationship".to_owned();
    needs.participant_needs_digest = digest(99);
    needs.session_public_key = session_public_key(34);
    needs.session_key_id = digest_key(&needs.session_public_key)?;
    errors.push(
        store
            .create_session(needs)
            .await
            .expect_err("needs mismatch"),
    );
    Ok(errors)
}

fn digest_key(public_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};

    Ok(URL_SAFE_NO_PAD.encode(Sha256::digest(URL_SAFE_NO_PAD.decode(public_key)?)))
}

#[tokio::test]
async fn in_memory_session_denials_do_not_rewrite_shared_authority(
) -> Result<(), Box<dyn std::error::Error>> {
    exercise_session_denials(InMemoryAuthorizationStore::default()).await
}

#[tokio::test]
async fn sqlite_session_denials_do_not_rewrite_shared_authority(
) -> Result<(), Box<dyn std::error::Error>> {
    exercise_session_denials(SqliteAuthorizationStore::open_in_memory()?).await
}

async fn exercise_session_denials<S>(store: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: PrincipalRepository
        + SessionRepository
        + ParticipantBindingRepository
        + IdentityAuthorityRepository
        + EvidenceRepository
        + AuthorizationMaterializationRepository
        + Clone,
{
    let fixture = participant_fixture()?;
    store
        .put_participant_binding(fixture.binding.clone())
        .await?;
    store
        .create_principal(PrincipalRecord {
            principal_id: "usr_session_denials".to_owned(),
            kind: PrincipalKind::User,
            state: PrincipalState::Active,
            created_at: NOW,
            updated_at: NOW,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        })
        .await?;
    for (session_id, seed) in [("ses_denied", 41_u8), ("ses_healthy", 42_u8)] {
        store
            .create_session(SessionRecord::from_new(NewSession {
                session_id: session_id.to_owned(),
                principal_id: "usr_session_denials".to_owned(),
                principal_kind: PrincipalKind::User,
                participant_id: fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKindV1::App,
                participant_artifact_digest: fixture.binding.artifact_digest.clone(),
                participant_needs_digest: fixture.binding.needs_digest.clone(),
                session_public_key: session_public_key(seed),
                inbox_prefix: format!("_INBOX.{session_id}"),
                created_at: NOW,
                expires_at: None,
            })?)
            .await?;
    }
    store
        .put_identity_authority(
            IdentityAuthorityRecord {
                authority_id: "ida_session_denials".to_owned(),
                principal_id: "usr_session_denials".to_owned(),
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
            },
            None,
        )
        .await?;
    store
        .replace_dependency_evidence(
            evidence_scope(
                AuthorityKind::Identity,
                "ida_session_denials",
                &fixture.binding,
            ),
            vec![fixture.required_dependency],
        )
        .await?;
    let facade = AuthorizationStateService::new(store.clone());
    let target = AuthorityTarget::new(AuthorityKind::Identity, "ida_session_denials")?;
    facade.reconcile_authority(&target, NOW + 1).await?;
    let initial = store
        .get_materialized_authority(AuthorityKind::Identity, "ida_session_denials")
        .await?
        .ok_or("missing initial materialization")?;

    store.revoke_session("ses_denied", 1, NOW + 2).await?;
    assert_eq!(
        facade.resolve_issuable_state("ses_denied", NOW + 3).await,
        Err(AuthorizationStateError::SessionRevoked)
    );
    assert!(facade
        .resolve_issuable_state("ses_healthy", NOW + 3)
        .await
        .is_ok());
    let after_session_denial = store
        .get_materialized_authority(AuthorityKind::Identity, "ida_session_denials")
        .await?
        .ok_or("missing materialization after session denial")?;
    assert_eq!(after_session_denial, initial);

    store
        .update_principal_authorization_state(
            "usr_session_denials",
            1,
            PrincipalAuthorizationChange {
                state: PrincipalState::Disabled,
                changed_at: NOW + 4,
            },
        )
        .await?;
    assert_eq!(
        facade.resolve_issuable_state("ses_healthy", NOW + 5).await,
        Err(AuthorizationStateError::PrincipalInactive)
    );
    facade.reconcile_authority(&target, NOW + 5).await?;
    let unavailable = store
        .get_materialized_authority(AuthorityKind::Identity, "ida_session_denials")
        .await?
        .ok_or("missing invalidated materialization")?;
    assert_eq!(
        unavailable.authority.state,
        MaterializationState::Unavailable
    );
    assert!(unavailable
        .authority
        .effective_grant_set
        .permissions()
        .is_empty());
    Ok(())
}

struct ParticipantFixture {
    binding: ParticipantBindingRecord,
    required_grants: GrantSetV1,
    all_grants: GrantSetV1,
    required_dependency: DependencyEvidence,
    optional_dependency: DependencyEvidence,
}

fn participant_fixture() -> Result<ParticipantFixture, Box<dyn std::error::Error>> {
    participant_fixture_for(ParticipantKindV1::App, "example.app")
}

fn participant_fixture_for(
    kind: ParticipantKindV1,
    participant_id: &str,
) -> Result<ParticipantFixture, Box<dyn std::error::Error>> {
    participant_fixture_with_resources(kind, participant_id, false)
}

fn participant_fixture_with_resources(
    kind: ParticipantKindV1,
    participant_id: &str,
    include_resources: bool,
) -> Result<ParticipantFixture, Box<dyn std::error::Error>> {
    let required_api = parse_api_v1(&json!({
        "format": "trellis.api.v1",
        "id": "required.api@v1",
        "displayName": "Required API",
        "description": "Required API fixture.",
        "schemas": { "Input": true, "Output": true },
        "rpc": {
            "Required.Get": {
                "version": "v1",
                "input": { "schema": "Input" },
                "output": { "schema": "Output" }
            }
        }
    }))?;
    let optional_api = parse_api_v1(&json!({
        "format": "trellis.api.v1",
        "id": "optional.api@v1",
        "displayName": "Optional API",
        "description": "Optional API fixture.",
        "schemas": { "Input": true, "Output": true },
        "rpc": {
            "Optional.Get": {
                "version": "v1",
                "input": { "schema": "Input" },
                "output": { "schema": "Output" }
            }
        }
    }))?;
    let required_digest = required_api.digest()?;
    let optional_digest = optional_api.digest()?;
    let kind_name = match kind {
        ParticipantKindV1::Service => "service",
        ParticipantKindV1::App => "app",
        ParticipantKindV1::Device => "device",
        ParticipantKindV1::Agent => "agent",
    };
    let mut participant_value = json!({
        "format": "trellis.participant.v1",
        "id": participant_id,
        "displayName": "Example App",
        "description": "Authorization materialization fixture.",
        "kind": kind_name,
        "uses": {
            "required": {
                "requiredApi": {
                    "api": "required.api@v1",
                    "apiDigest": required_digest,
                    "rpc": { "call": ["Required.Get"] }
                }
            },
            "optional": {
                "optionalApi": {
                    "api": "optional.api@v1",
                    "apiDigest": optional_digest,
                    "rpc": { "call": ["Optional.Get"] }
                }
            }
        }
    });
    if include_resources {
        participant_value["schemas"] = json!({ "CacheValue": true });
        participant_value["resources"] = json!({
            "kv": {
                "cache": {
                    "purpose": "Required cache storage.",
                    "schema": { "schema": "CacheValue" },
                    "required": true
                }
            },
            "store": {
                "attachments": {
                    "purpose": "Optional attachment storage.",
                    "required": false
                }
            }
        });
    }
    let participant = parse_participant_v1(&participant_value)?;
    let mut apis = BTreeMap::new();
    apis.insert(required_api.id().to_owned(), required_api.clone());
    apis.insert(optional_api.id().to_owned(), optional_api.clone());
    let resolved = resolve_participant_v1(&participant, &apis)?;
    let required_grants = resolved.needs().required().grant_set().clone();
    let all_grants = GrantSetV1::new(
        required_grants
            .permissions()
            .iter()
            .chain(resolved.needs().optional().grant_set().permissions())
            .cloned()
            .collect(),
    );
    let mut api_values = BTreeMap::<String, Value>::new();
    for (id, api) in &apis {
        api_values.insert(id.clone(), api.normalized_value()?);
    }
    let binding = ParticipantBindingRecord {
        participant_id: participant.id().to_owned(),
        participant_kind: participant.kind(),
        artifact_digest: participant.digest()?,
        needs_digest: resolved.needs().digest()?,
        participant_json: participant.canonical_json()?,
        api_artifacts_json: serde_json::to_string(&api_values)?,
        resolved_at: NOW,
        state: ParticipantBindingState::Resolved,
        error: None,
    };
    Ok(ParticipantFixture {
        binding,
        required_grants,
        all_grants,
        required_dependency: DependencyEvidence {
            alias: "requiredApi".to_owned(),
            required: true,
            api_id: required_api.id().to_owned(),
            api_digest: required_digest,
            provider_participant_id: "required.provider".to_owned(),
            provider_deployment_id: Some("required.deployment".to_owned()),
            provider_instance_id: Some("required.instance".to_owned()),
            state: DependencyState::Available,
            observed_at: NOW,
        },
        optional_dependency: DependencyEvidence {
            alias: "optionalApi".to_owned(),
            required: false,
            api_id: optional_api.id().to_owned(),
            api_digest: optional_digest,
            provider_participant_id: "optional.provider".to_owned(),
            provider_deployment_id: Some("optional.deployment".to_owned()),
            provider_instance_id: Some("optional.instance".to_owned()),
            state: DependencyState::Available,
            observed_at: NOW,
        },
    })
}

fn digest(byte: u8) -> String {
    URL_SAFE_NO_PAD.encode([byte; 32])
}

fn evidence_scope(
    kind: AuthorityKind,
    authority_id: &str,
    binding: &ParticipantBindingRecord,
) -> AuthorityEvidenceScope {
    AuthorityEvidenceScope {
        target: AuthorityTarget {
            kind,
            authority_id: authority_id.to_owned(),
        },
        participant_id: binding.participant_id.clone(),
        participant_artifact_digest: binding.artifact_digest.clone(),
        participant_needs_digest: binding.needs_digest.clone(),
    }
}

fn session_public_key(seed: u8) -> String {
    URL_SAFE_NO_PAD.encode(
        SigningKey::from_bytes(&[seed; 32])
            .verifying_key()
            .to_bytes(),
    )
}

fn assert_issuable_context_valid(
    state: &IssuableAuthorizationState,
) -> Result<(), Box<dyn std::error::Error>> {
    authorization_context_signing_digest_v1(&UnsignedAuthorizationContextV1 {
        format: AUTHORIZATION_CONTEXT_FORMAT_V1.to_owned(),
        authority: "test-authority".to_owned(),
        context_id: format!("ctx_{}", state.session_id),
        issuer_key_id: URL_SAFE_NO_PAD.encode([99_u8; 32]),
        session_id: state.session_id.clone(),
        session_key: state.session_public_key.clone(),
        principal: state.principal.clone(),
        participant: state.participant.clone(),
        authority_ref: state.authority_ref.clone(),
        deployment_id: state.deployment_id.clone(),
        instance_id: state.instance_id.clone(),
        inbox_prefix: state.inbox_prefix.clone(),
        issued_at: NOW,
        not_before: NOW,
        expires_at: NOW + 1,
        grant_set: state.grant_set.clone(),
        capabilities: state.capabilities.clone(),
        extensions: serde_json::Map::new(),
        critical: Vec::new(),
    })
    .map(|_| ())
    .map_err(Into::into)
}
