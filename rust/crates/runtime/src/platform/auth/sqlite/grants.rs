use std::collections::BTreeMap;

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use trellis_protocol::{
    ParticipantKind, ParticipantResourceKind, PermissionAction, PermissionTarget, PlatformPrivilege,
};
use trellis_runtime_apis::types::{AuthGrantsListRequestOwnerKind, AuthGrantsListRequestState};

use super::super::authority::{IssuanceConnection, IssuanceCredential};
use super::super::context::{
    load_sql_context_by_digest, revoke_sql_contexts, AuthorizationContextRevocationReason,
    AuthorizationContextSelector, AuthorizationContextState,
};
use super::super::domain::{require_protocol_timestamp, GrantBindingReplacement};
use super::super::{
    AuthorizationStateError, GrantBinding, GrantBindingState, GrantOwnerKind,
    IdempotencyResultRecord, MutationActor, ParticipantBindingRecord, PostCommitActionKind,
    PostCommitActionRecord, ResourceBindingEvidence,
};

pub(super) fn require_current_actor(
    connection: &Connection,
    actor: &MutationActor,
    require_admin: bool,
    now: i64,
) -> Result<(), AuthorizationStateError> {
    require_protocol_timestamp("now", now)?;
    let context = load_sql_context_by_digest(connection, &actor.context_digest)?
        .ok_or(AuthorizationStateError::NotAuthorized)?;
    let now_seconds = now.div_euclid(1_000);
    if context.state != AuthorizationContextState::Active
        || context.revoked_at.is_some()
        || context.not_before > now_seconds
        || context.expires_at <= now_seconds
        || context.principal_id != actor.principal_id
        || context.participant_id != actor.participant_id
        || context.owner_kind != actor.owner_kind
        || context.owner_id != actor.owner_id
        || context.grant_revision != actor.grant_revision
        || context.login_session_id != actor.login_session_id
        || context.session_public_key != actor.session_public_key
    {
        return Err(AuthorizationStateError::NotAuthorized);
    }
    let credential = match (&context.login_session_id, &context.identity_key_id) {
        (Some(login_session_id), None) => IssuanceCredential::Login(login_session_id.clone()),
        (None, Some(identity_key_id)) => IssuanceCredential::Native(identity_key_id.clone()),
        _ => return Err(AuthorizationStateError::NotAuthorized),
    };
    let mut snapshot = super::contexts::sqlite_issuance_snapshot(
        connection,
        &IssuanceConnection {
            credential,
            connection_id: context.connection_id.clone(),
            session_public_key: context.session_public_key.clone(),
        },
    )?;
    snapshot.issuer = super::contexts::load_eligible_authorization_issuer(
        connection,
        &context.issuer_key_id,
        now,
    )?;
    let current = super::super::issuance::resolve_snapshot(snapshot, now)?;
    if current.principal_id != context.principal_id
        || current.binding.owner_kind != context.owner_kind
        || current.binding.owner_id != context.owner_id
        || current.participant.participant_id != context.participant_id
        || current.binding.revision != context.grant_revision
        || current.binding.installed_revision != context.installed_revision
        || current.session_public_key != context.session_public_key
        || current.login_session_id != context.login_session_id
    {
        return Err(AuthorizationStateError::NotAuthorized);
    }
    if require_admin
        && !current
            .binding
            .platform_privileges
            .contains(&PlatformPrivilege::Admin)
    {
        return Err(AuthorizationStateError::NotAuthorized);
    }
    Ok(())
}
use super::common::{
    decode_enum, decode_json, encode_enum, encode_json, map_write_error, sql_error,
};
use super::deployments::{load_deployment_profile, upsert_deployment_profile_evidence};
use super::evidence::load_deployment;
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
                platform_privileges_json, revision, state, expires_at, provenance_json,
                created_at, updated_at
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
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
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
    if expected_revision.is_some_and(|revision| revision > super::super::MAX_PROTOCOL_INTEGER) {
        return Err(AuthorizationStateError::InvalidRecord(
            "expectedRevision exceeds safe integer range".to_owned(),
        ));
    }
    let platform_trusted = connection
        .query_row(
            "SELECT platform_trusted FROM auth_package_evidence WHERE package_digest = ?1",
            [&binding.package_digest],
            |row| row.get::<_, bool>(0),
        )
        .optional()
        .map_err(sql_error)?
        .unwrap_or(false);
    super::super::builtins::validate_binding_namespace(binding, platform_trusted)?;
    binding.resolve()?;
    if let Some((_, installed)) =
        load_installed_participant(connection, &binding.participant_id, None)?
    {
        if installed.participant_kind != binding.participant_kind {
            return Err(AuthorizationStateError::InvalidRecord(
                "installed participant kind cannot change".to_owned(),
            ));
        }
    }
    let current = connection
        .query_row(
            "SELECT revision FROM auth_installed_participants
         WHERE participant_id = ?1 ORDER BY revision DESC LIMIT 1",
            [&binding.participant_id],
            |row| row.get::<_, u64>(0),
        )
        .optional()
        .map_err(sql_error)?;
    let revision = current.unwrap_or(0);
    if let Some(expected) = expected_revision {
        if expected != revision {
            return Err(AuthorizationStateError::RevisionConflict {
                expected,
                current: revision,
            });
        }
    }
    let identical = connection
        .query_row(
            "SELECT revision FROM auth_installed_participants
               WHERE participant_id = ?1 AND participant_kind = ?2 AND participant_digest = ?3
                AND needs_digest = ?4 AND package_digest = ?5 AND participant_path = ?6
                AND projection_json = ?7
              ORDER BY revision DESC LIMIT 1",
            params![
                binding.participant_id,
                encode_enum(binding.participant_kind)?,
                binding.participant_digest,
                binding.needs_digest,
                binding.package_digest,
                binding.participant_path,
                encode_json(&binding.projection)?
            ],
            |row| row.get::<_, u64>(0),
        )
        .optional()
        .map_err(sql_error)?;
    if let Some(revision) = identical {
        return Ok(revision);
    }

    let revision = revision
        .checked_add(1)
        .filter(|revision| *revision <= super::super::MAX_PROTOCOL_INTEGER)
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("installed revision overflow".to_owned())
        })?;
    connection
        .execute(
            "INSERT INTO auth_installed_participants
               (participant_id, revision, participant_kind, participant_digest, needs_digest,
                installed_at, package_digest, participant_path, projection_json)
              VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                binding.participant_id,
                revision,
                encode_enum(binding.participant_kind)?,
                binding.participant_digest,
                binding.needs_digest,
                binding.resolved_at,
                binding.package_digest,
                binding.participant_path,
                encode_json(&binding.projection)?
            ],
        )
        .map_err(map_write_error)?;
    Ok(revision)
}

fn participant_install_event(
    binding: &ParticipantBindingRecord,
    revision: u64,
    now: i64,
) -> Result<PostCommitActionRecord, AuthorizationStateError> {
    Ok(PostCommitActionRecord {
        predecessor_action_id: None,
        action_id: trellis_protocol::digest_json(&json!({
            "event": "Auth.Participants.Installed",
            "participantId": binding.participant_id,
            "revision": revision,
        }))
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        kind: PostCommitActionKind::Event,
        payload: json!({
            "eventType": "Auth.Participants.Installed",
            "eventSubject": format!(
                "events.v1.Auth.Participants.Installed.{}",
                binding.participant_id
            ),
            "eventId": ulid::Ulid::new().to_string(),
            "occurredAt": now,
            "participantId": binding.participant_id,
            "participantKind": binding.participant_kind,
            "revision": revision,
            "participantDigest": binding.participant_digest,
        }),
        created_at: now,
        attempts: 0,
        next_attempt_at: now,
        claimed_until: None,
        last_error: None,
    })
}

pub(in crate::platform::auth) fn load_installed_participant(
    connection: &Connection,
    participant_id: &str,
    revision: Option<u64>,
) -> Result<Option<(u64, ParticipantBindingRecord)>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT revision, participant_id, participant_kind, participant_digest, needs_digest,
                installed_at, package_digest, participant_path, projection_json
         FROM auth_installed_participants WHERE participant_id = ?1
           AND (?2 IS NULL OR revision = ?2) ORDER BY revision DESC LIMIT 1",
            params![participant_id, revision],
            |row| {
                Ok((
                    row.get::<_, u64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            },
        )
        .optional()
        .map_err(sql_error)?
        .map(
            |(
                revision,
                participant_id,
                participant_kind,
                participant_digest,
                needs_digest,
                resolved_at,
                package_digest,
                participant_path,
                projection_json,
            )| {
                let binding = ParticipantBindingRecord {
                    participant_id,
                    participant_kind: decode_enum(participant_kind).map_err(sql_error)?,
                    participant_digest,
                    needs_digest,
                    package_digest,
                    participant_path,
                    projection: decode_json(projection_json).map_err(sql_error)?,
                    resolved_at,
                    state: super::super::ParticipantBindingState::Resolved,
                    error: None,
                };
                binding.resolve()?;
                Ok((revision, binding))
            },
        )
        .transpose()
}

pub(in crate::platform::auth) fn accept_package_evidence(
    connection: &Connection,
    package_digest: &str,
    root_package: &str,
    evidence_json: &str,
    platform_trust: bool,
    actor: Option<&MutationActor>,
    now: i64,
) -> Result<(), AuthorizationStateError> {
    if platform_trust && root_package != "trellis" {
        return Err(AuthorizationStateError::InvalidRecord(
            "platformTrust is only valid for the reserved Trellis package".to_owned(),
        ));
    }
    let internal_trust = platform_trust
        && actor.is_none()
        && super::super::builtins::is_trusted_package_evidence(package_digest, evidence_json);
    let trust_actor = if platform_trust && !internal_trust {
        let actor = actor.ok_or(AuthorizationStateError::NotAuthorized)?;
        require_current_actor(connection, actor, true, now)?;
        Some(actor)
    } else {
        None
    };
    let current = connection
        .query_row(
            "SELECT evidence_json, platform_trusted FROM auth_package_evidence WHERE package_digest = ?1",
            [package_digest],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?)),
        )
        .optional()
        .map_err(sql_error)?;
    if let Some((stored, trusted)) = current {
        if stored != evidence_json {
            return Err(AuthorizationStateError::InvalidRecord(
                "package evidence body disagrees with the cached immutable digest".to_owned(),
            ));
        }
        if platform_trust && !trusted {
            let trusted_by =
                trust_actor.map_or("trellis.runtime", |actor| actor.principal_id.as_str());
            connection
                .execute(
                    "UPDATE auth_package_evidence SET platform_trusted = 1, trusted_at = ?2, trusted_by = ?3 WHERE package_digest = ?1",
                    params![package_digest, now, trusted_by],
                )
                .map_err(map_write_error)?;
        }
    } else {
        if root_package == "trellis" && !platform_trust {
            return Err(AuthorizationStateError::NotAuthorized);
        }
        connection
            .execute(
                "INSERT INTO auth_package_evidence (package_digest, evidence_json, platform_trusted, accepted_at, trusted_at, trusted_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    package_digest,
                    evidence_json,
                    platform_trust,
                    now,
                    platform_trust.then_some(now),
                    platform_trust.then(|| actor.map_or_else(|| "trellis.runtime".to_owned(), |value| value.principal_id.clone())),
                ],
            )
            .map_err(map_write_error)?;
    }
    if root_package == "trellis" {
        let trusted = connection
            .query_row(
                "SELECT platform_trusted FROM auth_package_evidence WHERE package_digest = ?1",
                [package_digest],
                |row| row.get::<_, bool>(0),
            )
            .map_err(sql_error)?;
        if !trusted {
            return Err(AuthorizationStateError::NotAuthorized);
        }
    }
    Ok(())
}

pub(in crate::platform::auth) fn replace_grant_binding(
    connection: &Connection,
    replacement: GrantBindingReplacement,
    now: i64,
) -> Result<(GrantBinding, Vec<PostCommitActionRecord>), AuthorizationStateError> {
    if replacement.expected_revision > super::super::MAX_PROTOCOL_INTEGER {
        return Err(AuthorizationStateError::InvalidRecord(
            "expectedRevision exceeds safe integer range".to_owned(),
        ));
    }
    let current = load_grant_binding(
        connection,
        replacement.owner_kind,
        &replacement.owner_id,
        &replacement.participant_id,
    )?;
    let revision = current.as_ref().map_or(0, |binding| binding.revision);
    if revision != replacement.expected_revision {
        return Err(AuthorizationStateError::RevisionConflict {
            expected: replacement.expected_revision,
            current: revision,
        });
    }
    if let Some(expected) = replacement.expected_current_installed_revision {
        let current_installed_revision = connection
            .query_row(
                "SELECT revision FROM auth_installed_participants
                 WHERE participant_id = ?1 ORDER BY revision DESC LIMIT 1",
                [&replacement.participant_id],
                |row| row.get::<_, u64>(0),
            )
            .optional()
            .map_err(sql_error)?
            .unwrap_or(0);
        if current_installed_revision != expected {
            return Err(AuthorizationStateError::RevisionConflict {
                expected,
                current: current_installed_revision,
            });
        }
    }
    let (installed_revision, participant) = load_installed_participant(
        connection,
        &replacement.participant_id,
        Some(replacement.installed_revision),
    )?
    .ok_or(AuthorizationStateError::ParticipantMissing)?;
    let mut binding = GrantBinding {
        owner_kind: replacement.owner_kind,
        owner_id: replacement.owner_id,
        participant_id: replacement.participant_id,
        installed_revision,
        grants: replacement.grants,
        platform_privileges: replacement.platform_privileges,
        revision: revision.max(1),
        state: replacement.state,
        expires_at: replacement.expires_at,
        provenance: replacement.provenance,
        created_at: current.as_ref().map_or(now, |binding| binding.created_at),
        updated_at: current.as_ref().map_or(now, |binding| binding.updated_at),
    };
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
            let profile = load_deployment_profile(connection, &binding.owner_id)?
                .ok_or(AuthorizationStateError::DeploymentInactive)?;
            if !matches!(
                (profile.kind, participant.participant_kind),
                (
                    super::super::PrincipalKind::Service,
                    ParticipantKind::Service
                ) | (super::super::PrincipalKind::Device, ParticipantKind::Device)
            ) {
                return Err(AuthorizationStateError::InvalidRecord(
                    "deployment and installed participant kinds differ".to_owned(),
                ));
            }
        }
        GrantOwnerKind::User => {
            let principal = load_principal(connection, &binding.owner_id)?
                .ok_or(AuthorizationStateError::PrincipalMissing)?;
            if principal.kind != super::super::PrincipalKind::User
                || !matches!(
                    participant.participant_kind,
                    ParticipantKind::App | ParticipantKind::Agent
                )
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "user grant owner is not a user".to_owned(),
                ));
            }
        }
    }
    let resolved_participant = participant.resolve()?;
    let allowed = resolved_participant
        .required_grants
        .permissions()
        .iter()
        .chain(
            resolved_participant
                .optional_grant_bundles
                .values()
                .flat_map(|grant| grant.permissions()),
        );
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
    for permission in binding.grants.permissions() {
        if matches!(
            permission.target(),
            PermissionTarget::ParticipantResource {
                resource: ParticipantResourceKind::Kv | ParticipantResourceKind::Store,
                ..
            }
        ) {
            let paired_action = match permission.action() {
                PermissionAction::Write => PermissionAction::Delete,
                PermissionAction::Delete => PermissionAction::Write,
                _ => continue,
            };
            if !binding.grants.permissions().iter().any(|paired| {
                paired.target() == permission.target() && paired.action() == paired_action
            }) {
                return Err(AuthorizationStateError::InvalidRecord(
                    "direct KV/store write and delete permissions must be granted together"
                        .to_owned(),
                ));
            }
        }
    }
    if current.as_ref().is_some_and(|current| {
        current.owner_kind == GrantOwnerKind::User
            && current
                .platform_privileges
                .contains(&PlatformPrivilege::Admin)
            && (binding.state != GrantBindingState::Active
                || !binding
                    .platform_privileges
                    .contains(&PlatformPrivilege::Admin))
    }) && connection
        .query_row(
            "SELECT 1 FROM auth_bootstrap_administrator WHERE singleton = 1 AND principal_id = ?1",
            [&binding.owner_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(sql_error)?
        .is_some()
    {
        return Err(AuthorizationStateError::NotAuthorized);
    }
    if current.as_ref() == Some(&binding) {
        return Ok((binding, Vec::new()));
    }
    binding.updated_at = now;
    binding.validate()?;
    binding.revision = if current.is_none() {
        1
    } else {
        revision
            .checked_add(1)
            .filter(|revision| *revision <= super::super::MAX_PROTOCOL_INTEGER)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("grant revision overflow".to_owned())
            })?
    };
    connection.execute(
        "INSERT INTO auth_grant_bindings (owner_kind, owner_id, participant_id, installed_revision,
             grants_json, platform_privileges_json, revision, state, expires_at, provenance_json,
             created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(owner_kind, owner_id, participant_id) DO UPDATE SET
             installed_revision = excluded.installed_revision, grants_json = excluded.grants_json,
             platform_privileges_json = excluded.platform_privileges_json, revision = excluded.revision,
              state = excluded.state, expires_at = excluded.expires_at, provenance_json = excluded.provenance_json,
              updated_at = excluded.updated_at",
        params![encode_enum(binding.owner_kind)?, binding.owner_id, binding.participant_id,
            binding.installed_revision, encode_json(&binding.grants)?, encode_json(&binding.platform_privileges)?,
            binding.revision, encode_enum(binding.state)?, binding.expires_at,
            binding.provenance.as_ref().map(encode_json).transpose()?, binding.created_at, binding.updated_at],
    ).map_err(map_write_error)?;
    if binding.owner_kind == GrantOwnerKind::User {
        let resources =
            super::super::resources::identity_resources(&participant, &binding.owner_id, now)?;
        super::evidence::replace_sql_resource_bindings(
            connection,
            binding.owner_kind,
            &binding.owner_id,
            &binding.participant_id,
            binding.installed_revision,
            &resources,
        )?;
    }
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
    pub(crate) async fn user_is_admin(
        &self,
        user_id: String,
    ) -> Result<bool, AuthorizationStateError> {
        self.run_read(move |connection| {
            let mut statement = connection
                .prepare(
                    "SELECT platform_privileges_json FROM auth_grant_bindings
                     WHERE owner_kind = 'user' AND owner_id = ?1 AND state = 'active'",
                )
                .map_err(sql_error)?;
            let rows = statement
                .query_map([user_id], |row| row.get::<_, String>(0))
                .map_err(sql_error)?;
            for row in rows {
                let privileges: Vec<PlatformPrivilege> =
                    decode_json(row.map_err(sql_error)?).map_err(sql_error)?;
                if privileges.contains(&PlatformPrivilege::Admin) {
                    return Ok(true);
                }
            }
            Ok(false)
        })
        .await
    }

    pub(crate) async fn get_installed_participant_record(
        &self,
        participant_id: String,
        revision: Option<u64>,
    ) -> Result<Option<(u64, ParticipantBindingRecord)>, AuthorizationStateError> {
        self.run_read(move |connection| {
            load_installed_participant(connection, &participant_id, revision)
        })
        .await
    }

    pub(crate) async fn put_participant_binding(
        &self,
        participant: ParticipantBindingRecord,
    ) -> Result<u64, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            let evidence_json =
                super::super::builtins::trusted_package_evidence_json(&participant.package_digest)?
                    .ok_or(AuthorizationStateError::NotAuthorized)?;
            accept_package_evidence(
                &transaction,
                &participant.package_digest,
                "trellis",
                &evidence_json,
                true,
                None,
                participant.resolved_at,
            )?;
            let revision = install_participant(&transaction, &participant, None)?;
            transaction.commit().map_err(sql_error)?;
            Ok(revision)
        })
        .await
    }

    /// Inspect the exact retained installed snapshot, or the latest revision.
    pub(crate) async fn get_installed_participant(
        &self,
        participant_id: String,
        revision: Option<u64>,
    ) -> Result<Value, AuthorizationStateError> {
        if revision
            .is_some_and(|revision| revision == 0 || revision > super::super::MAX_PROTOCOL_INTEGER)
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "invalid installed revision".to_owned(),
            ));
        }
        self.run_read(move |connection| {
            let (revision, binding) = load_installed_participant(connection, &participant_id, revision)?
                .ok_or(AuthorizationStateError::ParticipantMissing)?;
            let resolved = binding.resolve()?;
            let package_evidence: Value = connection.query_row(
                "SELECT evidence_json FROM auth_package_evidence WHERE package_digest = ?1",
                [&binding.package_digest],
                |row| row.get::<_, String>(0),
            ).map_err(sql_error).and_then(|value| decode_json(value).map_err(sql_error))?;
            Ok(json!({"participant": {
                "participantId": binding.participant_id, "participantKind": binding.participant_kind,
                 "revision": revision, "participantDigest": binding.participant_digest, "installedAt": binding.resolved_at,
                "packageDigest": binding.package_digest, "participantPath": binding.participant_path,
                "packageEvidence": package_evidence,
                "requiredGrants": resolved.required_grants,
                "optionalBundles": resolved.optional_grant_bundles.iter().map(|(id, grant)| json!({
                    "id": id, "permissions": grant.permissions(),
                })).collect::<Vec<_>>(),
            }}))
        }).await
    }

    /// List one caller-scoped page in stable owner/participant tuple order.
    pub(crate) async fn list_grant_bindings(
        &self,
        request: trellis_runtime_apis::types::AuthGrantsListRequest,
    ) -> Result<Value, AuthorizationStateError> {
        let owner_kind = request
            .owner_kind
            .map(|kind| match kind {
                AuthGrantsListRequestOwnerKind::User => Ok(GrantOwnerKind::User),
                AuthGrantsListRequestOwnerKind::Deployment => Ok(GrantOwnerKind::Deployment),
                AuthGrantsListRequestOwnerKind::Unknown(_) => Err(
                    AuthorizationStateError::InvalidRecord("unknown grant owner kind".to_owned()),
                ),
            })
            .transpose()?;
        let state = request
            .state
            .map(|state| match state {
                AuthGrantsListRequestState::Active => Ok(GrantBindingState::Active),
                AuthGrantsListRequestState::Revoked => Ok(GrantBindingState::Revoked),
                AuthGrantsListRequestState::Unknown(_) => Err(
                    AuthorizationStateError::InvalidRecord("unknown grant state".to_owned()),
                ),
            })
            .transpose()?;
        let limit = request.limit.map_or(100, |limit| limit.0 .0);
        if !(1..=500).contains(&limit) {
            return Err(AuthorizationStateError::InvalidRecord(
                "limit must be between 1 and 500".to_owned(),
            ));
        }
        let offset = match request.cursor {
            Some(cursor)
                if !cursor.0.is_empty() && cursor.0.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                cursor.0.parse::<i64>().map_err(|_| {
                    AuthorizationStateError::InvalidRecord("invalid cursor".to_owned())
                })?
            }
            Some(_) => {
                return Err(AuthorizationStateError::InvalidRecord(
                    "invalid cursor".to_owned(),
                ))
            }
            None => 0,
        };
        let next = offset
            .checked_add(limit)
            .ok_or_else(|| AuthorizationStateError::InvalidRecord("cursor overflow".to_owned()))?;
        self.run_read(move |connection| {
            let transaction = connection.unchecked_transaction().map_err(sql_error)?;
            let keys = {
                let mut statement = transaction
                    .prepare(
                        "SELECT owner_kind, owner_id, participant_id FROM auth_grant_bindings
                     WHERE (?1 IS NULL OR owner_kind = ?1) AND (?2 IS NULL OR owner_id = ?2)
                       AND (?3 IS NULL OR participant_id = ?3) AND (?4 IS NULL OR state = ?4)
                     ORDER BY owner_kind, owner_id, participant_id LIMIT ?5 OFFSET ?6",
                    )
                    .map_err(sql_error)?;
                let keys = statement
                    .query_map(
                        params![
                            owner_kind.map(encode_enum).transpose()?,
                            request.owner_id.map(|owner| owner.0),
                            request.participant_id.map(|participant| participant.0),
                            state.map(encode_enum).transpose()?,
                            limit + 1,
                            offset
                        ],
                        |row| {
                            Ok((
                                decode_enum(row.get::<_, String>(0)?)?,
                                row.get::<_, String>(1)?,
                                row.get::<_, String>(2)?,
                            ))
                        },
                    )
                    .map_err(sql_error)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(sql_error)?;
                keys
            };
            let has_more = keys.len() > limit as usize;
            let entries = keys
                .into_iter()
                .take(limit as usize)
                .map(|(kind, owner, participant)| {
                    load_grant_binding(&transaction, kind, &owner, &participant)?
                        .ok_or(AuthorizationStateError::StorageConflict)
                })
                .collect::<Result<Vec<_>, _>>()?;
            transaction.commit().map_err(sql_error)?;
            Ok(json!({"entries": entries, "nextCursor": has_more.then(|| next.to_string())}))
        })
        .await
    }

    /// Atomically install definitions and apply explicit deployment permissions.
    pub(crate) async fn apply_deployment(
        &self,
        actor: MutationActor,
        deployment_id: String,
        deployment_evidence: (
            ParticipantBindingRecord,
            String,
            String,
            Vec<String>,
            Vec<ResourceBindingEvidence>,
        ),
        expected_revision: u64,
        mut idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        let (participant, root_package, evidence_json, optional_capabilities, resource_evidence) =
            deployment_evidence;
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            require_current_actor(&transaction, &actor, true, idempotency.created_at)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &idempotency)? {
                return Ok(result);
            }
            accept_package_evidence(&transaction, &participant.package_digest, &root_package, &evidence_json, false, None, idempotency.created_at)?;
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
            let grants = resolved.select_grants(&optional_capabilities)?;
            let previous_installed_revision = load_installed_participant(
                &transaction,
                &participant.participant_id,
                None,
            )?
            .map(|(revision, _)| revision);
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
            let (binding, mut actions) = replace_grant_binding(&transaction, GrantBindingReplacement {
                owner_kind: GrantOwnerKind::Deployment,
                owner_id: deployment_id,
                participant_id: participant.participant_id.clone(),
                installed_revision,
                grants,
                platform_privileges: current.as_ref().map_or_else(Vec::new, |binding| binding.platform_privileges.clone()),
                expected_revision,
                expected_current_installed_revision: None,
                state: GrantBindingState::Active,
                expires_at: current.as_ref().and_then(|binding| binding.expires_at),
                provenance: None,
            }, idempotency.created_at)?;
            if !actions.is_empty() {
                super::evidence::replace_sql_resource_bindings(
                    &transaction,
                    GrantOwnerKind::Deployment,
                    &binding.owner_id,
                    &binding.participant_id,
                    installed_revision,
                    &resource_evidence,
                )?;
            }
            if previous_installed_revision != Some(installed_revision) {
                actions.insert(
                    0,
                    participant_install_event(
                        &participant,
                        installed_revision,
                        idempotency.created_at,
                    )?,
                );
            }
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
        replacement: GrantBindingReplacement,
        mut idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &idempotency)? {
                return Ok(result);
            }
            let (binding, actions) =
                replace_grant_binding(&transaction, replacement, idempotency.created_at)?;
            idempotency.result = json!({"binding": binding});
            insert_sql_idempotency_and_actions(&transaction, &idempotency, &actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(idempotency.result)
        })
        .await
    }

    pub(crate) async fn admin_set_grant_binding(
        &self,
        actor: MutationActor,
        replacement: GrantBindingReplacement,
        mut idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            require_current_actor(&transaction, &actor, true, idempotency.created_at)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &idempotency)? {
                return Ok(result);
            }
            let (binding, actions) =
                replace_grant_binding(&transaction, replacement, idempotency.created_at)?;
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
        actor: MutationActor,
        owner_kind: GrantOwnerKind,
        owner_id: String,
        participant_id: String,
        expected_revision: u64,
        mut idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            require_current_actor(
                &transaction,
                &actor,
                owner_kind != GrantOwnerKind::User || owner_id != actor.principal_id,
                idempotency.created_at,
            )?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &idempotency)? {
                return Ok(result);
            }
            let binding = load_grant_binding(&transaction, owner_kind, &owner_id, &participant_id)?
                .ok_or(AuthorizationStateError::AuthorityMissing)?;
            let (binding, actions) = replace_grant_binding(
                &transaction,
                GrantBindingReplacement {
                    owner_kind,
                    owner_id,
                    participant_id,
                    installed_revision: binding.installed_revision,
                    grants: trellis_protocol::GrantSet::new(Vec::new()),
                    platform_privileges: Vec::new(),
                    state: GrantBindingState::Revoked,
                    expires_at: binding.expires_at,
                    provenance: binding.provenance,
                    expected_revision,
                    expected_current_installed_revision: None,
                },
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
        actor: MutationActor,
        binding: ParticipantBindingRecord,
        root_package: String,
        evidence_json: String,
        platform_trust: bool,
        expected_revision: u64,
        mut idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            require_current_actor(&transaction, &actor, true, idempotency.created_at)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &idempotency)? {
                return Ok(result);
            }
            accept_package_evidence(&transaction, &binding.package_digest, &root_package, &evidence_json, platform_trust, Some(&actor), idempotency.created_at)?;
            let revision = install_participant(&transaction, &binding, Some(expected_revision))?;
            let (_, installed) = load_installed_participant(&transaction, &binding.participant_id, Some(revision))?
                .ok_or(AuthorizationStateError::ParticipantMissing)?;
            idempotency.result = json!({"participant": {
                "participantId": installed.participant_id, "participantKind": installed.participant_kind,
                 "revision": revision, "participantDigest": installed.participant_digest, "installedAt": installed.resolved_at,
            }});
            let actions = if revision == expected_revision {
                Vec::new()
            } else {
                vec![participant_install_event(
                    &binding,
                    revision,
                    idempotency.created_at,
                )?]
            };
            insert_sql_idempotency_and_actions(&transaction, &idempotency, &actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(idempotency.result)
        }).await
    }
}

#[async_trait::async_trait]
impl super::super::GrantRepository for SqliteAuthorizationStore {
    async fn get_installed_participant_record(
        &self,
        participant_id: String,
        revision: Option<u64>,
    ) -> Result<Option<(u64, ParticipantBindingRecord)>, AuthorizationStateError> {
        SqliteAuthorizationStore::get_installed_participant_record(self, participant_id, revision)
            .await
    }

    async fn accept_presented_package(
        &self,
        input: super::super::evidence::PackageEvidenceInput,
        now: i64,
    ) -> Result<ParticipantBindingRecord, AuthorizationStateError> {
        self.run(move |connection| {
            let evidence_json = trellis_protocol::canonicalize_json(
                &serde_json::to_value(&input.package_evidence)
                    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
            )
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
            if let Some(stored) = connection
                .query_row(
                    "SELECT evidence_json FROM auth_package_evidence WHERE package_digest = ?1",
                    [&input.package_digest],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(sql_error)?
            {
                if stored != evidence_json {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "package evidence body disagrees with the cached immutable digest"
                            .to_owned(),
                    ));
                }
                return load_installed_participant_by_package(
                    connection,
                    &input.package_digest,
                    &input.participant_path,
                )?
                .ok_or(AuthorizationStateError::ParticipantMissing);
            }
            let (binding, evidence_json) =
                ParticipantBindingRecord::from_package_evidence(&input, now)?;
            accept_package_evidence(
                connection,
                &binding.package_digest,
                &input.package_evidence.root_package,
                &evidence_json,
                false,
                None,
                now,
            )?;
            Ok(binding)
        })
        .await
    }

    async fn get_credential_participant_assignment(
        &self,
        identity_key_id: String,
    ) -> Result<Option<String>, AuthorizationStateError> {
        self.run_read(move |connection| {
            connection
                .query_row(
                    "SELECT participant_id FROM auth_provisioned_identities
                     WHERE identity_key_id = ?1",
                    [identity_key_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(sql_error)
        })
        .await
    }

    async fn get_grant_binding(
        &self,
        owner_kind: GrantOwnerKind,
        owner_id: String,
        participant_id: String,
    ) -> Result<Option<GrantBinding>, AuthorizationStateError> {
        SqliteAuthorizationStore::get_grant_binding(self, owner_kind, owner_id, participant_id)
            .await
    }

    async fn set_grant_binding(
        &self,
        replacement: GrantBindingReplacement,
        idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        SqliteAuthorizationStore::set_grant_binding(self, replacement, idempotency).await
    }

    async fn set_portal_grant_binding(
        &self,
        replacement: GrantBindingReplacement,
        policy: super::super::PortalPolicySnapshot,
        idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            verify_portal_policy_snapshot(&transaction, &policy)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &idempotency)? {
                return Ok(result);
            }
            let (binding, actions) =
                replace_grant_binding(&transaction, replacement, idempotency.created_at)?;
            let mut idempotency = idempotency;
            idempotency.result = json!({"binding": binding});
            insert_sql_idempotency_and_actions(&transaction, &idempotency, &actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(idempotency.result)
        })
        .await
    }

    async fn revoke_portal_grant_binding(
        &self,
        owner_id: String,
        participant_id: String,
        expected_revision: u64,
        policy: super::super::PortalPolicySnapshot,
        mut idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            verify_portal_policy_snapshot(&transaction, &policy)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &idempotency)? {
                return Ok(result);
            }
            let binding = load_grant_binding(
                &transaction,
                GrantOwnerKind::User,
                &owner_id,
                &participant_id,
            )?
            .ok_or(AuthorizationStateError::AuthorityMissing)?;
            let (binding, actions) = replace_grant_binding(
                &transaction,
                GrantBindingReplacement {
                    owner_kind: GrantOwnerKind::User,
                    owner_id,
                    participant_id,
                    installed_revision: binding.installed_revision,
                    grants: trellis_protocol::GrantSet::new(Vec::new()),
                    platform_privileges: Vec::new(),
                    state: GrantBindingState::Revoked,
                    expires_at: binding.expires_at,
                    provenance: binding.provenance,
                    expected_revision,
                    expected_current_installed_revision: None,
                },
                idempotency.created_at,
            )?;
            idempotency.result = json!({"binding": binding});
            insert_sql_idempotency_and_actions(&transaction, &idempotency, &actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(idempotency.result)
        })
        .await
    }
}

fn load_installed_participant_by_package(
    connection: &Connection,
    package_digest: &str,
    participant_path: &str,
) -> Result<Option<ParticipantBindingRecord>, AuthorizationStateError> {
    let participant_id = connection
        .query_row(
            "SELECT participant_id FROM auth_installed_participants WHERE package_digest = ?1 AND participant_path = ?2 ORDER BY revision DESC LIMIT 1",
            params![package_digest, participant_path],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(sql_error)?;
    participant_id
        .map(|participant_id| {
            load_installed_participant(connection, &participant_id, None)
                .map(|value| value.map(|(_, binding)| binding))
        })
        .transpose()
        .map(Option::flatten)
}

fn verify_portal_policy_snapshot(
    connection: &Connection,
    snapshot: &super::super::PortalPolicySnapshot,
) -> Result<(), AuthorizationStateError> {
    if snapshot.portal_id.is_empty()
        || snapshot.participant_id.is_empty()
        || snapshot.policy_version.is_some() != snapshot.policy_fingerprint.is_some()
        || snapshot.capability_group_versions.len() != snapshot.capability_group_fingerprints.len()
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "portal policy snapshot is malformed".to_owned(),
        ));
    }
    let group_fingerprints = snapshot
        .capability_group_fingerprints
        .iter()
        .cloned()
        .collect::<BTreeMap<_, _>>();
    if group_fingerprints.len() != snapshot.capability_group_fingerprints.len()
        || snapshot
            .capability_group_versions
            .windows(2)
            .any(|pair| pair[0].0 >= pair[1].0)
        || snapshot
            .capability_group_fingerprints
            .windows(2)
            .any(|pair| pair[0].0 >= pair[1].0)
        || group_fingerprints.keys().ne(snapshot
            .capability_group_versions
            .iter()
            .map(|(group_key, _)| group_key))
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "portal policy snapshot capability-group evidence is malformed".to_owned(),
        ));
    }
    let Some((portal, settings)) =
        super::accounts::load_login_portal(connection, &snapshot.portal_id)?
    else {
        return Err(AuthorizationStateError::PortalPolicyChanged);
    };
    if portal.version != snapshot.portal_version
        || super::super::policy::policy_record_fingerprint(&portal)? != snapshot.portal_fingerprint
        || settings.version != snapshot.login_settings_version
        || super::super::policy::policy_record_fingerprint(&settings)?
            != snapshot.login_settings_fingerprint
    {
        return Err(AuthorizationStateError::PortalPolicyChanged);
    }
    let policy = super::policy::load_portal_grant_overrides(
        connection,
        Some(&snapshot.portal_id),
        Some(&snapshot.participant_id),
    )?
    .into_iter()
    .next();
    match (
        policy.as_ref(),
        snapshot.policy_version,
        snapshot.policy_fingerprint.as_ref(),
    ) {
        (None, None, None) => {}
        (Some(policy), Some(version), Some(fingerprint))
            if policy.version == version
                && super::super::policy::policy_record_fingerprint(policy)? == *fingerprint => {}
        _ => return Err(AuthorizationStateError::PortalPolicyChanged),
    }
    for (group_key, version) in &snapshot.capability_group_versions {
        let Some(group) = super::policy::load_capability_group(connection, group_key)? else {
            return Err(AuthorizationStateError::PortalPolicyChanged);
        };
        if group.version != *version
            || super::super::policy::policy_record_fingerprint(&group)?
                != group_fingerprints[group_key]
        {
            return Err(AuthorizationStateError::PortalPolicyChanged);
        }
    }
    Ok(())
}

#[cfg(test)]
mod package_evidence_tests {
    use super::*;
    use crate::platform::auth::evidence::ParticipantRuntimeProjection;
    use crate::platform::auth::{
        GrantRepository, ParticipantBindingState, ProvisionedIdentityKind,
        ProvisionedIdentityRecord, ProvisionedIdentityState,
    };
    use std::collections::BTreeMap;

    fn binding(display_name: &str, resolved_at: i64) -> ParticipantBindingRecord {
        let projection = ParticipantRuntimeProjection {
            participant_id: "example.Service".to_owned(),
            participant_kind: ParticipantKind::Service,
            display_name: display_name.to_owned(),
            implemented_apis: BTreeMap::new(),
            referenced_apis: BTreeMap::new(),
            resources: BTreeMap::new(),
            required_grants: trellis_protocol::GrantSet::new(Vec::new()),
            optional_grant_bundles: BTreeMap::new(),
            required_capabilities: Vec::new(),
            optional_capability_definitions: BTreeMap::new(),
        };
        ParticipantBindingRecord {
            participant_id: projection.participant_id.clone(),
            participant_kind: projection.participant_kind,
            participant_digest: "a".repeat(43),
            needs_digest: trellis_protocol::digest_json(&projection).expect("projection digest"),
            package_digest: "p".repeat(43),
            participant_path: projection.participant_id.clone(),
            projection,
            resolved_at,
            state: ParticipantBindingState::Resolved,
            error: None,
        }
    }

    #[tokio::test]
    async fn fresh_schema_retains_native_revision_history_with_cas() {
        let store = SqliteAuthorizationStore::open_in_memory().expect("store");
        store
            .run(|connection| {
                accept_package_evidence(
                    connection,
                    &"p".repeat(43),
                    "example",
                    "{}",
                    false,
                    None,
                    1,
                )?;
                assert_eq!(
                    install_participant(connection, &binding("v1", 1), Some(0))?,
                    1
                );
                assert_eq!(
                    install_participant(connection, &binding("v2", 2), Some(1))?,
                    2
                );
                assert_eq!(
                    install_participant(connection, &binding("v3", 3), Some(1)),
                    Err(AuthorizationStateError::RevisionConflict {
                        expected: 1,
                        current: 2,
                    })
                );
                assert_eq!(
                    load_installed_participant(connection, "example.Service", Some(1))?
                        .expect("revision one")
                        .1
                        .projection
                        .display_name,
                    "v1"
                );
                assert!(connection
                    .execute(
                        "UPDATE auth_installed_participants SET installed_at = 3
                         WHERE participant_id = 'example.Service' AND revision = 1",
                        [],
                    )
                    .is_err());
                Ok(())
            })
            .await
            .expect("native revision history");
    }

    #[tokio::test]
    async fn tampered_stored_native_projection_is_rejected() {
        let store = SqliteAuthorizationStore::open_in_memory().expect("store");
        let mut binding = binding("valid", 1);
        binding.projection.display_name = "tampered".to_owned();
        assert_eq!(
            store
                .run(move |connection| {
                    accept_package_evidence(
                        connection,
                        &binding.package_digest,
                        "example",
                        "{}",
                        false,
                        None,
                        1,
                    )?;
                    connection
                        .execute(
                            "INSERT INTO auth_installed_participants
                              (participant_id, revision, participant_kind, participant_digest,
                              needs_digest, package_digest, participant_path, projection_json,
                              installed_at)
                             VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
                            params![
                                binding.participant_id,
                                encode_enum(binding.participant_kind)?,
                                binding.participant_digest,
                                binding.needs_digest,
                                binding.package_digest,
                                binding.participant_path,
                                encode_json(&binding.projection)?,
                            ],
                        )
                        .map_err(map_write_error)?;
                    load_installed_participant(connection, "example.Service", None)
                })
                .await,
            Err(AuthorizationStateError::NeedsDigestMismatch)
        );
    }

    #[tokio::test]
    async fn credential_assignment_is_server_owned() {
        let store = SqliteAuthorizationStore::open_in_memory().expect("store");
        store
            .run(|connection| {
                connection.execute_batch(
                    "INSERT INTO auth_principals VALUES ('deployment', 'service', 'active', 1, 1, 1, NULL, NULL);
                     INSERT INTO auth_deployments VALUES ('deployment', 'example.Service', 'service', 'active', NULL);
                     INSERT INTO auth_deployment_profiles VALUES ('deployment', 'service', 'Service', 'example.Service', NULL, 0, NULL, 'active', 1, 1, 1, NULL);
                     INSERT INTO auth_instances VALUES ('instance', 'deployment', 'deployment', 'active', 1, 1, 1);",
                ).map_err(sql_error)?;
                super::super::provisioning::insert_sql_provisioned_identity(
                    connection,
                    &ProvisionedIdentityRecord {
                        identity_key_id: "k".repeat(43),
                        identity_public_key: "p".repeat(43),
                        principal_id: "deployment".to_owned(),
                        deployment_id: "deployment".to_owned(),
                        instance_id: "instance".to_owned(),
                        kind: ProvisionedIdentityKind::Service,
                        state: ProvisionedIdentityState::Active,
                        created_at: 1,
                        revoked_at: None,
                    },
                )?;
                connection
                    .execute(
                        "UPDATE auth_deployment_profiles SET participant_id = 'other.Service'
                         WHERE deployment_id = 'deployment'",
                        [],
                    )
                    .map_err(sql_error)?;
                Ok(())
            })
            .await
            .expect("assignment fixture");
        assert_eq!(
            store
                .get_credential_participant_assignment("k".repeat(43))
                .await
                .expect("assignment lookup"),
            Some("example.Service".to_owned())
        );
    }

    #[tokio::test]
    async fn platform_trust_requires_an_admin_actor() {
        let store = SqliteAuthorizationStore::open_in_memory().expect("store");
        assert_eq!(
            store
                .run(|connection| {
                    accept_package_evidence(
                        connection,
                        &"p".repeat(43),
                        "trellis",
                        "{}",
                        true,
                        None,
                        1,
                    )
                })
                .await,
            Err(AuthorizationStateError::NotAuthorized)
        );
    }

    #[tokio::test]
    async fn installed_namespace_uses_exact_stored_platform_trust() {
        let store = SqliteAuthorizationStore::open_in_memory().expect("store");
        store
            .run(|connection| {
                let mut participant =
                    super::super::super::builtins::auth_runtime_participant_binding(1)?;
                participant.package_digest = "x".repeat(43);
                connection
                    .execute(
                        "INSERT INTO auth_package_evidence
                         (package_digest, evidence_json, platform_trusted, accepted_at, trusted_at, trusted_by)
                         VALUES (?1, '{}', 1, 1, 1, 'admin')",
                        [&participant.package_digest],
                    )
                    .map_err(sql_error)?;
                assert_eq!(install_participant(connection, &participant, Some(0))?, 1);
                Ok(())
            })
            .await
            .expect("trusted Trellis participant installation");
    }

    #[tokio::test]
    async fn platform_trust_rejects_non_trellis_packages() {
        let store = SqliteAuthorizationStore::open_in_memory().expect("store");
        assert!(matches!(
            store
                .run(|connection| {
                    accept_package_evidence(
                        connection,
                        &"p".repeat(43),
                        "example",
                        "{}",
                        true,
                        None,
                        1,
                    )
                })
                .await,
            Err(AuthorizationStateError::InvalidRecord(_))
        ));
    }

    #[tokio::test]
    async fn immutable_digest_rejects_different_evidence_body() {
        let store = SqliteAuthorizationStore::open_in_memory().expect("store");
        store
            .run(|connection| {
                let digest = "x".repeat(43);
                accept_package_evidence(connection, &digest, "example", "{}", false, None, 1)?;
                let error = accept_package_evidence(
                    connection,
                    &digest,
                    "example",
                    r#"{"rootPackage":"different"}"#,
                    false,
                    None,
                    2,
                )
                .expect_err("digest body disagreement");
                assert!(matches!(error, AuthorizationStateError::InvalidRecord(_)));
                Ok(())
            })
            .await
            .expect("evidence check");
    }
}
