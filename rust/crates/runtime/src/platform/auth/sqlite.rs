use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::companion_repository::{
    local_login_attempt_result, next_version, post_commit_action_identity_equal,
    user_account_replacement, validate_account_flow, validate_account_list,
    validate_activation_decision, validate_activation_decision_changes, validate_activation_review,
    validate_authority_decision as validate_proposal_decision, validate_authority_proposal,
    validate_deployment_profile, validate_first_admin_authority, validate_idempotency_result,
    validate_login_portal, validate_new_user_account, validate_portal_route,
    validate_post_commit_action, validate_proposal_desired_authority,
    validate_provisioned_identity, validate_provisioning_aggregate, validate_provisioning_secret,
    validate_replacement_credential, validate_session_desired_authority,
    validate_session_revocation_actions, AccountCreation, AccountFlowCreation,
    ActivationReviewCreation, ActivationReviewDecision, AuthorityProposalCreation,
    AuthorityProposalDecision, ClientBootstrapAdmission, DeploymentProfileCreation,
    DeploymentProfileMutation, DeploymentProfileRepository, DeviceProvisioning,
    DeviceProvisioningSecretConsumption, FirstAdminCompletion, IdempotentOutcome,
    IdentityLinkCompletion, LocalLoginAttempt, LoginPortalMutation, PasswordResetCompletion,
    PortalRouteMutation, PortalRouteRemoval, ProvisionedInstanceMutation,
    ServiceIdentityProvisioning, SessionCreation, SessionRevocation, UserAccountMutation,
};
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
    AccountFlowRecord, AccountFlowRepository, AccountFlowState, AccountRepository,
    AuthSessionRepository, AuthorityDecision, AuthorityDecisionOutcome, AuthorityDecisionRecord,
    AuthorityEvidenceScope, AuthorityKind, AuthorityProposalRecord, AuthorityProposalRepository,
    AuthorityProposalState, AuthorityTarget, AuthorizationMaterializationRepository,
    AuthorizationStateError, AuthorizationTransitionOutboxRecord, DelegationEvidence,
    DependencyEvidence, DeploymentAuthorityRecord, DeploymentAuthorityRepository,
    DeploymentProfileRecord, DeploymentProfileState, DeploymentRecord, DesiredAuthorityRecord,
    DeviceActivationReviewRecord, DeviceActivationReviewState, DeviceDelegationRecord,
    DeviceDelegationState, DeviceEvidence, DeviceProvisioningSecretRecord, DeviceRecord,
    DeviceState, EvidenceRepository, IdempotencyRepository, IdempotencyResultRecord,
    IdentityAuthorityRecord, IdentityAuthorityRepository, LocalCredentialRecord, LoginPortalRecord,
    LoginPortalRepository, LoginSettingsRecord, MaterializationReplacement,
    MaterializedAuthorityRecord, ParticipantBindingRecord, ParticipantBindingRepository,
    PortalRouteRecord, PostCommitActionRecord, PostCommitActionRepository,
    PrincipalAuthorizationChange, PrincipalKind, PrincipalRecord, PrincipalRepository,
    PrincipalState, ProviderIdentityLink, ProviderIdentityRepository, ProvisionedIdentityKind,
    ProvisionedIdentityRecord, ProvisioningRepository, ProvisioningSecretState,
    ResourceBindingEvidence, RuntimeEvidence, RuntimeInstanceRecord, RuntimeInstanceState,
    ServiceEvidence, SessionRecord, SessionRepository, SessionRuntimeBinding, SessionState,
    UserProfileRecord,
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
        migrate_test_schema(&connection)?;
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
        migrate_test_schema(&connection)?;
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

fn migrate_test_schema(connection: &Connection) -> Result<(), AuthorizationStateError> {
    for (table, migration) in [
        (
            "auth_principals",
            include_str!("../../storage/sqlite/platform/V1001__authorization_state.sql"),
        ),
        (
            "auth_user_profiles",
            include_str!("../../storage/sqlite/platform/V1002__auth_service_cutover.sql"),
        ),
    ] {
        let migrated = connection
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sqlite_master
                    WHERE type = 'table' AND name = ?1
                 )",
                [table],
                |row| row.get::<_, bool>(0),
            )
            .map_err(sql_error)?;
        if !migrated {
            connection.execute_batch(migration).map_err(sql_error)?;
        }
    }
    Ok(())
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

    async fn list_provider_identities(
        &self,
        principal_id: &str,
    ) -> Result<Vec<ProviderIdentityLink>, AuthorizationStateError> {
        let principal_id = principal_id.to_owned();
        self.run(move |connection| {
            let mut statement = connection
                .prepare(
                    "SELECT provider, provider_subject, principal_id, linked_at, last_seen_at
                     FROM auth_provider_identities WHERE principal_id = ?1
                     ORDER BY provider, provider_subject",
                )
                .map_err(sql_error)?;
            let identities = statement
                .query_map([principal_id], |row| {
                    Ok(ProviderIdentityLink {
                        provider: row.get(0)?,
                        provider_subject: row.get(1)?,
                        principal_id: row.get(2)?,
                        linked_at: row.get(3)?,
                        last_seen_at: row.get(4)?,
                    })
                })
                .map_err(sql_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(sql_error)?;
            Ok(identities)
        })
        .await
    }

    async fn unlink_provider_identity(
        &self,
        command: super::companion_repository::ProviderIdentityUnlink,
    ) -> Result<IdempotentOutcome<bool>, AuthorizationStateError> {
        if command.provider == "local" {
            return Err(AuthorizationStateError::InvalidRecord(
                "local credentials cannot be unlinked".to_owned(),
            ));
        }
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(value) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(value));
            }
            let owner = transaction
                .query_row(
                    "SELECT principal_id FROM auth_provider_identities
                     WHERE provider = ?1 AND provider_subject = ?2",
                    params![command.provider, command.provider_subject],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(sql_error)?
                .filter(|owner| owner == &command.principal_id)
                .ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord("identity not found".to_owned())
                })?;
            let method_count: i64 = transaction
                .query_row(
                    "SELECT
                       (SELECT COUNT(*) FROM auth_local_credentials WHERE principal_id = ?1) +
                       (SELECT COUNT(*) FROM auth_provider_identities WHERE principal_id = ?1
                        AND NOT (provider = ?2 AND provider_subject = ?3))",
                    params![owner, command.provider, command.provider_subject],
                    |row| row.get(0),
                )
                .map_err(sql_error)?;
            if method_count == 0 {
                return Err(AuthorizationStateError::InvalidRecord(
                    "the last authentication method cannot be unlinked".to_owned(),
                ));
            }
            let changed = transaction
                .execute(
                    "DELETE FROM auth_provider_identities
                     WHERE provider = ?1 AND provider_subject = ?2 AND principal_id = ?3",
                    params![
                        command.provider,
                        command.provider_subject,
                        command.principal_id
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(true))
        })
        .await
    }
}

#[async_trait]
impl AccountRepository for SqliteAuthorizationStore {
    async fn create_user_account(
        &self,
        command: AccountCreation,
    ) -> Result<IdempotentOutcome<UserProfileRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_new_user_account(
                &command.principal,
                &command.profile,
                command.credential.as_ref(),
                command.identity.as_ref(),
            )?;
            insert_sql_user_account(
                &transaction,
                &command.principal,
                &command.profile,
                command.credential.as_ref(),
                command.identity.as_ref(),
            )?;
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.profile))
        })
        .await
    }

    async fn get_user_account(
        &self,
        principal_id: &str,
    ) -> Result<Option<(PrincipalRecord, UserProfileRecord)>, AuthorizationStateError> {
        let principal_id = principal_id.to_owned();
        self.run(move |connection| load_user_account(connection, &principal_id))
            .await
    }

    async fn list_user_accounts(
        &self,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(PrincipalRecord, UserProfileRecord)>, AuthorizationStateError> {
        validate_account_list(cursor, limit)?;
        let cursor = cursor.map(str::to_owned);
        self.run(move |connection| {
            let mut statement = connection
                .prepare(
                    "SELECT p.principal_id, p.kind, p.state, p.created_at, p.updated_at,
                            p.version, p.disabled_at, p.revoked_at,
                            u.principal_id, u.display_name, u.email, u.image_url,
                            u.created_at, u.updated_at, u.version
                     FROM auth_principals p
                     JOIN auth_user_profiles u ON u.principal_id = p.principal_id
                     WHERE p.kind = ?1 AND (?2 IS NULL OR p.principal_id > ?2)
                     ORDER BY p.principal_id ASC LIMIT ?3",
                )
                .map_err(sql_error)?;
            let accounts = statement
                .query_map(
                    params![
                        encode_enum(PrincipalKind::User)?,
                        cursor,
                        i64::try_from(limit).map_err(|_| {
                            AuthorizationStateError::InvalidRecord(
                                "user account list limit is invalid".to_owned(),
                            )
                        })?
                    ],
                    decode_user_account,
                )
                .map_err(sql_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(sql_error)?;
            Ok(accounts)
        })
        .await
    }

    async fn update_user_account(
        &self,
        command: UserAccountMutation,
    ) -> Result<IdempotentOutcome<(PrincipalRecord, UserProfileRecord)>, AuthorizationStateError>
    {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            let current_principal = load_principal(&transaction, &command.principal.principal_id)?
                .ok_or(AuthorizationStateError::PrincipalMissing)?;
            let current_profile = load_user_profile(&transaction, &command.principal.principal_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            let (principal, profile) = user_account_replacement(
                &current_principal,
                &current_profile,
                command.principal,
                command.profile,
                command.expected_version,
            )?;
            let principal_changed = transaction
                .execute(
                    "UPDATE auth_principals SET state = ?1, updated_at = ?2, version = ?3,
                            disabled_at = ?4, revoked_at = NULL
                     WHERE principal_id = ?5 AND version = ?6",
                    params![
                        encode_enum(principal.state)?,
                        principal.updated_at,
                        to_sql_version(principal.version)?,
                        principal.disabled_at,
                        principal.principal_id,
                        to_sql_version(command.expected_version)?,
                    ],
                )
                .map_err(map_write_error)?;
            let profile_changed = transaction
                .execute(
                    "UPDATE auth_user_profiles SET display_name = ?1, email = ?2, image_url = ?3,
                            updated_at = ?4, version = ?5
                     WHERE principal_id = ?6 AND version = ?7",
                    params![
                        profile.display_name,
                        profile.email,
                        profile.image_url,
                        profile.updated_at,
                        to_sql_version(profile.version)?,
                        profile.principal_id,
                        to_sql_version(command.expected_version)?,
                    ],
                )
                .map_err(map_write_error)?;
            if principal_changed != 1 || profile_changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied((principal, profile)))
        })
        .await
    }

    async fn get_user_profile(
        &self,
        principal_id: &str,
    ) -> Result<Option<UserProfileRecord>, AuthorizationStateError> {
        let principal_id = principal_id.to_owned();
        self.run(move |connection| load_user_profile(connection, &principal_id))
            .await
    }

    async fn get_local_credential(
        &self,
        principal_id: &str,
    ) -> Result<Option<LocalCredentialRecord>, AuthorizationStateError> {
        let principal_id = principal_id.to_owned();
        self.run(move |connection| load_local_credential(connection, &principal_id))
            .await
    }

    async fn change_password(
        &self,
        mut command: super::companion_repository::PasswordChange,
    ) -> Result<IdempotentOutcome<usize>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(value) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(value));
            }
            let current =
                load_local_credential(&transaction, &command.principal_id)?.ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord("local credential not found".to_owned())
                })?;
            super::companion_repository::validate_local_credential(&command.credential)?;
            if current.version != command.expected_version
                || command.credential.principal_id != command.principal_id
                || command.credential.version != next_version(command.expected_version)?
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let changed = transaction
                .execute(
                    "UPDATE auth_local_credentials SET password_hash = ?1, hash_profile = ?2,
                     failed_attempts = ?3, locked_until = ?4, password_changed_at = ?5,
                     updated_at = ?6, version = ?7 WHERE principal_id = ?8 AND version = ?9",
                    params![
                        command.credential.password_hash,
                        command.credential.hash_profile,
                        command.credential.failed_attempts,
                        command.credential.locked_until,
                        command.credential.password_changed_at,
                        command.credential.updated_at,
                        to_sql_version(command.credential.version)?,
                        command.principal_id,
                        to_sql_version(command.expected_version)?,
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let revoked = transaction
                .execute(
                    "UPDATE auth_sessions SET state = 'revoked', revoked_at = ?1,
                     last_seen_at = ?1, version = version + 1
                     WHERE principal_id = ?2 AND session_id <> ?3 AND state = 'active'",
                    params![
                        command.changed_at,
                        command.principal_id,
                        command.current_session_id
                    ],
                )
                .map_err(map_write_error)?;
            command.idempotency.result = json!({
                "changedAt": command.changed_at,
                "revokedSessionCount": revoked,
            });
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(revoked))
        })
        .await
    }

    async fn get_local_credential_by_username(
        &self,
        normalized_username: &str,
    ) -> Result<Option<LocalCredentialRecord>, AuthorizationStateError> {
        let normalized_username = normalized_username.to_owned();
        self.run(move |connection| {
            connection
                .query_row(
                    "SELECT principal_id, normalized_username, password_hash, hash_profile, failed_attempts, locked_until, password_changed_at, updated_at, version
                     FROM auth_local_credentials WHERE normalized_username = ?1",
                    [normalized_username],
                    decode_local_credential,
                )
                .optional()
                .map_err(sql_error)
        })
        .await
    }

    async fn record_local_login_attempt(
        &self,
        attempt: LocalLoginAttempt,
    ) -> Result<LocalCredentialRecord, AuthorizationStateError> {
        let principal_id = attempt.principal_id.clone();
        self.run(move |connection| {
            let current = load_local_credential(connection, &principal_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            let next = local_login_attempt_result(&current, &attempt)?;
            let changed = connection
                .execute(
                    "UPDATE auth_local_credentials SET failed_attempts = ?1, locked_until = ?2,
                        updated_at = ?3, version = ?4
                 WHERE principal_id = ?5 AND version = ?6",
                    params![
                        i64::from(next.failed_attempts),
                        next.locked_until,
                        next.updated_at,
                        to_sql_version(next.version)?,
                        next.principal_id,
                        to_sql_version(current.version)?,
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            Ok(next)
        })
        .await
    }

    async fn has_active_administrator(&self, now: i64) -> Result<bool, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("now", now)?;
        self.run(move |connection| sql_has_active_administrator(connection, now))
            .await
    }

    async fn replace_first_admin_flow(
        &self,
        flow: AccountFlowRecord,
        now: i64,
        rotate: bool,
    ) -> Result<Option<AccountFlowRecord>, AuthorizationStateError> {
        validate_account_flow(&flow)?;
        super::domain::require_protocol_timestamp("now", now)?;
        if flow.kind != super::AccountFlowKind::FirstAdmin
            || flow.created_at > now
            || flow.expires_at <= now
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "replacement first-admin flow must be pending and unexpired".to_owned(),
            ));
        }
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if sql_has_active_administrator(&transaction, now)? {
                return Ok(None);
            }
            let existing = transaction
                .query_row(
                    "SELECT flow_id, kind, token_hash, target_principal_id, target_provider_id,
                            return_location, payload_json, state, created_at, expires_at,
                            consumed_at, version
                     FROM auth_account_flows
                     WHERE kind = 'first_admin' AND state = 'pending' AND expires_at > ?1
                     ORDER BY created_at, flow_id LIMIT 1",
                    [now],
                    decode_account_flow,
                )
                .optional()
                .map_err(sql_error)?;
            if let Some(existing) = existing {
                if !rotate {
                    return Ok(Some(existing));
                }
                if existing.version >= super::MAX_PROTOCOL_INTEGER {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "first-admin flow version overflow".to_owned(),
                    ));
                }
                transaction
                    .execute(
                        "UPDATE auth_account_flows SET state = 'revoked', version = version + 1
                         WHERE flow_id = ?1 AND version = ?2",
                        params![existing.flow_id, existing.version],
                    )
                    .map_err(map_write_error)?;
            }
            transaction
                .execute(
                    "UPDATE auth_account_flows SET state = 'expired', version = version + 1
                     WHERE kind = 'first_admin' AND state = 'pending' AND expires_at <= ?1",
                    [now],
                )
                .map_err(map_write_error)?;
            insert_account_flow(&transaction, &flow)?;
            transaction.commit().map_err(sql_error)?;
            Ok(Some(flow))
        })
        .await
    }
}

#[async_trait]
impl DeploymentProfileRepository for SqliteAuthorizationStore {
    async fn create_deployment_profile(
        &self,
        command: DeploymentProfileCreation,
    ) -> Result<IdempotentOutcome<DeploymentProfileRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_principal(&command.principal)?;
            validate_deployment_profile(&command.profile)?;
            if command.principal.principal_id != command.profile.deployment_id
                || command.principal.kind != command.profile.kind
                || command.principal.version != 1
                || command.profile.version != 1
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            insert_sql_principal(&transaction, &command.principal)?;
            insert_deployment_profile(&transaction, &command.profile)?;
            upsert_deployment_profile_evidence(&transaction, &command.profile)?;
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.profile))
        })
        .await
    }

    async fn get_deployment_profile(
        &self,
        deployment_id: &str,
    ) -> Result<Option<DeploymentProfileRecord>, AuthorizationStateError> {
        let deployment_id = deployment_id.to_owned();
        self.run(move |connection| load_deployment_profile(connection, &deployment_id))
            .await
    }

    async fn list_deployment_profiles(
        &self,
    ) -> Result<Vec<DeploymentProfileRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let mut statement = connection
                .prepare(
                    "SELECT deployment_id, kind, display_name, participant_id, portal_id,
                            requires_device_delegation, expires_at, state, created_at,
                            updated_at, version
                     FROM auth_deployment_profiles ORDER BY deployment_id",
                )
                .map_err(sql_error)?;
            let records = statement
                .query_map([], decode_deployment_profile)
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            Ok(records)
        })
        .await
    }

    async fn put_deployment_profile(
        &self,
        command: DeploymentProfileMutation,
    ) -> Result<IdempotentOutcome<DeploymentProfileRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_deployment_profile(&command.profile)?;
            let current = load_deployment_profile(&transaction, &command.profile.deployment_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            if current.version != command.expected_version
                || command.profile.version != next_version(command.expected_version)?
                || current.created_at != command.profile.created_at
                || current.kind != command.profile.kind
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let changed = transaction
                .execute(
                    "UPDATE auth_deployment_profiles
                     SET display_name = ?1, participant_id = ?2, portal_id = ?3,
                         requires_device_delegation = ?4, expires_at = ?5, state = ?6,
                         updated_at = ?7, version = ?8
                     WHERE deployment_id = ?9 AND version = ?10",
                    params![
                        command.profile.display_name,
                        command.profile.participant_id,
                        command.profile.portal_id,
                        command.profile.requires_device_delegation,
                        command.profile.expires_at,
                        encode_enum(command.profile.state)?,
                        command.profile.updated_at,
                        to_sql_version(command.profile.version)?,
                        command.profile.deployment_id,
                        to_sql_version(command.expected_version)?,
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let principal_state = match command.profile.state {
                DeploymentProfileState::Active => PrincipalState::Active,
                DeploymentProfileState::Disabled => PrincipalState::Disabled,
                DeploymentProfileState::Removed => PrincipalState::Revoked,
            };
            transaction
                .execute(
                    "UPDATE auth_principals SET state = ?1, updated_at = ?2,
                         disabled_at = ?3, revoked_at = ?4, version = ?5
                     WHERE principal_id = ?6 AND version = ?7",
                    params![
                        encode_enum(principal_state)?,
                        command.profile.updated_at,
                        (principal_state == PrincipalState::Disabled)
                            .then_some(command.profile.updated_at),
                        (principal_state == PrincipalState::Revoked)
                            .then_some(command.profile.updated_at),
                        to_sql_version(command.profile.version)?,
                        command.profile.deployment_id,
                        to_sql_version(command.expected_version)?,
                    ],
                )
                .map_err(map_write_error)?;
            transaction
                .execute(
                    "UPDATE auth_deployments SET state = ?1 WHERE deployment_id = ?2",
                    params![
                        match command.profile.state {
                            DeploymentProfileState::Active => "active",
                            DeploymentProfileState::Disabled => "disabled",
                            DeploymentProfileState::Removed => "revoked",
                        },
                        command.profile.deployment_id,
                    ],
                )
                .map_err(map_write_error)?;
            upsert_deployment_profile_evidence(&transaction, &command.profile)?;
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.profile))
        })
        .await
    }
}

#[async_trait]
impl LoginPortalRepository for SqliteAuthorizationStore {
    async fn list_login_portals(&self) -> Result<Vec<LoginPortalRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let mut statement = connection
                .prepare("SELECT portal_id FROM auth_login_portals ORDER BY portal_id")
                .map_err(sql_error)?;
            let ids = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            ids.into_iter()
                .map(|portal_id| {
                    load_login_portal(connection, &portal_id)?
                        .map(|value| value.0)
                        .ok_or(AuthorizationStateError::StorageConflict)
                })
                .collect()
        })
        .await
    }

    async fn get_login_portal(
        &self,
        portal_id: &str,
    ) -> Result<Option<(LoginPortalRecord, LoginSettingsRecord)>, AuthorizationStateError> {
        let portal_id = portal_id.to_owned();
        self.run(move |connection| load_login_portal(connection, &portal_id))
            .await
    }

    async fn put_login_portal(
        &self,
        command: LoginPortalMutation,
    ) -> Result<IdempotentOutcome<(LoginPortalRecord, LoginSettingsRecord)>, AuthorizationStateError>
    {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_login_portal(&command.portal, &command.settings)?;
            match (load_login_portal(&transaction, &command.portal.portal_id)?, command.expected_version) {
                (None, None) if command.portal.version == 1 && command.settings.version == 1 => {
                    transaction.execute(
                        "INSERT INTO auth_login_portals (portal_id, display_name, entry_url, builtin, disabled, removed, local_registration_enabled, provider_ids_json, created_at, updated_at, version)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![command.portal.portal_id, command.portal.display_name, command.portal.entry_url, command.portal.builtin, command.portal.disabled, command.portal.removed, command.portal.local_registration_enabled, encode_json(&command.portal.provider_ids)?, command.portal.created_at, command.portal.updated_at, to_sql_version(command.portal.version)?],
                    ).map_err(map_write_error)?;
                    transaction.execute(
                        "INSERT INTO auth_login_settings (portal_id, default_provider_id, local_login_enabled, federated_registration_enabled, provider_selection_enabled, updated_at, version)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![command.settings.portal_id, command.settings.default_provider_id, command.settings.local_login_enabled, command.settings.federated_registration_enabled, command.settings.provider_selection_enabled, command.settings.updated_at, to_sql_version(command.settings.version)?],
                    ).map_err(map_write_error)?;
                }
                (Some((current, current_settings)), Some(expected))
                    if current.version == expected && current_settings.version == expected
                        && current.builtin == command.portal.builtin && current.created_at == command.portal.created_at
                        && command.portal.version == next_version(expected)? && command.settings.version == command.portal.version => {
                    let changed = transaction.execute(
                        "UPDATE auth_login_portals SET display_name = ?1, entry_url = ?2, disabled = ?3, removed = ?4, local_registration_enabled = ?5, provider_ids_json = ?6, updated_at = ?7, version = ?8
                         WHERE portal_id = ?9 AND version = ?10 AND builtin = ?11 AND created_at = ?12",
                        params![command.portal.display_name, command.portal.entry_url, command.portal.disabled, command.portal.removed, command.portal.local_registration_enabled, encode_json(&command.portal.provider_ids)?, command.portal.updated_at, to_sql_version(command.portal.version)?, command.portal.portal_id, to_sql_version(expected)?, command.portal.builtin, command.portal.created_at],
                    ).map_err(map_write_error)?;
                    let settings_changed = transaction.execute(
                        "UPDATE auth_login_settings SET default_provider_id = ?1, local_login_enabled = ?2, federated_registration_enabled = ?3, provider_selection_enabled = ?4, updated_at = ?5, version = ?6
                         WHERE portal_id = ?7 AND version = ?8",
                        params![command.settings.default_provider_id, command.settings.local_login_enabled, command.settings.federated_registration_enabled, command.settings.provider_selection_enabled, command.settings.updated_at, to_sql_version(command.settings.version)?, command.settings.portal_id, to_sql_version(expected)?],
                    ).map_err(map_write_error)?;
                    if changed != 1 || settings_changed != 1 { return Err(AuthorizationStateError::StorageConflict); }
                }
                _ => return Err(AuthorizationStateError::StorageConflict),
            }
            insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &command.actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied((command.portal, command.settings)))
        }).await
    }

    async fn put_portal_route(
        &self,
        command: PortalRouteMutation,
    ) -> Result<IdempotentOutcome<PortalRouteRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_portal_route(&command.route)?;
            if load_login_portal(&transaction, &command.route.portal_id)?.is_none() {
                return Err(AuthorizationStateError::InvalidRecord("portal route relationships do not exist".to_owned()));
            }
            if let Some(deployment_id) = &command.route.deployment_id {
                if load_deployment(&transaction, deployment_id)?.is_none() {
                    return Err(AuthorizationStateError::InvalidRecord("portal route relationships do not exist".to_owned()));
                }
            }
            let current = load_portal_route(&transaction, &command.route.route_id)?;
            match (current, command.expected_version) {
                (None, None) if command.route.version == 1 => {
                    transaction.execute(
                        "INSERT INTO auth_portal_routes (route_id, portal_id, participant_id, origin, deployment_id, priority, created_at, updated_at, version)
                          VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        params![command.route.route_id, command.route.portal_id, command.route.participant_id, command.route.origin, command.route.deployment_id, command.route.priority, command.route.created_at, command.route.updated_at, to_sql_version(command.route.version)?],
                    ).map_err(map_write_error)?;
                }
                (Some(current), Some(expected)) if current.version == expected
                    && current.created_at == command.route.created_at
                    && command.route.version == next_version(expected)? => {
                    let changed = transaction.execute(
                        "UPDATE auth_portal_routes SET portal_id = ?1, participant_id = ?2, origin = ?3, deployment_id = ?4, priority = ?5, updated_at = ?6, version = ?7
                          WHERE route_id = ?8 AND version = ?9",
                        params![command.route.portal_id, command.route.participant_id, command.route.origin, command.route.deployment_id, command.route.priority, command.route.updated_at, to_sql_version(command.route.version)?, command.route.route_id, to_sql_version(expected)?],
                    ).map_err(map_write_error)?;
                    if changed != 1 { return Err(AuthorizationStateError::StorageConflict); }
                }
                _ => return Err(AuthorizationStateError::StorageConflict),
            }
            insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &command.actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.route))
        }).await
    }

    async fn remove_portal_route(
        &self,
        command: PortalRouteRemoval,
    ) -> Result<IdempotentOutcome<PortalRouteRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            super::domain::require_nonempty("routeId", &command.route_id)?;
            super::domain::require_positive("expectedVersion", command.expected_version)?;
            let current = load_portal_route(&transaction, &command.route_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            if current.version != command.expected_version {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let changed = transaction
                .execute(
                    "DELETE FROM auth_portal_routes WHERE route_id = ?1 AND version = ?2",
                    params![command.route_id, to_sql_version(command.expected_version)?],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(current))
        })
        .await
    }

    async fn list_portal_routes(&self) -> Result<Vec<PortalRouteRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let mut statement = connection.prepare(
                "SELECT route_id, portal_id, participant_id, origin, deployment_id, priority, created_at, updated_at, version
                 FROM auth_portal_routes ORDER BY priority DESC, route_id",
            ).map_err(sql_error)?;
            let routes = statement
                .query_map([], decode_portal_route)
                .map_err(sql_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(sql_error)?;
            Ok(routes)
        }).await
    }
}

#[async_trait]
impl AccountFlowRepository for SqliteAuthorizationStore {
    async fn create_account_flow(
        &self,
        command: AccountFlowCreation,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_account_flow(&command.flow)?;
            if command.flow.kind == super::AccountFlowKind::FirstAdmin {
                return Err(AuthorizationStateError::InvalidRecord(
                    "first-admin flows must use replace_first_admin_flow".to_owned(),
                ));
            }
            insert_account_flow(&transaction, &command.flow)?;
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.flow))
        })
        .await
    }

    async fn get_account_flow_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<AccountFlowRecord>, AuthorizationStateError> {
        let token_hash = token_hash.to_owned();
        self.run(move |connection| load_account_flow_by_hash(connection, &token_hash))
            .await
    }

    async fn complete_password_reset(
        &self,
        command: PasswordResetCompletion,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            let flow = sqlite_pending_account_flow(&transaction, &command.token_hash, command.expected_flow_version, super::AccountFlowKind::PasswordReset, command.consumed_at)?;
            let principal_id = flow.target_principal_id.clone().ok_or_else(|| AuthorizationStateError::InvalidRecord("password-reset flow has no target principal".to_owned()))?;
            let current = load_local_credential(&transaction, &principal_id)?.ok_or(AuthorizationStateError::StorageConflict)?;
            validate_replacement_credential(&current, &command.replacement, &principal_id)?;
            validate_session_revocation_actions(&command.actions)?;
            let changed = transaction.execute(
                "UPDATE auth_local_credentials SET password_hash = ?1, hash_profile = ?2, failed_attempts = ?3, locked_until = ?4, password_changed_at = ?5, updated_at = ?6, version = ?7
                 WHERE principal_id = ?8 AND version = ?9",
                params![command.replacement.password_hash, i64::from(command.replacement.hash_profile), i64::from(command.replacement.failed_attempts), command.replacement.locked_until, command.replacement.password_changed_at, command.replacement.updated_at, to_sql_version(command.replacement.version)?, principal_id, to_sql_version(current.version)?],
            ).map_err(map_write_error)?;
            if changed != 1 { return Err(AuthorizationStateError::StorageConflict); }
            transaction.execute(
                "UPDATE auth_sessions SET state = 'revoked', revoked_at = ?1, version = version + 1
                 WHERE principal_id = ?2 AND state = 'active'",
                params![command.consumed_at, principal_id],
            ).map_err(map_write_error)?;
            let completed = consume_sql_flow(&transaction, &flow, command.consumed_at)?;
            insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &command.actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(completed))
        }).await
    }

    async fn complete_identity_link(
        &self,
        command: IdentityLinkCompletion,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_provider_identity(&command.identity)?;
            let flow = sqlite_pending_account_flow(&transaction, &command.token_hash, command.expected_flow_version, super::AccountFlowKind::IdentityLink, command.consumed_at)?;
            if flow.target_principal_id.as_deref() != Some(command.identity.principal_id.as_str())
                || flow.target_provider_id.as_deref() != Some(command.identity.provider.as_str())
                || load_principal(&transaction, &command.identity.principal_id)?.is_none_or(|principal| principal.kind != PrincipalKind::User)
            {
                return Err(AuthorizationStateError::InvalidRecord("identity-link flow target does not match the supplied identity".to_owned()));
            }
            transaction.execute(
                "INSERT INTO auth_provider_identities (provider, provider_subject, principal_id, linked_at, last_seen_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![command.identity.provider, command.identity.provider_subject, command.identity.principal_id, command.identity.linked_at, command.identity.last_seen_at],
            ).map_err(map_write_error)?;
            let completed = consume_sql_flow(&transaction, &flow, command.consumed_at)?;
            insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &command.actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(completed))
        }).await
    }

    async fn complete_first_admin(
        &self,
        command: FirstAdminCompletion,
    ) -> Result<IdempotentOutcome<AccountFlowRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_new_user_account(
                &command.principal,
                &command.profile,
                command.credential.as_ref(),
                Some(&command.identity),
            )?;
            validate_first_admin_authority(
                &command.authority,
                &command.principal,
                command.consumed_at,
            )?;
            let flow = sqlite_pending_account_flow(
                &transaction,
                &command.token_hash,
                command.expected_flow_version,
                super::AccountFlowKind::FirstAdmin,
                command.consumed_at,
            )?;
            if flow.target_principal_id.is_some()
                || flow.target_provider_id.is_some()
                || sql_has_active_administrator(&transaction, command.consumed_at)?
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            insert_sql_user_account(
                &transaction,
                &command.principal,
                &command.profile,
                command.credential.as_ref(),
                Some(&command.identity),
            )?;
            let mut authority = command.authority;
            put_identity_authority(&transaction, &mut authority, None)?;
            let completed = consume_sql_flow(&transaction, &flow, command.consumed_at)?;
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(completed))
        })
        .await
    }
}

#[async_trait]
impl AuthorityProposalRepository for SqliteAuthorizationStore {
    async fn list_authority_proposals(
        &self,
    ) -> Result<
        Vec<(AuthorityProposalRecord, Option<AuthorityDecisionRecord>)>,
        AuthorizationStateError,
    > {
        self.run(move |connection| {
            let mut statement = connection
                .prepare("SELECT proposal_id FROM auth_authority_proposals ORDER BY proposal_id")
                .map_err(sql_error)?;
            let ids = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            ids.into_iter()
                .map(|proposal_id| {
                    load_authority_proposal(connection, &proposal_id)?
                        .ok_or(AuthorizationStateError::StorageConflict)
                })
                .collect()
        })
        .await
    }

    async fn create_authority_proposal(
        &self,
        mut command: AuthorityProposalCreation,
    ) -> Result<IdempotentOutcome<AuthorityProposalRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            validate_authority_proposal(&command.proposal)?;
            let expired_overflow = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM auth_authority_proposals
                 WHERE authority_kind = ?1 AND authority_id = ?2 AND state = 'pending'
                   AND expires_at IS NOT NULL AND expires_at <= ?3 AND version >= ?4)",
                params![
                    encode_enum(command.proposal.authority_kind)?,
                    command.proposal.authority_id,
                    command.proposal.created_at,
                    super::MAX_PROTOCOL_INTEGER as i64,
                ],
                |row| row.get::<_, bool>(0),
            ).map_err(sql_error)?;
            if expired_overflow {
                return Err(AuthorizationStateError::InvalidRecord(
                    "expired proposal version overflow".to_owned(),
                ));
            }
            transaction.execute(
                "UPDATE auth_authority_proposals
                 SET state = 'expired', version = version + 1
                 WHERE authority_kind = ?1 AND authority_id = ?2 AND state = 'pending'
                   AND expires_at IS NOT NULL AND expires_at <= ?3",
                params![
                    encode_enum(command.proposal.authority_kind)?,
                    command.proposal.authority_id,
                    command.proposal.created_at,
                ],
            ).map_err(map_write_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                transaction.commit().map_err(sql_error)?;
                return Ok(IdempotentOutcome::Replayed(result));
            }
            let existing = transaction
                .query_row(
                    "SELECT proposal_id, authority_kind, authority_id, deployment_id, proposal_kind,
                            participant_id, participant_artifact_digest,
                            participant_needs_digest, proposed_grant_set_json,
                            proposed_capabilities_json, proposal_digest, payload_json, state,
                            created_at, expires_at, superseded_at, version
                     FROM auth_authority_proposals
                     WHERE authority_kind = ?1 AND authority_id = ?2
                       AND proposal_digest = ?3 AND state = 'pending'",
                    params![
                        encode_enum(command.proposal.authority_kind)?,
                        command.proposal.authority_id,
                        command.proposal.proposal_digest,
                    ],
                    decode_authority_proposal,
                )
                .optional()
                .map_err(sql_error)?;
            if let Some(existing) = existing {
                command.idempotency.result = json!({ "proposalId": existing.proposal_id });
                let result = command.idempotency.result.clone();
                insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &[])?;
                transaction.commit().map_err(sql_error)?;
                return Ok(IdempotentOutcome::Replayed(result));
            }
            let binding = load_participant_binding(
                &transaction,
                &command.proposal.participant_id,
                &command.proposal.participant_artifact_digest,
            )?;
            if binding
                .is_none_or(|value| value.needs_digest != command.proposal.participant_needs_digest)
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let overflow = transaction
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM auth_authority_proposals
                     WHERE authority_kind = ?1 AND authority_id = ?2 AND state = 'pending'
                       AND version >= ?3)",
                    params![
                        encode_enum(command.proposal.authority_kind)?,
                        command.proposal.authority_id,
                        super::MAX_PROTOCOL_INTEGER as i64,
                    ],
                    |row| row.get::<_, bool>(0),
                )
                .map_err(sql_error)?;
            if overflow {
                return Err(AuthorizationStateError::InvalidRecord(
                    "authority proposal version overflow".to_owned(),
                ));
            }
            transaction
                .execute(
                    "UPDATE auth_authority_proposals
                     SET state = 'superseded', superseded_at = ?3, version = version + 1
                     WHERE authority_kind = ?1 AND authority_id = ?2 AND state = 'pending'",
                    params![
                        encode_enum(command.proposal.authority_kind)?,
                        command.proposal.authority_id,
                        command.proposal.created_at,
                    ],
                )
                .map_err(map_write_error)?;
            transaction.execute(
                "INSERT INTO auth_authority_proposals (proposal_id, authority_kind, authority_id, deployment_id, proposal_kind, participant_id, participant_artifact_digest, participant_needs_digest, proposed_grant_set_json, proposed_capabilities_json, proposal_digest, payload_json, state, created_at, expires_at, superseded_at, version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![command.proposal.proposal_id, encode_enum(command.proposal.authority_kind)?, command.proposal.authority_id, command.proposal.deployment_id, encode_enum(command.proposal.proposal_kind)?, command.proposal.participant_id, command.proposal.participant_artifact_digest, command.proposal.participant_needs_digest, encode_json(&command.proposal.proposed_grant_set)?, encode_json(&command.proposal.proposed_capabilities)?, command.proposal.proposal_digest, encode_json(&command.proposal.payload)?, encode_enum(command.proposal.state)?, command.proposal.created_at, command.proposal.expires_at, command.proposal.superseded_at, to_sql_version(command.proposal.version)?],
            ).map_err(map_write_error)?;
            insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &command.actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.proposal))
        }).await
    }

    async fn get_authority_proposal(
        &self,
        proposal_id: &str,
    ) -> Result<
        Option<(AuthorityProposalRecord, Option<AuthorityDecisionRecord>)>,
        AuthorizationStateError,
    > {
        let proposal_id = proposal_id.to_owned();
        self.run(move |connection| load_authority_proposal(connection, &proposal_id))
            .await
    }

    async fn decide_authority_proposal(
        &self,
        command: AuthorityProposalDecision,
    ) -> Result<IdempotentOutcome<AuthorityProposalRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_proposal_decision(&command.proposal_id, &command.decision)?;
            let current = load_authority_proposal(&transaction, &command.proposal_id)?.map(|value| value.0).ok_or(AuthorizationStateError::StorageConflict)?;
            if current.version != command.expected_version || current.state != AuthorityProposalState::Pending
                || current.expires_at.is_some_and(|expires| command.decision.decided_at >= expires)
                || command.decision.decided_at < current.created_at
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            if command.decision.outcome == AuthorityDecisionOutcome::Accepted {
                let table = match current.authority_kind {
                    AuthorityKind::Identity => "auth_identity_authorities",
                    AuthorityKind::Deployment => "auth_deployment_authorities",
                };
                let current_authority_version = transaction
                    .query_row(
                        &format!("SELECT version FROM {table} WHERE authority_id = ?1"),
                        [&current.authority_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(sql_error)?
                    .map(from_sql_version)
                    .transpose()
                    .map_err(sql_error)?;
                if super::companion_repository::proposal_base_authority_version(&current)?
                    != current_authority_version
                {
                    return Err(AuthorizationStateError::StorageConflict);
                }
            }
            validate_proposal_desired_authority(&current, command.decision.outcome, command.desired_authority.as_ref())?;
            if let Some(deployment) = command.deployment {
                put_sql_deployment_evidence(&transaction, deployment)?;
            }
            if let Some(desired) = command.desired_authority {
                put_sql_desired_authority(&transaction, desired)?;
            }
            transaction.execute(
                "INSERT INTO auth_authority_decisions (proposal_id, outcome, decided_by, reason, decided_at, decision_digest)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![command.decision.proposal_id, encode_enum(command.decision.outcome)?, command.decision.decided_by, command.decision.reason, command.decision.decided_at, command.decision.decision_digest],
            ).map_err(map_write_error)?;
            let superseded_version_overflow = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM auth_authority_proposals
                 WHERE proposal_id != ?1 AND authority_kind = ?2 AND authority_id = ?3
                   AND state = 'pending' AND version >= ?4)",
                params![command.proposal_id, encode_enum(current.authority_kind)?, current.authority_id, super::MAX_PROTOCOL_INTEGER as i64],
                |row| row.get::<_, bool>(0),
            ).map_err(sql_error)?;
            if superseded_version_overflow {
                return Err(AuthorizationStateError::InvalidRecord(
                    "superseded proposal version overflow".to_owned(),
                ));
            }
            transaction.execute(
                "UPDATE auth_authority_proposals SET state = 'superseded', superseded_at = ?1, version = version + 1
                 WHERE proposal_id != ?2 AND authority_kind = ?3 AND authority_id = ?4 AND state = 'pending'",
                params![command.decision.decided_at, command.proposal_id, encode_enum(current.authority_kind)?, current.authority_id],
            ).map_err(map_write_error)?;
            let state = match command.decision.outcome { AuthorityDecisionOutcome::Accepted => "accepted", AuthorityDecisionOutcome::Rejected => "rejected" };
            let next = next_version(command.expected_version)?;
            let changed = transaction.execute(
                "UPDATE auth_authority_proposals SET state = ?1, version = ?2
                 WHERE proposal_id = ?3 AND version = ?4 AND state = 'pending'",
                params![state, to_sql_version(next)?, command.proposal_id, to_sql_version(command.expected_version)?],
            ).map_err(map_write_error)?;
            if changed != 1 { return Err(AuthorizationStateError::StorageConflict); }
            let result = load_authority_proposal(&transaction, &command.proposal_id)?.map(|value| value.0).ok_or(AuthorizationStateError::StorageConflict)?;
            insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &command.actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(result))
        }).await
    }
}

#[async_trait]
impl ProvisioningRepository for SqliteAuthorizationStore {
    async fn list_provisioned_identities(
        &self,
    ) -> Result<Vec<ProvisionedIdentityRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let mut statement = connection
                .prepare(
                    "SELECT identity_key_id FROM auth_provisioned_identities ORDER BY identity_key_id",
                )
                .map_err(sql_error)?;
            let ids = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            ids.into_iter()
                .map(|identity_key_id| {
                    load_provisioned_identity(connection, &identity_key_id)?
                        .ok_or(AuthorizationStateError::StorageConflict)
                })
                .collect()
        })
        .await
    }

    async fn get_provisioned_identity(
        &self,
        identity_key_id: &str,
    ) -> Result<Option<ProvisionedIdentityRecord>, AuthorizationStateError> {
        let identity_key_id = identity_key_id.to_owned();
        self.run(move |connection| load_provisioned_identity(connection, &identity_key_id))
            .await
    }

    async fn consume_device_provisioning_secret(
        &self,
        command: DeviceProvisioningSecretConsumption,
    ) -> Result<IdempotentOutcome<DeviceProvisioningSecretRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_provisioned_identity(&command.identity)?;
            let current = load_provisioning_secret_by_hash(&transaction, &command.secret_hash)?.ok_or(AuthorizationStateError::StorageConflict)?;
            if current.version != command.expected_version
                || current.state != ProvisioningSecretState::Pending
                || command.consumed_at < current.created_at
                || command.consumed_at >= current.expires_at
                || command.identity.instance_id != current.instance_id
                || command.identity.kind != ProvisionedIdentityKind::Device
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            validate_sql_identity_relationships(&transaction, &command.identity)?;
            insert_sql_provisioned_identity(&transaction, &command.identity)?;
            let next = next_version(command.expected_version)?;
            let changed = transaction.execute(
                "UPDATE auth_device_provisioning_secrets SET state = 'consumed', consumed_at = ?1, version = ?2
                 WHERE secret_hash = ?3 AND version = ?4 AND state = 'pending' AND expires_at > ?1",
                params![command.consumed_at, to_sql_version(next)?, command.secret_hash, to_sql_version(command.expected_version)?],
            ).map_err(map_write_error)?;
            if changed != 1 { return Err(AuthorizationStateError::StorageConflict); }
            let result = load_provisioning_secret_by_hash(&transaction, &command.secret_hash)?.ok_or(AuthorizationStateError::StorageConflict)?;
            insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &command.actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(result))
        }).await
    }

    async fn create_activation_review(
        &self,
        command: ActivationReviewCreation,
    ) -> Result<IdempotentOutcome<DeviceActivationReviewRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_activation_review(&command.review)?;
            let device = load_device(&transaction, &command.review.principal_id, &command.review.deployment_id)?;
            let instance = load_runtime_instance(&transaction, &command.review.instance_id)?;
            if device.is_none_or(|value| value.state == DeviceState::Revoked)
                || instance.is_none_or(|value| value.principal_id != command.review.principal_id || value.deployment_id != command.review.deployment_id)
            {
                return Err(AuthorizationStateError::InvalidRecord("activation review relationships do not match exactly".to_owned()));
            }
            transaction.execute(
                "INSERT INTO auth_device_activation_reviews (review_id, principal_id, deployment_id, instance_id, request_digest, payload_json, state, requested_at, decided_at, decided_by, reason, version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![command.review.review_id, command.review.principal_id, command.review.deployment_id, command.review.instance_id, command.review.request_digest, encode_json(&command.review.payload)?, encode_enum(command.review.state)?, command.review.requested_at, command.review.decided_at, command.review.decided_by, command.review.reason, to_sql_version(command.review.version)?],
            ).map_err(map_write_error)?;
            insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &command.actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.review))
        }).await
    }

    async fn get_activation_review(
        &self,
        review_id: &str,
    ) -> Result<Option<DeviceActivationReviewRecord>, AuthorizationStateError> {
        let review_id = review_id.to_owned();
        self.run(move |connection| load_activation_review(connection, &review_id))
            .await
    }

    async fn list_activation_reviews(
        &self,
    ) -> Result<Vec<DeviceActivationReviewRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let mut statement = connection
                .prepare("SELECT review_id FROM auth_activation_reviews ORDER BY review_id")
                .map_err(sql_error)?;
            let ids = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            ids.into_iter()
                .map(|review_id| {
                    load_activation_review(connection, &review_id)?
                        .ok_or(AuthorizationStateError::StorageConflict)
                })
                .collect()
        })
        .await
    }

    async fn decide_activation_review(
        &self,
        command: ActivationReviewDecision,
    ) -> Result<IdempotentOutcome<DeviceActivationReviewRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_activation_decision(command.state, command.decided_at, &command.decided_by)?;
            validate_activation_decision_changes(&command)?;
            let current = load_activation_review(&transaction, &command.review_id)?.ok_or(AuthorizationStateError::StorageConflict)?;
            if current.version != command.expected_version || current.state != DeviceActivationReviewState::Pending || command.decided_at < current.requested_at {
                return Err(AuthorizationStateError::StorageConflict);
            }
            if command.state == DeviceActivationReviewState::Approved {
                let changed = transaction.execute(
                    "UPDATE auth_devices SET state = ?1, updated_at = ?2, version = version + 1
                     WHERE principal_id = ?3 AND deployment_id = ?4",
                    params![encode_enum(DeviceState::Active)?, command.decided_at, current.principal_id, current.deployment_id],
                ).map_err(map_write_error)?;
                if changed != 1 { return Err(AuthorizationStateError::StorageConflict); }
            }
            if let Some(delegation) = &command.delegation {
                if delegation.principal_id != current.principal_id || delegation.deployment_id != current.deployment_id {
                    return Err(AuthorizationStateError::InvalidRecord("activation decision delegation does not match review".to_owned()));
                }
                transaction.execute(
                    "INSERT INTO auth_device_delegations (principal_id, deployment_id, required, state, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(principal_id, deployment_id) DO UPDATE SET required = excluded.required, state = excluded.state, expires_at = excluded.expires_at",
                    params![delegation.principal_id, delegation.deployment_id, delegation.required, encode_enum(delegation.state)?, delegation.expires_at],
                ).map_err(map_write_error)?;
            }
            let next = next_version(command.expected_version)?;
            let changed = transaction.execute(
                "UPDATE auth_device_activation_reviews SET state = ?1, decided_at = ?2, decided_by = ?3, reason = ?4, version = ?5
                 WHERE review_id = ?6 AND version = ?7 AND state = 'pending'",
                params![encode_enum(command.state)?, command.decided_at, command.decided_by, command.reason, to_sql_version(next)?, command.review_id, to_sql_version(command.expected_version)?],
            ).map_err(map_write_error)?;
            if changed != 1 { return Err(AuthorizationStateError::StorageConflict); }
            let result = load_activation_review(&transaction, &command.review_id)?.ok_or(AuthorizationStateError::StorageConflict)?;
            insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &command.actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(result))
        }).await
    }

    async fn provision_service_identity(
        &self,
        command: ServiceIdentityProvisioning,
    ) -> Result<IdempotentOutcome<ProvisionedIdentityRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_provisioning_aggregate(
                &command.principal,
                &command.instance,
                ProvisionedIdentityKind::Service,
            )?;
            validate_provisioned_identity(&command.identity)?;
            validate_sql_new_runtime_relationships(
                &transaction,
                &command.principal,
                &command.instance,
                ProvisionedIdentityKind::Service,
            )?;
            if command.identity.principal_id != command.principal.principal_id
                || command.identity.deployment_id != command.instance.deployment_id
                || command.identity.instance_id != command.instance.instance_id
                || command.identity.kind != ProvisionedIdentityKind::Service
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "service identity aggregate does not match exactly".to_owned(),
                ));
            }
            insert_sql_principal(&transaction, &command.principal)?;
            insert_sql_runtime_instance(&transaction, &command.instance)?;
            insert_sql_provisioned_identity(&transaction, &command.identity)?;
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.identity))
        })
        .await
    }

    async fn provision_device(
        &self,
        command: DeviceProvisioning,
    ) -> Result<IdempotentOutcome<DeviceProvisioningSecretRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_provisioning_aggregate(
                &command.principal,
                &command.instance,
                ProvisionedIdentityKind::Device,
            )?;
            validate_provisioning_secret(&command.secret)?;
            validate_device(&command.device)?;
            if let Some(identity) = &command.identity {
                validate_provisioned_identity(identity)?;
                if identity.principal_id != command.principal.principal_id
                    || identity.deployment_id != command.instance.deployment_id
                    || identity.instance_id != command.instance.instance_id
                    || identity.kind != ProvisionedIdentityKind::Device
                    || command.secret.state != ProvisioningSecretState::Consumed
                {
                    return Err(AuthorizationStateError::InvalidRecord("immediate device identity does not match provisioning".to_owned()));
                }
            }
            validate_sql_new_runtime_relationships(&transaction, &command.principal, &command.instance, ProvisionedIdentityKind::Device)?;
            if command.device.principal_id != command.principal.principal_id
                || command.device.deployment_id != command.instance.deployment_id
                || command.secret.instance_id != command.instance.instance_id
                || command.device.state != DeviceState::Pending
            {
                return Err(AuthorizationStateError::InvalidRecord("device provisioning aggregate does not match exactly".to_owned()));
            }
            insert_sql_principal(&transaction, &command.principal)?;
            insert_sql_runtime_instance(&transaction, &command.instance)?;
            transaction.execute(
                "INSERT INTO auth_devices (principal_id, deployment_id, state, created_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![command.device.principal_id, command.device.deployment_id, encode_enum(command.device.state)?, command.device.created_at, command.device.updated_at, to_sql_version(command.device.version)?],
            ).map_err(map_write_error)?;
            insert_sql_provisioning_secret(&transaction, &command.secret)?;
            if let Some(identity) = &command.identity {
                insert_sql_provisioned_identity(&transaction, identity)?;
            }
            insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &command.actions)?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.secret))
        }).await
    }

    async fn mutate_provisioned_instance(
        &self,
        command: ProvisionedInstanceMutation,
    ) -> Result<IdempotentOutcome<RuntimeInstanceRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            validate_runtime_instance(&command.instance)?;
            let current = load_runtime_instance(&transaction, &command.instance.instance_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            let current_device = command
                .device
                .as_ref()
                .map(|device| {
                    load_device(&transaction, &device.principal_id, &device.deployment_id)
                })
                .transpose()?
                .flatten();
            let visible_version = current_device
                .as_ref()
                .map_or(current.version, |device| device.version);
            if visible_version != command.expected_version
                || current.created_at != command.instance.created_at
                || current.deployment_id != command.instance.deployment_id
                || current.principal_id != command.instance.principal_id
                || command.instance.version != next_version(current.version)?
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let changed = transaction
                .execute(
                    "UPDATE auth_instances SET state = ?1, updated_at = ?2, version = ?3
                 WHERE instance_id = ?4 AND version = ?5",
                    params![
                        encode_enum(command.instance.state)?,
                        command.instance.updated_at,
                        to_sql_version(command.instance.version)?,
                        command.instance.instance_id,
                        to_sql_version(current.version)?
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let principal = load_principal(&transaction, &command.instance.principal_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            let principal_state = match command.instance.state {
                RuntimeInstanceState::Active => PrincipalState::Active,
                RuntimeInstanceState::Disabled | RuntimeInstanceState::Stale => {
                    PrincipalState::Disabled
                }
                RuntimeInstanceState::Revoked => PrincipalState::Revoked,
            };
            transaction
                .execute(
                    "UPDATE auth_principals SET state = ?1, updated_at = ?2, version = ?3,
                     disabled_at = ?4, revoked_at = ?5
                 WHERE principal_id = ?6 AND version = ?7",
                    params![
                        encode_enum(principal_state)?,
                        command.instance.updated_at,
                        to_sql_version(next_version(principal.version)?)?,
                        (principal_state == PrincipalState::Disabled)
                            .then_some(command.instance.updated_at),
                        (principal_state == PrincipalState::Revoked)
                            .then_some(command.instance.updated_at),
                        command.instance.principal_id,
                        to_sql_version(principal.version)?
                    ],
                )
                .map_err(map_write_error)?;
            if let Some(device) = &command.device {
                validate_device(device)?;
                if device.version != next_version(command.expected_version)? {
                    return Err(AuthorizationStateError::StorageConflict);
                }
                let changed = transaction
                    .execute(
                        "UPDATE auth_devices SET state = ?1, updated_at = ?2, version = ?3
                     WHERE principal_id = ?4 AND deployment_id = ?5 AND version = ?6",
                        params![
                            encode_enum(device.state)?,
                            device.updated_at,
                            to_sql_version(device.version)?,
                            device.principal_id,
                            device.deployment_id,
                            to_sql_version(command.expected_version)?
                        ],
                    )
                    .map_err(map_write_error)?;
                if changed != 1 {
                    return Err(AuthorizationStateError::StorageConflict);
                }
            }
            if let Some(identity) = &command.identity {
                let current_identity =
                    load_provisioned_identity(&transaction, &identity.identity_key_id)?
                        .ok_or(AuthorizationStateError::StorageConflict)?;
                if current_identity.identity_public_key != identity.identity_public_key
                    || current_identity.principal_id != identity.principal_id
                    || current_identity.deployment_id != identity.deployment_id
                    || current_identity.instance_id != identity.instance_id
                {
                    return Err(AuthorizationStateError::StorageConflict);
                }
                transaction
                    .execute(
                        "UPDATE auth_provisioned_identities SET state = ?1, revoked_at = ?2
                     WHERE identity_key_id = ?3",
                        params![
                            encode_enum(identity.state)?,
                            identity.revoked_at,
                            identity.identity_key_id
                        ],
                    )
                    .map_err(map_write_error)?;
            }
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.instance))
        })
        .await
    }

    async fn mutate_device_delegation(
        &self,
        command: super::companion_repository::DeviceDelegationMutation,
    ) -> Result<IdempotentOutcome<DeviceRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(value) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(value));
            }
            validate_device(&command.device)?;
            validate_device_delegation(&command.delegation)?;
            let current = load_device(
                &transaction,
                &command.device.principal_id,
                &command.device.deployment_id,
            )?
            .ok_or(AuthorizationStateError::StorageConflict)?;
            let current_delegation = load_device_delegation(
                &transaction,
                &command.device.principal_id,
                &command.device.deployment_id,
            )?
            .ok_or(AuthorizationStateError::StorageConflict)?;
            if current.version != command.expected_version
                || command.device.version != next_version(current.version)?
                || command.device.created_at != current.created_at
                || command.delegation.principal_id != current_delegation.principal_id
                || command.delegation.deployment_id != current_delegation.deployment_id
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let changed = transaction
                .execute(
                    "UPDATE auth_devices SET state = ?1, updated_at = ?2, version = ?3
                     WHERE principal_id = ?4 AND deployment_id = ?5 AND version = ?6",
                    params![
                        encode_enum(command.device.state)?,
                        command.device.updated_at,
                        to_sql_version(command.device.version)?,
                        command.device.principal_id,
                        command.device.deployment_id,
                        to_sql_version(command.expected_version)?,
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            transaction
                .execute(
                    "UPDATE auth_device_delegations SET state = ?1, expires_at = ?2
                     WHERE principal_id = ?3 AND deployment_id = ?4",
                    params![
                        encode_enum(command.delegation.state)?,
                        command.delegation.expires_at,
                        command.delegation.principal_id,
                        command.delegation.deployment_id,
                    ],
                )
                .map_err(map_write_error)?;
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.device))
        })
        .await
    }
}

#[async_trait]
impl IdempotencyRepository for SqliteAuthorizationStore {
    async fn get_idempotency_result(
        &self,
        purpose: &str,
        signer_id: &str,
        request_id: &str,
    ) -> Result<Option<IdempotencyResultRecord>, AuthorizationStateError> {
        let purpose = purpose.to_owned();
        let signer_id = signer_id.to_owned();
        let request_id = request_id.to_owned();
        self.run(move |connection| {
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
}

#[async_trait]
impl PostCommitActionRepository for SqliteAuthorizationStore {
    async fn list_ready_post_commit_actions(
        &self,
        now: i64,
        limit: usize,
    ) -> Result<Vec<PostCommitActionRecord>, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("now", now)?;
        let limit = i64::try_from(limit).map_err(|_| {
            AuthorizationStateError::InvalidRecord("limit exceeds SQLite range".to_owned())
        })?;
        self.run(move |connection| {
            let mut statement = connection.prepare(
                "SELECT action_id, kind, payload_json, created_at, attempts, next_attempt_at, claimed_until, last_error
                 FROM auth_post_commit_actions
                 WHERE next_attempt_at <= ?1 AND (claimed_until IS NULL OR claimed_until <= ?1)
                 ORDER BY next_attempt_at, action_id LIMIT ?2",
            ).map_err(sql_error)?;
            let actions = statement
                .query_map(params![now, limit], decode_post_commit_action)
                .map_err(sql_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(sql_error)?;
            Ok(actions)
        }).await
    }

    async fn claim_post_commit_action(
        &self,
        action_id: &str,
        now: i64,
        claimed_until: i64,
    ) -> Result<Option<PostCommitActionRecord>, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("now", now)?;
        super::domain::require_protocol_timestamp("claimedUntil", claimed_until)?;
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
            let changed = connection.execute(
                "UPDATE auth_post_commit_actions SET claimed_until = ?1, attempts = ?2
                 WHERE action_id = ?3 AND next_attempt_at <= ?4 AND (claimed_until IS NULL OR claimed_until <= ?4)",
                params![claimed_until, i64::from(attempts), action_id, now],
            ).map_err(map_write_error)?;
            if changed == 0 { return Ok(None); }
            load_post_commit_action(connection, &action_id)
        }).await
    }

    async fn fail_post_commit_action(
        &self,
        action_id: &str,
        expected_claimed_until: i64,
        next_attempt_at: i64,
        error: String,
    ) -> Result<PostCommitActionRecord, AuthorizationStateError> {
        super::domain::require_protocol_timestamp("expectedClaimedUntil", expected_claimed_until)?;
        super::domain::require_protocol_timestamp("nextAttemptAt", next_attempt_at)?;
        super::domain::require_nonempty("error", &error)?;
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
            let changed = connection.execute(
                "UPDATE auth_post_commit_actions SET attempts = attempts + 1, next_attempt_at = ?1, claimed_until = NULL, last_error = ?2
                 WHERE action_id = ?3 AND claimed_until = ?4 AND attempts < ?5",
                params![next_attempt_at, error, action_id, expected_claimed_until, i64::from(u32::MAX)],
            ).map_err(map_write_error)?;
            if changed != 1 { return Err(AuthorizationStateError::StorageConflict); }
            load_post_commit_action(connection, &action_id)?.ok_or(AuthorizationStateError::StorageConflict)
        }).await
    }

    async fn acknowledge_post_commit_action(
        &self,
        action_id: &str,
        expected_claimed_until: i64,
    ) -> Result<(), AuthorizationStateError> {
        super::domain::require_protocol_timestamp("expectedClaimedUntil", expected_claimed_until)?;
        let action_id = action_id.to_owned();
        self.run(move |connection| {
            let Some(current) = load_post_commit_action(connection, &action_id)? else {
                return Ok(());
            };
            if current.claimed_until != Some(expected_claimed_until) {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let changed = connection.execute(
                "DELETE FROM auth_post_commit_actions WHERE action_id = ?1 AND claimed_until = ?2",
                params![action_id, expected_claimed_until],
            ).map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
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
impl AuthSessionRepository for SqliteAuthorizationStore {
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
                        validate_session_desired_authority(&command.session, &desired)?;
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
                    validate_session_runtime_binding(&binding)?;
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
            super::domain::require_protocol_timestamp("revokedAt", command.revoked_at)?;
            validate_session_revocation_actions(&command.actions)?;
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

    async fn admit_client_bootstrap(
        &self,
        command: ClientBootstrapAdmission,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            super::domain::require_protocol_timestamp("observedAt", command.observed_at)?;
            let current = load_session(&transaction, &command.session_id)?
                .ok_or(AuthorizationStateError::SessionMissing)?;
            if current.state != SessionState::Active
                || current.principal_kind != PrincipalKind::User
                || command.observed_at < current.created_at
                || current
                    .expires_at
                    .is_some_and(|expires_at| command.observed_at >= expires_at)
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            transaction
                .execute(
                    "UPDATE auth_sessions SET last_seen_at = MAX(last_seen_at, ?1)
                     WHERE session_id = ?2 AND state = 'active'
                       AND (expires_at IS NULL OR expires_at > ?1)",
                    params![command.observed_at, command.session_id],
                )
                .map_err(map_write_error)?;
            let result = load_session(&transaction, &command.session_id)?
                .ok_or(AuthorizationStateError::SessionMissing)?;
            insert_sql_idempotency_and_actions(&transaction, &command.idempotency, &[])?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(result))
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

    async fn get_session_by_public_key(
        &self,
        public_key: &str,
    ) -> Result<Option<SessionRecord>, AuthorizationStateError> {
        let public_key = public_key.to_owned();
        self.run(move |connection| {
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
        self.run(move |connection| {
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
    async fn list_identity_authorities(
        &self,
    ) -> Result<Vec<IdentityAuthorityRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let mut statement = connection
                .prepare(
                    "SELECT authority_id, principal_id, participant_id,
                            participant_artifact_digest, accepted_needs_digest,
                            desired_grant_set_json, desired_capabilities_json, state, version,
                            created_at, updated_at, expires_at, decision_at, decision_by,
                            decision_reason
                     FROM auth_identity_authorities
                     ORDER BY authority_id",
                )
                .map_err(sql_error)?;
            let records = statement
                .query_map([], decode_identity_authority)
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            Ok(records)
        })
        .await
    }

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
    async fn list_deployment_authorities(
        &self,
    ) -> Result<Vec<DeploymentAuthorityRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let mut statement = connection
                .prepare(
                    "SELECT authority_id, deployment_id, participant_id, participant_kind,
                            participant_artifact_digest, accepted_needs_digest,
                            desired_grant_set_json, desired_capabilities_json, state, version,
                            created_at, updated_at, expires_at, decision_at, decision_by,
                            decision_reason
                     FROM auth_deployment_authorities
                     ORDER BY authority_id",
                )
                .map_err(sql_error)?;
            let records = statement
                .query_map([], decode_deployment_authority)
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            Ok(records)
        })
        .await
    }

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
    async fn list_runtime_instances(
        &self,
    ) -> Result<Vec<RuntimeInstanceRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let mut statement = connection
                .prepare("SELECT instance_id FROM auth_instances ORDER BY instance_id")
                .map_err(sql_error)?;
            let ids = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            ids.into_iter()
                .map(|instance_id| {
                    load_runtime_instance(connection, &instance_id)?
                        .ok_or(AuthorizationStateError::StorageConflict)
                })
                .collect()
        })
        .await
    }

    async fn list_devices(&self) -> Result<Vec<DeviceRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let mut statement = connection
                .prepare(
                    "SELECT principal_id, deployment_id, state, created_at, updated_at, version
                     FROM auth_devices ORDER BY deployment_id, principal_id",
                )
                .map_err(sql_error)?;
            let records = statement
                .query_map([], |row| {
                    Ok(DeviceRecord {
                        principal_id: row.get(0)?,
                        deployment_id: row.get(1)?,
                        state: decode_enum(row.get(2)?)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                        version: from_sql_version(row.get(5)?)?,
                    })
                })
                .map_err(sql_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql_error)?;
            Ok(records)
        })
        .await
    }

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
        self.run(move |connection| put_sql_deployment_evidence(connection, deployment))
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
                if existing.created_at != instance.created_at
                    || instance.version != next_version(existing.version)?
                {
                    return Err(AuthorizationStateError::StorageConflict);
                }
            } else if instance.version != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            transaction
                .execute(
                    "INSERT INTO auth_instances (instance_id, deployment_id, principal_id, state, created_at, updated_at, version)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(instance_id) DO UPDATE SET state = excluded.state, updated_at = excluded.updated_at, version = excluded.version",
                    params![
                        instance.instance_id,
                        instance.deployment_id,
                        instance.principal_id,
                        encode_enum(instance.state)?,
                        instance.created_at,
                        instance.updated_at,
                        to_sql_version(instance.version)?,
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
            if let Some(existing) =
                load_device(connection, &device.principal_id, &device.deployment_id)?
            {
                if existing.created_at != device.created_at
                    || device.version != next_version(existing.version)?
                {
                    return Err(AuthorizationStateError::StorageConflict);
                }
            } else if device.version != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            connection
                .execute(
                    "INSERT INTO auth_devices (principal_id, deployment_id, state, created_at, updated_at, version)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                     ON CONFLICT(principal_id, deployment_id) DO UPDATE SET state = excluded.state, updated_at = excluded.updated_at, version = excluded.version",
                    params![
                        device.principal_id,
                        device.deployment_id,
                        encode_enum(device.state)?,
                        device.created_at,
                        device.updated_at,
                        to_sql_version(device.version)?,
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
                            encode_json(&item.provider_identity)?,
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
            "SELECT instance_id, deployment_id, principal_id, state, created_at, updated_at, version
             FROM auth_instances WHERE instance_id = ?1",
            [instance_id],
            |row| {
                Ok(RuntimeInstanceRecord {
                    instance_id: row.get(0)?,
                    deployment_id: row.get(1)?,
                    principal_id: row.get(2)?,
                    state: decode_enum(row.get(3)?)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                    version: from_sql_version(row.get(6)?)?,
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
            "SELECT principal_id, deployment_id, state, created_at, updated_at, version FROM auth_devices
             WHERE principal_id = ?1 AND deployment_id = ?2",
            params![principal_id, deployment_id],
            |row| {
                Ok(DeviceRecord {
                    principal_id: row.get(0)?,
                    deployment_id: row.get(1)?,
                    state: decode_enum(row.get(2)?)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    version: from_sql_version(row.get(5)?)?,
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
        provider_identity: decode_json(row.get(4)?)?,
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

fn insert_sql_session(
    connection: &Connection,
    session: &SessionRecord,
) -> Result<(), AuthorizationStateError> {
    validate_session(session)?;
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

fn sql_has_active_administrator(
    connection: &Connection,
    now: i64,
) -> Result<bool, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM auth_identity_authorities authority
                JOIN auth_principals principal ON principal.principal_id = authority.principal_id
                JOIN json_each(authority.desired_capabilities_json) capability
                WHERE principal.kind = 'user' AND principal.state = 'active'
                  AND authority.state = 'accepted'
                  AND (authority.expires_at IS NULL OR authority.expires_at > ?1)
                  AND capability.value = 'admin'
            )",
            [now],
            |row| row.get::<_, bool>(0),
        )
        .map_err(sql_error)
}

fn insert_account_flow(
    connection: &Connection,
    flow: &AccountFlowRecord,
) -> Result<(), AuthorizationStateError> {
    connection.execute(
        "INSERT INTO auth_account_flows (flow_id, kind, token_hash, target_principal_id, target_provider_id, return_location, payload_json, state, created_at, expires_at, consumed_at, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![flow.flow_id, encode_enum(flow.kind)?, flow.token_hash, flow.target_principal_id, flow.target_provider_id, flow.return_location, encode_json(&flow.payload)?, encode_enum(flow.state)?, flow.created_at, flow.expires_at, flow.consumed_at, to_sql_version(flow.version)?],
    ).map_err(map_write_error)?;
    Ok(())
}

fn sqlite_idempotency_replay(
    connection: &Connection,
    input: &IdempotencyResultRecord,
) -> Result<Option<serde_json::Value>, AuthorizationStateError> {
    validate_idempotency_result(input)?;
    if let Some(existing) = load_idempotency_result(
        connection,
        &input.purpose,
        &input.signer_id,
        &input.request_id,
    )? {
        if existing.request_digest != input.request_digest {
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
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(None)
}

fn insert_sql_idempotency_and_actions(
    connection: &Connection,
    idempotency: &IdempotencyResultRecord,
    actions: &[PostCommitActionRecord],
) -> Result<(), AuthorizationStateError> {
    if sqlite_idempotency_replay(connection, idempotency)?.is_some() {
        return Err(AuthorizationStateError::StorageConflict);
    }
    for (index, action) in actions.iter().enumerate() {
        validate_post_commit_action(action)?;
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
    connection.execute(
        "INSERT INTO auth_idempotency_results (scope_key, purpose, signer_id, request_id, request_digest, result_json, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![idempotency.scope_key, idempotency.purpose, idempotency.signer_id, idempotency.request_id, idempotency.request_digest, encode_json(&idempotency.result)?, idempotency.created_at, idempotency.expires_at],
    ).map_err(map_write_error)?;
    for action in actions {
        if load_post_commit_action(connection, &action.action_id)?.is_none() {
            connection.execute(
                "INSERT INTO auth_post_commit_actions (action_id, kind, payload_json, created_at, attempts, next_attempt_at, claimed_until, last_error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![action.action_id, encode_enum(action.kind)?, encode_json(&action.payload)?, action.created_at, i64::from(action.attempts), action.next_attempt_at, action.claimed_until, action.last_error],
            ).map_err(map_write_error)?;
        }
    }
    Ok(())
}

fn sqlite_pending_account_flow(
    connection: &Connection,
    token_hash: &str,
    expected_version: u64,
    kind: super::AccountFlowKind,
    consumed_at: i64,
) -> Result<AccountFlowRecord, AuthorizationStateError> {
    super::domain::require_digest("tokenHash", token_hash)?;
    super::domain::require_protocol_timestamp("consumedAt", consumed_at)?;
    let flow = load_account_flow_by_hash(connection, token_hash)?
        .ok_or(AuthorizationStateError::StorageConflict)?;
    if flow.kind != kind
        || flow.version != expected_version
        || flow.state != AccountFlowState::Pending
        || consumed_at < flow.created_at
        || consumed_at >= flow.expires_at
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(flow)
}

fn consume_sql_flow(
    connection: &Connection,
    flow: &AccountFlowRecord,
    consumed_at: i64,
) -> Result<AccountFlowRecord, AuthorizationStateError> {
    let next = next_version(flow.version)?;
    let changed = connection
        .execute(
            "UPDATE auth_account_flows SET state = 'consumed', consumed_at = ?1, version = ?2
         WHERE flow_id = ?3 AND version = ?4 AND state = 'pending' AND expires_at > ?1",
            params![
                consumed_at,
                to_sql_version(next)?,
                flow.flow_id,
                to_sql_version(flow.version)?
            ],
        )
        .map_err(map_write_error)?;
    if changed != 1 {
        return Err(AuthorizationStateError::StorageConflict);
    }
    load_account_flow_by_hash(connection, &flow.token_hash)?
        .ok_or(AuthorizationStateError::StorageConflict)
}

fn insert_sql_user_account(
    connection: &Connection,
    principal: &PrincipalRecord,
    profile: &UserProfileRecord,
    credential: Option<&LocalCredentialRecord>,
    identity: Option<&ProviderIdentityLink>,
) -> Result<(), AuthorizationStateError> {
    validate_new_user_account(principal, profile, credential, identity)?;
    insert_sql_principal(connection, principal)?;
    connection.execute(
        "INSERT INTO auth_user_profiles (principal_id, display_name, email, image_url, created_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![profile.principal_id, profile.display_name, profile.email, profile.image_url, profile.created_at, profile.updated_at, to_sql_version(profile.version)?],
    ).map_err(map_write_error)?;
    if let Some(credential) = credential {
        connection.execute(
            "INSERT INTO auth_local_credentials (principal_id, normalized_username, password_hash, hash_profile, failed_attempts, locked_until, password_changed_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![credential.principal_id, credential.normalized_username, credential.password_hash, i64::from(credential.hash_profile), i64::from(credential.failed_attempts), credential.locked_until, credential.password_changed_at, credential.updated_at, to_sql_version(credential.version)?],
        ).map_err(map_write_error)?;
    }
    if let Some(identity) = identity {
        connection.execute(
            "INSERT INTO auth_provider_identities (provider, provider_subject, principal_id, linked_at, last_seen_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![identity.provider, identity.provider_subject, identity.principal_id, identity.linked_at, identity.last_seen_at],
        ).map_err(map_write_error)?;
    }
    Ok(())
}

fn put_sql_deployment_evidence(
    connection: &Connection,
    deployment: DeploymentRecord,
) -> Result<(), AuthorizationStateError> {
    validate_deployment_evidence(&deployment)?;
    if load_deployment(connection, &deployment.deployment_id)?.is_some_and(|existing| {
        existing.participant_id != deployment.participant_id
            || existing.participant_kind != deployment.participant_kind
    }) {
        return Err(AuthorizationStateError::InvalidRecord(
            "deployment participant identity cannot change".to_owned(),
        ));
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
}

fn put_sql_desired_authority(
    connection: &Connection,
    desired: DesiredAuthorityRecord,
) -> Result<(), AuthorizationStateError> {
    match desired {
        DesiredAuthorityRecord::Identity(mut record) => {
            let expected =
                load_identity_authority(connection, &record.principal_id, &record.participant_id)?
                    .map(|current| current.version);
            put_identity_authority(connection, &mut record, expected)?;
        }
        DesiredAuthorityRecord::Deployment(mut record) => {
            if load_deployment(connection, &record.deployment_id)?.is_none_or(|deployment| {
                deployment.participant_id != record.participant_id
                    || deployment.participant_kind != record.participant_kind
            }) {
                return Err(AuthorizationStateError::InvalidRecord(
                    "deployment authority target does not match its deployment".to_owned(),
                ));
            }
            let expected = load_deployment_authority(
                connection,
                &record.deployment_id,
                &record.participant_id,
            )?
            .map(|current| current.version);
            put_deployment_authority(connection, &mut record, expected)?;
        }
    }
    Ok(())
}

fn validate_sql_identity_relationships(
    connection: &Connection,
    identity: &ProvisionedIdentityRecord,
) -> Result<(), AuthorizationStateError> {
    let principal = load_principal(connection, &identity.principal_id)?.ok_or_else(|| {
        AuthorizationStateError::InvalidRecord(
            "provisioned identity principal is missing".to_owned(),
        )
    })?;
    let deployment = load_deployment(connection, &identity.deployment_id)?.ok_or_else(|| {
        AuthorizationStateError::InvalidRecord(
            "provisioned identity deployment is missing".to_owned(),
        )
    })?;
    let instance = load_runtime_instance(connection, &identity.instance_id)?.ok_or_else(|| {
        AuthorizationStateError::InvalidRecord(
            "provisioned identity instance is missing".to_owned(),
        )
    })?;
    let kinds_match = matches!(
        (identity.kind, principal.kind, deployment.participant_kind),
        (
            ProvisionedIdentityKind::Service,
            PrincipalKind::Service,
            trellis_protocol::ParticipantKindV1::Service
        ) | (
            ProvisionedIdentityKind::Device,
            PrincipalKind::Device,
            trellis_protocol::ParticipantKindV1::Device
        )
    );
    let device_matches = identity.kind != ProvisionedIdentityKind::Device
        || load_device(connection, &identity.principal_id, &identity.deployment_id)?.is_some();
    if !kinds_match
        || !device_matches
        || instance.principal_id != identity.principal_id
        || instance.deployment_id != identity.deployment_id
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "provisioned identity relationships do not match exactly".to_owned(),
        ));
    }
    Ok(())
}

fn validate_sql_new_runtime_relationships(
    connection: &Connection,
    principal: &PrincipalRecord,
    instance: &RuntimeInstanceRecord,
    kind: ProvisionedIdentityKind,
) -> Result<(), AuthorizationStateError> {
    let participant_kind = match kind {
        ProvisionedIdentityKind::Service => trellis_protocol::ParticipantKindV1::Service,
        ProvisionedIdentityKind::Device => trellis_protocol::ParticipantKindV1::Device,
    };
    if load_deployment(connection, &instance.deployment_id)?
        .is_none_or(|deployment| deployment.participant_kind != participant_kind)
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "provisioned instance deployment kind does not match".to_owned(),
        ));
    }
    if load_principal(connection, &principal.principal_id)?.is_some()
        || load_runtime_instance(connection, &instance.instance_id)?.is_some()
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(())
}

fn insert_sql_principal(
    connection: &Connection,
    principal: &PrincipalRecord,
) -> Result<(), AuthorizationStateError> {
    connection.execute(
        "INSERT INTO auth_principals (principal_id, kind, state, created_at, updated_at, version, disabled_at, revoked_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![principal.principal_id, encode_enum(principal.kind)?, encode_enum(principal.state)?, principal.created_at, principal.updated_at, to_sql_version(principal.version)?, principal.disabled_at, principal.revoked_at],
    ).map_err(map_write_error)?;
    Ok(())
}

fn insert_sql_runtime_instance(
    connection: &Connection,
    instance: &RuntimeInstanceRecord,
) -> Result<(), AuthorizationStateError> {
    connection.execute(
        "INSERT INTO auth_instances (instance_id, deployment_id, principal_id, state, created_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![instance.instance_id, instance.deployment_id, instance.principal_id, encode_enum(instance.state)?, instance.created_at, instance.updated_at, to_sql_version(instance.version)?],
    ).map_err(map_write_error)?;
    Ok(())
}

fn insert_sql_provisioned_identity(
    connection: &Connection,
    identity: &ProvisionedIdentityRecord,
) -> Result<(), AuthorizationStateError> {
    connection.execute(
        "INSERT INTO auth_provisioned_identities (identity_key_id, identity_public_key, principal_id, deployment_id, instance_id, kind, state, created_at, revoked_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![identity.identity_key_id, identity.identity_public_key, identity.principal_id, identity.deployment_id, identity.instance_id, encode_enum(identity.kind)?, encode_enum(identity.state)?, identity.created_at, identity.revoked_at],
    ).map_err(map_write_error)?;
    Ok(())
}

fn insert_sql_provisioning_secret(
    connection: &Connection,
    secret: &DeviceProvisioningSecretRecord,
) -> Result<(), AuthorizationStateError> {
    connection.execute(
        "INSERT INTO auth_device_provisioning_secrets (secret_id, instance_id, secret_hash, state, created_at, expires_at, consumed_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![secret.secret_id, secret.instance_id, secret.secret_hash, encode_enum(secret.state)?, secret.created_at, secret.expires_at, secret.consumed_at, to_sql_version(secret.version)?],
    ).map_err(map_write_error)?;
    Ok(())
}

fn decode_local_credential(row: &Row<'_>) -> rusqlite::Result<LocalCredentialRecord> {
    Ok(LocalCredentialRecord {
        principal_id: row.get(0)?,
        normalized_username: row.get(1)?,
        password_hash: row.get(2)?,
        hash_profile: from_sql_u32(row.get(3)?)?,
        failed_attempts: from_sql_u32(row.get(4)?)?,
        locked_until: row.get(5)?,
        password_changed_at: row.get(6)?,
        updated_at: row.get(7)?,
        version: from_sql_version(row.get(8)?)?,
    })
}

fn load_user_profile(
    connection: &Connection,
    principal_id: &str,
) -> Result<Option<UserProfileRecord>, AuthorizationStateError> {
    connection.query_row(
        "SELECT principal_id, display_name, email, image_url, created_at, updated_at, version FROM auth_user_profiles WHERE principal_id = ?1",
        [principal_id],
        |row| Ok(UserProfileRecord { principal_id: row.get(0)?, display_name: row.get(1)?, email: row.get(2)?, image_url: row.get(3)?, created_at: row.get(4)?, updated_at: row.get(5)?, version: from_sql_version(row.get(6)?)? }),
    ).optional().map_err(sql_error)
}

fn load_user_account(
    connection: &Connection,
    principal_id: &str,
) -> Result<Option<(PrincipalRecord, UserProfileRecord)>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT p.principal_id, p.kind, p.state, p.created_at, p.updated_at,
                    p.version, p.disabled_at, p.revoked_at,
                    u.principal_id, u.display_name, u.email, u.image_url,
                    u.created_at, u.updated_at, u.version
             FROM auth_principals p
             JOIN auth_user_profiles u ON u.principal_id = p.principal_id
             WHERE p.principal_id = ?1 AND p.kind = ?2",
            params![principal_id, encode_enum(PrincipalKind::User)?],
            decode_user_account,
        )
        .optional()
        .map_err(sql_error)
}

fn decode_user_account(row: &Row<'_>) -> rusqlite::Result<(PrincipalRecord, UserProfileRecord)> {
    Ok((
        PrincipalRecord {
            principal_id: row.get(0)?,
            kind: decode_enum(row.get::<_, String>(1)?)?,
            state: decode_enum(row.get::<_, String>(2)?)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
            version: from_sql_version(row.get(5)?)?,
            disabled_at: row.get(6)?,
            revoked_at: row.get(7)?,
        },
        UserProfileRecord {
            principal_id: row.get(8)?,
            display_name: row.get(9)?,
            email: row.get(10)?,
            image_url: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
            version: from_sql_version(row.get(14)?)?,
        },
    ))
}

fn load_local_credential(
    connection: &Connection,
    principal_id: &str,
) -> Result<Option<LocalCredentialRecord>, AuthorizationStateError> {
    connection.query_row(
        "SELECT principal_id, normalized_username, password_hash, hash_profile, failed_attempts, locked_until, password_changed_at, updated_at, version FROM auth_local_credentials WHERE principal_id = ?1",
        [principal_id],
        |row| Ok(LocalCredentialRecord { principal_id: row.get(0)?, normalized_username: row.get(1)?, password_hash: row.get(2)?, hash_profile: from_sql_u32(row.get(3)?)?, failed_attempts: from_sql_u32(row.get(4)?)?, locked_until: row.get(5)?, password_changed_at: row.get(6)?, updated_at: row.get(7)?, version: from_sql_version(row.get(8)?)? }),
    ).optional().map_err(sql_error)
}

fn insert_deployment_profile(
    connection: &Connection,
    profile: &DeploymentProfileRecord,
) -> Result<(), AuthorizationStateError> {
    connection
        .execute(
            "INSERT INTO auth_deployment_profiles
             (deployment_id, kind, display_name, participant_id, portal_id,
              requires_device_delegation, expires_at, state, created_at, updated_at, version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                profile.deployment_id,
                encode_enum(profile.kind)?,
                profile.display_name,
                profile.participant_id,
                profile.portal_id,
                profile.requires_device_delegation,
                profile.expires_at,
                encode_enum(profile.state)?,
                profile.created_at,
                profile.updated_at,
                to_sql_version(profile.version)?,
            ],
        )
        .map_err(map_write_error)?;
    Ok(())
}

fn upsert_deployment_profile_evidence(
    connection: &Connection,
    profile: &DeploymentProfileRecord,
) -> Result<(), AuthorizationStateError> {
    let Some(participant_id) = &profile.participant_id else {
        return Ok(());
    };
    let participant_kind = match profile.kind {
        PrincipalKind::Service => trellis_protocol::ParticipantKindV1::Service,
        PrincipalKind::Device => trellis_protocol::ParticipantKindV1::Device,
        PrincipalKind::User => {
            return Err(AuthorizationStateError::InvalidRecord(
                "deployment profile cannot use user kind".to_owned(),
            ));
        }
    };
    if let Some(existing) = load_deployment(connection, &profile.deployment_id)? {
        if existing.participant_id != *participant_id
            || existing.participant_kind != participant_kind
        {
            return Err(AuthorizationStateError::StorageConflict);
        }
    }
    let state = match profile.state {
        DeploymentProfileState::Active => "active",
        DeploymentProfileState::Disabled => "disabled",
        DeploymentProfileState::Removed => "revoked",
    };
    connection
        .execute(
            "INSERT INTO auth_deployments
             (deployment_id, participant_id, participant_kind, state, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(deployment_id) DO UPDATE SET
                 state = excluded.state, expires_at = excluded.expires_at",
            params![
                profile.deployment_id,
                participant_id,
                encode_enum(participant_kind)?,
                state,
                profile.expires_at,
            ],
        )
        .map_err(map_write_error)?;
    Ok(())
}

fn load_deployment_profile(
    connection: &Connection,
    deployment_id: &str,
) -> Result<Option<DeploymentProfileRecord>, AuthorizationStateError> {
    connection
        .query_row(
            "SELECT deployment_id, kind, display_name, participant_id, portal_id,
                    requires_device_delegation, expires_at, state, created_at, updated_at, version
             FROM auth_deployment_profiles WHERE deployment_id = ?1",
            [deployment_id],
            decode_deployment_profile,
        )
        .optional()
        .map_err(sql_error)
}

fn decode_deployment_profile(row: &Row<'_>) -> rusqlite::Result<DeploymentProfileRecord> {
    Ok(DeploymentProfileRecord {
        deployment_id: row.get(0)?,
        kind: decode_enum(row.get(1)?)?,
        display_name: row.get(2)?,
        participant_id: row.get(3)?,
        portal_id: row.get(4)?,
        requires_device_delegation: row.get(5)?,
        expires_at: row.get(6)?,
        state: decode_enum(row.get(7)?)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        version: from_sql_version(row.get(10)?)?,
    })
}

fn load_login_portal(
    connection: &Connection,
    portal_id: &str,
) -> Result<Option<(LoginPortalRecord, LoginSettingsRecord)>, AuthorizationStateError> {
    let portal = connection.query_row(
        "SELECT portal_id, display_name, entry_url, builtin, disabled, removed, local_registration_enabled, provider_ids_json, created_at, updated_at, version FROM auth_login_portals WHERE portal_id = ?1",
        [portal_id],
        |row| Ok(LoginPortalRecord { portal_id: row.get(0)?, display_name: row.get(1)?, entry_url: row.get(2)?, builtin: row.get(3)?, disabled: row.get(4)?, removed: row.get(5)?, local_registration_enabled: row.get(6)?, provider_ids: decode_json(row.get(7)?)?, created_at: row.get(8)?, updated_at: row.get(9)?, version: from_sql_version(row.get(10)?)? }),
    ).optional().map_err(sql_error)?;
    let Some(portal) = portal else {
        return Ok(None);
    };
    let settings = connection.query_row(
        "SELECT portal_id, default_provider_id, local_login_enabled, federated_registration_enabled, provider_selection_enabled, updated_at, version FROM auth_login_settings WHERE portal_id = ?1",
        [portal_id],
        |row| Ok(LoginSettingsRecord { portal_id: row.get(0)?, default_provider_id: row.get(1)?, local_login_enabled: row.get(2)?, federated_registration_enabled: row.get(3)?, provider_selection_enabled: row.get(4)?, updated_at: row.get(5)?, version: from_sql_version(row.get(6)?)? }),
    ).optional().map_err(sql_error)?.ok_or_else(|| AuthorizationStateError::Storage("login portal settings are missing".to_owned()))?;
    Ok(Some((portal, settings)))
}

fn decode_portal_route(row: &Row<'_>) -> rusqlite::Result<PortalRouteRecord> {
    Ok(PortalRouteRecord {
        route_id: row.get(0)?,
        portal_id: row.get(1)?,
        participant_id: row.get(2)?,
        origin: row.get(3)?,
        deployment_id: row.get(4)?,
        priority: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        version: from_sql_version(row.get(8)?)?,
    })
}

fn load_portal_route(
    connection: &Connection,
    route_id: &str,
) -> Result<Option<PortalRouteRecord>, AuthorizationStateError> {
    connection.query_row(
        "SELECT route_id, portal_id, participant_id, origin, deployment_id, priority, created_at, updated_at, version FROM auth_portal_routes WHERE route_id = ?1",
        [route_id], decode_portal_route,
    ).optional().map_err(sql_error)
}

fn decode_account_flow(row: &Row<'_>) -> rusqlite::Result<AccountFlowRecord> {
    Ok(AccountFlowRecord {
        flow_id: row.get(0)?,
        kind: decode_enum(row.get(1)?)?,
        token_hash: row.get(2)?,
        target_principal_id: row.get(3)?,
        target_provider_id: row.get(4)?,
        return_location: row.get(5)?,
        payload: decode_json(row.get(6)?)?,
        state: decode_enum(row.get(7)?)?,
        created_at: row.get(8)?,
        expires_at: row.get(9)?,
        consumed_at: row.get(10)?,
        version: from_sql_version(row.get(11)?)?,
    })
}

fn load_account_flow_by_hash(
    connection: &Connection,
    token_hash: &str,
) -> Result<Option<AccountFlowRecord>, AuthorizationStateError> {
    connection.query_row(
        "SELECT flow_id, kind, token_hash, target_principal_id, target_provider_id, return_location, payload_json, state, created_at, expires_at, consumed_at, version FROM auth_account_flows WHERE token_hash = ?1",
        [token_hash], decode_account_flow,
    ).optional().map_err(sql_error)
}

fn decode_authority_proposal(row: &Row<'_>) -> rusqlite::Result<AuthorityProposalRecord> {
    Ok(AuthorityProposalRecord {
        proposal_id: row.get(0)?,
        authority_kind: decode_enum(row.get::<_, String>(1)?)?,
        authority_id: row.get(2)?,
        deployment_id: row.get(3)?,
        proposal_kind: decode_enum(row.get(4)?)?,
        participant_id: row.get(5)?,
        participant_artifact_digest: row.get(6)?,
        participant_needs_digest: row.get(7)?,
        proposed_grant_set: decode_json(row.get(8)?)?,
        proposed_capabilities: decode_json(row.get(9)?)?,
        proposal_digest: row.get(10)?,
        payload: decode_json(row.get(11)?)?,
        state: decode_enum(row.get(12)?)?,
        created_at: row.get(13)?,
        expires_at: row.get(14)?,
        superseded_at: row.get(15)?,
        version: from_sql_version(row.get(16)?)?,
    })
}

fn load_authority_proposal(
    connection: &Connection,
    proposal_id: &str,
) -> Result<
    Option<(AuthorityProposalRecord, Option<AuthorityDecisionRecord>)>,
    AuthorizationStateError,
> {
    let proposal = connection.query_row(
        "SELECT proposal_id, authority_kind, authority_id, deployment_id, proposal_kind, participant_id, participant_artifact_digest, participant_needs_digest, proposed_grant_set_json, proposed_capabilities_json, proposal_digest, payload_json, state, created_at, expires_at, superseded_at, version FROM auth_authority_proposals WHERE proposal_id = ?1",
        [proposal_id], decode_authority_proposal,
    ).optional().map_err(sql_error)?;
    let Some(proposal) = proposal else {
        return Ok(None);
    };
    let decision = connection.query_row(
        "SELECT proposal_id, outcome, decided_by, reason, decided_at, decision_digest FROM auth_authority_decisions WHERE proposal_id = ?1",
        [proposal_id],
        |row| Ok(AuthorityDecisionRecord { proposal_id: row.get(0)?, outcome: decode_enum(row.get(1)?)?, decided_by: row.get(2)?, reason: row.get(3)?, decided_at: row.get(4)?, decision_digest: row.get(5)? }),
    ).optional().map_err(sql_error)?;
    Ok(Some((proposal, decision)))
}

fn load_provisioned_identity(
    connection: &Connection,
    identity_key_id: &str,
) -> Result<Option<ProvisionedIdentityRecord>, AuthorizationStateError> {
    connection.query_row(
        "SELECT identity_key_id, identity_public_key, principal_id, deployment_id, instance_id, kind, state, created_at, revoked_at FROM auth_provisioned_identities WHERE identity_key_id = ?1",
        [identity_key_id],
        |row| Ok(ProvisionedIdentityRecord { identity_key_id: row.get(0)?, identity_public_key: row.get(1)?, principal_id: row.get(2)?, deployment_id: row.get(3)?, instance_id: row.get(4)?, kind: decode_enum(row.get(5)?)?, state: decode_enum(row.get(6)?)?, created_at: row.get(7)?, revoked_at: row.get(8)? }),
    ).optional().map_err(sql_error)
}

fn decode_provisioning_secret(row: &Row<'_>) -> rusqlite::Result<DeviceProvisioningSecretRecord> {
    Ok(DeviceProvisioningSecretRecord {
        secret_id: row.get(0)?,
        instance_id: row.get(1)?,
        secret_hash: row.get(2)?,
        state: decode_enum(row.get(3)?)?,
        created_at: row.get(4)?,
        expires_at: row.get(5)?,
        consumed_at: row.get(6)?,
        version: from_sql_version(row.get(7)?)?,
    })
}

fn load_provisioning_secret_by_hash(
    connection: &Connection,
    secret_hash: &str,
) -> Result<Option<DeviceProvisioningSecretRecord>, AuthorizationStateError> {
    connection.query_row(
        "SELECT secret_id, instance_id, secret_hash, state, created_at, expires_at, consumed_at, version FROM auth_device_provisioning_secrets WHERE secret_hash = ?1",
        [secret_hash], decode_provisioning_secret,
    ).optional().map_err(sql_error)
}

fn load_activation_review(
    connection: &Connection,
    review_id: &str,
) -> Result<Option<DeviceActivationReviewRecord>, AuthorizationStateError> {
    connection.query_row(
        "SELECT review_id, principal_id, deployment_id, instance_id, request_digest, payload_json, state, requested_at, decided_at, decided_by, reason, version FROM auth_device_activation_reviews WHERE review_id = ?1",
        [review_id],
        |row| Ok(DeviceActivationReviewRecord { review_id: row.get(0)?, principal_id: row.get(1)?, deployment_id: row.get(2)?, instance_id: row.get(3)?, request_digest: row.get(4)?, payload: decode_json(row.get(5)?)?, state: decode_enum(row.get(6)?)?, requested_at: row.get(7)?, decided_at: row.get(8)?, decided_by: row.get(9)?, reason: row.get(10)?, version: from_sql_version(row.get(11)?)? }),
    ).optional().map_err(sql_error)
}

fn load_idempotency_result(
    connection: &Connection,
    purpose: &str,
    signer_id: &str,
    request_id: &str,
) -> Result<Option<IdempotencyResultRecord>, AuthorizationStateError> {
    connection.query_row(
        "SELECT scope_key, purpose, signer_id, request_id, request_digest, result_json, created_at, expires_at FROM auth_idempotency_results WHERE purpose = ?1 AND signer_id = ?2 AND request_id = ?3",
        params![purpose, signer_id, request_id],
        |row| Ok(IdempotencyResultRecord { scope_key: row.get(0)?, purpose: row.get(1)?, signer_id: row.get(2)?, request_id: row.get(3)?, request_digest: row.get(4)?, result: decode_json(row.get(5)?)?, created_at: row.get(6)?, expires_at: row.get(7)? }),
    ).optional().map_err(sql_error)
}

fn decode_post_commit_action(row: &Row<'_>) -> rusqlite::Result<PostCommitActionRecord> {
    Ok(PostCommitActionRecord {
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

fn load_post_commit_action(
    connection: &Connection,
    action_id: &str,
) -> Result<Option<PostCommitActionRecord>, AuthorizationStateError> {
    connection.query_row(
        "SELECT action_id, kind, payload_json, created_at, attempts, next_attempt_at, claimed_until, last_error FROM auth_post_commit_actions WHERE action_id = ?1",
        [action_id], decode_post_commit_action,
    ).optional().map_err(sql_error)
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

fn from_sql_u32(value: i64) -> rusqlite::Result<u32> {
    u32::try_from(value).map_err(|_| decode_failure("integer is outside u32 range"))
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
