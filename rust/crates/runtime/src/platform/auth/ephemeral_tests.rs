use std::process::{Child, Command, Stdio};

use super::*;

const DIGEST: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

#[test]
fn connection_kick_response_rejects_system_errors() {
    validate_connection_kick_response(br#"{"server":{}}"#).expect("successful response");
    validate_connection_kick_response(
        br#"{"error":{"code":500,"description":"no such client or leafnode id"}}"#,
    )
    .expect("already disconnected response");
    assert!(validate_connection_kick_response(
        br#"{"error":{"code":403,"description":"permission denied"}}"#
    )
    .is_err());
}

fn browser_flow() -> AuthBrowserFlow {
    let consent_view = serde_json::json!({ "title": "Authorize app" });
    let required_grant_set = GrantSet::new(Vec::new());
    let optional_grant_bundles = BTreeMap::new();
    let required_capabilities = Vec::<String>::new();
    let optional_capability_definitions = BTreeMap::new();
    AuthBrowserFlow {
        format: BROWSER_FLOW_FORMAT.to_owned(),
        flow_id: "flow-1".to_owned(),
        kind: AuthBrowserFlowKind::UserAuth,
        state: AuthBrowserFlowState::ChooseProvider,
        request_id: "request-1".to_owned(),
        request_digest: DIGEST.to_owned(),
        participant_id: "app-1".to_owned(),
        participant_artifact_digest: DIGEST.to_owned(),
        participant_needs_digest: DIGEST.to_owned(),
        consent: BrowserConsentProposal {
            participant_id: "app-1".to_owned(),
            participant_artifact_digest: DIGEST.to_owned(),
            participant_needs_digest: DIGEST.to_owned(),
            consent_view_digest: trellis_protocol::digest_json(&consent_view).unwrap(),
            proposal_digest: trellis_protocol::digest_json(&serde_json::json!({
                "participantId": "app-1",
                "participantArtifactDigest": DIGEST,
                "participantNeedsDigest": DIGEST,
                "requiredGrantSet": required_grant_set,
                "optionalGrantBundles": optional_grant_bundles,
                "requiredCapabilities": required_capabilities,
                "optionalCapabilityDefinitions": optional_capability_definitions,
            }))
            .unwrap(),
            consent_view,
            required_grant_set,
            optional_grant_bundles,
            required_capabilities,
            optional_capability_definitions,
        },
        session_public_key: "session-key".to_owned(),
        session_nkey: "USESSIONKEY".to_owned(),
        portal_id: "builtin".to_owned(),
        redirect_target: Some("https://app.example/callback".to_owned()),
        principal_id: None,
        claim_owner: None,
        claimed_at: None,
        durable_result_digest: None,
        completed_at: None,
        created_at: 100,
        expires_at: 1_000,
        version: 1,
    }
}

fn oauth_state() -> AuthOAuthState {
    AuthOAuthState {
        format: OAUTH_STATE_FORMAT.to_owned(),
        state_id: "state-1".to_owned(),
        provider_id: "provider-1".to_owned(),
        kind: AuthOAuthKind::Browser,
        flow_id: "flow-1".to_owned(),
        status: AuthOAuthStatus::Pending,
        pkce_verifier: "pkce-verifier".to_owned(),
        nonce: "nonce".to_owned(),
        redirect_uri: "https://auth.example/callback".to_owned(),
        browser_binding_digest: DIGEST.to_owned(),
        portal_id: Some("builtin".to_owned()),
        portal_policy_digest: Some(DIGEST.to_owned()),
        claim_owner: None,
        result_digest: None,
        created_at: 100,
        expires_at: 1_000,
        version: 1,
    }
}

async fn repository_conformance(repository: impl AuthEphemeralRepository + Clone) {
    let flow = browser_flow();
    repository.create_browser_flow(flow.clone()).await.unwrap();
    assert_eq!(
        repository.get_browser_flow(&flow.flow_id).await.unwrap(),
        Some(flow.clone())
    );
    assert_eq!(
        repository.create_browser_flow(flow.clone()).await,
        Err(AuthorizationStateError::StorageConflict)
    );

    let mut authenticated = flow.clone();
    authenticated.state = AuthBrowserFlowState::Authenticated;
    authenticated.principal_id = Some("user-1".to_owned());
    authenticated.version = 2;
    repository
        .replace_browser_flow(1, authenticated.clone())
        .await
        .unwrap();
    assert_eq!(
        repository
            .replace_browser_flow(1, authenticated.clone())
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );
    for changed_consent in [
        {
            let mut consent = authenticated.consent.clone();
            consent.consent_view = serde_json::json!({ "title": "Changed" });
            consent
        },
        {
            let mut consent = authenticated.consent.clone();
            consent.consent_view_digest = DIGEST.replace('A', "B");
            consent
        },
        {
            let mut consent = authenticated.consent.clone();
            consent.proposal_digest = DIGEST.replace('A', "C");
            consent
        },
        {
            let mut consent = authenticated.consent.clone();
            consent
                .optional_capability_definitions
                .insert("extra".to_owned(), GrantSet::new(Vec::new()));
            consent
        },
    ] {
        let mut changed = authenticated.clone();
        changed.state = AuthBrowserFlowState::ApprovalRequired;
        changed.consent = changed_consent;
        changed.version = 3;
        assert_eq!(
            repository.replace_browser_flow(2, changed).await,
            Err(AuthorizationStateError::StorageConflict)
        );
    }
    let mut changed_transcript = authenticated;
    changed_transcript.request_id = "changed".to_owned();
    changed_transcript.version = 3;
    assert_eq!(
        repository.replace_browser_flow(2, changed_transcript).await,
        Err(AuthorizationStateError::StorageConflict)
    );
    let mut skipped_state = flow.clone();
    skipped_state.flow_id = "flow-skipped".to_owned();
    repository
        .create_browser_flow(skipped_state.clone())
        .await
        .unwrap();
    skipped_state.state = AuthBrowserFlowState::Approved;
    skipped_state.principal_id = Some("user-1".to_owned());
    skipped_state.durable_result_digest = Some(DIGEST.to_owned());
    skipped_state.completed_at = Some(200);
    skipped_state.version = 2;
    assert_eq!(
        repository.replace_browser_flow(1, skipped_state).await,
        Err(AuthorizationStateError::StorageConflict)
    );

    let oauth = oauth_state();
    repository.create_oauth_state(oauth.clone()).await.unwrap();
    assert_eq!(
        repository.get_oauth_state(&oauth.state_id).await.unwrap(),
        Some(oauth)
    );
    let mut skipped_exchange = oauth_state();
    skipped_exchange.status = AuthOAuthStatus::ExchangeStarted;
    skipped_exchange.claim_owner = Some("owner-1".to_owned());
    skipped_exchange.version = 2;
    assert_eq!(
        repository.replace_oauth_state(1, skipped_exchange).await,
        Err(AuthorizationStateError::StorageConflict)
    );

    let first = repository.clone();
    let second = repository.clone();
    let (left, right) = tokio::join!(
        claim_oauth_state(&first, "state-1", "owner-1"),
        claim_oauth_state(&second, "state-1", "owner-2")
    );
    assert_ne!(left.is_ok(), right.is_ok());
    assert!(matches!(
        left.as_ref().err().or(right.as_ref().err()),
        Some(AuthorizationStateError::StorageConflict)
    ));

    let mut current = repository
        .get_oauth_state("state-1")
        .await
        .unwrap()
        .unwrap();
    current.status = AuthOAuthStatus::ExchangeStarted;
    current.version += 1;
    repository
        .replace_oauth_state(current.version - 1, current.clone())
        .await
        .unwrap();

    let stale = current.clone();
    current.status = AuthOAuthStatus::RestartRequired;
    current.version += 1;
    repository
        .replace_oauth_state(current.version - 1, current.clone())
        .await
        .unwrap();
    assert_eq!(
        repository.replace_oauth_state(stale.version, stale).await,
        Err(AuthorizationStateError::StorageConflict)
    );

    let mut completed = oauth_state();
    completed.state_id = "state-2".to_owned();
    repository
        .create_oauth_state(completed.clone())
        .await
        .unwrap();
    completed = claim_oauth_state(&repository, "state-2", "owner-1")
        .await
        .unwrap();
    completed.status = AuthOAuthStatus::ExchangeStarted;
    completed.version += 1;
    repository
        .replace_oauth_state(completed.version - 1, completed.clone())
        .await
        .unwrap();
    completed.status = AuthOAuthStatus::Completed;
    completed.result_digest = Some(DIGEST.to_owned());
    completed.version += 1;
    repository
        .replace_oauth_state(completed.version - 1, completed.clone())
        .await
        .unwrap();
    assert_eq!(
        repository.get_oauth_state("state-2").await.unwrap(),
        Some(completed)
    );

    let connection = AuthConnectionPresence {
        format: "trellis.auth-connection-presence.v1".to_owned(),
        connection_id: DIGEST.to_owned(),
        session_id: "ses_01".to_owned(),
        context_digest: DIGEST.to_owned(),
        server_id: "server-1".to_owned(),
        client_id: "42".to_owned(),
        user_nkey: "user-nkey".to_owned(),
        remote_address: Some("127.0.0.1".to_owned()),
        connected_at: 1_000,
        last_seen_at: 1_000,
        version: 1,
    };
    repository
        .put_connection_presence(connection.clone())
        .await
        .unwrap();
    let mut second_connection = connection;
    let second_connection_id =
        trellis_protocol::digest_json(&serde_json::json!("second connection")).unwrap();
    second_connection.connection_id = second_connection_id.clone();
    second_connection.client_id = "43".to_owned();
    repository
        .put_connection_presence(second_connection)
        .await
        .unwrap();
    assert_eq!(
        repository
            .list_connection_presence(Some("ses_01"))
            .await
            .unwrap()
            .len(),
        2
    );
    repository.delete_connection_presence(DIGEST).await.unwrap();
    let remaining = repository
        .list_connection_presence(Some("ses_01"))
        .await
        .unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].connection_id, second_connection_id);
}

#[tokio::test]
async fn in_memory_repository_conforms() {
    repository_conformance(InMemoryAuthEphemeralRepository::default()).await;
}

#[tokio::test]
#[ignore = "requires Podman for a live NATS JetStream container"]
async fn nats_kv_repository_conforms() {
    struct Server {
        child: Child,
        name: String,
    }

    impl Drop for Server {
        fn drop(&mut self) {
            let _ = Command::new("podman")
                .args(["rm", "-f", &self.name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = self.child.wait();
        }
    }

    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let name = format!("trellis-auth-ephemeral-test-{}-{port}", std::process::id());
    let server = Server {
        child: Command::new("podman")
            .args([
                "run",
                "--rm",
                "--name",
                &name,
                "-p",
                &format!("127.0.0.1:{port}:4222"),
                "docker.io/library/nats:2-alpine",
                "-js",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
        name,
    };
    let url = format!("nats://127.0.0.1:{port}");
    let mut client = None;
    for _ in 0..100 {
        match async_nats::connect(&url).await {
            Ok(connected) => {
                client = Some(connected);
                break;
            }
            Err(_) => tokio::time::sleep(std::time::Duration::from_millis(25)).await,
        }
    }
    let client = client.expect("NATS did not start");
    let repository = NatsAuthEphemeralRepository::ensure(
        client.clone(),
        std::time::Duration::from_millis(120_000),
    )
    .await
    .unwrap();
    repository_conformance(repository).await;
    let error =
        NatsAuthEphemeralRepository::ensure(client, std::time::Duration::from_millis(360_000))
            .await
            .expect_err("old connection presence retention must be incompatible");
    let error = error.to_string();
    assert!(error.contains("max_age"), "{error}");
    assert!(error.contains("360000ms"), "{error}");
    assert!(error.contains("120000ms"), "{error}");
    drop(server);
}

#[test]
fn strict_json_keeps_required_nullable_fields() {
    let value = serde_json::to_value(browser_flow()).unwrap();
    assert_eq!(value["principalId"], serde_json::Value::Null);
    assert_eq!(value["claimOwner"], serde_json::Value::Null);
    assert_eq!(value["claimedAt"], serde_json::Value::Null);
    assert_eq!(value["durableResultDigest"], serde_json::Value::Null);
    assert_eq!(value["completedAt"], serde_json::Value::Null);

    let mut value = serde_json::to_value(oauth_state()).unwrap();
    assert_eq!(value["claimOwner"], serde_json::Value::Null);
    assert_eq!(value["resultDigest"], serde_json::Value::Null);
    value["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<AuthOAuthState>(value).is_err());
}
