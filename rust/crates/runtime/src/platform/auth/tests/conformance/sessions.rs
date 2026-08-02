use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use trellis_protocol::ParticipantKindV1;

use super::fixtures::{
    digest, evidence_scope, participant_fixture, session_public_key, test_session_creation,
    test_session_revocation, NOW,
};
use crate::platform::auth::application::repository::{AccountRepository, SessionRepository};
use crate::platform::auth::authority::{
    AuthorityEvidenceRepository, AuthorityRepository, ContextRepository,
};
use crate::platform::auth::domain::{
    AuthorityDecision, AuthorityKind, AuthorityState, AuthorityTarget, AuthorizationStateError,
    DesiredAuthorityRecord, IdentityAuthorityRecord, NewSession, PrincipalKind, PrincipalRecord,
    PrincipalState, SessionRecord,
};
use crate::platform::auth::issuance::AuthorizationStateService;
use crate::platform::auth::sqlite::SqliteAuthorizationStore;

#[tokio::test]
async fn invalid_session_relationship_errors_are_rejected() -> Result<(), Box<dyn std::error::Error>>
{
    let errors =
        invalid_session_relationship_errors(SqliteAuthorizationStore::open_in_memory()?).await?;
    assert_eq!(
        errors,
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

async fn invalid_session_relationship_errors(
    store: SqliteAuthorizationStore,
) -> Result<Vec<AuthorizationStateError>, Box<dyn std::error::Error>> {
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
async fn sqlite_session_denials_do_not_rewrite_shared_authority(
) -> Result<(), Box<dyn std::error::Error>> {
    exercise_session_denials(SqliteAuthorizationStore::open_in_memory()?).await
}

async fn exercise_session_denials(
    store: SqliteAuthorizationStore,
) -> Result<(), Box<dyn std::error::Error>> {
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
    let authority = IdentityAuthorityRecord {
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
    };
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
        let desired_authority = (session_id == "ses_denied")
            .then(|| DesiredAuthorityRecord::Identity(authority.clone()));
        store
            .create_session(test_session_creation(session, desired_authority, None))
            .await?;
    }
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

    Ok(())
}
