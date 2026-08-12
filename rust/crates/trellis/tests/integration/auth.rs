use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::task::JoinHandle;
use trellis_rs::client::{EventDescriptor, RpcDescriptor};
use trellis_rs::generated::Caller;
use trellis_rs::sdk::auth as auth_sdk;
use trellis_rs::service::{
    ConnectedServiceRuntime, ServiceEventListenOptions, ServiceEventListenerMode,
};

use crate::support::assertions::assert_runtime_case_registered;

const SERVICE_ID: &str = "trellis.integration.trusted-portal-service@v1";
const CLIENT_ID: &str = "trellis.integration.trusted-portal-client@v1";
const READ_CAPABILITY: &str = "trellis.integration.trusted-portal-service::read";
const PUBLISH_CAPABILITY: &str = "trellis.integration.trusted-portal-service::publish";
const API_SOURCE: &str = r#"{
  "format": "trellis.api.v1",
  "id": "trellis.integration.trusted-portal-service@v1",
  "displayName": "Trusted Portal Integration Service",
  "description": "Exercises trusted-portal authority transitions.",
  "capabilities": {
    "trellis.integration.trusted-portal-service::read": {"allows": [
      {"target": {"kind": "apiSurface", "api": "trellis.integration.trusted-portal-service@v1", "surface": "rpc", "name": "Value.Get"}, "action": "call"}
    ]},
    "trellis.integration.trusted-portal-service::publish": {"allows": [
      {"target": {"kind": "apiSurface", "api": "trellis.integration.trusted-portal-service@v1", "surface": "event", "name": "Value.Changed"}, "action": "publish"}
    ]}
  },
  "schemas": {
    "ValueGetInput": {"type": "object", "required": ["value"], "properties": {"value": {"type": "string"}}},
    "ValueGetOutput": {"type": "object", "required": ["value"], "properties": {"value": {"type": "string"}}}
  },
  "rpc": {
    "Value.Get": {"version": "v1", "input": {"schema": "ValueGetInput"}, "output": {"schema": "ValueGetOutput"}, "errors": []}
  },
  "events": {
    "Value.Changed": {"version": "v1", "event": {"schema": "ValueGetOutput"}}
  }
}"#;

struct TrustedPortalServiceContract;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct ValueMessage {
    value: String,
}

struct ValueGet;

struct ValueChanged;

struct TrustedPortalListenerContract;

impl RpcDescriptor for ValueGet {
    type Input = ValueMessage;
    type Output = ValueMessage;

    const KEY: &'static str = "Value.Get";
    const SUBJECT: &'static str = "rpc.v1.Value.Get";
    const CALLER_CAPABILITIES: &'static [&'static str] = &[READ_CAPABILITY];
    const ERRORS: &'static [&'static str] = &[];
    const INPUT_SCHEMA_JSON: &'static str =
        r#"{"type":"object","required":["value"],"properties":{"value":{"type":"string"}}}"#;
    const OUTPUT_SCHEMA_JSON: &'static str = Self::INPUT_SCHEMA_JSON;
}

impl EventDescriptor for ValueChanged {
    type Event = ValueMessage;

    const KEY: &'static str = "Value.Changed";
    const SUBJECT: &'static str = "events.v1.Value.Changed";
    const PUBLISH_CAPABILITIES: &'static [&'static str] = &[PUBLISH_CAPABILITY];
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &[];
    const EVENT_SCHEMA_JSON: &'static str = ValueGet::OUTPUT_SCHEMA_JSON;
}

struct AbortOnDrop(Option<JoinHandle<Result<(), trellis_rs::service::ServiceRuntimeError>>>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        if let Some(handle) = &self.0 {
            handle.abort();
        }
    }
}

struct Fixture {
    runtime: trellis_test::TrellisTestRuntime,
    admin: trellis_test::TrellisTestAdmin,
    bootstrap_url: String,
    portal_id: String,
    client_contract: trellis_test::TrellisTestContract,
    read_capability: String,
    publish_capability: String,
    _service: AbortOnDrop,
}

async fn start_fixture(user_jwt_ttl_ms: Option<u64>, use_test_oidc_provider: bool) -> Fixture {
    let mut options = trellis_test::TrellisTestRuntimeOptions::repo_platform();
    options.nats_user_jwt_ttl_ms = user_jwt_ttl_ms;
    options.use_shared_test_oidc_provider = use_test_oidc_provider;
    let runtime = trellis_test::TrellisTestRuntime::start(options)
        .await
        .expect("start isolated trusted-portal runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first administrator bootstrap URL");
    let service_contract = trellis_test::TrellisTestContract::from_native_api_json(
        API_SOURCE,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build trusted-portal service contract");
    let client_contract =
        trellis_test::TrellisTestContract::from_builder_with_referenced_contracts(
            trellis_rs::contracts::ContractBuilder::authoring(
                CLIENT_ID,
                "Trusted Portal Integration Client",
                "Exercises optional authority selected by a trusted portal.",
                trellis_rs::contracts::ContractKind::App,
            )
            .optional_use_ref(
                "service",
                trellis_rs::contracts::use_contract(SERVICE_ID)
                    .with_rpc_call(["Value.Get"])
                    .with_event_publish(["Value.Changed"]),
            ),
            &[&service_contract],
        )
        .expect("build trusted-portal client contract");
    let read_capability = format!(
        "{}::read",
        runtime
            .scoped_contract(&service_contract)
            .expect("scope trusted-portal service contract")
            .id()
            .strip_suffix("@v1")
            .expect("versioned trusted-portal service ID")
    );
    let publish_capability = read_capability.replace("::read", "::publish");
    let mut admin = runtime.admin();
    let participant_id = runtime
        .scoped_contract(&client_contract)
        .expect("scope trusted-portal client contract")
        .id()
        .to_owned();
    let portal_id = format!("portal-{participant_id}");
    admin
        .put_test_login_portal(
            &bootstrap_url,
            &portal_id,
            &participant_id,
            vec!["local".to_owned(), "test-oidc".to_owned()],
        )
        .await
        .expect("create participant-scoped login portal");
    let key = admin
        .provision_service_instance(&bootstrap_url, &service_contract, None, None)
        .await
        .expect("provision trusted-portal service");
    let mut service: ConnectedServiceRuntime<TrustedPortalServiceContract> =
        trellis_test::connect_service_runtime(runtime.trellis_url(), &key)
            .await
            .expect("connect trusted-portal service");
    service.register_rpc::<ValueGet, _, _>(|_, input| async move { Ok(input) });
    let service = AbortOnDrop(Some(tokio::spawn(async move { service.run().await })));
    Fixture {
        runtime,
        admin,
        bootstrap_url,
        portal_id,
        client_contract,
        read_capability,
        publish_capability,
        _service: service,
    }
}

fn authority_rows(
    runtime: &trellis_test::TrellisTestRuntime,
    session_id: &str,
) -> Vec<Map<String, Value>> {
    runtime
        .control_plane_sqlite()
        .query(
            "SELECT a.principal_id, a.participant_id, a.state, a.version, a.desired_capabilities_json FROM auth_identity_authorities a JOIN auth_sessions s ON s.principal_id = a.principal_id AND s.participant_id = a.participant_id WHERE s.session_id = ?",
            [session_id],
        )
        .expect("query trusted portal authority")
}

fn binding_rows(
    runtime: &trellis_test::TrellisTestRuntime,
    session_id: &str,
) -> Vec<Map<String, Value>> {
    runtime
        .control_plane_sqlite()
        .query(
            "SELECT b.principal_id, b.provider_id, b.roles_json, b.authority_version FROM auth_portal_authority_bindings b JOIN auth_sessions s ON s.principal_id = b.principal_id AND s.participant_id = b.participant_id WHERE s.session_id = ?",
            [session_id],
        )
        .expect("query trusted portal binding")
}

fn portal_policy_version(
    runtime: &trellis_test::TrellisTestRuntime,
    portal_id: &str,
    participant_id: &str,
) -> Option<i64> {
    runtime
        .control_plane_sqlite()
        .query(
            "SELECT version FROM auth_portal_grant_overrides WHERE portal_id = ? AND participant_id = ?",
            [portal_id, participant_id],
        )
        .expect("query portal grant override")
        .first()
        .and_then(|row| row["version"].as_i64())
}

fn context_states(runtime: &trellis_test::TrellisTestRuntime, session_id: &str) -> Vec<String> {
    runtime
        .control_plane_sqlite()
        .query(
            "SELECT state FROM auth_authorization_contexts WHERE session_id = ? ORDER BY version",
            [session_id],
        )
        .expect("query authorization contexts")
        .into_iter()
        .map(|row| row["state"].as_str().expect("context state").to_owned())
        .collect()
}

fn session_count(runtime: &trellis_test::TrellisTestRuntime, session_id: &str) -> usize {
    runtime
        .control_plane_sqlite()
        .query(
            "SELECT session_id FROM auth_sessions WHERE session_id = ?",
            [session_id],
        )
        .expect("query retained session")
        .len()
}

async fn list_connections(
    fixture: &mut Fixture,
    session_id: &str,
) -> Vec<auth_sdk::types::AuthConnectionsListResponseEntriesItem> {
    fixture
        .admin
        .list_connections(&fixture.bootstrap_url, session_id)
        .await
        .expect("list physical connections")
}

async fn update_policy(fixture: &mut Fixture, capabilities: Vec<String>) {
    let participant_id = fixture
        .runtime
        .scoped_contract(&fixture.client_contract)
        .expect("scope trusted portal client")
        .id()
        .to_owned();
    let version = portal_policy_version(&fixture.runtime, &fixture.portal_id, &participant_id)
        .expect("trusted portal grant override");
    fixture
        .admin
        .put_portal_grant_override(
            &fixture.bootstrap_url,
            &fixture.portal_id,
            &participant_id,
            Some(version),
            capabilities,
        )
        .await
        .expect("update trusted portal policy");
}

async fn wait_for_context_refresh(client: &Caller) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match client.refresh_authorization_context().await {
            Ok(_) => return,
            Err(_) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(error) => panic!("authorization context did not refresh: {error}"),
        }
    }
}

async fn wait_for_no_connections(fixture: &mut Fixture, session_id: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !list_connections(fixture, session_id).await.is_empty() {
        assert!(
            Instant::now() < deadline,
            "physical connections were not removed"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_authority(
    runtime: &trellis_test::TrellisTestRuntime,
    session_id: &str,
    state: &str,
    capabilities: &[&str],
) -> Map<String, Value> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let observed = authority_rows(runtime, session_id);
        if let Some(row) = observed.first().cloned() {
            let selected = row["desired_capabilities_json"]
                .as_str()
                .and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
                .unwrap_or_default();
            if row["state"] == state && selected == capabilities {
                return row;
            }
        }
        assert!(
            Instant::now() < deadline,
            "authority did not converge: {observed:?}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn call_value(client: &Caller, value: &str) -> Result<ValueMessage, String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match client
            .call::<ValueGet>(&ValueMessage {
                value: value.to_owned(),
            })
            .await
        {
            Ok(response) => return Ok(response),
            Err(_) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

#[tokio::test]
async fn trusted_local_registration_applies_portal_policy() {
    assert_runtime_case_registered(
        "auth.trusted-local-registration-applies-portal-policy",
        "auth",
        "auth",
    );
    let mut fixture = start_fixture(None, false).await;
    let (client, reconnect) = fixture
        .admin
        .connect_new_trusted_local_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            &fixture.portal_id,
            "trusted-local-user",
            "trusted-local-password-123",
            vec![fixture.read_capability.clone()],
        )
        .await
        .expect("trusted local registration connects without consent");

    drop(client);
    assert_eq!(
        wait_for_authority(
            &fixture.runtime,
            reconnect.session_id(),
            "accepted",
            &[fixture.read_capability.as_str()]
        )
        .await["version"],
        1
    );
    let bindings = binding_rows(&fixture.runtime, reconnect.session_id());
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0]["provider_id"], "local");
    assert_eq!(bindings[0]["roles_json"], "[]");
    assert_eq!(bindings[0]["authority_version"], 1);
    assert!(!reconnect.session_id().is_empty());
}

#[tokio::test]
async fn portal_policy_removal_requires_new_login() {
    assert_runtime_case_registered(
        "auth.portal-policy-removal-requires-new-login",
        "auth",
        "auth",
    );
    let mut fixture = start_fixture(None, false).await;
    let (_, reconnect) = fixture
        .admin
        .connect_new_trusted_local_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            &fixture.portal_id,
            "policy-removal-user",
            "policy-removal-password-123",
            vec![fixture.read_capability.clone()],
        )
        .await
        .expect("trusted local registration");
    let session_id = reconnect.session_id().to_owned();
    let participant_id = fixture
        .runtime
        .scoped_contract(&fixture.client_contract)
        .expect("scope trusted portal client")
        .id()
        .to_owned();
    let version = portal_policy_version(&fixture.runtime, &fixture.portal_id, &participant_id)
        .expect("trusted portal grant override");

    fixture
        .admin
        .remove_portal_grant_override(
            &fixture.bootstrap_url,
            &fixture.portal_id,
            &participant_id,
            version,
        )
        .await
        .expect("remove trusted portal policy");

    let revoked = wait_for_authority(
        &fixture.runtime,
        &session_id,
        "revoked",
        &[fixture.read_capability.as_str()],
    )
    .await;
    assert!(binding_rows(&fixture.runtime, &session_id).is_empty());
    assert!(context_states(&fixture.runtime, &session_id)
        .iter()
        .all(|state| state == "revoked"));
    assert!(reconnect.connect_bound_only().await.is_err());

    fixture
        .admin
        .put_portal_grant_override(
            &fixture.bootstrap_url,
            &fixture.portal_id,
            &participant_id,
            None,
            vec![fixture.read_capability.clone()],
        )
        .await
        .expect("restore trusted portal policy");
    tokio::time::sleep(Duration::from_secs(2)).await;
    let still_revoked = authority_rows(&fixture.runtime, &session_id)
        .into_iter()
        .next()
        .expect("retained revoked authority");
    assert_eq!(still_revoked["state"], "revoked");
    assert_eq!(still_revoked["version"], revoked["version"]);
    assert!(binding_rows(&fixture.runtime, &session_id).is_empty());

    let (_, fresh) = fixture
        .admin
        .connect_local_user_for_portal_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            "policy-removal-user",
            "policy-removal-password-123",
        )
        .await
        .expect("fresh verified login restores authority");
    let restored = wait_for_authority(
        &fixture.runtime,
        fresh.session_id(),
        "accepted",
        &[fixture.read_capability.as_str()],
    )
    .await;
    assert!(restored["version"].as_i64() > revoked["version"].as_i64());
    assert_eq!(binding_rows(&fixture.runtime, fresh.session_id()).len(), 1);
}

#[tokio::test]
async fn portal_policy_reduction_and_expansion_converge() {
    assert_runtime_case_registered(
        "auth.portal-policy-reduction-and-expansion-converge",
        "auth",
        "auth",
    );
    let mut fixture = start_fixture(None, false).await;
    let (client, reconnect) = fixture
        .admin
        .connect_new_trusted_local_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            &fixture.portal_id,
            "policy-convergence-user",
            "policy-convergence-password-123",
            vec![fixture.read_capability.clone()],
        )
        .await
        .expect("trusted local registration");
    let session_id = reconnect.session_id().to_owned();

    update_policy(&mut fixture, Vec::new()).await;
    let reduced = wait_for_authority(&fixture.runtime, &session_id, "accepted", &[]).await;
    assert_eq!(session_count(&fixture.runtime, &session_id), 1);

    let read_capability = fixture.read_capability.clone();
    update_policy(&mut fixture, vec![read_capability]).await;
    let expanded = wait_for_authority(
        &fixture.runtime,
        &session_id,
        "accepted",
        &[fixture.read_capability.as_str()],
    )
    .await;
    assert!(expanded["version"].as_i64() > reduced["version"].as_i64());
    wait_for_context_refresh(&client).await;
    assert_eq!(
        call_value(&client, "expanded").await.unwrap().value,
        "expanded"
    );
    assert_eq!(session_count(&fixture.runtime, &session_id), 1);
}

#[tokio::test]
async fn hostile_old_context_is_denied_after_reduction() {
    assert_runtime_case_registered(
        "auth.hostile-old-context-and-proofs-are-denied",
        "auth",
        "auth",
    );
    let mut fixture = start_fixture(None, false).await;
    let service_contract = trellis_test::TrellisTestContract::from_native_api_json(
        API_SOURCE,
        trellis_rs::contracts::ContractKind::Service,
    )
    .expect("build replay service contract");
    let listener_contract =
        trellis_test::TrellisTestContract::from_builder_with_referenced_contracts(
            trellis_rs::contracts::ContractBuilder::authoring(
                "trellis.integration.trusted-portal-listener@v1",
                "Trusted Portal Replay Listener",
                "Receives trusted-portal replay attempts through local proof validation.",
                trellis_rs::contracts::ContractKind::Service,
            )
            .use_ref(
                "service",
                trellis_rs::contracts::use_contract(SERVICE_ID)
                    .with_event_subscribe(["Value.Changed"]),
            ),
            &[&service_contract],
        )
        .expect("build trusted-portal replay listener contract");
    let listener_key = fixture
        .admin
        .provision_service_instance(
            &fixture.bootstrap_url,
            &listener_contract,
            Some("trusted-portal-replay-listener"),
            None,
        )
        .await
        .expect("provision trusted-portal replay listener");
    let listener_service = trellis_test::connect_service_runtime::<TrustedPortalListenerContract>(
        fixture.runtime.trellis_url(),
        &listener_key,
    )
    .await
    .expect("connect trusted-portal replay listener");
    let delivered = Arc::new(AtomicUsize::new(0));
    let handler_delivered = Arc::clone(&delivered);
    let listener = listener_service
        .listen_event_with_api_id::<ValueChanged, _, _>(
            SERVICE_ID,
            move |_, _| {
                let handler_delivered = Arc::clone(&handler_delivered);
                async move {
                    handler_delivered.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Ephemeral,
                group: None,
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start trusted-portal replay listener");
    let _listener = listener;

    let observer = async_nats::ConnectOptions::new()
        .credentials_file(
            fixture
                .runtime
                .workdir()
                .join("nats/creds/trellis-auth.creds"),
        )
        .await
        .expect("load trusted-portal observer credentials")
        .connect(fixture.runtime.nats_url())
        .await
        .expect("connect trusted-portal raw observer");
    let mut rpc_observer = observer
        .subscribe(
            fixture
                .runtime
                .integration_test_descriptor_subject(ValueGet::SUBJECT),
        )
        .await
        .expect("subscribe to raw RPC requests");
    let mut event_observer = observer
        .subscribe(
            fixture
                .runtime
                .integration_test_descriptor_subject(ValueChanged::SUBJECT),
        )
        .await
        .expect("subscribe to raw events");
    observer.flush().await.expect("flush raw observers");

    let capabilities = vec![
        fixture.read_capability.clone(),
        fixture.publish_capability.clone(),
    ];
    let (client, reconnect) = fixture
        .admin
        .connect_new_trusted_local_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            &fixture.portal_id,
            "hostile-context-user",
            "hostile-context-password-123",
            capabilities,
        )
        .await
        .expect("trusted local registration");
    let session_id = reconnect.session_id().to_owned();
    let old_contexts = fixture
        .runtime
        .control_plane_sqlite()
        .query(
            "SELECT context_digest FROM auth_authorization_contexts WHERE session_id = ? AND state = 'active'",
            [&session_id],
        )
        .expect("query old authorization context");
    assert_eq!(old_contexts.len(), 1);
    assert_eq!(
        call_value(&client, "before-reduction").await.unwrap().value,
        "before-reduction"
    );
    let signed_rpc = tokio::time::timeout(Duration::from_secs(5), rpc_observer.next())
        .await
        .expect("observe signed RPC request")
        .expect("signed RPC message");
    client
        .publish::<ValueChanged>(&ValueMessage {
            value: "before-reduction".to_owned(),
        })
        .await
        .expect("publish signed event before reduction");
    let signed_event = tokio::time::timeout(Duration::from_secs(5), event_observer.next())
        .await
        .expect("observe signed event")
        .expect("signed event message");
    let deadline = Instant::now() + Duration::from_secs(5);
    while delivered.load(Ordering::SeqCst) != 1 {
        assert!(
            Instant::now() < deadline,
            "fresh signed event was not delivered"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    update_policy(&mut fixture, Vec::new()).await;
    wait_for_authority(&fixture.runtime, &session_id, "accepted", &[]).await;
    assert_eq!(context_states(&fixture.runtime, &session_id), ["revoked"]);
    wait_for_no_connections(&mut fixture, &session_id).await;
    drop(client);
    assert!(reconnect
        .connect_captured_admission(old_contexts[0]["context_digest"].as_str().unwrap())
        .await
        .is_err());
    assert!(list_connections(&mut fixture, &session_id).await.is_empty());
    let reply = signed_rpc.reply.expect("signed RPC reply subject");
    let mut reply_observer = observer
        .subscribe(reply.clone())
        .await
        .expect("subscribe to replay reply");
    observer
        .publish_with_reply_and_headers(
            signed_rpc.subject,
            reply,
            signed_rpc.headers.expect("signed RPC headers"),
            signed_rpc.payload,
        )
        .await
        .expect("replay exact signed RPC request");
    if let Ok(Some(response)) =
        tokio::time::timeout(Duration::from_secs(1), reply_observer.next()).await
    {
        assert_ne!(
            serde_json::from_slice::<ValueMessage>(&response.payload).ok(),
            Some(ValueMessage {
                value: "before-reduction".to_owned(),
            }),
            "stale signed RPC proof reached the handler"
        );
    }
    observer
        .publish_with_headers(
            signed_event.subject,
            signed_event.headers.expect("signed event headers"),
            signed_event.payload,
        )
        .await
        .expect("replay exact signed event");
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert_eq!(
        delivered.load(Ordering::SeqCst),
        1,
        "stale signed event proof reached the handler"
    );
    assert_eq!(session_count(&fixture.runtime, &session_id), 1);
}

#[tokio::test]
async fn portal_reconciliation_converges_a_b_a() {
    assert_runtime_case_registered("auth.portal-reconciliation-a-b-a-converges", "auth", "auth");
    let mut fixture = start_fixture(None, false).await;
    let (_, reconnect) = fixture
        .admin
        .connect_new_trusted_local_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            &fixture.portal_id,
            "aba-user",
            "aba-password-123",
            vec![fixture.read_capability.clone()],
        )
        .await
        .expect("trusted local registration");
    let session_id = reconnect.session_id().to_owned();
    let a1 = wait_for_authority(
        &fixture.runtime,
        &session_id,
        "accepted",
        &[fixture.read_capability.as_str()],
    )
    .await;
    update_policy(&mut fixture, Vec::new()).await;
    let b = wait_for_authority(&fixture.runtime, &session_id, "accepted", &[]).await;
    let read_capability = fixture.read_capability.clone();
    update_policy(&mut fixture, vec![read_capability]).await;
    let a2 = wait_for_authority(
        &fixture.runtime,
        &session_id,
        "accepted",
        &[fixture.read_capability.as_str()],
    )
    .await;
    assert!(a1["version"].as_i64() < b["version"].as_i64());
    assert!(b["version"].as_i64() < a2["version"].as_i64());
    assert_eq!(binding_rows(&fixture.runtime, &session_id).len(), 1);
}

#[tokio::test]
async fn concurrent_portal_provenance_converges() {
    assert_runtime_case_registered(
        "auth.concurrent-portal-provenance-converges",
        "auth",
        "auth",
    );
    let mut fixture = start_fixture(None, false).await;
    let (_, initial) = fixture
        .admin
        .connect_new_trusted_local_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            &fixture.portal_id,
            "concurrent-user",
            "concurrent-password-123",
            vec![fixture.read_capability.clone()],
        )
        .await
        .expect("trusted local registration");
    let initial_authority = wait_for_authority(
        &fixture.runtime,
        initial.session_id(),
        "accepted",
        &[fixture.read_capability.as_str()],
    )
    .await;
    let mut first_admin = fixture.runtime.admin();
    let mut second_admin = fixture.runtime.admin();
    let first_login = first_admin.connect_local_user_for_portal_reconnectable(
        &fixture.bootstrap_url,
        &fixture.client_contract,
        "concurrent-user",
        "concurrent-password-123",
    );
    let second_login = second_admin.connect_local_user_for_portal_reconnectable(
        &fixture.bootstrap_url,
        &fixture.client_contract,
        "concurrent-user",
        "concurrent-password-123",
    );
    let (first, second) = tokio::join!(first_login, second_login);
    let (_, first) = first.expect("first concurrent verified login");
    let (_, second) = second.expect("second concurrent verified login");
    let authority = wait_for_authority(
        &fixture.runtime,
        second.session_id(),
        "accepted",
        &[fixture.read_capability.as_str()],
    )
    .await;
    assert_eq!(authority["version"], initial_authority["version"]);
    assert_eq!(binding_rows(&fixture.runtime, first.session_id()).len(), 1);
    assert_eq!(binding_rows(&fixture.runtime, second.session_id()).len(), 1);
}

#[tokio::test]
async fn oidc_role_mapping_converges_authority() {
    assert_runtime_case_registered("auth.oidc-role-mapping-converges-authority", "auth", "auth");
    let mut fixture = start_fixture(None, true).await;
    let participant_id = fixture
        .runtime
        .scoped_contract(&fixture.client_contract)
        .expect("scope trusted portal client")
        .id()
        .to_owned();
    let suffix = ulid::Ulid::new().to_string().to_ascii_lowercase();
    let leaf = format!("oidc-leaf-{suffix}");
    let parent = format!("oidc-parent-{suffix}");
    fixture
        .admin
        .put_capability_group(
            &fixture.bootstrap_url,
            &leaf,
            vec![fixture.read_capability.clone()],
            Vec::new(),
        )
        .await
        .expect("put OIDC leaf capability group");
    fixture
        .admin
        .put_capability_group(&fixture.bootstrap_url, &parent, Vec::new(), vec![leaf])
        .await
        .expect("put recursive OIDC capability group");
    fixture
        .admin
        .put_portal_role_mappings(
            &fixture.bootstrap_url,
            &fixture.portal_id,
            &participant_id,
            None,
            vec![
                auth_sdk::types::AuthPortalsGrantOverridesPutRequestRoleMappingsItem {
                    capability_group_keys: Vec::new(),
                    direct_capabilities: vec![fixture.read_capability.clone()],
                    provider_id: "test-oidc".to_owned(),
                    role: "direct".to_owned(),
                },
                auth_sdk::types::AuthPortalsGrantOverridesPutRequestRoleMappingsItem {
                    capability_group_keys: vec![parent],
                    direct_capabilities: Vec::new(),
                    provider_id: "test-oidc".to_owned(),
                    role: "group".to_owned(),
                },
                auth_sdk::types::AuthPortalsGrantOverridesPutRequestRoleMappingsItem {
                    capability_group_keys: Vec::new(),
                    direct_capabilities: vec![fixture.read_capability.clone()],
                    provider_id: "test-oidc".to_owned(),
                    role: "same-authority".to_owned(),
                },
                auth_sdk::types::AuthPortalsGrantOverridesPutRequestRoleMappingsItem {
                    capability_group_keys: Vec::new(),
                    direct_capabilities: vec![fixture.read_capability.clone()],
                    provider_id: "other-oidc".to_owned(),
                    role: "provider-scoped".to_owned(),
                },
            ],
        )
        .await
        .expect("put OIDC role mappings");

    fixture
        .admin
        .set_test_oidc_claims(serde_json::json!({ "roles": ["direct"] }))
        .await
        .expect("set direct OIDC role");
    let (_, direct) = fixture
        .admin
        .connect_oidc_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            "test-oidc",
        )
        .await
        .expect("connect direct-role OIDC user");
    let direct_authority = wait_for_authority(
        &fixture.runtime,
        direct.session_id(),
        "accepted",
        &[fixture.read_capability.as_str()],
    )
    .await;
    let direct_binding = binding_rows(&fixture.runtime, direct.session_id());
    assert_eq!(direct_binding[0]["provider_id"], "test-oidc");
    assert_eq!(direct_binding[0]["roles_json"], "[\"direct\"]");

    fixture
        .admin
        .set_test_oidc_claims(serde_json::json!({ "roles": ["group"] }))
        .await
        .expect("set recursive-group OIDC role");
    let (_, grouped) = fixture
        .admin
        .connect_oidc_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            "test-oidc",
        )
        .await
        .expect("connect recursive-group OIDC user");
    let grouped_authority = wait_for_authority(
        &fixture.runtime,
        grouped.session_id(),
        "accepted",
        &[fixture.read_capability.as_str()],
    )
    .await;
    assert_eq!(grouped_authority["version"], direct_authority["version"]);
    assert_eq!(
        binding_rows(&fixture.runtime, grouped.session_id())[0]["roles_json"],
        "[\"group\"]"
    );

    fixture
        .admin
        .set_test_oidc_claims(serde_json::json!({ "roles": ["same-authority"] }))
        .await
        .expect("set authority-equivalent OIDC role");
    let (_, equivalent) = fixture
        .admin
        .connect_oidc_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            "test-oidc",
        )
        .await
        .expect("connect authority-equivalent OIDC user");
    let equivalent_authority = wait_for_authority(
        &fixture.runtime,
        equivalent.session_id(),
        "accepted",
        &[fixture.read_capability.as_str()],
    )
    .await;
    assert_eq!(equivalent_authority["version"], direct_authority["version"]);
    assert_eq!(
        binding_rows(&fixture.runtime, equivalent.session_id())[0]["roles_json"],
        "[\"same-authority\"]"
    );

    fixture
        .admin
        .set_test_oidc_claims(serde_json::json!({ "roles": ["provider-scoped"] }))
        .await
        .expect("set provider-scoped OIDC role");
    let (_, scoped) = fixture
        .admin
        .connect_oidc_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            "test-oidc",
        )
        .await
        .expect("connect provider-scoped OIDC user");
    wait_for_authority(&fixture.runtime, scoped.session_id(), "accepted", &[]).await;

    fixture
        .admin
        .set_test_oidc_claims(serde_json::json!({ "roles": { "invalid": true } }))
        .await
        .expect("set malformed OIDC role claim");
    assert!(fixture
        .admin
        .connect_oidc_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            "test-oidc",
        )
        .await
        .is_err());
}

#[tokio::test]
async fn same_session_connections_survive_jwt_window_and_are_kicked() {
    assert_runtime_case_registered(
        "auth.same-session-multiple-connections-are-kicked",
        "auth",
        "auth",
    );
    let mut fixture = start_fixture(Some(10_000), false).await;
    assert_eq!(
        fixture
            .runtime
            .auth_connection_presence_status()
            .await
            .expect("inspect connection-presence bucket")
            .max_age,
        Duration::from_secs(70)
    );
    let (first, reconnect) = fixture
        .admin
        .connect_new_trusted_local_user_reconnectable(
            &fixture.bootstrap_url,
            &fixture.client_contract,
            &fixture.portal_id,
            "presence-user",
            "presence-password-123",
            vec![fixture.read_capability.clone()],
        )
        .await
        .expect("trusted local registration");
    let second = reconnect
        .connect_bound_only()
        .await
        .expect("second physical connection");
    let session_id = reconnect.session_id().to_owned();
    let initial = list_connections(&mut fixture, &session_id).await;
    assert_eq!(initial.len(), 2);
    assert_ne!(initial[0].connection_id, initial[1].connection_id);

    tokio::time::sleep(Duration::from_secs(3)).await;
    assert_eq!(list_connections(&mut fixture, &session_id).await.len(), 2);

    update_policy(&mut fixture, Vec::new()).await;
    wait_for_authority(&fixture.runtime, &session_id, "accepted", &[]).await;
    let deadline = Instant::now() + Duration::from_secs(10);
    while !list_connections(&mut fixture, &session_id).await.is_empty() {
        assert!(
            Instant::now() < deadline,
            "physical connections were not removed"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    drop((first, second));
    assert_eq!(session_count(&fixture.runtime, &session_id), 1);
}
