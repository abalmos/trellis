use serde_json::Value;

use super::fixtures::{
    assert_issuable_context_valid, evidence_scope, participant_fixture_for, session_public_key,
    test_session_creation, NOW,
};
use crate::platform::auth::application::repository::{
    AccountRepository, ActivationReviewCreation, ActivationReviewDecision,
    DeviceDelegationMutation, DeviceProvisioning, ProvisionedInstanceMutation,
    ProvisioningRepository, SessionRepository,
};
use crate::platform::auth::{
    AuthorityDecision, AuthorityEvidenceRepository, AuthorityKind, AuthorityRepository,
    AuthorityState, AuthorityTarget, AuthorizationStateError, AuthorizationStateService,
    ContextRepository, DeploymentAuthorityRecord, DeploymentRecord, DeviceActivationReviewRecord,
    DeviceActivationReviewState, DeviceDelegationRecord, DeviceDelegationState,
    DeviceProvisioningSecretRecord, DeviceRecord, DeviceState, IdempotencyResultRecord,
    MaterializationState, NewSession, PrincipalKind, PrincipalRecord, PrincipalState,
    ProvisioningSecretState, RuntimeInstanceRecord, RuntimeInstanceState, SessionRecord,
    SessionRuntimeBinding, SqliteAuthorizationStore,
};
use trellis_protocol::ParticipantKind;

#[tokio::test]
async fn sqlite_activation_review_expires_after_domain_ttl(
) -> Result<(), Box<dyn std::error::Error>> {
    let store = SqliteAuthorizationStore::open_in_memory()?;
    let fixture = participant_fixture_for(ParticipantKind::Device, "expiry.device")?;
    store
        .put_participant_binding(fixture.binding.clone())
        .await?;
    store
        .put_deployment_authority(
            DeploymentAuthorityRecord {
                authority_id: "dpa_expiry".to_owned(),
                deployment_id: "dep_expiry".to_owned(),
                participant_id: fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKind::Device,
                participant_artifact_digest: fixture.binding.artifact_digest.clone(),
                accepted_needs_digest: fixture.binding.needs_digest.clone(),
                desired_grant_set: fixture.all_grants,
                desired_capabilities: Vec::new(),
                state: AuthorityState::Accepted,
                version: 1,
                created_at: NOW,
                updated_at: NOW,
                expires_at: None,
                decision: None,
            },
            None,
        )
        .await?;
    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_expiry".to_owned(),
            participant_id: fixture.binding.participant_id,
            participant_kind: ParticipantKind::Device,
            active: true,
            expires_at: None,
        })
        .await?;
    store
        .provision_device(DeviceProvisioning {
            principal: PrincipalRecord {
                principal_id: "dev_expiry".to_owned(),
                kind: PrincipalKind::Device,
                state: PrincipalState::Active,
                created_at: NOW,
                updated_at: NOW,
                version: 1,
                disabled_at: None,
                revoked_at: None,
            },
            instance: RuntimeInstanceRecord {
                instance_id: "inst_expiry".to_owned(),
                deployment_id: "dep_expiry".to_owned(),
                principal_id: "dev_expiry".to_owned(),
                state: RuntimeInstanceState::Active,
                created_at: NOW,
                updated_at: NOW,
                version: 1,
            },
            device: DeviceRecord {
                principal_id: "dev_expiry".to_owned(),
                deployment_id: "dep_expiry".to_owned(),
                state: DeviceState::Pending,
                created_at: NOW,
                updated_at: NOW,
                version: 1,
            },
            identity: None,
            secret: DeviceProvisioningSecretRecord {
                secret_id: "dps_expiry".to_owned(),
                instance_id: "inst_expiry".to_owned(),
                secret_hash: super::fixtures::digest(90),
                state: ProvisioningSecretState::Pending,
                created_at: NOW,
                expires_at: NOW + crate::platform::auth::DEVICE_ACTIVATION_REVIEW_TTL_MS,
                consumed_at: None,
                version: 1,
            },
            idempotency: IdempotencyResultRecord {
                scope_key: super::fixtures::digest(91),
                purpose: "device.provision.expiry".to_owned(),
                signer_id: "usr_admin".to_owned(),
                request_id: "device-provision-expiry".to_owned(),
                request_digest: super::fixtures::digest(92),
                result: Value::Null,
                created_at: NOW,
                expires_at: NOW + crate::platform::auth::DEVICE_ACTIVATION_REVIEW_TTL_MS,
            },
            actions: Vec::new(),
        })
        .await?;
    let expires_at = NOW + crate::platform::auth::DEVICE_ACTIVATION_REVIEW_TTL_MS;
    store
        .create_activation_review(ActivationReviewCreation {
            review: DeviceActivationReviewRecord {
                review_id: "dar_expiry".to_owned(),
                principal_id: "dev_expiry".to_owned(),
                deployment_id: "dep_expiry".to_owned(),
                instance_id: "inst_expiry".to_owned(),
                request_digest: super::fixtures::digest(93),
                payload: Value::Null,
                state: DeviceActivationReviewState::Pending,
                requested_at: NOW,
                expires_at,
                activated_by_user_principal_id: None,
                decided_at: None,
                decided_by: None,
                reason: None,
                version: 1,
            },
            idempotency: IdempotencyResultRecord {
                scope_key: super::fixtures::digest(94),
                purpose: "device.review.expiry".to_owned(),
                signer_id: "usr_admin".to_owned(),
                request_id: "device-review-expiry".to_owned(),
                request_digest: super::fixtures::digest(95),
                result: Value::Null,
                created_at: NOW,
                expires_at,
            },
            actions: Vec::new(),
        })
        .await?;

    let expired = store.expire_due_activation_reviews(expires_at + 1).await?;

    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].state, DeviceActivationReviewState::Expired);
    assert_eq!(expired[0].version, 2);
    let expired_review = store
        .get_activation_review("dar_expiry")
        .await?
        .ok_or("missing expired review")?;
    assert_eq!(expired_review.state, DeviceActivationReviewState::Expired,);
    assert_eq!(
        store
            .decide_activation_review(ActivationReviewDecision {
                review_id: "dar_expiry".to_owned(),
                expected_version: expired_review.version,
                state: DeviceActivationReviewState::Approved,
                decided_at: expires_at + 2,
                decided_by: "usr_admin".to_owned(),
                reason: None,
                delegation: None,
                activate_device: false,
                idempotency: IdempotencyResultRecord {
                    scope_key: super::fixtures::digest(96),
                    purpose: "device.review.decide-expired".to_owned(),
                    signer_id: "usr_admin".to_owned(),
                    request_id: "device-review-decide-expired".to_owned(),
                    request_digest: super::fixtures::digest(97),
                    result: Value::Null,
                    created_at: expires_at + 2,
                    expires_at: expires_at + 1_000,
                },
                actions: Vec::new(),
            })
            .await,
        Err(AuthorizationStateError::StorageConflict),
    );
    assert_eq!(
        store
            .get_activation_review("dar_expiry")
            .await?
            .ok_or("missing expired review after decision conflict")?,
        expired_review,
    );
    Ok(())
}

pub(super) async fn exercise_deployed_principals(
    store: SqliteAuthorizationStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let service_fixture = participant_fixture_for(ParticipantKind::Service, "example.service")?;
    store
        .put_participant_binding(service_fixture.binding.clone())
        .await?;
    let service_principal = PrincipalRecord {
        principal_id: "svc_01".to_owned(),
        kind: PrincipalKind::Service,
        state: PrincipalState::Active,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
        disabled_at: None,
        revoked_at: None,
    };
    store.create_principal(service_principal.clone()).await?;
    let service_session = SessionRecord::from_new(NewSession {
        session_id: "ses_service".to_owned(),
        principal_id: service_principal.principal_id.clone(),
        principal_kind: PrincipalKind::Service,
        participant_id: service_fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKind::Service,
        participant_artifact_digest: service_fixture.binding.artifact_digest.clone(),
        participant_needs_digest: service_fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(9),
        inbox_prefix: "_INBOX.service".to_owned(),
        created_at: NOW,
        expires_at: Some(NOW + 1_000),
    })?;
    store
        .put_deployment_authority(
            DeploymentAuthorityRecord {
                authority_id: "dpa_service".to_owned(),
                deployment_id: "dep_service".to_owned(),
                participant_id: service_fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKind::Service,
                participant_artifact_digest: service_fixture.binding.artifact_digest.clone(),
                accepted_needs_digest: service_fixture.binding.needs_digest.clone(),
                desired_grant_set: service_fixture.all_grants.clone(),
                desired_capabilities: Vec::new(),
                state: AuthorityState::Accepted,
                version: 1,
                created_at: NOW,
                updated_at: NOW,
                expires_at: Some(NOW + 900),
                decision: Some(AuthorityDecision {
                    decided_at: NOW,
                    decided_by: "usr_admin".to_owned(),
                    reason: None,
                }),
            },
            None,
        )
        .await?;
    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_service".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKind::Service,
            active: true,
            expires_at: Some(NOW + 800),
        })
        .await?;
    store
        .put_runtime_instance(RuntimeInstanceRecord {
            instance_id: "inst_service".to_owned(),
            deployment_id: "dep_service".to_owned(),
            principal_id: service_principal.principal_id.clone(),
            state: RuntimeInstanceState::Active,
            created_at: NOW,
            updated_at: NOW,
            version: 1,
        })
        .await?;
    let service_runtime_binding = SessionRuntimeBinding {
        session_id: service_session.session_id.clone(),
        deployment_id: "dep_service".to_owned(),
        instance_id: "inst_service".to_owned(),
    };
    assert!(matches!(
        store
            .create_session(test_session_creation(service_session.clone(), None, None,))
            .await,
        Err(AuthorizationStateError::InvalidRecord(_))
    ));
    assert_eq!(store.get_session(&service_session.session_id).await?, None);
    store
        .create_session(test_session_creation(
            service_session.clone(),
            None,
            Some(service_runtime_binding.clone()),
        ))
        .await?;
    assert_eq!(
        store
            .get_session_runtime_binding(&service_session.session_id)
            .await?,
        Some(service_runtime_binding)
    );
    store
        .replace_dependency_evidence(
            evidence_scope(
                AuthorityKind::Deployment,
                "dpa_service",
                &service_fixture.binding,
            ),
            vec![service_fixture.required_dependency.clone()],
        )
        .await?;
    let facade = AuthorizationStateService::new(store.clone());
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_service")?,
            NOW + 10,
        )
        .await?;
    let service_state = facade
        .resolve_issuable_state(&service_session.session_id, NOW + 10)
        .await?;
    assert_eq!(service_state.deployment_id.as_deref(), Some("dep_service"));
    assert_eq!(service_state.instance_id.as_deref(), Some("inst_service"));
    assert_eq!(
        service_state.effective_authority_expires_at,
        Some(NOW + 800)
    );
    assert_issuable_context_valid(&service_state)?;
    assert_eq!(
        facade
            .resolve_issuable_state(&service_session.session_id, NOW + 800)
            .await,
        Err(AuthorizationStateError::DeploymentInactive)
    );

    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_service".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKind::Service,
            active: true,
            expires_at: None,
        })
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&service_session.session_id, NOW + 20)
            .await,
        Err(AuthorizationStateError::MaterializationStale)
    );
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_service")?,
            NOW + 20,
        )
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&service_session.session_id, NOW + 20)
            .await?
            .effective_authority_expires_at,
        Some(NOW + 900)
    );

    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_service".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKind::Service,
            active: true,
            expires_at: Some(NOW + 800),
        })
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&service_session.session_id, NOW + 30)
            .await,
        Err(AuthorizationStateError::MaterializationStale)
    );
    let mut service_authority = store
        .get_deployment_authority("dep_service", &service_fixture.binding.participant_id)
        .await?
        .ok_or("missing service authority")?;
    service_authority.expires_at = None;
    service_authority.updated_at = NOW + 30;
    service_authority.version = 2;
    store
        .put_deployment_authority(service_authority, Some(1))
        .await?;
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_service")?,
            NOW + 30,
        )
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&service_session.session_id, NOW + 30)
            .await?
            .effective_authority_expires_at,
        Some(NOW + 800)
    );

    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_service".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKind::Service,
            active: false,
            expires_at: Some(NOW + 700),
        })
        .await?;
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_service")?,
            NOW + 40,
        )
        .await?;
    let unavailable = store
        .get_materialized_authority(AuthorityKind::Deployment, "dpa_service")
        .await?
        .ok_or("missing unavailable service materialization")?;
    assert_eq!(
        unavailable.authority.state,
        MaterializationState::Unavailable
    );
    assert_eq!(unavailable.authority.expires_at, Some(NOW + 700));
    let unavailable_version = unavailable.authority.materialization_version;
    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_service".to_owned(),
            participant_id: service_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKind::Service,
            active: false,
            expires_at: Some(NOW + 600),
        })
        .await?;
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_service")?,
            NOW + 50,
        )
        .await?;
    let unavailable = store
        .get_materialized_authority(AuthorityKind::Deployment, "dpa_service")
        .await?
        .ok_or("missing updated unavailable service materialization")?;
    assert_eq!(unavailable.authority.expires_at, Some(NOW + 600));
    assert_eq!(
        unavailable.authority.materialization_version,
        unavailable_version + 1
    );

    let device_fixture = participant_fixture_for(ParticipantKind::Device, "example.device")?;
    store
        .put_participant_binding(device_fixture.binding.clone())
        .await?;
    store
        .put_deployment_authority(
            DeploymentAuthorityRecord {
                authority_id: "dpa_device".to_owned(),
                deployment_id: "dep_device".to_owned(),
                participant_id: device_fixture.binding.participant_id.clone(),
                participant_kind: ParticipantKind::Device,
                participant_artifact_digest: device_fixture.binding.artifact_digest.clone(),
                accepted_needs_digest: device_fixture.binding.needs_digest.clone(),
                desired_grant_set: device_fixture.all_grants.clone(),
                desired_capabilities: Vec::new(),
                state: AuthorityState::Accepted,
                version: 1,
                created_at: NOW,
                updated_at: NOW,
                expires_at: None,
                decision: Some(AuthorityDecision {
                    decided_at: NOW,
                    decided_by: "usr_admin".to_owned(),
                    reason: None,
                }),
            },
            None,
        )
        .await?;
    store
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: "dep_device".to_owned(),
            participant_id: device_fixture.binding.participant_id.clone(),
            participant_kind: ParticipantKind::Device,
            active: true,
            expires_at: Some(NOW + 750),
        })
        .await?;
    let proof = |byte: u8, purpose: &str| IdempotencyResultRecord {
        scope_key: super::fixtures::digest(byte),
        purpose: purpose.to_owned(),
        signer_id: "usr_admin".to_owned(),
        request_id: format!("{purpose}-{byte}"),
        request_digest: super::fixtures::digest(byte + 1),
        result: Value::Null,
        created_at: NOW,
        expires_at: NOW + 10_000,
    };
    store
        .provision_device(DeviceProvisioning {
            principal: PrincipalRecord {
                principal_id: "dev_01".to_owned(),
                kind: PrincipalKind::Device,
                state: PrincipalState::Active,
                created_at: NOW,
                updated_at: NOW,
                version: 1,
                disabled_at: None,
                revoked_at: None,
            },
            instance: RuntimeInstanceRecord {
                instance_id: "inst_device".to_owned(),
                deployment_id: "dep_device".to_owned(),
                principal_id: "dev_01".to_owned(),
                state: RuntimeInstanceState::Active,
                created_at: NOW,
                updated_at: NOW,
                version: 1,
            },
            device: DeviceRecord {
                principal_id: "dev_01".to_owned(),
                deployment_id: "dep_device".to_owned(),
                state: DeviceState::Pending,
                created_at: NOW,
                updated_at: NOW,
                version: 1,
            },
            identity: None,
            secret: DeviceProvisioningSecretRecord {
                secret_id: "dps_device".to_owned(),
                instance_id: "inst_device".to_owned(),
                secret_hash: super::fixtures::digest(60),
                state: ProvisioningSecretState::Pending,
                created_at: NOW,
                expires_at: NOW + 1_000,
                consumed_at: None,
                version: 1,
            },
            idempotency: proof(60, "device.provision"),
            actions: Vec::new(),
        })
        .await?;
    let review = DeviceActivationReviewRecord {
        review_id: "dar_device".to_owned(),
        principal_id: "dev_01".to_owned(),
        deployment_id: "dep_device".to_owned(),
        instance_id: "inst_device".to_owned(),
        request_digest: super::fixtures::digest(61),
        payload: Value::Null,
        state: DeviceActivationReviewState::Pending,
        requested_at: NOW,
        expires_at: NOW + 1_000,
        activated_by_user_principal_id: None,
        decided_at: None,
        decided_by: None,
        reason: None,
        version: 1,
    };
    store
        .create_activation_review(ActivationReviewCreation {
            review,
            idempotency: proof(61, "device.review.create"),
            actions: Vec::new(),
        })
        .await?;
    store
        .decide_activation_review(ActivationReviewDecision {
            review_id: "dar_device".to_owned(),
            expected_version: 1,
            state: DeviceActivationReviewState::Approved,
            decided_at: NOW + 1,
            decided_by: "usr_admin".to_owned(),
            reason: None,
            delegation: Some(DeviceDelegationRecord {
                principal_id: "dev_01".to_owned(),
                deployment_id: "dep_device".to_owned(),
                required: true,
                state: DeviceDelegationState::Active,
                expires_at: Some(NOW + 700),
            }),
            activate_device: true,
            idempotency: proof(62, "device.review.decide"),
            actions: Vec::new(),
        })
        .await?;
    let device_session = SessionRecord::from_new(NewSession {
        session_id: "ses_device".to_owned(),
        principal_id: "dev_01".to_owned(),
        principal_kind: PrincipalKind::Device,
        participant_id: device_fixture.binding.participant_id.clone(),
        participant_kind: ParticipantKind::Device,
        participant_artifact_digest: device_fixture.binding.artifact_digest.clone(),
        participant_needs_digest: device_fixture.binding.needs_digest.clone(),
        session_public_key: session_public_key(10),
        inbox_prefix: "_INBOX.device".to_owned(),
        created_at: NOW,
        expires_at: None,
    })?;
    let device_runtime_binding = SessionRuntimeBinding {
        session_id: device_session.session_id.clone(),
        deployment_id: "dep_device".to_owned(),
        instance_id: "inst_device".to_owned(),
    };
    store
        .create_session(test_session_creation(
            device_session.clone(),
            None,
            Some(device_runtime_binding.clone()),
        ))
        .await?;
    assert_eq!(
        store
            .get_session_runtime_binding(&device_session.session_id)
            .await?,
        Some(device_runtime_binding)
    );
    store
        .replace_dependency_evidence(
            evidence_scope(
                AuthorityKind::Deployment,
                "dpa_device",
                &device_fixture.binding,
            ),
            vec![device_fixture.required_dependency.clone()],
        )
        .await?;
    facade
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, "dpa_device")?,
            NOW + 10,
        )
        .await?;
    let device_state = facade
        .resolve_issuable_state(&device_session.session_id, NOW + 10)
        .await?;
    assert_eq!(device_state.deployment_id.as_deref(), Some("dep_device"));
    assert_eq!(device_state.instance_id.as_deref(), Some("inst_device"));
    assert_eq!(device_state.delegation_expires_at, Some(NOW + 700));
    assert_issuable_context_valid(&device_state)?;

    let mut disabled_instance = store
        .get_runtime_instance("inst_device")
        .await?
        .ok_or("device instance missing")?;
    disabled_instance.updated_at = NOW + 20;
    disabled_instance.version = 2;
    let mut disabled_device = store
        .get_device("dev_01", "dep_device")
        .await?
        .ok_or("device record missing")?;
    disabled_device.state = DeviceState::Disabled;
    disabled_device.updated_at = NOW + 20;
    disabled_device.version = 3;
    store
        .mutate_provisioned_instance(ProvisionedInstanceMutation {
            instance: disabled_instance,
            device: Some(disabled_device),
            identity: None,
            expected_version: 2,
            idempotency: proof(63, "device.disable"),
            actions: Vec::new(),
        })
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&device_session.session_id, NOW + 20)
            .await,
        Err(AuthorizationStateError::DeviceInactive)
    );
    let mut active_instance = store
        .get_runtime_instance("inst_device")
        .await?
        .ok_or("device instance missing after disable")?;
    active_instance.updated_at = NOW + 21;
    active_instance.version = 3;
    let mut active_device = store
        .get_device("dev_01", "dep_device")
        .await?
        .ok_or("device record missing after disable")?;
    active_device.state = DeviceState::Active;
    active_device.updated_at = NOW + 21;
    active_device.version = 4;
    store
        .mutate_provisioned_instance(ProvisionedInstanceMutation {
            instance: active_instance,
            device: Some(active_device),
            identity: None,
            expected_version: 3,
            idempotency: proof(64, "device.enable"),
            actions: Vec::new(),
        })
        .await?;
    let mut delegated_device = store
        .get_device("dev_01", "dep_device")
        .await?
        .ok_or("device record missing after enable")?;
    delegated_device.updated_at = NOW + 22;
    delegated_device.version = 5;
    store
        .mutate_device_delegation(DeviceDelegationMutation {
            device: delegated_device,
            delegation: DeviceDelegationRecord {
                principal_id: "dev_01".to_owned(),
                deployment_id: "dep_device".to_owned(),
                required: true,
                state: DeviceDelegationState::Missing,
                expires_at: Some(NOW + 700),
            },
            expected_version: 4,
            idempotency: proof(65, "device.delegation.missing"),
            actions: Vec::new(),
        })
        .await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&device_session.session_id, NOW + 20)
            .await,
        Err(AuthorizationStateError::ActivationMissing)
    );
    Ok(())
}
