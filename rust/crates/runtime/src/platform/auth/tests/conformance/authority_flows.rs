use std::collections::BTreeMap;

use serde_json::{json, Value};
use trellis_protocol::{parse_api, ParticipantKind};

use super::fixtures::{digest, participant_fixture_for, NOW};
use crate::platform::auth::application::repository::{
    AccountRepository, ActivationReviewCreation, ActivationReviewDecision,
    AuthorityProposalCreation, AuthorityProposalDecision, IdempotentOutcome, OutboxRepository,
    ProvisioningRepository,
};
use crate::platform::auth::{
    deployment_authority_id, AuthService, AuthServiceConfig, AuthorityDecision,
    AuthorityDecisionOutcome, AuthorityDecisionRecord, AuthorityEvidenceRepository, AuthorityKind,
    AuthorityProposalKind, AuthorityProposalRecord, AuthorityProposalState, AuthorityRepository,
    AuthorityState, AuthorizationStateError, DeploymentAuthorityRecord, DesiredAuthorityRecord,
    DeviceActivationReviewRecord, DeviceActivationReviewState, DeviceRecord, DeviceState,
    IdempotencyResultRecord, PostCommitActionKind, PostCommitActionRecord,
    PresentDeploymentAuthorityInput, SqliteAuthorizationStore,
};

pub(super) async fn exercise_authority_flows(
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
    let action = |byte: u8, event: &str| PostCommitActionRecord {
        predecessor_action_id: None,
        action_id: digest(byte),
        kind: PostCommitActionKind::Event,
        payload: json!({ "event": event }),
        created_at: NOW,
        attempts: 0,
        next_attempt_at: NOW,
        claimed_until: None,
        last_error: None,
    };
    let device_fixture = participant_fixture_for(ParticipantKind::Device, "example.device")?;
    let service_fixture = participant_fixture_for(ParticipantKind::Service, "example.service")?;
    let device_principal = store
        .get_principal("dev_companion")
        .await?
        .ok_or("device principal missing")?;
    let device_instance = store
        .get_runtime_instance("inst_companion")
        .await?
        .ok_or("device instance missing")?;
    let device = store
        .get_device(
            &device_principal.principal_id,
            &device_instance.deployment_id,
        )
        .await?
        .ok_or("device missing")?;
    let admin_id = "usr_first_admin".to_owned();

    let proposal = AuthorityProposalRecord {
        proposal_id: "proposal_1".to_owned(),
        authority_kind: AuthorityKind::Deployment,
        authority_id: deployment_authority_id(
            "dep_companion",
            &device_fixture.binding.participant_id,
        )?,
        deployment_id: Some("dep_companion".to_owned()),
        proposal_kind: AuthorityProposalKind::Initial,
        participant_id: device_fixture.binding.participant_id.clone(),
        participant_artifact_digest: device_fixture.binding.artifact_digest.clone(),
        participant_needs_digest: device_fixture.binding.needs_digest.clone(),
        proposed_grant_set: device_fixture.required_grants.clone(),
        proposed_capabilities: vec!["device.use".to_owned()],
        proposal_digest: digest(70),
        payload: json!({
            "deploymentId": "dep_companion",
            "baseAuthorityVersion": null,
            "plan": "fixed"
        }),
        state: AuthorityProposalState::Pending,
        created_at: NOW,
        expires_at: Some(NOW + 10),
        superseded_at: None,
        version: 1,
    };
    store
        .create_authority_proposal(AuthorityProposalCreation {
            proposal: proposal.clone(),
            idempotency: proof(68, "authority-proposal.create"),
            actions: Vec::new(),
        })
        .await
        .map_err(|error| format!("create initial authority proposal: {error}"))?;
    let mut equivalent_proposal = proposal.clone();
    equivalent_proposal.proposal_id = "proposal_equivalent".to_owned();
    equivalent_proposal.created_at += 1;
    equivalent_proposal.expires_at = Some(NOW + 200);
    assert_eq!(
        store
            .create_authority_proposal(AuthorityProposalCreation {
                proposal: equivalent_proposal.clone(),
                idempotency: proof(69, "authority-proposal.create"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Replayed(json!({ "proposalId": proposal.proposal_id }))
    );
    assert!(store
        .get_authority_proposal(&equivalent_proposal.proposal_id)
        .await?
        .is_none());
    let rejection = AuthorityDecisionRecord {
        proposal_id: proposal.proposal_id.clone(),
        outcome: AuthorityDecisionOutcome::Rejected,
        decided_by: admin_id.clone(),
        reason: Some("not yet".to_owned()),
        decided_at: NOW + 2,
        decision_digest: digest(90),
    };
    store
        .decide_authority_proposal(AuthorityProposalDecision {
            proposal_id: proposal.proposal_id.clone(),
            expected_version: 1,
            expected_base_authority_version: None,
            decision: rejection,
            desired_authority: None,
            deployment: None,
            idempotency: proof(183, "authority-proposal.reject"),
            portal_binding: None,
            expected_portal_binding: None,
            portal_policy_snapshot: None,
            actions: Vec::new(),
        })
        .await?;
    let mut rejected_retry = proposal.clone();
    rejected_retry.proposal_id = "proposal_rejected_retry".to_owned();
    rejected_retry.created_at = NOW + 3;
    rejected_retry.expires_at = Some(NOW + 4);
    assert!(matches!(
        store
            .create_authority_proposal(AuthorityProposalCreation {
                proposal: rejected_retry.clone(),
                idempotency: proof(184, "authority-proposal.create"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    let mut elapsed_replay = rejected_retry.clone();
    elapsed_replay.proposal_id = "proposal_elapsed_replay".to_owned();
    elapsed_replay.created_at = NOW + 4;
    elapsed_replay.expires_at = Some(NOW + 100);
    assert_eq!(
        store
            .create_authority_proposal(AuthorityProposalCreation {
                proposal: elapsed_replay,
                idempotency: proof(184, "authority-proposal.create"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Replayed(json!({ "request": 184 }))
    );
    assert_eq!(
        store
            .get_authority_proposal(&rejected_retry.proposal_id)
            .await?
            .ok_or("elapsed replay proposal missing")?
            .0
            .state,
        AuthorityProposalState::Expired
    );
    let mut expired_retry = rejected_retry.clone();
    expired_retry.proposal_id = "proposal_expired_retry".to_owned();
    expired_retry.created_at = NOW + 4;
    expired_retry.expires_at = Some(NOW + 100);
    assert!(matches!(
        store
            .create_authority_proposal(AuthorityProposalCreation {
                proposal: expired_retry.clone(),
                idempotency: proof(185, "authority-proposal.create"),
                actions: Vec::new(),
            })
            .await?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store
            .get_authority_proposal(&rejected_retry.proposal_id)
            .await?
            .ok_or("expired proposal missing")?
            .0
            .state,
        AuthorityProposalState::Expired
    );
    let mut superseded_proposal = proposal.clone();
    superseded_proposal.proposal_id = "proposal_2".to_owned();
    superseded_proposal.proposal_digest = digest(71);
    superseded_proposal.created_at = NOW + 5;
    superseded_proposal.expires_at = Some(NOW + 100);
    store
        .create_authority_proposal(AuthorityProposalCreation {
            proposal: superseded_proposal.clone(),
            idempotency: proof(70, "authority-proposal.create"),
            actions: Vec::new(),
        })
        .await?;
    let decision = AuthorityDecisionRecord {
        proposal_id: superseded_proposal.proposal_id.clone(),
        outcome: AuthorityDecisionOutcome::Accepted,
        decided_by: admin_id.clone(),
        reason: None,
        decided_at: NOW + 6,
        decision_digest: digest(72),
    };
    let desired = DeploymentAuthorityRecord {
        authority_id: superseded_proposal.authority_id.clone(),
        deployment_id: "dep_companion".to_owned(),
        participant_id: superseded_proposal.participant_id.clone(),
        participant_kind: ParticipantKind::Device,
        participant_artifact_digest: superseded_proposal.participant_artifact_digest.clone(),
        accepted_needs_digest: superseded_proposal.participant_needs_digest.clone(),
        desired_grant_set: superseded_proposal.proposed_grant_set.clone(),
        desired_capabilities: superseded_proposal.proposed_capabilities.clone(),
        state: AuthorityState::Accepted,
        version: 1,
        created_at: NOW + 6,
        updated_at: NOW + 6,
        expires_at: None,
        decision: Some(AuthorityDecision {
            decided_at: NOW + 6,
            decided_by: admin_id.clone(),
            reason: None,
        }),
    };
    let proposal_proof = proof(58, "authority-proposal.decide");
    let decision_command = AuthorityProposalDecision {
        proposal_id: superseded_proposal.proposal_id.clone(),
        expected_version: 1,
        expected_base_authority_version: None,
        decision,
        desired_authority: Some(DesiredAuthorityRecord::Deployment(desired.clone())),
        deployment: None,
        idempotency: proposal_proof,
        portal_binding: None,
        expected_portal_binding: None,
        portal_policy_snapshot: None,
        actions: vec![action(62, "authority.accepted")],
    };
    assert!(matches!(
        store
            .decide_authority_proposal(decision_command.clone())
            .await
            .map_err(|error| format!("accept authority proposal: {error}"))?,
        IdempotentOutcome::Applied(_)
    ));
    assert!(matches!(
        store
            .decide_authority_proposal(decision_command)
            .await
            .map_err(|error| format!("replay authority proposal acceptance: {error}"))?,
        IdempotentOutcome::Replayed(_)
    ));
    assert_eq!(
        store
            .get_authority_proposal(&proposal.proposal_id)
            .await?
            .ok_or("superseded proposal missing")?
            .0
            .state,
        AuthorityProposalState::Rejected
    );
    assert_eq!(
        store
            .get_authority_proposal(&expired_retry.proposal_id)
            .await?
            .ok_or("superseded proposal missing")?
            .0
            .state,
        AuthorityProposalState::Superseded
    );
    assert_eq!(
        store
            .list_authority_proposals()
            .await?
            .into_iter()
            .filter(|(candidate, _)| candidate.proposal_digest == proposal.proposal_digest)
            .count(),
        3,
        "rejected, expired, and superseded semantic history must coexist",
    );
    let proposal_ids = store
        .list_authority_proposals()
        .await?
        .into_iter()
        .map(|(candidate, _)| candidate.proposal_id)
        .collect::<Vec<_>>();
    let mut sorted_proposal_ids = proposal_ids.clone();
    sorted_proposal_ids.sort();
    assert_eq!(proposal_ids, sorted_proposal_ids);
    assert_eq!(
        store
            .get_deployment_authority(&desired.deployment_id, &desired.participant_id)
            .await?,
        Some(desired.clone())
    );

    let mut wrong_expected_base = proposal.clone();
    wrong_expected_base.proposal_id = "proposal_wrong_expected_base".to_owned();
    wrong_expected_base.proposal_digest = digest(93);
    wrong_expected_base.created_at = NOW + 7;
    wrong_expected_base.expires_at = Some(NOW + 100);
    wrong_expected_base.payload["baseAuthorityVersion"] = json!(1);
    store
        .create_authority_proposal(AuthorityProposalCreation {
            proposal: wrong_expected_base.clone(),
            idempotency: proof(240, "authority-proposal.create"),
            actions: Vec::new(),
        })
        .await
        .map_err(|error| format!("create wrong-base proposal: {error}"))?;
    let mut wrong_expected_desired = desired.clone();
    wrong_expected_desired.version = 2;
    wrong_expected_desired.updated_at = NOW + 7;
    assert_eq!(
        store
            .decide_authority_proposal(AuthorityProposalDecision {
                proposal_id: wrong_expected_base.proposal_id.clone(),
                expected_version: 1,
                expected_base_authority_version: Some(Some(0)),
                decision: AuthorityDecisionRecord {
                    proposal_id: wrong_expected_base.proposal_id,
                    outcome: AuthorityDecisionOutcome::Accepted,
                    decided_by: admin_id.clone(),
                    reason: None,
                    decided_at: NOW + 7,
                    decision_digest: digest(94),
                },
                desired_authority: Some(
                    DesiredAuthorityRecord::Deployment(wrong_expected_desired,)
                ),
                deployment: None,
                portal_binding: None,
                expected_portal_binding: None,
                portal_policy_snapshot: None,
                idempotency: proof(241, "authority-proposal.accept-wrong-base"),
                actions: Vec::new(),
            })
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );

    let mut stale_initial = proposal.clone();
    stale_initial.proposal_id = "proposal_stale_initial".to_owned();
    stale_initial.created_at = NOW + 7;
    stale_initial.expires_at = Some(NOW + 100);
    assert_eq!(
        store
            .create_authority_proposal(AuthorityProposalCreation {
                proposal: stale_initial,
                idempotency: proof(186, "authority-proposal.create"),
                actions: Vec::new(),
            })
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );

    let mut noop_proposal = superseded_proposal.clone();
    noop_proposal.proposal_id = "proposal_accepted_noop".to_owned();
    noop_proposal.proposal_digest = digest(242);
    noop_proposal.created_at = NOW + 9;
    noop_proposal.expires_at = Some(NOW + 100);
    noop_proposal.payload["baseAuthorityVersion"] = json!(desired.version);
    store
        .create_authority_proposal(AuthorityProposalCreation {
            proposal: noop_proposal.clone(),
            idempotency: proof(242, "authority-proposal.create-noop"),
            actions: Vec::new(),
        })
        .await?;
    let mut noop_desired = desired.clone();
    noop_desired.version += 1;
    noop_desired.updated_at = NOW + 9;
    noop_desired.decision = Some(AuthorityDecision {
        decided_at: NOW + 9,
        decided_by: "different_admin".to_owned(),
        reason: Some("must not replace durable metadata".to_owned()),
    });
    store
        .decide_authority_proposal(AuthorityProposalDecision {
            proposal_id: noop_proposal.proposal_id.clone(),
            expected_version: noop_proposal.version,
            expected_base_authority_version: Some(Some(desired.version)),
            decision: AuthorityDecisionRecord {
                proposal_id: noop_proposal.proposal_id.clone(),
                outcome: AuthorityDecisionOutcome::Accepted,
                decided_by: admin_id.clone(),
                reason: None,
                decided_at: NOW + 9,
                decision_digest: digest(244),
            },
            desired_authority: Some(DesiredAuthorityRecord::Deployment(noop_desired)),
            deployment: None,
            idempotency: proof(245, "authority-proposal.accept-noop"),
            portal_binding: None,
            expected_portal_binding: None,
            portal_policy_snapshot: None,
            actions: Vec::new(),
        })
        .await?;
    assert_eq!(
        store
            .get_deployment_authority(&desired.deployment_id, &desired.participant_id)
            .await?,
        Some(desired.clone()),
        "semantic no-op must preserve authority version and decision metadata",
    );
    assert_eq!(
        store
            .get_authority_proposal(&noop_proposal.proposal_id)
            .await?
            .ok_or("accepted no-op proposal missing")?
            .0
            .state,
        AuthorityProposalState::Accepted,
    );
    assert!(!store
        .list_authority_proposals()
        .await?
        .iter()
        .any(
            |(candidate, _)| candidate.authority_id == desired.authority_id
                && candidate.state == AuthorityProposalState::Pending
        ));

    let service = AuthService::new(store.clone(), AuthServiceConfig::default())?;
    for (index, (fixture, deployment_id)) in [
        (&service_fixture, "dep_service_lineage"),
        (&device_fixture, "dep_device_lineage"),
    ]
    .into_iter()
    .enumerate()
    {
        let participant_artifact = serde_json::from_str(&fixture.binding.participant_json)?;
        let referenced_api_artifacts =
            serde_json::from_str::<BTreeMap<String, Value>>(&fixture.binding.api_artifacts_json)?
                .into_values()
                .collect::<Vec<_>>();
        let input = PresentDeploymentAuthorityInput {
            deployment_id: deployment_id.to_owned(),
            participant_artifact,
            referenced_api_artifacts,
            created_at: NOW + 20 + index as i64,
            expires_at: Some(NOW + 200),
            idempotency: proof(190 + index as u8, "deployment.authority.present-initial"),
            actions: Vec::new(),
        };
        let initial = match service.present_deployment_authority(input.clone()).await? {
            IdempotentOutcome::Applied(value) => value,
            IdempotentOutcome::Replayed(_) => return Err("initial presentation replayed".into()),
        };
        assert_eq!(
            initial.authority_id,
            deployment_authority_id(deployment_id, &fixture.binding.participant_id)?
        );
        let mut repeated = input;
        repeated.created_at += 10;
        repeated.expires_at = Some(NOW + 300);
        repeated.idempotency = proof(192 + index as u8, "deployment.authority.present-initial");
        assert_eq!(
            service.present_deployment_authority(repeated).await?,
            IdempotentOutcome::Replayed(json!({ "proposalId": initial.proposal_id }))
        );
    }
    let mut compatible_participant: Value =
        serde_json::from_str(&device_fixture.binding.participant_json)?;
    compatible_participant["displayName"] = json!("Updated device wording");
    let mut api_values = serde_json::from_str::<BTreeMap<String, Value>>(
        &device_fixture.binding.api_artifacts_json,
    )?;
    api_values
        .get_mut("required.api@v1")
        .ok_or("required API missing")?["schemas"]["Output"] = json!({
        "type": "object",
        "properties": { "added": { "type": "string" } }
    });
    compatible_participant["uses"]["required"]["requiredApi"]["apiDigest"] =
        json!(parse_api(&api_values["required.api@v1"])?.digest()?);
    let mut compatible_api_artifacts = api_values.values().cloned().collect::<Vec<_>>();
    compatible_api_artifacts.push(compatible_api_artifacts[0].clone());
    let compatible_input = PresentDeploymentAuthorityInput {
        deployment_id: "dep_companion".to_owned(),
        participant_artifact: compatible_participant.clone(),
        referenced_api_artifacts: compatible_api_artifacts,
        created_at: NOW + 9,
        expires_at: Some(NOW + 200),
        idempotency: proof(180, "deployment.authority.present"),
        actions: Vec::new(),
    };
    let compatible = match service
        .present_deployment_authority(compatible_input.clone())
        .await?
    {
        IdempotentOutcome::Applied(value) => value,
        IdempotentOutcome::Replayed(_) => return Err("first presentation replayed".into()),
    };
    assert_eq!(compatible.proposal_kind, AuthorityProposalKind::Update);
    assert_eq!(compatible.authority_id, desired.authority_id);
    assert_eq!(compatible.proposed_grant_set, device_fixture.all_grants);
    let mut equivalent_input = compatible_input;
    equivalent_input.created_at += 1;
    equivalent_input.expires_at = Some(NOW + 300);
    equivalent_input.idempotency = proof(181, "deployment.authority.present");
    assert_eq!(
        service
            .present_deployment_authority(equivalent_input)
            .await?,
        IdempotentOutcome::Replayed(json!({ "proposalId": compatible.proposal_id }))
    );

    let mut incompatible_apis = api_values;
    incompatible_apis
        .get_mut("required.api@v1")
        .ok_or("required API missing")?["schemas"]["Input"] = json!({
        "type": "object",
        "required": ["changed"],
        "properties": { "changed": { "type": "string" } }
    });
    let incompatible_api = parse_api(&incompatible_apis["required.api@v1"])?;
    compatible_participant["uses"]["required"]["requiredApi"]["apiDigest"] =
        json!(incompatible_api.digest()?);
    let migration = match service
        .present_deployment_authority(PresentDeploymentAuthorityInput {
            deployment_id: "dep_companion".to_owned(),
            participant_artifact: compatible_participant,
            referenced_api_artifacts: incompatible_apis.into_values().collect(),
            created_at: NOW + 11,
            expires_at: None,
            idempotency: proof(182, "deployment.authority.present"),
            actions: Vec::new(),
        })
        .await?
    {
        IdempotentOutcome::Applied(value) => value,
        IdempotentOutcome::Replayed(_) => return Err("migration presentation replayed".into()),
    };
    assert_eq!(migration.proposal_kind, AuthorityProposalKind::Migration);
    assert_eq!(
        store
            .get_authority_proposal(&compatible.proposal_id)
            .await?
            .ok_or("compatible proposal missing")?
            .0
            .state,
        AuthorityProposalState::Superseded
    );

    let review = DeviceActivationReviewRecord {
        review_id: "review_1".to_owned(),
        principal_id: device_principal.principal_id.clone(),
        deployment_id: device_instance.deployment_id.clone(),
        instance_id: device_instance.instance_id.clone(),
        request_digest: digest(73),
        payload: json!({ "device": "request" }),
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
            review: review.clone(),
            idempotency: proof(72, "activation-review.create"),
            actions: Vec::new(),
        })
        .await
        .map_err(|error| format!("create activation review: {error}"))?;
    let review_proof = proof(64, "activation-review.decide");
    let review_action = action(63, "device.approved");
    let approved_device = DeviceRecord {
        state: DeviceState::Active,
        updated_at: NOW + 6,
        version: 2,
        ..device
    };
    assert!(matches!(
        store
            .decide_activation_review(ActivationReviewDecision {
                review_id: review.review_id.clone(),
                expected_version: 1,
                state: DeviceActivationReviewState::Approved,
                decided_at: NOW + 6,
                decided_by: admin_id,
                reason: None,
                delegation: None,
                activate_device: true,
                idempotency: review_proof.clone(),
                actions: vec![review_action],
            })
            .await
            .map_err(|error| format!("decide activation review: {error}"))?,
        IdempotentOutcome::Applied(_)
    ));
    assert_eq!(
        store
            .get_device(
                &approved_device.principal_id,
                &approved_device.deployment_id
            )
            .await?,
        Some(approved_device)
    );

    let account_action_id = digest(20);
    let password_action_id = digest(61);
    let ready = store.list_ready_post_commit_actions(NOW, 100).await?;
    assert_eq!(
        ready
            .iter()
            .filter(|candidate| candidate.action_id == account_action_id)
            .count(),
        1
    );
    assert_eq!(
        ready
            .iter()
            .filter(|candidate| candidate.action_id == password_action_id)
            .count(),
        1
    );
    store
        .claim_post_commit_action(&password_action_id, NOW, NOW + 10)
        .await?
        .ok_or("password action was not claimed")?;
    let failed = store
        .fail_post_commit_action(&password_action_id, NOW + 10, NOW + 20, "retry".to_owned())
        .await?;
    assert_eq!(failed.attempts, 1);
    store
        .claim_post_commit_action(&password_action_id, NOW + 20, NOW + 30)
        .await?
        .ok_or("password action was not reclaimed")?;
    store
        .acknowledge_post_commit_action(&password_action_id, NOW + 30)
        .await?;
    store
        .acknowledge_post_commit_action(&password_action_id, NOW + 30)
        .await?;
    Ok(())
}
