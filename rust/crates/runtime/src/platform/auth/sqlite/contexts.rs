use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::super::authority::ContextRepository;
use super::super::authority::{
    IssuanceConnection, IssuanceCredential, IssuanceCredentialRecord, IssuanceSnapshot,
};
use super::super::{AuthorizationStateError, GrantOwnerKind, ResourceBindingEvidence};
use super::common::{decode_enum, decode_json, encode_enum, sql_error, to_sql_version};
use super::evidence::load_deployment;
use super::principals::load_principal;
use super::sessions::load_session;
use super::SqliteAuthorizationStore;

impl SqliteAuthorizationStore {
    pub(crate) async fn get_resource_bindings(
        &self,
        owner_kind: GrantOwnerKind,
        owner_id: String,
        participant_id: String,
        installed_revision: u64,
    ) -> Result<Vec<ResourceBindingEvidence>, AuthorizationStateError> {
        self.run_read(move |connection| {
            load_resource_bindings(
                connection,
                owner_kind,
                &owner_id,
                &participant_id,
                installed_revision,
            )
        })
        .await
    }
}

#[async_trait]
impl ContextRepository for SqliteAuthorizationStore {
    async fn load_issuance_snapshot(
        &self,
        request: &IssuanceConnection,
    ) -> Result<IssuanceSnapshot, AuthorizationStateError> {
        let request = request.clone();
        self.run_read(move |connection| {
            let transaction = connection.transaction().map_err(sql_error)?;
            let snapshot = sqlite_issuance_snapshot(&transaction, &request)?;
            transaction.commit().map_err(sql_error)?;
            Ok(snapshot)
        })
        .await
    }
}

pub(in crate::platform::auth) fn sqlite_issuance_snapshot(
    connection: &Connection,
    request: &IssuanceConnection,
) -> Result<IssuanceSnapshot, AuthorizationStateError> {
    let connection_id = request.connection_id.parse::<ulid::Ulid>().map_err(|_| {
        AuthorizationStateError::InvalidRecord("connectionId must be a ULID".to_owned())
    })?;
    if connection_id.to_string() != request.connection_id {
        return Err(AuthorizationStateError::InvalidRecord(
            "connectionId must be canonical".to_owned(),
        ));
    }
    super::super::domain::validate_ed25519_public_key("sessionKey", &request.session_public_key)?;
    let (credential, principal_id, owner_kind, owner_id, participant_id) = match &request.credential
    {
        IssuanceCredential::Login(id) => {
            let login =
                load_session(connection, id)?.ok_or(AuthorizationStateError::SessionMissing)?;
            (
                IssuanceCredentialRecord::Login(login.clone()),
                login.principal_id.clone(),
                super::super::GrantOwnerKind::User,
                login.principal_id,
                login.participant_id,
            )
        }
        IssuanceCredential::Native(id) => {
            let identity = super::provisioning::load_provisioned_identity(connection, id)?
                .ok_or(AuthorizationStateError::IdentityMissing)?;
            let instance =
                super::evidence::load_runtime_instance(connection, &identity.instance_id)?
                    .ok_or(AuthorizationStateError::InstanceInactive)?;
            let deployment = load_deployment(connection, &identity.deployment_id)?
                .ok_or(AuthorizationStateError::DeploymentInactive)?;
            let device = super::evidence::load_device(
                connection,
                &identity.principal_id,
                &identity.deployment_id,
            )?;
            let delegation = super::evidence::load_device_delegation(
                connection,
                &identity.principal_id,
                &identity.deployment_id,
            )?;
            let principal_id = identity.principal_id.clone();
            let owner_id = deployment.deployment_id.clone();
            let participant_id = deployment.participant_id.clone();
            (
                IssuanceCredentialRecord::Native {
                    identity,
                    instance,
                    deployment,
                    device,
                    delegation,
                },
                principal_id,
                super::super::GrantOwnerKind::Deployment,
                owner_id,
                participant_id,
            )
        }
    };
    let principal = load_principal(connection, &principal_id)?
        .ok_or(AuthorizationStateError::PrincipalMissing)?;
    let binding =
        super::grants::load_grant_binding(connection, owner_kind, &owner_id, &participant_id)?
            .ok_or(AuthorizationStateError::NotAuthorized)?;
    let (_, participant) = super::grants::load_installed_participant(
        connection,
        &participant_id,
        Some(binding.installed_revision),
    )?
    .ok_or(AuthorizationStateError::ParticipantMissing)?;
    let resources = load_resource_bindings(
        connection,
        owner_kind,
        &owner_id,
        &participant_id,
        binding.installed_revision,
    )?;
    let issuer = connection.query_row(
        "SELECT key_id, public_key FROM auth_authorization_issuers WHERE is_current = 1 AND revoked_at IS NULL",
        [], |row| Ok(trellis_protocol::AuthorizationIssuerKey {
            key_id: row.get(0)?, public_key: row.get(1)?, state: trellis_protocol::AuthorizationIssuerState::Active,
        }),
    ).optional().map_err(sql_error)?.ok_or(AuthorizationStateError::IssuerMissing)?;
    issuer
        .verifying_key()
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    Ok(IssuanceSnapshot {
        connection: request.clone(),
        credential,
        principal,
        binding,
        participant,
        resources,
        issuer,
    })
}

pub(in crate::platform::auth) fn load_resource_bindings(
    connection: &Connection,
    owner_kind: super::super::GrantOwnerKind,
    owner_id: &str,
    participant_id: &str,
    installed_revision: u64,
) -> Result<Vec<ResourceBindingEvidence>, AuthorizationStateError> {
    let mut statement = connection
        .prepare(
            "SELECT resource_kind, local_name, binding_id, participant_id, provider_identity,
                state, materialized_at, error FROM auth_resource_binding_evidence
         WHERE owner_kind = ?1 AND owner_id = ?2 AND participant_id = ?3 AND installed_revision = ?4
         ORDER BY resource_kind, local_name",
        )
        .map_err(sql_error)?;
    let resources = statement
        .query_map(
            params![
                encode_enum(owner_kind)?,
                owner_id,
                participant_id,
                to_sql_version(installed_revision)?
            ],
            decode_resource,
        )
        .map_err(sql_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(sql_error)?;
    Ok(resources)
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
