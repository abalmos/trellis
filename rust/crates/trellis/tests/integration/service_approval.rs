use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use trellis_rs::client::{ServiceConnectWithContractOptions, TrellisClient, TrellisClientError};

use crate::support::assertions::assert_case_registered;

const SERVICE_CONTRACT_ID: &str = "trellis.integration.service-approval-service@v1";
const SERVICE_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.service-approval-service@v1",
  "displayName": "Trellis Integration Service Approval Service",
  "description": "Exercises service startup waiting for deployment authority approval.",
  "kind": "service",
  "capabilities": {
    "trellis.integration.service-approval-service::ping": {
      "displayName": "Ping approval service",
      "description": "Call the service after startup approval completes."
    }
  },
  "schemas": {
    "StartupPingInput": {
      "type": "object",
      "required": ["message"],
      "properties": { "message": { "type": "string" } }
    },
    "StartupPingOutput": {
      "type": "object",
      "required": ["message", "approved"],
      "properties": {
        "message": { "type": "string" },
        "approved": { "type": "boolean" }
      }
    }
  },
  "rpc": {
    "Startup.Ping": {
      "version": "v1",
      "subject": "rpc.v1.Startup.Ping",
      "input": { "schema": "StartupPingInput" },
      "output": { "schema": "StartupPingOutput" },
      "capabilities": { "call": ["trellis.integration.service-approval-service::ping"] },
      "errors": []
    }
  }
}"#;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct StartupPingInput {
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct StartupPingOutput {
    message: String,
    approved: bool,
}

struct StartupPingRpc;

impl trellis_rs::client::RpcDescriptor for StartupPingRpc {
    type Input = StartupPingInput;
    type Output = StartupPingOutput;

    const KEY: &'static str = "Startup.Ping";
    const SUBJECT: &'static str = "rpc.v1.Startup.Ping";
    const CALLER_CAPABILITIES: &'static [&'static str] =
        &["trellis.integration.service-approval-service::ping"];
    const ERRORS: &'static [&'static str] = &[];
    const INPUT_SCHEMA_JSON: &'static str =
        r#"{"type":"object","required":["message"],"properties":{"message":{"type":"string"}}}"#;
    const OUTPUT_SCHEMA_JSON: &'static str = r#"{"type":"object","required":["message","approved"],"properties":{"message":{"type":"string"},"approved":{"type":"boolean"}}}"#;
}

struct ServiceApprovalContract;

struct AbortOnDrop {
    handle: Option<JoinHandle<()>>,
}

impl AbortOnDrop {
    fn new(handle: JoinHandle<()>) -> Self {
        Self {
            handle: Some(handle),
        }
    }

    async fn abort_and_wait(mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
            let _ = handle.await;
        }
    }
}

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        if let Some(handle) = &self.handle {
            handle.abort();
        }
    }
}

#[tokio::test]
async fn service_approval_startup_blocks_before_authority_approval() {
    assert_case_registered(
        "service-approval.startup-blocks-before-authority-approval",
        "service-approval",
        "service_approval",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client for direct Auth RPCs");

    let service_contract =
        trellis_test::TrellisTestContract::from_manifest_json(SERVICE_CONTRACT_JSON)
            .expect("build service approval service contract");

    admin
        .create_deployment(&bootstrap_url, None, None)
        .await
        .expect("create deployment");

    let seed = trellis_rs::auth::generate_session_keypair().0;
    let auth_material = trellis_rs::client::SessionAuth::from_seed_base64url(&seed)
        .expect("build session auth from seed");

    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("get admin client");
    let auth = trellis_rs::sdk::auth::AuthClient::new(admin_client);
    auth.rpc()
        .auth()
        .service_instances_provision(
            &trellis_rs::sdk::auth::types::AuthServiceInstancesProvisionRequest {
                deployment_id: "test".to_string(),
                instance_key: auth_material.session_key.clone(),
            },
        )
        .await
        .expect("provision service instance key before authority approval");

    let connect_trellis_url = runtime.trellis_url().to_string();
    let connect_seed = seed.clone();
    let contract_digest = service_contract.digest().to_string();

    let (connected_tx, connected_rx) = oneshot::channel::<()>();
    let connect_handle: JoinHandle<
        trellis_rs::service::ConnectedServiceRuntime<ServiceApprovalContract>,
    > = tokio::spawn(async move {
        let client =
            TrellisClient::connect_service_with_contract(ServiceConnectWithContractOptions {
                trellis_url: &connect_trellis_url,
                contract_id: SERVICE_CONTRACT_ID,
                contract_digest: &contract_digest,
                contract_json: SERVICE_CONTRACT_JSON,
                session_key_seed_base64url: &connect_seed,
                timeout_ms: trellis_rs::service::DEFAULT_TIMEOUT_MS,
                retry_delay_ms: trellis_rs::service::DEFAULT_RETRY_DELAY_MS,
                authority_pending_timeout_ms:
                    trellis_rs::service::DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS,
            })
            .await
            .expect("service connect should succeed after approval");
        let service =
            trellis_rs::service::ConnectedServiceRuntime::<ServiceApprovalContract>::from_connected_client(
                "service-approval-fixture-service",
                Arc::new(client),
            )
            .expect("build connected service runtime from client");
        let _ = connected_tx.send(());
        service
    });

    let pending = tokio::time::timeout(Duration::from_millis(500), connected_rx).await;
    match pending {
        Err(_) => {}
        Ok(Ok(())) => {
            panic!("service connected before deployment authority approval");
        }
        Ok(Err(_)) => {
            panic!("service connect task failed before approval");
        }
    }

    connect_handle.abort();
    let _ = connect_handle.await;
}

#[tokio::test]
async fn service_approval_startup_completes_after_authority_approval() {
    assert_case_registered(
        "service-approval.startup-completes-after-authority-approval",
        "service-approval",
        "service_approval",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client for direct Auth RPCs");

    let service_contract =
        trellis_test::TrellisTestContract::from_manifest_json(SERVICE_CONTRACT_JSON)
            .expect("build service approval service contract");

    admin
        .create_deployment(&bootstrap_url, None, None)
        .await
        .expect("create deployment");

    let seed = trellis_rs::auth::generate_session_keypair().0;
    let auth_material = trellis_rs::client::SessionAuth::from_seed_base64url(&seed)
        .expect("build session auth from seed");

    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("get admin client");
    let auth = trellis_rs::sdk::auth::AuthClient::new(admin_client);
    auth.rpc()
        .auth()
        .service_instances_provision(
            &trellis_rs::sdk::auth::types::AuthServiceInstancesProvisionRequest {
                deployment_id: "test".to_string(),
                instance_key: auth_material.session_key.clone(),
            },
        )
        .await
        .expect("provision service instance key before authority approval");

    let connect_trellis_url = runtime.trellis_url().to_string();
    let connect_seed = seed.clone();
    let contract_digest = service_contract.digest().to_string();

    let (connected_tx, connected_rx) = oneshot::channel::<()>();
    let connect_handle: JoinHandle<
        trellis_rs::service::ConnectedServiceRuntime<ServiceApprovalContract>,
    > = tokio::spawn(async move {
        let client =
            TrellisClient::connect_service_with_contract(ServiceConnectWithContractOptions {
                trellis_url: &connect_trellis_url,
                contract_id: SERVICE_CONTRACT_ID,
                contract_digest: &contract_digest,
                contract_json: SERVICE_CONTRACT_JSON,
                session_key_seed_base64url: &connect_seed,
                timeout_ms: trellis_rs::service::DEFAULT_TIMEOUT_MS,
                retry_delay_ms: trellis_rs::service::DEFAULT_RETRY_DELAY_MS,
                authority_pending_timeout_ms:
                    trellis_rs::service::DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS,
            })
            .await
            .expect("service connect should succeed after approval");
        let service =
            trellis_rs::service::ConnectedServiceRuntime::<ServiceApprovalContract>::from_connected_client(
                "service-approval-fixture-service",
                Arc::new(client),
            )
            .expect("build connected service runtime from client");
        let _ = connected_tx.send(());
        service
    });

    let pending = tokio::time::timeout(Duration::from_millis(500), connected_rx).await;
    match pending {
        Err(_) => {}
        Ok(Ok(())) => {
            panic!("service connected before deployment authority approval");
        }
        Ok(Err(_)) => {
            panic!("service connect task failed before approval");
        }
    }

    admin
        .approve_contract(&bootstrap_url, &service_contract, None, &[])
        .await
        .expect("approve service contract");

    let service = tokio::time::timeout(Duration::from_secs(10), connect_handle)
        .await
        .expect("timed out waiting for service connect after approval")
        .expect("service connect task panicked");

    let service_task = AbortOnDrop::new(tokio::spawn(async move {
        service.run().await.expect("service runtime loop failed")
    }));

    service_task.abort_and_wait().await;
}

#[tokio::test]
async fn service_approval_approved_service_handles_client_rpc() {
    assert_case_registered(
        "service-approval.approved-service-handles-client-rpc",
        "service-approval",
        "service_approval",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client for direct Auth RPCs");

    let service_contract =
        trellis_test::TrellisTestContract::from_manifest_json(SERVICE_CONTRACT_JSON)
            .expect("build service approval service contract");

    admin
        .create_deployment(&bootstrap_url, None, None)
        .await
        .expect("create deployment");

    let seed = trellis_rs::auth::generate_session_keypair().0;
    let auth_material = trellis_rs::client::SessionAuth::from_seed_base64url(&seed)
        .expect("build session auth from seed");

    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("get admin client");
    let auth = trellis_rs::sdk::auth::AuthClient::new(admin_client);
    auth.rpc()
        .auth()
        .service_instances_provision(
            &trellis_rs::sdk::auth::types::AuthServiceInstancesProvisionRequest {
                deployment_id: "test".to_string(),
                instance_key: auth_material.session_key.clone(),
            },
        )
        .await
        .expect("provision service instance key before authority approval");

    let connect_trellis_url = runtime.trellis_url().to_string();
    let connect_seed = seed.clone();
    let contract_digest = service_contract.digest().to_string();

    let (connected_tx, connected_rx) = oneshot::channel::<()>();
    let connect_handle: JoinHandle<
        trellis_rs::service::ConnectedServiceRuntime<ServiceApprovalContract>,
    > = tokio::spawn(async move {
        let client =
            TrellisClient::connect_service_with_contract(ServiceConnectWithContractOptions {
                trellis_url: &connect_trellis_url,
                contract_id: SERVICE_CONTRACT_ID,
                contract_digest: &contract_digest,
                contract_json: SERVICE_CONTRACT_JSON,
                session_key_seed_base64url: &connect_seed,
                timeout_ms: trellis_rs::service::DEFAULT_TIMEOUT_MS,
                retry_delay_ms: trellis_rs::service::DEFAULT_RETRY_DELAY_MS,
                authority_pending_timeout_ms:
                    trellis_rs::service::DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS,
            })
            .await
            .expect("service connect should succeed after approval");
        let service =
            trellis_rs::service::ConnectedServiceRuntime::<ServiceApprovalContract>::from_connected_client(
                "service-approval-fixture-service",
                Arc::new(client),
            )
            .expect("build connected service runtime from client");
        let _ = connected_tx.send(());
        service
    });

    let pending = tokio::time::timeout(Duration::from_millis(500), connected_rx).await;
    match pending {
        Err(_) => {}
        Ok(Ok(())) => {
            panic!("service connected before deployment authority approval");
        }
        Ok(Err(_)) => {
            panic!("service connect task failed before approval");
        }
    }

    admin
        .approve_contract(&bootstrap_url, &service_contract, None, &[])
        .await
        .expect("approve service contract");

    let mut service = tokio::time::timeout(Duration::from_secs(10), connect_handle)
        .await
        .expect("timed out waiting for service connect after approval")
        .expect("service connect task panicked");

    service.register_rpc::<StartupPingRpc, _, _>(|_context, input| async move {
        Ok(StartupPingOutput {
            message: input.message,
            approved: true,
        })
    });

    let service_task = AbortOnDrop::new(tokio::spawn(async move {
        service.run().await.expect("service runtime loop failed")
    }));

    let client_contract = {
        let manifest = trellis_rs::contracts::ContractManifestBuilder::new(
            "trellis.integration.service-approval-client@v1",
            "Trellis Integration Service Approval Client",
            "App/client participant for the service approval fixture.",
            trellis_rs::contracts::ContractKind::App,
        )
        .use_ref(
            "approvalService",
            trellis_rs::contracts::use_contract(SERVICE_CONTRACT_ID)
                .with_rpc_call(["Startup.Ping"]),
        )
        .build()
        .expect("build service approval client contract manifest");
        trellis_test::TrellisTestContract::from_manifest_value(
            serde_json::to_value(manifest).expect("serialize client contract manifest"),
        )
        .expect("build test contract from manifest")
    };

    let client = admin
        .connect_client(&bootstrap_url, &client_contract)
        .await
        .expect("connect client");

    let output = call_startup_ping_with_retry(&client, "approved-startup").await;

    service_task.abort_and_wait().await;

    assert_eq!(
        output,
        StartupPingOutput {
            message: "approved-startup".to_string(),
            approved: true,
        }
    );
}

#[tokio::test]
async fn service_approval_service_bootstrap_denies_missing_disabled_and_digest_drift() {
    assert_case_registered(
        "service-approval.service-bootstrap-denies-missing-disabled-and-digest-drift",
        "service-approval",
        "service_approval",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();
    admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client for direct Auth RPCs");

    let service_contract =
        trellis_test::TrellisTestContract::from_manifest_json(SERVICE_CONTRACT_JSON)
            .expect("build service approval service contract");
    admin
        .create_deployment(&bootstrap_url, None, None)
        .await
        .expect("create deployment");
    admin
        .approve_contract(&bootstrap_url, &service_contract, None, &[])
        .await
        .expect("approve service contract");

    let seed = trellis_rs::auth::generate_session_keypair().0;
    let auth_material = trellis_rs::client::SessionAuth::from_seed_base64url(&seed)
        .expect("build session auth from seed");
    {
        let admin_client = admin
            .connect_admin(&bootstrap_url)
            .await
            .expect("get admin client");
        let auth = trellis_rs::sdk::auth::AuthClient::new(admin_client);
        auth.rpc()
            .auth()
            .service_instances_provision(
                &trellis_rs::sdk::auth::types::AuthServiceInstancesProvisionRequest {
                    deployment_id: "test".to_string(),
                    instance_key: auth_material.session_key.clone(),
                },
            )
            .await
            .expect("provision service instance key");
    }
    admin
        .reconcile(&bootstrap_url, "test")
        .await
        .expect("reconcile service authority after provision");
    admin
        .wait_ready(&bootstrap_url, "test")
        .await
        .expect("wait for materialized authority after provision");
    let sqlite = runtime.control_plane_sqlite();

    sqlite
        .execute(
            "UPDATE service_deployments SET disabled = 1 WHERE deployment_id = ?",
            ["test"],
        )
        .expect("disable service deployment");
    let state = stored_service_state(&sqlite, &auth_material.session_key);
    assert_denied_service_bootstrap(
        runtime.trellis_url(),
        &auth_material,
        service_contract.digest(),
        None,
        403,
        "service_deployment_disabled",
    )
    .await;
    assert_eq!(
        stored_service_state(&sqlite, &auth_material.session_key),
        state
    );
    sqlite
        .execute(
            "UPDATE service_deployments SET disabled = 0 WHERE deployment_id = ?",
            ["test"],
        )
        .expect("enable service deployment");
    connect_service_client(runtime.trellis_url(), service_contract.digest(), &seed).await;

    let instance_id = stored_service_state(&sqlite, &auth_material.session_key).instance
        ["instance_id"]
        .as_str()
        .expect("service instance id")
        .to_string();
    sqlite
        .execute(
            "UPDATE service_instances SET disabled = 1 WHERE instance_id = ?",
            [instance_id.as_str()],
        )
        .expect("disable service instance");
    let state = stored_service_state(&sqlite, &auth_material.session_key);
    assert_denied_service_bootstrap(
        runtime.trellis_url(),
        &auth_material,
        service_contract.digest(),
        None,
        403,
        "service_disabled",
    )
    .await;
    assert_eq!(
        stored_service_state(&sqlite, &auth_material.session_key),
        state
    );
    sqlite
        .execute(
            "UPDATE service_instances SET disabled = 0 WHERE instance_id = ?",
            [instance_id.as_str()],
        )
        .expect("enable service instance");
    connect_service_client(runtime.trellis_url(), service_contract.digest(), &seed).await;

    sqlite
        .execute(
            "DELETE FROM contracts WHERE digest = ?",
            [service_contract.digest()],
        )
        .expect("delete stored service contract for manifest-required branch");
    let state = stored_service_state(&sqlite, &auth_material.session_key);
    assert_denied_service_bootstrap(
        runtime.trellis_url(),
        &auth_material,
        service_contract.digest(),
        None,
        409,
        "manifest_required",
    )
    .await;
    assert_eq!(
        stored_service_state(&sqlite, &auth_material.session_key),
        state
    );
    assert_service_bootstrap_ready(
        runtime.trellis_url(),
        &auth_material,
        service_contract.digest(),
        Some(serde_json::from_str(SERVICE_CONTRACT_JSON).expect("parse service contract json")),
    )
    .await;

    let drifted_contract = drifted_service_contract_json();
    let state = stored_service_state(&sqlite, &auth_material.session_key);
    assert_denied_service_bootstrap(
        runtime.trellis_url(),
        &auth_material,
        service_contract.digest(),
        Some(drifted_contract),
        409,
        "presented_contract_digest_mismatch",
    )
    .await;
    assert_eq!(
        stored_service_state(&sqlite, &auth_material.session_key),
        state
    );

    let materialized = materialized_authority(&sqlite);
    sqlite
        .execute(
            "UPDATE materialized_authority SET status = ?, reconciled_at = NULL WHERE deployment_id = ?",
            params!["pending", "test"],
        )
        .expect("force materialized authority pending");
    let state = stored_service_state(&sqlite, &auth_material.session_key);
    assert_denied_service_bootstrap(
        runtime.trellis_url(),
        &auth_material,
        service_contract.digest(),
        Some(serde_json::from_str(SERVICE_CONTRACT_JSON).expect("parse service contract json")),
        202,
        "authority_reconciliation_pending",
    )
    .await;
    assert_eq!(
        stored_service_state(&sqlite, &auth_material.session_key),
        state
    );
    restore_materialized_authority(&sqlite, &materialized);

    sqlite
        .execute(
            "UPDATE materialized_authority SET status = ?, error = ? WHERE deployment_id = ?",
            params!["failed", "forced test failure", "test"],
        )
        .expect("force materialized authority failed");
    let state = stored_service_state(&sqlite, &auth_material.session_key);
    assert_denied_service_bootstrap(
        runtime.trellis_url(),
        &auth_material,
        service_contract.digest(),
        Some(serde_json::from_str(SERVICE_CONTRACT_JSON).expect("parse service contract json")),
        202,
        "authority_reconciliation_failed",
    )
    .await;
    assert_eq!(
        stored_service_state(&sqlite, &auth_material.session_key),
        state
    );
    restore_materialized_authority(&sqlite, &materialized);
    connect_service_client(runtime.trellis_url(), service_contract.digest(), &seed).await;
}

async fn call_startup_ping_with_retry(
    client: &trellis_rs::client::TrellisClient,
    message: &str,
) -> StartupPingOutput {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match client
            .call::<StartupPingRpc>(&StartupPingInput {
                message: message.to_string(),
            })
            .await
        {
            Ok(output) => return output,
            Err(error)
                if is_retryable_service_startup_error(&error) && Instant::now() < deadline =>
            {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(error) => panic!("call live Startup.Ping RPC: {error}"),
        }
    }
}

fn is_retryable_service_startup_error(error: &TrellisClientError) -> bool {
    match error {
        TrellisClientError::NatsRequest(message) => {
            message.contains("no responders") || message.contains("NoResponders")
        }
        TrellisClientError::Timeout => true,
        _ => false,
    }
}

#[derive(Debug, PartialEq)]
struct StoredServiceState {
    deployment: Value,
    instance: Value,
}

#[derive(Debug)]
struct MaterializedAuthorityRow {
    status: String,
    reconciled_at: Option<String>,
    error: Option<String>,
}

fn stored_service_state(
    sqlite: &trellis_test::TrellisControlPlaneSqlite,
    instance_key: &str,
) -> StoredServiceState {
    let deployment = sqlite
        .query(
            "SELECT * FROM service_deployments WHERE deployment_id = ?",
            ["test"],
        )
        .expect("query service deployment")
        .into_iter()
        .next()
        .expect("find service deployment");
    let instance = sqlite
        .query(
            "SELECT * FROM service_instances WHERE instance_key = ?",
            [instance_key],
        )
        .expect("query service instance")
        .into_iter()
        .next()
        .expect("find service instance");
    StoredServiceState {
        deployment: Value::Object(deployment),
        instance: Value::Object(instance),
    }
}

async fn assert_denied_service_bootstrap(
    trellis_url: &str,
    auth: &trellis_rs::client::SessionAuth,
    contract_digest: &str,
    contract: Option<Value>,
    expected_status: u16,
    expected_reason: &str,
) {
    let (status, body) = service_bootstrap(trellis_url, auth, contract_digest, contract).await;
    assert_eq!(status, expected_status);
    assert_eq!(body["reason"], expected_reason);
}

async fn assert_service_bootstrap_ready(
    trellis_url: &str,
    auth: &trellis_rs::client::SessionAuth,
    contract_digest: &str,
    contract: Option<Value>,
) {
    let (status, body) = service_bootstrap(trellis_url, auth, contract_digest, contract).await;
    assert_eq!(status, 200);
    assert_eq!(body["status"], "ready");
}

async fn service_bootstrap(
    trellis_url: &str,
    auth: &trellis_rs::client::SessionAuth,
    contract_digest: &str,
    contract: Option<Value>,
) -> (u16, Value) {
    let iat = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_secs();
    let mut body = json!({
        "sessionKey": auth.session_key,
        "contractId": SERVICE_CONTRACT_ID,
        "contractDigest": contract_digest,
        "iat": iat,
        "sig": auth.sign_sha256_domain("nats-connect", &format!("{iat}:{contract_digest}")),
    });
    if let Some(contract) = contract {
        body["contract"] = contract;
    }
    let response = reqwest::Client::new()
        .post(format!(
            "{}/bootstrap/service",
            trellis_url.trim_end_matches('/')
        ))
        .json(&body)
        .send()
        .await
        .expect("post service bootstrap");
    let status = response.status().as_u16();
    let body = response
        .json::<Value>()
        .await
        .expect("decode bootstrap json");
    (status, body)
}

async fn connect_service_client(trellis_url: &str, contract_digest: &str, seed: &str) {
    TrellisClient::connect_service_with_contract(ServiceConnectWithContractOptions {
        trellis_url,
        contract_id: SERVICE_CONTRACT_ID,
        contract_digest,
        contract_json: SERVICE_CONTRACT_JSON,
        session_key_seed_base64url: seed,
        timeout_ms: trellis_rs::service::DEFAULT_TIMEOUT_MS,
        retry_delay_ms: trellis_rs::service::DEFAULT_RETRY_DELAY_MS,
        authority_pending_timeout_ms: trellis_rs::service::DEFAULT_AUTHORITY_PENDING_TIMEOUT_MS,
    })
    .await
    .expect("service reconnect should succeed after state is restored");
}

fn drifted_service_contract_json() -> Value {
    let mut contract: Value = serde_json::from_str(SERVICE_CONTRACT_JSON).expect("parse contract");
    contract["capabilities"]["drift"] = json!({
        "displayName": "Digest drift",
        "description": "Forces a different manifest digest."
    });
    contract
}

fn materialized_authority(
    sqlite: &trellis_test::TrellisControlPlaneSqlite,
) -> MaterializedAuthorityRow {
    let row = sqlite
        .query(
            "SELECT status, reconciled_at, error FROM materialized_authority WHERE deployment_id = ?",
            ["test"],
        )
        .expect("query materialized authority")
        .into_iter()
        .next()
        .expect("materialized authority row exists");
    MaterializedAuthorityRow {
        status: row["status"].as_str().expect("status string").to_string(),
        reconciled_at: row["reconciled_at"].as_str().map(str::to_string),
        error: row["error"].as_str().map(str::to_string),
    }
}

fn restore_materialized_authority(
    sqlite: &trellis_test::TrellisControlPlaneSqlite,
    row: &MaterializedAuthorityRow,
) {
    sqlite
        .execute(
            "UPDATE materialized_authority SET status = ?, reconciled_at = ?, error = ? WHERE deployment_id = ?",
            params![
                &row.status,
                row.reconciled_at.as_deref(),
                row.error.as_deref(),
                "test"
            ],
        )
        .expect("restore materialized authority");
}
