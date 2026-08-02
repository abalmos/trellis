use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};

use super::super::authority::ContextRepository;
use super::super::authority::{
    materialization_semantics_equal, validate_materialization, AuthorityMaterializationSnapshot,
    AuthorityReconciliationOutcome, AuthoritySnapshotToken, AuthoritySubjectRecord,
    IssuanceSnapshot,
};
use super::super::context::{
    revoke_sql_contexts, AuthorizationContextRevocationReason, AuthorizationContextSelector,
};
use super::super::materializer::{materialize_authority, transition_for_change};
use super::super::{
    AuthorityKind, AuthorityTarget, AuthorizationStateError, DependencyEvidence,
    DesiredAuthorityRecord, MaterializationReplacement, MaterializedAuthorityRecord, PrincipalKind,
    ResourceBindingEvidence, RuntimeEvidence,
};
use super::authority::{load_deployment_authority, load_identity_authority};
use super::common::{
    decode_enum, decode_json, encode_enum, encode_json, from_sql_version, map_write_error,
    sql_error, to_sql_version,
};
use super::evidence::{
    load_dependency_evidence, load_deployment, load_desired_authority, load_evidence_scope,
    load_participant_binding, load_resource_evidence, load_runtime_evidence,
};
use super::principals::load_principal;
use super::sessions::load_session;
use super::SqliteAuthorizationStore;

#[async_trait]
impl ContextRepository for SqliteAuthorizationStore {
    async fn get_materialized_authority(
        &self,
        kind: AuthorityKind,
        authority_id: &str,
    ) -> Result<Option<MaterializationReplacement>, AuthorizationStateError> {
        let authority_id = authority_id.to_owned();
        self.run_read(move |connection| load_materialization(connection, kind, &authority_id))
            .await
    }

    async fn reconcile_authority(
        &self,
        target: &AuthorityTarget,
        now: i64,
    ) -> Result<AuthorityReconciliationOutcome, AuthorizationStateError> {
        let target = target.clone();
        self.run(move |connection| {
            let transaction = connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(sql_error)?;
            let snapshot = sqlite_materialization_snapshot(&transaction, &target)?;
            let token = snapshot.token.clone();
            let previous = snapshot.previous.clone();
            let mut replacement = materialize_authority(&snapshot, now);
            if replacement.as_ref().is_some_and(|current| {
                previous
                    .as_ref()
                    .is_some_and(|previous| materialization_semantics_equal(previous, current))
            }) {
                transaction.commit().map_err(sql_error)?;
                return Ok(AuthorityReconciliationOutcome {
                    target,
                    snapshot_token: token,
                    materialization: previous,
                    changed: false,
                });
            }
            if replacement.is_none() && previous.is_none() {
                transaction.commit().map_err(sql_error)?;
                return Ok(AuthorityReconciliationOutcome {
                    target,
                    snapshot_token: token,
                    materialization: None,
                    changed: false,
                });
            }
            if let Some(current) = replacement.as_mut() {
                current.authority.materialization_version = match previous.as_ref() {
                    Some(previous) => next_sql_version(
                        "materializationVersion",
                        previous.authority.materialization_version,
                    )?,
                    None => 1,
                };
                validate_materialization(current)?;
                write_materialization(&transaction, current)?;
            } else {
                transaction
                    .execute(
                        "DELETE FROM auth_materialized_authorities
                         WHERE authority_kind = ?1 AND authority_id = ?2",
                        params![encode_enum(target.kind)?, target.authority_id],
                    )
                    .map_err(map_write_error)?;
            }
            if let Some(transition) =
                transition_for_change(previous.as_ref(), replacement.as_ref(), now)?
            {
                transaction
                    .execute(
                        "INSERT OR IGNORE INTO auth_transition_outbox (
                            event_id, transition_json, created_at
                         ) VALUES (?1, ?2, ?3)",
                        params![
                            transition.event_id,
                            encode_json(&transition)?,
                            transition.created_at,
                        ],
                    )
                    .map_err(map_write_error)?;
                revoke_sql_contexts(
                    &transaction,
                    &AuthorizationContextSelector::Authority(
                        target.kind,
                        target.authority_id.clone(),
                    ),
                    AuthorizationContextRevocationReason::MaterializationChanged,
                    now.div_euclid(1_000),
                )?;
            }
            transaction.commit().map_err(sql_error)?;
            Ok(AuthorityReconciliationOutcome {
                target,
                snapshot_token: token,
                materialization: replacement,
                changed: true,
            })
        })
        .await
    }

    async fn list_reconciliation_targets(
        &self,
    ) -> Result<Vec<AuthorityTarget>, AuthorizationStateError> {
        self.run_read(|connection| {
            let mut statement = connection
                .prepare(
                    "SELECT authority_kind, authority_id FROM (
                        SELECT 'identity' AS authority_kind, authority_id
                        FROM auth_identity_authorities
                        UNION
                        SELECT 'deployment', authority_id
                        FROM auth_deployment_authorities
                        UNION
                        SELECT authority_kind, authority_id
                        FROM auth_materialized_authorities
                     ) ORDER BY authority_kind, authority_id",
                )
                .map_err(sql_error)?;
            let records = statement
                .query_map([], |row| {
                    Ok(AuthorityTarget {
                        kind: decode_enum(row.get::<_, String>(0)?)?,
                        authority_id: row.get(1)?,
                    })
                })
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            Ok(records)
        })
        .await
    }

    async fn load_issuance_snapshot(
        &self,
        session_id: &str,
    ) -> Result<IssuanceSnapshot, AuthorizationStateError> {
        let session_id = session_id.to_owned();
        self.run_read(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            let snapshot = sqlite_issuance_snapshot(&transaction, &session_id)?;
            transaction.commit().map_err(sql_error)?;
            Ok(snapshot)
        })
        .await
    }
}

pub(in crate::platform::auth) fn sqlite_materialization_snapshot(
    connection: &Connection,
    target: &AuthorityTarget,
) -> Result<AuthorityMaterializationSnapshot, AuthorizationStateError> {
    let authority = load_desired_authority(connection, target)?;
    let subject = match authority.as_ref() {
        Some(DesiredAuthorityRecord::Identity(record)) => {
            load_principal(connection, &record.principal_id)?.map(AuthoritySubjectRecord::Identity)
        }
        Some(DesiredAuthorityRecord::Deployment(record)) => {
            load_deployment(connection, &record.deployment_id)?
                .map(AuthoritySubjectRecord::Deployment)
        }
        None => None,
    };
    let participant = match authority.as_ref() {
        Some(authority) => load_participant_binding(
            connection,
            authority.participant_id(),
            authority.participant_artifact_digest(),
        )?,
        None => None,
    };
    let dependencies = load_dependency_evidence(connection, target)?;
    let dependency_scope = load_evidence_scope(connection, "auth_dependency_evidence", target)?;
    let resources = load_resource_evidence(connection, target)?;
    let resource_scope = load_evidence_scope(connection, "auth_resource_binding_evidence", target)?;
    let previous = load_materialization(connection, target.kind, &target.authority_id)?;
    let token = sql_snapshot_token(&(
        &authority,
        &subject,
        &participant,
        &dependency_scope,
        &dependencies,
        &resource_scope,
        &resources,
        &previous,
    ))?;
    Ok(AuthorityMaterializationSnapshot {
        target: target.clone(),
        token,
        authority,
        subject,
        participant,
        dependencies,
        dependency_scope,
        resources,
        resource_scope,
        previous,
    })
}

pub(in crate::platform::auth) fn sqlite_issuance_snapshot(
    connection: &Connection,
    session_id: &str,
) -> Result<IssuanceSnapshot, AuthorizationStateError> {
    let session = load_session(connection, session_id)?;
    let principal = match session.as_ref() {
        Some(session) => load_principal(connection, &session.principal_id)?,
        None => None,
    };
    let participant = match session.as_ref() {
        Some(session) => load_participant_binding(
            connection,
            &session.participant_id,
            &session.participant_artifact_digest,
        )?,
        None => None,
    };
    let runtime = match principal.as_ref().map(|principal| principal.kind) {
        Some(PrincipalKind::User) => Some(RuntimeEvidence::User),
        Some(PrincipalKind::Service | PrincipalKind::Device) => {
            load_runtime_evidence(connection, session_id)?
        }
        None => None,
    };
    let deployment = match runtime.as_ref() {
        Some(RuntimeEvidence::Service(evidence)) => {
            load_deployment(connection, &evidence.deployment_id)?
        }
        Some(RuntimeEvidence::Device(evidence)) => {
            load_deployment(connection, &evidence.deployment_id)?
        }
        Some(RuntimeEvidence::User) | None => None,
    };
    let authority = match (session.as_ref(), principal.as_ref(), runtime.as_ref()) {
        (Some(session), Some(principal), Some(RuntimeEvidence::User)) => {
            load_identity_authority(connection, &principal.principal_id, &session.participant_id)?
                .map(DesiredAuthorityRecord::Identity)
        }
        (Some(session), _, Some(RuntimeEvidence::Service(evidence))) => {
            load_deployment_authority(connection, &evidence.deployment_id, &session.participant_id)?
                .map(DesiredAuthorityRecord::Deployment)
        }
        (Some(session), _, Some(RuntimeEvidence::Device(evidence))) => {
            load_deployment_authority(connection, &evidence.deployment_id, &session.participant_id)?
                .map(DesiredAuthorityRecord::Deployment)
        }
        _ => None,
    };
    let materialization = match authority.as_ref() {
        Some(authority) => {
            let target = authority.target();
            load_materialization(connection, target.kind, &target.authority_id)?
        }
        None => None,
    };
    Ok(IssuanceSnapshot {
        session,
        principal,
        participant,
        runtime,
        deployment,
        authority,
        materialization,
    })
}

pub(in crate::platform::auth) fn sql_snapshot_token<T: serde::Serialize>(
    value: &T,
) -> Result<AuthoritySnapshotToken, AuthorizationStateError> {
    let encoded = serde_json::to_vec(value).map_err(|error| {
        AuthorizationStateError::Storage(format!("cannot encode authority snapshot: {error}"))
    })?;
    Ok(AuthoritySnapshotToken(
        URL_SAFE_NO_PAD.encode(Sha256::digest(encoded)),
    ))
}

pub(in crate::platform::auth) fn next_sql_version(
    field: &str,
    current: u64,
) -> Result<u64, AuthorizationStateError> {
    let next = current
        .checked_add(1)
        .ok_or_else(|| AuthorizationStateError::InvalidRecord(format!("{field} overflow")))?;
    super::super::domain::require_positive(field, next)?;
    Ok(next)
}

pub(in crate::platform::auth) fn decode_dependency(
    row: &Row<'_>,
) -> rusqlite::Result<DependencyEvidence> {
    Ok(DependencyEvidence {
        alias: row.get(0)?,
        required: row.get(1)?,
        api_id: row.get(2)?,
        api_digest: row.get(3)?,
        provider_participant_id: row.get(4)?,
        provider_deployment_id: row.get(5)?,
        provider_instance_id: row.get(6)?,
        state: decode_enum(row.get::<_, String>(7)?)?,
        observed_at: row.get(8)?,
    })
}

pub(in crate::platform::auth) fn decode_resource(
    row: &Row<'_>,
) -> rusqlite::Result<ResourceBindingEvidence> {
    Ok(ResourceBindingEvidence {
        resource_kind: row.get(0)?,
        local_name: row.get(1)?,
        binding_id: row.get(2)?,
        owner_participant_id: row.get(3)?,
        provider_identity: decode_json(row.get(4)?)?,
        state: decode_enum(row.get::<_, String>(5)?)?,
        materialized_at: row.get(6)?,
        error: row.get(7)?,
    })
}

pub(in crate::platform::auth) fn load_materialization(
    connection: &Connection,
    kind: AuthorityKind,
    authority_id: &str,
) -> Result<Option<MaterializationReplacement>, AuthorizationStateError> {
    let authority = connection
        .query_row(
            "SELECT materialization_id, authority_kind, authority_id,
                authority_version, materialization_version, subject_id,
                participant_id, participant_kind, participant_artifact_digest,
                participant_needs_digest, effective_grant_set_json,
                effective_capabilities_json, state, reconciled_at, error, expires_at
         FROM auth_materialized_authorities
         WHERE authority_kind = ?1 AND authority_id = ?2",
            params![encode_enum(kind)?, authority_id],
            decode_materialized_authority,
        )
        .optional()
        .map_err(sql_error)?;
    let Some(authority) = authority else {
        return Ok(None);
    };
    let mut dependency_statement = connection
        .prepare(
            "SELECT alias, required, api_id, api_digest, provider_participant_id,
                provider_deployment_id, provider_instance_id, state, observed_at
         FROM auth_materialized_dependencies
         WHERE materialization_id = ?1 ORDER BY required DESC, alias",
        )
        .map_err(sql_error)?;
    let dependencies = dependency_statement
        .query_map([&authority.materialization_id], decode_dependency)
        .map_err(sql_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(sql_error)?;
    drop(dependency_statement);
    let mut resource_statement = connection
        .prepare(
            "SELECT resource_kind, local_name, binding_id, owner_participant_id,
                provider_identity, state, materialized_at, error
         FROM auth_materialized_resource_bindings
         WHERE materialization_id = ?1 ORDER BY resource_kind, local_name",
        )
        .map_err(sql_error)?;
    let resources = resource_statement
        .query_map([&authority.materialization_id], decode_resource)
        .map_err(sql_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(sql_error)?;
    Ok(Some(MaterializationReplacement {
        authority,
        dependencies,
        resources,
    }))
}

pub(in crate::platform::auth) fn decode_materialized_authority(
    row: &Row<'_>,
) -> rusqlite::Result<MaterializedAuthorityRecord> {
    Ok(MaterializedAuthorityRecord {
        materialization_id: row.get(0)?,
        authority_kind: decode_enum(row.get::<_, String>(1)?)?,
        authority_id: row.get(2)?,
        authority_version: from_sql_version(row.get(3)?)?,
        materialization_version: from_sql_version(row.get(4)?)?,
        subject_id: row.get(5)?,
        participant_id: row.get(6)?,
        participant_kind: decode_enum(row.get::<_, String>(7)?)?,
        participant_artifact_digest: row.get(8)?,
        participant_needs_digest: row.get(9)?,
        effective_grant_set: decode_json(row.get::<_, String>(10)?)?,
        effective_capabilities: decode_json(row.get::<_, String>(11)?)?,
        state: decode_enum(row.get::<_, String>(12)?)?,
        reconciled_at: row.get(13)?,
        error: row.get(14)?,
        expires_at: row.get(15)?,
    })
}

pub(in crate::platform::auth) fn write_materialization(
    transaction: &Transaction<'_>,
    replacement: &MaterializationReplacement,
) -> Result<(), AuthorizationStateError> {
    let record = &replacement.authority;
    transaction
        .execute(
            "INSERT INTO auth_materialized_authorities (
            materialization_id, authority_kind, authority_id, authority_version,
            materialization_version, subject_id, participant_id, participant_kind,
            participant_artifact_digest, participant_needs_digest,
            effective_grant_set_json, effective_capabilities_json, state,
            reconciled_at, error, expires_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
         ON CONFLICT(authority_kind, authority_id) DO UPDATE SET
            authority_version = excluded.authority_version,
            materialization_version = excluded.materialization_version,
            subject_id = excluded.subject_id,
            participant_id = excluded.participant_id,
            participant_kind = excluded.participant_kind,
            participant_artifact_digest = excluded.participant_artifact_digest,
            participant_needs_digest = excluded.participant_needs_digest,
            effective_grant_set_json = excluded.effective_grant_set_json,
            effective_capabilities_json = excluded.effective_capabilities_json,
            state = excluded.state,
            reconciled_at = excluded.reconciled_at,
            error = excluded.error,
            expires_at = excluded.expires_at",
            params![
                record.materialization_id,
                encode_enum(record.authority_kind)?,
                record.authority_id,
                to_sql_version(record.authority_version)?,
                to_sql_version(record.materialization_version)?,
                record.subject_id,
                record.participant_id,
                encode_enum(record.participant_kind)?,
                record.participant_artifact_digest,
                record.participant_needs_digest,
                encode_json(&record.effective_grant_set)?,
                encode_json(&record.effective_capabilities)?,
                encode_enum(record.state)?,
                record.reconciled_at,
                record.error,
                record.expires_at,
            ],
        )
        .map_err(map_write_error)?;
    transaction
        .execute(
            "DELETE FROM auth_materialized_dependencies WHERE materialization_id = ?1",
            [&record.materialization_id],
        )
        .map_err(map_write_error)?;
    transaction
        .execute(
            "DELETE FROM auth_materialized_resource_bindings WHERE materialization_id = ?1",
            [&record.materialization_id],
        )
        .map_err(map_write_error)?;
    for item in &replacement.dependencies {
        transaction
            .execute(
                "INSERT INTO auth_materialized_dependencies (
                materialization_id, alias, required, api_id, api_digest,
                provider_participant_id, provider_deployment_id,
                provider_instance_id, state, observed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    record.materialization_id,
                    item.alias,
                    item.required,
                    item.api_id,
                    item.api_digest,
                    item.provider_participant_id,
                    item.provider_deployment_id,
                    item.provider_instance_id,
                    encode_enum(item.state)?,
                    item.observed_at,
                ],
            )
            .map_err(map_write_error)?;
    }
    for item in &replacement.resources {
        transaction
            .execute(
                "INSERT INTO auth_materialized_resource_bindings (
                materialization_id, resource_kind, local_name, binding_id,
                owner_participant_id, provider_identity, state,
                materialized_at, error
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    record.materialization_id,
                    item.resource_kind,
                    item.local_name,
                    item.binding_id,
                    item.owner_participant_id,
                    encode_json(&item.provider_identity)?,
                    encode_enum(item.state)?,
                    item.materialized_at,
                    item.error,
                ],
            )
            .map_err(map_write_error)?;
    }
    Ok(())
}
