use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension};

use super::super::application::repository::{
    ActivationReviewClaim, ActivationReviewCreation, ActivationReviewDecision, DeviceProvisioning,
    DeviceProvisioningSecretConsumption, IdempotentOutcome, ProvisionedInstanceMutation,
    ProvisioningRepository, ServiceIdentityProvisioning,
};
use super::super::context::{
    revoke_sql_contexts, AuthorizationContextRevocationReason, AuthorizationContextSelector,
};
use super::super::{
    activation_review_event, activation_review_event_action_id, AuthorizationStateError,
    DeviceActivationReviewRecord, DeviceActivationReviewState, DeviceDelegationState,
    DeviceProvisioningSecretRecord, DeviceRecord, DeviceState, IdempotencyResultRecord,
    PrincipalKind, PrincipalRecord, PrincipalState, ProvisionedIdentityKind,
    ProvisionedIdentityRecord, ProvisioningSecretState, RuntimeInstanceRecord,
    RuntimeInstanceState,
};
use super::common::{
    decode_enum, decode_json, encode_enum, encode_json, from_sql_version, map_write_error,
    sql_error, to_sql_version,
};
use super::evidence::{
    load_deployment, load_device, load_device_delegation, load_runtime_instance,
    validate_sql_device_relationships,
};
use super::outbox::{insert_sql_idempotency_and_actions, sqlite_idempotency_replay};
use super::principals::load_principal;
use super::validation::next_version;
use super::SqliteAuthorizationStore;
use crate::platform::auth::model::validate_provisioned_identity;

#[async_trait]
impl ProvisioningRepository for SqliteAuthorizationStore {
    async fn list_provisioned_identities(
        &self,
    ) -> Result<Vec<ProvisionedIdentityRecord>, AuthorizationStateError> {
        self.run_read(move |connection| {
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
        self.run_read(move |connection| load_provisioned_identity(connection, &identity_key_id))
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
            let current = load_provisioning_secret_by_hash(&transaction, &command.secret_hash)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
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
            let changed = transaction
                .execute(
                    "UPDATE auth_device_provisioning_secrets SET state = 'consumed', consumed_at = ?1, version = ?2
                 WHERE secret_hash = ?3 AND version = ?4 AND state = 'pending' AND expires_at > ?1",
                    params![
                        command.consumed_at,
                        to_sql_version(next)?,
                        command.secret_hash,
                        to_sql_version(command.expected_version)?
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let result = load_provisioning_secret_by_hash(&transaction, &command.secret_hash)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
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

    async fn create_activation_review(
        &self,
        command: ActivationReviewCreation,
    ) -> Result<IdempotentOutcome<DeviceActivationReviewRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            let device = load_device(
                &transaction,
                &command.review.principal_id,
                &command.review.deployment_id,
            )?;
            let instance = load_runtime_instance(&transaction, &command.review.instance_id)?;
            if device.is_none_or(|value| value.state == DeviceState::Revoked)
                || instance.is_none_or(|value| {
                    value.principal_id != command.review.principal_id
                        || value.deployment_id != command.review.deployment_id
                })
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "activation review relationships do not match exactly".to_owned(),
                ));
            }
            transaction
                .execute(
                    "INSERT INTO auth_device_activation_reviews (review_id, principal_id, deployment_id, instance_id, request_digest, payload_json, state, requested_at, expires_at, activated_by_user_principal_id, decided_at, decided_by, reason, version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                    params![
                        command.review.review_id,
                        command.review.principal_id,
                        command.review.deployment_id,
                        command.review.instance_id,
                        command.review.request_digest,
                        encode_json(&command.review.payload)?,
                        encode_enum(command.review.state)?,
                        command.review.requested_at,
                        command.review.expires_at,
                        command.review.activated_by_user_principal_id,
                        command.review.decided_at,
                        command.review.decided_by,
                        command.review.reason,
                        to_sql_version(command.review.version)?
                    ],
                )
                .map_err(map_write_error)?;
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.review))
        })
        .await
    }

    async fn get_activation_review(
        &self,
        review_id: &str,
    ) -> Result<Option<DeviceActivationReviewRecord>, AuthorizationStateError> {
        let review_id = review_id.to_owned();
        self.run_read(move |connection| load_activation_review(connection, &review_id))
            .await
    }

    async fn expire_due_activation_reviews(
        &self,
        now: i64,
    ) -> Result<Vec<DeviceActivationReviewRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            let ids = {
                let mut statement = transaction
                    .prepare(
                        "SELECT review.review_id, review.state, review.activated_by_user_principal_id
                         FROM auth_device_activation_reviews AS review
                         WHERE review.state IN ('pending', 'approved')
                           AND review.expires_at <= ?1
                           AND EXISTS (
                               SELECT 1 FROM auth_devices AS device
                               WHERE device.principal_id = review.principal_id
                                 AND device.deployment_id = review.deployment_id
                                 AND device.state = 'pending'
                           )
                         ORDER BY review.review_id",
                    )
                    .map_err(sql_error)?;
                let ids = statement
                    .query_map([now], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            decode_enum::<DeviceActivationReviewState>(row.get::<_, String>(1)?)?,
                            row.get::<_, Option<String>>(2)?,
                        ))
                    })
                    .map_err(sql_error)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(sql_error)?;
                ids
            };
            transaction
                .execute(
                    "UPDATE auth_device_activation_reviews AS review
                     SET state = 'expired', version = version + 1
                     WHERE review.state IN ('pending', 'approved')
                       AND review.expires_at <= ?1
                       AND EXISTS (
                           SELECT 1 FROM auth_devices AS device
                           WHERE device.principal_id = review.principal_id
                             AND device.deployment_id = review.deployment_id
                             AND device.state = 'pending'
                       )",
                    [now],
                )
                .map_err(map_write_error)?;
            let reviews = ids
                .into_iter()
                .map(|(review_id, previous_state, claimant)| {
                    let review = load_activation_review(&transaction, &review_id)?
                        .ok_or(AuthorizationStateError::StorageConflict)?;
                    let mut action = activation_review_event(
                        &review,
                        "resolved",
                        "Auth.DeviceUserAuthorities.Resolved",
                        now,
                        serde_json::json!({ "state": "expired" }),
                    )?;
                    action.predecessor_action_id = Some(activation_review_event_action_id(
                        &review_id,
                        if previous_state == DeviceActivationReviewState::Approved {
                            "approved"
                        } else if claimant.is_some() {
                            "requested"
                        } else {
                            "review-requested"
                        },
                    )?);
                    insert_sql_idempotency_and_actions(
                        &transaction,
                        &IdempotencyResultRecord {
                            scope_key: trellis_protocol::digest_json(&serde_json::json!({
                                "purpose": "device.activation.expire",
                                "reviewId": review_id,
                            }))
                            .map_err(|error| {
                                AuthorizationStateError::InvalidRecord(error.to_string())
                            })?,
                            purpose: "device.activation.expire".to_owned(),
                            signer_id: "trellis.auth".to_owned(),
                            request_id: review_id,
                            request_digest: review.request_digest.clone(),
                            result: serde_json::Value::Null,
                            created_at: now,
                            expires_at: now.checked_add(86_400_000).ok_or_else(|| {
                                AuthorizationStateError::InvalidRecord(
                                    "activation expiry idempotency overflow".to_owned(),
                                )
                            })?,
                        },
                        &[action],
                    )?;
                    Ok(review)
                })
                .collect::<Result<Vec<_>, _>>()?;
            transaction.commit().map_err(sql_error)?;
            Ok(reviews)
        })
        .await
    }

    async fn claim_activation_review(
        &self,
        command: ActivationReviewClaim,
    ) -> Result<IdempotentOutcome<DeviceActivationReviewRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
            let current = load_activation_review(&transaction, &command.review_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            if current.version != command.expected_version
                || !matches!(
                    current.state,
                    DeviceActivationReviewState::Pending
                        | DeviceActivationReviewState::Approved
                )
                || current.activated_by_user_principal_id.as_deref().is_some_and(|principal| {
                    principal != command.activated_by_user_principal_id
                })
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            if let Some(delegation) = &command.delegation {
                if current.state != DeviceActivationReviewState::Approved
                    || delegation.principal_id != current.principal_id
                    || delegation.deployment_id != current.deployment_id
                    || !delegation.required
                    || delegation.state != DeviceDelegationState::Active
                {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "activation claim delegation does not match approved review".to_owned(),
                    ));
                }
                transaction
                    .execute(
                        "INSERT INTO auth_device_delegations (principal_id, deployment_id, required, state, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)
                      ON CONFLICT(principal_id, deployment_id) DO UPDATE SET required = excluded.required, state = excluded.state, expires_at = excluded.expires_at",
                        params![
                            delegation.principal_id,
                            delegation.deployment_id,
                            delegation.required,
                            encode_enum(delegation.state)?,
                            delegation.expires_at
                        ],
                    )
                    .map_err(map_write_error)?;
                let changed = transaction
                    .execute(
                        "UPDATE auth_devices SET state = ?1, updated_at = ?2, version = version + 1
                         WHERE principal_id = ?3 AND deployment_id = ?4",
                        params![
                            encode_enum(DeviceState::Active)?,
                            command.now,
                            current.principal_id,
                            current.deployment_id,
                        ],
                    )
                    .map_err(map_write_error)?;
                if changed != 1 {
                    return Err(AuthorizationStateError::StorageConflict);
                }
            }
            let next = next_version(command.expected_version)?;
            let changed = transaction
                .execute(
                    "UPDATE auth_device_activation_reviews SET activated_by_user_principal_id = ?1, version = ?2 WHERE review_id = ?3 AND version = ?4 AND expires_at > ?5 AND (activated_by_user_principal_id IS NULL OR activated_by_user_principal_id = ?1)",
                    params![
                        command.activated_by_user_principal_id,
                        to_sql_version(next)?,
                        command.review_id,
                        to_sql_version(command.expected_version)?,
                        command.now,
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let result = load_activation_review(&transaction, &command.review_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
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

    async fn list_activation_reviews(
        &self,
    ) -> Result<Vec<DeviceActivationReviewRecord>, AuthorizationStateError> {
        self.run_read(move |connection| {
            let mut statement = connection
                .prepare("SELECT review_id FROM auth_device_activation_reviews ORDER BY review_id")
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
            let current = load_activation_review(&transaction, &command.review_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            if current.version != command.expected_version
                || current.state != DeviceActivationReviewState::Pending
                || command.decided_at < current.requested_at
            {
                return Err(AuthorizationStateError::StorageConflict);
            }
            if command.activate_device {
                let changed = transaction
                    .execute(
                        "UPDATE auth_devices SET state = ?1, updated_at = ?2, version = version + 1
                     WHERE principal_id = ?3 AND deployment_id = ?4",
                        params![
                            encode_enum(DeviceState::Active)?,
                            command.decided_at,
                            current.principal_id,
                            current.deployment_id
                        ],
                    )
                    .map_err(map_write_error)?;
                if changed != 1 {
                    return Err(AuthorizationStateError::StorageConflict);
                }
            }
            if let Some(delegation) = &command.delegation {
                if delegation.principal_id != current.principal_id
                    || delegation.deployment_id != current.deployment_id
                {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "activation decision delegation does not match review".to_owned(),
                    ));
                }
                transaction
                    .execute(
                        "INSERT INTO auth_device_delegations (principal_id, deployment_id, required, state, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)
                      ON CONFLICT(principal_id, deployment_id) DO UPDATE SET required = excluded.required, state = excluded.state, expires_at = excluded.expires_at",
                        params![
                            delegation.principal_id,
                            delegation.deployment_id,
                            delegation.required,
                            encode_enum(delegation.state)?,
                            delegation.expires_at
                        ],
                    )
                    .map_err(map_write_error)?;
            }
            let next = next_version(command.expected_version)?;
            let changed = transaction
                .execute(
                    "UPDATE auth_device_activation_reviews SET state = ?1, decided_at = ?2, decided_by = ?3, reason = ?4, version = ?5
                   WHERE review_id = ?6 AND version = ?7 AND state = 'pending' AND expires_at > ?2",
                    params![
                        encode_enum(command.state)?,
                        command.decided_at,
                        command.decided_by,
                        command.reason,
                        to_sql_version(next)?,
                        command.review_id,
                        to_sql_version(command.expected_version)?
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            let result = load_activation_review(&transaction, &command.review_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
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

    async fn provision_service_identity(
        &self,
        command: ServiceIdentityProvisioning,
    ) -> Result<IdempotentOutcome<ProvisionedIdentityRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(result) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(result));
            }
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
            if let Some(identity) = &command.identity {
                if identity.principal_id != command.principal.principal_id
                    || identity.deployment_id != command.instance.deployment_id
                    || identity.instance_id != command.instance.instance_id
                    || identity.kind != ProvisionedIdentityKind::Device
                    || command.secret.state != ProvisioningSecretState::Consumed
                {
                    return Err(AuthorizationStateError::InvalidRecord(
                        "immediate device identity does not match provisioning".to_owned(),
                    ));
                }
            }
            validate_sql_new_runtime_relationships(
                &transaction,
                &command.principal,
                &command.instance,
                ProvisionedIdentityKind::Device,
            )?;
            if command.device.principal_id != command.principal.principal_id
                || command.device.deployment_id != command.instance.deployment_id
                || command.secret.instance_id != command.instance.instance_id
                || command.device.state != DeviceState::Pending
            {
                return Err(AuthorizationStateError::InvalidRecord(
                    "device provisioning aggregate does not match exactly".to_owned(),
                ));
            }
            insert_sql_principal(&transaction, &command.principal)?;
            validate_sql_device_relationships(&transaction, &command.device)?;
            insert_sql_runtime_instance(&transaction, &command.instance)?;
            transaction
                .execute(
                    "INSERT INTO auth_devices (principal_id, deployment_id, state, created_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        command.device.principal_id,
                        command.device.deployment_id,
                        encode_enum(command.device.state)?,
                        command.device.created_at,
                        command.device.updated_at,
                        to_sql_version(command.device.version)?
                    ],
                )
                .map_err(map_write_error)?;
            insert_sql_provisioning_secret(&transaction, &command.secret)?;
            if let Some(identity) = &command.identity {
                insert_sql_provisioned_identity(&transaction, identity)?;
            }
            insert_sql_idempotency_and_actions(
                &transaction,
                &command.idempotency,
                &command.actions,
            )?;
            transaction.commit().map_err(sql_error)?;
            Ok(IdempotentOutcome::Applied(command.secret))
        })
        .await
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
            let principal = load_principal(&transaction, &command.instance.principal_id)?
                .ok_or(AuthorizationStateError::StorageConflict)?;
            let principal_state = match command.instance.state {
                RuntimeInstanceState::Active => PrincipalState::Active,
                RuntimeInstanceState::Disabled | RuntimeInstanceState::Stale => {
                    PrincipalState::Disabled
                }
                RuntimeInstanceState::Revoked => PrincipalState::Revoked,
            };
            let current_identity = command
                .identity
                .as_ref()
                .map(|identity| load_provisioned_identity(&transaction, &identity.identity_key_id))
                .transpose()?
                .flatten();
            let authorization_changed = current.state != command.instance.state
                || principal.state != principal_state
                || match (&current_device, &command.device) {
                    (Some(current), Some(next)) => current.state != next.state,
                    (None, None) => false,
                    _ => true,
                }
                || match (&current_identity, &command.identity) {
                    (Some(current), Some(next)) => {
                        current.state != next.state || current.revoked_at != next.revoked_at
                    }
                    (None, None) => false,
                    _ => true,
                };
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
                let current_identity = current_identity
                    .as_ref()
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
            if authorization_changed {
                revoke_sql_contexts(
                    &transaction,
                    &AuthorizationContextSelector::Instance(command.instance.instance_id.clone()),
                    AuthorizationContextRevocationReason::InstanceChanged,
                    command.instance.updated_at.div_euclid(1_000),
                )?;
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
        command: super::super::application::repository::DeviceDelegationMutation,
    ) -> Result<IdempotentOutcome<DeviceRecord>, AuthorizationStateError> {
        self.run(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            if let Some(value) = sqlite_idempotency_replay(&transaction, &command.idempotency)? {
                return Ok(IdempotentOutcome::Replayed(value));
            }
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
            let delegation_changed = current_delegation.required != command.delegation.required
                || current_delegation.state != command.delegation.state
                || current_delegation.expires_at != command.delegation.expires_at;
            let authorization_changed = current.state != command.device.state || delegation_changed;
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
                        to_sql_version(command.expected_version)?
                    ],
                )
                .map_err(map_write_error)?;
            if changed != 1 {
                return Err(AuthorizationStateError::StorageConflict);
            }
            transaction
                .execute(
                    "UPDATE auth_device_delegations SET required = ?1, state = ?2, expires_at = ?3
                      WHERE principal_id = ?4 AND deployment_id = ?5",
                    params![
                        command.delegation.required,
                        encode_enum(command.delegation.state)?,
                        command.delegation.expires_at,
                        command.delegation.principal_id,
                        command.delegation.deployment_id
                    ],
                )
                .map_err(map_write_error)?;
            if authorization_changed {
                let reason = if delegation_changed {
                    AuthorizationContextRevocationReason::DelegationChanged
                } else {
                    AuthorizationContextRevocationReason::DeviceChanged
                };
                revoke_sql_contexts(
                    &transaction,
                    &AuthorizationContextSelector::Principal(command.device.principal_id.clone()),
                    reason,
                    command.device.updated_at.div_euclid(1_000),
                )?;
            }
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

pub(in crate::platform::auth) fn validate_sql_identity_relationships(
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

pub(in crate::platform::auth) fn validate_sql_new_runtime_relationships(
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

pub(in crate::platform::auth) fn insert_sql_principal(
    connection: &Connection,
    principal: &PrincipalRecord,
) -> Result<(), AuthorizationStateError> {
    connection
    .execute(
        "INSERT INTO auth_principals (principal_id, kind, state, created_at, updated_at, version, disabled_at, revoked_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            principal.principal_id,
            encode_enum(principal.kind)?,
            encode_enum(principal.state)?,
            principal.created_at,
            principal.updated_at,
            to_sql_version(principal.version)?,
            principal.disabled_at,
            principal.revoked_at
        ],
    )
    .map_err(map_write_error)?;
    Ok(())
}

pub(in crate::platform::auth) fn insert_sql_runtime_instance(
    connection: &Connection,
    instance: &RuntimeInstanceRecord,
) -> Result<(), AuthorizationStateError> {
    connection
    .execute(
        "INSERT INTO auth_instances (instance_id, deployment_id, principal_id, state, created_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            instance.instance_id,
            instance.deployment_id,
            instance.principal_id,
            encode_enum(instance.state)?,
            instance.created_at,
            instance.updated_at,
            to_sql_version(instance.version)?
        ],
    )
    .map_err(map_write_error)?;
    Ok(())
}

pub(in crate::platform::auth) fn insert_sql_provisioned_identity(
    connection: &Connection,
    identity: &ProvisionedIdentityRecord,
) -> Result<(), AuthorizationStateError> {
    connection
    .execute(
        "INSERT INTO auth_provisioned_identities (identity_key_id, identity_public_key, principal_id, deployment_id, instance_id, kind, state, created_at, revoked_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            identity.identity_key_id,
            identity.identity_public_key,
            identity.principal_id,
            identity.deployment_id,
            identity.instance_id,
            encode_enum(identity.kind)?,
            encode_enum(identity.state)?,
            identity.created_at,
            identity.revoked_at
        ],
    )
    .map_err(map_write_error)?;
    Ok(())
}

pub(in crate::platform::auth) fn insert_sql_provisioning_secret(
    connection: &Connection,
    secret: &DeviceProvisioningSecretRecord,
) -> Result<(), AuthorizationStateError> {
    connection
    .execute(
        "INSERT INTO auth_device_provisioning_secrets (secret_id, instance_id, secret_hash, state, created_at, expires_at, consumed_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            secret.secret_id,
            secret.instance_id,
            secret.secret_hash,
            encode_enum(secret.state)?,
            secret.created_at,
            secret.expires_at,
            secret.consumed_at,
            to_sql_version(secret.version)?
        ],
    )
    .map_err(map_write_error)?;
    Ok(())
}

pub(in crate::platform::auth) fn load_provisioned_identity(
    connection: &Connection,
    identity_key_id: &str,
) -> Result<Option<ProvisionedIdentityRecord>, AuthorizationStateError> {
    let identity = connection
        .query_row(
        "SELECT identity_key_id, identity_public_key, principal_id, deployment_id, instance_id, kind, state, created_at, revoked_at FROM auth_provisioned_identities WHERE identity_key_id = ?1",
        [identity_key_id],
        |row| {
            Ok(ProvisionedIdentityRecord {
                identity_key_id: row.get(0)?,
                identity_public_key: row.get(1)?,
                principal_id: row.get(2)?,
                deployment_id: row.get(3)?,
                instance_id: row.get(4)?,
                kind: decode_enum(row.get(5)?)?,
                state: decode_enum(row.get(6)?)?,
                created_at: row.get(7)?,
                revoked_at: row.get(8)?,
            })
        },
        )
        .optional()
        .map_err(sql_error)?;
    if let Some(identity) = &identity {
        validate_provisioned_identity(identity)?;
    }
    Ok(identity)
}

pub(in crate::platform::auth) fn decode_provisioning_secret(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<DeviceProvisioningSecretRecord> {
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

pub(in crate::platform::auth) fn load_provisioning_secret_by_hash(
    connection: &Connection,
    secret_hash: &str,
) -> Result<Option<DeviceProvisioningSecretRecord>, AuthorizationStateError> {
    connection
    .query_row(
        "SELECT secret_id, instance_id, secret_hash, state, created_at, expires_at, consumed_at, version FROM auth_device_provisioning_secrets WHERE secret_hash = ?1",
        [secret_hash],
        decode_provisioning_secret,
    )
    .optional()
    .map_err(sql_error)
}

pub(in crate::platform::auth) fn load_activation_review(
    connection: &Connection,
    review_id: &str,
) -> Result<Option<DeviceActivationReviewRecord>, AuthorizationStateError> {
    connection
    .query_row(
        "SELECT review_id, principal_id, deployment_id, instance_id, request_digest, payload_json, state, requested_at, expires_at, activated_by_user_principal_id, decided_at, decided_by, reason, version FROM auth_device_activation_reviews WHERE review_id = ?1",
        [review_id],
        |row| {
            Ok(DeviceActivationReviewRecord {
                review_id: row.get(0)?,
                principal_id: row.get(1)?,
                deployment_id: row.get(2)?,
                instance_id: row.get(3)?,
                request_digest: row.get(4)?,
                payload: decode_json(row.get(5)?)?,
                state: decode_enum(row.get(6)?)?,
                requested_at: row.get(7)?,
                expires_at: row.get(8)?,
                activated_by_user_principal_id: row.get(9)?,
                decided_at: row.get(10)?,
                decided_by: row.get(11)?,
                reason: row.get(12)?,
                version: from_sql_version(row.get(13)?)?,
            })
        },
    )
    .optional()
    .map_err(sql_error)
}
