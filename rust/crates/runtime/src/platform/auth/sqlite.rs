use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};
use sha2::{Digest, Sha256};

use super::materializer::{materialize_authority, transition_for_change};
use super::repository::{
    deployment_enforceability_equal, identity_enforceability_equal,
    materialization_semantics_equal, validate_dependency_evidence, validate_deployment_authority,
    validate_deployment_evidence, validate_device, validate_device_delegation,
    validate_identity_authority, validate_materialization, validate_principal,
    validate_provider_identity, validate_resource_evidence, validate_runtime_instance,
    validate_session, validate_session_runtime_binding, AuthorityMaterializationSnapshot,
    AuthorityReconciliationOutcome, AuthoritySnapshotToken, AuthoritySubjectRecord,
    IssuanceSnapshot,
};
use super::{
    AuthorityDecision, AuthorityEvidenceScope, AuthorityKind, AuthorityTarget,
    AuthorizationMaterializationRepository, AuthorizationStateError,
    AuthorizationTransitionOutboxRecord, DelegationEvidence, DependencyEvidence,
    DeploymentAuthorityRecord, DeploymentAuthorityRepository, DeploymentRecord,
    DesiredAuthorityRecord, DeviceDelegationRecord, DeviceDelegationState, DeviceEvidence,
    DeviceRecord, DeviceState, EvidenceRepository, IdentityAuthorityRecord,
    IdentityAuthorityRepository, MaterializationReplacement, MaterializedAuthorityRecord,
    ParticipantBindingRecord, ParticipantBindingRepository, PrincipalAuthorizationChange,
    PrincipalKind, PrincipalRecord, PrincipalRepository, PrincipalState, ProviderIdentityLink,
    ProviderIdentityRepository, ResourceBindingEvidence, RuntimeEvidence, RuntimeInstanceRecord,
    RuntimeInstanceState, ServiceEvidence, SessionRecord, SessionRepository, SessionRuntimeBinding,
    SessionState,
};
use crate::storage::{SqliteStore, StoreError};

/// Owner-scoped SQLite implementation of every authorization repository port.
#[derive(Clone, Debug)]
pub struct SqliteAuthorizationStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteAuthorizationStore {
    pub(crate) fn open(store: &SqliteStore) -> Result<Self, StoreError> {
        Ok(Self {
            connection: Arc::new(Mutex::new(store.open()?)),
        })
    }

    /// Create an isolated migrated in-memory store.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationStateError::Storage`] if SQLite cannot open the
    /// database or apply the platform authorization schema.
    pub fn open_in_memory() -> Result<Self, AuthorizationStateError> {
        let connection = Connection::open_in_memory().map_err(sql_error)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(sql_error)?;
        let migrated = connection
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sqlite_master
                    WHERE type = 'table' AND name = 'auth_principals'
                 )",
                [],
                |row| row.get::<_, bool>(0),
            )
            .map_err(sql_error)?;
        if !migrated {
            connection
                .execute_batch(include_str!(
                    "../../storage/sqlite/platform/V1001__authorization_state.sql"
                ))
                .map_err(sql_error)?;
        }
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    #[cfg(test)]
    pub(crate) fn open_path(path: &std::path::Path) -> Result<Self, AuthorizationStateError> {
        let connection = Connection::open(path).map_err(sql_error)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(sql_error)?;
        let migrated = connection
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sqlite_master
                    WHERE type = 'table' AND name = 'auth_principals'
                 )",
                [],
                |row| row.get::<_, bool>(0),
            )
            .map_err(sql_error)?;
        if !migrated {
            connection
                .execute_batch(include_str!(
                    "../../storage/sqlite/platform/V1001__authorization_state.sql"
                ))
                .map_err(sql_error)?;
        }
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    async fn run<T, F>(&self, operation: F) -> Result<T, AuthorizationStateError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, AuthorizationStateError> + Send + 'static,
    {
        let connection = Arc::clone(&self.connection);
        tokio::task::spawn_blocking(move || {
            let mut connection = connection.lock().map_err(|_| {
                AuthorizationStateError::Storage("SQLite connection lock poisoned".to_owned())
            })?;
            operation(&mut connection)
        })
        .await
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?
    }
}

#[async_trait]
impl PrincipalRepository for SqliteAuthorizationStore {
    async fn get_principal(
        &self,
        id: &str,
    ) -> Result<Option<PrincipalRecord>, AuthorizationStateError> {
        let id = id.to_owned();
        self.run(move |connection| load_principal(connection, &id))
            .await
    }

    async fn create_principal(
        &self,
        record: PrincipalRecord,
    ) -> Result<PrincipalRecord, AuthorizationStateError> {
        validate_principal(&record)?;
        self.run(move |connection| {
            connection
                .execute(
                    "INSERT INTO auth_principals (
                        principal_id, kind, state, created_at, updated_at, version,
                        disabled_at, revoked_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        record.principal_id,
                        encode_enum(record.kind)?,
                        encode_enum(record.state)?,
                        record.created_at,
                        record.updated_at,
                        to_sql_version(record.version)?,
                        record.disabled_at,
                        record.revoked_at,
                    ],
                )
                .map_err(map_write_error)?;
            Ok(record)
        })
        .await
    }

    async fn update_principal_authorization_state(
        &self,
        id: &str,
        expected_version: u64,
        change: PrincipalAuthorizationChange,
    ) -> Result<PrincipalRecord, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("changedAt", change.changed_at)?;
        let id = id.to_owned();
        self.run(move |connection| {
            let current = load_principal(connection, &id)?
                .ok_or(AuthorizationStateError::PrincipalMissing)?;
            if current.version != expected_version {
                return Err(AuthorizationStateError::StorageConflict);
            }
            if current.state == change.state {
                return Ok(current);
            }
            if current.state == PrincipalState::Revoked {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let next_version = expected_version.checked_add(1).ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("principal version overflow".to_owned())
            })?;
            let disabled_at =
                (change.state == PrincipalState::Disabled).then_some(change.changed_at);
            let revoked_at = (change.state == PrincipalState::Revoked).then_some(change.changed_at);
            let changed = connection
                .execute(
                    "UPDATE auth_principals SET
                        state = ?1, updated_at = ?2, version = ?3,
                        disabled_at = ?4, revoked_at = ?5
                     WHERE principal_id = ?6 AND version = ?7",
                    params![
                        encode_enum(change.state)?,
                        change.changed_at,
                        to_sql_version(next_version)?,
                        disabled_at,
                        revoked_at,
                        id,
                        to_sql_version(expected_version)?,
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            load_principal(connection, &id)?.ok_or(AuthorizationStateError::PrincipalMissing)
        })
        .await
    }
}

#[async_trait]
impl ProviderIdentityRepository for SqliteAuthorizationStore {
    async fn get_provider_identity(
        &self,
        provider: &str,
        subject: &str,
    ) -> Result<Option<ProviderIdentityLink>, AuthorizationStateError> {
        let provider = provider.to_owned();
        let subject = subject.to_owned();
        self.run(move |connection| {
            connection
                .query_row(
                    "SELECT provider, provider_subject, principal_id, linked_at, last_seen_at
                     FROM auth_provider_identities
                     WHERE provider = ?1 AND provider_subject = ?2",
                    params![provider, subject],
                    |row| {
                        Ok(ProviderIdentityLink {
                            provider: row.get(0)?,
                            provider_subject: row.get(1)?,
                            principal_id: row.get(2)?,
                            linked_at: row.get(3)?,
                            last_seen_at: row.get(4)?,
                        })
                    },
                )
                .optional()
                .map_err(sql_error)
        })
        .await
    }

    async fn link_provider_identity(
        &self,
        link: ProviderIdentityLink,
    ) -> Result<(), AuthorizationStateError> {
        validate_provider_identity(&link)?;
        self.run(move |connection| {
            let principal = load_principal(connection, &link.principal_id)?
                .ok_or(AuthorizationStateError::PrincipalMissing)?;
            if principal.kind != PrincipalKind::User {
                return Err(AuthorizationStateError::InvalidRecord(
                    "provider identities may link only to user principals".to_owned(),
                ));
            }
            connection
                .execute(
                    "INSERT INTO auth_provider_identities (
                        provider, provider_subject, principal_id, linked_at, last_seen_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        link.provider,
                        link.provider_subject,
                        link.principal_id,
                        link.linked_at,
                        link.last_seen_at,
                    ],
                )
                .map_err(map_write_error)?;
            Ok(())
        })
        .await
    }
}

#[async_trait]
impl ParticipantBindingRepository for SqliteAuthorizationStore {
    async fn get_participant_binding(
        &self,
        participant_id: &str,
        artifact_digest: &str,
    ) -> Result<Option<ParticipantBindingRecord>, AuthorizationStateError> {
        let participant_id = participant_id.to_owned();
        let artifact_digest = artifact_digest.to_owned();
        self.run(move |connection| {
            connection
                .query_row(
                    "SELECT participant_id, participant_kind, artifact_digest, needs_digest,
                            participant_json, api_artifacts_json, resolved_at, state, error
                     FROM auth_participant_bindings
                     WHERE participant_id = ?1 AND artifact_digest = ?2",
                    params![participant_id, artifact_digest],
                    decode_participant_binding,
                )
                .optional()
                .map_err(sql_error)
                .and_then(|value| {
                    value.map_or(Ok(None), |value| {
                        value.resolve()?;
                        Ok(Some(value))
                    })
                })
        })
        .await
    }

    async fn put_participant_binding(
        &self,
        binding: ParticipantBindingRecord,
    ) -> Result<(), AuthorizationStateError> {
        super::domain::require_protocol_timestamp("resolvedAt", binding.resolved_at)?;
        binding.resolve()?;
        self.run(move |connection| {
            connection
                .execute(
                    "INSERT INTO auth_participant_bindings (
                        participant_id, participant_kind, artifact_digest, needs_digest,
                        participant_json, api_artifacts_json, resolved_at, state, error
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                     ON CONFLICT(participant_id, artifact_digest) DO UPDATE SET
                        participant_kind = excluded.participant_kind,
                        needs_digest = excluded.needs_digest,
                        participant_json = excluded.participant_json,
                        api_artifacts_json = excluded.api_artifacts_json,
                        resolved_at = excluded.resolved_at,
                        state = excluded.state,
                        error = excluded.error",
                    params![
                        binding.participant_id,
                        encode_enum(binding.participant_kind)?,
                        binding.artifact_digest,
                        binding.needs_digest,
                        binding.participant_json,
                        binding.api_artifacts_json,
                        binding.resolved_at,
                        encode_enum(binding.state)?,
                        binding.error,
                    ],
                )
                .map_err(map_write_error)?;
            Ok(())
        })
        .await
    }
}

#[async_trait]
impl SessionRepository for SqliteAuthorizationStore {
    async fn get_session(
        &self,
        id: &str,
    ) -> Result<Option<SessionRecord>, AuthorizationStateError> {
        let id = id.to_owned();
        self.run(move |connection| load_session(connection, &id))
            .await
    }

    async fn create_session(
        &self,
        record: SessionRecord,
    ) -> Result<SessionRecord, AuthorizationStateError> {
        validate_session(&record)?;
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            let principal = load_principal(&transaction, &record.principal_id)?
                .ok_or(AuthorizationStateError::PrincipalMissing)?;
            if principal.kind != record.principal_kind {
                return Err(AuthorizationStateError::InvalidRecord(
                    "session principal kind does not match principal".to_owned(),
                ));
            }
            let binding = load_participant_binding(
                &transaction,
                &record.participant_id,
                &record.participant_artifact_digest,
            )?
            .ok_or(AuthorizationStateError::ParticipantMissing)?;
            if binding.participant_kind != record.participant_kind {
                return Err(AuthorizationStateError::InvalidRecord(
                    "session participant kind does not match participant binding".to_owned(),
                ));
            }
            if binding.needs_digest != record.participant_needs_digest {
                return Err(AuthorizationStateError::NeedsDigestMismatch);
            }
            transaction
                .execute(
                    "INSERT INTO auth_sessions (
                        session_id, principal_id, principal_kind, participant_id,
                        participant_kind, participant_artifact_digest,
                        participant_needs_digest, session_public_key, session_key_id,
                        inbox_prefix, state, created_at, last_seen_at, expires_at,
                        revoked_at, version
                     ) VALUES (
                        ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                        ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
                     )",
                    params![
                        record.session_id,
                        record.principal_id,
                        encode_enum(record.principal_kind)?,
                        record.participant_id,
                        encode_enum(record.participant_kind)?,
                        record.participant_artifact_digest,
                        record.participant_needs_digest,
                        record.session_public_key,
                        record.session_key_id,
                        record.inbox_prefix,
                        encode_enum(record.state)?,
                        record.created_at,
                        record.last_seen_at,
                        record.expires_at,
                        record.revoked_at,
                        to_sql_version(record.version)?,
                    ],
                )
                .map_err(map_write_error)?;
            transaction.commit().map_err(sql_error)?;
            Ok(record)
        })
        .await
    }

    async fn touch_session(
        &self,
        id: &str,
        observed_at: i64,
    ) -> Result<(), AuthorizationStateError> {
        super::domain::require_protocol_timestamp("observedAt", observed_at)?;
        let id = id.to_owned();
        self.run(move |connection| {
            let changed = connection
                .execute(
                    "UPDATE auth_sessions
                     SET last_seen_at = max(last_seen_at, ?1)
                     WHERE session_id = ?2 AND state = 'active'",
                    params![observed_at, id],
                )
                .map_err(map_write_error)?;
            if changed == 1 {
                return Ok(());
            }
            match load_session(connection, &id)? {
                None => Err(AuthorizationStateError::SessionMissing),
                Some(record) if record.state == SessionState::Revoked => {
                    Err(AuthorizationStateError::SessionRevoked)
                }
                Some(_) => Err(AuthorizationStateError::SessionExpired),
            }
        })
        .await
    }

    async fn revoke_session(
        &self,
        id: &str,
        expected_version: u64,
        revoked_at: i64,
    ) -> Result<SessionRecord, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("revokedAt", revoked_at)?;
        let id = id.to_owned();
        self.run(move |connection| {
            let next = expected_version.checked_add(1).ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("session version overflow".to_owned())
            })?;
            let changed = connection
                .execute(
                    "UPDATE auth_sessions
                     SET state = 'revoked', revoked_at = ?1, version = ?2
                     WHERE session_id = ?3 AND state = 'active' AND version = ?4",
                    params![
                        revoked_at,
                        to_sql_version(next)?,
                        id,
                        to_sql_version(expected_version)?,
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            load_session(connection, &id)?.ok_or(AuthorizationStateError::SessionMissing)
        })
        .await
    }

    async fn expire_session(
        &self,
        id: &str,
        expected_version: u64,
        expired_at: i64,
    ) -> Result<SessionRecord, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("expiredAt", expired_at)?;
        let id = id.to_owned();
        self.run(move |connection| {
            let next = expected_version.checked_add(1).ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("session version overflow".to_owned())
            })?;
            let changed = connection
                .execute(
                    "UPDATE auth_sessions
                     SET state = 'expired', version = ?1
                     WHERE session_id = ?2 AND state = 'active' AND version = ?3
                       AND (expires_at IS NULL OR expires_at <= ?4)",
                    params![
                        to_sql_version(next)?,
                        id,
                        to_sql_version(expected_version)?,
                        expired_at,
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            load_session(connection, &id)?.ok_or(AuthorizationStateError::SessionMissing)
        })
        .await
    }

    async fn rebind_session(
        &self,
        id: &str,
        expected_version: u64,
        participant: &ParticipantBindingRecord,
    ) -> Result<SessionRecord, AuthorizationStateError> {
        participant.resolve()?;
        let principal_kind = self
            .get_session(id)
            .await?
            .ok_or(AuthorizationStateError::SessionMissing)?
            .principal_kind;
        super::domain::validate_principal_participant(
            principal_kind,
            participant.participant_kind,
        )?;
        let id = id.to_owned();
        let participant = participant.clone();
        self.run(move |connection| {
            let next = expected_version.checked_add(1).ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("session version overflow".to_owned())
            })?;
            let changed = connection
                .execute(
                    "UPDATE auth_sessions SET
                        participant_id = ?1, participant_kind = ?2,
                        participant_artifact_digest = ?3, participant_needs_digest = ?4,
                        version = ?5
                     WHERE session_id = ?6 AND state = 'active' AND version = ?7",
                    params![
                        participant.participant_id,
                        encode_enum(participant.participant_kind)?,
                        participant.artifact_digest,
                        participant.needs_digest,
                        to_sql_version(next)?,
                        id,
                        to_sql_version(expected_version)?,
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            load_session(connection, &id)?.ok_or(AuthorizationStateError::SessionMissing)
        })
        .await
    }

    async fn list_sessions_for_principal(
        &self,
        principal_id: &str,
    ) -> Result<Vec<SessionRecord>, AuthorizationStateError> {
        let principal_id = principal_id.to_owned();
        self.run(move |connection| {
            let mut statement = connection
                .prepare(&format!(
                    "{} WHERE principal_id = ?1 ORDER BY session_id",
                    SESSION_SELECT
                ))
                .map_err(sql_error)?;
            let records = statement
                .query_map([principal_id], decode_session)
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            Ok(records)
        })
        .await
    }
}

#[async_trait]
impl IdentityAuthorityRepository for SqliteAuthorizationStore {
    async fn get_identity_authority(
        &self,
        principal_id: &str,
        participant_id: &str,
    ) -> Result<Option<IdentityAuthorityRecord>, AuthorizationStateError> {
        let principal_id = principal_id.to_owned();
        let participant_id = participant_id.to_owned();
        self.run(move |connection| {
            load_identity_authority(connection, &principal_id, &participant_id)
        })
        .await
    }

    async fn put_identity_authority(
        &self,
        mut record: IdentityAuthorityRecord,
        expected_version: Option<u64>,
    ) -> Result<IdentityAuthorityRecord, AuthorizationStateError> {
        validate_identity_authority(&mut record)?;
        self.run(move |connection| {
            put_identity_authority(connection, &mut record, expected_version)?;
            Ok(record)
        })
        .await
    }
}

#[async_trait]
impl DeploymentAuthorityRepository for SqliteAuthorizationStore {
    async fn get_deployment_authority(
        &self,
        deployment_id: &str,
        participant_id: &str,
    ) -> Result<Option<DeploymentAuthorityRecord>, AuthorizationStateError> {
        let deployment_id = deployment_id.to_owned();
        let participant_id = participant_id.to_owned();
        self.run(move |connection| {
            load_deployment_authority(connection, &deployment_id, &participant_id)
        })
        .await
    }

    async fn put_deployment_authority(
        &self,
        mut record: DeploymentAuthorityRecord,
        expected_version: Option<u64>,
    ) -> Result<DeploymentAuthorityRecord, AuthorizationStateError> {
        validate_deployment_authority(&mut record)?;
        self.run(move |connection| {
            put_deployment_authority(connection, &mut record, expected_version)?;
            Ok(record)
        })
        .await
    }
}

#[async_trait]
impl EvidenceRepository for SqliteAuthorizationStore {
    async fn get_deployment_evidence(
        &self,
        deployment_id: &str,
    ) -> Result<Option<DeploymentRecord>, AuthorizationStateError> {
        let deployment_id = deployment_id.to_owned();
        self.run(move |connection| load_deployment(connection, &deployment_id))
            .await
    }

    async fn put_deployment_evidence(
        &self,
        deployment: DeploymentRecord,
    ) -> Result<(), AuthorizationStateError> {
        validate_deployment_evidence(&deployment)?;
        self.run(move |connection| {
            if let Some(existing) = load_deployment(connection, &deployment.deployment_id)? {
                if existing.participant_id != deployment.participant_id
                    || existing.participant_kind != deployment.participant_kind
                {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "deployment participant identity cannot change".to_owned(),
                    ));
                }
            }
            connection
                .execute(
                    "INSERT INTO auth_deployments (
                        deployment_id, participant_id, participant_kind, state, expires_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(deployment_id) DO UPDATE SET
                        state = excluded.state,
                        expires_at = excluded.expires_at",
                    params![
                        deployment.deployment_id,
                        deployment.participant_id,
                        encode_enum(deployment.participant_kind)?,
                        if deployment.active {
                            "active"
                        } else {
                            "disabled"
                        },
                        deployment.expires_at,
                    ],
                )
                .map_err(map_write_error)?;
            Ok(())
        })
        .await
    }

    async fn get_runtime_evidence(
        &self,
        session_id: &str,
    ) -> Result<Option<RuntimeEvidence>, AuthorizationStateError> {
        let session_id = session_id.to_owned();
        self.run(move |connection| load_runtime_evidence(connection, &session_id))
            .await
    }

    async fn get_runtime_instance(
        &self,
        instance_id: &str,
    ) -> Result<Option<RuntimeInstanceRecord>, AuthorizationStateError> {
        let instance_id = instance_id.to_owned();
        self.run(move |connection| load_runtime_instance(connection, &instance_id))
            .await
    }

    async fn put_runtime_instance(
        &self,
        instance: RuntimeInstanceRecord,
    ) -> Result<(), AuthorizationStateError> {
        validate_runtime_instance(&instance)?;
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            validate_sql_runtime_instance_relationships(&transaction, &instance)?;
            if let Some(existing) = load_runtime_instance(&transaction, &instance.instance_id)? {
                if existing.deployment_id != instance.deployment_id
                    || existing.principal_id != instance.principal_id
                {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "runtime instance identity cannot change".to_owned(),
                    ));
                }
            }
            transaction
                .execute(
                    "INSERT INTO auth_instances (instance_id, deployment_id, principal_id, state)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(instance_id) DO UPDATE SET state = excluded.state",
                    params![
                        instance.instance_id,
                        instance.deployment_id,
                        instance.principal_id,
                        encode_enum(instance.state)?,
                    ],
                )
                .map_err(map_write_error)?;
            transaction.commit().map_err(sql_error)
        })
        .await
    }

    async fn get_device(
        &self,
        principal_id: &str,
        deployment_id: &str,
    ) -> Result<Option<DeviceRecord>, AuthorizationStateError> {
        let principal_id = principal_id.to_owned();
        let deployment_id = deployment_id.to_owned();
        self.run(move |connection| load_device(connection, &principal_id, &deployment_id))
            .await
    }

    async fn put_device(&self, device: DeviceRecord) -> Result<(), AuthorizationStateError> {
        validate_device(&device)?;
        self.run(move |connection| {
            validate_sql_device_relationships(connection, &device)?;
            connection
                .execute(
                    "INSERT INTO auth_devices (principal_id, deployment_id, state)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(principal_id, deployment_id) DO UPDATE SET state = excluded.state",
                    params![
                        device.principal_id,
                        device.deployment_id,
                        encode_enum(device.state)?,
                    ],
                )
                .map_err(map_write_error)?;
            Ok(())
        })
        .await
    }

    async fn get_device_delegation(
        &self,
        principal_id: &str,
        deployment_id: &str,
    ) -> Result<Option<DeviceDelegationRecord>, AuthorizationStateError> {
        let principal_id = principal_id.to_owned();
        let deployment_id = deployment_id.to_owned();
        self.run(move |connection| {
            load_device_delegation(connection, &principal_id, &deployment_id)
        })
        .await
    }

    async fn put_device_delegation(
        &self,
        delegation: DeviceDelegationRecord,
    ) -> Result<(), AuthorizationStateError> {
        validate_device_delegation(&delegation)?;
        self.run(move |connection| {
            if load_device(
                connection,
                &delegation.principal_id,
                &delegation.deployment_id,
            )?
            .is_none()
            {
                return Err(AuthorizationStateError::DeviceInactive);
            }
            connection
                .execute(
                    "INSERT INTO auth_device_delegations (
                        principal_id, deployment_id, required, state, expires_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(principal_id, deployment_id) DO UPDATE SET
                        required = excluded.required,
                        state = excluded.state,
                        expires_at = excluded.expires_at",
                    params![
                        delegation.principal_id,
                        delegation.deployment_id,
                        delegation.required,
                        encode_enum(delegation.state)?,
                        delegation.expires_at,
                    ],
                )
                .map_err(map_write_error)?;
            Ok(())
        })
        .await
    }

    async fn get_session_runtime_binding(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionRuntimeBinding>, AuthorizationStateError> {
        let session_id = session_id.to_owned();
        self.run(move |connection| load_session_runtime_binding(connection, &session_id))
            .await
    }

    async fn put_session_runtime_binding(
        &self,
        binding: SessionRuntimeBinding,
    ) -> Result<(), AuthorizationStateError> {
        validate_session_runtime_binding(&binding)?;
        self.run(move |connection| {
            validate_sql_session_runtime_binding_relationships(connection, &binding)?;
            connection
                .execute(
                    "INSERT INTO auth_session_runtime_bindings (
                        session_id, deployment_id, instance_id
                     ) VALUES (?1, ?2, ?3)
                     ON CONFLICT(session_id) DO UPDATE SET
                        deployment_id = excluded.deployment_id,
                        instance_id = excluded.instance_id",
                    params![
                        binding.session_id,
                        binding.deployment_id,
                        binding.instance_id
                    ],
                )
                .map_err(map_write_error)?;
            Ok(())
        })
        .await
    }

    async fn remove_session_runtime_binding(
        &self,
        session_id: &str,
    ) -> Result<(), AuthorizationStateError> {
        super::domain::require_nonempty("sessionId", session_id)?;
        let session_id = session_id.to_owned();
        self.run(move |connection| {
            connection
                .execute(
                    "DELETE FROM auth_session_runtime_bindings WHERE session_id = ?1",
                    [&session_id],
                )
                .map_err(map_write_error)?;
            Ok(())
        })
        .await
    }

    async fn list_dependency_evidence(
        &self,
        target: &AuthorityTarget,
    ) -> Result<Vec<DependencyEvidence>, AuthorizationStateError> {
        let target = target.clone();
        self.run(move |connection| load_dependency_evidence(connection, &target))
            .await
    }

    async fn replace_dependency_evidence(
        &self,
        scope: AuthorityEvidenceScope,
        evidence: Vec<DependencyEvidence>,
    ) -> Result<(), AuthorizationStateError> {
        validate_dependency_evidence(&evidence)?;
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            validate_sql_evidence_scope(&transaction, &scope)?;
            transaction
                .execute(
                    "DELETE FROM auth_dependency_evidence
                     WHERE authority_kind = ?1 AND authority_id = ?2",
                    params![encode_enum(scope.target.kind)?, scope.target.authority_id],
                )
                .map_err(map_write_error)?;
            for item in evidence {
                transaction
                    .execute(
                        "INSERT INTO auth_dependency_evidence (
                            authority_kind, authority_id, participant_id,
                            participant_artifact_digest, participant_needs_digest,
                            alias, required, api_id, api_digest,
                            provider_participant_id, provider_deployment_id,
                            provider_instance_id, state, observed_at
                         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                        params![
                            encode_enum(scope.target.kind)?,
                            scope.target.authority_id,
                            scope.participant_id,
                            scope.participant_artifact_digest,
                            scope.participant_needs_digest,
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
            transaction.commit().map_err(sql_error)
        })
        .await
    }

    async fn list_resource_evidence(
        &self,
        target: &AuthorityTarget,
    ) -> Result<Vec<ResourceBindingEvidence>, AuthorizationStateError> {
        let target = target.clone();
        self.run(move |connection| load_resource_evidence(connection, &target))
            .await
    }

    async fn replace_resource_evidence(
        &self,
        scope: AuthorityEvidenceScope,
        evidence: Vec<ResourceBindingEvidence>,
    ) -> Result<(), AuthorizationStateError> {
        validate_resource_evidence(&evidence)?;
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            validate_sql_evidence_scope(&transaction, &scope)?;
            transaction
                .execute(
                    "DELETE FROM auth_resource_binding_evidence
                     WHERE authority_kind = ?1 AND authority_id = ?2",
                    params![encode_enum(scope.target.kind)?, scope.target.authority_id],
                )
                .map_err(map_write_error)?;
            for item in evidence {
                transaction
                    .execute(
                        "INSERT INTO auth_resource_binding_evidence (
                            authority_kind, authority_id, participant_id,
                            participant_artifact_digest, participant_needs_digest,
                            resource_kind, local_name, binding_id,
                            provider_identity, state, materialized_at, error
                         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                        params![
                            encode_enum(scope.target.kind)?,
                            scope.target.authority_id,
                            scope.participant_id,
                            scope.participant_artifact_digest,
                            scope.participant_needs_digest,
                            item.resource_kind,
                            item.local_name,
                            item.binding_id,
                            item.provider_identity,
                            encode_enum(item.state)?,
                            item.materialized_at,
                            item.error,
                        ],
                    )
                    .map_err(map_write_error)?;
            }
            transaction.commit().map_err(sql_error)
        })
        .await
    }
}

#[async_trait]
impl AuthorizationMaterializationRepository for SqliteAuthorizationStore {
    async fn get_materialized_authority(
        &self,
        kind: AuthorityKind,
        authority_id: &str,
    ) -> Result<Option<MaterializationReplacement>, AuthorizationStateError> {
        let authority_id = authority_id.to_owned();
        self.run(move |connection| load_materialization(connection, kind, &authority_id))
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
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
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
        self.run(|connection| {
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
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            let snapshot = sqlite_issuance_snapshot(&transaction, &session_id)?;
            transaction.commit().map_err(sql_error)?;
            Ok(snapshot)
        })
        .await
    }

    async fn list_transition_outbox(
        &self,
        limit: usize,
    ) -> Result<Vec<AuthorizationTransitionOutboxRecord>, AuthorizationStateError> {
        let limit = i64::try_from(limit).map_err(|_| {
            AuthorizationStateError::InvalidRecord("outbox limit exceeds i64".to_owned())
        })?;
        self.run(move |connection| {
            let mut statement = connection
                .prepare(
                    "SELECT event_id, transition_json, created_at
                     FROM auth_transition_outbox
                     ORDER BY created_at, event_id LIMIT ?1",
                )
                .map_err(sql_error)?;
            let records = statement
                .query_map([limit], |row| {
                    Ok(AuthorizationTransitionOutboxRecord {
                        event_id: row.get(0)?,
                        transition: decode_json(row.get::<_, String>(1)?)?,
                        created_at: row.get(2)?,
                    })
                })
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            Ok(records)
        })
        .await
    }

    async fn acknowledge_transition(&self, event_id: &str) -> Result<(), AuthorizationStateError> {
        let event_id = event_id.to_owned();
        self.run(move |connection| {
            connection
                .execute(
                    "DELETE FROM auth_transition_outbox WHERE event_id = ?1",
                    [event_id],
                )
                .map_err(map_write_error)?;
            Ok(())
        })
        .await
    }
}

fn load_runtime_instance(
    connection: &Connection,
    instance_id: &str,
) -> Result<Option<RuntimeInstanceRecord>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT instance_id, deployment_id, principal_id, state
             FROM auth_instances WHERE instance_id = ?1",
            [instance_id],
            |row| {
                Ok(RuntimeInstanceRecord {
                    instance_id: row.get(0)?,
                    deployment_id: row.get(1)?,
                    principal_id: row.get(2)?,
                    state: decode_enum(row.get(3)?)?,
                })
            },
        )
        .optional()
        .map_err(sql_error)
}

fn load_device(
    connection: &Connection,
    principal_id: &str,
    deployment_id: &str,
) -> Result<Option<DeviceRecord>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT principal_id, deployment_id, state FROM auth_devices
             WHERE principal_id = ?1 AND deployment_id = ?2",
            params![principal_id, deployment_id],
            |row| {
                Ok(DeviceRecord {
                    principal_id: row.get(0)?,
                    deployment_id: row.get(1)?,
                    state: decode_enum(row.get(2)?)?,
                })
            },
        )
        .optional()
        .map_err(sql_error)
}

fn load_device_delegation(
    connection: &Connection,
    principal_id: &str,
    deployment_id: &str,
) -> Result<Option<DeviceDelegationRecord>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT principal_id, deployment_id, required, state, expires_at
             FROM auth_device_delegations
             WHERE principal_id = ?1 AND deployment_id = ?2",
            params![principal_id, deployment_id],
            |row| {
                Ok(DeviceDelegationRecord {
                    principal_id: row.get(0)?,
                    deployment_id: row.get(1)?,
                    required: row.get(2)?,
                    state: decode_enum(row.get(3)?)?,
                    expires_at: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(sql_error)
}

fn load_session_runtime_binding(
    connection: &Connection,
    session_id: &str,
) -> Result<Option<SessionRuntimeBinding>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT session_id, deployment_id, instance_id
             FROM auth_session_runtime_bindings WHERE session_id = ?1",
            [session_id],
            |row| {
                Ok(SessionRuntimeBinding {
                    session_id: row.get(0)?,
                    deployment_id: row.get(1)?,
                    instance_id: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(sql_error)
}

fn validate_sql_runtime_instance_relationships(
    connection: &Connection,
    instance: &RuntimeInstanceRecord,
) -> Result<(), AuthorizationStateError> {
    let deployment = load_deployment(connection, &instance.deployment_id)?
        .ok_or(AuthorizationStateError::DeploymentInactive)?;
    let principal = load_principal(connection, &instance.principal_id)?
        .ok_or(AuthorizationStateError::PrincipalMissing)?;
    let kind_matches = matches!(
        (principal.kind, deployment.participant_kind),
        (
            PrincipalKind::Service,
            trellis_protocol::ParticipantKindV1::Service
        ) | (
            PrincipalKind::Device,
            trellis_protocol::ParticipantKindV1::Device
        )
    );
    if !kind_matches {
        return Err(AuthorizationStateError::InvalidRecord(
            "runtime instance principal kind does not match deployment".to_owned(),
        ));
    }
    Ok(())
}

fn validate_sql_device_relationships(
    connection: &Connection,
    device: &DeviceRecord,
) -> Result<(), AuthorizationStateError> {
    let deployment = load_deployment(connection, &device.deployment_id)?
        .ok_or(AuthorizationStateError::DeploymentInactive)?;
    let principal = load_principal(connection, &device.principal_id)?
        .ok_or(AuthorizationStateError::PrincipalMissing)?;
    if principal.kind != PrincipalKind::Device
        || deployment.participant_kind != trellis_protocol::ParticipantKindV1::Device
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "device evidence requires a device principal and deployment".to_owned(),
        ));
    }
    Ok(())
}

fn validate_sql_session_runtime_binding_relationships(
    connection: &Connection,
    binding: &SessionRuntimeBinding,
) -> Result<(), AuthorizationStateError> {
    let session = load_session(connection, &binding.session_id)?
        .ok_or(AuthorizationStateError::SessionMissing)?;
    if session.principal_kind == PrincipalKind::User {
        return Err(AuthorizationStateError::InvalidRecord(
            "user sessions cannot have runtime bindings".to_owned(),
        ));
    }
    let deployment = load_deployment(connection, &binding.deployment_id)?
        .ok_or(AuthorizationStateError::DeploymentInactive)?;
    if deployment.participant_id != session.participant_id
        || deployment.participant_kind != session.participant_kind
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "session runtime deployment does not match participant".to_owned(),
        ));
    }
    let instance = load_runtime_instance(connection, &binding.instance_id)?.ok_or_else(|| {
        AuthorizationStateError::InvalidRecord(
            "session runtime binding references a missing instance".to_owned(),
        )
    })?;
    if instance.deployment_id != binding.deployment_id
        || instance.principal_id != session.principal_id
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "session runtime binding does not match instance identity".to_owned(),
        ));
    }
    Ok(())
}

fn load_runtime_evidence(
    connection: &Connection,
    session_id: &str,
) -> Result<Option<RuntimeEvidence>, AuthorizationStateError> {
    let session =
        load_session(connection, session_id)?.ok_or(AuthorizationStateError::SessionMissing)?;
    let principal = load_principal(connection, &session.principal_id)?
        .ok_or(AuthorizationStateError::PrincipalMissing)?;
    if principal.kind == PrincipalKind::User {
        return Ok(Some(RuntimeEvidence::User));
    }
    let Some(binding) = load_session_runtime_binding(connection, session_id)? else {
        return Ok(None);
    };
    let instance_id = binding.instance_id;
    let instance = load_runtime_instance(connection, &instance_id)?.ok_or_else(|| {
        AuthorizationStateError::InvalidRecord(
            "session runtime binding references a missing instance".to_owned(),
        )
    })?;
    if instance.deployment_id != binding.deployment_id
        || instance.principal_id != session.principal_id
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "session runtime binding does not match instance identity".to_owned(),
        ));
    }
    let instance_active = instance.state == RuntimeInstanceState::Active;
    match principal.kind {
        PrincipalKind::Service => Ok(Some(RuntimeEvidence::Service(ServiceEvidence {
            deployment_id: binding.deployment_id,
            instance_id,
            instance_active,
        }))),
        PrincipalKind::Device => {
            let device_active =
                load_device(connection, &principal.principal_id, &binding.deployment_id)?
                    .is_some_and(|device| device.state == DeviceState::Active);
            let delegation = load_device_delegation(
                connection,
                &principal.principal_id,
                &binding.deployment_id,
            )?
            .map(|record| DelegationEvidence {
                required: record.required,
                active: record.state == DeviceDelegationState::Active,
                expires_at: record.expires_at,
            });
            Ok(Some(RuntimeEvidence::Device(DeviceEvidence {
                deployment_id: binding.deployment_id,
                instance_id,
                device_active,
                instance_active,
                delegation,
            })))
        }
        PrincipalKind::User => Ok(Some(RuntimeEvidence::User)),
    }
}

fn load_participant_binding(
    connection: &Connection,
    participant_id: &str,
    artifact_digest: &str,
) -> Result<Option<ParticipantBindingRecord>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT participant_id, participant_kind, artifact_digest, needs_digest,
                    participant_json, api_artifacts_json, resolved_at, state, error
             FROM auth_participant_bindings
             WHERE participant_id = ?1 AND artifact_digest = ?2",
            params![participant_id, artifact_digest],
            decode_participant_binding,
        )
        .optional()
        .map_err(sql_error)
}

fn load_deployment(
    connection: &Connection,
    deployment_id: &str,
) -> Result<Option<DeploymentRecord>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT deployment_id, participant_id, participant_kind, state, expires_at
             FROM auth_deployments WHERE deployment_id = ?1",
            [deployment_id],
            |row| {
                Ok(DeploymentRecord {
                    deployment_id: row.get(0)?,
                    participant_id: row.get(1)?,
                    participant_kind: decode_enum(row.get::<_, String>(2)?)?,
                    active: row.get::<_, String>(3)? == "active",
                    expires_at: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(sql_error)
}

fn load_desired_authority(
    connection: &Connection,
    target: &AuthorityTarget,
) -> Result<Option<DesiredAuthorityRecord>, AuthorizationStateError> {
    match target.kind {
        AuthorityKind::Identity => connection
            .query_row(
                "SELECT authority_id, principal_id, participant_id,
                        participant_artifact_digest, accepted_needs_digest,
                        desired_grant_set_json, desired_capabilities_json, state,
                        version, created_at, updated_at, expires_at,
                        decision_at, decision_by, decision_reason
                 FROM auth_identity_authorities WHERE authority_id = ?1",
                [&target.authority_id],
                decode_identity_authority,
            )
            .optional()
            .map_err(sql_error)
            .map(|value| value.map(DesiredAuthorityRecord::Identity)),
        AuthorityKind::Deployment => connection
            .query_row(
                "SELECT authority_id, deployment_id, participant_id, participant_kind,
                        participant_artifact_digest, accepted_needs_digest,
                        desired_grant_set_json, desired_capabilities_json, state,
                        version, created_at, updated_at, expires_at,
                        decision_at, decision_by, decision_reason
                 FROM auth_deployment_authorities WHERE authority_id = ?1",
                [&target.authority_id],
                decode_deployment_authority,
            )
            .optional()
            .map_err(sql_error)
            .map(|value| value.map(DesiredAuthorityRecord::Deployment)),
    }
}

fn load_dependency_evidence(
    connection: &Connection,
    target: &AuthorityTarget,
) -> Result<Vec<DependencyEvidence>, AuthorizationStateError> {
    let mut statement = connection
        .prepare(
            "SELECT alias, required, api_id, api_digest, provider_participant_id,
                    provider_deployment_id, provider_instance_id, state, observed_at
             FROM auth_dependency_evidence
             WHERE authority_kind = ?1 AND authority_id = ?2
             ORDER BY required DESC, alias",
        )
        .map_err(sql_error)?;
    let records = statement
        .query_map(
            params![encode_enum(target.kind)?, target.authority_id],
            decode_dependency,
        )
        .map_err(sql_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(sql_error)?;
    Ok(records)
}

fn load_resource_evidence(
    connection: &Connection,
    target: &AuthorityTarget,
) -> Result<Vec<ResourceBindingEvidence>, AuthorizationStateError> {
    let mut statement = connection
        .prepare(
            "SELECT resource_kind, local_name, binding_id, participant_id,
                    provider_identity, state, materialized_at, error
             FROM auth_resource_binding_evidence
             WHERE authority_kind = ?1 AND authority_id = ?2
             ORDER BY resource_kind, local_name",
        )
        .map_err(sql_error)?;
    let records = statement
        .query_map(
            params![encode_enum(target.kind)?, target.authority_id],
            decode_resource,
        )
        .map_err(sql_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(sql_error)?;
    Ok(records)
}

fn load_evidence_scope(
    connection: &Connection,
    table: &str,
    target: &AuthorityTarget,
) -> Result<Option<AuthorityEvidenceScope>, AuthorizationStateError> {
    connection
        .query_row(
            &format!(
                "SELECT participant_id, participant_artifact_digest, participant_needs_digest
                 FROM {table} WHERE authority_kind = ?1 AND authority_id = ?2 LIMIT 1"
            ),
            params![encode_enum(target.kind)?, target.authority_id],
            |row| {
                Ok(AuthorityEvidenceScope {
                    target: target.clone(),
                    participant_id: row.get(0)?,
                    participant_artifact_digest: row.get(1)?,
                    participant_needs_digest: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(sql_error)
}

fn validate_sql_evidence_scope(
    connection: &Connection,
    scope: &AuthorityEvidenceScope,
) -> Result<(), AuthorizationStateError> {
    let authority = load_desired_authority(connection, &scope.target)?
        .ok_or(AuthorizationStateError::AuthorityMissing)?;
    if authority.participant_id() != scope.participant_id
        || authority.participant_artifact_digest() != scope.participant_artifact_digest
    {
        return Err(AuthorizationStateError::ParticipantDigestMismatch);
    }
    if authority.accepted_needs_digest() != scope.participant_needs_digest {
        return Err(AuthorizationStateError::NeedsDigestMismatch);
    }
    Ok(())
}

fn sqlite_materialization_snapshot(
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

fn sqlite_issuance_snapshot(
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

fn sql_snapshot_token<T: serde::Serialize>(
    value: &T,
) -> Result<AuthoritySnapshotToken, AuthorizationStateError> {
    let encoded = serde_json::to_vec(value).map_err(|error| {
        AuthorizationStateError::Storage(format!("cannot encode authority snapshot: {error}"))
    })?;
    Ok(AuthoritySnapshotToken(
        URL_SAFE_NO_PAD.encode(Sha256::digest(encoded)),
    ))
}

fn next_sql_version(field: &str, current: u64) -> Result<u64, AuthorizationStateError> {
    let next = current
        .checked_add(1)
        .ok_or_else(|| AuthorizationStateError::InvalidRecord(format!("{field} overflow")))?;
    super::domain::require_positive(field, next)?;
    Ok(next)
}

fn decode_dependency(row: &Row<'_>) -> rusqlite::Result<DependencyEvidence> {
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

fn decode_resource(row: &Row<'_>) -> rusqlite::Result<ResourceBindingEvidence> {
    Ok(ResourceBindingEvidence {
        resource_kind: row.get(0)?,
        local_name: row.get(1)?,
        binding_id: row.get(2)?,
        owner_participant_id: row.get(3)?,
        provider_identity: row.get(4)?,
        state: decode_enum(row.get::<_, String>(5)?)?,
        materialized_at: row.get(6)?,
        error: row.get(7)?,
    })
}

fn load_materialization(
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

fn decode_materialized_authority(row: &Row<'_>) -> rusqlite::Result<MaterializedAuthorityRecord> {
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

fn write_materialization(
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
                    item.provider_identity,
                    encode_enum(item.state)?,
                    item.materialized_at,
                    item.error,
                ],
            )
            .map_err(map_write_error)?;
    }
    Ok(())
}

const SESSION_SELECT: &str = "SELECT
    session_id, principal_id, principal_kind, participant_id, participant_kind,
    participant_artifact_digest, participant_needs_digest, session_public_key,
    session_key_id, inbox_prefix, state, created_at, last_seen_at, expires_at,
    revoked_at, version
    FROM auth_sessions";

fn load_principal(
    connection: &Connection,
    id: &str,
) -> Result<Option<PrincipalRecord>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT principal_id, kind, state, created_at, updated_at, version,
                    disabled_at, revoked_at
             FROM auth_principals WHERE principal_id = ?1",
            [id],
            |row| {
                Ok(PrincipalRecord {
                    principal_id: row.get(0)?,
                    kind: decode_enum(row.get::<_, String>(1)?)?,
                    state: decode_enum(row.get::<_, String>(2)?)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    version: from_sql_version(row.get(5)?)?,
                    disabled_at: row.get(6)?,
                    revoked_at: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(sql_error)
}

fn load_session(
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
}

fn decode_session(row: &Row<'_>) -> rusqlite::Result<SessionRecord> {
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

fn decode_participant_binding(row: &Row<'_>) -> rusqlite::Result<ParticipantBindingRecord> {
    Ok(ParticipantBindingRecord {
        participant_id: row.get(0)?,
        participant_kind: decode_enum(row.get::<_, String>(1)?)?,
        artifact_digest: row.get(2)?,
        needs_digest: row.get(3)?,
        participant_json: row.get(4)?,
        api_artifacts_json: row.get(5)?,
        resolved_at: row.get(6)?,
        state: decode_enum(row.get::<_, String>(7)?)?,
        error: row.get(8)?,
    })
}

fn load_identity_authority(
    connection: &Connection,
    principal_id: &str,
    participant_id: &str,
) -> Result<Option<IdentityAuthorityRecord>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT authority_id, principal_id, participant_id,
                    participant_artifact_digest, accepted_needs_digest,
                    desired_grant_set_json, desired_capabilities_json, state, version,
                    created_at, updated_at, expires_at, decision_at, decision_by,
                    decision_reason
             FROM auth_identity_authorities
             WHERE principal_id = ?1 AND participant_id = ?2",
            params![principal_id, participant_id],
            decode_identity_authority,
        )
        .optional()
        .map_err(sql_error)
}

fn decode_identity_authority(row: &Row<'_>) -> rusqlite::Result<IdentityAuthorityRecord> {
    Ok(IdentityAuthorityRecord {
        authority_id: row.get(0)?,
        principal_id: row.get(1)?,
        participant_id: row.get(2)?,
        participant_artifact_digest: row.get(3)?,
        accepted_needs_digest: row.get(4)?,
        desired_grant_set: decode_json(row.get::<_, String>(5)?)?,
        desired_capabilities: decode_json(row.get::<_, String>(6)?)?,
        state: decode_enum(row.get::<_, String>(7)?)?,
        version: from_sql_version(row.get(8)?)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        expires_at: row.get(11)?,
        decision: decode_decision(row, 12, 13, 14)?,
    })
}

fn put_identity_authority(
    connection: &Connection,
    record: &mut IdentityAuthorityRecord,
    expected_version: Option<u64>,
) -> Result<(), AuthorizationStateError> {
    let principal = load_principal(connection, &record.principal_id)?
        .ok_or(AuthorizationStateError::PrincipalMissing)?;
    if principal.kind != PrincipalKind::User {
        return Err(AuthorizationStateError::PrincipalMissing);
    }
    let binding = load_participant_binding(
        connection,
        &record.participant_id,
        &record.participant_artifact_digest,
    )?
    .ok_or(AuthorizationStateError::ParticipantMissing)?;
    super::domain::validate_principal_participant(PrincipalKind::User, binding.participant_kind)?;
    if binding.needs_digest != record.accepted_needs_digest {
        return Err(AuthorizationStateError::NeedsDigestMismatch);
    }
    if let Some(expected) = expected_version {
        let current =
            load_identity_authority(connection, &record.principal_id, &record.participant_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
        if current.version != expected || current.authority_id != record.authority_id {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if identity_enforceability_equal(&current, record) {
            record.version = expected;
        } else if record.version
            != expected.checked_add(1).ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("authority version overflow".to_owned())
            })?
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
    }
    let grant_set = encode_json(&record.desired_grant_set)?;
    let capabilities = encode_json(&record.desired_capabilities)?;
    let (decision_at, decision_by, decision_reason) = encode_decision(&record.decision);
    match expected_version {
        None if record.version == 1 => {
            connection
                .execute(
                    "INSERT INTO auth_identity_authorities (
                    authority_id, principal_id, participant_id, participant_artifact_digest,
                    accepted_needs_digest, desired_grant_set_json,
                    desired_capabilities_json, state, version, created_at, updated_at,
                    expires_at, decision_at, decision_by, decision_reason
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                    params![
                        record.authority_id,
                        record.principal_id,
                        record.participant_id,
                        record.participant_artifact_digest,
                        record.accepted_needs_digest,
                        grant_set,
                        capabilities,
                        encode_enum(record.state)?,
                        to_sql_version(record.version)?,
                        record.created_at,
                        record.updated_at,
                        record.expires_at,
                        decision_at,
                        decision_by,
                        decision_reason,
                    ],
                )
                .map_err(map_write_error)?;
        }
        Some(expected) => {
            let changed = connection
                .execute(
                    "UPDATE auth_identity_authorities SET
                    participant_artifact_digest = ?1, accepted_needs_digest = ?2,
                    desired_grant_set_json = ?3, desired_capabilities_json = ?4,
                    state = ?5, version = ?6, updated_at = ?7, expires_at = ?8,
                    decision_at = ?9, decision_by = ?10, decision_reason = ?11
                 WHERE principal_id = ?12 AND participant_id = ?13
                   AND authority_id = ?14 AND version = ?15",
                    params![
                        record.participant_artifact_digest,
                        record.accepted_needs_digest,
                        grant_set,
                        capabilities,
                        encode_enum(record.state)?,
                        to_sql_version(record.version)?,
                        record.updated_at,
                        record.expires_at,
                        decision_at,
                        decision_by,
                        decision_reason,
                        record.principal_id,
                        record.participant_id,
                        record.authority_id,
                        to_sql_version(expected)?,
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
        }
        _ => return Err(AuthorizationStateError::StorageConflict),
    }
    Ok(())
}

fn load_deployment_authority(
    connection: &Connection,
    deployment_id: &str,
    participant_id: &str,
) -> Result<Option<DeploymentAuthorityRecord>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT authority_id, deployment_id, participant_id, participant_kind,
                    participant_artifact_digest, accepted_needs_digest,
                    desired_grant_set_json, desired_capabilities_json, state, version,
                    created_at, updated_at, expires_at, decision_at, decision_by,
                    decision_reason
             FROM auth_deployment_authorities
             WHERE deployment_id = ?1 AND participant_id = ?2",
            params![deployment_id, participant_id],
            decode_deployment_authority,
        )
        .optional()
        .map_err(sql_error)
}

fn decode_deployment_authority(row: &Row<'_>) -> rusqlite::Result<DeploymentAuthorityRecord> {
    Ok(DeploymentAuthorityRecord {
        authority_id: row.get(0)?,
        deployment_id: row.get(1)?,
        participant_id: row.get(2)?,
        participant_kind: decode_enum(row.get::<_, String>(3)?)?,
        participant_artifact_digest: row.get(4)?,
        accepted_needs_digest: row.get(5)?,
        desired_grant_set: decode_json(row.get::<_, String>(6)?)?,
        desired_capabilities: decode_json(row.get::<_, String>(7)?)?,
        state: decode_enum(row.get::<_, String>(8)?)?,
        version: from_sql_version(row.get(9)?)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        expires_at: row.get(12)?,
        decision: decode_decision(row, 13, 14, 15)?,
    })
}

fn put_deployment_authority(
    connection: &Connection,
    record: &mut DeploymentAuthorityRecord,
    expected_version: Option<u64>,
) -> Result<(), AuthorizationStateError> {
    let binding = load_participant_binding(
        connection,
        &record.participant_id,
        &record.participant_artifact_digest,
    )?
    .ok_or(AuthorizationStateError::ParticipantMissing)?;
    if binding.participant_kind != record.participant_kind {
        return Err(AuthorizationStateError::InvalidRecord(
            "deployment authority participant kind does not match binding".to_owned(),
        ));
    }
    if binding.needs_digest != record.accepted_needs_digest {
        return Err(AuthorizationStateError::NeedsDigestMismatch);
    }
    if let Some(expected) = expected_version {
        let current =
            load_deployment_authority(connection, &record.deployment_id, &record.participant_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
        if current.version != expected || current.authority_id != record.authority_id {
            return Err(AuthorizationStateError::StorageConflict);
        }
        if deployment_enforceability_equal(&current, record) {
            record.version = expected;
        } else if record.version
            != expected.checked_add(1).ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("authority version overflow".to_owned())
            })?
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
    }
    let grant_set = encode_json(&record.desired_grant_set)?;
    let capabilities = encode_json(&record.desired_capabilities)?;
    let (decision_at, decision_by, decision_reason) = encode_decision(&record.decision);
    match expected_version {
        None if record.version == 1 => {
            connection
                .execute(
                    "INSERT INTO auth_deployment_authorities (
                    authority_id, deployment_id, participant_id, participant_kind,
                    participant_artifact_digest, accepted_needs_digest,
                    desired_grant_set_json, desired_capabilities_json, state, version,
                    created_at, updated_at, expires_at, decision_at, decision_by,
                    decision_reason
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                    params![
                        record.authority_id,
                        record.deployment_id,
                        record.participant_id,
                        encode_enum(record.participant_kind)?,
                        record.participant_artifact_digest,
                        record.accepted_needs_digest,
                        grant_set,
                        capabilities,
                        encode_enum(record.state)?,
                        to_sql_version(record.version)?,
                        record.created_at,
                        record.updated_at,
                        record.expires_at,
                        decision_at,
                        decision_by,
                        decision_reason,
                    ],
                )
                .map_err(map_write_error)?;
        }
        Some(expected) => {
            let changed = connection
                .execute(
                    "UPDATE auth_deployment_authorities SET
                    participant_kind = ?1, participant_artifact_digest = ?2,
                    accepted_needs_digest = ?3, desired_grant_set_json = ?4,
                    desired_capabilities_json = ?5, state = ?6, version = ?7,
                    updated_at = ?8, expires_at = ?9, decision_at = ?10,
                    decision_by = ?11, decision_reason = ?12
                 WHERE deployment_id = ?13 AND participant_id = ?14
                   AND authority_id = ?15 AND version = ?16",
                    params![
                        encode_enum(record.participant_kind)?,
                        record.participant_artifact_digest,
                        record.accepted_needs_digest,
                        grant_set,
                        capabilities,
                        encode_enum(record.state)?,
                        to_sql_version(record.version)?,
                        record.updated_at,
                        record.expires_at,
                        decision_at,
                        decision_by,
                        decision_reason,
                        record.deployment_id,
                        record.participant_id,
                        record.authority_id,
                        to_sql_version(expected)?,
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
        }
        _ => return Err(AuthorizationStateError::StorageConflict),
    }
    Ok(())
}

fn encode_decision(
    decision: &Option<AuthorityDecision>,
) -> (Option<i64>, Option<&str>, Option<&str>) {
    decision.as_ref().map_or((None, None, None), |value| {
        (
            Some(value.decided_at),
            Some(value.decided_by.as_str()),
            value.reason.as_deref(),
        )
    })
}

fn decode_decision(
    row: &Row<'_>,
    at: usize,
    by: usize,
    reason: usize,
) -> rusqlite::Result<Option<AuthorityDecision>> {
    let decided_at: Option<i64> = row.get(at)?;
    let decided_by: Option<String> = row.get(by)?;
    match (decided_at, decided_by) {
        (None, None) => Ok(None),
        (Some(decided_at), Some(decided_by)) => Ok(Some(AuthorityDecision {
            decided_at,
            decided_by,
            reason: row.get(reason)?,
        })),
        _ => Err(decode_failure("paired authority decision columns disagree")),
    }
}

fn encode_enum<T: serde::Serialize>(value: T) -> Result<String, AuthorizationStateError> {
    let encoded = serde_json::to_string(&value)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
    Ok(encoded.trim_matches('"').to_owned())
}

fn decode_enum<T: serde::de::DeserializeOwned>(value: String) -> rusqlite::Result<T> {
    serde_json::from_str(&format!("\"{value}\""))
        .map_err(|error| decode_failure(&error.to_string()))
}

fn encode_json<T: serde::Serialize>(value: &T) -> Result<String, AuthorizationStateError> {
    serde_json::to_string(value)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))
}

fn decode_json<T: serde::de::DeserializeOwned>(value: String) -> rusqlite::Result<T> {
    serde_json::from_str(&value).map_err(|error| decode_failure(&error.to_string()))
}

fn to_sql_version(value: u64) -> Result<i64, AuthorizationStateError> {
    super::domain::require_positive("version", value)?;
    i64::try_from(value).map_err(|_| {
        AuthorizationStateError::InvalidRecord("version exceeds SQLite integer range".to_owned())
    })
}

fn from_sql_version(value: i64) -> rusqlite::Result<u64> {
    let value = u64::try_from(value).map_err(|_| decode_failure("version must be positive"))?;
    if value == 0 || value > super::MAX_PROTOCOL_INTEGER {
        return Err(decode_failure(
            "version exceeds protocol-safe integer range",
        ));
    }
    Ok(value)
}

fn decode_failure(message: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message.to_owned(),
        )),
    )
}

fn map_write_error(error: rusqlite::Error) -> AuthorizationStateError {
    if matches!(
        error.sqlite_error_code(),
        Some(rusqlite::ErrorCode::ConstraintViolation)
    ) {
        AuthorizationStateError::StorageConflict
    } else {
        sql_error(error)
    }
}

fn sql_error(error: rusqlite::Error) -> AuthorizationStateError {
    AuthorizationStateError::Storage(error.to_string())
}
