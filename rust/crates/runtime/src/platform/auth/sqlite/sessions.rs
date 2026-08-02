use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::super::application::repository::{
    IdempotentOutcome, SessionCreation, SessionRepository, SessionRevocation,
};
use super::super::authority::validate_persisted_session;
use super::super::context::{
    revoke_sql_contexts, AuthorizationContextRevocationReason, AuthorizationContextSelector,
};
use super::super::{AuthorizationStateError, PrincipalKind, SessionRecord, SessionState};
use super::authority::put_sql_desired_authority;
use super::common::{
    decode_enum, encode_enum, from_sql_version, map_write_error, sql_error, to_sql_version,
};
use super::evidence::{
    load_participant_binding, validate_sql_session_runtime_binding_relationships,
};
use super::outbox::{insert_sql_idempotency_and_actions, sqlite_idempotency_replay};
use super::principals::load_principal;
use super::validation::next_version;
use super::SqliteAuthorizationStore;

const SESSION_SELECT: &str = "SELECT
    session_id, principal_id, principal_kind, participant_id, participant_kind,
    participant_artifact_digest, participant_needs_digest, session_public_key,
    session_key_id, inbox_prefix, state, created_at, last_seen_at, expires_at,
    revoked_at, version
    FROM auth_sessions";

#[async_trait]
impl SessionRepository for SqliteAuthorizationStore {
    async fn create_session(
        &self,
        command: SessionCreation,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            insert_sql_session(&transaction, &command.session)?;
            match command.session.principal_kind {
                PrincipalKind::User => {
                    if command.runtime_binding.is_some() {
                        return Err(AuthorizationStateError::InvalidRecord(
                            "user sessions cannot have runtime bindings".to_owned(),
                        ));
                    }
                    if let Some(desired) = command.desired_authority {
                        put_sql_desired_authority(&transaction, desired)?;
                    }
                }
                PrincipalKind::Service | PrincipalKind::Device => {
                    if command.desired_authority.is_some() {
                        return Err(AuthorizationStateError::InvalidRecord(
                            "deployed sessions cannot put user desired authority".to_owned(),
                        ));
                    }
                    let binding = command.runtime_binding.ok_or_else(|| {
                        AuthorizationStateError::InvalidRecord(
                            "deployed sessions require a runtime binding".to_owned(),
                        )
                    })?;
                    if binding.session_id != command.session.session_id {
                        return Err(AuthorizationStateError::InvalidRecord(
                            "runtime binding does not identify the created session".to_owned(),
                        ));
                    }
                    validate_sql_session_runtime_binding_relationships(&transaction, &binding)?;
                    transaction
                        .execute(
                            "INSERT INTO auth_session_runtime_bindings (session_id, deployment_id, instance_id)
                             VALUES (?1, ?2, ?3)",
                            params![binding.session_id, binding.deployment_id, binding.instance_id],
                        )
                        .map_err(map_write_error)?;
                }
            }
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.session))
        })
        .await
    }

    async fn revoke_session(
        &self,
        command: SessionRevocation,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            super::super::domain::require_protocol_timestamp("revokedAt", command.revoked_at)?;
            let current = load_session(&transaction, &command.session_id)?
                .ok_or(AuthorizationStateError::SessionMissing)?;
            if current.version != command.expected_version
                || current.state != SessionState::Active
                || command.revoked_at < current.created_at
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let next = next_version(command.expected_version)?;
            let changed = transaction
                .execute(
                    "UPDATE auth_sessions SET state = 'revoked', revoked_at = ?1, version = ?2
                      WHERE session_id = ?3 AND state = 'active' AND version = ?4",
                    params![
                        command.revoked_at,
                        to_sql_version(next)?,
                        command.session_id,
                        to_sql_version(command.expected_version)?
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let result = load_session(&transaction, &command.session_id)?
                .ok_or(AuthorizationStateError::SessionMissing)?;
            revoke_sql_contexts(
                &transaction,
                &AuthorizationContextSelector::Session(command.session_id.clone()),
                AuthorizationContextRevocationReason::SessionRevoked,
                command.revoked_at.div_euclid(1_000),
            )?;
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(result))
        })
        .await
    }

    async fn get_session(
        &self,
        id: &str,
    ) -> Result<Option<SessionRecord>, AuthorizationStateError> {
        let id = id.to_owned();
        self.run_read(move |connection| load_session(connection, &id))
            .await
    }

    async fn get_session_by_public_key(
        &self,
        public_key: &str,
    ) -> Result<Option<SessionRecord>, AuthorizationStateError> {
        let public_key = public_key.to_owned();
        self.run_read(move |connection| {
            let session_id = connection
                .query_row(
                    "SELECT session_id FROM auth_sessions WHERE session_public_key = ?1",
                    params![public_key],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(sql_error)?;
            session_id
                .map(|session_id| load_session(connection, &session_id))
                .transpose()
                .map(Option::flatten)
        })
        .await
    }

    async fn list_sessions(&self) -> Result<Vec<SessionRecord>, AuthorizationStateError> {
        self.run_read(move |connection| {
            let mut statement = connection
                .prepare("SELECT session_id FROM auth_sessions ORDER BY session_id")
                .map_err(sql_error)?;
            let ids = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(sql_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(sql_error)?;
            ids.into_iter()
                .map(|id| {
                    load_session(connection, &id)?.ok_or(AuthorizationStateError::SessionMissing)
                })
                .collect()
        })
        .await
    }
}

pub(in crate::platform::auth) fn insert_sql_session(
    connection: &Connection,
    session: &SessionRecord,
) -> Result<(), AuthorizationStateError> {
    let principal = load_principal(connection, &session.principal_id)?
        .ok_or(AuthorizationStateError::PrincipalMissing)?;
    if principal.kind != session.principal_kind {
        return Err(AuthorizationStateError::InvalidRecord(
            "session principal kind does not match principal".to_owned(),
        ));
    }
    let participant = load_participant_binding(
        connection,
        &session.participant_id,
        &session.participant_artifact_digest,
    )?
    .ok_or(AuthorizationStateError::ParticipantMissing)?;
    if participant.participant_kind != session.participant_kind {
        return Err(AuthorizationStateError::InvalidRecord(
            "session participant kind does not match participant binding".to_owned(),
        ));
    }
    if participant.needs_digest != session.participant_needs_digest {
        return Err(AuthorizationStateError::NeedsDigestMismatch);
    }
    connection
        .execute(
            "INSERT INTO auth_sessions (
            session_id, principal_id, principal_kind, participant_id,
            participant_kind, participant_artifact_digest,
            participant_needs_digest, session_public_key, session_key_id,
            inbox_prefix, state, created_at, last_seen_at, expires_at,
            revoked_at, version
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                session.session_id,
                session.principal_id,
                encode_enum(session.principal_kind)?,
                session.participant_id,
                encode_enum(session.participant_kind)?,
                session.participant_artifact_digest,
                session.participant_needs_digest,
                session.session_public_key,
                session.session_key_id,
                session.inbox_prefix,
                encode_enum(session.state)?,
                session.created_at,
                session.last_seen_at,
                session.expires_at,
                session.revoked_at,
                to_sql_version(session.version)?
            ],
        )
        .map_err(map_write_error)?;
    Ok(())
}

pub(in crate::platform::auth) fn load_session(
    connection: &Connection,
    id: &str,
) -> Result<Option<SessionRecord>, AuthorizationStateError> {
    connection
        .query_row(
            &format!("{SESSION_SELECT} WHERE session_id = ?1"),
            [id],
            decode_session,
        )
        .optional()
        .map_err(sql_error)
        .and_then(|session| {
            session.map_or(Ok(None), |session| {
                validate_persisted_session(&session)?;
                Ok(Some(session))
            })
        })
}

pub(in crate::platform::auth) fn decode_session(row: &Row<'_>) -> rusqlite::Result<SessionRecord> {
    Ok(SessionRecord {
        session_id: row.get(0)?,
        principal_id: row.get(1)?,
        principal_kind: decode_enum(row.get::<_, String>(2)?)?,
        participant_id: row.get(3)?,
        participant_kind: decode_enum(row.get::<_, String>(4)?)?,
        participant_artifact_digest: row.get(5)?,
        participant_needs_digest: row.get(6)?,
        session_public_key: row.get(7)?,
        session_key_id: row.get(8)?,
        inbox_prefix: row.get(9)?,
        state: decode_enum(row.get::<_, String>(10)?)?,
        created_at: row.get(11)?,
        last_seen_at: row.get(12)?,
        expires_at: row.get(13)?,
        revoked_at: row.get(14)?,
        version: from_sql_version(row.get(15)?)?,
    })
}
