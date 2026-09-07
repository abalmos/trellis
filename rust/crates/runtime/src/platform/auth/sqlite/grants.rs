use std::collections::BTreeMap;

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use trellis_protocol::{compare_api_replacement, parse_api};

use super::super::context::{
    revoke_sql_contexts, AuthorizationContextRevocationReason, AuthorizationContextSelector,
};
use super::super::{
    AuthorizationStateError, GrantBinding, GrantBindingState, GrantOwnerKind,
    IdempotencyResultRecord, ParticipantBindingRecord, PostCommitActionKind,
    PostCommitActionRecord,
};
use super::common::{
    decode_enum, decode_json, encode_enum, encode_json, map_write_error, sql_error,
};
use super::deployments::{load_deployment_profile, upsert_deployment_profile_evidence};
use super::evidence::{load_deployment, load_participant_binding};
use super::outbox::{insert_sql_idempotency_and_actions, sqlite_idempotency_replay};
use super::principals::load_principal;
use super::SqliteAuthorizationStore;

pub(in crate::platform::auth) fn load_grant_binding(
    connection: &Connection,
    owner_kind: GrantOwnerKind,
    owner_id: &str,
    participant_id: &str,
) -> Result<Option<GrantBinding>, AuthorizationStateError> {
    let binding = connection
        .query_row(
            "SELECT owner_kind, owner_id, participant_id, installed_revision, grants_json,
                platform_privileges_json, revision, state, expires_at, provenance_json
         FROM auth_grant_bindings WHERE owner_kind = ?1 AND owner_id = ?2 AND participant_id = ?3",
            params![encode_enum(owner_kind)?, owner_id, participant_id],
            |row| {
                Ok(GrantBinding {
                    owner_kind: decode_enum(row.get::<_, String>(0)?)?,
                    owner_id: row.get(1)?,
                    participant_id: row.get(2)?,
                    installed_revision: row.get(3)?,
                    grants: decode_json(row.get::<_, String>(4)?)?,
                    platform_privileges: decode_json(row.get::<_, String>(5)?)?,
                    revision: row.get(6)?,
                    state: decode_enum(row.get::<_, String>(7)?)?,
                    expires_at: row.get(8)?,
                    provenance: row
                        .get::<_, Option<String>>(9)?
                        .map(decode_json)
                        .transpose()?,
                })
            },
        )
        .optional()
        .map_err(sql_error)?;
    if let Some(binding) = &binding {
        let mut canonical = binding.clone();
        canonical.validate()?;
        if canonical != *binding {
            return Err(AuthorizationStateError::InvalidRecord(
                "stored grant binding is not canonical".to_owned(),
            ));
        }
    }
    Ok(binding)
}

pub(in crate::platform::auth) fn install_participant(
    connection: &Connection,
    binding: &ParticipantBindingRecord,
    expected_revision: Option<u64>,
) -> Result<u64, AuthorizationStateError> {
    binding.resolve()?;
    let current = connection
        .query_row(
            "SELECT revision, artifact_digest FROM auth_installed_participants
         WHERE participant_id = ?1 ORDER BY revision DESC LIMIT 1",
            [&binding.participant_id],
            |row| Ok((row.get::<_, u64>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(sql_error)?;
    let revision = current.as_ref().map_or(0, |(revision, _)| *revision);
    if let Some(expected) = expected_revision {
        if expected != revision {
            return Err(AuthorizationStateError::RevisionConflict {
                expected,
                current: revision,
            });
        }
    }
    if current
        .as_ref()
        .is_some_and(|(_, digest)| digest == &binding.artifact_digest)
    {
        return Ok(revision);
    }

    let api_values: BTreeMap<String, Value> = serde_json::from_str(&binding.api_artifacts_json)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    for (id, value) in api_values {
        let candidate = parse_api(&value)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let mut statement = connection
            .prepare(
                "SELECT DISTINCT api.value FROM auth_installed_participants installed
             JOIN auth_participant_bindings binding
               ON binding.participant_id = installed.participant_id
              AND binding.artifact_digest = installed.artifact_digest,
             json_each(binding.api_artifacts_json) api WHERE api.key = ?1",
            )
            .map_err(sql_error)?;
        let previous = statement
            .query_map([&id], |row| row.get::<_, String>(0))
            .map_err(sql_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql_error)?;
        for previous in previous {
            let previous = serde_json::from_str(&previous)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let previous = parse_api(&previous)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let report = compare_api_replacement(&previous, &candidate)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            if !report.compatible {
                return Err(AuthorizationStateError::InvalidRecord(format!(
                    "incompatible replacement for {id}; use a new API version: {:?}",
                    report.issues,
                )));
            }
        }
    }
    let revision = revision
        .checked_add(1)
        .filter(|revision| *revision <= super::super::MAX_PROTOCOL_INTEGER)
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("installed revision overflow".to_owned())
        })?;
    if load_participant_binding(
        connection,
        &binding.participant_id,
        &binding.artifact_digest,
    )?
    .is_none()
    {
        connection
            .execute(
                "INSERT INTO auth_participant_bindings
             (participant_id, participant_kind, artifact_digest, needs_digest, participant_json,
              api_artifacts_json, resolved_at, state, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'resolved', NULL)",
                params![
                    binding.participant_id,
                    encode_enum(binding.participant_kind)?,
                    binding.artifact_digest,
                    binding.needs_digest,
                    binding.participant_json,
                    binding.api_artifacts_json,
                    binding.resolved_at
                ],
            )
            .map_err(map_write_error)?;
    }
    connection
        .execute(
            "INSERT INTO auth_installed_participants (participant_id, revision, artifact_digest)
         VALUES (?1, ?2, ?3)",
            params![binding.participant_id, revision, binding.artifact_digest],
        )
        .map_err(map_write_error)?;
    Ok(revision)
}

pub(in crate::platform::auth) fn replace_grant_binding(
    connection: &Connection,
    mut binding: GrantBinding,
    expected_revision: u64,
    now: i64,
) -> Result<(GrantBinding, Vec<PostCommitActionRecord>), AuthorizationStateError> {
    let current = load_grant_binding(
        connection,
        binding.owner_kind,
        &binding.owner_id,
        &binding.participant_id,
    )?;
    let revision = current.as_ref().map_or(0, |binding| binding.revision);
    if revision != expected_revision {
        return Err(AuthorizationStateError::RevisionConflict {
            expected: expected_revision,
            current: revision,
        });
    }
    binding.revision = revision.max(1);
    binding.validate()?;
    match binding.owner_kind {
        GrantOwnerKind::Deployment => {
            let deployment = load_deployment(connection, &binding.owner_id)?
                .ok_or(AuthorizationStateError::DeploymentInactive)?;
            if deployment.participant_id != binding.participant_id {
                return Err(AuthorizationStateError::InvalidRecord(
                    "deployment is assigned to another participant".to_owned(),
                ));
            }
        }
        GrantOwnerKind::User => {
            let principal = load_principal(connection, &binding.owner_id)?
                .ok_or(AuthorizationStateError::PrincipalMissing)?;
            if principal.kind != super::super::PrincipalKind::User {
                return Err(AuthorizationStateError::InvalidRecord(
                    "user grant owner is not a user".to_owned(),
                ));
            }
        }
    }
    let digest: String = connection
        .query_row(
            "SELECT artifact_digest FROM auth_installed_participants
         WHERE participant_id = ?1 AND revision = ?2",
            params![binding.participant_id, binding.installed_revision],
            |row| row.get(0),
        )
        .optional()
        .map_err(sql_error)?
        .ok_or(AuthorizationStateError::ParticipantMissing)?;
    let participant = load_participant_binding(connection, &binding.participant_id, &digest)?
        .ok_or(AuthorizationStateError::ParticipantMissing)?
        .resolve()?;
    let allowed = participant
        .proposal()
        .required()
        .grant_set()
        .permissions()
        .iter()
        .chain(participant.proposal().optional().grant_set().permissions());
    let allowed = allowed.collect::<Vec<_>>();
    if binding
        .grants
        .permissions()
        .iter()
        .any(|permission| !allowed.contains(&permission))
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "grant contains a permission outside the installed participant definitions".to_owned(),
        ));
    }
    if current.as_ref() == Some(&binding) {
        return Ok((binding, Vec::new()));
    }
    binding.revision = revision
        .checked_add(1)
        .filter(|revision| *revision <= super::super::MAX_PROTOCOL_INTEGER)
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("grant revision overflow".to_owned())
        })?;
    connection.execute(
        "INSERT INTO auth_grant_bindings (owner_kind, owner_id, participant_id, installed_revision,
             grants_json, platform_privileges_json, revision, state, expires_at, provenance_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(owner_kind, owner_id, participant_id) DO UPDATE SET
             installed_revision = excluded.installed_revision, grants_json = excluded.grants_json,
             platform_privileges_json = excluded.platform_privileges_json, revision = excluded.revision,
             state = excluded.state, expires_at = excluded.expires_at, provenance_json = excluded.provenance_json",
        params![encode_enum(binding.owner_kind)?, binding.owner_id, binding.participant_id,
            binding.installed_revision, encode_json(&binding.grants)?, encode_json(&binding.platform_privileges)?,
            binding.revision, encode_enum(binding.state)?, binding.expires_at,
            binding.provenance.as_ref().map(encode_json).transpose()?],
    ).map_err(map_write_error)?;
    revoke_sql_contexts(
        connection,
        &AuthorizationContextSelector::Grant(
            binding.owner_kind,
            binding.owner_id.clone(),
            binding.participant_id.clone(),
        ),
        if binding.state == GrantBindingState::Revoked {
            AuthorizationContextRevocationReason::AuthorityRevoked
        } else {
            AuthorizationContextRevocationReason::AuthorityChanged
        },
        now.div_euclid(1_000),
    )?;
    let event = PostCommitActionRecord {
        predecessor_action_id: None,
        action_id: trellis_protocol::digest_json(&json!({
            "event": "Auth.Grants.Changed", "ownerKind": binding.owner_kind,
            "ownerId": binding.owner_id, "participantId": binding.participant_id,
            "revision": binding.revision,
        }))
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        kind: PostCommitActionKind::Event,
        payload: json!({
            "eventType": "Auth.Grants.Changed",
            "eventSubject": format!("events.v1.Auth.Grants.Changed.{}", binding.owner_id),
            "eventId": ulid::Ulid::new().to_string(), "occurredAt": now, "binding": binding,
        }),
        created_at: now,
        attempts: 0,
        next_attempt_at: now,
        claimed_until: None,
        last_error: None,
    };
    Ok((binding, vec![event]))
}

impl SqliteAuthorizationStore {
    /// Atomically install definitions and apply explicit deployment permissions.
    pub(crate) async fn apply_deployment(
        &self,
        deployment_id: String,
        participant: ParticipantBindingRecord,
        optional_capabilities: Vec<String>,
        expected_revision: u64,
        mut idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &idempotency)? {
                return Ok(result);
            }
            let mut deployment = load_deployment_profile(&transaction, &deployment_id)?
                .ok_or(AuthorizationStateError::DeploymentInactive)?;
            let matching_kind = matches!(
                (deployment.kind, participant.participant_kind),
                (super::super::PrincipalKind::Service, trellis_protocol::ParticipantKind::Service)
                    | (super::super::PrincipalKind::Device, trellis_protocol::ParticipantKind::Device)
            );
            if !matching_kind || deployment.participant_id.as_ref().is_some_and(|id| id != &participant.participant_id) {
                return Err(AuthorizationStateError::InvalidRecord(
                    "participant must match the deployment kind and existing assignment".to_owned(),
                ));
            }
            let current = load_grant_binding(&transaction, GrantOwnerKind::Deployment, &deployment_id, &participant.participant_id)?;
            let current_revision = current.as_ref().map_or(0, |binding| binding.revision);
            if current_revision != expected_revision {
                return Err(AuthorizationStateError::RevisionConflict { expected: expected_revision, current: current_revision });
            }
            let resolved = participant.resolve()?;
            let grants = resolved.select_grants(&optional_capabilities)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            let installed_revision = install_participant(&transaction, &participant, None)?;
            if deployment.participant_id.is_none() {
                deployment.participant_id = Some(participant.participant_id.clone());
                deployment.updated_at = idempotency.created_at;
                deployment.version = super::validation::next_version(deployment.version)?;
                transaction.execute(
                    "UPDATE auth_deployment_profiles SET participant_id = ?1, updated_at = ?2, version = ?3
                     WHERE deployment_id = ?4 AND participant_id IS NULL",
                    params![deployment.participant_id, deployment.updated_at, deployment.version, deployment_id],
                ).map_err(map_write_error)?;
            }
            upsert_deployment_profile_evidence(&transaction, &deployment)?;
            let (binding, actions) = replace_grant_binding(&transaction, GrantBinding {
                owner_kind: GrantOwnerKind::Deployment,
                owner_id: deployment_id,
                participant_id: participant.participant_id,
                installed_revision,
                grants,
                platform_privileges: current.map_or_else(Vec::new, |binding| binding.platform_privileges),
                revision: 1,
                state: GrantBindingState::Active,
                expires_at: None,
                provenance: None,
            }, expected_revision, idempotency.created_at)?;
            let principal = super::principals::load_principal(&transaction, &deployment.deployment_id)?
                .ok_or(AuthorizationStateError::PrincipalMissing)?;
            let mut deployment_value = serde_json::to_value(&deployment)
                .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
            if deployment.state == super::super::DeploymentProfileState::Removed {
                deployment_value["state"] = json!("revoked");
            }
            deployment_value["disabledAt"] = json!(principal.disabled_at);
            deployment_value["revokedAt"] = json!(principal.revoked_at);
            idempotency.result = json!({"deployment": deployment_value, "binding": binding});
            insert_sql_idempotency_and_actions(&transaction, &idempotency, &actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(idempotency.result)
        }).await
    }

    /// Read the retained current binding for an exact owner and participant.
    pub async fn get_grant_binding(
        &self,
        owner_kind: GrantOwnerKind,
        owner_id: String,
        participant_id: String,
    ) -> Result<Option<GrantBinding>, AuthorizationStateError> {
        self.run_read(move |connection| {
            load_grant_binding(connection, owner_kind, &owner_id, &participant_id)
        })
        .await
    }

    /// Replace one administratively authorized binding with revision and replay fences.
    pub(crate) async fn set_grant_binding(
        &self,
        mut binding: GrantBinding,
        expected_revision: u64,
        mut idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &idempotency)? {
                return Ok(result);
            }
            binding.installed_revision = transaction.query_row(
                "SELECT revision FROM auth_installed_participants WHERE participant_id = ?1 ORDER BY revision DESC LIMIT 1",
                [&binding.participant_id], |row| row.get(0),
            ).optional().map_err(sql_error)?.ok_or(AuthorizationStateError::ParticipantMissing)?;
            let (binding, actions) = replace_grant_binding(
                &transaction,
                binding,
                expected_revision,
                idempotency.created_at,
            )?;
            idempotency.result = json!({"binding": binding});
            insert_sql_idempotency_and_actions(&transaction, &idempotency, &actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(idempotency.result)
        })
        .await
    }

    /// Revoke effective authority while retaining its current revisioned row.
    pub(crate) async fn revoke_grant_binding(
        &self,
        owner_kind: GrantOwnerKind,
        owner_id: String,
        participant_id: String,
        expected_revision: u64,
        mut idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &idempotency)? {
                return Ok(result);
            }
            let mut binding =
                load_grant_binding(&transaction, owner_kind, &owner_id, &participant_id)?
                    .ok_or(AuthorizationStateError::AuthorityMissing)?;
            binding.grants = trellis_protocol::GrantSet::new(Vec::new());
            binding.platform_privileges.clear();
            binding.state = GrantBindingState::Revoked;
            let (binding, actions) = replace_grant_binding(
                &transaction,
                binding,
                expected_revision,
                idempotency.created_at,
            )?;
            idempotency.result = json!({"binding": binding});
            insert_sql_idempotency_and_actions(&transaction, &idempotency, &actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(idempotency.result)
        })
        .await
    }

    /// Install canonical definitions without granting access or rotating credentials.
    pub(crate) async fn install_participant(
        &self,
        binding: ParticipantBindingRecord,
        expected_revision: u64,
        mut idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &idempotency)? {
                return Ok(result);
            }
            let revision = install_participant(&transaction, &binding, Some(expected_revision))?;
            idempotency.result = json!({"participantId": binding.participant_id, "revision": revision, "digest": binding.artifact_digest});
            insert_sql_idempotency_and_actions(&transaction, &idempotency, &[])?;
            transaction.commit().map_err(sql_error)?;
            Ok(idempotency.result)
        }).await
    }
}
