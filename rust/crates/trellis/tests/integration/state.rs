use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::support::assertions::assert_case_registered;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Preferences {
    theme: String,
    density: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Draft {
    title: String,
    body: String,
}

#[tokio::test]
async fn state_value_store_missing_read() {
    assert_case_registered("state.value-store-missing-read", "state", "state");

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    let contract = state_client_contract().expect("build state client test contract");

    let client = admin
        .connect_client(&bootstrap_url, &contract)
        .await
        .expect("connect live Rust state client");

    let preferences =
        trellis_rs::client::ValueStateStore::<_, Preferences>::new(&client, "preferences");

    assert_eq!(
        call_state_get_missing_with_retry(&preferences).await,
        trellis_rs::client::StateGetResult::Missing { found: false }
    );
}

#[tokio::test]
async fn state_value_store_create_read_delete() {
    assert_case_registered("state.value-store-create-read-delete", "state", "state");

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    let contract = state_client_contract().expect("build state client test contract");

    let client = admin
        .connect_client(&bootstrap_url, &contract)
        .await
        .expect("connect live Rust state client");

    let preferences =
        trellis_rs::client::ValueStateStore::<_, Preferences>::new(&client, "preferences");

    let created = preferences
        .put_with_options(
            &Preferences {
                theme: "dark".to_string(),
                density: "comfortable".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("create preferences");
    assert!(created.applied);
    let created_entry = match created.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => entry,
        _ => panic!("expected current preferences entry"),
    };
    assert_eq!(created_entry.value.theme, "dark");

    match preferences.get().await.expect("read preferences") {
        trellis_rs::client::StateGetResult::Found { entry, .. } => {
            assert_eq!(entry.value.density, "comfortable");
        }
        other => panic!("expected found preferences, got {other:?}"),
    }

    let deleted = preferences
        .delete_with_options(&trellis_rs::client::DeleteStateOptions {
            expected_revision: Some(created_entry.revision),
        })
        .await
        .expect("delete preferences");
    assert!(deleted.deleted);

    assert_eq!(
        preferences.get().await.expect("read deleted preferences"),
        trellis_rs::client::StateGetResult::Missing { found: false }
    );
}

#[tokio::test]
async fn state_value_store_stale_revision_rejected() {
    assert_case_registered(
        "state.value-store-stale-revision-rejected",
        "state",
        "state",
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

    let contract = state_client_contract().expect("build state client test contract");

    let client = admin
        .connect_client(&bootstrap_url, &contract)
        .await
        .expect("connect live Rust state client");

    let preferences =
        trellis_rs::client::ValueStateStore::<_, Preferences>::new(&client, "preferences");

    let created = preferences
        .put_with_options(
            &Preferences {
                theme: "dark".to_string(),
                density: "comfortable".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("create preferences");
    assert!(created.applied);

    let stale_write = preferences
        .put_with_options(
            &Preferences {
                theme: "light".to_string(),
                density: "compact".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::Revision(
                    "stale-revision".to_string(),
                ),
                ..Default::default()
            },
        )
        .await
        .expect("stale write should not error");
    assert!(!stale_write.applied);

    let stale_delete = preferences
        .delete_with_options(&trellis_rs::client::DeleteStateOptions {
            expected_revision: Some("stale-revision".to_string()),
        })
        .await
        .expect("stale delete should not error");
    assert!(!stale_delete.deleted);
}

#[tokio::test]
async fn state_value_and_map_conflict_shapes_live() {
    assert_case_registered("state.value-and-map-conflict-shapes-live", "state", "state");

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    let contract = state_client_contract().expect("build state client test contract");

    let client = admin
        .connect_client(&bootstrap_url, &contract)
        .await
        .expect("connect live Rust state client");

    let preferences =
        trellis_rs::client::ValueStateStore::<_, Preferences>::new(&client, "preferences");
    let drafts = trellis_rs::client::MapStateStore::<_, Draft>::new(&client, "drafts")
        .prefix("conflict-shapes");

    let created_preferences = preferences
        .put_with_options(
            &Preferences {
                theme: "dark".to_string(),
                density: "comfortable".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("create preferences");
    assert!(created_preferences.applied);

    let value_create_conflict = preferences
        .put_with_options(
            &Preferences {
                theme: "light".to_string(),
                density: "compact".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("value create conflict should not error");
    assert!(!value_create_conflict.applied);
    match value_create_conflict.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => {
            assert_eq!(entry.value.theme, "dark");
            assert_eq!(entry.value.density, "comfortable");
        }
        other => panic!("expected current value create conflict entry, got {other:?}"),
    }

    let value_stale_conflict = preferences
        .put_with_options(
            &Preferences {
                theme: "light".to_string(),
                density: "compact".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::Revision(
                    "stale-revision".to_string(),
                ),
                ..Default::default()
            },
        )
        .await
        .expect("value stale conflict should not error");
    assert!(!value_stale_conflict.applied);
    match value_stale_conflict.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => {
            assert_eq!(entry.value.theme, "dark");
            assert_eq!(entry.value.density, "comfortable");
        }
        other => panic!("expected current value stale conflict entry, got {other:?}"),
    }

    let created_draft = drafts
        .put_with_options(
            "state-draft",
            &Draft {
                title: "Conflict Draft".to_string(),
                body: "from Rust".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("create draft");
    assert!(created_draft.applied);

    let map_create_conflict = drafts
        .put_with_options(
            "state-draft",
            &Draft {
                title: "Replacement Draft".to_string(),
                body: "should not apply".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("map create conflict should not error");
    assert!(!map_create_conflict.applied);
    match map_create_conflict.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => {
            assert_eq!(entry.key, "conflict-shapes/state-draft");
            assert_eq!(entry.value.title, "Conflict Draft");
            assert_eq!(entry.value.body, "from Rust");
        }
        other => panic!("expected current map create conflict entry, got {other:?}"),
    }

    let map_stale_conflict = drafts
        .put_with_options(
            "state-draft",
            &Draft {
                title: "Replacement Draft".to_string(),
                body: "should not apply".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::Revision(
                    "stale-revision".to_string(),
                ),
                ..Default::default()
            },
        )
        .await
        .expect("map stale conflict should not error");
    assert!(!map_stale_conflict.applied);
    match map_stale_conflict.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => {
            assert_eq!(entry.key, "conflict-shapes/state-draft");
            assert_eq!(entry.value.title, "Conflict Draft");
            assert_eq!(entry.value.body, "from Rust");
        }
        other => panic!("expected current map stale conflict entry, got {other:?}"),
    }
}

#[tokio::test]
async fn state_ttl_expiry_is_absent_live() {
    assert_case_registered("state.ttl-expiry-is-absent-live", "state", "state");

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    let contract = state_client_contract().expect("build state client test contract");

    let client = admin
        .connect_client(&bootstrap_url, &contract)
        .await
        .expect("connect live Rust state client");

    let preferences =
        trellis_rs::client::ValueStateStore::<_, Preferences>::new(&client, "preferences");
    let drafts =
        trellis_rs::client::MapStateStore::<_, Draft>::new(&client, "drafts").prefix("ttl-expiry");

    let created_preferences = preferences
        .put_with_options(
            &Preferences {
                theme: "dark".to_string(),
                density: "comfortable".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                ttl_ms: Some(100),
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
            },
        )
        .await
        .expect("create expiring preferences");
    assert!(created_preferences.applied);
    let created_preferences_entry = match created_preferences.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => entry,
        _ => panic!("expected current preferences entry"),
    };
    assert!(!created_preferences_entry.revision.is_empty());

    let created_draft = drafts
        .put_with_options(
            "state-draft",
            &Draft {
                title: "TTL Draft".to_string(),
                body: "from Rust".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                ttl_ms: Some(100),
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
            },
        )
        .await
        .expect("create expiring draft");
    assert!(created_draft.applied);
    let created_draft_entry = match created_draft.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => entry,
        _ => panic!("expected current draft entry"),
    };

    tokio::time::sleep(Duration::from_millis(250)).await;

    assert_eq!(
        preferences.get().await.expect("read expired preferences"),
        trellis_rs::client::StateGetResult::Missing { found: false }
    );

    let listed = drafts
        .list(&trellis_rs::client::ListStateOptions {
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("list drafts after expiry");
    assert!(!listed.entries.iter().any(|entry| matches!(
        entry,
        trellis_rs::client::StateValue::Current(entry) if entry.key == "ttl-expiry/state-draft"
    )));

    let created_over_expired = preferences
        .put_with_options(
            &Preferences {
                theme: "light".to_string(),
                density: "compact".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("create preferences over expired entry");
    assert!(created_over_expired.applied);

    let deleted_expired = drafts
        .delete_with_options(
            "state-draft",
            &trellis_rs::client::DeleteStateOptions {
                expected_revision: Some(created_draft_entry.revision),
            },
        )
        .await
        .expect("delete expired draft");
    assert!(!deleted_expired.deleted);
}

#[tokio::test]
async fn state_map_store_prefix_put_get_list_delete() {
    assert_case_registered(
        "state.map-store-prefix-put-get-list-delete",
        "state",
        "state",
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

    let contract = state_client_contract().expect("build state client test contract");

    let client = admin
        .connect_client(&bootstrap_url, &contract)
        .await
        .expect("connect live Rust state client");

    let drafts =
        trellis_rs::client::MapStateStore::<_, Draft>::new(&client, "drafts").prefix("inspection");

    let created = drafts
        .put_with_options(
            "state-draft",
            &Draft {
                title: "State Draft".to_string(),
                body: "from Rust".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("create draft");
    let created_entry = match created.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => entry,
        _ => panic!("expected current draft entry"),
    };
    assert_eq!(created_entry.key, "inspection/state-draft");

    match drafts.get("state-draft").await.expect("read draft") {
        trellis_rs::client::StateGetResult::Found { entry, .. } => {
            assert_eq!(entry.value.title, "State Draft");
        }
        other => panic!("expected found draft, got {other:?}"),
    }

    let listed = drafts
        .list(&trellis_rs::client::ListStateOptions {
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("list drafts");
    assert_eq!(listed.count, 1);
    let listed_entry = match listed.entries.first() {
        Some(trellis_rs::client::StateValue::Current(entry)) => entry,
        other => panic!("expected listed current draft, got {other:?}"),
    };
    assert_eq!(listed_entry.key, "inspection/state-draft");

    let deleted = drafts
        .delete_with_options(
            "state-draft",
            &trellis_rs::client::DeleteStateOptions {
                expected_revision: Some(created_entry.revision),
            },
        )
        .await
        .expect("delete draft");
    assert!(deleted.deleted);

    assert_eq!(
        drafts.get("state-draft").await.expect("read deleted draft"),
        trellis_rs::client::StateGetResult::Missing { found: false }
    );
}

#[tokio::test]
async fn state_map_store_list_limit() {
    assert_case_registered("state.map-store-list-limit", "state", "state");

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    let contract = state_client_contract().expect("build state client test contract");

    let client = admin
        .connect_client(&bootstrap_url, &contract)
        .await
        .expect("connect live Rust state client");

    let drafts =
        trellis_rs::client::MapStateStore::<_, Draft>::new(&client, "drafts").prefix("limit-test");

    for i in 1..=5 {
        let result = drafts
            .put_with_options(
                &format!("entry-{i}"),
                &Draft {
                    title: format!("Entry {i}"),
                    body: "body".to_string(),
                },
                &trellis_rs::client::PutStateOptions {
                    expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                    ..Default::default()
                },
            )
            .await
            .expect("create draft entry");
        assert!(result.applied);
    }

    let listed = drafts
        .list(&trellis_rs::client::ListStateOptions {
            limit: Some(2),
            ..Default::default()
        })
        .await
        .expect("list drafts");
    assert!(
        listed.entries.len() <= 2,
        "expected ≤ 2 entries, got {}",
        listed.entries.len()
    );
}

#[tokio::test]
async fn state_migration_required_is_returned_live() {
    assert_case_registered(
        "state.migration-required-is-returned-live",
        "state",
        "state",
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

    let client_v1_contract = state_client_contract().expect("build state client v1 test contract");
    let client_v2_contract =
        state_client_v2_contract().expect("build state client v2 test contract");
    let admin_contract = state_admin_contract().expect("build state admin test contract");

    let client_v1 = admin
        .connect_client(&bootstrap_url, &client_v1_contract)
        .await
        .expect("connect live Rust state v1 client");
    let client_v2 = admin
        .connect_client(&bootstrap_url, &client_v2_contract)
        .await
        .expect("connect live Rust state v2 client");
    let admin_client = admin
        .connect_client(&bootstrap_url, &admin_contract)
        .await
        .expect("connect live Rust state admin client");

    let preferences_v1 =
        trellis_rs::client::ValueStateStore::<_, Preferences>::new(&client_v1, "preferences");
    let drafts_v1 = trellis_rs::client::MapStateStore::<_, Draft>::new(&client_v1, "drafts")
        .prefix("inspection");
    let preferences_v2 =
        trellis_rs::client::ValueStateStore::<_, serde_json::Value>::new(&client_v2, "preferences");
    let drafts_v2 =
        trellis_rs::client::MapStateStore::<_, serde_json::Value>::new(&client_v2, "drafts")
            .prefix("inspection");

    let preferences_created = preferences_v1
        .put_with_options(
            &Preferences {
                theme: "dark".to_string(),
                density: "comfortable".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("create v1 preferences");
    let preferences_entry = match preferences_created.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => entry,
        _ => panic!("expected current preferences entry"),
    };

    let draft_created = drafts_v1
        .put_with_options(
            "state-draft",
            &Draft {
                title: "Migration Draft".to_string(),
                body: "from Rust".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("create v1 draft");
    let draft_entry = match draft_created.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => entry,
        _ => panic!("expected current draft entry"),
    };

    let writer_digest = client_v1_contract.digest().to_string();
    match preferences_v2.get().await.expect("read v2 preferences") {
        trellis_rs::client::StateGetResult::MigrationRequired(migration) => {
            assert_state_migration(
                &serde_json::to_value(migration).expect("serialize preferences migration"),
                json!({ "theme": "dark", "density": "comfortable" }),
                &preferences_entry.revision,
                "preferences.v1",
                "preferences.v2",
                &writer_digest,
            );
        }
        other => panic!("expected preferences migration-required, got {other:?}"),
    }

    match drafts_v2.get("state-draft").await.expect("read v2 draft") {
        trellis_rs::client::StateGetResult::MigrationRequired(migration) => {
            assert_state_migration(
                &serde_json::to_value(migration).expect("serialize draft migration"),
                json!({ "title": "Migration Draft", "body": "from Rust" }),
                &draft_entry.revision,
                "drafts.v1",
                "drafts.v2",
                &writer_digest,
            );
        }
        other => panic!("expected draft migration-required, got {other:?}"),
    }

    let admin_auth = trellis_rs::sdk::auth::AuthClient::new(crate::generated_caller(&admin_client));
    let target_user = find_user_target_for_contract(
        &admin_auth
            .rpc()
            .auth()
            .sessions_list(&auth_sessions_list_request())
            .await
            .expect("list sessions for state admin target"),
        "trellis.integration.state-client@v1",
    )
    .expect("Auth.Sessions.List should include state client session");

    let admin_state =
        trellis_rs::sdk::state::StateClient::new(crate::generated_caller(&admin_client));
    let state_target = json!({
        "scope": "userApp",
        "contractId": "trellis.integration.state-client@v1",
        "contractDigest": client_v2_contract.digest(),
        "user": target_user,
    });

    let admin_preferences = admin_state
        .rpc()
        .state()
        .admin_get(
            &serde_json::from_value(json_object_merge(
                &state_target,
                json!({ "store": "preferences" }),
            ))
            .expect("valid State.Admin.Get request"),
        )
        .await
        .expect("admin get v2 preferences");
    let trellis_rs::sdk::state::types::StateAdminGetResponse::Variant3 {
        current_state_version,
        entry,
        migration_required,
        state_version,
        writer_contract_digest,
    } = admin_preferences
    else {
        panic!("expected migration-required preferences response");
    };
    assert!(migration_required);
    assert_eq!(
        entry.value,
        json!({ "theme": "dark", "density": "comfortable" })
    );
    assert_eq!(entry.revision, preferences_entry.revision);
    assert_eq!(state_version, "preferences.v1");
    assert_eq!(current_state_version, "preferences.v2");
    assert_eq!(writer_contract_digest, writer_digest);

    let admin_list = admin_state
        .rpc()
        .state()
        .admin_list(
            &serde_json::from_value(json_object_merge(
                &state_target,
                json!({
                    "store": "drafts",
                    "prefix": "inspection",
                    "offset": 0,
                    "limit": 10,
                }),
            ))
            .expect("valid State.Admin.List request"),
        )
        .await
        .expect("admin list v2 drafts");
    let admin_draft = admin_list
        .entries
        .into_iter()
        .find_map(|entry| match entry {
            trellis_rs::sdk::state::types::StateAdminListResponseEntriesItem::Variant2 {
                current_state_version,
                entry,
                migration_required,
                state_version,
                writer_contract_digest,
            } if entry.key.as_deref() == Some("inspection/state-draft") => Some((
                current_state_version,
                entry,
                migration_required,
                state_version,
                writer_contract_digest,
            )),
            _ => None,
        })
        .expect("admin list should include migrated draft state");
    assert!(admin_draft.2);
    assert_eq!(
        admin_draft.1.value,
        json!({ "title": "Migration Draft", "body": "from Rust" })
    );
    assert_eq!(admin_draft.1.revision, draft_entry.revision);
    assert_eq!(admin_draft.3, "drafts.v1");
    assert_eq!(admin_draft.0, "drafts.v2");
    assert_eq!(admin_draft.4, writer_digest);
}

#[tokio::test]
async fn state_admin_inspect_and_delete_state() {
    assert_case_registered("state.admin-inspect-and-delete-state", "state", "state");

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    let client_contract = state_client_contract().expect("build state client test contract");
    let admin_contract = state_admin_contract().expect("build state admin test contract");

    let client = admin
        .connect_client(&bootstrap_url, &client_contract)
        .await
        .expect("connect live Rust state client");
    let admin_client = admin
        .connect_client(&bootstrap_url, &admin_contract)
        .await
        .expect("connect live Rust state admin client");

    let preferences =
        trellis_rs::client::ValueStateStore::<_, Preferences>::new(&client, "preferences");
    let drafts =
        trellis_rs::client::MapStateStore::<_, Draft>::new(&client, "drafts").prefix("inspection");

    let preferences_created = preferences
        .put_with_options(
            &Preferences {
                theme: "dark".to_string(),
                density: "comfortable".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("create preferences");
    assert!(preferences_created.applied);
    let preferences_entry = match preferences_created.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => entry,
        _ => panic!("expected current preferences entry"),
    };

    let draft_created = drafts
        .put_with_options(
            "state-draft",
            &Draft {
                title: "Admin Inspection".to_string(),
                body: "from Rust".to_string(),
            },
            &trellis_rs::client::PutStateOptions {
                expected_revision: trellis_rs::client::ExpectedPutRevision::CreateIfAbsent,
                ..Default::default()
            },
        )
        .await
        .expect("create draft");
    assert!(draft_created.applied);
    let draft_entry = match draft_created.entry {
        Some(trellis_rs::client::StateValue::Current(entry)) => entry,
        _ => panic!("expected current draft entry"),
    };

    let admin_auth = trellis_rs::sdk::auth::AuthClient::new(crate::generated_caller(&admin_client));
    let target_user = find_user_target_for_contract(
        &admin_auth
            .rpc()
            .auth()
            .sessions_list(&auth_sessions_list_request())
            .await
            .expect("list sessions for state admin target"),
        "trellis.integration.state-client@v1",
    )
    .expect("Auth.Sessions.List should include state client session");

    let admin_state =
        trellis_rs::sdk::state::StateClient::new(crate::generated_caller(&admin_client));
    let state_target = json!({
        "scope": "userApp",
        "contractId": "trellis.integration.state-client@v1",
        "contractDigest": client_contract.digest(),
        "user": target_user,
    });

    let admin_preferences = admin_state
        .rpc()
        .state()
        .admin_get(
            &serde_json::from_value(json_object_merge(
                &state_target,
                json!({ "store": "preferences" }),
            ))
            .expect("valid State.Admin.Get request"),
        )
        .await
        .expect("admin get preferences");
    let trellis_rs::sdk::state::types::StateAdminGetResponse::Variant2 { entry, found } =
        admin_preferences
    else {
        panic!("expected found preferences response");
    };
    assert!(found);
    assert_eq!(
        entry.value,
        json!({ "theme": "dark", "density": "comfortable" })
    );
    assert_eq!(entry.revision, preferences_entry.revision);
    assert!(!entry.updated_at.is_empty());

    let admin_list = admin_state
        .rpc()
        .state()
        .admin_list(
            &serde_json::from_value(json_object_merge(
                &state_target,
                json!({
                    "store": "drafts",
                    "prefix": "inspection",
                    "offset": 0,
                    "limit": 10,
                }),
            ))
            .expect("valid State.Admin.List request"),
        )
        .await
        .expect("admin list drafts");
    let listed_draft = admin_list
        .entries
        .into_iter()
        .find_map(|entry| match entry {
            trellis_rs::sdk::state::types::StateAdminListResponseEntriesItem::Variant1 {
                key: Some(key),
                revision,
                value,
                ..
            } if key == "inspection/state-draft" => Some((revision, value)),
            _ => None,
        })
        .expect("admin list should include draft state");
    assert_eq!(
        listed_draft.1,
        json!({ "title": "Admin Inspection", "body": "from Rust" })
    );
    assert_eq!(listed_draft.0, draft_entry.revision);

    let deleted_preferences = admin_state
        .rpc()
        .state()
        .admin_delete(
            &serde_json::from_value(json_object_merge(
                &state_target,
                json!({
                    "store": "preferences",
                    "expectedRevision": preferences_entry.revision.clone(),
                }),
            ))
            .expect("valid State.Admin.Delete request"),
        )
        .await
        .expect("admin delete preferences");
    assert!(deleted_preferences.deleted);

    let deleted_draft = admin_state
        .rpc()
        .state()
        .admin_delete(
            &serde_json::from_value(json_object_merge(
                &state_target,
                json!({
                    "store": "drafts",
                    "key": "inspection/state-draft",
                    "expectedRevision": draft_entry.revision.clone(),
                }),
            ))
            .expect("valid State.Admin.Delete request"),
        )
        .await
        .expect("admin delete draft");
    assert!(deleted_draft.deleted);

    assert_eq!(
        preferences
            .get()
            .await
            .expect("read admin-deleted preferences"),
        trellis_rs::client::StateGetResult::Missing { found: false }
    );
    assert_eq!(
        drafts
            .get("state-draft")
            .await
            .expect("read admin-deleted draft"),
        trellis_rs::client::StateGetResult::Missing { found: false }
    );
}

#[tokio::test]
async fn state_admin_deletes_corrupt_state_entry() {
    assert_case_registered("state.admin-deletes-corrupt-state-entry", "state", "state");

    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let mut admin = runtime.admin();

    let client_contract = state_client_contract().expect("build state client test contract");
    let admin_contract = state_admin_contract().expect("build state admin test contract");

    let client = admin
        .connect_client(&bootstrap_url, &client_contract)
        .await
        .expect("connect live Rust state client");
    let admin_client = admin
        .connect_client(&bootstrap_url, &admin_contract)
        .await
        .expect("connect live Rust state admin client");

    let admin_auth = trellis_rs::sdk::auth::AuthClient::new(crate::generated_caller(&admin_client));
    let target_user = find_user_target_for_contract(
        &admin_auth
            .rpc()
            .auth()
            .sessions_list(&auth_sessions_list_request())
            .await
            .expect("list sessions for state admin target"),
        "trellis.integration.state-client@v1",
    )
    .expect("Auth.Sessions.List should include state client session");
    let user_id = target_user["userId"]
        .as_str()
        .expect("state admin target should include userId");

    let storage_key = [
        encode_state_component("user"),
        encode_state_component(user_id),
        encode_state_component("trellis.integration.state-client@v1"),
        encode_state_component("preferences"),
        "=value".to_string(),
    ]
    .join(".");
    runtime
        .seed_raw_state_entry(trellis_test::TrellisRawStateEntry {
            key: storage_key,
            value: json!({
                "value": { "theme": "dark", "density": "comfortable" },
                "updatedAt": "2026-01-01T00:00:00.000Z",
                "stateVersion": "preferences.v1"
            }),
        })
        .await
        .expect("seed corrupt raw state entry");

    let preferences =
        trellis_rs::client::ValueStateStore::<_, Preferences>::new(&client, "preferences");
    assert!(
        preferences.get().await.is_err(),
        "corrupt raw state entry should fail public read"
    );

    let admin_state =
        trellis_rs::sdk::state::StateClient::new(crate::generated_caller(&admin_client));
    let deleted = admin_state
        .rpc()
        .state()
        .admin_delete(
            &serde_json::from_value(json!({
                "scope": "userApp",
                "contractId": "trellis.integration.state-client@v1",
                "contractDigest": client_contract.digest(),
                "user": target_user,
                "store": "preferences"
            }))
            .expect("valid State.Admin.Delete request"),
        )
        .await
        .expect("admin delete corrupt preferences");
    assert!(deleted.deleted);

    assert_eq!(
        preferences
            .get()
            .await
            .expect("read admin-deleted corrupt preferences"),
        trellis_rs::client::StateGetResult::Missing { found: false }
    );
}

fn state_client_contract(
) -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractManifestBuilder::new(
        "trellis.integration.state-client@v1",
        "Trellis Integration State Client",
        "Exercises generated contract-owned state store surfaces.",
        trellis_rs::contracts::ContractKind::App,
    )
    .schema(
        "Preferences",
        json!({
            "type": "object",
            "required": ["theme", "density"],
            "properties": {
                "theme": { "type": "string" },
                "density": { "type": "string" }
            }
        }),
    )
    .schema(
        "Draft",
        json!({
            "type": "object",
            "required": ["title", "body"],
            "properties": {
                "title": { "type": "string" },
                "body": { "type": "string" }
            }
        }),
    )
    .use_ref(
        "state",
        trellis_rs::contracts::use_contract("trellis.state@v1").with_rpc_call([
            "State.Get",
            "State.Put",
            "State.Delete",
            "State.List",
        ]),
    )
    .state(
        "preferences",
        trellis_rs::contracts::state(
            trellis_rs::contracts::ContractStateKind::Value,
            "Preferences",
        )
        .state_version("preferences.v1"),
    )
    .state(
        "drafts",
        trellis_rs::contracts::state(trellis_rs::contracts::ContractStateKind::Map, "Draft")
            .state_version("drafts.v1"),
    )
    .build()?;

    trellis_test::TrellisTestContract::from_manifest_value(serde_json::to_value(manifest)?)
}

fn state_client_v2_contract(
) -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest = trellis_rs::contracts::ContractManifestBuilder::new(
        "trellis.integration.state-client@v1",
        "Trellis Integration State Client v2",
        "Exercises generated contract-owned state store migration surfaces.",
        trellis_rs::contracts::ContractKind::App,
    )
    .schema(
        "Preferences",
        json!({
            "type": "object",
            "required": ["theme", "density"],
            "properties": {
                "theme": { "type": "string" },
                "density": { "type": "string" }
            }
        }),
    )
    .schema(
        "PreferencesV2",
        json!({
            "type": "object",
            "required": ["theme", "density", "contrast"],
            "properties": {
                "theme": { "type": "string" },
                "density": { "type": "string" },
                "contrast": { "type": "string" }
            }
        }),
    )
    .schema(
        "Draft",
        json!({
            "type": "object",
            "required": ["title", "body"],
            "properties": {
                "title": { "type": "string" },
                "body": { "type": "string" }
            }
        }),
    )
    .schema(
        "DraftV2",
        json!({
            "type": "object",
            "required": ["title", "body", "status"],
            "properties": {
                "title": { "type": "string" },
                "body": { "type": "string" },
                "status": { "type": "string" }
            }
        }),
    )
    .use_ref(
        "state",
        trellis_rs::contracts::use_contract("trellis.state@v1").with_rpc_call([
            "State.Get",
            "State.Put",
            "State.Delete",
            "State.List",
        ]),
    )
    .state(
        "preferences",
        trellis_rs::contracts::state(
            trellis_rs::contracts::ContractStateKind::Value,
            "PreferencesV2",
        )
        .state_version("preferences.v2")
        .accepted_version("preferences.v1", "Preferences"),
    )
    .state(
        "drafts",
        trellis_rs::contracts::state(trellis_rs::contracts::ContractStateKind::Map, "DraftV2")
            .state_version("drafts.v2")
            .accepted_version("drafts.v1", "Draft"),
    )
    .build()?;

    trellis_test::TrellisTestContract::from_manifest_value(serde_json::to_value(manifest)?)
}

fn state_admin_contract(
) -> Result<trellis_test::TrellisTestContract, trellis_test::TrellisTestError> {
    let manifest =
        trellis_rs::contracts::ContractManifestBuilder::new(
            "trellis.integration.state-admin@v1",
            "Trellis Integration State Admin",
            "Admin participant for inspecting and deleting state through public generated RPCs.",
            trellis_rs::contracts::ContractKind::App,
        )
        .use_ref(
            "auth",
            trellis_rs::contracts::use_contract(trellis_rs::sdk::auth::CONTRACT_ID)
                .with_rpc_call(["Auth.Sessions.List"]),
        )
        .use_ref(
            "state",
            trellis_rs::contracts::use_contract(trellis_rs::sdk::state::CONTRACT_ID)
                .with_rpc_call(["State.Admin.Delete", "State.Admin.Get", "State.Admin.List"]),
        )
        .build()?;

    trellis_test::TrellisTestContract::from_manifest_value(serde_json::to_value(manifest)?)
}

fn auth_sessions_list_request() -> trellis_rs::sdk::auth::types::AuthSessionsListRequest {
    trellis_rs::sdk::auth::types::AuthSessionsListRequest {
        limit: 500,
        offset: None,
        user: None,
    }
}

fn find_user_target_for_contract(
    sessions: &trellis_rs::sdk::auth::types::AuthSessionsListResponse,
    contract_id: &str,
) -> Option<serde_json::Value> {
    sessions.entries.iter().find_map(|entry| {
        let trellis_rs::sdk::auth::types::AuthSessionsListResponseEntriesItem::App {
            contract_id: entry_contract_id,
            principal,
            ..
        } = entry
        else {
            return None;
        };
        if entry_contract_id != contract_id {
            return None;
        }
        Some(json!({
            "origin": principal.identity.provider,
            "id": principal.identity.subject,
            "userId": principal.user_id,
        }))
    })
}

fn json_object_merge(base: &serde_json::Value, overlay: serde_json::Value) -> serde_json::Value {
    let mut merged = base
        .as_object()
        .expect("base JSON value should be an object")
        .clone();
    let overlay = overlay
        .as_object()
        .expect("overlay JSON value should be an object");
    for (key, value) in overlay {
        merged.insert(key.clone(), value.clone());
    }
    serde_json::Value::Object(merged)
}

fn assert_state_migration(
    actual: &serde_json::Value,
    value: serde_json::Value,
    revision: &str,
    state_version: &str,
    current_state_version: &str,
    writer_digest: &str,
) {
    assert_eq!(actual["migrationRequired"], json!(true));
    assert_eq!(actual["entry"]["value"], value);
    assert_eq!(actual["entry"]["revision"], json!(revision));
    assert_eq!(actual["stateVersion"], json!(state_version));
    assert_eq!(actual["currentStateVersion"], json!(current_state_version));
    assert_eq!(actual["writerContractDigest"], json!(writer_digest));
}

fn encode_state_component(value: &str) -> String {
    let mut encoded = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '/' | '-') {
            encoded.push(ch);
            continue;
        }
        let mut buffer = [0; 4];
        for byte in ch.encode_utf8(&mut buffer).bytes() {
            encoded.push_str(&format!("={byte:02X}"));
        }
    }
    encoded
}

async fn call_state_get_missing_with_retry(
    store: &trellis_rs::generated::ValueStateStore<'_, trellis_rs::generated::Caller, Preferences>,
) -> trellis_rs::client::StateGetResult<trellis_rs::client::StateEntry<Preferences>> {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        match store.get().await {
            Ok(result) => return result,
            Err(error) if is_retryable_state_error(&error) && Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(error) => panic!("state get: {error}"),
        }
    }
}

fn is_retryable_state_error(error: &trellis_rs::generated::TrellisClientError) -> bool {
    match error {
        trellis_rs::generated::TrellisClientError::NatsRequest(message) => {
            message.contains("no responders") || message.contains("NoResponders")
        }
        trellis_rs::generated::TrellisClientError::Timeout => true,
        _ => false,
    }
}
