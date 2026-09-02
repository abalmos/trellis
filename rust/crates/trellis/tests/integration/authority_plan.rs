use std::time::{Duration, Instant};

use base64::Engine as _;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use trellis_rs::client::RpcDescriptor;
use trellis_rs::service::ConnectedServiceRuntime;
use trellis_test::{TrellisTestContract, TrellisTestRuntimeOptions};

use super::resources::{resources_service_contract, ResourcesServiceContract};
use crate::support::assertions::assert_runtime_case_registered;

const DEPLOYMENT: &str = "authority-plan-integration";
const SERVICE_ID: &str = "integration.authority-plan-service@v1";
const BASE_API: &str = r#"{
  "format": "trellis.api.v1",
  "id": "integration.authority-plan-service@v1",
  "version": "1.0.0",
  "displayName": "Authority Plan Integration Service",
  "description": "Exercises authority-plan service startup.",
  "schemas": {
    "Value": {"type": "object", "required": ["value"], "properties": {"value": {"type": "string"}}}
  },
  "rpc": {
    "Value.Get": {"version": "v1", "input": {"schema": "Value"}, "output": {"schema": "Value"}, "errors": []}
  }
}"#;
const METADATA_API: &str = r#"{
  "format": "trellis.api.v1",
  "id": "integration.authority-plan-service@v1",
  "version": "1.0.0",
  "displayName": "Renamed Authority Plan Service",
  "description": "Changes human-facing metadata without changing machine identity.",
  "schemas": {
    "Value": {"type": "object", "required": ["value"], "properties": {"value": {"type": "string"}}}
  },
  "rpc": {
    "Value.Get": {"version": "v1", "input": {"schema": "Value"}, "output": {"schema": "Value"}, "errors": []}
  }
}"#;
const INCOMPATIBLE_API: &str = r#"{
  "format": "trellis.api.v1",
  "id": "integration.authority-plan-service@v1",
  "version": "1.0.0",
  "displayName": "Authority Plan Integration Service",
  "description": "Changes the existing RPC schema incompatibly.",
  "schemas": {
    "Value": {"type": "object", "required": ["value"], "properties": {"value": {"type": "integer"}}}
  },
  "rpc": {
    "Value.Get": {"version": "v1", "input": {"schema": "Value"}, "output": {"schema": "Value"}, "errors": []}
  }
}"#;
const ADDITIVE_API: &str = r#"{
  "format": "trellis.api.v1",
  "id": "integration.authority-plan-service@v1",
  "version": "1.0.0",
  "displayName": "Authority Plan Integration Service",
  "description": "Exercises authority-plan service startup.",
  "schemas": {
    "Value": {"type": "object", "required": ["value"], "properties": {"value": {"type": "string"}}}
  },
  "rpc": {
    "Value.Get": {"version": "v1", "input": {"schema": "Value"}, "output": {"schema": "Value"}, "errors": []},
    "Value.Put": {"version": "v1", "input": {"schema": "Value"}, "output": {"schema": "Value"}, "errors": []}
  }
}"#;

struct AuthorityPlanService;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct ValueMessage {
    value: String,
}

struct ValueGet;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct NumberMessage {
    value: i64,
}

struct NumberGet;

impl RpcDescriptor for ValueGet {
    type Input = ValueMessage;
    type Output = ValueMessage;

    const KEY: &'static str = "Value.Get";
    const SUBJECT: &'static str = "rpc.v1.Value.Get";
    const CALLER_CAPABILITIES: &'static [&'static str] = &[];
    const ERRORS: &'static [&'static str] = &[];
    const INPUT_SCHEMA_JSON: &'static str =
        r#"{"type":"object","required":["value"],"properties":{"value":{"type":"string"}}}"#;
    const OUTPUT_SCHEMA_JSON: &'static str = Self::INPUT_SCHEMA_JSON;
}

impl RpcDescriptor for NumberGet {
    type Input = NumberMessage;
    type Output = NumberMessage;

    const KEY: &'static str = "Value.Get";
    const SUBJECT: &'static str = "rpc.v1.Value.Get";
    const CALLER_CAPABILITIES: &'static [&'static str] = &[];
    const ERRORS: &'static [&'static str] = &[];
    const INPUT_SCHEMA_JSON: &'static str =
        r#"{"type":"object","required":["value"],"properties":{"value":{"type":"integer"}}}"#;
    const OUTPUT_SCHEMA_JSON: &'static str = Self::INPUT_SCHEMA_JSON;
}

#[tokio::test]
async fn service_startup_waits_for_additive_authority_approval() {
    assert_runtime_case_registered(
        "service-approval.startup-completes-after-authority-approval",
        "service-approval",
        "authority_plan",
    );
    additive_approval_flow().await;
}

#[tokio::test]
async fn accepted_additive_update_unblocks_service_and_rpc() {
    assert_runtime_case_registered(
        "authority-plan.presented-update-approved-then-connects",
        "authority-plan",
        "authority_plan",
    );
    additive_approval_flow().await;
}

#[tokio::test]
async fn service_bootstrap_denies_invalid_or_disabled_identity() {
    assert_runtime_case_registered(
        "service-approval.service-bootstrap-denies-missing-disabled-and-digest-drift",
        "service-approval",
        "authority_plan",
    );
    let runtime =
        trellis_test::TrellisTestRuntime::start(TrellisTestRuntimeOptions::repo_platform())
            .await
            .expect("start service-bootstrap denial runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe bootstrap URL");
    runtime
        .complete_bootstrap()
        .await
        .expect("complete bootstrap");
    let base = TrellisTestContract::from_native_api_json(
        SERVICE_ID,
        BASE_API,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build service-bootstrap contract");
    let key = runtime
        .admin()
        .provision_service_instance(&bootstrap_url, &base, Some(DEPLOYMENT), None)
        .await
        .expect("provision service identity");

    let mut missing = key.clone();
    missing.identity_seed = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([9_u8; 32]);
    let missing_error = service_bootstrap_error(&runtime, &missing).await;
    assert!(
        missing_error.contains("identity") || missing_error.contains("not found"),
        "unexpected missing-identity error: {missing_error}"
    );

    let mut drifted = key.clone();
    drifted.participant_digest = "A".repeat(43);
    let drift_error = service_bootstrap_error(&runtime, &drifted).await;
    assert!(
        drift_error.contains("digest") || drift_error.contains("artifact"),
        "unexpected digest-drift error: {drift_error}"
    );

    runtime
        .admin()
        .disable_service_instance(&bootstrap_url, &key.instance_id, 1)
        .await
        .expect("disable provisioned service instance");
    let disabled_error = service_bootstrap_error(&runtime, &key).await;
    assert!(
        disabled_error.contains("disabled")
            || disabled_error.contains("inactive")
            || disabled_error.contains("instance_mismatch"),
        "unexpected disabled-instance error: {disabled_error}"
    );
}

async fn service_bootstrap_error(
    runtime: &trellis_test::TrellisTestRuntime,
    key: &trellis_test::TrellisTestServiceKey,
) -> String {
    match tokio::time::timeout(
        Duration::from_secs(5),
        trellis_test::connect_service_runtime::<AuthorityPlanService>(runtime.trellis_url(), key),
    )
    .await
    {
        Ok(Err(error)) => error.to_string(),
        Ok(Ok(_)) => panic!("invalid service bootstrap connected"),
        Err(_) => panic!("invalid service bootstrap did not terminate"),
    }
}

#[tokio::test]
async fn accepted_resource_migration_binds_changed_kv_definition() {
    assert_runtime_case_registered(
        "authority-plan.resource-change-migration-approved-and-bound",
        "authority-plan",
        "authority_plan",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(TrellisTestRuntimeOptions::repo_platform())
            .await
            .expect("start authority-plan resource runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe authority-plan resource bootstrap URL");
    let base = resources_service_contract();
    let mut key = runtime
        .admin()
        .provision_service_instance(&bootstrap_url, &base, Some(DEPLOYMENT), None)
        .await
        .expect("provision base resource service");

    let mut changed_participant = base.participant().clone();
    let mut records = changed_participant["resources"]["kv"]
        .as_object_mut()
        .expect("resource KV definitions")
        .remove("records")
        .expect("base records definition");
    records["history"] = serde_json::json!(2);
    changed_participant["resources"]["kv"]
        .as_object_mut()
        .expect("resource KV definitions")
        .insert("changedRecords".to_owned(), records);
    let changed = TrellisTestContract::from_native_json(
        &serde_json::to_string(base.api()).expect("serialize resource API"),
        &serde_json::to_string(&changed_participant).expect("serialize changed participant"),
    )
    .expect("build changed resource service contract");
    key.participant_json =
        serde_json::to_string(changed.participant()).expect("serialize changed participant");
    key.participant_digest = changed.digest().to_owned();
    key.participant_needs_digest = changed.needs_digest().to_owned();
    let deployment_id = key.deployment_id.clone();
    let approved_key = key.clone();
    let connect_url = runtime.trellis_url().to_owned();
    let connect = tokio::spawn(async move {
        trellis_test::connect_service_runtime::<ResourcesServiceContract>(&connect_url, &key).await
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    let proposal = loop {
        let rows = runtime
            .control_plane_sqlite()
            .query(
                "SELECT proposal_id, proposal_kind, json_extract(payload_json, '$.baseAuthorityVersion') AS proposal_base_version FROM auth_authority_proposals WHERE deployment_id = ?1 AND state = 'pending'",
                [&deployment_id],
            )
            .expect("query pending resource migration");
        if let Some(row) = rows.first() {
            break (
                row["proposal_id"]
                    .as_str()
                    .expect("resource proposal ID")
                    .to_owned(),
                row["proposal_kind"]
                    .as_str()
                    .expect("resource proposal kind")
                    .to_owned(),
                row["proposal_base_version"]
                    .as_i64()
                    .expect("resource proposal base version"),
            );
        }
        assert!(
            Instant::now() < deadline,
            "resource migration proposal was not created"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    assert_eq!(proposal.1, "update");
    runtime
        .admin()
        .accept_authority_update(&bootstrap_url, &proposal.0, Some(proposal.2))
        .await
        .expect("accept resource migration");
    if let Ok(Ok(Ok(service))) = tokio::time::timeout(Duration::from_secs(1), connect).await {
        drop(service);
    }

    let service: ConnectedServiceRuntime<ResourcesServiceContract> = tokio::time::timeout(
        Duration::from_secs(10),
        trellis_test::connect_service_runtime(runtime.trellis_url(), &approved_key),
    )
    .await
    .expect("changed resource connect timed out")
    .expect("connect changed resource service");
    assert!(!service.resources().kv.contains_key("records"));
    assert_eq!(service.resources().kv["changedRecords"].history, 2);
    let kv = service
        .generated_handle()
        .kv_client("changedRecords")
        .await
        .expect("open changed records KV");
    kv.put("migration-check", Bytes::from_static(b"bound"))
        .await
        .expect("write changed records KV");
    assert_eq!(
        kv.get("migration-check")
            .await
            .expect("read changed records KV")
            .expect("changed records value"),
        Bytes::from_static(b"bound")
    );
    kv.delete("migration-check")
        .await
        .expect("delete changed records KV");
}

#[tokio::test]
async fn invalid_authority_acceptance_preserves_desired_and_proposal_state() {
    assert_runtime_case_registered(
        "authority-plan.acceptance-rejects-wrong-classification-expired-and-version-mismatch",
        "authority-plan",
        "authority_plan",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(TrellisTestRuntimeOptions::repo_platform())
            .await
            .expect("start invalid-acceptance runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe invalid-acceptance bootstrap URL");
    let base = TrellisTestContract::from_native_api_json(
        SERVICE_ID,
        BASE_API,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build base invalid-acceptance contract");
    let mut key = runtime
        .admin()
        .provision_service_instance(&bootstrap_url, &base, Some(DEPLOYMENT), None)
        .await
        .expect("provision invalid-acceptance service");
    let additive = TrellisTestContract::from_native_api_json(
        SERVICE_ID,
        ADDITIVE_API,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build additive invalid-acceptance contract");
    key.participant_json =
        serde_json::to_string(additive.participant()).expect("serialize additive participant");
    key.participant_digest = additive.digest().to_owned();
    key.participant_needs_digest = additive.needs_digest().to_owned();
    key.api_json = serde_json::to_string(additive.api()).expect("serialize additive API");
    key.api_digest = additive.api_digest().to_owned();
    let deployment_id = key.deployment_id.clone();
    let desired_before = runtime
        .control_plane_sqlite()
        .query(
            "SELECT participant_artifact_digest, accepted_needs_digest, desired_grant_set_json, desired_capabilities_json, state, version FROM auth_deployment_authorities WHERE deployment_id = ?1 AND participant_id = ?2",
            rusqlite::params![&deployment_id, additive.id()],
        )
        .expect("read desired authority before invalid acceptance");
    let connect_url = runtime.trellis_url().to_owned();
    let connect = tokio::spawn(async move {
        trellis_test::connect_service_runtime::<AuthorityPlanService>(&connect_url, &key).await
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    let proposal = loop {
        let rows = runtime
            .control_plane_sqlite()
            .query(
                "SELECT proposal_id, proposal_kind, json_extract(payload_json, '$.baseAuthorityVersion') AS base_authority_version FROM auth_authority_proposals WHERE deployment_id = ?1 AND state = 'pending'",
                [&deployment_id],
            )
            .expect("query invalid-acceptance proposal");
        if let Some(row) = rows.first() {
            break (
                row["proposal_id"]
                    .as_str()
                    .expect("invalid-acceptance proposal ID")
                    .to_owned(),
                row["proposal_kind"]
                    .as_str()
                    .expect("invalid-acceptance proposal kind")
                    .to_owned(),
                row["base_authority_version"]
                    .as_i64()
                    .expect("invalid-acceptance base version"),
            );
        }
        assert!(
            Instant::now() < deadline,
            "invalid-acceptance proposal was not created"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    assert_eq!(proposal.1, "update");
    assert!(runtime
        .admin()
        .accept_authority_migration(&bootstrap_url, &proposal.0, Some(proposal.2))
        .await
        .is_err());
    assert!(runtime
        .admin()
        .accept_authority_update(&bootstrap_url, &proposal.0, Some(proposal.2 + 1))
        .await
        .is_err());
    connect.abort();
    let _ = connect.await;
    runtime
        .control_plane_sqlite()
        .execute(
            "UPDATE auth_authority_proposals SET expires_at = created_at + 1 WHERE proposal_id = ?1",
            [&proposal.0],
        )
        .expect("expire authority proposal");
    assert!(runtime
        .admin()
        .accept_authority_update(&bootstrap_url, &proposal.0, Some(proposal.2))
        .await
        .is_err());
    assert_eq!(
        runtime
            .control_plane_sqlite()
            .query(
                "SELECT participant_artifact_digest, accepted_needs_digest, desired_grant_set_json, desired_capabilities_json, state, version FROM auth_deployment_authorities WHERE deployment_id = ?1 AND participant_id = ?2",
                rusqlite::params![&deployment_id, additive.id()],
            )
            .expect("read desired authority after invalid acceptance"),
        desired_before
    );
    assert_eq!(
        runtime
            .control_plane_sqlite()
            .query(
                "SELECT state, version FROM auth_authority_proposals WHERE proposal_id = ?1",
                [&proposal.0],
            )
            .expect("read proposal after invalid acceptance")[0]["state"],
        "pending"
    );
}

#[tokio::test]
async fn compatible_metadata_replacement_connects_without_approval() {
    assert_runtime_case_registered(
        "authority-plan.compatible-replacement-auto-allowed-strict",
        "authority-plan",
        "authority_plan",
    );
    let runtime =
        trellis_test::TrellisTestRuntime::start(TrellisTestRuntimeOptions::repo_platform())
            .await
            .expect("start compatible-replacement runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe bootstrap URL");
    runtime
        .complete_bootstrap()
        .await
        .expect("complete bootstrap");
    let base = TrellisTestContract::from_native_api_json(
        SERVICE_ID,
        BASE_API,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build base service contract");
    let metadata_source = TrellisTestContract::from_native_api_json(
        SERVICE_ID,
        METADATA_API,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build metadata-only service contract");
    let mut key = runtime
        .admin()
        .provision_service_instance(&bootstrap_url, &base, Some(DEPLOYMENT), None)
        .await
        .expect("provision base service identity");
    let metadata = &metadata_source;
    assert_eq!(key.participant_digest, metadata.digest());
    assert_eq!(key.participant_needs_digest, metadata.needs_digest());
    assert_eq!(key.api_digest, metadata.api_digest());
    key.participant_json =
        serde_json::to_string(metadata.participant()).expect("serialize metadata participant");
    key.api_json = serde_json::to_string(metadata.api()).expect("serialize metadata API");
    let deployment_id = key.deployment_id.clone();

    let mut service: ConnectedServiceRuntime<AuthorityPlanService> = tokio::time::timeout(
        Duration::from_secs(5),
        trellis_test::connect_service_runtime(runtime.trellis_url(), &key),
    )
    .await
    .expect("metadata-only service connect timed out")
    .expect("connect metadata-only replacement");
    assert!(runtime
        .control_plane_sqlite()
        .query(
            "SELECT proposal_id FROM auth_authority_proposals WHERE deployment_id = ?1 AND state = 'pending'",
            [&deployment_id],
        )
        .expect("query pending replacement proposals")
        .is_empty());
    service.register_rpc::<ValueGet, _, _>(|_, input| async move { Ok(input) });
    let service_task = tokio::spawn(async move { service.run().await });
    let client_contract = TrellisTestContract::from_builder_with_referenced_contracts(
        trellis_rs::contracts::ContractBuilder::authoring(
            "integration.authority-plan-metadata-client@v1",
            "integration.authority-plan-metadata-client@v1",
            "1.0.0",
            "Authority Plan Metadata Client",
            "Calls the metadata-only replacement service.",
            trellis_rs::contracts::ContractKind::App,
        )
        .use_ref(
            "service",
            trellis_rs::contracts::use_contract(SERVICE_ID).with_rpc_call(["Value.Get"]),
        ),
        &[&metadata_source],
    )
    .expect("build metadata client contract");
    let client = runtime
        .admin()
        .connect_new_local_user(
            &bootstrap_url,
            &client_contract,
            "metadata-client",
            "metadata-client-password-123",
        )
        .await
        .expect("connect metadata client");
    assert_eq!(
        client
            .call::<ValueGet>(&ValueMessage {
                value: "compatible".to_owned(),
            })
            .await
            .expect("call metadata-only replacement RPC")
            .value,
        "compatible"
    );
    service_task.abort();
}

#[tokio::test]
async fn accepted_incompatible_migration_replaces_service_contract() {
    assert_runtime_case_registered(
        "authority-plan.incompatible-migration-approved-replaces-contract",
        "authority-plan",
        "authority_plan",
    );
    let runtime =
        trellis_test::TrellisTestRuntime::start(TrellisTestRuntimeOptions::repo_platform())
            .await
            .expect("start incompatible-migration runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe bootstrap URL");
    runtime
        .complete_bootstrap()
        .await
        .expect("complete bootstrap");
    let base = TrellisTestContract::from_native_api_json(
        SERVICE_ID,
        BASE_API,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build base service contract");
    let incompatible_source = TrellisTestContract::from_native_api_json(
        SERVICE_ID,
        INCOMPATIBLE_API,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build incompatible service contract");
    let mut key = runtime
        .admin()
        .provision_service_instance(&bootstrap_url, &base, Some(DEPLOYMENT), None)
        .await
        .expect("provision base service identity");
    let incompatible = &incompatible_source;
    assert_ne!(key.participant_digest, incompatible.digest());
    key.participant_id = incompatible.id().to_owned();
    key.participant_digest = incompatible.digest().to_owned();
    key.participant_needs_digest = incompatible.needs_digest().to_owned();
    key.participant_json = serde_json::to_string(incompatible.participant())
        .expect("serialize incompatible participant");
    key.api_json = serde_json::to_string(incompatible.api()).expect("serialize incompatible API");
    key.api_digest = incompatible.api_digest().to_owned();
    let deployment_id = key.deployment_id.clone();
    let approved_key = key.clone();

    let trellis_url = runtime.trellis_url().to_owned();
    let connect = tokio::spawn(async move {
        trellis_test::connect_service_runtime::<AuthorityPlanService>(&trellis_url, &key).await
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    let proposal = loop {
        let pending = runtime
            .control_plane_sqlite()
            .query(
                "SELECT proposal_id, proposal_kind, payload_json FROM auth_authority_proposals WHERE deployment_id = ?1 AND state = 'pending'",
                [&deployment_id],
            )
            .expect("query migration proposal");
        if !pending.is_empty() {
            assert_eq!(pending[0]["proposal_kind"], "migration");
            let payload: serde_json::Value = serde_json::from_str(
                pending[0]["payload_json"]
                    .as_str()
                    .expect("migration proposal has a payload"),
            )
            .expect("parse migration proposal payload");
            break (
                pending[0]["proposal_id"]
                    .as_str()
                    .expect("migration proposal has an ID")
                    .to_owned(),
                payload["baseAuthorityVersion"].as_i64(),
            );
        }
        assert!(
            Instant::now() < deadline,
            "service did not create a migration proposal"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    runtime
        .admin()
        .accept_authority_migration(&bootstrap_url, &proposal.0, proposal.1)
        .await
        .expect("accept incompatible migration");
    if let Ok(Ok(Ok(service))) = tokio::time::timeout(Duration::from_secs(1), connect).await {
        drop(service);
    }
    let mut service: ConnectedServiceRuntime<AuthorityPlanService> = tokio::time::timeout(
        Duration::from_secs(10),
        trellis_test::connect_service_runtime(runtime.trellis_url(), &approved_key),
    )
    .await
    .expect("replacement connect timed out")
    .expect("connect incompatible replacement");
    service.register_rpc::<NumberGet, _, _>(|_, input| async move { Ok(input) });
    let service_task = tokio::spawn(async move { service.run().await });
    let client_contract = TrellisTestContract::from_builder_with_referenced_contracts(
        trellis_rs::contracts::ContractBuilder::authoring(
            "integration.authority-plan-incompatible-client@v1",
            "integration.authority-plan-incompatible-client@v1",
            "1.0.0",
            "Authority Plan Incompatible Client",
            "Calls the accepted incompatible replacement.",
            trellis_rs::contracts::ContractKind::App,
        )
        .use_ref(
            "service",
            trellis_rs::contracts::use_contract(SERVICE_ID).with_rpc_call(["Value.Get"]),
        ),
        &[&incompatible_source],
    )
    .expect("build incompatible client contract");
    let client = runtime
        .admin()
        .connect_new_local_user(
            &bootstrap_url,
            &client_contract,
            "incompatible-client",
            "incompatible-client-password-123",
        )
        .await
        .expect("connect incompatible client");
    assert_eq!(
        client
            .call::<NumberGet>(&NumberMessage { value: 42 })
            .await
            .expect("call replacement RPC"),
        NumberMessage { value: 42 }
    );
    service_task.abort();
}

async fn additive_approval_flow() {
    let runtime =
        trellis_test::TrellisTestRuntime::start(TrellisTestRuntimeOptions::repo_platform())
            .await
            .expect("start authority-plan runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe bootstrap URL");
    runtime
        .complete_bootstrap()
        .await
        .expect("complete bootstrap");
    let base = TrellisTestContract::from_native_api_json(
        SERVICE_ID,
        BASE_API,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build base service contract");
    let additive_source = TrellisTestContract::from_native_api_json(
        SERVICE_ID,
        ADDITIVE_API,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build additive service contract");
    let mut key = runtime
        .admin()
        .provision_service_instance(&bootstrap_url, &base, Some(DEPLOYMENT), None)
        .await
        .expect("provision base service identity");
    let additive = &additive_source;
    key.participant_id = additive.id().to_owned();
    key.participant_digest = additive.digest().to_owned();
    key.participant_needs_digest = additive.needs_digest().to_owned();
    key.participant_json =
        serde_json::to_string(additive.participant()).expect("serialize additive participant");
    key.api_json = serde_json::to_string(additive.api()).expect("serialize additive API");
    key.api_digest = additive.api_digest().to_owned();
    let deployment_id = key.deployment_id.clone();
    let approved_key = key.clone();

    let trellis_url = runtime.trellis_url().to_owned();
    let connect = tokio::spawn(async move {
        trellis_test::connect_service_runtime::<AuthorityPlanService>(&trellis_url, &key).await
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    let proposal_id = loop {
        let pending = runtime
            .control_plane_sqlite()
            .query(
                "SELECT proposal_id, json_extract(payload_json, '$.baseAuthorityVersion') AS base_authority_version FROM auth_authority_proposals WHERE deployment_id = ?1 AND state = 'pending'",
                [&deployment_id],
            )
            .expect("query pending authority proposal");
        if !pending.is_empty() {
            break (
                pending[0]["proposal_id"]
                    .as_str()
                    .expect("pending proposal has an ID")
                    .to_owned(),
                pending[0]["base_authority_version"].as_i64(),
            );
        }
        assert!(
            Instant::now() < deadline,
            "service did not create a pending authority proposal"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    assert!(
        !connect.is_finished(),
        "service connected before administrator approval"
    );

    runtime
        .admin()
        .accept_authority_update(&bootstrap_url, &proposal_id.0, proposal_id.1)
        .await
        .expect("approve additive authority update");
    let proposals = runtime
        .control_plane_sqlite()
        .query(
            "SELECT deployment_id, state, participant_artifact_digest FROM auth_authority_proposals WHERE deployment_id = ?1 ORDER BY created_at",
            [&deployment_id],
        )
        .expect("query authority proposals after approval");
    assert!(
        !proposals.iter().any(|row| row["state"] == "pending"),
        "service proposal remained pending after approval: {proposals:?}"
    );
    let service: ConnectedServiceRuntime<AuthorityPlanService> =
        tokio::time::timeout(Duration::from_secs(40), connect)
            .await
            .expect("service connect did not resume after approval")
            .expect("service connect task panicked")
            .expect("connect service after approval");
    drop(service);

    let mut service: ConnectedServiceRuntime<AuthorityPlanService> = tokio::time::timeout(
        Duration::from_secs(5),
        trellis_test::connect_service_runtime(runtime.trellis_url(), &approved_key),
    )
    .await
    .expect("pre-approved service connect timed out")
    .expect("connect pre-approved service");
    service.register_rpc::<ValueGet, _, _>(|_, input| async move { Ok(input) });
    let service_task = tokio::spawn(async move { service.run().await });

    let client_contract = TrellisTestContract::from_builder_with_referenced_contracts(
        trellis_rs::contracts::ContractBuilder::authoring(
            "integration.authority-plan-client@v1",
            "integration.authority-plan-client@v1",
            "1.0.0",
            "Authority Plan Integration Client",
            "Calls the service after authority approval.",
            trellis_rs::contracts::ContractKind::App,
        )
        .use_ref(
            "service",
            trellis_rs::contracts::use_contract(SERVICE_ID).with_rpc_call(["Value.Get"]),
        ),
        &[&additive_source],
    )
    .expect("build authority-plan client contract");
    let client = runtime
        .admin()
        .connect_new_local_user(
            &bootstrap_url,
            &client_contract,
            format!("authority-plan-client-{deployment_id}"),
            "authority-plan-password-123",
        )
        .await
        .expect("connect approved authority-plan client");
    assert_eq!(
        client
            .call::<ValueGet>(&ValueMessage {
                value: "approved".to_owned(),
            })
            .await
            .expect("call typed RPC after authority approval"),
        ValueMessage {
            value: "approved".to_owned()
        }
    );
    service_task.abort();
}
