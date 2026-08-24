use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::super::authority::{
    validate_dependency_evidence, validate_deployment_evidence, validate_device,
    validate_device_delegation, validate_resource_evidence, validate_runtime_instance,
    validate_session_runtime_binding, AuthorityEvidenceRepository,
};
use super::super::{
    ActiveProviderEvidence, AuthorityEvidenceScope, AuthorityKind, AuthorityTarget,
    AuthorizationStateError, DelegationEvidence, DependencyEvidence, DeploymentRecord,
    DesiredAuthorityRecord, DeviceDelegationRecord, DeviceDelegationState, DeviceEvidence,
    DeviceRecord, DeviceState, ParticipantBindingRecord, PrincipalKind, ResourceBindingEvidence,
    RuntimeEvidence, RuntimeInstanceRecord, RuntimeInstanceState, ServiceEvidence,
    SessionRuntimeBinding,
};
use super::authority::{
    decode_deployment_authority, decode_identity_authority, load_deployment_authority,
};
use super::common::{
    decode_enum, encode_enum, encode_json, from_sql_version, map_write_error, sql_error,
    to_sql_version,
};
use super::contexts::{decode_dependency, decode_resource};
use super::principals::load_principal;
use super::sessions::load_session;
use super::validation::next_version;
use super::SqliteAuthorizationStore;

pub(in crate::platform::auth) fn decode_participant_binding(
    row: &Row<'_>,
) -> rusqlite::Result<ParticipantBindingRecord> {
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

#[async_trait]
impl AuthorityEvidenceRepository for SqliteAuthorizationStore {
    async fn list_active_provider_evidence(
        &self,
        now: i64,
    ) -> Result<Vec<ActiveProviderEvidence>, AuthorizationStateError> {
        self.run_read(move |connection| {
            let keys = {
                let mut statement = connection
                    .prepare(
                        "SELECT deployment_id, participant_id
                         FROM auth_deployment_authorities
                         WHERE state = 'accepted'
                         ORDER BY deployment_id, participant_id",
                    )
                    .map_err(sql_error)?;
                let keys = statement
                    .query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })
                    .map_err(sql_error)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(sql_error)?;
                keys
            };
            let mut evidence = Vec::new();
            for (deployment_id, participant_id) in keys {
                let Some(authority) =
                    load_deployment_authority(connection, &deployment_id, &participant_id)?
                else {
                    continue;
                };
                if authority
                    .expires_at
                    .is_some_and(|expires_at| expires_at <= now)
                    || !load_deployment(connection, &deployment_id)?.is_some_and(|deployment| {
                        deployment.active
                            && deployment
                                .expires_at
                                .is_none_or(|expires_at| expires_at > now)
                    })
                {
                    continue;
                }
                let instance_id = connection
                    .query_row(
                        "SELECT instance_id FROM auth_instances
                         WHERE deployment_id = ?1 AND state = 'active'
                         ORDER BY instance_id LIMIT 1",
                        params![deployment_id],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()
                    .map_err(sql_error)?;
                let Some(instance) = instance_id
                    .map(|instance_id| load_runtime_instance(connection, &instance_id))
                    .transpose()?
                    .flatten()
                else {
                    continue;
                };
                let Some(binding) = load_participant_binding(
                    connection,
                    &authority.participant_id,
                    &authority.participant_artifact_digest,
                )?
                else {
                    continue;
                };
                evidence.push(ActiveProviderEvidence {
                    authority,
                    instance,
                    binding,
                });
            }
            Ok(evidence)
        })
        .await
    }

    async fn list_runtime_instances(
        &self,
    ) -> Result<Vec<RuntimeInstanceRecord>, AuthorizationStateError> {
        self.run_read(move |connection| {
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
        self.run_read(move |connection| {
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
        self.run_read(move |connection| load_deployment(connection, &deployment_id))
            .await
    }

    async fn put_deployment_evidence(
        &self,
        deployment: DeploymentRecord,
    ) -> Result<(), AuthorizationStateError> {
        self.run(move |connection| put_sql_deployment_evidence(connection, deployment))
            .await
    }

    async fn get_runtime_instance(
        &self,
        instance_id: &str,
    ) -> Result<Option<RuntimeInstanceRecord>, AuthorizationStateError> {
        let instance_id = instance_id.to_owned();
        self.run_read(move |connection| load_runtime_instance(connection, &instance_id))
            .await
    }

    async fn put_runtime_instance(
        &self,
        instance: RuntimeInstanceRecord,
    ) -> Result<(), AuthorizationStateError> {
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
        self.run_read(move |connection| load_device(connection, &principal_id, &deployment_id))
            .await
    }

    async fn get_device_delegation(
        &self,
        principal_id: &str,
        deployment_id: &str,
    ) -> Result<Option<DeviceDelegationRecord>, AuthorizationStateError> {
        let principal_id = principal_id.to_owned();
        let deployment_id = deployment_id.to_owned();
        self.run_read(move |connection| {
            load_device_delegation(connection, &principal_id, &deployment_id)
        })
        .await
    }

    async fn get_session_runtime_binding(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionRuntimeBinding>, AuthorizationStateError> {
        let session_id = session_id.to_owned();
        self.run_read(move |connection| load_session_runtime_binding(connection, &session_id))
            .await
    }

    async fn replace_dependency_evidence(
        &self,
        scope: AuthorityEvidenceScope,
        evidence: Vec<DependencyEvidence>,
    ) -> Result<(), AuthorizationStateError> {
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

    async fn replace_resource_evidence(
        &self,
        scope: AuthorityEvidenceScope,
        evidence: Vec<ResourceBindingEvidence>,
    ) -> Result<(), AuthorizationStateError> {
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

pub(in crate::platform::auth) fn put_sql_deployment_evidence(
    connection: &Connection,
    deployment: DeploymentRecord,
) -> Result<(), AuthorizationStateError> {
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

pub(in crate::platform::auth) fn load_runtime_instance(
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
    .and_then(|instance| {
        instance.map_or(Ok(None), |instance| {
            validate_runtime_instance(&instance)?;
            Ok(Some(instance))
        })
    })
}

pub(in crate::platform::auth) fn load_device(
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
    .and_then(|device| {
        device.map_or(Ok(None), |device| {
            validate_device(&device)?;
            Ok(Some(device))
        })
    })
}

pub(in crate::platform::auth) fn load_device_delegation(
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
        .and_then(|delegation| {
            delegation.map_or(Ok(None), |delegation| {
                validate_device_delegation(&delegation)?;
                Ok(Some(delegation))
            })
        })
}

pub(in crate::platform::auth) fn load_session_runtime_binding(
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
        .and_then(|binding| {
            binding.map_or(Ok(None), |binding| {
                validate_session_runtime_binding(&binding)?;
                Ok(Some(binding))
            })
        })
}

pub(in crate::platform::auth) fn validate_sql_runtime_instance_relationships(
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
            trellis_protocol::ParticipantKind::Service
        ) | (
            PrincipalKind::Device,
            trellis_protocol::ParticipantKind::Device
        )
    );
    if !kind_matches {
        return Err(AuthorizationStateError::InvalidRecord(
            "runtime instance principal kind does not match deployment".to_owned(),
        ));
    }
    Ok(())
}

pub(in crate::platform::auth) fn validate_sql_device_relationships(
    connection: &Connection,
    device: &DeviceRecord,
) -> Result<(), AuthorizationStateError> {
    let deployment = load_deployment(connection, &device.deployment_id)?
        .ok_or(AuthorizationStateError::DeploymentInactive)?;
    let principal = load_principal(connection, &device.principal_id)?
        .ok_or(AuthorizationStateError::PrincipalMissing)?;
    if principal.kind != PrincipalKind::Device
        || deployment.participant_kind != trellis_protocol::ParticipantKind::Device
    {
        return Err(AuthorizationStateError::InvalidRecord(
            "device evidence requires a device principal and deployment".to_owned(),
        ));
    }
    Ok(())
}

pub(in crate::platform::auth) fn validate_sql_session_runtime_binding_relationships(
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

pub(in crate::platform::auth) fn load_runtime_evidence(
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

pub(in crate::platform::auth) fn load_participant_binding(
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

pub(in crate::platform::auth) fn load_deployment(
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
        .and_then(|deployment| {
            deployment.map_or(Ok(None), |deployment| {
                validate_deployment_evidence(&deployment)?;
                Ok(Some(deployment))
            })
        })
}

pub(in crate::platform::auth) fn load_desired_authority(
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

pub(in crate::platform::auth) fn load_dependency_evidence(
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
    validate_dependency_evidence(&records)?;
    Ok(records)
}

pub(in crate::platform::auth) fn load_resource_evidence(
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
    validate_resource_evidence(&records)?;
    Ok(records)
}

pub(in crate::platform::auth) fn load_evidence_scope(
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

pub(in crate::platform::auth) fn validate_sql_evidence_scope(
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
