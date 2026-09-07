use serde_json::json;
use trellis_protocol::{GrantSet, PlatformPrivilege};
use ulid::Ulid;

use super::fixtures::NOW;
use crate::platform::auth::rpc::rpc_idempotency;
use crate::platform::auth::{
    AccountRepository, AuthorizationStateError, GrantBinding, GrantBindingState, GrantOwnerKind,
    OutboxRepository, PrincipalKind, PrincipalRecord, PrincipalState, SqliteAuthorizationStore,
};

#[tokio::test]
async fn current_grants_replace_atomically_and_revocation_reserves_the_revision(
) -> Result<(), Box<dyn std::error::Error>> {
    let store = SqliteAuthorizationStore::open_in_memory()?;
    let owner = Ulid::new().to_string();
    store
        .create_principal(PrincipalRecord {
            principal_id: owner.clone(),
            kind: PrincipalKind::User,
            state: PrincipalState::Active,
            created_at: NOW,
            updated_at: NOW,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        })
        .await?;
    let participant = crate::platform::auth::builtins::cli_participant_binding(NOW)?;
    let participant_id = participant.participant_id.clone();
    let installed = store
        .install_participant(
            participant.clone(),
            0,
            rpc_idempotency(
                "Auth.Participants.Install",
                &owner,
                &Ulid::new().to_string(),
                &json!({"participant": participant.artifact_digest}),
                NOW,
            )?,
        )
        .await?;
    assert_eq!(installed["revision"], 1);
    assert!(store
        .get_grant_binding(GrantOwnerKind::User, owner.clone(), participant_id.clone())
        .await?
        .is_none());

    let binding = GrantBinding {
        owner_kind: GrantOwnerKind::User,
        owner_id: owner.clone(),
        participant_id: participant_id.clone(),
        installed_revision: 1,
        grants: participant.resolve()?.select_grants(&[])?,
        platform_privileges: vec![
            PlatformPrivilege::Delegate,
            PlatformPrivilege::Admin,
            PlatformPrivilege::Admin,
        ],
        revision: 1,
        state: GrantBindingState::Active,
        expires_at: None,
        provenance: None,
    };
    let key = Ulid::new().to_string();
    let input = serde_json::to_value(&binding)?;
    let idempotency = rpc_idempotency("Auth.Grants.Set", &owner, &key, &input, NOW)?;
    let first = store
        .set_grant_binding(binding.clone(), 0, idempotency.clone())
        .await?;
    assert_eq!(first["binding"]["revision"], 1);
    assert_eq!(
        first["binding"]["platformPrivileges"],
        json!(["trellis.auth::admin", "trellis.auth::capabilities.delegate"])
    );
    assert_eq!(
        store
            .set_grant_binding(binding.clone(), 0, idempotency)
            .await?,
        first
    );
    assert_eq!(
        store.list_ready_post_commit_actions(NOW, 100).await?.len(),
        1
    );
    assert!(matches!(
        store
            .set_grant_binding(
                binding.clone(),
                0,
                rpc_idempotency(
                    "Auth.Grants.Set",
                    &owner,
                    &Ulid::new().to_string(),
                    &input,
                    NOW
                )?
            )
            .await,
        Err(AuthorizationStateError::RevisionConflict {
            expected: 0,
            current: 1
        })
    ));

    let mut reduced = binding.clone();
    reduced.grants = GrantSet::new(Vec::new());
    reduced.platform_privileges.clear();
    let replacement = store
        .set_grant_binding(
            reduced.clone(),
            1,
            rpc_idempotency(
                "Auth.Grants.Set",
                &owner,
                &Ulid::new().to_string(),
                &serde_json::to_value(&reduced)?,
                NOW,
            )?,
        )
        .await?;
    assert_eq!(replacement["binding"]["revision"], 2);
    assert_eq!(replacement["binding"]["grants"]["permissions"], json!([]));
    assert_eq!(replacement["binding"]["platformPrivileges"], json!([]));
    let revoked = store
        .revoke_grant_binding(
            GrantOwnerKind::User,
            owner.clone(),
            participant_id.clone(),
            2,
            rpc_idempotency(
                "Auth.Grants.Revoke",
                &owner,
                &Ulid::new().to_string(),
                &json!({"participantId": participant_id}),
                NOW,
            )?,
        )
        .await?;
    assert_eq!(revoked["binding"]["revision"], 3);
    assert_eq!(revoked["binding"]["state"], "revoked");
    assert!(matches!(
        store
            .set_grant_binding(
                binding,
                0,
                rpc_idempotency(
                    "Auth.Grants.Set",
                    &owner,
                    &Ulid::new().to_string(),
                    &input,
                    NOW
                )?
            )
            .await,
        Err(AuthorizationStateError::RevisionConflict {
            expected: 0,
            current: 3
        })
    ));
    assert_eq!(
        store.list_ready_post_commit_actions(NOW, 100).await?.len(),
        3
    );
    Ok(())
}
