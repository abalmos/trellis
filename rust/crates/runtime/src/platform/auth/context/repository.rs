use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rusqlite::{params, OptionalExtension, Row, ToSql};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest as _, Sha256};
use trellis_protocol::{
    canonicalize_json, parse_authorization_context_v1, AuthorizationAuthorityKindV1,
    AuthorizationPrincipalKindV1,
};

use super::super::{
    companion_repository::IdempotentOutcome,
    repository::{issuance_snapshot_token, IssuanceSnapshotToken},
    sqlite::{
        insert_sql_idempotency_and_actions, sqlite_idempotency_replay, sqlite_issuance_snapshot,
        SqliteAuthorizationStore,
    },
    AuthorityKind, AuthorizationStateError, IdempotencyResultRecord, PostCommitActionKind,
    PostCommitActionRecord, PrincipalKind,
};

const MAXIMUM_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// Durable rollback floor for one installation authorization root.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationTrustStateRecord {
    /// Signed authorization namespace.
    pub authority: String,
    /// Pinned root key ID.
    pub root_key_id: String,
    /// Pinned root object digest.
    pub root_digest: String,
    /// Highest accepted issuer-manifest generation.
    pub manifest_generation: u64,
    /// Digest accepted at `manifest_generation`.
    pub manifest_digest: String,
    /// Current online issuer key ID.
    pub active_issuer_key_id: String,
    /// Last accepted update in Unix milliseconds.
    pub updated_at: i64,
    /// Positive optimistic version.
    pub version: u64,
}

/// Durable authorization-context lifecycle state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationContextState {
    /// Signed context is within its durable lease and has not been revoked.
    Active,
    /// Context was invalidated before its signed expiry.
    Revoked,
    /// Signed expiry elapsed without a semantic revocation.
    Expired,
}

/// Stable safe reason published for an authorization-context revocation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationContextRevocationReason {
    SessionRevoked,
    SessionExpired,
    SessionRebound,
    CredentialChanged,
    PrincipalChanged,
    PrincipalInactive,
    AuthorityChanged,
    AuthorityRevoked,
    MaterializationChanged,
    MaterializationUnavailable,
    DeploymentInactive,
    DeploymentChanged,
    InstanceInactive,
    InstanceChanged,
    DeviceInactive,
    DeviceChanged,
    DelegationChanged,
    ParticipantChanged,
    IssuerRevoked,
    ContextReplaced,
    AdministrativeRevoke,
}

/// Durable signed authorization context and its publication lifecycle.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationContextRecord {
    pub context_id: String,
    pub context_digest: String,
    pub session_id: String,
    pub principal_id: String,
    pub principal_kind: PrincipalKind,
    pub participant_id: String,
    pub participant_artifact_digest: String,
    pub participant_needs_digest: String,
    pub authority_kind: AuthorityKind,
    pub authority_id: String,
    pub authority_version: u64,
    pub materialization_version: u64,
    pub deployment_id: Option<String>,
    pub instance_id: Option<String>,
    pub issuer_key_id: String,
    pub signed_context_json: String,
    pub context_token: String,
    pub issuance_snapshot_token: String,
    pub trust_generation: u64,
    pub issued_at: i64,
    pub not_before: i64,
    pub expires_at: i64,
    pub refresh_at: i64,
    pub state: AuthorizationContextState,
    pub published_at: Option<i64>,
    pub revoked_at: Option<i64>,
    pub revocation_reason: Option<AuthorizationContextRevocationReason>,
    pub version: u64,
}

/// Aggregate optimistic context-issuance commit.
#[derive(Clone, Debug)]
pub struct AuthorizationContextCommit {
    pub expected_snapshot_token: IssuanceSnapshotToken,
    pub context: AuthorizationContextRecord,
    pub idempotency: IdempotencyResultRecord,
    pub now: i64,
    pub minimum_remaining_seconds: i64,
}

/// Exact durable selector for context invalidation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthorizationContextSelector {
    Context(String),
    Session(String),
    Principal(String),
    Authority(AuthorityKind, String),
    Deployment(String),
    Instance(String),
    Issuer(String),
}

/// Durable trust-floor and authorization-context repository boundary.
#[async_trait]
pub trait AuthorizationContextRepository: Send + Sync {
    async fn get_trust_state(
        &self,
    ) -> Result<Option<AuthorizationTrustStateRecord>, AuthorizationStateError>;

    async fn accept_trust_state(
        &self,
        state: AuthorizationTrustStateRecord,
        removed_issuer_key_ids: Vec<String>,
        revoked_at: i64,
    ) -> Result<AuthorizationTrustStateRecord, AuthorizationStateError>;

    async fn get_context_by_id(
        &self,
        context_id: &str,
    ) -> Result<Option<AuthorizationContextRecord>, AuthorizationStateError>;

    async fn get_context_by_digest(
        &self,
        context_digest: &str,
    ) -> Result<Option<AuthorizationContextRecord>, AuthorizationStateError>;

    #[cfg(test)]
    async fn list_active_contexts_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<AuthorizationContextRecord>, AuthorizationStateError>;

    async fn list_unpublished_contexts(
        &self,
        limit: usize,
    ) -> Result<Vec<AuthorizationContextRecord>, AuthorizationStateError>;

    async fn list_active_issuer_key_ids(&self) -> Result<Vec<String>, AuthorizationStateError>;

    async fn list_revoked_contexts(
        &self,
        after_context_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<AuthorizationContextRecord>, AuthorizationStateError>;

    async fn commit_context(
        &self,
        commit: AuthorizationContextCommit,
    ) -> Result<IdempotentOutcome<AuthorizationContextRecord>, AuthorizationStateError>;

    async fn mark_context_published(
        &self,
        context_id: &str,
        expected_version: u64,
        published_at: i64,
    ) -> Result<AuthorizationContextRecord, AuthorizationStateError>;

    async fn expire_contexts(
        &self,
        now: i64,
    ) -> Result<Vec<AuthorizationContextRecord>, AuthorizationStateError>;

    async fn delete_terminal_contexts(
        &self,
        before: i64,
        limit: usize,
    ) -> Result<usize, AuthorizationStateError>;
}

#[async_trait]
impl AuthorizationContextRepository for SqliteAuthorizationStore {
    async fn get_trust_state(
        &self,
    ) -> Result<Option<AuthorizationTrustStateRecord>, AuthorizationStateError> {
        self.run(|connection| load_sql_trust_state(connection))
            .await
    }

    async fn accept_trust_state(
        &self,
        state: AuthorizationTrustStateRecord,
        removed_issuer_key_ids: Vec<String>,
        revoked_at: i64,
    ) -> Result<AuthorizationTrustStateRecord, AuthorizationStateError> {
        validate_trust_state(&state)?;
        valid_timestamp("revokedAt", revoked_at)?;
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            let accepted = accept_trust_state(load_sql_trust_state(&transaction)?.as_ref(), state)?;
            transaction
                .execute(
                    "INSERT INTO auth_authorization_trust_state (
                        singleton_id, authority, root_key_id, root_digest, manifest_generation,
                        manifest_digest, active_issuer_key_id, updated_at, version
                     ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT(singleton_id) DO UPDATE SET
                        authority = excluded.authority,
                        root_key_id = excluded.root_key_id,
                        root_digest = excluded.root_digest,
                        manifest_generation = excluded.manifest_generation,
                        manifest_digest = excluded.manifest_digest,
                        active_issuer_key_id = excluded.active_issuer_key_id,
                        updated_at = excluded.updated_at,
                        version = excluded.version",
                    params![
                        accepted.authority,
                        accepted.root_key_id,
                        accepted.root_digest,
                        sql_version(accepted.manifest_generation)?,
                        accepted.manifest_digest,
                        accepted.active_issuer_key_id,
                        accepted.updated_at,
                        sql_version(accepted.version)?,
                    ],
                )
                .map_err(write_error)?;
            for issuer_key_id in removed_issuer_key_ids {
                revoke_sql_contexts(
                    &transaction,
                    &AuthorizationContextSelector::Issuer(issuer_key_id),
                    AuthorizationContextRevocationReason::IssuerRevoked,
                    revoked_at,
                )?;
            }
            transaction.commit().map_err(sql_error)?;
            Ok(accepted)
        })
        .await
    }

    async fn get_context_by_id(
        &self,
        context_id: &str,
    ) -> Result<Option<AuthorizationContextRecord>, AuthorizationStateError> {
        let id = context_id.to_owned();
        self.run(move |connection| load_sql_context_by_id(connection, &id))
            .await
    }

    async fn get_context_by_digest(
        &self,
        context_digest: &str,
    ) -> Result<Option<AuthorizationContextRecord>, AuthorizationStateError> {
        let digest = context_digest.to_owned();
        self.run(move |connection| {
            connection
                .query_row(
                    &format!("{} WHERE context_digest = ?1", CONTEXT_SELECT),
                    [&digest],
                    decode_sql_context,
                )
                .optional()
                .map_err(sql_error)
        })
        .await
    }

    #[cfg(test)]
    async fn list_active_contexts_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<AuthorizationContextRecord>, AuthorizationStateError> {
        let id = session_id.to_owned();
        self.run(move |connection| {
            query_sql_contexts(
                connection,
                "session_id = ?1 AND state = 'active' ORDER BY expires_at, context_id",
                &[&id],
            )
        })
        .await
    }

    async fn list_unpublished_contexts(
        &self,
        limit: usize,
    ) -> Result<Vec<AuthorizationContextRecord>, AuthorizationStateError> {
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        self.run(move |connection| {
            query_sql_contexts(
                connection,
                "state = 'active' AND published_at IS NULL ORDER BY issued_at, context_id LIMIT ?1",
                &[&limit],
            )
        })
        .await
    }

    async fn list_active_issuer_key_ids(&self) -> Result<Vec<String>, AuthorizationStateError> {
        self.run(|connection| {
            let mut statement = connection
                .prepare(
                    "SELECT DISTINCT issuer_key_id FROM auth_authorization_contexts
                     WHERE state = 'active' ORDER BY issuer_key_id",
                )
                .map_err(sql_error)?;
            let values = statement
                .query_map([], |row| row.get(0))
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            Ok(values)
        })
        .await
    }

    async fn list_revoked_contexts(
        &self,
        after_context_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<AuthorizationContextRecord>, AuthorizationStateError> {
        let after_context_id = after_context_id.map(str::to_owned);
        self.run(move |connection| {
            let limit = i64::try_from(limit).unwrap_or(i64::MAX);
            match after_context_id.as_deref() {
                Some(after) => query_sql_contexts(
                    connection,
                    "state = 'revoked' AND context_id > ?1 ORDER BY context_id LIMIT ?2",
                    &[&after, &limit],
                ),
                None => query_sql_contexts(
                    connection,
                    "state = 'revoked' ORDER BY context_id LIMIT ?1",
                    &[&limit],
                ),
            }
        })
        .await
    }

    async fn commit_context(
        &self,
        commit: AuthorizationContextCommit,
    ) -> Result<IdempotentOutcome<AuthorizationContextRecord>, AuthorizationStateError> {
        validate_context_record(&commit.context)?;
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(replay) = sqlite_idempotency_replay(&transaction, &commit.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(replay));
            }
            let snapshot = sqlite_issuance_snapshot(&transaction, &commit.context.session_id)?;
            if issuance_snapshot_token(&snapshot)? != commit.expected_snapshot_token {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let active = query_sql_contexts(
                &transaction,
                "session_id = ?1 AND state = 'active' ORDER BY issued_at DESC, context_id DESC",
                &[&commit.context.session_id],
            )?;
            for displaced in active.iter().skip(2) {
                revoke_sql_contexts(
                    &transaction,
                    &AuthorizationContextSelector::Context(displaced.context_id.clone()),
                    AuthorizationContextRevocationReason::ContextReplaced,
                    commit.now,
                )?;
            }
            if let Some(existing) = active.iter().take(2).find(|context| {
                context.issuance_snapshot_token == commit.context.issuance_snapshot_token
                    && context.issuer_key_id == commit.context.issuer_key_id
                    && context.trust_generation == commit.context.trust_generation
                    && context.refresh_at > commit.now
                    && context.expires_at - commit.now >= commit.minimum_remaining_seconds
            }) {
                let mut idempotency = commit.idempotency;
                idempotency.result = json!({ "contextId": existing.context_id });
                insert_sql_idempotency_and_actions(&transaction, &idempotency, &[])?;
                let existing = existing.clone();
                transaction.commit().map_err(sql_error)?;
                return Ok(IdempotentOutcome::Applied(existing));
            }
            for displaced in active.iter().skip(1) {
                revoke_sql_contexts(
                    &transaction,
                    &AuthorizationContextSelector::Context(displaced.context_id.clone()),
                    AuthorizationContextRevocationReason::ContextReplaced,
                    commit.now,
                )?;
            }
            let action =
                context_action(&commit.context, PostCommitActionKind::ContextPublish, None)?;
            insert_sql_context(&transaction, &commit.context)?;
            let mut idempotency = commit.idempotency;
            idempotency.result = json!({ "contextId": commit.context.context_id });
            insert_sql_idempotency_and_actions(&transaction, &idempotency, &[action])?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(commit.context))
        })
        .await
    }

    async fn mark_context_published(
        &self,
        context_id: &str,
        expected_version: u64,
        published_at: i64,
    ) -> Result<AuthorizationContextRecord, AuthorizationStateError> {
        valid_timestamp("publishedAt", published_at)?;
        let id = context_id.to_owned();
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            let current = load_sql_context_by_id(&transaction, &id)?.ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("context is missing".to_owned())
            })?;
            if current.version != expected_version || published_at > current.expires_at {
                return Err(AuthorizationStateError::StorageConflict);
            }
            if let Some(existing) = current.published_at {
                return if existing == published_at {
                    Ok(current)
                } else {
                    Err(AuthorizationStateError::StorageConflict)
                };
            }
            let version = next_version(current.version)?;
            transaction
                .execute(
                    "UPDATE auth_authorization_contexts
                     SET published_at = ?1, version = ?2
                     WHERE context_id = ?3 AND version = ?4 AND published_at IS NULL",
                    params![
                        published_at,
                        sql_version(version)?,
                        id,
                        sql_version(expected_version)?,
                    ],
                )
                .map_err(write_error)
                .and_then(|changed| {
                    if changed == 1 {
                        Ok(())
                    } else {
                        Err(AuthorizationStateError::StorageConflict)
                    }
                })?;
            let updated = load_sql_context_by_id(&transaction, &id)?.ok_or_else(|| {
                AuthorizationStateError::Storage("published context disappeared".to_owned())
            })?;
            transaction.commit().map_err(sql_error)?;
            Ok(updated)
        })
        .await
    }

    async fn expire_contexts(
        &self,
        now: i64,
    ) -> Result<Vec<AuthorizationContextRecord>, AuthorizationStateError> {
        valid_timestamp("now", now)?;
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            let contexts = query_sql_contexts(
                &transaction,
                "state = 'active' AND expires_at <= ?1 ORDER BY expires_at, context_id",
                &[&now],
            )?;
            for context in &contexts {
                transaction
                    .execute(
                        "UPDATE auth_authorization_contexts SET state = 'expired', version = ?1
                         WHERE context_id = ?2 AND state = 'active' AND version = ?3",
                        params![
                            sql_version(next_version(context.version)?)?,
                            context.context_id,
                            sql_version(context.version)?,
                        ],
                    )
                    .map_err(write_error)?;
            }
            let expired = contexts
                .into_iter()
                .map(|mut context| {
                    context.state = AuthorizationContextState::Expired;
                    context.version = next_version(context.version)?;
                    Ok(context)
                })
                .collect::<Result<Vec<_>, AuthorizationStateError>>()?;
            transaction.commit().map_err(sql_error)?;
            Ok(expired)
        })
        .await
    }

    async fn delete_terminal_contexts(
        &self,
        before: i64,
        limit: usize,
    ) -> Result<usize, AuthorizationStateError> {
        valid_timestamp("before", before)?;
        let limit = i64::try_from(limit).map_err(|_| {
            AuthorizationStateError::InvalidRecord("context cleanup limit is too large".to_owned())
        })?;
        self.run(move |connection| {
            let before_millis = before.checked_mul(1_000).ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(
                    "context cleanup timestamp overflow".to_owned(),
                )
            })?;
            connection
                .execute(
                    "DELETE FROM auth_idempotency_results
                     WHERE purpose = 'authorizationContextIssue' AND expires_at <= ?1",
                    [before_millis],
                )
                .map_err(write_error)?;
            connection
                .execute(
                    "DELETE FROM auth_authorization_contexts
                     WHERE context_id IN (
                       SELECT context_id FROM auth_authorization_contexts AS context
                       WHERE state IN ('expired', 'revoked')
                          AND expires_at <= ?1
                         AND NOT EXISTS (
                           SELECT 1 FROM auth_post_commit_actions AS action
                           WHERE json_extract(action.payload_json, '$.contextDigest') = context.context_digest
                         )
                        ORDER BY expires_at, context_id
                       LIMIT ?2
                     )",
                    params![before, limit],
                )
                .map_err(write_error)
        })
        .await
    }
}

const CONTEXT_SELECT: &str = "SELECT
    context_id, context_digest, session_id, principal_id, principal_kind,
    participant_id, participant_artifact_digest, participant_needs_digest,
    authority_kind, authority_id, authority_version, materialization_version,
    deployment_id, instance_id, issuer_key_id, signed_context_json, context_token,
    issuance_snapshot_token, trust_generation, issued_at, not_before, expires_at,
    refresh_at, state, published_at, revoked_at, revocation_reason, version
    FROM auth_authorization_contexts";

fn load_sql_trust_state(
    connection: &rusqlite::Connection,
) -> Result<Option<AuthorizationTrustStateRecord>, AuthorizationStateError> {
    let table_exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'auth_authorization_trust_state')",
            [],
            |row| row.get::<_, bool>(0),
        )
        .map_err(sql_error)?;
    if !table_exists {
        return Ok(None);
    }
    connection
        .query_row(
            "SELECT authority, root_key_id, root_digest, manifest_generation, manifest_digest,
                    active_issuer_key_id, updated_at, version
             FROM auth_authorization_trust_state WHERE singleton_id = 1",
            [],
            |row| {
                Ok(AuthorizationTrustStateRecord {
                    authority: row.get(0)?,
                    root_key_id: row.get(1)?,
                    root_digest: row.get(2)?,
                    manifest_generation: row.get::<_, u64>(3)?,
                    manifest_digest: row.get(4)?,
                    active_issuer_key_id: row.get(5)?,
                    updated_at: row.get(6)?,
                    version: row.get::<_, u64>(7)?,
                })
            },
        )
        .optional()
        .map_err(sql_error)
}

fn insert_sql_context(
    connection: &rusqlite::Connection,
    context: &AuthorizationContextRecord,
) -> Result<(), AuthorizationStateError> {
    connection
        .execute(
            "INSERT INTO auth_authorization_contexts (
                context_id, context_digest, session_id, principal_id, principal_kind,
                participant_id, participant_artifact_digest, participant_needs_digest,
                authority_kind, authority_id, authority_version, materialization_version,
                deployment_id, instance_id, issuer_key_id, signed_context_json, context_token,
                issuance_snapshot_token, trust_generation, issued_at, not_before, expires_at,
                refresh_at, state, published_at, revoked_at, revocation_reason, version
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28
             )",
            params![
                context.context_id,
                context.context_digest,
                context.session_id,
                context.principal_id,
                enum_string(context.principal_kind)?,
                context.participant_id,
                context.participant_artifact_digest,
                context.participant_needs_digest,
                enum_string(context.authority_kind)?,
                context.authority_id,
                sql_version(context.authority_version)?,
                sql_version(context.materialization_version)?,
                context.deployment_id,
                context.instance_id,
                context.issuer_key_id,
                context.signed_context_json,
                context.context_token,
                context.issuance_snapshot_token,
                sql_version(context.trust_generation)?,
                context.issued_at,
                context.not_before,
                context.expires_at,
                context.refresh_at,
                enum_string(context.state)?,
                context.published_at,
                context.revoked_at,
                context.revocation_reason.map(enum_string).transpose()?,
                sql_version(context.version)?,
            ],
        )
        .map_err(write_error)?;
    Ok(())
}

fn load_sql_context_by_id(
    connection: &rusqlite::Connection,
    context_id: &str,
) -> Result<Option<AuthorizationContextRecord>, AuthorizationStateError> {
    connection
        .query_row(
            &format!("{} WHERE context_id = ?1", CONTEXT_SELECT),
            [context_id],
            decode_sql_context,
        )
        .optional()
        .map_err(sql_error)
}

fn query_sql_contexts(
    connection: &rusqlite::Connection,
    predicate: &str,
    parameters: &[&dyn ToSql],
) -> Result<Vec<AuthorizationContextRecord>, AuthorizationStateError> {
    let mut statement = connection
        .prepare(&format!("{} WHERE {}", CONTEXT_SELECT, predicate))
        .map_err(sql_error)?;
    let contexts = statement
        .query_map(parameters, decode_sql_context)
        .map_err(sql_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_error)?;
    Ok(contexts)
}

fn decode_sql_context(row: &Row<'_>) -> rusqlite::Result<AuthorizationContextRecord> {
    Ok(AuthorizationContextRecord {
        context_id: row.get(0)?,
        context_digest: row.get(1)?,
        session_id: row.get(2)?,
        principal_id: row.get(3)?,
        principal_kind: parse_enum(row.get::<_, String>(4)?)?,
        participant_id: row.get(5)?,
        participant_artifact_digest: row.get(6)?,
        participant_needs_digest: row.get(7)?,
        authority_kind: parse_enum(row.get::<_, String>(8)?)?,
        authority_id: row.get(9)?,
        authority_version: row.get(10)?,
        materialization_version: row.get(11)?,
        deployment_id: row.get(12)?,
        instance_id: row.get(13)?,
        issuer_key_id: row.get(14)?,
        signed_context_json: row.get(15)?,
        context_token: row.get(16)?,
        issuance_snapshot_token: row.get(17)?,
        trust_generation: row.get(18)?,
        issued_at: row.get(19)?,
        not_before: row.get(20)?,
        expires_at: row.get(21)?,
        refresh_at: row.get(22)?,
        state: parse_enum(row.get::<_, String>(23)?)?,
        published_at: row.get(24)?,
        revoked_at: row.get(25)?,
        revocation_reason: row
            .get::<_, Option<String>>(26)?
            .map(parse_enum)
            .transpose()?,
        version: row.get(27)?,
    })
}

pub(crate) fn revoke_sql_contexts(
    connection: &rusqlite::Connection,
    selector: &AuthorizationContextSelector,
    reason: AuthorizationContextRevocationReason,
    revoked_at: i64,
) -> Result<Vec<AuthorizationContextRecord>, AuthorizationStateError> {
    valid_timestamp("revokedAt", revoked_at)?;
    let contexts = match selector {
        AuthorizationContextSelector::Context(id) => {
            query_sql_contexts(connection, "state = 'active' AND context_id = ?1", &[id])?
        }
        AuthorizationContextSelector::Session(id) => query_sql_contexts(
            connection,
            "state = 'active' AND session_id = ?1 ORDER BY context_id",
            &[id],
        )?,
        AuthorizationContextSelector::Principal(id) => query_sql_contexts(
            connection,
            "state = 'active' AND principal_id = ?1 ORDER BY context_id",
            &[id],
        )?,
        AuthorizationContextSelector::Authority(kind, id) => {
            let kind = enum_string(*kind)?;
            query_sql_contexts(
                connection,
                "state = 'active' AND authority_kind = ?1 AND authority_id = ?2 ORDER BY context_id",
                &[&kind, id],
            )?
        }
        AuthorizationContextSelector::Deployment(id) => query_sql_contexts(
            connection,
            "state = 'active' AND deployment_id = ?1 ORDER BY context_id",
            &[id],
        )?,
        AuthorizationContextSelector::Instance(id) => query_sql_contexts(
            connection,
            "state = 'active' AND instance_id = ?1 ORDER BY context_id",
            &[id],
        )?,
        AuthorizationContextSelector::Issuer(id) => query_sql_contexts(
            connection,
            "state = 'active' AND issuer_key_id = ?1 ORDER BY context_id",
            &[id],
        )?,
    };
    let mut revoked = Vec::with_capacity(contexts.len());
    for mut context in contexts {
        context.state = AuthorizationContextState::Revoked;
        context.revoked_at = Some(revoked_at);
        context.revocation_reason = Some(reason);
        context.version = next_version(context.version)?;
        connection
            .execute(
                "UPDATE auth_authorization_contexts
                 SET state = 'revoked', revoked_at = ?1, revocation_reason = ?2, version = ?3
                 WHERE context_id = ?4 AND state = 'active' AND version = ?5",
                params![
                    revoked_at,
                    enum_string(reason)?,
                    sql_version(context.version)?,
                    context.context_id,
                    sql_version(context.version - 1)?,
                ],
            )
            .map_err(write_error)
            .and_then(|changed| {
                if changed == 1 {
                    Ok(())
                } else {
                    Err(AuthorizationStateError::StorageConflict)
                }
            })?;
        insert_context_action(
            connection,
            &context_action(&context, PostCommitActionKind::ContextRevoke, Some(reason))?,
        )?;
        revoked.push(context);
    }
    Ok(revoked)
}

fn insert_context_action(
    connection: &rusqlite::Connection,
    action: &PostCommitActionRecord,
) -> Result<(), AuthorizationStateError> {
    let payload = canonicalize_json(&action.payload)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
    if let Some((kind, existing_payload)) = connection
        .query_row(
            "SELECT kind, payload_json FROM auth_post_commit_actions WHERE action_id = ?1",
            [&action.action_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(sql_error)?
    {
        return if kind == enum_string(action.kind)? && existing_payload == payload {
            Ok(())
        } else {
            Err(AuthorizationStateError::StorageConflict)
        };
    }
    connection
        .execute(
            "INSERT INTO auth_post_commit_actions (
                action_id, kind, payload_json, created_at, attempts, next_attempt_at,
                claimed_until, last_error
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                action.action_id,
                enum_string(action.kind)?,
                payload,
                action.created_at,
                i64::from(action.attempts),
                action.next_attempt_at,
                action.claimed_until,
                action.last_error,
            ],
        )
        .map_err(write_error)?;
    Ok(())
}

fn enum_string<T: Serialize>(value: T) -> Result<String, AuthorizationStateError> {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or_else(|| AuthorizationStateError::Storage("cannot encode context enum".to_owned()))
}

fn parse_enum<T: for<'de> Deserialize<'de>>(value: String) -> rusqlite::Result<T> {
    serde_json::from_value(Value::String(value)).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
    })
}

fn sql_version(value: u64) -> Result<i64, AuthorizationStateError> {
    i64::try_from(value).map_err(|_| {
        AuthorizationStateError::InvalidRecord("version exceeds SQLite integer range".to_owned())
    })
}

fn sql_error(error: rusqlite::Error) -> AuthorizationStateError {
    AuthorizationStateError::Storage(error.to_string())
}

fn write_error(error: rusqlite::Error) -> AuthorizationStateError {
    match error {
        rusqlite::Error::SqliteFailure(code, _)
            if code.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AuthorizationStateError::StorageConflict
        }
        error => sql_error(error),
    }
}

fn validate_trust_state(
    state: &AuthorizationTrustStateRecord,
) -> Result<(), AuthorizationStateError> {
    nonempty("authority", &state.authority)?;
    digest("rootKeyId", &state.root_key_id)?;
    digest("rootDigest", &state.root_digest)?;
    digest("manifestDigest", &state.manifest_digest)?;
    digest("activeIssuerKeyId", &state.active_issuer_key_id)?;
    positive("manifestGeneration", state.manifest_generation)?;
    positive("version", state.version)?;
    valid_timestamp("updatedAt", state.updated_at)
}

fn accept_trust_state(
    current: Option<&AuthorizationTrustStateRecord>,
    next: AuthorizationTrustStateRecord,
) -> Result<AuthorizationTrustStateRecord, AuthorizationStateError> {
    let Some(current) = current else {
        if next.version != 1 {
            return Err(AuthorizationStateError::StorageConflict);
        }
        return Ok(next);
    };
    if current.authority != next.authority
        || current.root_key_id != next.root_key_id
        || current.root_digest != next.root_digest
        || next.manifest_generation < current.manifest_generation
        || (next.manifest_generation == current.manifest_generation
            && next.manifest_digest != current.manifest_digest)
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    if next.manifest_generation == current.manifest_generation {
        return Ok(current.clone());
    }
    if next.version != next_version(current.version)? {
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(next)
}

fn validate_context_record(
    record: &AuthorizationContextRecord,
) -> Result<(), AuthorizationStateError> {
    nonempty("contextId", &record.context_id)?;
    digest("contextDigest", &record.context_digest)?;
    nonempty("sessionId", &record.session_id)?;
    nonempty("principalId", &record.principal_id)?;
    nonempty("participantId", &record.participant_id)?;
    digest(
        "participantArtifactDigest",
        &record.participant_artifact_digest,
    )?;
    digest("participantNeedsDigest", &record.participant_needs_digest)?;
    nonempty("authorityId", &record.authority_id)?;
    positive("authorityVersion", record.authority_version)?;
    positive("materializationVersion", record.materialization_version)?;
    digest("issuerKeyId", &record.issuer_key_id)?;
    digest("issuanceSnapshotToken", &record.issuance_snapshot_token)?;
    positive("trustGeneration", record.trust_generation)?;
    positive("version", record.version)?;
    for (name, value) in [
        ("issuedAt", record.issued_at),
        ("notBefore", record.not_before),
        ("expiresAt", record.expires_at),
        ("refreshAt", record.refresh_at),
    ] {
        valid_timestamp(name, value)?;
    }
    if record.not_before > record.issued_at
        || record.expires_at <= record.not_before
        || !(record.issued_at..record.expires_at).contains(&record.refresh_at)
        || record.deployment_id.is_none() != record.instance_id.is_none()
        || (record.state == AuthorizationContextState::Revoked)
            != (record.revoked_at.is_some() && record.revocation_reason.is_some())
        || (record.state != AuthorizationContextState::Revoked
            && (record.revoked_at.is_some() || record.revocation_reason.is_some()))
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "authorization context lifecycle is inconsistent".to_owned(),
        ));
    }
    let bytes = URL_SAFE_NO_PAD.decode(&record.context_token).map_err(|_| {
        AuthorizationStateError::InvalidRecord("context token is not base64url".to_owned())
    })?;
    if URL_SAFE_NO_PAD.encode(&bytes) != record.context_token
        || bytes.as_slice() != record.signed_context_json.as_bytes()
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "context token does not encode signed context JSON".to_owned(),
        ));
    }
    let value: Value = serde_json::from_str(&record.signed_context_json).map_err(|error| {
        AuthorizationStateError::InvalidRecord(format!("invalid signed context: {error}"))
    })?;
    let signed = parse_authorization_context_v1(&value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let canonical = canonicalize_json(&value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    if canonical != record.signed_context_json
        || signed
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
            != record.context_digest
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "signed context canonical digest does not match".to_owned(),
        ));
    }
    let context = signed.unsigned;
    if context.context_id != record.context_id
        || context.session_id != record.session_id
        || context.principal.id != record.principal_id
        || principal_kind(context.principal.kind) != record.principal_kind
        || context.participant.id != record.participant_id
        || context.participant.artifact_digest != record.participant_artifact_digest
        || context.participant.needs_digest != record.participant_needs_digest
        || authority_kind(context.authority_ref.kind) != record.authority_kind
        || context.authority_ref.id != record.authority_id
        || context.authority_ref.version != record.authority_version
        || context.deployment_id != record.deployment_id
        || context.instance_id != record.instance_id
        || context.issuer_key_id != record.issuer_key_id
        || context.issued_at != record.issued_at
        || context.not_before != record.not_before
        || context.expires_at != record.expires_at
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "context record metadata does not match signed context".to_owned(),
        ));
    }
    Ok(())
}

fn context_action(
    context: &AuthorizationContextRecord,
    kind: PostCommitActionKind,
    reason: Option<AuthorizationContextRevocationReason>,
) -> Result<PostCommitActionRecord, AuthorizationStateError> {
    let payload = match kind {
        PostCommitActionKind::ContextPublish => json!({
            "format": "trellis.authorization-context-publish-action.v1",
            "contextId": context.context_id,
            "contextDigest": context.context_digest,
        }),
        PostCommitActionKind::ContextRevoke => json!({
            "format": "trellis.authorization-context-revoke-action.v1",
            "contextId": context.context_id,
            "contextDigest": context.context_digest,
            "reason": reason,
            "version": context.version,
        }),
        PostCommitActionKind::Event | PostCommitActionKind::Kick => unreachable!(),
    };
    let canonical = canonicalize_json(&payload)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
    let action_at = context
        .revoked_at
        .unwrap_or(context.issued_at)
        .checked_mul(1_000)
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("context time overflow".to_owned())
        })?;
    Ok(PostCommitActionRecord {
        action_id: URL_SAFE_NO_PAD.encode(Sha256::digest(canonical.as_bytes())),
        kind,
        payload,
        created_at: action_at,
        attempts: 0,
        next_attempt_at: action_at,
        claimed_until: None,
        last_error: None,
    })
}

fn principal_kind(kind: AuthorizationPrincipalKindV1) -> PrincipalKind {
    match kind {
        AuthorizationPrincipalKindV1::User => PrincipalKind::User,
        AuthorizationPrincipalKindV1::Service => PrincipalKind::Service,
        AuthorizationPrincipalKindV1::Device => PrincipalKind::Device,
    }
}

fn authority_kind(kind: AuthorizationAuthorityKindV1) -> AuthorityKind {
    match kind {
        AuthorizationAuthorityKindV1::Identity => AuthorityKind::Identity,
        AuthorizationAuthorityKindV1::Deployment => AuthorityKind::Deployment,
    }
}

fn nonempty(name: &str, value: &str) -> Result<(), AuthorizationStateError> {
    if value.is_empty() || value.trim() != value {
        Err(AuthorizationStateError::InvalidRecord(format!(
            "{name} must be nonempty protocol-safe text"
        )))
    } else {
        Ok(())
    }
}

fn digest(name: &str, value: &str) -> Result<(), AuthorizationStateError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| AuthorizationStateError::InvalidRecord(format!("{name} must be a digest")))?;
    if bytes.len() == 32 && URL_SAFE_NO_PAD.encode(bytes) == value {
        Ok(())
    } else {
        Err(AuthorizationStateError::InvalidRecord(format!(
            "{name} must canonically encode 32 bytes"
        )))
    }
}

fn positive(name: &str, value: u64) -> Result<(), AuthorizationStateError> {
    if (1..=MAXIMUM_SAFE_INTEGER).contains(&value) {
        Ok(())
    } else {
        Err(AuthorizationStateError::InvalidRecord(format!(
            "{name} must be a positive safe integer"
        )))
    }
}

fn valid_timestamp(name: &str, value: i64) -> Result<(), AuthorizationStateError> {
    if (0..=MAXIMUM_SAFE_INTEGER as i64).contains(&value) {
        Ok(())
    } else {
        Err(AuthorizationStateError::InvalidRecord(format!(
            "{name} must be a nonnegative safe integer"
        )))
    }
}

fn next_version(version: u64) -> Result<u64, AuthorizationStateError> {
    version
        .checked_add(1)
        .filter(|version| *version <= MAXIMUM_SAFE_INTEGER)
        .ok_or_else(|| AuthorizationStateError::InvalidRecord("version overflow".to_owned()))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        sync::Arc,
    };

    use ed25519_dalek::SigningKey;
    use trellis_protocol::{
        encode_authorization_context_token_v1, parse_authorization_context_v1,
        parse_participant_v1, resolve_participant_v1, sign_authorization_context_v1,
        AuthorizationAuthorityRefV1, AuthorizationParticipantV1, AuthorizationPrincipalV1,
        GrantSetV1, ParticipantKindV1, UnsignedAuthorizationContextV1,
        AUTHORIZATION_CONTEXT_FORMAT_V1,
    };

    use super::*;
    use crate::platform::auth::{
        AuthSessionRepository, AuthorityDecision, AuthorityState, AuthorityTarget,
        AuthorizationMaterializationRepository, DesiredAuthorityRecord, IdentityAuthorityRecord,
        IdentityAuthorityRepository, ParticipantBindingRecord, ParticipantBindingRepository,
        ParticipantBindingState, PrincipalRepository, PrincipalState, SessionCreation,
        SessionRecord, SessionRepository, SessionState, SqliteAuthorizationStore,
    };

    const NOW_MS: i64 = 1_735_689_600_000;
    const NOW_SECONDS: i64 = 1_735_689_600;
    const DIGEST: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    async fn repository_conformance<R>(repository: &R) -> Result<(), AuthorizationStateError>
    where
        R: AuthorizationContextRepository
            + AuthorizationMaterializationRepository
            + AuthSessionRepository
            + ParticipantBindingRepository
            + PrincipalRepository
            + SessionRepository
            + Send
            + Sync,
    {
        let (context, snapshot_token) = seed_context(repository).await?;
        let trust = AuthorizationTrustStateRecord {
            authority: "example.test".to_owned(),
            root_key_id: DIGEST.to_owned(),
            root_digest: DIGEST.to_owned(),
            manifest_generation: 1,
            manifest_digest: DIGEST.to_owned(),
            active_issuer_key_id: context.issuer_key_id.clone(),
            updated_at: NOW_MS,
            version: 1,
        };
        assert_eq!(
            repository
                .accept_trust_state(trust.clone(), Vec::new(), NOW_SECONDS)
                .await?,
            trust
        );
        assert_eq!(
            repository
                .accept_trust_state(trust.clone(), Vec::new(), NOW_SECONDS)
                .await?,
            trust
        );
        let mut equivocation = trust.clone();
        equivocation.manifest_digest = context.context_digest.clone();
        assert_eq!(
            repository
                .accept_trust_state(equivocation, Vec::new(), NOW_SECONDS)
                .await,
            Err(AuthorizationStateError::StorageConflict)
        );
        let mut advanced = trust;
        advanced.manifest_generation = 2;
        advanced.manifest_digest = context.context_digest.clone();
        advanced.version = 2;
        assert_eq!(
            repository
                .accept_trust_state(advanced.clone(), Vec::new(), NOW_SECONDS)
                .await?,
            advanced
        );

        repository.touch_session("ses_context", NOW_MS + 1).await?;
        let current_snapshot = repository.load_issuance_snapshot("ses_context").await?;
        assert_eq!(issuance_snapshot_token(&current_snapshot)?, snapshot_token);
        let idempotency = IdempotencyResultRecord {
            scope_key: DIGEST.to_owned(),
            purpose: "authorizationContextIssue".to_owned(),
            signer_id: "ses_context".to_owned(),
            request_id: "req_context".to_owned(),
            request_digest: DIGEST.to_owned(),
            result: Value::Null,
            created_at: NOW_MS,
            expires_at: NOW_MS + 60_000,
        };
        let commit = AuthorizationContextCommit {
            expected_snapshot_token: snapshot_token,
            context: context.clone(),
            idempotency: idempotency.clone(),
            now: NOW_SECONDS,
            minimum_remaining_seconds: 1,
        };
        assert!(matches!(
            repository.commit_context(commit.clone()).await?,
            IdempotentOutcome::Applied(record) if record == context
        ));
        assert!(matches!(
            repository.commit_context(commit).await?,
            IdempotentOutcome::Replayed(value)
                if value == json!({ "contextId": "ctx_context" })
        ));
        assert_eq!(
            repository
                .get_context_by_digest(&context.context_digest)
                .await?,
            Some(context.clone())
        );
        assert_eq!(repository.list_unpublished_contexts(10).await?.len(), 1);
        let published = repository
            .mark_context_published(&context.context_id, 1, NOW_SECONDS + 2)
            .await?;
        assert_eq!(published.published_at, Some(NOW_SECONDS + 2));
        assert!(repository.list_unpublished_contexts(10).await?.is_empty());
        let mut rotated = advanced;
        rotated.manifest_generation = 3;
        rotated.manifest_digest = DIGEST.to_owned();
        rotated.active_issuer_key_id = context.context_digest.clone();
        rotated.version = 3;
        repository
            .accept_trust_state(
                rotated,
                vec![context.issuer_key_id.clone()],
                NOW_SECONDS + 3,
            )
            .await?;
        let revoked = repository
            .get_context_by_id(&context.context_id)
            .await?
            .expect("context remains as revocation evidence");
        assert_eq!(revoked.state, AuthorizationContextState::Revoked);
        assert_eq!(
            revoked.revocation_reason,
            Some(AuthorizationContextRevocationReason::IssuerRevoked)
        );
        assert!(repository
            .list_active_contexts_for_session("ses_context")
            .await?
            .is_empty());
        Ok(())
    }

    async fn seed_context<R>(
        repository: &R,
    ) -> Result<(AuthorizationContextRecord, IssuanceSnapshotToken), AuthorizationStateError>
    where
        R: AuthorizationMaterializationRepository
            + AuthSessionRepository
            + ParticipantBindingRepository
            + PrincipalRepository
            + Send
            + Sync,
    {
        let participant_value = json!({
            "format": "trellis.participant.v1",
            "id": "context-test-app",
            "displayName": "Context test app",
            "description": "Context repository conformance participant.",
            "kind": "app",
            "schemas": {},
            "implements": {},
            "uses": { "required": {}, "optional": {} },
            "state": {},
            "jobQueues": {},
            "eventConsumers": {},
            "resources": {},
        });
        let participant = parse_participant_v1(&participant_value)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let resolved = resolve_participant_v1(&participant, &BTreeMap::new())
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let participant_digest = resolved.participant_digest().to_owned();
        let needs_digest = resolved
            .needs()
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        repository
            .put_participant_binding(ParticipantBindingRecord {
                participant_id: resolved.participant_id().to_owned(),
                participant_kind: ParticipantKindV1::App,
                artifact_digest: participant_digest.clone(),
                needs_digest: needs_digest.clone(),
                participant_json: canonicalize_json(&participant_value)
                    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
                api_artifacts_json: "{}".to_owned(),
                resolved_at: NOW_MS,
                state: ParticipantBindingState::Resolved,
                error: None,
            })
            .await?;
        repository
            .create_principal(crate::platform::auth::PrincipalRecord {
                principal_id: "usr_context".to_owned(),
                kind: PrincipalKind::User,
                state: PrincipalState::Active,
                created_at: NOW_MS,
                updated_at: NOW_MS,
                version: 1,
                disabled_at: None,
                revoked_at: None,
            })
            .await?;
        let session_key = SigningKey::from_bytes(&[17; 32]);
        let session_public_key = URL_SAFE_NO_PAD.encode(session_key.verifying_key().as_bytes());
        let session_key_id =
            URL_SAFE_NO_PAD.encode(Sha256::digest(session_key.verifying_key().as_bytes()));
        let authority = IdentityAuthorityRecord {
            authority_id: "iau_context".to_owned(),
            principal_id: "usr_context".to_owned(),
            participant_id: "context-test-app".to_owned(),
            participant_artifact_digest: participant_digest.clone(),
            accepted_needs_digest: needs_digest.clone(),
            desired_grant_set: GrantSetV1::new(Vec::new()),
            desired_capabilities: Vec::new(),
            state: AuthorityState::Accepted,
            version: 1,
            created_at: NOW_MS,
            updated_at: NOW_MS,
            expires_at: Some(NOW_MS + 600_000),
            decision: Some(AuthorityDecision {
                decided_at: NOW_MS,
                decided_by: "usr_context".to_owned(),
                reason: None,
            }),
        };
        repository
            .create_session(SessionCreation {
                session: SessionRecord {
                    session_id: "ses_context".to_owned(),
                    principal_id: "usr_context".to_owned(),
                    principal_kind: PrincipalKind::User,
                    participant_id: "context-test-app".to_owned(),
                    participant_kind: ParticipantKindV1::App,
                    participant_artifact_digest: participant_digest.clone(),
                    participant_needs_digest: needs_digest.clone(),
                    session_public_key: session_public_key.clone(),
                    session_key_id: session_key_id.clone(),
                    inbox_prefix: "_INBOX.ses_context".to_owned(),
                    state: SessionState::Active,
                    created_at: NOW_MS,
                    last_seen_at: NOW_MS,
                    expires_at: Some(NOW_MS + 600_000),
                    revoked_at: None,
                    version: 1,
                },
                desired_authority: Some(DesiredAuthorityRecord::Identity(authority)),
                runtime_binding: None,
                idempotency: IdempotencyResultRecord {
                    scope_key: URL_SAFE_NO_PAD.encode([2; 32]),
                    purpose: "testSessionCreate".to_owned(),
                    signer_id: "usr_context".to_owned(),
                    request_id: "req_session".to_owned(),
                    request_digest: DIGEST.to_owned(),
                    result: json!({ "sessionId": "ses_context" }),
                    created_at: NOW_MS,
                    expires_at: NOW_MS + 60_000,
                },
                actions: Vec::new(),
            })
            .await?;
        repository
            .reconcile_authority(
                &AuthorityTarget {
                    kind: AuthorityKind::Identity,
                    authority_id: "iau_context".to_owned(),
                },
                NOW_MS,
            )
            .await?;
        let snapshot = repository.load_issuance_snapshot("ses_context").await?;
        let token = issuance_snapshot_token(&snapshot)?;
        let issuer_key = SigningKey::from_bytes(&[23; 32]);
        let issuer_key_id =
            URL_SAFE_NO_PAD.encode(Sha256::digest(issuer_key.verifying_key().as_bytes()));
        let signed = sign_authorization_context_v1(
            UnsignedAuthorizationContextV1 {
                format: AUTHORIZATION_CONTEXT_FORMAT_V1.to_owned(),
                authority: "example.test".to_owned(),
                context_id: "ctx_context".to_owned(),
                issuer_key_id: issuer_key_id.clone(),
                session_id: "ses_context".to_owned(),
                session_key: session_public_key,
                principal: AuthorizationPrincipalV1 {
                    kind: AuthorizationPrincipalKindV1::User,
                    id: "usr_context".to_owned(),
                },
                participant: AuthorizationParticipantV1 {
                    kind: ParticipantKindV1::App,
                    id: "context-test-app".to_owned(),
                    artifact_digest: participant_digest.clone(),
                    needs_digest: needs_digest.clone(),
                },
                authority_ref: AuthorizationAuthorityRefV1 {
                    kind: AuthorizationAuthorityKindV1::Identity,
                    id: "iau_context".to_owned(),
                    version: 1,
                },
                deployment_id: None,
                instance_id: None,
                inbox_prefix: "_INBOX.ses_context".to_owned(),
                issued_at: NOW_SECONDS,
                not_before: NOW_SECONDS - 30,
                expires_at: NOW_SECONDS + 300,
                grant_set: GrantSetV1::new(Vec::new()),
                capabilities: Vec::new(),
                extensions: serde_json::Map::new(),
                critical: Vec::new(),
            },
            &issuer_key,
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let signed_context_json = canonicalize_json(
            &serde_json::to_value(&signed)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let context_digest = signed
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let context_token = encode_authorization_context_token_v1(&signed)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        Ok((
            AuthorizationContextRecord {
                context_id: "ctx_context".to_owned(),
                context_digest,
                session_id: "ses_context".to_owned(),
                principal_id: "usr_context".to_owned(),
                principal_kind: PrincipalKind::User,
                participant_id: "context-test-app".to_owned(),
                participant_artifact_digest: participant_digest,
                participant_needs_digest: needs_digest,
                authority_kind: AuthorityKind::Identity,
                authority_id: "iau_context".to_owned(),
                authority_version: 1,
                materialization_version: 1,
                deployment_id: None,
                instance_id: None,
                issuer_key_id,
                signed_context_json,
                context_token,
                issuance_snapshot_token: token.0.clone(),
                trust_generation: 2,
                issued_at: NOW_SECONDS,
                not_before: NOW_SECONDS - 30,
                expires_at: NOW_SECONDS + 300,
                refresh_at: NOW_SECONDS + 240,
                state: AuthorizationContextState::Active,
                published_at: None,
                revoked_at: None,
                revocation_reason: None,
                version: 1,
            },
            token,
        ))
    }

    #[tokio::test]
    async fn sqlite_context_repository_conforms() -> Result<(), AuthorizationStateError> {
        repository_conformance(&SqliteAuthorizationStore::open_in_memory()?).await
    }

    fn replacement_context(
        base: &AuthorizationContextRecord,
        index: usize,
        issuer_seed: u8,
    ) -> Result<AuthorizationContextRecord, AuthorizationStateError> {
        let value: Value = serde_json::from_str(&base.signed_context_json)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let mut unsigned = parse_authorization_context_v1(&value)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
            .unsigned;
        unsigned.context_id = format!("ctx_replacement_{index:03}");
        let issuer_key = SigningKey::from_bytes(&[issuer_seed; 32]);
        unsigned.issuer_key_id =
            URL_SAFE_NO_PAD.encode(Sha256::digest(issuer_key.verifying_key().as_bytes()));
        let signed = sign_authorization_context_v1(unsigned, &issuer_key)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let mut context = base.clone();
        context.context_id = signed.unsigned.context_id.clone();
        context.issuer_key_id = signed.unsigned.issuer_key_id.clone();
        context.signed_context_json = canonicalize_json(
            &serde_json::to_value(&signed)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        context.context_token = encode_authorization_context_token_v1(&signed)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        context.context_digest = signed
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        Ok(context)
    }

    #[tokio::test]
    async fn concurrent_context_commits_reuse_one_and_bound_active_overlap(
    ) -> Result<(), AuthorizationStateError> {
        let repository = Arc::new(SqliteAuthorizationStore::open_in_memory()?);
        let (base, snapshot_token) = seed_context(repository.as_ref()).await?;
        let commits = (0..100).map(|index| {
            let repository = Arc::clone(&repository);
            let snapshot_token = snapshot_token.clone();
            let context = replacement_context(&base, index, 23).expect("valid test context");
            async move {
                repository
                    .commit_context(AuthorizationContextCommit {
                        expected_snapshot_token: snapshot_token,
                        context,
                        idempotency: IdempotencyResultRecord {
                            scope_key: URL_SAFE_NO_PAD
                                .encode(Sha256::digest(format!("concurrent-{index}"))),
                            purpose: "authorizationContextIssue".to_owned(),
                            signer_id: "ses_context".to_owned(),
                            request_id: format!("req_concurrent_{index:03}"),
                            request_digest: DIGEST.to_owned(),
                            result: Value::Null,
                            created_at: NOW_MS,
                            expires_at: NOW_MS + 60_000,
                        },
                        now: NOW_SECONDS,
                        minimum_remaining_seconds: 1,
                    })
                    .await
            }
        });
        let outcomes = futures_util::future::join_all(commits).await;
        let mut digests = BTreeSet::new();
        for outcome in outcomes {
            match outcome? {
                IdempotentOutcome::Applied(context) => {
                    digests.insert(context.context_digest);
                }
                IdempotentOutcome::Replayed(_) => panic!("unique requests must not replay"),
            }
        }
        assert_eq!(digests.len(), 1);
        assert_eq!(
            repository
                .list_active_contexts_for_session("ses_context")
                .await?
                .len(),
            1
        );
        let publish_actions = repository
            .run(|connection| {
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM auth_post_commit_actions WHERE kind = 'context_publish'",
                        [],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(sql_error)
            })
            .await?;
        assert_eq!(publish_actions, 1);

        for (index, seed) in [(100, 24), (101, 25)] {
            let context = replacement_context(&base, index, seed)?;
            repository
                .commit_context(AuthorizationContextCommit {
                    expected_snapshot_token: snapshot_token.clone(),
                    context,
                    idempotency: IdempotencyResultRecord {
                        scope_key: URL_SAFE_NO_PAD
                            .encode(Sha256::digest(format!("overlap-{index}"))),
                        purpose: "authorizationContextIssue".to_owned(),
                        signer_id: "ses_context".to_owned(),
                        request_id: format!("req_overlap_{index}"),
                        request_digest: DIGEST.to_owned(),
                        result: Value::Null,
                        created_at: NOW_MS,
                        expires_at: NOW_MS + 60_000,
                    },
                    now: NOW_SECONDS,
                    minimum_remaining_seconds: 1,
                })
                .await?;
        }
        assert_eq!(
            repository
                .list_active_contexts_for_session("ses_context")
                .await?
                .len(),
            2
        );
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_session_expiry_revokes_context_but_liveness_does_not(
    ) -> Result<(), AuthorizationStateError> {
        let repository = SqliteAuthorizationStore::open_in_memory()?;
        let (context, snapshot_token) = seed_context(&repository).await?;
        repository
            .commit_context(AuthorizationContextCommit {
                expected_snapshot_token: snapshot_token,
                context: context.clone(),
                idempotency: IdempotencyResultRecord {
                    scope_key: DIGEST.to_owned(),
                    purpose: "authorizationContextIssue".to_owned(),
                    signer_id: "ses_context".to_owned(),
                    request_id: "req_context_expiry".to_owned(),
                    request_digest: DIGEST.to_owned(),
                    result: Value::Null,
                    created_at: NOW_MS,
                    expires_at: NOW_MS + 60_000,
                },
                now: NOW_SECONDS,
                minimum_remaining_seconds: 1,
            })
            .await?;

        repository.touch_session("ses_context", NOW_MS + 1).await?;
        repository
            .reconcile_authority(
                &AuthorityTarget {
                    kind: AuthorityKind::Identity,
                    authority_id: "iau_context".to_owned(),
                },
                NOW_MS + 1,
            )
            .await?;
        assert_eq!(
            repository
                .get_context_by_digest(&context.context_digest)
                .await?
                .expect("context")
                .state,
            AuthorizationContextState::Active
        );

        repository
            .expire_session("ses_context", 1, NOW_MS + 600_000)
            .await?;
        let revoked = repository
            .get_context_by_digest(&context.context_digest)
            .await?
            .expect("context");
        assert_eq!(revoked.state, AuthorizationContextState::Revoked);
        assert_eq!(
            revoked.revocation_reason,
            Some(AuthorizationContextRevocationReason::SessionExpired)
        );
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_authority_change_revokes_active_context() -> Result<(), AuthorizationStateError>
    {
        let repository = SqliteAuthorizationStore::open_in_memory()?;
        let (context, snapshot_token) = seed_context(&repository).await?;
        repository
            .commit_context(AuthorizationContextCommit {
                expected_snapshot_token: snapshot_token,
                context: context.clone(),
                idempotency: IdempotencyResultRecord {
                    scope_key: DIGEST.to_owned(),
                    purpose: "authorizationContextIssue".to_owned(),
                    signer_id: "ses_context".to_owned(),
                    request_id: "req_context_authority_change".to_owned(),
                    request_digest: DIGEST.to_owned(),
                    result: Value::Null,
                    created_at: NOW_MS,
                    expires_at: NOW_MS + 60_000,
                },
                now: NOW_SECONDS,
                minimum_remaining_seconds: 1,
            })
            .await?;
        let mut authority = repository
            .get_identity_authority("usr_context", "context-test-app")
            .await?
            .expect("authority");
        authority.version = 2;
        authority.updated_at = NOW_MS + 1;
        authority.expires_at = Some(NOW_MS + 500_000);
        repository
            .put_identity_authority(authority, Some(1))
            .await?;

        let revoked = repository
            .get_context_by_digest(&context.context_digest)
            .await?
            .expect("context");
        assert_eq!(revoked.state, AuthorizationContextState::Revoked);
        assert_eq!(
            revoked.revocation_reason,
            Some(AuthorizationContextRevocationReason::AuthorityChanged)
        );
        Ok(())
    }
}
