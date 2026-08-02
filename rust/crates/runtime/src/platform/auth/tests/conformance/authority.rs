use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::{json, Value};
use trellis_protocol::ParticipantKindV1;

use super::companion::exercise_companion_repositories;
use super::evidence::exercise_resources;
use super::fixtures::{
    assert_issuable_context_valid, digest, evidence_scope, participant_fixture,
    participant_fixture_for, session_public_key, test_digest, test_session_creation,
    test_session_revocation, NOW,
};
use super::provisioning::exercise_deployed_principals;
use crate::platform::auth::application::repository::{
    AccountRepository, IdempotentOutcome, OutboxRepository, SessionRepository,
};
use crate::platform::auth::application::PresentDeploymentAuthorityInput;
use crate::platform::auth::authority::{
    AuthorityEvidenceRepository, AuthorityRepository, ContextRepository,
};
use crate::platform::auth::model::deployment_authority_id;
use crate::platform::auth::reconciliation::{
    authorization_reconciliation_channel, ReconciliationCause,
};
use crate::platform::auth::{
    AuthService, AuthServiceConfig, AuthorityDecision, AuthorityKind, AuthorityState,
    AuthorityTarget, AuthorizationStateError, AuthorizationStateService, DesiredAuthorityRecord,
    IdempotencyResultRecord, IdentityAuthorityRecord, MaterializationState, NewSession,
    PostCommitActionKind, PostCommitActionRecord, PrincipalKind, PrincipalRecord, PrincipalState,
    SessionRecord, SessionState, SqliteAuthorizationStore,
};

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
async fn sqlite_authorization_state_conformance() -> Result<(), Box<dyn std::error::Error>> {
    let store = exercise_store(SqliteAuthorizationStore::open_in_memory()?).await?;
    let outcomes = AuthorizationStateService::new(store.clone())
        .reconcile_all(NOW + 120)
        .await?;
    assert_eq!(outcomes.len(), 1);
    let materialization = store
        .get_materialized_authority(AuthorityKind::Identity, "ida_01")
        .await?
        .ok_or("missing startup-reconciled materialization")?;
    assert_eq!(
        materialization.authority.state,
        MaterializationState::Unavailable
    );
    assert!(materialization
        .authority
        .effective_grant_set
        .permissions()
        .is_empty());
    Ok(())
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
async fn sqlite_service_and_device_issuable_state() -> Result<(), Box<dyn std::error::Error>> {
    exercise_deployed_principals(SqliteAuthorizationStore::open_in_memory()?).await
}

#[tokio::test]
async fn sqlite_required_and_optional_resource_materialization(
) -> Result<(), Box<dyn std::error::Error>> {
    exercise_resources(SqliteAuthorizationStore::open_in_memory()?).await
}

#[tokio::test]
async fn reconciliation_worker_contains_expected_denials_and_stops(
) -> Result<(), Box<dyn std::error::Error>> {
    let service = AuthorizationStateService::new(SqliteAuthorizationStore::open_in_memory()?);
    let (handle, worker) = authorization_reconciliation_channel(service, 4);
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

pub(super) async fn exercise_store(
    store: SqliteAuthorizationStore,
) -> Result<SqliteAuthorizationStore, Box<dyn std::error::Error>> {
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
                Some(DesiredAuthorityRecord::Identity(authority.clone())),
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
    let outcome = service.reconcile_authority(&target, NOW + 30).await?;
    assert!(!outcome.changed);
    let unchanged = service
        .resolve_issuable_state(&session.session_id, NOW + 30)
        .await?;
    assert_eq!(unchanged.materialization_version, 1);
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
    let revocation = test_session_revocation(&session, 1, NOW + 60, "ses_01");
    let revoked = match store.revoke_session(revocation.clone()).await? {
        IdempotentOutcome::Applied(session) => session,
        IdempotentOutcome::Replayed(_) => return Err("unexpected revocation replay".into()),
    };
    assert_eq!(revoked.version, 2);
    assert_eq!(revoked.state, SessionState::Revoked);
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

    Ok(store)
}
