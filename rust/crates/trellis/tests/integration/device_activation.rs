use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use bytes::Bytes;
use rusqlite::params;
use serde_json::{json, Value};
use trellis_rs::client::{OperationState, ServiceConnectWithContractOptions};
use trellis_rs::sdk::auth::types::{
    AuthDeploymentAuthorityPlanRequest, AuthDeploymentsCreateRequest,
    AuthDeviceUserAuthoritiesListRequest, AuthDeviceUserAuthoritiesResolveInput,
    AuthDeviceUserAuthoritiesReviewsDecideRequest, AuthDeviceUserAuthoritiesReviewsListRequest,
    AuthDeviceUserAuthoritiesRevokeRequest, AuthDevicesProvisionRequest,
    AuthServiceInstancesListRequest, AuthSessionsListRequest, AuthSessionsRevokeRequest,
};
use trellis_rs::sdk::auth::AuthClient as GeneratedAuthClient;

use crate::support::assertions::assert_case_registered;

const DEVICE_CONTRACT_ID: &str = "trellis.integration.device-activation-device@v1";
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

fn device_contract() -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractManifestBuilder::new(
        DEVICE_CONTRACT_ID,
        "Trellis Integration Activated Device",
        "Activated device participant for the device activation integration fixture.",
        trellis_rs::contracts::ContractKind::Device,
    )
    .use_ref(
        "auth",
        trellis_rs::contracts::use_contract(trellis_rs::sdk::auth::CONTRACT_ID)
            .with_rpc_call(["Auth.Sessions.Me"]),
    )
    .build()?;
    trellis_test::TrellisTestContract::from_manifest_value(serde_json::to_value(manifest)?)
}

fn review_contract() -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractManifestBuilder::new(
        "trellis.integration.device-activation-reviewer@v1",
        "Trellis Integration Device Activation Reviewer",
        "Reviews pending device activation requests.",
        trellis_rs::contracts::ContractKind::App,
    )
    .use_ref(
        "auth",
        trellis_rs::contracts::use_contract(trellis_rs::sdk::auth::CONTRACT_ID).with_rpc_call([
            "Auth.DeviceUserAuthorities.Reviews.List",
            "Auth.DeviceUserAuthorities.Reviews.Decide",
        ]),
    )
    .build()?;
    trellis_test::TrellisTestContract::from_manifest_value(serde_json::to_value(manifest)?)
}

fn device_root_secret() -> [u8; 32] {
    let mut secret = [0x44; 32];
    secret[0] = 1;
    secret[31] = 0x99;
    secret
}

fn generate_deployment_id() -> String {
    format!(
        "device-activation-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

fn generate_nonce() -> String {
    format!(
        "nonce-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

async fn create_device_deployment(auth: &GeneratedAuthClient<'_>, deployment_id: &str) {
    create_device_deployment_with_review_mode(auth, deployment_id, "none").await;
}

async fn create_device_deployment_with_review_mode(
    auth: &GeneratedAuthClient<'_>,
    deployment_id: &str,
    review_mode: &str,
) {
    auth.rpc()
        .auth()
        .deployments_create(&AuthDeploymentsCreateRequest(json!({
            "deploymentId": deployment_id,
            "kind": "device",
            "reviewMode": review_mode,
        })))
        .await
        .expect("create device deployment");
}

async fn wait_for_pending_review(
    auth: &GeneratedAuthClient<'_>,
    deployment_id: &str,
    instance_id: &str,
    public_identity_key: &str,
) -> trellis_rs::sdk::auth::types::AuthDeviceUserAuthoritiesReviewsListResponseEntriesItem {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let reviews = auth
            .rpc()
            .auth()
            .device_user_authorities_reviews_list(&AuthDeviceUserAuthoritiesReviewsListRequest {
                deployment_id: Some(deployment_id.to_string()),
                instance_id: Some(instance_id.to_string()),
                limit: 20,
                offset: None,
                state: Some("pending".to_string()),
            })
            .await
            .expect("list device activation reviews");
        if let Some(review) = reviews.entries.into_iter().find(|entry| {
            entry.deployment_id == deployment_id
                && entry.instance_id == instance_id
                && entry.public_identity_key == public_identity_key
        }) {
            return review;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for pending device activation review"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn approve_device_contract(
    auth: &GeneratedAuthClient<'_>,
    deployment_id: &str,
    device_contract: &trellis_test::TrellisTestContract,
) -> String {
    let device_contract_digest = device_contract.digest().to_string();

    let contract_map: BTreeMap<String, Value> = device_contract
        .manifest()
        .as_object()
        .expect("device contract manifest should be a JSON object")
        .clone()
        .into_iter()
        .collect();
    let planned = auth
        .rpc()
        .auth()
        .deployment_authority_plan(&AuthDeploymentAuthorityPlanRequest {
            deployment_id: deployment_id.to_string(),
            contract: contract_map,
            expected_digest: device_contract.digest().to_string(),
        })
        .await
        .expect("plan device contract authority");

    if planned.plan.get("classification").and_then(Value::as_str) == Some("update") {
        auth.rpc()
            .auth()
            .deployment_authority_accept_update(
                &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityAcceptUpdateRequest {
                    plan_id: planned
                        .plan
                        .get("planId")
                        .and_then(Value::as_str)
                        .expect("planId")
                        .to_string(),
                    expected_desired_version: None,
                },
            )
            .await
            .expect("accept device contract update");
    } else {
        auth.rpc()
            .auth()
            .deployment_authority_accept_migration(
                &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityAcceptMigrationRequest {
                    plan_id: planned
                        .plan
                        .get("planId")
                        .and_then(Value::as_str)
                        .expect("planId")
                        .to_string(),
                    expected_desired_version: None,
                    acknowledgement: "Approved by device activation integration test.".to_string(),
                },
            )
            .await
            .expect("accept device contract migration");
    }

    auth.rpc()
        .auth()
        .deployment_authority_reconcile(
            &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityReconcileRequest {
                deployment_id: deployment_id.to_string(),
                desired_version: None,
            },
        )
        .await
        .expect("reconcile device deployment authority");

    wait_for_deployment_authority_ready(auth, deployment_id).await;
    device_contract_digest
}

async fn provision_device(
    auth: &GeneratedAuthClient<'_>,
    deployment_id: &str,
    identity: &trellis_rs::auth::DeviceIdentity,
) -> trellis_rs::sdk::auth::types::AuthDevicesProvisionResponse {
    let provisioned = auth
        .rpc()
        .auth()
        .devices_provision(&AuthDevicesProvisionRequest {
            deployment_id: deployment_id.to_string(),
            public_identity_key: identity.public_identity_key.clone(),
            activation_key: identity.activation_key_base64url.clone(),
            metadata: Some(
                [(
                    "name".to_string(),
                    "Integration Activated Device".to_string(),
                )]
                .into_iter()
                .collect(),
            ),
        })
        .await
        .expect("provision device");

    assert_eq!(provisioned.instance.deployment_id, deployment_id);
    assert_eq!(
        provisioned.instance.public_identity_key,
        identity.public_identity_key
    );

    provisioned
}

async fn session_key_for(auth: &GeneratedAuthClient<'_>, kind: &str, id: &str) -> String {
    auth.rpc()
        .auth()
        .sessions_list(&AuthSessionsListRequest {
            limit: 500,
            offset: None,
            user: None,
        })
        .await
        .expect("list sessions")
        .entries
        .into_iter()
        .find(|entry| {
            entry.get("sessionKey").and_then(Value::as_str) == Some(id)
                || (entry.get("participantKind").and_then(Value::as_str) == Some(kind)
                    && principal_matches(entry, id))
        })
        .and_then(|entry| {
            entry
                .get("sessionKey")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| panic!("expected {kind} session"))
}

fn principal_matches(entry: &Value, id: &str) -> bool {
    let Some(principal) = entry.get("principal") else {
        return false;
    };
    ["deviceId", "id", "instanceId"]
        .into_iter()
        .any(|field| principal.get(field).and_then(Value::as_str) == Some(id))
}

async fn wait_for_session_absent(auth: &GeneratedAuthClient<'_>, session_key: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let sessions = auth
            .rpc()
            .auth()
            .sessions_list(&AuthSessionsListRequest {
                limit: 500,
                offset: None,
                user: None,
            })
            .await
            .expect("list sessions after revocation");
        if sessions
            .entries
            .iter()
            .all(|entry| entry.get("sessionKey").and_then(Value::as_str) != Some(session_key))
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for revoked session removal"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_for_sessions_me_denied(auth: &GeneratedAuthClient<'_>) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if auth.rpc().auth().sessions_me().await.is_err() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for Auth.Sessions.Me denial"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_for_device_authority_state(
    auth: &GeneratedAuthClient<'_>,
    deployment_id: &str,
    instance_id: &str,
    public_identity_key: &str,
    state: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let listings = auth
            .rpc()
            .auth()
            .device_user_authorities_list(&AuthDeviceUserAuthoritiesListRequest {
                deployment_id: Some(deployment_id.to_string()),
                instance_id: Some(instance_id.to_string()),
                limit: 20,
                offset: None,
                state: Some(state.to_string()),
            })
            .await
            .expect("list device user authorities");
        if listings.entries.iter().any(|entry| {
            entry.instance_id == instance_id
                && entry.public_identity_key == public_identity_key
                && entry.deployment_id == deployment_id
                && entry.state == state
        }) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for device authority state"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_for_disabled_service_instance(auth: &GeneratedAuthClient<'_>, instance_key: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let instances = auth
            .rpc()
            .auth()
            .service_instances_list(&AuthServiceInstancesListRequest {
                deployment_id: Some("test".to_string()),
                disabled: Some(true),
                limit: 100,
                offset: None,
            })
            .await
            .expect("list disabled service instances");
        if instances
            .entries
            .iter()
            .any(|entry| entry.instance_key == instance_key && entry.disabled)
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for disabled service instance"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

fn narrow_reviewer_session(
    sqlite: &trellis_test::TrellisControlPlaneSqlite,
    session_key: &str,
    capability: &str,
) {
    let rows = sqlite
        .query(
            "SELECT trellis_id, session FROM sessions WHERE session_key = ?",
            [session_key],
        )
        .expect("query reviewer session row");
    let row = rows.first().expect("expected reviewer session row");
    let user_id = row
        .get("trellis_id")
        .and_then(Value::as_str)
        .expect("reviewer session user id");
    let mut session: Value = serde_json::from_str(
        row.get("session")
            .and_then(Value::as_str)
            .expect("reviewer session JSON"),
    )
    .expect("parse reviewer session JSON");
    session["delegatedCapabilities"] = json!([capability]);

    let user_rows = sqlite
        .query(
            "SELECT capabilities FROM users WHERE user_id = ?",
            [user_id],
        )
        .expect("query reviewer user row");
    let mut capabilities: Vec<String> = serde_json::from_str(
        user_rows
            .first()
            .and_then(|row| row.get("capabilities"))
            .and_then(Value::as_str)
            .expect("reviewer user capabilities"),
    )
    .expect("parse reviewer capabilities");
    capabilities.push(capability.to_string());
    capabilities.sort();
    capabilities.dedup();

    sqlite
        .execute(
            "UPDATE users SET capabilities = ? WHERE user_id = ?",
            params![
                serde_json::to_string(&capabilities).expect("serialize reviewer capabilities"),
                user_id
            ],
        )
        .expect("update reviewer user capabilities");
    sqlite
        .execute(
            "UPDATE sessions SET session = ? WHERE session_key = ?",
            params![session.to_string(), session_key],
        )
        .expect("update reviewer session capabilities");
}

async fn wait_for_deployment_authority_ready(auth: &GeneratedAuthClient<'_>, deployment_id: &str) {
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        let result = auth
            .rpc()
            .auth()
            .deployment_authority_get(
                &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityGetRequest {
                    deployment_id: deployment_id.to_string(),
                },
            )
            .await
            .expect("get deployment authority");
        let materialized = &result.materialized_authority;
        if materialized.is_null() {
            tokio::time::sleep(Duration::from_millis(25)).await;
            continue;
        }
        let obj = materialized
            .as_object()
            .expect("materialized authority should be object");
        match obj.get("status").and_then(Value::as_str) {
            Some("current") => {
                if obj.get("desiredVersion").and_then(Value::as_str)
                    == Some(&result.authority.version as &str)
                    && obj.get("reconciledAt").is_some_and(|v| !v.is_null())
                {
                    return;
                }
            }
            Some("failed") => {
                panic!(
                    "deployment authority reconciliation failed: {}",
                    obj.get("error")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                );
            }
            _ => {}
        }
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for deployment authority ready"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn post_auth_json(trellis_url: &str, path: &str, body: &Value) -> (u16, Value) {
    let response = reqwest::Client::new()
        .post(
            url::Url::parse(trellis_url)
                .expect("parse trellis URL")
                .join(path)
                .expect("build auth URL"),
        )
        .json(body)
        .send()
        .await
        .expect("POST auth JSON");
    let status = response.status().as_u16();
    let body = response.json().await.expect("parse auth JSON response");
    (status, body)
}

fn assert_auth_reason(status: u16, body: &Value, expected_status: u16, expected_reason: &str) {
    assert_eq!(status, expected_status);
    assert_eq!(
        body.get("reason").and_then(Value::as_str),
        Some(expected_reason)
    );
}

fn corrupt_signature(sig: &str) -> String {
    format!(
        "{}{}",
        if sig.starts_with('A') { "B" } else { "A" },
        &sig[1..]
    )
}

async fn expire_device_activation_flow(runtime: &trellis_test::TrellisTestRuntime, flow_id: &str) {
    let creds_path = runtime.workdir().join("nats/creds/auth-auth.creds");
    let client = async_nats::ConnectOptions::new()
        .credentials_file(creds_path)
        .await
        .expect("read auth NATS credentials")
        .connect(runtime.nats_url())
        .await
        .expect("connect to auth NATS account");
    let kv = async_nats::jetstream::new(client)
        .get_key_value("trellis_browser_flows")
        .await
        .expect("open browser-flow KV");
    let mut value: Value = serde_json::from_slice(
        &kv.get(flow_id)
            .await
            .expect("read device activation flow")
            .expect("device activation flow should exist"),
    )
    .expect("decode device activation flow JSON");
    value["expiresAt"] = json!("2000-01-01T00:00:00.000Z");
    kv.put(
        flow_id.to_string(),
        Bytes::from(serde_json::to_vec(&value).expect("encode expired device activation flow")),
    )
    .await
    .expect("write expired device activation flow");
}

#[tokio::test]
async fn device_activation_admin_provisions_known_device() {
    assert_case_registered(
        "device-activation.admin-provisions-known-device",
        "device-activation",
        "device_activation",
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
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");

    create_device_deployment(&auth, &deployment_id).await;
    approve_device_contract(&auth, &deployment_id, &device_contract).await;

    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let _provisioned = provision_device(&auth, &deployment_id, &identity).await;
}

#[tokio::test]
async fn device_activation_device_starts_activation_request() {
    assert_case_registered(
        "device-activation.device-starts-activation-request",
        "device-activation",
        "device_activation",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");

    create_device_deployment(&auth, &deployment_id).await;
    approve_device_contract(&auth, &deployment_id, &device_contract).await;

    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let _provisioned = provision_device(&auth, &deployment_id, &identity).await;

    let nonce = generate_nonce();
    let payload = trellis_rs::auth::build_device_activation_payload(
        &identity.activation_key_base64url,
        &identity.public_identity_key,
        &nonce,
    )
    .expect("build activation payload");

    let activation = trellis_rs::auth::start_device_activation_request(&trellis_url, &payload)
        .await
        .expect("start device activation request");

    let flow_id = url::Url::parse(&activation.activation_url)
        .expect("parse activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()));
    assert!(flow_id.is_some(), "activation URL should contain a flowId");
}

#[tokio::test]
async fn device_activation_admin_resolves_activation_operation() {
    assert_case_registered(
        "device-activation.admin-resolves-activation-operation",
        "device-activation",
        "device_activation",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");

    create_device_deployment(&auth, &deployment_id).await;
    let _device_contract_digest =
        approve_device_contract(&auth, &deployment_id, &device_contract).await;

    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let _provisioned = provision_device(&auth, &deployment_id, &identity).await;

    let nonce = generate_nonce();
    let payload = trellis_rs::auth::build_device_activation_payload(
        &identity.activation_key_base64url,
        &identity.public_identity_key,
        &nonce,
    )
    .expect("build activation payload");

    let activation = trellis_rs::auth::start_device_activation_request(&trellis_url, &payload)
        .await
        .expect("start device activation request");

    let flow_id = url::Url::parse(&activation.activation_url)
        .expect("parse activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()))
        .expect("activation URL should contain a flowId");

    let resolve_op = auth
        .operation()
        .auth()
        .device_user_authorities_resolve()
        .start(&AuthDeviceUserAuthoritiesResolveInput {
            flow_id: flow_id.clone(),
        })
        .await
        .expect("start device user authorities resolve operation");

    let terminal = resolve_op
        .wait()
        .await
        .expect("wait for resolve operation to complete");
    assert_eq!(terminal.state, OperationState::Completed);
    let output = terminal
        .output
        .expect("resolve operation completed with output");
    assert_eq!(
        output.0.get("status").and_then(Value::as_str),
        Some("activated")
    );
}

#[tokio::test]
async fn device_activation_review_reject_denies_connect() {
    assert_case_registered(
        "device-activation.review-reject-denies-connect",
        "device-activation",
        "device_activation",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");

    create_device_deployment_with_review_mode(&auth, &deployment_id, "required").await;
    let device_contract_digest =
        approve_device_contract(&auth, &deployment_id, &device_contract).await;

    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let provisioned = provision_device(&auth, &deployment_id, &identity).await;

    let nonce = generate_nonce();
    let payload = trellis_rs::auth::build_device_activation_payload(
        &identity.activation_key_base64url,
        &identity.public_identity_key,
        &nonce,
    )
    .expect("build activation payload");

    let activation = trellis_rs::auth::start_device_activation_request(&trellis_url, &payload)
        .await
        .expect("start device activation request");

    let flow_id = url::Url::parse(&activation.activation_url)
        .expect("parse activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()))
        .expect("activation URL should contain a flowId");

    let resolve_op = auth
        .operation()
        .auth()
        .device_user_authorities_resolve()
        .start(&AuthDeviceUserAuthoritiesResolveInput {
            flow_id: flow_id.clone(),
        })
        .await
        .expect("start device user authorities resolve operation");

    let review = wait_for_pending_review(
        &auth,
        &deployment_id,
        &provisioned.instance.instance_id,
        &identity.public_identity_key,
    )
    .await;

    let rejection_reason = "integration review rejected";
    let decided = auth
        .rpc()
        .auth()
        .device_user_authorities_reviews_decide(&AuthDeviceUserAuthoritiesReviewsDecideRequest {
            review_id: review.review_id,
            decision: "reject".to_string(),
            reason: Some(rejection_reason.to_string()),
        })
        .await
        .expect("reject device activation review");
    assert_eq!(decided.review.state, "rejected");
    assert_eq!(decided.review.reason.as_deref(), Some(rejection_reason));

    let terminal = resolve_op
        .wait()
        .await
        .expect("wait for resolve operation to complete");
    assert_eq!(terminal.state, OperationState::Completed);
    let output = terminal
        .output
        .expect("resolve operation completed with output");
    assert_eq!(
        output.0.get("status").and_then(Value::as_str),
        Some("rejected")
    );
    assert_eq!(
        output.0.get("reason").and_then(Value::as_str),
        Some(rejection_reason)
    );

    let wait_error = trellis_rs::auth::wait_for_device_activation(
        trellis_rs::auth::WaitForDeviceActivationOpts {
            trellis_url: &trellis_url,
            flow_id: &flow_id,
            public_identity_key: &identity.public_identity_key,
            nonce: &nonce,
            identity_seed_base64url: &identity.identity_seed_base64url,
            contract_digest: Some(&device_contract_digest),
            poll_interval: Duration::from_millis(25),
        },
    )
    .await
    .expect_err("rejected activation wait should fail");
    assert!(
        matches!(&wait_error, trellis_rs::auth::TrellisAuthError::DeviceActivationRejected(reason) if reason.contains(rejection_reason)),
        "unexpected device activation wait error: {wait_error:?}"
    );

    let connect = trellis_rs::client::TrellisClient::connect_device(
        trellis_rs::client::DeviceConnectOptions {
            trellis_url: &trellis_url,
            contract_digest: &device_contract_digest,
            public_identity_key: &identity.public_identity_key,
            identity_seed_base64url: &identity.identity_seed_base64url,
            timeout_ms: 15_000,
        },
    )
    .await;
    assert!(connect.is_err(), "rejected device should not connect");
}

#[tokio::test]
async fn device_activation_review_capability_is_deployment_scoped() {
    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let own_deployment_id = generate_deployment_id();
    let other_deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");
    create_device_deployment_with_review_mode(&auth, &own_deployment_id, "required").await;
    create_device_deployment_with_review_mode(&auth, &other_deployment_id, "required").await;
    approve_device_contract(&auth, &own_deployment_id, &device_contract).await;
    approve_device_contract(&auth, &other_deployment_id, &device_contract).await;

    let own_secret = device_root_secret();
    let own_identity =
        trellis_rs::auth::derive_device_identity(&own_secret).expect("derive own identity");
    let other_secret = [0x45; 32];
    let other_identity =
        trellis_rs::auth::derive_device_identity(&other_secret).expect("derive other identity");
    let own_device = provision_device(&auth, &own_deployment_id, &own_identity).await;
    let other_device = provision_device(&auth, &other_deployment_id, &other_identity).await;

    let own_payload = trellis_rs::auth::build_device_activation_payload(
        &own_identity.activation_key_base64url,
        &own_identity.public_identity_key,
        &generate_nonce(),
    )
    .expect("build own activation payload");
    let other_payload = trellis_rs::auth::build_device_activation_payload(
        &other_identity.activation_key_base64url,
        &other_identity.public_identity_key,
        &generate_nonce(),
    )
    .expect("build other activation payload");
    let own_activation =
        trellis_rs::auth::start_device_activation_request(&trellis_url, &own_payload)
            .await
            .expect("start own activation request");
    let other_activation =
        trellis_rs::auth::start_device_activation_request(&trellis_url, &other_payload)
            .await
            .expect("start other activation request");
    let own_flow_id = url::Url::parse(&own_activation.activation_url)
        .expect("parse own activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()))
        .expect("own activation URL should contain a flowId");
    let other_flow_id = url::Url::parse(&other_activation.activation_url)
        .expect("parse other activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()))
        .expect("other activation URL should contain a flowId");

    let own_resolve = auth
        .operation()
        .auth()
        .device_user_authorities_resolve()
        .start(&AuthDeviceUserAuthoritiesResolveInput {
            flow_id: own_flow_id,
        })
        .await
        .expect("start own resolve operation");
    let _other_resolve = auth
        .operation()
        .auth()
        .device_user_authorities_resolve()
        .start(&AuthDeviceUserAuthoritiesResolveInput {
            flow_id: other_flow_id,
        })
        .await
        .expect("start other resolve operation");
    let own_review = wait_for_pending_review(
        &auth,
        &own_deployment_id,
        &own_device.instance.instance_id,
        &own_identity.public_identity_key,
    )
    .await;
    let other_review = wait_for_pending_review(
        &auth,
        &other_deployment_id,
        &other_device.instance.instance_id,
        &other_identity.public_identity_key,
    )
    .await;

    let admin_reviews = auth
        .rpc()
        .auth()
        .device_user_authorities_reviews_list(&AuthDeviceUserAuthoritiesReviewsListRequest {
            deployment_id: None,
            instance_id: None,
            limit: 20,
            offset: None,
            state: Some("pending".to_string()),
        })
        .await
        .expect("admin list pending reviews");
    assert!(admin_reviews
        .entries
        .iter()
        .any(|entry| entry.review_id == own_review.review_id));
    assert!(admin_reviews
        .entries
        .iter()
        .any(|entry| entry.review_id == other_review.review_id));

    let reviewer_seed = trellis_rs::auth::generate_session_keypair().0;
    let reviewer_session_key = trellis_rs::client::SessionAuth::from_seed_base64url(&reviewer_seed)
        .expect("derive reviewer session key")
        .session_key;
    let reviewer_contract = review_contract().expect("build reviewer contract");
    let mut reviewer_admin = runtime.admin();
    let reviewer_client = reviewer_admin
        .connect_client_with_session_seed(&bootstrap_url, &reviewer_contract, reviewer_seed)
        .await
        .expect("connect scoped reviewer client");
    narrow_reviewer_session(
        &runtime.control_plane_sqlite(),
        &reviewer_session_key,
        &format!("trellis.auth::device.review.{own_deployment_id}"),
    );
    let reviewer_auth = GeneratedAuthClient::new(&reviewer_client);
    let scoped_reviews = reviewer_auth
        .rpc()
        .auth()
        .device_user_authorities_reviews_list(&AuthDeviceUserAuthoritiesReviewsListRequest {
            deployment_id: None,
            instance_id: None,
            limit: 20,
            offset: None,
            state: Some("pending".to_string()),
        })
        .await
        .expect("scoped reviewer list pending reviews");
    assert!(scoped_reviews
        .entries
        .iter()
        .any(|entry| entry.review_id == own_review.review_id));
    assert!(scoped_reviews
        .entries
        .iter()
        .all(|entry| entry.deployment_id != other_deployment_id));

    let denied = reviewer_auth
        .rpc()
        .auth()
        .device_user_authorities_reviews_decide(&AuthDeviceUserAuthoritiesReviewsDecideRequest {
            review_id: other_review.review_id,
            decision: "approve".to_string(),
            reason: None,
        })
        .await;
    assert!(
        denied.is_err(),
        "scoped reviewer must not decide other deployment"
    );

    let decided = reviewer_auth
        .rpc()
        .auth()
        .device_user_authorities_reviews_decide(&AuthDeviceUserAuthoritiesReviewsDecideRequest {
            review_id: own_review.review_id,
            decision: "approve".to_string(),
            reason: None,
        })
        .await
        .expect("scoped reviewer approves own deployment");
    assert_eq!(decided.review.state, "approved");
    assert_eq!(decided.review.deployment_id, own_deployment_id);

    let terminal = own_resolve
        .wait()
        .await
        .expect("wait for own resolve operation");
    assert_eq!(terminal.state, OperationState::Completed);
}

#[tokio::test]
async fn device_activation_revoked_device_cannot_reconnect() {
    assert_case_registered(
        "device-activation.revoked-device-cannot-reconnect",
        "device-activation",
        "device_activation",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");

    create_device_deployment(&auth, &deployment_id).await;
    let device_contract_digest =
        approve_device_contract(&auth, &deployment_id, &device_contract).await;

    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let provisioned = provision_device(&auth, &deployment_id, &identity).await;

    let nonce = generate_nonce();
    let payload = trellis_rs::auth::build_device_activation_payload(
        &identity.activation_key_base64url,
        &identity.public_identity_key,
        &nonce,
    )
    .expect("build activation payload");

    let activation = trellis_rs::auth::start_device_activation_request(&trellis_url, &payload)
        .await
        .expect("start device activation request");

    let flow_id = url::Url::parse(&activation.activation_url)
        .expect("parse activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()))
        .expect("activation URL should contain a flowId");

    let resolve_op = auth
        .operation()
        .auth()
        .device_user_authorities_resolve()
        .start(&AuthDeviceUserAuthoritiesResolveInput {
            flow_id: flow_id.clone(),
        })
        .await
        .expect("start device user authorities resolve operation");

    let terminal = resolve_op
        .wait()
        .await
        .expect("wait for resolve operation to complete");
    assert_eq!(terminal.state, OperationState::Completed);

    let _connect_info = trellis_rs::auth::wait_for_device_activation(
        trellis_rs::auth::WaitForDeviceActivationOpts {
            trellis_url: &trellis_url,
            flow_id: &flow_id,
            public_identity_key: &identity.public_identity_key,
            nonce: &nonce,
            identity_seed_base64url: &identity.identity_seed_base64url,
            contract_digest: Some(&device_contract_digest),
            poll_interval: Duration::from_millis(25),
        },
    )
    .await
    .expect("wait for device activation");

    let device = trellis_rs::client::TrellisClient::connect_device(
        trellis_rs::client::DeviceConnectOptions {
            trellis_url: &trellis_url,
            contract_digest: &device_contract_digest,
            public_identity_key: &identity.public_identity_key,
            identity_seed_base64url: &identity.identity_seed_base64url,
            timeout_ms: 15_000,
        },
    )
    .await
    .expect("connect device client");
    device
        .flush()
        .await
        .expect("device NATS flush should succeed");

    let device_auth = GeneratedAuthClient::new(&device);
    let me = device_auth
        .rpc()
        .auth()
        .sessions_me()
        .await
        .expect("call Auth.Sessions.Me as device");
    assert_eq!(me.participant_kind.as_str(), Some("device"));

    let revoked = auth
        .rpc()
        .auth()
        .device_user_authorities_revoke(&AuthDeviceUserAuthoritiesRevokeRequest {
            instance_id: provisioned.instance.instance_id.clone(),
        })
        .await
        .expect("revoke device activation");
    assert!(revoked.success, "device activation revoke should succeed");

    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let listings = auth
            .rpc()
            .auth()
            .device_user_authorities_list(&AuthDeviceUserAuthoritiesListRequest {
                deployment_id: Some(deployment_id.clone()),
                instance_id: Some(provisioned.instance.instance_id.clone()),
                limit: 20,
                offset: None,
                state: Some("revoked".to_string()),
            })
            .await
            .expect("list revoked device user authorities");
        let found = listings.entries.iter().any(|entry| {
            entry.instance_id == provisioned.instance.instance_id
                && entry.public_identity_key == identity.public_identity_key
                && entry.deployment_id == deployment_id
                && entry.state == "revoked"
        });
        if found {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for revoked device activation state"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        if device_auth.rpc().auth().sessions_me().await.is_err() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for existing device session denial"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    let reconnect = trellis_rs::client::TrellisClient::connect_device(
        trellis_rs::client::DeviceConnectOptions {
            trellis_url: &trellis_url,
            contract_digest: &device_contract_digest,
            public_identity_key: &identity.public_identity_key,
            identity_seed_base64url: &identity.identity_seed_base64url,
            timeout_ms: 15_000,
        },
    )
    .await;
    assert!(reconnect.is_err(), "revoked device should not reconnect");
}

#[tokio::test]
async fn auth_sessions_revoke_revokes_device_and_service_access() {
    assert_case_registered(
        "auth.sessions-revoke-revokes-device-and-service-access",
        "auth",
        "device_activation",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let service_contract =
        trellis_test::TrellisTestContract::from_manifest_json(SERVICE_CONTRACT_JSON)
            .expect("build service contract");
    let service_key = admin
        .provision_service_instance(&bootstrap_url, &service_contract, None, None)
        .await
        .expect("provision service instance");
    let service_digest = service_contract.digest().to_string();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");
    create_device_deployment(&auth, &deployment_id).await;
    let device_contract_digest =
        approve_device_contract(&auth, &deployment_id, &device_contract).await;

    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let provisioned = provision_device(&auth, &deployment_id, &identity).await;
    let nonce = generate_nonce();
    let payload = trellis_rs::auth::build_device_activation_payload(
        &identity.activation_key_base64url,
        &identity.public_identity_key,
        &nonce,
    )
    .expect("build activation payload");
    let activation = trellis_rs::auth::start_device_activation_request(&trellis_url, &payload)
        .await
        .expect("start device activation request");
    let flow_id = url::Url::parse(&activation.activation_url)
        .expect("parse activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()))
        .expect("activation URL should contain a flowId");
    let resolve_op = auth
        .operation()
        .auth()
        .device_user_authorities_resolve()
        .start(&AuthDeviceUserAuthoritiesResolveInput {
            flow_id: flow_id.clone(),
        })
        .await
        .expect("start device user authorities resolve operation");
    let terminal = resolve_op
        .wait()
        .await
        .expect("wait for resolve operation to complete");
    assert_eq!(terminal.state, OperationState::Completed);
    trellis_rs::auth::wait_for_device_activation(trellis_rs::auth::WaitForDeviceActivationOpts {
        trellis_url: &trellis_url,
        flow_id: &flow_id,
        public_identity_key: &identity.public_identity_key,
        nonce: &nonce,
        identity_seed_base64url: &identity.identity_seed_base64url,
        contract_digest: Some(&device_contract_digest),
        poll_interval: Duration::from_millis(25),
    })
    .await
    .expect("wait for device activation");

    let device = trellis_rs::client::TrellisClient::connect_device(
        trellis_rs::client::DeviceConnectOptions {
            trellis_url: &trellis_url,
            contract_digest: &device_contract_digest,
            public_identity_key: &identity.public_identity_key,
            identity_seed_base64url: &identity.identity_seed_base64url,
            timeout_ms: 15_000,
        },
    )
    .await
    .expect("connect device client");
    device
        .flush()
        .await
        .expect("device NATS flush should succeed");
    let device_auth = GeneratedAuthClient::new(&device);
    device_auth
        .rpc()
        .auth()
        .sessions_me()
        .await
        .expect("call Auth.Sessions.Me as device");
    let device_session_key =
        session_key_for(&auth, "device", &provisioned.instance.instance_id).await;

    let revoked = auth
        .rpc()
        .auth()
        .sessions_revoke(&AuthSessionsRevokeRequest {
            session_key: device_session_key.clone(),
        })
        .await
        .expect("revoke device session through Auth.Sessions.Revoke");
    assert!(revoked.success);
    wait_for_device_authority_state(
        &auth,
        &deployment_id,
        &provisioned.instance.instance_id,
        &identity.public_identity_key,
        "revoked",
    )
    .await;
    wait_for_session_absent(&auth, &device_session_key).await;
    wait_for_sessions_me_denied(&device_auth).await;

    let reconnect_device = trellis_rs::client::TrellisClient::connect_device(
        trellis_rs::client::DeviceConnectOptions {
            trellis_url: &trellis_url,
            contract_digest: &device_contract_digest,
            public_identity_key: &identity.public_identity_key,
            identity_seed_base64url: &identity.identity_seed_base64url,
            timeout_ms: 15_000,
        },
    )
    .await;
    assert!(
        reconnect_device.is_err(),
        "revoked device should not reconnect"
    );

    let service = trellis_rs::client::TrellisClient::connect_service_with_contract(
        ServiceConnectWithContractOptions {
            trellis_url: &trellis_url,
            contract_id: SERVICE_CONTRACT_ID,
            contract_digest: &service_digest,
            contract_json: SERVICE_CONTRACT_JSON,
            session_key_seed_base64url: &service_key.seed,
            timeout_ms: 15_000,
            retry_delay_ms: 100,
            authority_pending_timeout_ms: 1_000,
        },
    )
    .await
    .expect("connect service client");
    service
        .flush()
        .await
        .expect("service NATS flush should succeed");

    let service_session_key = session_key_for(&auth, "service", &service_key.session_key).await;
    let revoked = auth
        .rpc()
        .auth()
        .sessions_revoke(&AuthSessionsRevokeRequest {
            session_key: service_session_key.clone(),
        })
        .await
        .expect("revoke service session through Auth.Sessions.Revoke");
    assert!(revoked.success);
    wait_for_session_absent(&auth, &service_session_key).await;
    wait_for_disabled_service_instance(&auth, &service_key.session_key).await;

    let reconnect_service = trellis_rs::client::TrellisClient::connect_service_with_contract(
        ServiceConnectWithContractOptions {
            trellis_url: &trellis_url,
            contract_id: SERVICE_CONTRACT_ID,
            contract_digest: &service_digest,
            contract_json: SERVICE_CONTRACT_JSON,
            session_key_seed_base64url: &service_key.seed,
            timeout_ms: 2_000,
            retry_delay_ms: 100,
            authority_pending_timeout_ms: 500,
        },
    )
    .await;
    assert!(
        reconnect_service.is_err(),
        "disabled service instance should not reconnect"
    );
}

#[tokio::test]
async fn device_activation_connect_info_admin_reviewed_before_activation() {
    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");
    create_device_deployment(&auth, &deployment_id).await;
    let device_contract_digest =
        approve_device_contract(&auth, &deployment_id, &device_contract).await;

    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let provisioned = provision_device(&auth, &deployment_id, &identity).await;

    let iat = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let signed = trellis_rs::auth::sign_device_wait_request(
        "connect-info",
        &identity.public_identity_key,
        "connect-info",
        &identity.identity_seed_base64url,
        Some(&device_contract_digest),
        iat,
    )
    .expect("sign connect-info request");
    let response: Value = reqwest::Client::new()
        .post(
            url::Url::parse(&trellis_url)
                .expect("parse trellis URL")
                .join("/auth/devices/connect-info")
                .expect("build connect-info URL"),
        )
        .json(&json!({
            "publicIdentityKey": signed.public_identity_key,
            "contractDigest": device_contract_digest,
            "iat": signed.iat,
            "sig": signed.sig,
        }))
        .send()
        .await
        .expect("POST /auth/devices/connect-info")
        .json()
        .await
        .expect("parse connect-info response");
    assert_eq!(
        response.get("status").and_then(Value::as_str),
        Some("ready")
    );
    assert_eq!(
        response
            .pointer("/connectInfo/auth/authority")
            .and_then(Value::as_str),
        Some("admin_reviewed")
    );

    let reviews = auth
        .rpc()
        .auth()
        .device_user_authorities_reviews_list(&AuthDeviceUserAuthoritiesReviewsListRequest {
            deployment_id: Some(deployment_id.clone()),
            instance_id: Some(provisioned.instance.instance_id.clone()),
            limit: 20,
            offset: None,
            state: None,
        })
        .await
        .expect("list device activation reviews");
    assert!(reviews.entries.is_empty());

    let device = trellis_rs::client::TrellisClient::connect_device(
        trellis_rs::client::DeviceConnectOptions {
            trellis_url: &trellis_url,
            contract_digest: response
                .pointer("/connectInfo/contractDigest")
                .and_then(Value::as_str)
                .expect("connect info contract digest"),
            public_identity_key: &identity.public_identity_key,
            identity_seed_base64url: &identity.identity_seed_base64url,
            timeout_ms: 15_000,
        },
    )
    .await
    .expect("connect device client");
    device
        .flush()
        .await
        .expect("device NATS flush should succeed");

    let me = GeneratedAuthClient::new(&device)
        .rpc()
        .auth()
        .sessions_me()
        .await
        .expect("call Auth.Sessions.Me as device");
    assert_eq!(me.participant_kind.as_str(), Some("device"));
}

#[tokio::test]
async fn device_activation_wait_and_connect_info_reject_bad_proofs_and_stale_iats() {
    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");
    create_device_deployment(&auth, &deployment_id).await;
    let device_contract_digest =
        approve_device_contract(&auth, &deployment_id, &device_contract).await;

    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let _provisioned = provision_device(&auth, &deployment_id, &identity).await;

    let nonce = generate_nonce();
    let payload = trellis_rs::auth::build_device_activation_payload(
        &identity.activation_key_base64url,
        &identity.public_identity_key,
        &nonce,
    )
    .expect("build activation payload");
    let activation = trellis_rs::auth::start_device_activation_request(&trellis_url, &payload)
        .await
        .expect("start device activation request");
    let flow_id = url::Url::parse(&activation.activation_url)
        .expect("parse activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()))
        .expect("activation URL should contain a flowId");
    let stale_iat = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .saturating_sub(3_600);

    let stale_wait = trellis_rs::auth::sign_device_wait_request(
        &flow_id,
        &identity.public_identity_key,
        &nonce,
        &identity.identity_seed_base64url,
        Some(&device_contract_digest),
        stale_iat,
    )
    .expect("sign stale wait request");
    let (status, body) = post_auth_json(
        &trellis_url,
        "/auth/devices/activate/wait",
        &json!(stale_wait),
    )
    .await;
    assert_auth_reason(status, &body, 400, "iat_out_of_range");
    assert!(
        body.get("serverNow").and_then(Value::as_u64).is_some(),
        "stale wait proof should return serverNow"
    );

    let signed_wait = trellis_rs::auth::sign_device_wait_request(
        &flow_id,
        &identity.public_identity_key,
        &nonce,
        &identity.identity_seed_base64url,
        Some(&device_contract_digest),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    )
    .expect("sign wait request");
    let mut bad_wait = signed_wait.clone();
    bad_wait.sig = corrupt_signature(&bad_wait.sig);
    let (status, body) = post_auth_json(
        &trellis_url,
        "/auth/devices/activate/wait",
        &json!(bad_wait),
    )
    .await;
    assert_auth_reason(status, &body, 400, "invalid_signature");

    let wrong_nonce = trellis_rs::auth::sign_device_wait_request(
        &flow_id,
        &identity.public_identity_key,
        "wrong-nonce",
        &identity.identity_seed_base64url,
        Some(&device_contract_digest),
        signed_wait.iat,
    )
    .expect("sign wrong-nonce wait request");
    let (status, body) = post_auth_json(
        &trellis_url,
        "/auth/devices/activate/wait",
        &json!(wrong_nonce),
    )
    .await;
    assert_auth_reason(status, &body, 400, "invalid_request");

    let wrong_root_secret = [0x55; 32];
    let wrong_identity = trellis_rs::auth::derive_device_identity(&wrong_root_secret)
        .expect("derive wrong device identity");
    let wrong_key = trellis_rs::auth::sign_device_wait_request(
        &flow_id,
        &wrong_identity.public_identity_key,
        &nonce,
        &wrong_identity.identity_seed_base64url,
        Some(&device_contract_digest),
        signed_wait.iat,
    )
    .expect("sign wrong-key wait request");
    let (status, body) = post_auth_json(
        &trellis_url,
        "/auth/devices/activate/wait",
        &json!(wrong_key),
    )
    .await;
    assert_auth_reason(status, &body, 400, "invalid_request");

    let missing_flow = trellis_rs::auth::sign_device_wait_request(
        &format!("missing-{flow_id}"),
        &identity.public_identity_key,
        &nonce,
        &identity.identity_seed_base64url,
        Some(&device_contract_digest),
        signed_wait.iat,
    )
    .expect("sign missing-flow wait request");
    let (status, body) = post_auth_json(
        &trellis_url,
        "/auth/devices/activate/wait",
        &json!(missing_flow),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(body.get("status").and_then(Value::as_str), Some("rejected"));
    assert_eq!(
        body.get("reason").and_then(Value::as_str),
        Some("device_activation_flow_not_found")
    );

    expire_device_activation_flow(&runtime, &flow_id).await;
    let (status, body) = post_auth_json(
        &trellis_url,
        "/auth/devices/activate/wait",
        &json!(&signed_wait),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(body.get("status").and_then(Value::as_str), Some("rejected"));
    assert_eq!(
        body.get("reason").and_then(Value::as_str),
        Some("device_flow_expired")
    );

    let stale_connect = trellis_rs::auth::sign_device_wait_request(
        "connect-info",
        &identity.public_identity_key,
        "connect-info",
        &identity.identity_seed_base64url,
        Some(&device_contract_digest),
        stale_iat,
    )
    .expect("sign stale connect-info request");
    let (status, body) = post_auth_json(
        &trellis_url,
        "/auth/devices/connect-info",
        &json!({
            "publicIdentityKey": stale_connect.public_identity_key,
            "contractDigest": &device_contract_digest,
            "iat": stale_connect.iat,
            "sig": stale_connect.sig,
        }),
    )
    .await;
    assert_auth_reason(status, &body, 400, "iat_out_of_range");
    assert!(
        body.get("serverNow").and_then(Value::as_u64).is_some(),
        "stale connect-info proof should return serverNow"
    );

    let signed_connect = trellis_rs::auth::sign_device_wait_request(
        "connect-info",
        &identity.public_identity_key,
        "connect-info",
        &identity.identity_seed_base64url,
        Some(&device_contract_digest),
        signed_wait.iat,
    )
    .expect("sign connect-info request");
    let (status, body) = post_auth_json(
        &trellis_url,
        "/auth/devices/connect-info",
        &json!({
            "publicIdentityKey": signed_connect.public_identity_key,
            "contractDigest": &device_contract_digest,
            "iat": signed_connect.iat,
            "sig": corrupt_signature(&signed_connect.sig),
        }),
    )
    .await;
    assert_auth_reason(status, &body, 400, "invalid_signature");

    let unauthorized_digest = "unauthorized_device_contract";
    let unauthorized = trellis_rs::auth::sign_device_wait_request(
        "connect-info",
        &identity.public_identity_key,
        "connect-info",
        &identity.identity_seed_base64url,
        Some(unauthorized_digest),
        signed_wait.iat,
    )
    .expect("sign unauthorized connect-info request");
    let (status, body) = post_auth_json(
        &trellis_url,
        "/auth/devices/connect-info",
        &json!({
            "publicIdentityKey": unauthorized.public_identity_key,
            "contractDigest": unauthorized_digest,
            "iat": unauthorized.iat,
            "sig": unauthorized.sig,
        }),
    )
    .await;
    assert_auth_reason(status, &body, 403, "contract_digest_not_allowed");
}

#[tokio::test]
async fn device_activation_device_receives_connect_info() {
    assert_case_registered(
        "device-activation.device-receives-connect-info",
        "device-activation",
        "device_activation",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");

    create_device_deployment(&auth, &deployment_id).await;
    let device_contract_digest =
        approve_device_contract(&auth, &deployment_id, &device_contract).await;

    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let _provisioned = provision_device(&auth, &deployment_id, &identity).await;

    let nonce = generate_nonce();
    let payload = trellis_rs::auth::build_device_activation_payload(
        &identity.activation_key_base64url,
        &identity.public_identity_key,
        &nonce,
    )
    .expect("build activation payload");

    let activation = trellis_rs::auth::start_device_activation_request(&trellis_url, &payload)
        .await
        .expect("start device activation request");

    let flow_id = url::Url::parse(&activation.activation_url)
        .expect("parse activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()))
        .expect("activation URL should contain a flowId");

    let resolve_op = auth
        .operation()
        .auth()
        .device_user_authorities_resolve()
        .start(&AuthDeviceUserAuthoritiesResolveInput {
            flow_id: flow_id.clone(),
        })
        .await
        .expect("start device user authorities resolve operation");

    let terminal = resolve_op
        .wait()
        .await
        .expect("wait for resolve operation to complete");
    assert_eq!(terminal.state, OperationState::Completed);

    let activated = trellis_rs::auth::wait_for_device_activation(
        trellis_rs::auth::WaitForDeviceActivationOpts {
            trellis_url: &trellis_url,
            flow_id: &flow_id,
            public_identity_key: &identity.public_identity_key,
            nonce: &nonce,
            identity_seed_base64url: &identity.identity_seed_base64url,
            contract_digest: Some(&device_contract_digest),
            poll_interval: Duration::from_millis(25),
        },
    )
    .await
    .expect("wait for device activation");

    assert_eq!(
        activated.pointer("/deploymentId").and_then(Value::as_str),
        Some(&deployment_id as &str)
    );
    assert_eq!(
        activated.pointer("/contractDigest").and_then(Value::as_str),
        Some(&device_contract_digest as &str)
    );
}

#[tokio::test]
async fn device_activation_activated_device_connects_and_authenticates() {
    assert_case_registered(
        "device-activation.activated-device-connects-and-authenticates",
        "device-activation",
        "device_activation",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");

    create_device_deployment(&auth, &deployment_id).await;
    let device_contract_digest =
        approve_device_contract(&auth, &deployment_id, &device_contract).await;

    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let _provisioned = provision_device(&auth, &deployment_id, &identity).await;

    let nonce = generate_nonce();
    let payload = trellis_rs::auth::build_device_activation_payload(
        &identity.activation_key_base64url,
        &identity.public_identity_key,
        &nonce,
    )
    .expect("build activation payload");

    let activation = trellis_rs::auth::start_device_activation_request(&trellis_url, &payload)
        .await
        .expect("start device activation request");

    let flow_id = url::Url::parse(&activation.activation_url)
        .expect("parse activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()))
        .expect("activation URL should contain a flowId");

    let resolve_op = auth
        .operation()
        .auth()
        .device_user_authorities_resolve()
        .start(&AuthDeviceUserAuthoritiesResolveInput {
            flow_id: flow_id.clone(),
        })
        .await
        .expect("start device user authorities resolve operation");

    let terminal = resolve_op
        .wait()
        .await
        .expect("wait for resolve operation to complete");
    assert_eq!(terminal.state, OperationState::Completed);

    let _connect_info = trellis_rs::auth::wait_for_device_activation(
        trellis_rs::auth::WaitForDeviceActivationOpts {
            trellis_url: &trellis_url,
            flow_id: &flow_id,
            public_identity_key: &identity.public_identity_key,
            nonce: &nonce,
            identity_seed_base64url: &identity.identity_seed_base64url,
            contract_digest: Some(&device_contract_digest),
            poll_interval: Duration::from_millis(25),
        },
    )
    .await
    .expect("wait for device activation");

    let device = trellis_rs::client::TrellisClient::connect_device(
        trellis_rs::client::DeviceConnectOptions {
            trellis_url: &trellis_url,
            contract_digest: &device_contract_digest,
            public_identity_key: &identity.public_identity_key,
            identity_seed_base64url: &identity.identity_seed_base64url,
            timeout_ms: 15_000,
        },
    )
    .await
    .expect("connect device client");
    device
        .flush()
        .await
        .expect("device NATS flush should succeed");

    let device_auth = GeneratedAuthClient::new(&device);
    let me = device_auth
        .rpc()
        .auth()
        .sessions_me()
        .await
        .expect("call Auth.Sessions.Me as device");

    assert_eq!(
        me.participant_kind.as_str(),
        Some("device"),
        "session should identify as device"
    );
    let device_info = me
        .device
        .as_object()
        .expect("device session should have device info");
    assert_eq!(
        device_info.get("deploymentId").and_then(Value::as_str),
        Some(&deployment_id as &str)
    );
    assert_eq!(
        device_info.get("runtimePublicKey").and_then(Value::as_str),
        Some(&identity.public_identity_key as &str)
    );
}

#[tokio::test]
async fn device_activation_activated_device_authority_is_listed() {
    assert_case_registered(
        "device-activation.activated-device-authority-is-listed",
        "device-activation",
        "device_activation",
    );

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);

    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");

    create_device_deployment(&auth, &deployment_id).await;
    let device_contract_digest =
        approve_device_contract(&auth, &deployment_id, &device_contract).await;

    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let provisioned = provision_device(&auth, &deployment_id, &identity).await;

    let nonce = generate_nonce();
    let payload = trellis_rs::auth::build_device_activation_payload(
        &identity.activation_key_base64url,
        &identity.public_identity_key,
        &nonce,
    )
    .expect("build activation payload");

    let activation = trellis_rs::auth::start_device_activation_request(&trellis_url, &payload)
        .await
        .expect("start device activation request");

    let flow_id = url::Url::parse(&activation.activation_url)
        .expect("parse activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()))
        .expect("activation URL should contain a flowId");

    let resolve_op = auth
        .operation()
        .auth()
        .device_user_authorities_resolve()
        .start(&AuthDeviceUserAuthoritiesResolveInput {
            flow_id: flow_id.clone(),
        })
        .await
        .expect("start device user authorities resolve operation");

    let terminal = resolve_op
        .wait()
        .await
        .expect("wait for resolve operation to complete");
    assert_eq!(terminal.state, OperationState::Completed);

    let _connect_info = trellis_rs::auth::wait_for_device_activation(
        trellis_rs::auth::WaitForDeviceActivationOpts {
            trellis_url: &trellis_url,
            flow_id: &flow_id,
            public_identity_key: &identity.public_identity_key,
            nonce: &nonce,
            identity_seed_base64url: &identity.identity_seed_base64url,
            contract_digest: Some(&device_contract_digest),
            poll_interval: Duration::from_millis(25),
        },
    )
    .await
    .expect("wait for device activation");

    let device = trellis_rs::client::TrellisClient::connect_device(
        trellis_rs::client::DeviceConnectOptions {
            trellis_url: &trellis_url,
            contract_digest: &device_contract_digest,
            public_identity_key: &identity.public_identity_key,
            identity_seed_base64url: &identity.identity_seed_base64url,
            timeout_ms: 15_000,
        },
    )
    .await
    .expect("connect device client");
    device
        .flush()
        .await
        .expect("device NATS flush should succeed");

    let device_auth = GeneratedAuthClient::new(&device);
    let me = device_auth
        .rpc()
        .auth()
        .sessions_me()
        .await
        .expect("call Auth.Sessions.Me as device");

    assert_eq!(
        me.participant_kind.as_str(),
        Some("device"),
        "session should identify as device"
    );

    let listings = auth
        .rpc()
        .auth()
        .device_user_authorities_list(&AuthDeviceUserAuthoritiesListRequest {
            deployment_id: Some(deployment_id.clone()),
            instance_id: Some(provisioned.instance.instance_id.clone()),
            limit: 20,
            offset: None,
            state: Some("activated".to_string()),
        })
        .await
        .expect("list device user authorities");

    let found = listings.entries.iter().any(|entry| {
        entry.instance_id == provisioned.instance.instance_id
            && entry.public_identity_key == identity.public_identity_key
            && entry.deployment_id == deployment_id
            && entry.state == "activated"
    });
    assert!(
        found,
        "activated device should be listed by Auth.DeviceUserAuthorities.List"
    );
}

#[tokio::test]
async fn auth_sessions_me_reports_device_envelope() {
    assert_case_registered(
        "auth.sessions-me-reports-device-envelope",
        "auth",
        "device_activation",
    );

    let (_runtime, mut admin, bootstrap_url, deployment_id, identity, provisioned, device) =
        connect_activated_device_for_auth_case().await;
    let me = GeneratedAuthClient::new(&device)
        .rpc()
        .auth()
        .sessions_me()
        .await
        .expect("call Auth.Sessions.Me as activated device");

    assert_eq!(me.participant_kind.as_str(), Some("device"));
    assert!(!me.user.is_null(), "activated device should include user");
    let device = me
        .device
        .as_object()
        .expect("device session should include device envelope");
    assert_eq!(
        device.get("deploymentId").and_then(Value::as_str),
        Some(deployment_id.as_str())
    );
    assert_eq!(
        device.get("runtimePublicKey").and_then(Value::as_str),
        Some(identity.public_identity_key.as_str())
    );
    assert_eq!(device.get("active").and_then(Value::as_bool), Some(true));
    assert!(me.service.is_null());

    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("reuse admin client");
    let sessions = GeneratedAuthClient::new(admin_client)
        .rpc()
        .auth()
        .sessions_list(&AuthSessionsListRequest {
            limit: 500,
            offset: None,
            user: None,
        })
        .await
        .expect("list sessions for activated device metadata");
    let device_session = sessions
        .entries
        .iter()
        .find(|entry| {
            entry.get("participantKind").and_then(Value::as_str) == Some("device")
                && entry.get("sessionKey").and_then(Value::as_str)
                    == Some(identity.public_identity_key.as_str())
        })
        .expect("Auth.Sessions.List should include activated device metadata row");
    assert_eq!(
        device_session
            .get("principal")
            .and_then(|principal| principal.get("type"))
            .and_then(Value::as_str),
        Some("device")
    );
    assert_eq!(
        device_session
            .get("principal")
            .and_then(|principal| principal.get("deviceId"))
            .and_then(Value::as_str),
        Some(provisioned.instance.instance_id.as_str())
    );
    assert_eq!(
        device_session
            .get("principal")
            .and_then(|principal| principal.get("deploymentId"))
            .and_then(Value::as_str),
        Some(deployment_id.as_str())
    );
    assert_eq!(
        device_session
            .get("principal")
            .and_then(|principal| principal.get("runtimePublicKey"))
            .and_then(Value::as_str),
        Some(identity.public_identity_key.as_str())
    );
    assert_eq!(
        device_session.get("contractId").and_then(Value::as_str),
        Some(DEVICE_CONTRACT_ID)
    );
}

#[tokio::test]
async fn auth_sessions_me_rejects_stale_device_principals() {
    assert_case_registered(
        "auth.sessions-me-rejects-stale-device-principals",
        "auth",
        "device_activation",
    );

    let (runtime, _admin, _bootstrap_url, deployment_id, identity, provisioned, device) =
        connect_activated_device_for_auth_case().await;
    let device_auth = GeneratedAuthClient::new(&device);
    device_auth
        .rpc()
        .auth()
        .sessions_me()
        .await
        .expect("call Auth.Sessions.Me before stale device mutations");

    let sqlite = runtime.control_plane_sqlite();
    let snapshot = sqlite
        .take_session(&identity.public_identity_key)
        .expect("delete device session row")
        .expect("device session row should exist");
    assert!(device_auth.rpc().auth().sessions_me().await.is_err());
    snapshot.restore().expect("restore device session row");

    sqlite
        .execute(
            "UPDATE device_instances SET deployment_id = ? WHERE instance_id = ?",
            params![
                format!("{deployment_id}.stale"),
                provisioned.instance.instance_id
            ],
        )
        .expect("make device activation deployment stale");
    assert!(device_auth.rpc().auth().sessions_me().await.is_err());

    sqlite
        .execute(
            "UPDATE device_instances SET deployment_id = ? WHERE instance_id = ?",
            params![deployment_id, provisioned.instance.instance_id],
        )
        .expect("restore device instance deployment");
    sqlite
        .execute(
            "UPDATE device_activations SET public_identity_key = ? WHERE instance_id = ?",
            params![
                format!("B{}", &identity.public_identity_key[1..]),
                provisioned.instance.instance_id
            ],
        )
        .expect("make device activation identity key stale");
    assert!(device_auth.rpc().auth().sessions_me().await.is_err());
}

async fn connect_activated_device_for_auth_case() -> (
    trellis_test::TrellisTestRuntime,
    trellis_test::TrellisTestAdmin,
    String,
    String,
    trellis_rs::auth::DeviceIdentity,
    trellis_rs::sdk::auth::types::AuthDevicesProvisionResponse,
    trellis_rs::client::TrellisClient,
) {
    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let trellis_url = runtime.trellis_url().to_string();
    let mut admin = runtime.admin();
    let admin_client = admin
        .connect_admin(&bootstrap_url)
        .await
        .expect("connect admin client");
    let auth = GeneratedAuthClient::new(admin_client);
    let deployment_id = generate_deployment_id();
    let device_contract = device_contract().expect("build device contract");

    create_device_deployment(&auth, &deployment_id).await;
    let device_contract_digest =
        approve_device_contract(&auth, &deployment_id, &device_contract).await;
    let root_secret = device_root_secret();
    let identity =
        trellis_rs::auth::derive_device_identity(&root_secret).expect("derive device identity");
    let provisioned = provision_device(&auth, &deployment_id, &identity).await;
    let nonce = generate_nonce();
    let payload = trellis_rs::auth::build_device_activation_payload(
        &identity.activation_key_base64url,
        &identity.public_identity_key,
        &nonce,
    )
    .expect("build activation payload");
    let activation = trellis_rs::auth::start_device_activation_request(&trellis_url, &payload)
        .await
        .expect("start device activation request");
    let flow_id = url::Url::parse(&activation.activation_url)
        .expect("parse activation URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "flowId").then(|| value.into_owned()))
        .expect("activation URL should contain a flowId");
    let resolve_op = auth
        .operation()
        .auth()
        .device_user_authorities_resolve()
        .start(&AuthDeviceUserAuthoritiesResolveInput {
            flow_id: flow_id.clone(),
        })
        .await
        .expect("start device user authorities resolve operation");
    let terminal = resolve_op
        .wait()
        .await
        .expect("wait for resolve operation to complete");
    assert_eq!(terminal.state, OperationState::Completed);
    trellis_rs::auth::wait_for_device_activation(trellis_rs::auth::WaitForDeviceActivationOpts {
        trellis_url: &trellis_url,
        flow_id: &flow_id,
        public_identity_key: &identity.public_identity_key,
        nonce: &nonce,
        identity_seed_base64url: &identity.identity_seed_base64url,
        contract_digest: Some(&device_contract_digest),
        poll_interval: Duration::from_millis(25),
    })
    .await
    .expect("wait for device activation");
    let device = trellis_rs::client::TrellisClient::connect_device(
        trellis_rs::client::DeviceConnectOptions {
            trellis_url: &trellis_url,
            contract_digest: &device_contract_digest,
            public_identity_key: &identity.public_identity_key,
            identity_seed_base64url: &identity.identity_seed_base64url,
            timeout_ms: 15_000,
        },
    )
    .await
    .expect("connect device client");
    device
        .flush()
        .await
        .expect("device NATS flush should succeed");
    drop(auth);

    (
        runtime,
        admin,
        bootstrap_url,
        deployment_id,
        identity,
        provisioned,
        device,
    )
}
