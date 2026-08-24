use serde_json::json;
use trellis_protocol::ParticipantKind;

use super::fixtures::{digest, participant_fixture_for, session_public_key, NOW};
use crate::platform::auth::application::repository::{
    DeviceProvisioning, DeviceProvisioningSecretConsumption, IdempotentOutcome,
    ProvisioningRepository, ServiceIdentityProvisioning,
};
use crate::platform::auth::{
    AuthorityEvidenceRepository, AuthorityRepository, AuthorizationStateError, DeploymentRecord,
    DeviceProvisioningSecretRecord, DeviceRecord, DeviceState, IdempotencyResultRecord,
    PrincipalKind, PrincipalRecord, PrincipalState, ProvisionedIdentityKind,
    ProvisionedIdentityRecord, ProvisionedIdentityState, ProvisioningSecretState,
    RuntimeInstanceRecord, RuntimeInstanceState, SqliteAuthorizationStore,
};

pub(super) async fn exercise_provisioning(
    store: SqliteAuthorizationStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let proof = |byte: u8, purpose: &str| IdempotencyResultRecord {
        scope_key: digest(byte),
        purpose: purpose.to_owned(),
        signer_id: "signer_companion".to_owned(),
        request_id: format!("request_{byte}"),
        request_digest: digest(byte + 1),
        result: json!({ "request": byte }),
        created_at: NOW,
        expires_at: NOW + 1_000,
    };
    let device_fixture = participant_fixture_for(ParticipantKind::Device, "example.device")?;
    store
        .put_participant_binding(device_fixture.binding.clone())
        .await?;
    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_companion".to_owned(),
            participant_id: device_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKind::Device,
            active: true,
            expires_at: None,
        })
        .await?;
    let device_principal = PrincipalRecord {
        principal_id: "dev_companion".to_owned(),
        kind: PrincipalKind::Device,
        state: PrincipalState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
        disabled_at: None,
        revoked_at: None,
    };
    let device_instance = RuntimeInstanceRecord {
        instance_id: "inst_companion".to_owned(),
        deployment_id: "dep_companion".to_owned(),
        principal_id: device_principal.principal_id.clone(),
        state: RuntimeInstanceState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    let device = DeviceRecord {
        principal_id: device_principal.principal_id.clone(),
        deployment_id: device_instance.deployment_id.clone(),
        state: DeviceState::Pending,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    let secret = DeviceProvisioningSecretRecord {
        secret_id: "secret_1".to_owned(),
        instance_id: device_instance.instance_id.clone(),
        secret_hash: digest(50),
        state: ProvisioningSecretState::Pending,
        created_at: NOW,
        expires_at: NOW + 100,
        consumed_at: None,
        version: 1,
    };
    assert!(matches!(
        store
            .provision_device(DeviceProvisioning {
                principal: device_principal.clone(),
                instance: device_instance.clone(),
                device: device.clone(),
                identity: None,
                secret: secret.clone(),
                idempotency: proof(46, "device.provision"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    let provisioned = ProvisionedIdentityRecord {
        identity_key_id: crate::platform::auth::domain::validate_ed25519_public_key(
            "identityPublicKey",
            &session_public_key(50),
        )?,
        identity_public_key: session_public_key(50),
        principal_id: device_principal.principal_id.clone(),
        deployment_id: device_instance.deployment_id.clone(),
        instance_id: device_instance.instance_id.clone(),
        kind: ProvisionedIdentityKind::Device,
        state: ProvisionedIdentityState::Active,
        created_at: NOW + 1,
        revoked_at: None,
    };
    assert!(matches!(
        store
            .consume_device_provisioning_secret(DeviceProvisioningSecretConsumption {
                secret_hash: secret.secret_hash.clone(),
                expected_version: 1,
                identity: provisioned.clone(),
                consumed_at: NOW + 1,
                idempotency: proof(48, "device-secret.consume"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store
            .get_provisioned_identity(&provisioned.identity_key_id)
            .await?,
        Some(provisioned.clone())
    );

    let second_principal = PrincipalRecord {
        principal_id: "dev_companion_2".to_owned(),
        ..device_principal.clone()
    };
    let second_instance = RuntimeInstanceRecord {
        instance_id: "inst_companion_2".to_owned(),
        principal_id: second_principal.principal_id.clone(),
        ..device_instance.clone()
    };
    let second_device = DeviceRecord {
        principal_id: second_principal.principal_id.clone(),
        ..device.clone()
    };
    let second_secret = DeviceProvisioningSecretRecord {
        secret_id: "secret_2".to_owned(),
        instance_id: second_instance.instance_id.clone(),
        secret_hash: digest(52),
        ..secret.clone()
    };
    store
        .provision_device(DeviceProvisioning {
            principal: second_principal.clone(),
            instance: second_instance.clone(),
            device: second_device,
            identity: None,
            secret: second_secret.clone(),
            idempotency: proof(52, "device.provision.second"),
            actions: Vec::new(),
        })
        .await?;
    let reassignment_proof = proof(54, "device-secret.consume.second");
    let reassigned = ProvisionedIdentityRecord {
        principal_id: second_principal.principal_id.clone(),
        instance_id: second_instance.instance_id.clone(),
        ..provisioned.clone()
    };
    assert_eq!(
        store
            .consume_device_provisioning_secret(DeviceProvisioningSecretConsumption {
                secret_hash: second_secret.secret_hash.clone(),
                expected_version: 1,
                identity: reassigned,
                consumed_at: NOW + 2,
                idempotency: reassignment_proof.clone(),
                actions: Vec::new(),
            })
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );
    let second_identity = ProvisionedIdentityRecord {
        identity_key_id: crate::platform::auth::domain::validate_ed25519_public_key(
            "identityPublicKey",
            &session_public_key(51),
        )?,
        identity_public_key: session_public_key(51),
        principal_id: second_principal.principal_id,
        deployment_id: second_instance.deployment_id.clone(),
        instance_id: second_instance.instance_id,
        kind: ProvisionedIdentityKind::Device,
        state: ProvisionedIdentityState::Active,
        created_at: NOW + 2,
        revoked_at: None,
    };
    assert!(matches!(
        store
            .consume_device_provisioning_secret(DeviceProvisioningSecretConsumption {
                secret_hash: second_secret.secret_hash,
                expected_version: 1,
                identity: second_identity,
                consumed_at: NOW + 2,
                idempotency: reassignment_proof,
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));

    let service_fixture = participant_fixture_for(ParticipantKind::Service, "example.service")?;
    store
        .put_participant_binding(service_fixture.binding.clone())
        .await?;
    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_service_companion".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKind::Service,
            active: true,
            expires_at: None,
        })
        .await?;
    let service_principal = PrincipalRecord {
        principal_id: "svc_companion".to_owned(),
        kind: PrincipalKind::Service,
        state: PrincipalState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
        disabled_at: None,
        revoked_at: None,
    };
    let service_instance = RuntimeInstanceRecord {
        instance_id: "inst_service_companion".to_owned(),
        deployment_id: "dep_service_companion".to_owned(),
        principal_id: service_principal.principal_id.clone(),
        state: RuntimeInstanceState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    };
    let service_identity = ProvisionedIdentityRecord {
        identity_key_id: crate::platform::auth::domain::validate_ed25519_public_key(
            "identityPublicKey",
            &session_public_key(52),
        )?,
        identity_public_key: session_public_key(52),
        principal_id: service_principal.principal_id.clone(),
        deployment_id: service_instance.deployment_id.clone(),
        instance_id: service_instance.instance_id.clone(),
        kind: ProvisionedIdentityKind::Service,
        state: ProvisionedIdentityState::Active,
        created_at: NOW,
        revoked_at: None,
    };
    assert!(matches!(
        store
            .provision_service_identity(ServiceIdentityProvisioning {
                principal: service_principal,
                instance: service_instance,
                identity: service_identity.clone(),
                idempotency: proof(56, "service.provision"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store
            .get_provisioned_identity(&service_identity.identity_key_id)
            .await?,
        Some(service_identity)
    );
    Ok(())
}
