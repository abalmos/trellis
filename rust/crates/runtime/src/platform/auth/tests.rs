use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::SigningKey;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use trellis_protocol::{
    authorization_context_signing_digest_v1, parse_api_v1, parse_participant_v1,
    resolve_participant_v1, GrantSetV1, ParticipantKindV1, UnsignedAuthorizationContextV1,
    AUTHORIZATION_CONTEXT_FORMAT_V1,
};

use super::service_domain::deployment_authority_id;
use super::*;

const NOW: i64 = 1_800_000_000_000;

#[test]
fn deployment_authority_lineage_depends_only_on_deployment_and_participant() {
    let first = deployment_authority_id("dep_a", "participant@v1").unwrap();
    assert_eq!(
        first,
        deployment_authority_id("dep_a", "participant@v1").unwrap()
    );
    assert_ne!(
        first,
        deployment_authority_id("dep_b", "participant@v1").unwrap()
    );
    assert_ne!(first, deployment_authority_id("dep_a", "other@v1").unwrap());
    assert_eq!(
        first,
        deployment_authority_id("dep_a", "participant@v1").unwrap(),
        "artifact revisions, request IDs, timestamps, and principal kind are not lineage inputs",
    );
}

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
async fn in_memory_auth_companion_repository_conformance() -> Result<(), Box<dyn std::error::Error>>
{
    exercise_companion_repositories(InMemoryAuthorizationStore::default()).await
}

#[tokio::test]
async fn sqlite_auth_companion_repository_conformance() -> Result<(), Box<dyn std::error::Error>> {
    exercise_companion_repositories(SqliteAuthorizationStore::open_in_memory()?).await
}

#[tokio::test]
async fn sqlite_initial_deployment_proposal_reuses_after_reopen(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let path = temp_dir.path().join("authority-lineage.sqlite");
    let fixture = participant_fixture_for(ParticipantKindV1::Service, "restart.service")?;
    let participant_artifact = serde_json::from_str(&fixture.binding.participant_json)?;
    let referenced_api_artifacts =
        serde_json::from_str::<BTreeMap<String, Value>>(&fixture.binding.api_artifacts_json)?
            .into_values()
            .collect::<Vec<_>>();
    let proof = |byte: u8| IdempotencyResultRecord {
        scope_key: digest(byte),
        purpose: "deployment.authority.present-restart".to_owned(),
        signer_id: "restart-signer".to_owned(),
        request_id: format!("restart-request-{byte}"),
        request_digest: digest(byte + 1),
        result: Value::Null,
        created_at: NOW,
        expires_at: NOW + 1_000,
    };
    let store = SqliteAuthorizationStore::open_path(&path)?;
    store
        .put_participant_binding(fixture.binding.clone())
        .await?;
    let service = AuthService::new(store, AuthServiceConfig::default())?;
    let input = PresentDeploymentAuthorityInput {
        deployment_id: "dep_restart".to_owned(),
        participant_artifact,
        referenced_api_artifacts,
        created_at: NOW,
        expires_at: Some(NOW + 1_000),
        idempotency: proof(200),
        actions: Vec::new(),
    };
    let first = match service.present_deployment_authority(input.clone()).await? {
        IdempotentOutcome::Applied(proposal) => proposal,
        IdempotentOutcome::Replayed(_) => return Err("initial proposal replayed".into()),
    };
    drop(service);

    let reopened = AuthService::new(
        SqliteAuthorizationStore::open_path(&path)?,
        AuthServiceConfig::default(),
    )?;
    let mut repeated = input;
    repeated.created_at += 1;
    repeated.idempotency = proof(201);
    assert_eq!(
        reopened.present_deployment_authority(repeated).await?,
        IdempotentOutcome::Replayed(json!({ "proposalId": first.proposal_id }))
    );
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
        store
            .create_session(test_session_creation(session, None, None))
            .await,
        Err(AuthorizationStateError::PrincipalMissing)
    );
    Ok(())
}

async fn exercise_store<S>(store: S) -> Result<S, Box<dyn std::error::Error>>
where
    S: PrincipalRepository
        + ProviderIdentityRepository
        + SessionRepository
        + AuthSessionRepository
        + IdempotencyRepository
        + PostCommitActionRepository
        + AuthSessionRepository
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
    let creation_action = PostCommitActionRecord {
        action_id: test_digest("session.create.action:ses_01"),
        kind: PostCommitActionKind::Event,
        payload: json!({ "sessionId": session.session_id }),
        created_at: NOW,
        attempts: 0,
        next_attempt_at: NOW,
        claimed_until: None,
        last_error: None,
    };
    let mut session_creation = test_session_creation(session.clone(), None, None);
    session_creation.actions.push(creation_action.clone());
    assert_eq!(
        store.create_session(session_creation.clone()).await?,
        IdempotentOutcome::Applied(session.clone())
    );
    assert_eq!(
        store.create_session(session_creation.clone()).await?,
        IdempotentOutcome::Replayed(session_creation.idempotency.result.clone())
    );
    let mut bootstrap_idempotency =
        test_session_idempotency("bootstrap.client:ses_01", &session.session_key_id, NOW + 1);
    bootstrap_idempotency.purpose = "bootstrap.client".to_owned();
    bootstrap_idempotency.result = json!({ "sessionId": session.session_id });
    let admission = ClientBootstrapAdmission {
        session_id: session.session_id.clone(),
        observed_at: NOW + 1,
        idempotency: bootstrap_idempotency.clone(),
    };
    assert!(matches!(
        store.admit_client_bootstrap(admission.clone()).await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store.admit_client_bootstrap(admission.clone()).await?,
        IdempotentOutcome::Replayed(bootstrap_idempotency.result.clone())
    );
    let mut conflicting_admission = admission;
    conflicting_admission.idempotency.request_digest = digest(97);
    assert_eq!(
        store.admit_client_bootstrap(conflicting_admission).await,
        Err(AuthorizationStateError::StorageConflict)
    );
    let mut mismatched_creation = session_creation;
    mismatched_creation.idempotency.request_digest = digest(98);
    assert_eq!(
        store.create_session(mismatched_creation).await,
        Err(AuthorizationStateError::StorageConflict)
    );
    assert_eq!(
        store
            .list_ready_post_commit_actions(NOW, 100)
            .await?
            .iter()
            .filter(|action| action.action_id == creation_action.action_id)
            .count(),
        1
    );
    let rollback_session = SessionRecord::from_new(NewSession {
        session_id: "ses_create_rollback".to_owned(),
        principal_id: principal.principal_id.clone(),
        principal_kind: PrincipalKind::User,
        participant_id: fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKindV1::App,
        participant_artifact_digest: fixture.binding.artifact_digest.clone(),
        participant_needs_digest: fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(8),
        inbox_prefix: "_INBOX.session-rollback".to_owned(),
        created_at: NOW,
        expires_at: None,
    })?;
    let mut rollback_creation = test_session_creation(rollback_session.clone(), None, None);
    rollback_creation.actions.push(PostCommitActionRecord {
        payload: json!({ "different": true }),
        ..creation_action
    });
    let rollback_idempotency = rollback_creation.idempotency.clone();
    assert_eq!(
        store.create_session(rollback_creation).await,
        Err(AuthorizationStateError::StorageConflict)
    );
    assert_eq!(store.get_session(&rollback_session.session_id).await?, None);
    assert!(store
        .get_idempotency_result(
            &rollback_idempotency.purpose,
            &rollback_idempotency.signer_id,
            &rollback_idempotency.request_id,
        )
        .await?
        .is_none());
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
    let browser_session = SessionRecord::from_new(NewSession {
        session_id: "ses_browser_bind".to_owned(),
        principal_id: principal.principal_id.clone(),
        principal_kind: PrincipalKind::User,
        participant_id: fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKindV1::App,
        participant_artifact_digest: fixture.binding.artifact_digest.clone(),
        participant_needs_digest: fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(12),
        inbox_prefix: "_INBOX.browser-bind".to_owned(),
        created_at: NOW + 1,
        expires_at: None,
    })?;
    assert!(matches!(
        store
            .create_session(test_session_creation(
                browser_session,
                Some(DesiredAuthorityRecord::Identity(metadata_only)),
                None,
            ))
            .await?,
        IdempotentOutcome::Applied(_)
    ));

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

    let revocation = test_session_revocation(&session, 1, NOW + 60, "ses_01");
    let revoked = match store.revoke_session(revocation.clone()).await? {
        IdempotentOutcome::Applied(session) => session,
        IdempotentOutcome::Replayed(_) => return Err("unexpected revocation replay".into()),
    };
    assert_eq!(revoked.version, 2);
    assert_eq!(revoked.state, SessionState::Revoked);
    assert_eq!(
        store.touch_session(&session.session_id, NOW + 70).await,
        Err(AuthorizationStateError::SessionRevoked)
    );
    assert_eq!(
        store.revoke_session(revocation.clone()).await?,
        IdempotentOutcome::Replayed(revocation.idempotency.result.clone())
    );
    assert_eq!(
        store
            .get_session(&session.session_id)
            .await?
            .ok_or("session missing after revocation replay")?
            .version,
        2
    );
    let ready_actions = store.list_ready_post_commit_actions(NOW + 60, 100).await?;
    for expected in &revocation.actions {
        assert_eq!(
            ready_actions
                .iter()
                .filter(|action| action.action_id == expected.action_id)
                .count(),
            1
        );
    }
    assert_eq!(
        store
            .revoke_session(test_session_revocation(
                &session,
                1,
                NOW + 80,
                "ses_01_stale",
            ))
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

async fn exercise_companion_repositories<S>(store: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: AccountRepository
        + DeploymentProfileRepository
        + LoginPortalRepository
        + AccountFlowRepository
        + AuthorityProposalRepository
        + ProvisioningRepository
        + IdempotencyRepository
        + PostCommitActionRepository
        + PrincipalRepository
        + ProviderIdentityRepository
        + ParticipantBindingRepository
        + AuthorizationMaterializationRepository
        + EvidenceRepository
        + DeploymentAuthorityRepository
        + AuthSessionRepository
        + SessionRepository
        + Clone,
{
    let proof = |byte: u8, purpose: &str| IdempotencyResultRecord {
        scope_key: digest(byte),
        purpose: purpose.to_owned(),
        signer_id: "signer_companion".to_owned(),
        request_id: format!("request_{byte}"),
        request_digest: digest(byte + 1),
        result: json!({ "request": byte }),
        created_at: NOW,
        expires_at: NOW + 1_000,
    };
    let action = |byte: u8, event: &str| PostCommitActionRecord {
        action_id: digest(byte),
        kind: PostCommitActionKind::Event,
        payload: json!({ "event": event }),
        created_at: NOW,
        attempts: 0,
        next_attempt_at: NOW,
        claimed_until: None,
        last_error: None,
    };
    let deployment_principal = PrincipalRecord {
        principal_id: "dep_profile".to_owned(),
        kind: PrincipalKind::Service,
        state: PrincipalState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
        disabled_at: None,
        revoked_at: None,
    };
    let mut deployment_profile = DeploymentProfileRecord {
        deployment_id: deployment_principal.principal_id.clone(),
        kind: PrincipalKind::Service,
        display_name: "Profile Service".to_owned(),
        participant_id: None,
        portal_id: None,
        requires_device_delegation: false,
        expires_at: None,
        state: DeploymentProfileState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    assert!(matches!(
        store
            .create_deployment_profile(DeploymentProfileCreation {
                principal: deployment_principal,
                profile: deployment_profile.clone(),
                idempotency: proof(90, "deployment.create"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store.list_deployment_profiles().await?,
        vec![deployment_profile.clone()]
    );
    deployment_profile.state = DeploymentProfileState::Disabled;
    deployment_profile.updated_at += 1;
    deployment_profile.version += 1;
    store
        .put_deployment_profile(DeploymentProfileMutation {
            profile: deployment_profile.clone(),
            expected_version: 1,
            idempotency: proof(92, "deployment.disable"),
            actions: Vec::new(),
        })
        .await?;
    assert_eq!(
        store
            .get_deployment_profile("dep_profile")
            .await?
            .map(|value| value.state),
        Some(DeploymentProfileState::Disabled)
    );
    let user = PrincipalRecord {
        principal_id: "usr_companion".to_owned(),
        kind: PrincipalKind::User,
        state: PrincipalState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
        disabled_at: None,
        revoked_at: None,
    };
    let profile = UserProfileRecord {
        principal_id: user.principal_id.clone(),
        display_name: Some("Companion User".to_owned()),
        email: Some("user@example.com".to_owned()),
        image_url: None,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    let credential = LocalCredentialRecord {
        principal_id: user.principal_id.clone(),
        normalized_username: "companion".to_owned(),
        password_hash: "argon2id-hash".to_owned(),
        hash_profile: 1,
        failed_attempts: 0,
        locked_until: None,
        password_changed_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    let identity = ProviderIdentityLink {
        provider: "local".to_owned(),
        provider_subject: "companion".to_owned(),
        principal_id: user.principal_id.clone(),
        linked_at: NOW,
        last_seen_at: NOW,
    };
    let account_action = action(20, "account.created");
    let account_creation = AccountCreation {
        principal: user.clone(),
        profile: profile.clone(),
        credential: Some(credential.clone()),
        identity: Some(identity.clone()),
        idempotency: proof(20, "account.create"),
        actions: vec![account_action.clone()],
    };
    assert_eq!(
        store.create_user_account(account_creation.clone()).await?,
        IdempotentOutcome::Applied(profile.clone())
    );
    let mut replayed_account = account_creation.clone();
    replayed_account.profile.version = 99;
    replayed_account.actions[0].payload = json!({ "different": true });
    assert_eq!(
        store.create_user_account(replayed_account).await?,
        IdempotentOutcome::Replayed(account_creation.idempotency.result.clone())
    );
    let mut mismatched_account = account_creation.clone();
    mismatched_account.idempotency.request_digest = digest(99);
    assert_eq!(
        store.create_user_account(mismatched_account).await,
        Err(AuthorizationStateError::StorageConflict)
    );
    assert_eq!(
        store.get_user_profile(&user.principal_id).await?,
        Some(profile.clone())
    );
    assert_eq!(
        store.get_local_credential(&user.principal_id).await?,
        Some(credential.clone())
    );
    assert_eq!(
        store
            .get_provider_identity(&identity.provider, &identity.provider_subject)
            .await?,
        Some(identity)
    );

    let mut conflicting_user = user.clone();
    conflicting_user.principal_id = "usr_rolled_back".to_owned();
    let mut conflicting_profile = profile_for(&conflicting_user.principal_id);
    conflicting_profile.display_name = Some("Rolled Back".to_owned());
    let mut conflicting_credential = credential.clone();
    conflicting_credential.principal_id = conflicting_user.principal_id.clone();
    conflicting_credential.normalized_username = "rolled-back".to_owned();
    let conflicting_identity = ProviderIdentityLink {
        provider: "local".to_owned(),
        provider_subject: "rolled-back".to_owned(),
        principal_id: conflicting_user.principal_id.clone(),
        linked_at: NOW,
        last_seen_at: NOW,
    };
    let account_rollback_proof = proof(22, "account.create");
    assert_eq!(
        store
            .create_user_account(AccountCreation {
                principal: conflicting_user.clone(),
                profile: conflicting_profile,
                credential: Some(conflicting_credential),
                identity: Some(conflicting_identity),
                idempotency: account_rollback_proof.clone(),
                actions: vec![PostCommitActionRecord {
                    payload: json!({ "different": true }),
                    ..account_action.clone()
                }],
            })
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );
    assert_eq!(
        store.get_principal(&conflicting_user.principal_id).await?,
        None
    );
    assert!(store
        .get_idempotency_result(
            &account_rollback_proof.purpose,
            &account_rollback_proof.signer_id,
            &account_rollback_proof.request_id,
        )
        .await?
        .is_none());

    let managed_user = PrincipalRecord {
        principal_id: "usr_account_a".to_owned(),
        ..user.clone()
    };
    let managed_profile = UserProfileRecord {
        principal_id: managed_user.principal_id.clone(),
        display_name: None,
        email: None,
        image_url: None,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    assert_eq!(
        store
            .create_user_account(AccountCreation {
                principal: managed_user.clone(),
                profile: managed_profile.clone(),
                credential: None,
                identity: None,
                idempotency: proof(100, "account.create-managed"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(managed_profile.clone())
    );
    assert_eq!(
        store.get_user_account(&managed_user.principal_id).await?,
        Some((managed_user.clone(), managed_profile.clone()))
    );
    assert_eq!(
        store
            .get_local_credential(&managed_user.principal_id)
            .await?,
        None
    );
    let managed_user_b = PrincipalRecord {
        principal_id: "usr_account_b".to_owned(),
        ..user.clone()
    };
    let managed_profile_b = UserProfileRecord {
        principal_id: managed_user_b.principal_id.clone(),
        ..managed_profile.clone()
    };
    store
        .create_user_account(AccountCreation {
            principal: managed_user_b.clone(),
            profile: managed_profile_b.clone(),
            credential: None,
            identity: None,
            idempotency: proof(102, "account.create-managed"),
            actions: Vec::new(),
        })
        .await?;
    assert_eq!(
        store.list_user_accounts(None, 2).await?,
        vec![
            (managed_user.clone(), managed_profile.clone()),
            (managed_user_b.clone(), managed_profile_b.clone()),
        ]
    );
    assert_eq!(
        store
            .list_user_accounts(Some(&managed_user.principal_id), 2)
            .await?,
        vec![
            (managed_user_b, managed_profile_b),
            (user.clone(), profile.clone()),
        ]
    );

    let account_update_action = action(100, "account.updated");
    let mut disabled_user = managed_user.clone();
    disabled_user.state = PrincipalState::Disabled;
    disabled_user.updated_at = NOW + 5;
    disabled_user.version = 2;
    let mut updated_profile = managed_profile.clone();
    updated_profile.email = Some("managed@example.com".to_owned());
    updated_profile.updated_at = NOW + 5;
    updated_profile.version = 2;
    let account_update = UserAccountMutation {
        principal: disabled_user,
        profile: updated_profile,
        expected_version: 1,
        idempotency: proof(104, "account.update"),
        actions: vec![account_update_action.clone()],
    };
    let (mut disabled_user, updated_profile) =
        match store.update_user_account(account_update.clone()).await? {
            IdempotentOutcome::Applied(account) => account,
            IdempotentOutcome::Replayed(_) => unreachable!(),
        };
    assert_eq!(disabled_user.disabled_at, Some(NOW + 5));
    let persisted_disabled_account = (disabled_user.clone(), updated_profile.clone());
    let mut malformed_replay = account_update.clone();
    malformed_replay.principal.version = 99;
    malformed_replay.actions[0].payload = json!({ "different": true });
    assert_eq!(
        store.update_user_account(malformed_replay).await?,
        IdempotentOutcome::Replayed(account_update.idempotency.result.clone())
    );

    disabled_user.state = PrincipalState::Active;
    disabled_user.updated_at = NOW + 6;
    disabled_user.version = 3;
    let mut rollback_profile = updated_profile.clone();
    rollback_profile.display_name = Some("Should Roll Back".to_owned());
    rollback_profile.updated_at = NOW + 6;
    rollback_profile.version = 3;
    let account_update_rollback_proof = proof(106, "account.update");
    assert_eq!(
        store
            .update_user_account(UserAccountMutation {
                principal: disabled_user,
                profile: rollback_profile,
                expected_version: 2,
                idempotency: account_update_rollback_proof.clone(),
                actions: vec![PostCommitActionRecord {
                    payload: json!({ "different": true }),
                    ..account_update_action
                }],
            })
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );
    assert_eq!(
        store.get_user_account(&managed_user.principal_id).await?,
        Some(persisted_disabled_account)
    );
    assert!(store
        .get_idempotency_result(
            &account_update_rollback_proof.purpose,
            &account_update_rollback_proof.signer_id,
            &account_update_rollback_proof.request_id,
        )
        .await?
        .is_none());

    let portal = LoginPortalRecord {
        portal_id: "builtin".to_owned(),
        display_name: "Built-in".to_owned(),
        entry_url: None,
        builtin: true,
        disabled: false,
        removed: false,
        local_registration_enabled: true,
        provider_ids: vec!["local".to_owned()],
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    let settings = LoginSettingsRecord {
        portal_id: portal.portal_id.clone(),
        default_provider_id: Some("local".to_owned()),
        local_login_enabled: true,
        federated_registration_enabled: true,
        provider_selection_enabled: false,
        updated_at: NOW,
        version: 1,
    };
    store
        .put_login_portal(LoginPortalMutation {
            portal: portal.clone(),
            settings: settings.clone(),
            expected_version: None,
            idempotency: proof(24, "portal.put"),
            actions: Vec::new(),
        })
        .await?;
    let mut updated_portal = portal.clone();
    updated_portal.display_name = "Built-in Portal".to_owned();
    updated_portal.updated_at = NOW + 1;
    updated_portal.version = 2;
    let mut updated_settings = settings;
    updated_settings.updated_at = NOW + 1;
    updated_settings.version = 2;
    let updated_portal = match store
        .put_login_portal(LoginPortalMutation {
            portal: updated_portal,
            settings: updated_settings,
            expected_version: Some(1),
            idempotency: proof(26, "portal.put"),
            actions: Vec::new(),
        })
        .await?
    {
        IdempotentOutcome::Applied((portal, _)) => portal,
        IdempotentOutcome::Replayed(_) => return Err("unexpected portal replay".into()),
    };
    assert_eq!(updated_portal.version, 2);
    let mut removable = updated_portal.clone();
    removable.builtin = false;
    let (_, current_settings) = store
        .get_login_portal(&portal.portal_id)
        .await?
        .ok_or("missing portal")?;
    assert_eq!(
        store
            .put_login_portal(LoginPortalMutation {
                portal: removable,
                settings: current_settings,
                expected_version: Some(2),
                idempotency: proof(28, "portal.put"),
                actions: Vec::new(),
            })
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );
    let route = PortalRouteRecord {
        route_id: "route_1".to_owned(),
        portal_id: portal.portal_id.clone(),
        participant_id: Some("example.device".to_owned()),
        origin: None,
        deployment_id: None,
        priority: 10,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    store
        .put_portal_route(PortalRouteMutation {
            route: route.clone(),
            expected_version: None,
            idempotency: proof(30, "portal-route.put"),
            actions: Vec::new(),
        })
        .await?;
    assert_eq!(store.list_portal_routes().await?, vec![route.clone()]);
    let route_removal = PortalRouteRemoval {
        route_id: route.route_id.clone(),
        expected_version: 1,
        idempotency: proof(32, "portal-route.remove"),
        actions: Vec::new(),
    };
    assert_eq!(
        store.remove_portal_route(route_removal.clone()).await?,
        IdempotentOutcome::Applied(route)
    );
    assert_eq!(
        store.remove_portal_route(route_removal.clone()).await?,
        IdempotentOutcome::Replayed(route_removal.idempotency.result)
    );
    assert!(store.list_portal_routes().await?.is_empty());

    let app_fixture = participant_fixture_for(ParticipantKindV1::App, "example.admin")?;
    store
        .put_participant_binding(app_fixture.binding.clone())
        .await?;
    let old_first_admin = AccountFlowRecord {
        flow_id: "flow_first_admin_old".to_owned(),
        kind: AccountFlowKind::FirstAdmin,
        token_hash: digest(30),
        target_principal_id: None,
        target_provider_id: None,
        return_location: None,
        payload: json!({}),
        state: AccountFlowState::Pending,
        created_at: NOW,
        expires_at: NOW + 100,
        consumed_at: None,
        version: 1,
    };
    assert_eq!(
        store
            .replace_first_admin_flow(old_first_admin.clone(), NOW, false)
            .await?,
        Some(old_first_admin.clone())
    );
    let mut duplicate_first_admin = old_first_admin.clone();
    duplicate_first_admin.flow_id = "flow_first_admin".to_owned();
    duplicate_first_admin.token_hash = digest(31);
    assert_eq!(
        store
            .replace_first_admin_flow(duplicate_first_admin, NOW + 1, false)
            .await?
            .ok_or("pending first-admin flow missing")?,
        old_first_admin
    );
    let first_admin_flow = old_first_admin.clone();
    let admin = PrincipalRecord {
        principal_id: "usr_first_admin".to_owned(),
        kind: PrincipalKind::User,
        state: PrincipalState::Active,
        created_at: NOW + 1,
        updated_at: NOW + 1,
        version: 1,
        disabled_at: None,
        revoked_at: None,
    };
    let admin_profile = profile_for(&admin.principal_id);
    let admin_identity = ProviderIdentityLink {
        provider: "oidc-admin".to_owned(),
        provider_subject: "subject-first-admin".to_owned(),
        principal_id: admin.principal_id.clone(),
        linked_at: NOW + 1,
        last_seen_at: NOW + 1,
    };
    let admin_authority = IdentityAuthorityRecord {
        authority_id: "authority_first_admin".to_owned(),
        principal_id: admin.principal_id.clone(),
        participant_id: app_fixture.binding.participant_id.clone(),
        participant_artifact_digest: app_fixture.binding.artifact_digest.clone(),
        accepted_needs_digest: app_fixture.binding.needs_digest.clone(),
        desired_grant_set: app_fixture.required_grants.clone(),
        desired_capabilities: vec!["admin".to_owned()],
        state: AuthorityState::Accepted,
        version: 1,
        created_at: NOW + 1,
        updated_at: NOW + 1,
        expires_at: None,
        decision: Some(AuthorityDecision {
            decided_at: NOW + 1,
            decided_by: "bootstrap".to_owned(),
            reason: None,
        }),
    };
    let first_admin_action = action(60, "first-admin.created");
    let first_admin_proof = proof(40, "first-admin.complete");
    assert!(matches!(
        store
            .complete_first_admin(FirstAdminCompletion {
                token_hash: first_admin_flow.token_hash.clone(),
                expected_flow_version: 1,
                principal: admin.clone(),
                profile: admin_profile,
                credential: None,
                identity: admin_identity,
                authority: admin_authority,
                consumed_at: NOW + 2,
                idempotency: first_admin_proof.clone(),
                actions: vec![first_admin_action.clone()],
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert!(store.has_active_administrator(NOW + 2).await?);
    let mut blocked_flow = first_admin_flow.clone();
    blocked_flow.flow_id = "flow_first_admin_blocked".to_owned();
    blocked_flow.token_hash = digest(32);
    assert_eq!(
        store
            .replace_first_admin_flow(blocked_flow, NOW + 2, false)
            .await?,
        None
    );

    let password_flow = AccountFlowRecord {
        flow_id: "flow_password".to_owned(),
        kind: AccountFlowKind::PasswordReset,
        token_hash: digest(33),
        target_principal_id: Some(user.principal_id.clone()),
        target_provider_id: None,
        return_location: Some("/account".to_owned()),
        payload: json!({ "immutable": true }),
        state: AccountFlowState::Pending,
        created_at: NOW,
        expires_at: NOW + 100,
        consumed_at: None,
        version: 1,
    };
    store
        .create_account_flow(AccountFlowCreation {
            flow: password_flow.clone(),
            idempotency: proof(34, "account-flow.create"),
            actions: Vec::new(),
        })
        .await?;
    let replacement = LocalCredentialRecord {
        password_hash: "replacement-hash".to_owned(),
        password_changed_at: NOW + 3,
        updated_at: NOW + 3,
        version: 2,
        ..credential.clone()
    };
    let password_proof = proof(42, "password-reset.complete");
    let password_action = action(61, "password.changed");
    let mut password_kick = action(98, "sessions.kick");
    password_kick.kind = PostCommitActionKind::Kick;
    let mut conflicting_action = first_admin_action.clone();
    conflicting_action.payload = json!({ "different": true });
    assert_eq!(
        store
            .complete_password_reset(PasswordResetCompletion {
                token_hash: password_flow.token_hash.clone(),
                expected_flow_version: 1,
                replacement: replacement.clone(),
                consumed_at: NOW + 3,
                idempotency: password_proof.clone(),
                actions: vec![conflicting_action, password_kick.clone()],
            })
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );
    assert_eq!(
        store.get_local_credential(&user.principal_id).await?,
        Some(credential.clone())
    );
    assert_eq!(
        store
            .get_account_flow_by_hash(&password_flow.token_hash)
            .await?
            .ok_or("password flow missing")?
            .state,
        AccountFlowState::Pending
    );
    assert!(store
        .get_idempotency_result(
            &password_proof.purpose,
            &password_proof.signer_id,
            &password_proof.request_id,
        )
        .await?
        .is_none());
    let password_command = PasswordResetCompletion {
        token_hash: password_flow.token_hash.clone(),
        expected_flow_version: 1,
        replacement: replacement.clone(),
        consumed_at: NOW + 3,
        idempotency: password_proof.clone(),
        actions: vec![password_action.clone(), password_kick],
    };
    assert!(matches!(
        store
            .complete_password_reset(password_command.clone())
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store
            .get_local_credential_by_username(&credential.normalized_username)
            .await?,
        Some(replacement.clone())
    );
    let mut replay_command = password_command;
    replay_command.replacement.version = 99;
    replay_command.actions = vec![PostCommitActionRecord {
        payload: json!({ "different": true }),
        ..password_action.clone()
    }];
    assert_eq!(
        store.complete_password_reset(replay_command).await?,
        IdempotentOutcome::Replayed(password_proof.result.clone())
    );
    assert_eq!(
        store.get_local_credential(&user.principal_id).await?,
        Some(replacement)
    );
    let mut mismatched_proof = password_proof.clone();
    mismatched_proof.request_digest = digest(99);
    let mut mismatch_command = PasswordResetCompletion {
        token_hash: password_flow.token_hash,
        expected_flow_version: 2,
        replacement: credential.clone(),
        consumed_at: NOW + 4,
        idempotency: mismatched_proof,
        actions: Vec::new(),
    };
    mismatch_command.replacement.version = 3;
    assert_eq!(
        store.complete_password_reset(mismatch_command).await,
        Err(AuthorizationStateError::StorageConflict)
    );

    let identity_flow = AccountFlowRecord {
        flow_id: "flow_identity".to_owned(),
        kind: AccountFlowKind::IdentityLink,
        token_hash: digest(34),
        target_principal_id: Some(user.principal_id.clone()),
        target_provider_id: Some("github".to_owned()),
        return_location: None,
        payload: json!({}),
        state: AccountFlowState::Pending,
        created_at: NOW,
        expires_at: NOW + 100,
        consumed_at: None,
        version: 1,
    };
    store
        .create_account_flow(AccountFlowCreation {
            flow: identity_flow.clone(),
            idempotency: proof(36, "account-flow.create"),
            actions: Vec::new(),
        })
        .await?;
    let linked_identity = ProviderIdentityLink {
        provider: "github".to_owned(),
        provider_subject: "companion-gh".to_owned(),
        principal_id: user.principal_id.clone(),
        linked_at: NOW + 4,
        last_seen_at: NOW + 4,
    };
    assert!(matches!(
        store
            .complete_identity_link(IdentityLinkCompletion {
                token_hash: identity_flow.token_hash,
                expected_flow_version: 1,
                identity: linked_identity.clone(),
                consumed_at: NOW + 4,
                idempotency: proof(44, "identity-link.complete"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store
            .get_provider_identity(&linked_identity.provider, &linked_identity.provider_subject)
            .await?,
        Some(linked_identity)
    );
    assert_eq!(
        store
            .list_provider_identities(&user.principal_id)
            .await?
            .len(),
        2
    );
    let mut unlink_idempotency = proof(45, "identity.unlink");
    unlink_idempotency.result = json!({ "unlinked": true });
    assert_eq!(
        store
            .unlink_provider_identity(ProviderIdentityUnlink {
                provider: "github".to_owned(),
                provider_subject: "companion-gh".to_owned(),
                principal_id: user.principal_id.clone(),
                idempotency: unlink_idempotency.clone(),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(true)
    );
    assert_eq!(
        store
            .unlink_provider_identity(ProviderIdentityUnlink {
                provider: "github".to_owned(),
                provider_subject: "companion-gh".to_owned(),
                principal_id: user.principal_id.clone(),
                idempotency: unlink_idempotency,
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Replayed(json!({ "unlinked": true }))
    );

    let user_fixture = participant_fixture_for(ParticipantKindV1::App, "example.user")?;
    store
        .put_participant_binding(user_fixture.binding.clone())
        .await?;
    let authority = IdentityAuthorityRecord {
        authority_id: "ida_password_change".to_owned(),
        principal_id: user.principal_id.clone(),
        participant_id: user_fixture.binding.participant_id.clone(),
        participant_artifact_digest: user_fixture.binding.artifact_digest.clone(),
        accepted_needs_digest: user_fixture.binding.needs_digest.clone(),
        desired_grant_set: user_fixture.required_grants.clone(),
        desired_capabilities: Vec::new(),
        state: AuthorityState::Accepted,
        version: 1,
        created_at: NOW,
        updated_at: NOW,
        expires_at: None,
        decision: Some(AuthorityDecision {
            decided_at: NOW,
            decided_by: "admin".to_owned(),
            reason: None,
        }),
    };
    let current_session = SessionRecord::from_new(NewSession {
        session_id: "ses_password_current".to_owned(),
        principal_id: user.principal_id.clone(),
        principal_kind: PrincipalKind::User,
        participant_id: user_fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKindV1::App,
        participant_artifact_digest: user_fixture.binding.artifact_digest.clone(),
        participant_needs_digest: user_fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(61),
        inbox_prefix: "_INBOX.password.current".to_owned(),
        created_at: NOW,
        expires_at: Some(NOW + 1_000),
    })?;
    let sibling_session = SessionRecord::from_new(NewSession {
        session_id: "ses_password_sibling".to_owned(),
        session_public_key: session_public_key(62),
        inbox_prefix: "_INBOX.password.sibling".to_owned(),
        ..NewSession {
            session_id: current_session.session_id.clone(),
            principal_id: current_session.principal_id.clone(),
            principal_kind: current_session.principal_kind,
            participant_id: current_session.participant_id.clone(),
            participant_kind: current_session.participant_kind,
            participant_artifact_digest: current_session.participant_artifact_digest.clone(),
            participant_needs_digest: current_session.participant_needs_digest.clone(),
            session_public_key: current_session.session_public_key.clone(),
            inbox_prefix: current_session.inbox_prefix.clone(),
            created_at: current_session.created_at,
            expires_at: current_session.expires_at,
        }
    })?;
    store
        .create_session(SessionCreation {
            session: current_session.clone(),
            desired_authority: Some(DesiredAuthorityRecord::Identity(authority)),
            runtime_binding: None,
            idempotency: proof(201, "password.session.current"),
            actions: Vec::new(),
        })
        .await
        .map_err(|error| format!("create current password session: {error}"))?;
    store
        .create_session(SessionCreation {
            session: sibling_session.clone(),
            desired_authority: None,
            runtime_binding: None,
            idempotency: proof(202, "password.session.sibling"),
            actions: Vec::new(),
        })
        .await
        .map_err(|error| format!("create sibling password session: {error}"))?;
    let credential = store
        .get_local_credential(&user.principal_id)
        .await?
        .ok_or("local credential missing")?;
    let replacement = LocalCredentialRecord {
        version: credential.version + 1,
        password_changed_at: NOW + 6,
        updated_at: NOW + 6,
        ..credential.clone()
    };
    assert_eq!(
        store
            .change_password(PasswordChange {
                principal_id: user.principal_id.clone(),
                current_session_id: current_session.session_id.clone(),
                credential: replacement,
                expected_version: credential.version,
                changed_at: NOW + 6,
                idempotency: proof(203, "password.change"),
                actions: Vec::new(),
            })
            .await
            .map_err(|error| format!("change password: {error}"))?,
        IdempotentOutcome::Applied(1)
    );
    assert_eq!(
        store
            .get_session(&sibling_session.session_id)
            .await?
            .map(|session| session.state),
        Some(SessionState::Revoked)
    );

    let device_fixture = participant_fixture_for(ParticipantKindV1::Device, "example.device")?;
    store
        .put_participant_binding(device_fixture.binding.clone())
        .await?;
    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_companion".to_owned(),
            participant_id: device_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKindV1::Device,
            active: true,
            expires_at: None,
        })
        .await?;
    let device_principal = PrincipalRecord {
        principal_id: "dev_companion".to_owned(),
        kind: PrincipalKind::Device,
        state: PrincipalState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
        disabled_at: None,
        revoked_at: None,
    };
    let device_instance = RuntimeInstanceRecord {
        instance_id: "inst_companion".to_owned(),
        deployment_id: "dep_companion".to_owned(),
        principal_id: device_principal.principal_id.clone(),
        state: RuntimeInstanceState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    let device = DeviceRecord {
        principal_id: device_principal.principal_id.clone(),
        deployment_id: device_instance.deployment_id.clone(),
        state: DeviceState::Pending,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    let secret = DeviceProvisioningSecretRecord {
        secret_id: "secret_1".to_owned(),
        instance_id: device_instance.instance_id.clone(),
        secret_hash: digest(50),
        state: ProvisioningSecretState::Pending,
        created_at: NOW,
        expires_at: NOW + 100,
        consumed_at: None,
        version: 1,
    };
    assert!(matches!(
        store
            .provision_device(DeviceProvisioning {
                principal: device_principal.clone(),
                instance: device_instance.clone(),
                device: device.clone(),
                identity: None,
                secret: secret.clone(),
                idempotency: proof(46, "device.provision"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    let provisioned = ProvisionedIdentityRecord {
        identity_key_id: super::domain::validate_ed25519_public_key(
            "identityPublicKey",
            &session_public_key(50),
        )?,
        identity_public_key: session_public_key(50),
        principal_id: device_principal.principal_id.clone(),
        deployment_id: device_instance.deployment_id.clone(),
        instance_id: device_instance.instance_id.clone(),
        kind: ProvisionedIdentityKind::Device,
        state: ProvisionedIdentityState::Active,
        created_at: NOW + 1,
        revoked_at: None,
    };
    assert!(matches!(
        store
            .consume_device_provisioning_secret(DeviceProvisioningSecretConsumption {
                secret_hash: secret.secret_hash.clone(),
                expected_version: 1,
                identity: provisioned.clone(),
                consumed_at: NOW + 1,
                idempotency: proof(48, "device-secret.consume"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store
            .get_provisioned_identity(&provisioned.identity_key_id)
            .await?,
        Some(provisioned.clone())
    );

    let second_principal = PrincipalRecord {
        principal_id: "dev_companion_2".to_owned(),
        ..device_principal.clone()
    };
    let second_instance = RuntimeInstanceRecord {
        instance_id: "inst_companion_2".to_owned(),
        principal_id: second_principal.principal_id.clone(),
        ..device_instance.clone()
    };
    let second_device = DeviceRecord {
        principal_id: second_principal.principal_id.clone(),
        ..device.clone()
    };
    let second_secret = DeviceProvisioningSecretRecord {
        secret_id: "secret_2".to_owned(),
        instance_id: second_instance.instance_id.clone(),
        secret_hash: digest(52),
        ..secret.clone()
    };
    store
        .provision_device(DeviceProvisioning {
            principal: second_principal.clone(),
            instance: second_instance.clone(),
            device: second_device,
            identity: None,
            secret: second_secret.clone(),
            idempotency: proof(52, "device.provision.second"),
            actions: Vec::new(),
        })
        .await?;
    let reassignment_proof = proof(54, "device-secret.consume.second");
    let reassigned = ProvisionedIdentityRecord {
        principal_id: second_principal.principal_id.clone(),
        instance_id: second_instance.instance_id.clone(),
        ..provisioned.clone()
    };
    assert_eq!(
        store
            .consume_device_provisioning_secret(DeviceProvisioningSecretConsumption {
                secret_hash: second_secret.secret_hash.clone(),
                expected_version: 1,
                identity: reassigned,
                consumed_at: NOW + 2,
                idempotency: reassignment_proof.clone(),
                actions: Vec::new(),
            })
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );
    let second_identity = ProvisionedIdentityRecord {
        identity_key_id: super::domain::validate_ed25519_public_key(
            "identityPublicKey",
            &session_public_key(51),
        )?,
        identity_public_key: session_public_key(51),
        principal_id: second_principal.principal_id,
        deployment_id: second_instance.deployment_id.clone(),
        instance_id: second_instance.instance_id,
        kind: ProvisionedIdentityKind::Device,
        state: ProvisionedIdentityState::Active,
        created_at: NOW + 2,
        revoked_at: None,
    };
    assert!(matches!(
        store
            .consume_device_provisioning_secret(DeviceProvisioningSecretConsumption {
                secret_hash: second_secret.secret_hash,
                expected_version: 1,
                identity: second_identity,
                consumed_at: NOW + 2,
                idempotency: reassignment_proof,
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));

    let service_fixture = participant_fixture_for(ParticipantKindV1::Service, "example.service")?;
    store
        .put_participant_binding(service_fixture.binding.clone())
        .await?;
    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_service_companion".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKindV1::Service,
            active: true,
            expires_at: None,
        })
        .await?;
    let service_principal = PrincipalRecord {
        principal_id: "svc_companion".to_owned(),
        kind: PrincipalKind::Service,
        state: PrincipalState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
        disabled_at: None,
        revoked_at: None,
    };
    let service_instance = RuntimeInstanceRecord {
        instance_id: "inst_service_companion".to_owned(),
        deployment_id: "dep_service_companion".to_owned(),
        principal_id: service_principal.principal_id.clone(),
        state: RuntimeInstanceState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    let service_identity = ProvisionedIdentityRecord {
        identity_key_id: super::domain::validate_ed25519_public_key(
            "identityPublicKey",
            &session_public_key(52),
        )?,
        identity_public_key: session_public_key(52),
        principal_id: service_principal.principal_id.clone(),
        deployment_id: service_instance.deployment_id.clone(),
        instance_id: service_instance.instance_id.clone(),
        kind: ProvisionedIdentityKind::Service,
        state: ProvisionedIdentityState::Active,
        created_at: NOW,
        revoked_at: None,
    };
    assert!(matches!(
        store
            .provision_service_identity(ServiceIdentityProvisioning {
                principal: service_principal,
                instance: service_instance,
                identity: service_identity.clone(),
                idempotency: proof(56, "service.provision"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store
            .get_provisioned_identity(&service_identity.identity_key_id)
            .await?,
        Some(service_identity)
    );

    let proposal = AuthorityProposalRecord {
        proposal_id: "proposal_1".to_owned(),
        authority_kind: AuthorityKind::Deployment,
        authority_id: deployment_authority_id(
            "dep_companion",
            &device_fixture.binding.participant_id,
        )?,
        deployment_id: Some("dep_companion".to_owned()),
        proposal_kind: AuthorityProposalKind::Initial,
        participant_id: device_fixture.binding.participant_id.clone(),
        participant_artifact_digest: device_fixture.binding.artifact_digest.clone(),
        participant_needs_digest: device_fixture.binding.needs_digest.clone(),
        proposed_grant_set: device_fixture.required_grants.clone(),
        proposed_capabilities: vec!["device.use".to_owned()],
        proposal_digest: digest(70),
        payload: json!({
            "deploymentId": "dep_companion",
            "baseAuthorityVersion": null,
            "plan": "fixed"
        }),
        state: AuthorityProposalState::Pending,
        created_at: NOW,
        expires_at: Some(NOW + 10),
        superseded_at: None,
        version: 1,
    };
    store
        .create_authority_proposal(AuthorityProposalCreation {
            proposal: proposal.clone(),
            idempotency: proof(68, "authority-proposal.create"),
            actions: Vec::new(),
        })
        .await?;
    let mut equivalent_proposal = proposal.clone();
    equivalent_proposal.proposal_id = "proposal_equivalent".to_owned();
    equivalent_proposal.created_at += 1;
    equivalent_proposal.expires_at = Some(NOW + 200);
    assert_eq!(
        store
            .create_authority_proposal(AuthorityProposalCreation {
                proposal: equivalent_proposal.clone(),
                idempotency: proof(69, "authority-proposal.create"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Replayed(json!({ "proposalId": proposal.proposal_id }))
    );
    assert!(store
        .get_authority_proposal(&equivalent_proposal.proposal_id)
        .await?
        .is_none());
    let rejection = AuthorityDecisionRecord {
        proposal_id: proposal.proposal_id.clone(),
        outcome: AuthorityDecisionOutcome::Rejected,
        decided_by: admin.principal_id.clone(),
        reason: Some("not yet".to_owned()),
        decided_at: NOW + 2,
        decision_digest: digest(90),
    };
    store
        .decide_authority_proposal(AuthorityProposalDecision {
            proposal_id: proposal.proposal_id.clone(),
            expected_version: 1,
            decision: rejection,
            desired_authority: None,
            deployment: None,
            idempotency: proof(183, "authority-proposal.reject"),
            actions: Vec::new(),
        })
        .await?;
    let mut rejected_retry = proposal.clone();
    rejected_retry.proposal_id = "proposal_rejected_retry".to_owned();
    rejected_retry.created_at = NOW + 3;
    rejected_retry.expires_at = Some(NOW + 4);
    assert!(matches!(
        store
            .create_authority_proposal(AuthorityProposalCreation {
                proposal: rejected_retry.clone(),
                idempotency: proof(184, "authority-proposal.create"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    let mut elapsed_replay = rejected_retry.clone();
    elapsed_replay.proposal_id = "proposal_elapsed_replay".to_owned();
    elapsed_replay.created_at = NOW + 4;
    elapsed_replay.expires_at = Some(NOW + 100);
    assert_eq!(
        store
            .create_authority_proposal(AuthorityProposalCreation {
                proposal: elapsed_replay,
                idempotency: proof(184, "authority-proposal.create"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Replayed(json!({ "request": 184 }))
    );
    assert_eq!(
        store
            .get_authority_proposal(&rejected_retry.proposal_id)
            .await?
            .ok_or("elapsed replay proposal missing")?
            .0
            .state,
        AuthorityProposalState::Expired
    );
    let mut expired_retry = rejected_retry.clone();
    expired_retry.proposal_id = "proposal_expired_retry".to_owned();
    expired_retry.created_at = NOW + 4;
    expired_retry.expires_at = Some(NOW + 100);
    assert!(matches!(
        store
            .create_authority_proposal(AuthorityProposalCreation {
                proposal: expired_retry.clone(),
                idempotency: proof(185, "authority-proposal.create"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store
            .get_authority_proposal(&rejected_retry.proposal_id)
            .await?
            .ok_or("expired proposal missing")?
            .0
            .state,
        AuthorityProposalState::Expired
    );
    let mut superseded_proposal = proposal.clone();
    superseded_proposal.proposal_id = "proposal_2".to_owned();
    superseded_proposal.proposal_digest = digest(71);
    superseded_proposal.created_at = NOW + 5;
    superseded_proposal.expires_at = Some(NOW + 100);
    store
        .create_authority_proposal(AuthorityProposalCreation {
            proposal: superseded_proposal.clone(),
            idempotency: proof(70, "authority-proposal.create"),
            actions: Vec::new(),
        })
        .await?;
    let decision = AuthorityDecisionRecord {
        proposal_id: superseded_proposal.proposal_id.clone(),
        outcome: AuthorityDecisionOutcome::Accepted,
        decided_by: admin.principal_id.clone(),
        reason: None,
        decided_at: NOW + 6,
        decision_digest: digest(72),
    };
    let desired = DeploymentAuthorityRecord {
        authority_id: superseded_proposal.authority_id.clone(),
        deployment_id: "dep_companion".to_owned(),
        participant_id: superseded_proposal.participant_id.clone(),
        participant_kind: ParticipantKindV1::Device,
        participant_artifact_digest: superseded_proposal.participant_artifact_digest.clone(),
        accepted_needs_digest: superseded_proposal.participant_needs_digest.clone(),
        desired_grant_set: superseded_proposal.proposed_grant_set.clone(),
        desired_capabilities: superseded_proposal.proposed_capabilities.clone(),
        state: AuthorityState::Accepted,
        version: 1,
        created_at: NOW + 6,
        updated_at: NOW + 6,
        expires_at: None,
        decision: Some(AuthorityDecision {
            decided_at: NOW + 6,
            decided_by: admin.principal_id.clone(),
            reason: None,
        }),
    };
    let proposal_proof = proof(58, "authority-proposal.decide");
    assert!(matches!(
        store
            .decide_authority_proposal(AuthorityProposalDecision {
                proposal_id: superseded_proposal.proposal_id.clone(),
                expected_version: 1,
                decision,
                desired_authority: Some(DesiredAuthorityRecord::Deployment(desired.clone())),
                deployment: None,
                idempotency: proposal_proof.clone(),
                actions: vec![action(62, "authority.accepted")],
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store
            .get_authority_proposal(&proposal.proposal_id)
            .await?
            .ok_or("superseded proposal missing")?
            .0
            .state,
        AuthorityProposalState::Rejected
    );
    assert_eq!(
        store
            .get_authority_proposal(&expired_retry.proposal_id)
            .await?
            .ok_or("superseded proposal missing")?
            .0
            .state,
        AuthorityProposalState::Superseded
    );
    assert_eq!(
        store
            .list_authority_proposals()
            .await?
            .into_iter()
            .filter(|(candidate, _)| candidate.proposal_digest == proposal.proposal_digest)
            .count(),
        3,
        "rejected, expired, and superseded semantic history must coexist",
    );
    let proposal_ids = store
        .list_authority_proposals()
        .await?
        .into_iter()
        .map(|(candidate, _)| candidate.proposal_id)
        .collect::<Vec<_>>();
    let mut sorted_proposal_ids = proposal_ids.clone();
    sorted_proposal_ids.sort();
    assert_eq!(proposal_ids, sorted_proposal_ids);
    assert_eq!(
        store
            .get_deployment_authority(&desired.deployment_id, &desired.participant_id)
            .await?,
        Some(desired.clone())
    );

    let mut stale_initial = proposal.clone();
    stale_initial.proposal_id = "proposal_stale_initial".to_owned();
    stale_initial.created_at = NOW + 7;
    stale_initial.expires_at = Some(NOW + 100);
    store
        .create_authority_proposal(AuthorityProposalCreation {
            proposal: stale_initial.clone(),
            idempotency: proof(186, "authority-proposal.create"),
            actions: Vec::new(),
        })
        .await?;
    let mut stale_desired = desired.clone();
    stale_desired.version = 2;
    stale_desired.updated_at = NOW + 7;
    assert_eq!(
        store
            .decide_authority_proposal(AuthorityProposalDecision {
                proposal_id: stale_initial.proposal_id.clone(),
                expected_version: 1,
                decision: AuthorityDecisionRecord {
                    proposal_id: stale_initial.proposal_id.clone(),
                    outcome: AuthorityDecisionOutcome::Accepted,
                    decided_by: admin.principal_id.clone(),
                    reason: None,
                    decided_at: NOW + 7,
                    decision_digest: digest(91),
                },
                desired_authority: Some(DesiredAuthorityRecord::Deployment(stale_desired)),
                deployment: None,
                idempotency: proof(187, "authority-proposal.accept-stale"),
                actions: Vec::new(),
            })
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );
    store
        .decide_authority_proposal(AuthorityProposalDecision {
            proposal_id: stale_initial.proposal_id.clone(),
            expected_version: 1,
            decision: AuthorityDecisionRecord {
                proposal_id: stale_initial.proposal_id,
                outcome: AuthorityDecisionOutcome::Rejected,
                decided_by: admin.principal_id.clone(),
                reason: Some("stale initial generation".to_owned()),
                decided_at: NOW + 8,
                decision_digest: digest(92),
            },
            desired_authority: None,
            deployment: None,
            idempotency: proof(188, "authority-proposal.reject-stale"),
            actions: Vec::new(),
        })
        .await?;

    let service = AuthService::new(store.clone(), AuthServiceConfig::default())?;
    for (index, (fixture, deployment_id)) in [
        (&service_fixture, "dep_service_lineage"),
        (&device_fixture, "dep_device_lineage"),
    ]
    .into_iter()
    .enumerate()
    {
        let participant_artifact = serde_json::from_str(&fixture.binding.participant_json)?;
        let referenced_api_artifacts =
            serde_json::from_str::<BTreeMap<String, Value>>(&fixture.binding.api_artifacts_json)?
                .into_values()
                .collect::<Vec<_>>();
        let input = PresentDeploymentAuthorityInput {
            deployment_id: deployment_id.to_owned(),
            participant_artifact,
            referenced_api_artifacts,
            created_at: NOW + 20 + index as i64,
            expires_at: Some(NOW + 200),
            idempotency: proof(190 + index as u8, "deployment.authority.present-initial"),
            actions: Vec::new(),
        };
        let initial = match service.present_deployment_authority(input.clone()).await? {
            IdempotentOutcome::Applied(value) => value,
            IdempotentOutcome::Replayed(_) => return Err("initial presentation replayed".into()),
        };
        assert_eq!(
            initial.authority_id,
            deployment_authority_id(deployment_id, &fixture.binding.participant_id)?
        );
        let mut repeated = input;
        repeated.created_at += 10;
        repeated.expires_at = Some(NOW + 300);
        repeated.idempotency = proof(192 + index as u8, "deployment.authority.present-initial");
        assert_eq!(
            service.present_deployment_authority(repeated).await?,
            IdempotentOutcome::Replayed(json!({ "proposalId": initial.proposal_id }))
        );
    }
    let mut compatible_participant: Value =
        serde_json::from_str(&device_fixture.binding.participant_json)?;
    compatible_participant["displayName"] = json!("Updated device wording");
    let mut api_values = serde_json::from_str::<BTreeMap<String, Value>>(
        &device_fixture.binding.api_artifacts_json,
    )?;
    api_values
        .get_mut("required.api@v1")
        .ok_or("required API missing")?["schemas"]["Output"] = json!({
        "type": "object",
        "properties": { "added": { "type": "string" } }
    });
    compatible_participant["uses"]["required"]["requiredApi"]["apiDigest"] =
        json!(parse_api_v1(&api_values["required.api@v1"])?.digest()?);
    let mut compatible_api_artifacts = api_values.values().cloned().collect::<Vec<_>>();
    compatible_api_artifacts.push(compatible_api_artifacts[0].clone());
    let compatible_input = PresentDeploymentAuthorityInput {
        deployment_id: "dep_companion".to_owned(),
        participant_artifact: compatible_participant.clone(),
        referenced_api_artifacts: compatible_api_artifacts,
        created_at: NOW + 9,
        expires_at: Some(NOW + 200),
        idempotency: proof(180, "deployment.authority.present"),
        actions: Vec::new(),
    };
    let compatible = match service
        .present_deployment_authority(compatible_input.clone())
        .await?
    {
        IdempotentOutcome::Applied(value) => value,
        IdempotentOutcome::Replayed(_) => return Err("first presentation replayed".into()),
    };
    assert_eq!(compatible.proposal_kind, AuthorityProposalKind::Update);
    assert_eq!(compatible.authority_id, desired.authority_id);
    assert_eq!(compatible.proposed_grant_set, device_fixture.all_grants);
    let mut equivalent_input = compatible_input;
    equivalent_input.created_at += 1;
    equivalent_input.expires_at = Some(NOW + 300);
    equivalent_input.idempotency = proof(181, "deployment.authority.present");
    assert_eq!(
        service
            .present_deployment_authority(equivalent_input)
            .await?,
        IdempotentOutcome::Replayed(json!({ "proposalId": compatible.proposal_id }))
    );

    let mut incompatible_apis = api_values;
    incompatible_apis
        .get_mut("required.api@v1")
        .ok_or("required API missing")?["schemas"]["Input"] = json!({
        "type": "object",
        "required": ["changed"],
        "properties": { "changed": { "type": "string" } }
    });
    let incompatible_api = parse_api_v1(&incompatible_apis["required.api@v1"])?;
    compatible_participant["uses"]["required"]["requiredApi"]["apiDigest"] =
        json!(incompatible_api.digest()?);
    let migration = match service
        .present_deployment_authority(PresentDeploymentAuthorityInput {
            deployment_id: "dep_companion".to_owned(),
            participant_artifact: compatible_participant,
            referenced_api_artifacts: incompatible_apis.into_values().collect(),
            created_at: NOW + 11,
            expires_at: None,
            idempotency: proof(182, "deployment.authority.present"),
            actions: Vec::new(),
        })
        .await?
    {
        IdempotentOutcome::Applied(value) => value,
        IdempotentOutcome::Replayed(_) => return Err("migration presentation replayed".into()),
    };
    assert_eq!(migration.proposal_kind, AuthorityProposalKind::Migration);
    assert_eq!(
        store
            .get_authority_proposal(&compatible.proposal_id)
            .await?
            .ok_or("compatible proposal missing")?
            .0
            .state,
        AuthorityProposalState::Superseded
    );

    let review = DeviceActivationReviewRecord {
        review_id: "review_1".to_owned(),
        principal_id: device_principal.principal_id.clone(),
        deployment_id: device_instance.deployment_id.clone(),
        instance_id: device_instance.instance_id,
        request_digest: digest(73),
        payload: json!({ "device": "request" }),
        state: DeviceActivationReviewState::Pending,
        requested_at: NOW,
        decided_at: None,
        decided_by: None,
        reason: None,
        version: 1,
    };
    store
        .create_activation_review(ActivationReviewCreation {
            review: review.clone(),
            idempotency: proof(72, "activation-review.create"),
            actions: Vec::new(),
        })
        .await?;
    let review_proof = proof(64, "activation-review.decide");
    let review_action = action(63, "device.approved");
    let approved_device = DeviceRecord {
        state: DeviceState::Active,
        updated_at: NOW + 6,
        version: 2,
        ..device
    };
    assert!(matches!(
        store
            .decide_activation_review(ActivationReviewDecision {
                review_id: review.review_id.clone(),
                expected_version: 1,
                state: DeviceActivationReviewState::Approved,
                decided_at: NOW + 6,
                decided_by: admin.principal_id,
                reason: None,
                delegation: None,
                idempotency: review_proof.clone(),
                actions: vec![review_action],
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store
            .get_device(
                &approved_device.principal_id,
                &approved_device.deployment_id
            )
            .await?,
        Some(approved_device)
    );

    let ready = store.list_ready_post_commit_actions(NOW, 100).await?;
    assert_eq!(
        ready
            .iter()
            .filter(|candidate| candidate.action_id == account_action.action_id)
            .count(),
        1
    );
    assert_eq!(
        ready
            .iter()
            .filter(|candidate| candidate.action_id == password_action.action_id)
            .count(),
        1
    );
    store
        .claim_post_commit_action(&password_action.action_id, NOW, NOW + 10)
        .await?
        .ok_or("password action was not claimed")?;
    let failed = store
        .fail_post_commit_action(
            &password_action.action_id,
            NOW + 10,
            NOW + 20,
            "retry".to_owned(),
        )
        .await?;
    assert_eq!(failed.attempts, 1);
    store
        .claim_post_commit_action(&password_action.action_id, NOW + 20, NOW + 30)
        .await?
        .ok_or("password action was not reclaimed")?;
    store
        .acknowledge_post_commit_action(&password_action.action_id, NOW + 30)
        .await?;
    store
        .acknowledge_post_commit_action(&password_action.action_id, NOW + 30)
        .await?;
    Ok(())
}

fn profile_for(principal_id: &str) -> UserProfileRecord {
    UserProfileRecord {
        principal_id: principal_id.to_owned(),
        display_name: Some("User".to_owned()),
        email: None,
        image_url: None,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    }
}

async fn exercise_deployed_principals<S>(store: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: PrincipalRepository
        + SessionRepository
        + AuthSessionRepository
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
            created_at: NOW,
            updated_at: NOW,
            version: 1,
        })
        .await?;
    let service_runtime_binding = SessionRuntimeBinding {
        session_id: service_session.session_id.clone(),
        deployment_id: "dep_service".to_owned(),
        instance_id: "inst_service".to_owned(),
    };
    assert!(matches!(
        store
            .create_session(test_session_creation(service_session.clone(), None, None,))
            .await,
        Err(AuthorizationStateError::InvalidRecord(_))
    ));
    assert_eq!(store.get_session(&service_session.session_id).await?, None);
    store
        .create_session(test_session_creation(
            service_session.clone(),
            None,
            Some(service_runtime_binding.clone()),
        ))
        .await?;
    assert_eq!(
        store
            .get_session_runtime_binding(&service_session.session_id)
            .await?,
        Some(service_runtime_binding)
    );
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
            created_at: NOW,
            updated_at: NOW,
            version: 1,
        })
        .await?;
    store
        .put_device(DeviceRecord {
            principal_id: device_principal.principal_id.clone(),
            deployment_id: "dep_device".to_owned(),
            state: DeviceState::Active,
            created_at: NOW,
            updated_at: NOW,
            version: 1,
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
    let device_runtime_binding = SessionRuntimeBinding {
        session_id: device_session.session_id.clone(),
        deployment_id: "dep_device".to_owned(),
        instance_id: "inst_device".to_owned(),
    };
    store
        .create_session(test_session_creation(
            device_session.clone(),
            None,
            Some(device_runtime_binding.clone()),
        ))
        .await?;
    assert_eq!(
        store
            .get_session_runtime_binding(&device_session.session_id)
            .await?,
        Some(device_runtime_binding)
    );
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
            created_at: NOW,
            updated_at: NOW + 20,
            version: 2,
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
            created_at: NOW,
            updated_at: NOW + 21,
            version: 3,
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
        + AuthSessionRepository
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
                created_at: NOW,
                updated_at: NOW,
                version: 1,
            })
            .await?;
    }

    let mut instance = RuntimeInstanceRecord {
        instance_id: "inst_shared".to_owned(),
        deployment_id: "dep_shared_a".to_owned(),
        principal_id: "dev_shared".to_owned(),
        state: RuntimeInstanceState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
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
        created_at: NOW,
        updated_at: NOW,
        version: 1,
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

    for (session_id, seed) in [("ses_shared_a", 51_u8), ("ses_shared_b", 52_u8)] {
        let session = SessionRecord::from_new(NewSession {
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
        })?;
        let binding = SessionRuntimeBinding {
            session_id: session_id.to_owned(),
            deployment_id: "dep_shared_a".to_owned(),
            instance_id: "inst_shared".to_owned(),
        };
        store
            .create_session(test_session_creation(session, None, Some(binding.clone())))
            .await?;
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
    disabled_instance.updated_at = NOW + 1;
    disabled_instance.version = 2;
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
    instance.updated_at = NOW + 2;
    instance.version = 3;
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
            created_at: NOW,
            updated_at: NOW,
            version: 1,
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
        + AuthSessionRepository
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
    store
        .create_session(test_session_creation(session.clone(), None, None))
        .await?;
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
        + AuthSessionRepository
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
            participant_kind: ParticipantKindV1::Service,
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
        + AuthSessionRepository
        + IdempotencyRepository
        + PostCommitActionRepository
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
    S: PrincipalRepository
        + SessionRepository
        + AuthSessionRepository
        + ParticipantBindingRepository,
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
            .create_session(test_session_creation(base.clone(), None, None))
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
            .create_session(test_session_creation(participant_kind, None, None))
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
            .create_session(test_session_creation(needs, None, None))
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
        + AuthSessionRepository
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
        let session = SessionRecord::from_new(NewSession {
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
        })?;
        store
            .create_session(test_session_creation(session, None, None))
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

    let denied_session = store
        .get_session("ses_denied")
        .await?
        .ok_or("denied session missing")?;
    store
        .revoke_session(test_session_revocation(
            &denied_session,
            1,
            NOW + 2,
            "ses_denied",
        ))
        .await?;
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

fn test_session_creation(
    session: SessionRecord,
    desired_authority: Option<DesiredAuthorityRecord>,
    runtime_binding: Option<SessionRuntimeBinding>,
) -> SessionCreation {
    let scope = format!("session.create:{}", session.session_id);
    SessionCreation {
        idempotency: test_session_idempotency(&scope, &session.principal_id, session.created_at),
        session,
        desired_authority,
        runtime_binding,
        actions: Vec::new(),
    }
}

fn test_session_revocation(
    session: &SessionRecord,
    expected_version: u64,
    revoked_at: i64,
    request: &str,
) -> SessionRevocation {
    let event_id = test_digest(&format!("session.revoke.event:{request}"));
    let kick_id = test_digest(&format!("session.revoke.kick:{request}"));
    SessionRevocation {
        session_id: session.session_id.clone(),
        expected_version,
        revoked_at,
        idempotency: test_session_idempotency(
            &format!("session.revoke:{request}"),
            &session.principal_id,
            revoked_at,
        ),
        actions: vec![
            PostCommitActionRecord {
                action_id: event_id,
                kind: PostCommitActionKind::Event,
                payload: json!({ "sessionId": session.session_id }),
                created_at: revoked_at,
                attempts: 0,
                next_attempt_at: revoked_at,
                claimed_until: None,
                last_error: None,
            },
            PostCommitActionRecord {
                action_id: kick_id,
                kind: PostCommitActionKind::Kick,
                payload: json!({ "sessionId": session.session_id }),
                created_at: revoked_at,
                attempts: 0,
                next_attempt_at: revoked_at,
                claimed_until: None,
                last_error: None,
            },
        ],
    }
}

fn test_session_idempotency(
    request: &str,
    signer_id: &str,
    created_at: i64,
) -> IdempotencyResultRecord {
    IdempotencyResultRecord {
        scope_key: test_digest(&format!("scope:{request}")),
        purpose: if request.starts_with("session.create:") {
            "session.create"
        } else {
            "session.revoke"
        }
        .to_owned(),
        signer_id: signer_id.to_owned(),
        request_id: request.to_owned(),
        request_digest: test_digest(&format!("request:{request}")),
        result: json!({ "request": request }),
        created_at,
        expires_at: created_at + 10_000,
    }
}

fn test_digest(value: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(value.as_bytes()))
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
