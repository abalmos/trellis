use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension};

use super::super::application::repository::{IdempotentOutcome, OutboxRepository};
use super::super::{AuthorizationStateError, IdempotencyResultRecord, PostCommitActionRecord};
use super::common::{
    decode_enum, decode_json, encode_enum, encode_json, from_sql_u32, map_write_error, sql_error,
};
use super::validation::post_commit_action_identity_equal;
use super::SqliteAuthorizationStore;

#[async_trait]
impl OutboxRepository for SqliteAuthorizationStore {
    async fn get_idempotency_result(
        &self,
        purpose: &str,
        signer_id: &str,
        request_id: &str,
    ) -> Result<Option<IdempotencyResultRecord>, AuthorizationStateError> {
        let purpose = purpose.to_owned();
        let signer_id = signer_id.to_owned();
        let request_id = request_id.to_owned();
        self.run_read(move |connection| {
            load_idempotency_result(connection, &purpose, &signer_id, &request_id)
        })
        .await
    }

    async fn record_idempotency_result(
        &self,
        record: IdempotencyResultRecord,
    ) -> Result<IdempotentOutcome<serde_json::Value>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(value) = sqlite_idempotency_replay(&transaction, &record)? {
                return Ok(IdempotentOutcome::Replayed(value));
            }
            let value = record.result.clone();
            insert_sql_idempotency_and_actions(&transaction, &record, &[])?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(value))
        })
        .await
    }

    async fn list_ready_post_commit_actions(
        &self,
        now: i64,
        limit: usize,
    ) -> Result<Vec<PostCommitActionRecord>, AuthorizationStateError> {
        super::super::domain::require_protocol_timestamp("now", now)?;
        let limit = i64::try_from(limit).map_err(|_| {
            AuthorizationStateError::InvalidRecord("limit exceeds SQLite range".to_owned())
        })?;
        self.run_read(move |connection| {
            let mut statement = connection
                .prepare(
                    "SELECT action.action_id, action.kind, action.payload_json, action.created_at,
                            action.attempts, action.next_attempt_at, action.claimed_until,
                            action.last_error, action.predecessor_action_id
                     FROM auth_post_commit_actions AS action
                     WHERE action.next_attempt_at <= ?1
                       AND (action.claimed_until IS NULL OR action.claimed_until <= ?1)
                       AND (action.predecessor_action_id IS NULL OR NOT EXISTS (
                           SELECT 1 FROM auth_post_commit_actions AS predecessor
                           WHERE predecessor.action_id = action.predecessor_action_id
                       ))
                     ORDER BY action.next_attempt_at, action.rowid LIMIT ?2",
                )
                .map_err(sql_error)?;
            let actions = statement
                .query_map(params![now, limit], decode_post_commit_action)
                .map_err(sql_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(sql_error)?;
            Ok(actions)
        })
        .await
    }

    async fn claim_post_commit_action(
        &self,
        action_id: &str,
        now: i64,
        claimed_until: i64,
    ) -> Result<Option<PostCommitActionRecord>, AuthorizationStateError> {
        super::super::domain::require_protocol_timestamp("now", now)?;
        super::super::domain::require_protocol_timestamp("claimedUntil", claimed_until)?;
        if claimed_until <= now {
            return Err(AuthorizationStateError::InvalidRecord(
                "claimedUntil must follow now".to_owned(),
            ));
        }
        let action_id = action_id.to_owned();
        self.run(move |connection| {
            let Some(current) = load_post_commit_action(connection, &action_id)? else {
                return Ok(None);
            };
            if current.claimed_until == Some(claimed_until) {
                return Ok(Some(current));
            }
            if current.next_attempt_at > now
                || current.claimed_until.is_some_and(|until| until > now)
            {
                return Ok(None);
            }
            let attempts = if current.claimed_until.is_some() {
                current.attempts.checked_add(1).ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord("attempts overflow".to_owned())
                })?
            } else {
                current.attempts
            };
            let changed = connection
                .execute(
                    "UPDATE auth_post_commit_actions SET claimed_until = ?1, attempts = ?2
                  WHERE action_id = ?3 AND next_attempt_at <= ?4 AND (claimed_until IS NULL OR claimed_until <= ?4)",
                    params![claimed_until, i64::from(attempts), action_id, now],
                )
                .map_err(map_write_error)?;
            if changed == 0 {
                return Ok(None);
            }
            load_post_commit_action(connection, &action_id)
        })
        .await
    }

    async fn fail_post_commit_action(
        &self,
        action_id: &str,
        expected_claimed_until: i64,
        next_attempt_at: i64,
        error: String,
    ) -> Result<PostCommitActionRecord, AuthorizationStateError> {
        super::super::domain::require_protocol_timestamp(
            "expectedClaimedUntil",
            expected_claimed_until,
        )?;
        super::super::domain::require_protocol_timestamp("nextAttemptAt", next_attempt_at)?;
        super::super::domain::require_nonempty("error", &error)?;
        let action_id = action_id.to_owned();
        self.run(move |connection| {
            let current = load_post_commit_action(connection, &action_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            if current.claimed_until.is_none()
                && current.next_attempt_at == next_attempt_at
                && current.last_error.as_deref() == Some(error.as_str())
            {
                return Ok(current);
            }
            let changed = connection
                .execute(
                    "UPDATE auth_post_commit_actions SET attempts = attempts + 1, next_attempt_at = ?1, claimed_until = NULL, last_error = ?2
                  WHERE action_id = ?3 AND claimed_until = ?4 AND attempts < ?5",
                    params![
                        next_attempt_at,
                        error,
                        action_id,
                        expected_claimed_until,
                        i64::from(u32::MAX)
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            load_post_commit_action(connection, &action_id)?
                .ok_or(AuthorizationStateError::StorageConflict)
        })
        .await
    }

    async fn acknowledge_post_commit_action(
        &self,
        action_id: &str,
        expected_claimed_until: i64,
    ) -> Result<(), AuthorizationStateError> {
        super::super::domain::require_protocol_timestamp(
            "expectedClaimedUntil",
            expected_claimed_until,
        )?;
        let action_id = action_id.to_owned();
        self.run(move |connection| {
            let Some(current) = load_post_commit_action(connection, &action_id)? else {
                return Ok(());
            };
            if current.claimed_until != Some(expected_claimed_until) {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let changed = connection
                .execute(
                    "DELETE FROM auth_post_commit_actions WHERE action_id = ?1 AND claimed_until = ?2",
                    params![action_id, expected_claimed_until],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            Ok(())
        })
        .await
    }
}

pub(in crate::platform::auth) fn sqlite_idempotency_replay(
    connection: &Connection,
    input: &IdempotencyResultRecord,
) -> Result<Option<serde_json::Value>, AuthorizationStateError> {
    if let Some(existing) = load_idempotency_result(
        connection,
        &input.purpose,
        &input.signer_id,
        &input.request_id,
    )? {
        if existing.request_digest != input.request_digest {
            tracing::warn!(
                purpose = %input.purpose,
                signer_id = %input.signer_id,
                request_id = %input.request_id,
                "idempotency request digest conflict"
            );
            return Err(AuthorizationStateError::StorageConflict);
        }
        return Ok(Some(existing.result));
    }
    let scope_exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM auth_idempotency_results WHERE scope_key = ?1)",
            [&input.scope_key],
            |row| row.get::<_, bool>(0),
        )
        .map_err(sql_error)?;
    if scope_exists {
        tracing::warn!(
            purpose = %input.purpose,
            signer_id = %input.signer_id,
            request_id = %input.request_id,
            "idempotency scope conflict"
        );
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(None)
}

pub(in crate::platform::auth) fn insert_sql_idempotency_and_actions(
    connection: &Connection,
    idempotency: &IdempotencyResultRecord,
    actions: &[PostCommitActionRecord],
) -> Result<(), AuthorizationStateError> {
    if sqlite_idempotency_replay(connection, idempotency)?.is_some() {
        return Err(AuthorizationStateError::StorageConflict);
    }
    for (index, action) in actions.iter().enumerate() {
        if actions[..index].iter().any(|existing| {
            existing.action_id == action.action_id
                && !post_commit_action_identity_equal(existing, action)
        }) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if load_post_commit_action(connection, &action.action_id)?
            .is_some_and(|existing| !post_commit_action_identity_equal(&existing, action))
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
    }
    connection
    .execute(
        "INSERT INTO auth_idempotency_results (scope_key, purpose, signer_id, request_id, request_digest, result_json, created_at, expires_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            idempotency.scope_key,
            idempotency.purpose,
            idempotency.signer_id,
            idempotency.request_id,
            idempotency.request_digest,
            encode_json(&idempotency.result)?,
            idempotency.created_at,
            idempotency.expires_at
        ],
    )
    .map_err(map_write_error)?;
    let mut predecessor_action_id: Option<&str> = None;
    for action in actions {
        let action_predecessor_id = action
            .predecessor_action_id
            .as_deref()
            .or(predecessor_action_id);
        if load_post_commit_action(connection, &action.action_id)?.is_none() {
            connection
            .execute(
                "INSERT INTO auth_post_commit_actions (action_id, kind, payload_json, created_at, attempts, next_attempt_at, claimed_until, last_error, predecessor_action_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    action.action_id,
                    encode_enum(action.kind)?,
                    encode_json(&action.payload)?,
                    action.created_at,
                    i64::from(action.attempts),
                    action.next_attempt_at,
                    action.claimed_until,
                    action.last_error,
                    action_predecessor_id,
                ],
            )
            .map_err(map_write_error)?;
        }
        predecessor_action_id = Some(&action.action_id);
    }
    Ok(())
}

pub(in crate::platform::auth) fn load_idempotency_result(
    connection: &Connection,
    purpose: &str,
    signer_id: &str,
    request_id: &str,
) -> Result<Option<IdempotencyResultRecord>, AuthorizationStateError> {
    connection
    .query_row(
        "SELECT scope_key, purpose, signer_id, request_id, request_digest, result_json, created_at, expires_at FROM auth_idempotency_results WHERE purpose = ?1 AND signer_id = ?2 AND request_id = ?3",
        params![purpose, signer_id, request_id],
        |row| {
            Ok(IdempotencyResultRecord {
                scope_key: row.get(0)?,
                purpose: row.get(1)?,
                signer_id: row.get(2)?,
                request_id: row.get(3)?,
                request_digest: row.get(4)?,
                result: decode_json(row.get(5)?)?,
                created_at: row.get(6)?,
                expires_at: row.get(7)?,
            })
        },
    )
    .optional()
    .map_err(sql_error)
}

pub(in crate::platform::auth) fn decode_post_commit_action(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PostCommitActionRecord> {
    Ok(PostCommitActionRecord {
        predecessor_action_id: row.get(8)?,
        action_id: row.get(0)?,
        kind: decode_enum(row.get(1)?)?,
        payload: decode_json(row.get(2)?)?,
        created_at: row.get(3)?,
        attempts: from_sql_u32(row.get(4)?)?,
        next_attempt_at: row.get(5)?,
        claimed_until: row.get(6)?,
        last_error: row.get(7)?,
    })
}

pub(in crate::platform::auth) fn load_post_commit_action(
    connection: &Connection,
    action_id: &str,
) -> Result<Option<PostCommitActionRecord>, AuthorizationStateError> {
    connection
    .query_row(
        "SELECT action_id, kind, payload_json, created_at, attempts, next_attempt_at, claimed_until, last_error, predecessor_action_id FROM auth_post_commit_actions WHERE action_id = ?1",
        [action_id],
        decode_post_commit_action,
    )
    .optional()
    .map_err(sql_error)
}
