use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use trellis_rs::client::{
    DeleteStateOptions, ExpectedPutRevision, ListStateOptions, MapStateStore, PutStateOptions,
    StateGetResult, StateValue, ValueStateStore,
};
use trellis_rs::sdk::state::types::{
    StateAdminDeleteRequest, StateAdminGetRequest, StateAdminGetResponse, StateAdminListRequest,
};
use trellis_rs::service::GeneratedServiceContract;

const CLIENT_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.state-client@v1",
  "displayName": "Trellis Integration State Client",
  "description": "Exercises Trellis-managed State.",
  "kind": "app",
  "schemas": {
    "Preferences": {
      "type": "object",
      "required": ["theme"],
      "properties": { "theme": { "type": "string" } }
    },
    "Draft": {
      "type": "object",
      "required": ["title"],
      "properties": { "title": { "type": "string" } }
    }
  },
  "state": {
    "preferences": { "kind": "value", "schema": { "schema": "Preferences" } },
    "drafts": { "kind": "map", "schema": { "schema": "Draft" } }
  },
  "uses": {
    "required": {
      "state": {
        "contract": "trellis.state@v1",
        "rpc": { "call": ["State.Delete", "State.Get", "State.List", "State.Put"] }
      }
    }
  }
}"#;

const MIGRATION_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.state-client@v1",
  "displayName": "Trellis Integration State Client V2",
  "description": "Exercises Trellis-managed State migration.",
  "kind": "app",
  "schemas": {
    "PreferencesV1": {
      "type": "object",
      "required": ["theme"],
      "properties": { "theme": { "type": "string" } }
    },
    "Preferences": {
      "type": "object",
      "required": ["theme", "compact"],
      "properties": { "theme": { "type": "string" }, "compact": { "type": "boolean" } }
    },
    "DraftV1": {
      "type": "object",
      "required": ["title"],
      "properties": { "title": { "type": "string" } }
    },
    "Draft": {
      "type": "object",
      "required": ["title", "pinned"],
      "properties": { "title": { "type": "string" }, "pinned": { "type": "boolean" } }
    }
  },
  "state": {
    "preferences": {
      "kind": "value",
      "schema": { "schema": "Preferences" },
      "stateVersion": "preferences.v2",
      "acceptedVersions": { "v1": { "schema": "PreferencesV1" } }
    },
    "drafts": {
      "kind": "map",
      "schema": { "schema": "Draft" },
      "stateVersion": "drafts.v2",
      "acceptedVersions": { "v1": { "schema": "DraftV1" } }
    }
  },
  "uses": {
    "required": {
      "state": {
        "contract": "trellis.state@v1",
        "rpc": { "call": ["State.Delete", "State.Get", "State.List", "State.Put"] }
      }
    }
  }
}"#;

const COMPATIBLE_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.state-client@v1",
  "displayName": "Trellis Integration State Client Compatible",
  "description": "Exercises compatible State lineage changes.",
  "kind": "app",
  "schemas": {
    "Preferences": {
      "type": "object",
      "required": ["theme"],
      "properties": {
        "theme": { "type": "string" },
        "compact": { "type": "boolean" }
      }
    },
    "Draft": {
      "type": "object",
      "required": ["title"],
      "properties": { "title": { "type": "string" } }
    }
  },
  "state": {
    "preferences": { "kind": "value", "schema": { "schema": "Preferences" } },
    "drafts": { "kind": "map", "schema": { "schema": "Draft" } }
  },
  "uses": {
    "required": {
      "state": {
        "contract": "trellis.state@v1",
        "rpc": { "call": ["State.Delete", "State.Get", "State.List", "State.Put"] }
      }
    }
  }
}"#;

const OTHER_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.state-other@v1",
  "displayName": "Trellis Integration Other State Client",
  "description": "Exercises State contract namespace isolation.",
  "kind": "app",
  "schemas": {
    "Preferences": {
      "type": "object",
      "required": ["theme"],
      "properties": { "theme": { "type": "string" } }
    }
  },
  "state": {
    "preferences": { "kind": "value", "schema": { "schema": "Preferences" } }
  },
  "uses": {
    "required": {
      "state": {
        "contract": "trellis.state@v1",
        "rpc": { "call": ["State.Delete", "State.Get", "State.Put"] }
      }
    }
  }
}"#;

const AGENT_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.state-agent@v1",
  "displayName": "Trellis Integration State Agent",
  "description": "Proves agents cannot use normal State.",
  "kind": "agent",
  "schemas": { "Value": { "type": "string" } },
  "state": { "value": { "kind": "value", "schema": { "schema": "Value" } } },
  "uses": {
    "required": {
      "state": {
        "contract": "trellis.state@v1",
        "rpc": { "call": ["State.Get"] }
      }
    }
  }
}"#;

const SERVICE_CONTRACT_JSON: &str = r#"{
  "format": "trellis.contract.v1",
  "id": "trellis.integration.state-service@v1",
  "displayName": "Trellis Integration State Service",
  "description": "Proves services cannot use normal State.",
  "kind": "service",
  "uses": {
    "required": {
      "state": {
        "contract": "trellis.state@v1",
        "rpc": { "call": ["State.Get"] }
      }
    }
  }
}"#;

struct StateServiceContract;

impl GeneratedServiceContract for StateServiceContract {
    const CONTRACT_ID: &'static str = "trellis.integration.state-service@v1";
    const CONTRACT_DIGEST: &'static str = "";
    const CONTRACT_JSON: &'static str = SERVICE_CONTRACT_JSON;
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
struct Preferences {
    theme: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
struct Draft {
    title: String,
}

async fn client() -> (
    trellis_test::TrellisTestRuntime,
    trellis_rs::generated::Caller,
) {
    let runtime = trellis_test::TrellisTestRuntime::start(
        trellis_test::TrellisTestRuntimeOptions::repo_platform(),
    )
    .await
    .expect("start live Trellis runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("observe first admin bootstrap URL");
    let contract = trellis_test::TrellisTestContract::from_manifest_json(CLIENT_CONTRACT_JSON)
        .expect("build State client contract");
    let caller = runtime
        .admin()
        .connect_client(&bootstrap_url, &contract)
        .await
        .expect("connect State client");
    (runtime, caller)
}

#[tokio::test]
async fn state_value_store_missing_read() {
    let (_runtime, caller) = client().await;
    let store = ValueStateStore::<_, Preferences>::new(&caller, "preferences");
    assert!(matches!(
        store.get().await.expect("read empty State value store"),
        StateGetResult::Missing { .. }
    ));
}

#[tokio::test]
async fn state_rust_client_reaches_rust_owner() {
    let (_runtime, caller) = client().await;
    let store = ValueStateStore::<_, Preferences>::new(&caller, "preferences");
    assert!(matches!(
        store
            .get()
            .await
            .expect("Rust client reaches Rust State owner"),
        StateGetResult::Missing { .. }
    ));
}

fn put_revision<T, M>(result: &trellis_rs::client::StatePutResult<T, M>) -> String
where
    T: Revision,
{
    match result.entry.as_ref().expect("put result entry") {
        StateValue::Current(entry) => entry.revision().to_owned(),
        StateValue::MigrationRequired(_) => panic!("new write unexpectedly requires migration"),
    }
}

trait Revision {
    fn revision(&self) -> &str;
}

impl<T> Revision for trellis_rs::client::StateEntry<T> {
    fn revision(&self) -> &str {
        &self.revision
    }
}

impl<T> Revision for trellis_rs::client::MapStateEntry<T> {
    fn revision(&self) -> &str {
        &self.revision
    }
}

#[tokio::test]
async fn state_value_store_create_read_delete() {
    let (_runtime, caller) = client().await;
    let store = ValueStateStore::<_, Preferences>::new(&caller, "preferences");
    let value = Preferences {
        theme: "dark".into(),
    };
    let created = store
        .put_with_options(
            &value,
            &PutStateOptions {
                ttl_ms: None,
                expected_revision: ExpectedPutRevision::CreateIfAbsent,
            },
        )
        .await
        .expect("create State value");
    assert!(created.applied);
    let revision = put_revision(&created);
    assert!(matches!(
        store.get().await.expect("read State value"),
        StateGetResult::Found { entry, .. } if entry.value == value
    ));
    assert!(
        store
            .delete_with_options(&DeleteStateOptions {
                expected_revision: Some(revision),
            })
            .await
            .expect("delete State value")
            .deleted
    );
    assert!(matches!(
        store.get().await.expect("read deleted State value"),
        StateGetResult::Missing { .. }
    ));
}

#[tokio::test]
async fn state_value_store_stale_revision_rejected() {
    let (_runtime, caller) = client().await;
    let store = ValueStateStore::<_, Preferences>::new(&caller, "preferences");
    let first = store
        .put(&Preferences {
            theme: "dark".into(),
        })
        .await
        .expect("write first State value");
    let stale = put_revision(&first);
    let second = store
        .put(&Preferences {
            theme: "light".into(),
        })
        .await
        .expect("overwrite State value");
    let conflict = store
        .put_with_options(
            &Preferences {
                theme: "blue".into(),
            },
            &PutStateOptions {
                ttl_ms: None,
                expected_revision: ExpectedPutRevision::Revision(stale.clone()),
            },
        )
        .await
        .expect("return stale write conflict");
    assert!(!conflict.applied);
    assert_eq!(conflict.found, Some(true));
    assert!(
        !store
            .delete_with_options(&DeleteStateOptions {
                expected_revision: Some(stale)
            })
            .await
            .expect("return stale delete result")
            .deleted
    );
    assert_eq!(
        put_revision(&second),
        match store.get().await.expect("read current value") {
            StateGetResult::Found { entry, .. } => entry.revision,
            _ => panic!("current value missing"),
        }
    );
}

#[tokio::test]
async fn state_value_and_map_conflict_shapes_live() {
    let runtime = trellis_test::TrellisTestRuntime::start(
        trellis_test::TrellisTestRuntimeOptions::repo_platform(),
    )
    .await
    .expect("start live Trellis runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("bootstrap URL");
    let contract = trellis_test::TrellisTestContract::from_manifest_json(CLIENT_CONTRACT_JSON)
        .expect("State client contract");
    let mut admin = runtime.admin();
    let (caller, first_session) = admin
        .connect_client_with_session_seed_reconnectable(
            &bootstrap_url,
            &contract,
            URL_SAFE_NO_PAD.encode([8_u8; 32]),
        )
        .await
        .expect("connect first State client session");
    let same_session_connection = first_session
        .connect_bound_only()
        .await
        .expect("connect a second NATS client under the first session");
    assert!(matches!(
        MapStateStore::<_, Draft>::new(&same_session_connection, "drafts")
            .get("same-session-probe")
            .await
            .expect("call State through the second connection"),
        StateGetResult::Missing { .. }
    ));
    let (caller_two, second_session) = admin
        .connect_client_with_session_seed_reconnectable(
            &bootstrap_url,
            &contract,
            URL_SAFE_NO_PAD.encode([9_u8; 32]),
        )
        .await
        .expect("connect second State client session");
    assert_ne!(first_session.session_id(), second_session.session_id());
    caller
        .refresh_authorization_context()
        .await
        .expect("refresh first State session context");
    caller_two
        .refresh_authorization_context()
        .await
        .expect("refresh second State session context");
    let store = MapStateStore::<_, Draft>::new(&caller, "drafts");
    assert!(matches!(
        MapStateStore::<_, Draft>::new(&caller_two, "drafts")
            .get("second-session-probe")
            .await
            .expect("call State through the second session"),
        StateGetResult::Missing { .. }
    ));
    let first = store
        .put_with_options(
            "draft",
            &Draft {
                title: "one".into(),
            },
            &PutStateOptions {
                ttl_ms: None,
                expected_revision: ExpectedPutRevision::CreateIfAbsent,
            },
        )
        .await
        .expect("create map State");
    let revision = put_revision(&first);
    let duplicate = store
        .put_with_options(
            "draft",
            &Draft {
                title: "two".into(),
            },
            &PutStateOptions {
                ttl_ms: None,
                expected_revision: ExpectedPutRevision::CreateIfAbsent,
            },
        )
        .await
        .expect("return create conflict");
    assert!(!duplicate.applied);
    assert_eq!(duplicate.found, Some(true));
    let stale = store
        .put_with_options(
            "draft",
            &Draft {
                title: "three".into(),
            },
            &PutStateOptions {
                ttl_ms: None,
                expected_revision: ExpectedPutRevision::Revision("999999".into()),
            },
        )
        .await
        .expect("return revision conflict");
    assert!(!stale.applied);
    assert_eq!(stale.found, Some(true));
    assert!(!revision.is_empty());

    let first_store = MapStateStore::<_, Draft>::new(&caller, "drafts");
    let second_store = MapStateStore::<_, Draft>::new(&caller_two, "drafts");
    let first_candidate = Draft {
        title: "concurrent-one".into(),
    };
    let second_candidate = Draft {
        title: "concurrent-two".into(),
    };
    let options = PutStateOptions {
        ttl_ms: None,
        expected_revision: ExpectedPutRevision::CreateIfAbsent,
    };
    let (first, second) = tokio::join!(
        first_store.put_with_options("concurrent", &first_candidate, &options),
        second_store.put_with_options("concurrent", &second_candidate, &options),
    );
    let first = first.expect("first concurrent create result");
    let second = second.expect("second concurrent create result");
    assert_eq!(usize::from(first.applied) + usize::from(second.applied), 1);
    let loser = if first.applied { &second } else { &first };
    assert_eq!(loser.found, Some(true));
    let winner_revision = if first.applied {
        put_revision(&first)
    } else {
        put_revision(&second)
    };
    assert!(matches!(
        first_store
            .get("concurrent")
            .await
            .expect("read concurrent winner"),
        StateGetResult::Found { entry, .. }
            if (entry.value == first_candidate || entry.value == second_candidate)
                && entry.revision == winner_revision
    ));

    admin
        .revoke_session(
            &bootstrap_url,
            &trellis_rs::sdk::auth::AuthSessionsRevokeRequest {
                expected_version: None,
                idempotency_key: "state-independent-session-revoke".to_owned(),
                reason: Some("verify sibling session isolation".to_owned()),
                session_id: first_session.session_id().to_owned(),
            },
        )
        .await
        .expect("revoke only the first State session");
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if first_store.get("concurrent").await.is_err() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("first State session remained authorized after revocation");
    assert!(matches!(
        second_store
            .get("concurrent")
            .await
            .expect("sibling State session remains active"),
        StateGetResult::Found { .. }
    ));
}

#[tokio::test]
async fn state_map_store_prefix_put_get_list_delete() {
    let (_runtime, caller) = client().await;
    let root = MapStateStore::<_, Draft>::new(&caller, "drafts");
    for invalid in ["/open", "open/", "inspection//open"] {
        let result = root.get(invalid).await;
        let error = format!("{result:?}");
        assert!(error.contains("ValidationError"), "{error}");
        assert!(error.contains("/key"), "{error}");
        let result = root
            .put(
                invalid,
                &Draft {
                    title: "invalid".into(),
                },
            )
            .await;
        let error = format!("{result:?}");
        assert!(error.contains("ValidationError"), "{error}");
        assert!(error.contains("/key"), "{error}");
        let result = root.delete(invalid).await;
        let error = format!("{result:?}");
        assert!(error.contains("ValidationError"), "{error}");
        assert!(error.contains("/key"), "{error}");
    }
    for invalid in ["/inspection", "inspection/", "inspection//active"] {
        let result = root.prefix(invalid).get("open").await;
        let error = format!("{result:?}");
        assert!(error.contains("ValidationError"), "{error}");
        assert!(error.contains("/key"), "{error}");
        let result = root
            .prefix(invalid)
            .put(
                "open",
                &Draft {
                    title: "invalid".into(),
                },
            )
            .await;
        let error = format!("{result:?}");
        assert!(error.contains("ValidationError"), "{error}");
        assert!(error.contains("/key"), "{error}");
        let result = root.prefix(invalid).delete("open").await;
        let error = format!("{result:?}");
        assert!(error.contains("ValidationError"), "{error}");
        assert!(error.contains("/key"), "{error}");
    }
    for operation in [
        root.prefix("inspection")
            .put(
                "/open",
                &Draft {
                    title: "invalid".into(),
                },
            )
            .await
            .map(|_| ()),
        root.prefix("inspection").delete("/open").await.map(|_| ()),
    ] {
        let error = format!("{operation:?}");
        assert!(error.contains("ValidationError"), "{error}");
        assert!(error.contains("/key"), "{error}");
    }
    let invalid_list = root
        .prefix("/inspection")
        .list(&ListStateOptions::default())
        .await;
    let error = format!("{invalid_list:?}");
    assert!(error.contains("ValidationError"), "{error}");
    assert!(error.contains("/prefix"), "{error}");
    let store = MapStateStore::<_, Draft>::new(&caller, "drafts").prefix("inspection/active");
    let value = Draft {
        title: "Open".into(),
    };
    let created = store.put("open", &value).await.expect("write map State");
    let revision = put_revision(&created);
    assert!(
        matches!(store.get("open").await.expect("read map State"), StateGetResult::Found { entry, .. } if entry.value == value)
    );
    let page = store
        .list(&ListStateOptions {
            offset: None,
            limit: Some(10),
        })
        .await
        .expect("list map State");
    assert_eq!(page.count, 1);
    assert!(
        matches!(&page.entries[0], StateValue::Current(entry) if entry.key == "inspection/active/open")
    );
    assert!(
        store
            .delete_with_options(
                "open",
                &DeleteStateOptions {
                    expected_revision: Some(revision)
                }
            )
            .await
            .expect("delete map State")
            .deleted
    );
    assert!(matches!(
        store.get("open").await.expect("read deleted map State"),
        StateGetResult::Missing { .. }
    ));
}

#[tokio::test]
async fn state_map_store_list_limit() {
    let (_runtime, caller) = client().await;
    let store = MapStateStore::<_, Draft>::new(&caller, "drafts");
    for key in ["c", "a", "b"] {
        store
            .put(key, &Draft { title: key.into() })
            .await
            .expect("write map item");
    }
    let page = store
        .list(&ListStateOptions {
            offset: None,
            limit: Some(2),
        })
        .await
        .expect("list bounded map State");
    assert_eq!(page.count, 3);
    assert_eq!(page.entries.len(), 2);
    assert_eq!(page.next_offset, Some(2));
    let keys = page
        .entries
        .iter()
        .map(|entry| match entry {
            StateValue::Current(entry) => entry.key.as_str(),
            StateValue::MigrationRequired(_) => panic!("new map value requires migration"),
        })
        .collect::<Vec<_>>();
    assert_eq!(keys, ["a", "b"]);
    let count = store
        .list(&ListStateOptions {
            offset: None,
            limit: Some(0),
        })
        .await
        .expect("count map State");
    assert!(count.entries.is_empty());
    assert_eq!(count.count, 3);
    assert_eq!(count.next_offset, None);

    let running = Arc::new(AtomicBool::new(true));
    let writes = Arc::new(AtomicUsize::new(0));
    let writer_running = running.clone();
    let writer_writes = writes.clone();
    let writer_caller = caller.clone();
    let writer = tokio::spawn(async move {
        let writer_store = MapStateStore::<_, Draft>::new(&writer_caller, "drafts");
        while writer_running.load(Ordering::Acquire) {
            let index = writer_writes.fetch_add(1, Ordering::AcqRel);
            writer_store
                .put(
                    &format!("concurrent/{index}"),
                    &Draft {
                        title: format!("concurrent-{index}"),
                    },
                )
                .await
                .map_err(|error| format!("{error:?}"))?;
        }
        Ok::<_, String>(())
    });
    while writes.load(Ordering::Acquire) < 3 {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let concurrent_page = tokio::time::timeout(
        Duration::from_secs(5),
        store.list(&ListStateOptions {
            offset: None,
            limit: Some(2),
        }),
    )
    .await
    .expect("bounded State.List timed out under concurrent writes")
    .expect("list State under concurrent writes");
    running.store(false, Ordering::Release);
    writer
        .await
        .expect("join concurrent State writer")
        .expect("write concurrent State entries");
    assert!(concurrent_page.entries.len() <= 2);
}

#[tokio::test]
async fn state_ttl_expiry_is_logically_absent() {
    let (runtime, caller) = client().await;
    let store = ValueStateStore::<_, Preferences>::new(&caller, "preferences");
    let map = MapStateStore::<_, Draft>::new(&caller, "drafts");
    store
        .put_with_options(
            &Preferences {
                theme: "short".into(),
            },
            &PutStateOptions {
                ttl_ms: Some(2_000),
                expected_revision: ExpectedPutRevision::CreateIfAbsent,
            },
        )
        .await
        .expect("write expiring State value");
    map.put_with_options(
        "short",
        &Draft {
            title: "short".into(),
        },
        &PutStateOptions {
            ttl_ms: Some(2_000),
            expected_revision: ExpectedPutRevision::CreateIfAbsent,
        },
    )
    .await
    .expect("write expiring map State");
    map.put_with_options(
        "admin-expired",
        &Draft {
            title: "admin-expired".into(),
        },
        &PutStateOptions {
            ttl_ms: Some(2_000),
            expected_revision: ExpectedPutRevision::CreateIfAbsent,
        },
    )
    .await
    .expect("write admin-expiring map State");
    assert!(matches!(
        store.get().await.expect("read live expiring value"),
        StateGetResult::Found { .. }
    ));
    tokio::time::sleep(Duration::from_millis(2_100)).await;
    assert!(matches!(
        store.get().await.expect("read expired value"),
        StateGetResult::Missing { .. }
    ));
    assert!(matches!(
        map.get("short").await.expect("read expired map value"),
        StateGetResult::Missing { .. }
    ));
    let context = caller
        .authorization_context()
        .expect("read caller context")
        .expect("caller context");
    let user_id = context.context["principal"]["id"]
        .as_str()
        .expect("user principal id");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(2))
        .await
        .expect("bootstrap URL");
    let contract = runtime
        .scoped_contract(
            &trellis_test::TrellisTestContract::from_manifest_json(CLIENT_CONTRACT_JSON)
                .expect("client contract"),
        )
        .expect("scope client contract");
    assert!(
        !runtime
            .admin()
            .state_admin_delete(
                &bootstrap_url,
                &StateAdminDeleteRequest::UserApp {
                    contract_digest: contract.digest().to_owned(),
                    contract_id: contract.id().to_owned(),
                    expected_revision: None,
                    key: Some("admin-expired".into()),
                    store: "drafts".into(),
                    user_id: user_id.to_owned(),
                },
            )
            .await
            .expect("admin delete expired State entry")
            .deleted
    );
    assert!(matches!(
        map.get("admin-expired")
            .await
            .expect("read admin-cleaned expired entry"),
        StateGetResult::Missing { .. }
    ));
    assert!(map
        .list(&ListStateOptions::default())
        .await
        .expect("list expired map value")
        .entries
        .is_empty());
    assert!(!store.delete().await.expect("delete expired value").deleted);
    assert!(
        !map.delete("short")
            .await
            .expect("delete expired map value")
            .deleted
    );
    assert!(
        store
            .put_with_options(
                &Preferences {
                    theme: "replacement".into()
                },
                &PutStateOptions {
                    ttl_ms: None,
                    expected_revision: ExpectedPutRevision::CreateIfAbsent
                },
            )
            .await
            .expect("replace expired State value")
            .applied
    );
    assert!(
        map.put_with_options(
            "short",
            &Draft {
                title: "replacement".into(),
            },
            &PutStateOptions {
                ttl_ms: None,
                expected_revision: ExpectedPutRevision::CreateIfAbsent,
            },
        )
        .await
        .expect("replace expired map State")
        .applied
    );
}

#[tokio::test]
async fn state_store_kind_and_schema_validation() {
    let (_runtime, caller) = client().await;
    let values = ValueStateStore::<_, serde_json::Value>::new(&caller, "preferences");
    let invalid = values.put(&serde_json::json!({"compact": true})).await;
    assert!(format!("{invalid:?}").contains("ValidationError"));
    let wrong_kind = MapStateStore::<_, Draft>::new(&caller, "preferences")
        .get("key")
        .await;
    assert!(format!("{wrong_kind:?}").contains("ValidationError"));
}

#[tokio::test]
async fn state_raw_bucket_access_is_denied() {
    let (_runtime, caller) = client().await;
    let jetstream = async_nats::jetstream::new(caller.integration_test_nats());
    assert!(
        jetstream.get_key_value("trellis_state").await.is_err(),
        "ordinary State caller opened the raw State bucket"
    );
}

#[tokio::test]
async fn state_agent_normal_access_is_denied() {
    let runtime = trellis_test::TrellisTestRuntime::start(
        trellis_test::TrellisTestRuntimeOptions::repo_platform(),
    )
    .await
    .expect("start live Trellis runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("bootstrap URL");
    let contract = trellis_test::TrellisTestContract::from_manifest_json(AGENT_CONTRACT_JSON)
        .expect("agent contract");
    let agent = runtime
        .admin()
        .connect_client(&bootstrap_url, &contract)
        .await
        .expect("connect agent");
    let result = ValueStateStore::<_, String>::new(&agent, "value")
        .get()
        .await;
    assert!(format!("{result:?}").contains("AuthError"));
}

#[tokio::test]
async fn state_service_normal_access_is_denied() {
    let runtime = trellis_test::TrellisTestRuntime::start(
        trellis_test::TrellisTestRuntimeOptions::repo_platform(),
    )
    .await
    .expect("start live Trellis runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("bootstrap URL");
    let contract = trellis_test::TrellisTestContract::from_manifest_json(SERVICE_CONTRACT_JSON)
        .expect("service contract");
    let key = runtime
        .admin()
        .provision_service_instance(&bootstrap_url, &contract, Some("state-service"), None)
        .await
        .expect("provision State service");
    let service = trellis_test::connect_service_runtime::<StateServiceContract>(
        runtime.trellis_url(),
        SERVICE_CONTRACT_JSON,
        &key,
    )
    .await
    .expect("connect State service");
    let result = service
        .integration_test_request_json_value(
            "rpc.v1.State.Get",
            &serde_json::json!({"store": "preferences"}),
        )
        .await;
    assert!(format!("{result:?}").contains("AuthError"));
}

#[tokio::test]
async fn state_migration_required_is_returned_live() {
    let (runtime, caller_v1) = client().await;
    let v1 = ValueStateStore::<_, serde_json::Value>::new(&caller_v1, "preferences");
    let map_v1 = MapStateStore::<_, serde_json::Value>::new(&caller_v1, "drafts");
    let written = v1
        .put(&serde_json::json!({"theme": "dark"}))
        .await
        .expect("write v1 State value");
    let old_value_revision = put_revision(&written);
    let map_written = map_v1
        .put("old", &serde_json::json!({"title": "old"}))
        .await
        .expect("write v1 map State value");
    let old_map_revision = put_revision(&map_written);
    let v1_contract = runtime
        .scoped_contract(
            &trellis_test::TrellisTestContract::from_manifest_json(CLIENT_CONTRACT_JSON)
                .expect("v1 contract"),
        )
        .expect("scope v1 contract");
    let old_writer_digest = v1_contract.digest().to_owned();
    let context = caller_v1
        .authorization_context()
        .expect("read v1 caller context")
        .expect("v1 caller context");
    let user_id = context.context["principal"]["id"]
        .as_str()
        .expect("user principal id")
        .to_owned();
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(2))
        .await
        .expect("bootstrap URL");
    let v2_contract =
        trellis_test::TrellisTestContract::from_manifest_json(MIGRATION_CONTRACT_JSON)
            .expect("v2 contract");
    let mut admin = runtime.admin();
    let caller_v2 = admin
        .connect_client(&bootstrap_url, &v2_contract)
        .await
        .expect("connect v2 State client");
    let v2 = ValueStateStore::<_, serde_json::Value>::new(&caller_v2, "preferences");
    let map_v2 = MapStateStore::<_, serde_json::Value>::new(&caller_v2, "drafts");
    let migration = v2.get().await.expect("read old State value through v2");
    let StateGetResult::MigrationRequired(migration) = migration else {
        panic!("old State value did not require migration");
    };
    assert_eq!(migration.entry.value, serde_json::json!({"theme": "dark"}));
    assert_eq!(migration.state_version, "v1");
    assert_eq!(migration.current_state_version, "preferences.v2");
    assert_eq!(migration.writer_contract_digest, old_writer_digest);
    assert_eq!(migration.entry.revision, old_value_revision);

    let map_migration = map_v2
        .get("old")
        .await
        .expect("read old map State value through v2");
    let StateGetResult::MigrationRequired(map_migration) = map_migration else {
        panic!("old map State value did not require migration");
    };
    assert_eq!(
        map_migration.entry.value,
        serde_json::json!({"title": "old"})
    );
    assert_eq!(map_migration.state_version, "v1");
    assert_eq!(map_migration.current_state_version, "drafts.v2");
    assert_eq!(map_migration.writer_contract_digest, old_writer_digest);
    assert_eq!(map_migration.entry.revision, old_map_revision);
    let listed = map_v2
        .list(&ListStateOptions {
            offset: None,
            limit: Some(10),
        })
        .await
        .expect("list old map State value through v2");
    assert_eq!(listed.count, 1);
    assert!(matches!(
        &listed.entries[0],
        StateValue::MigrationRequired(item)
            if item.entry.value == serde_json::json!({"title": "old"})
                && item.entry.revision == old_map_revision
                && item.state_version == "v1"
                && item.current_state_version == "drafts.v2"
                && item.writer_contract_digest == old_writer_digest
    ));

    let v2_contract = runtime
        .scoped_contract(&v2_contract)
        .expect("scope v2 contract");
    for (store, key, revision, current_version, value) in [
        (
            "preferences",
            None,
            old_value_revision.as_str(),
            "preferences.v2",
            serde_json::json!({"theme": "dark"}),
        ),
        (
            "drafts",
            Some("old".to_owned()),
            old_map_revision.as_str(),
            "drafts.v2",
            serde_json::json!({"title": "old"}),
        ),
    ] {
        let response = admin
            .state_admin_get(
                &bootstrap_url,
                &StateAdminGetRequest::UserApp {
                    contract_digest: v2_contract.digest().to_owned(),
                    contract_id: v2_contract.id().to_owned(),
                    key,
                    store: store.into(),
                    user_id: user_id.clone(),
                },
            )
            .await
            .expect("admin get migration-required State");
        let response = serde_json::to_value(response).expect("encode admin migration response");
        assert_eq!(response["migrationRequired"], true);
        assert_eq!(response["entry"]["value"], value);
        assert_eq!(response["entry"]["revision"], revision);
        assert_eq!(response["stateVersion"], "v1");
        assert_eq!(response["currentStateVersion"], current_version);
        assert_eq!(response["writerContractDigest"], old_writer_digest);
    }
    let admin_list = admin
        .state_admin_list(
            &bootstrap_url,
            &StateAdminListRequest::UserApp {
                contract_digest: v2_contract.digest().to_owned(),
                contract_id: v2_contract.id().to_owned(),
                limit: 10,
                offset: None,
                prefix: None,
                store: "drafts".into(),
                user_id,
            },
        )
        .await
        .expect("admin list migration-required map State");
    let admin_list = serde_json::to_value(admin_list).expect("encode admin migration list");
    assert_eq!(admin_list["count"], 1);
    assert_eq!(admin_list["entries"][0]["migrationRequired"], true);
    assert_eq!(
        admin_list["entries"][0]["entry"]["value"],
        serde_json::json!({"title": "old"})
    );
    assert_eq!(
        admin_list["entries"][0]["entry"]["revision"],
        old_map_revision
    );
    assert_eq!(admin_list["entries"][0]["stateVersion"], "v1");
    assert_eq!(admin_list["entries"][0]["currentStateVersion"], "drafts.v2");
    assert_eq!(
        admin_list["entries"][0]["writerContractDigest"],
        old_writer_digest
    );

    let migrated = v2
        .put_with_options(
            &serde_json::json!({"theme": "dark", "compact": true}),
            &PutStateOptions {
                ttl_ms: None,
                expected_revision: ExpectedPutRevision::Revision(old_value_revision),
            },
        )
        .await
        .expect("write migrated State value");
    assert!(migrated.applied);
    let map_migrated = map_v2
        .put_with_options(
            "old",
            &serde_json::json!({"title": "old", "pinned": true}),
            &PutStateOptions {
                ttl_ms: None,
                expected_revision: ExpectedPutRevision::Revision(old_map_revision),
            },
        )
        .await
        .expect("write migrated map State value");
    assert!(map_migrated.applied);
    assert!(matches!(
        v2.get().await.expect("read migrated value"),
        StateGetResult::Found { entry, .. }
            if entry.value == serde_json::json!({"theme": "dark", "compact": true})
    ));
    assert!(matches!(
        map_v2.get("old").await.expect("read migrated map value"),
        StateGetResult::Found { entry, .. }
            if entry.value == serde_json::json!({"title": "old", "pinned": true})
    ));
}

#[tokio::test]
async fn state_lineage_survives_compatible_contract_digest_change() {
    let runtime = trellis_test::TrellisTestRuntime::start(
        trellis_test::TrellisTestRuntimeOptions::repo_platform(),
    )
    .await
    .expect("start live Trellis runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("bootstrap URL");
    let contract = trellis_test::TrellisTestContract::from_manifest_json(CLIENT_CONTRACT_JSON)
        .expect("v1 contract");
    let (caller, reconnect) = runtime
        .admin()
        .connect_client_with_session_seed_reconnectable(
            &bootstrap_url,
            &contract,
            URL_SAFE_NO_PAD.encode([7_u8; 32]),
        )
        .await
        .expect("connect initial State client");
    let store = ValueStateStore::<_, Preferences>::new(&caller, "preferences");
    store
        .put(&Preferences {
            theme: "lineage".into(),
        })
        .await
        .expect("write initial State value");
    let reconnected = reconnect
        .connect_bound_only()
        .await
        .expect("reconnect session");
    assert!(matches!(
        ValueStateStore::<_, Preferences>::new(&reconnected, "preferences")
            .get()
            .await
            .expect("read after reconnect"),
        StateGetResult::Found { .. }
    ));
    let compatible =
        trellis_test::TrellisTestContract::from_manifest_json(COMPATIBLE_CONTRACT_JSON)
            .expect("compatible contract");
    let upgraded = runtime
        .admin()
        .connect_client(&bootstrap_url, &compatible)
        .await
        .expect("connect compatible State client");
    assert!(matches!(
        ValueStateStore::<_, serde_json::Value>::new(&upgraded, "preferences")
            .get()
            .await
            .expect("read after compatible digest change"),
        StateGetResult::Found { .. }
    ));
}

#[tokio::test]
async fn state_contract_namespaces_are_isolated() {
    let (runtime, first) = client().await;
    ValueStateStore::<_, Preferences>::new(&first, "preferences")
        .put(&Preferences {
            theme: "first".into(),
        })
        .await
        .expect("write first contract State");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(2))
        .await
        .expect("bootstrap URL");
    let other = trellis_test::TrellisTestContract::from_manifest_json(OTHER_CONTRACT_JSON)
        .expect("other contract");
    let second = runtime
        .admin()
        .connect_client(&bootstrap_url, &other)
        .await
        .expect("connect other State contract");
    assert!(matches!(
        ValueStateStore::<_, Preferences>::new(&second, "preferences")
            .get()
            .await
            .expect("read isolated contract State"),
        StateGetResult::Missing { .. }
    ));
}

#[tokio::test]
async fn state_distinct_users_with_same_contract_are_isolated() {
    let runtime = trellis_test::TrellisTestRuntime::start(
        trellis_test::TrellisTestRuntimeOptions::repo_platform(),
    )
    .await
    .expect("start live Trellis runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("bootstrap URL");
    let contract = trellis_test::TrellisTestContract::from_manifest_json(CLIENT_CONTRACT_JSON)
        .expect("State client contract");
    let mut admin = runtime.admin();
    let user_a = admin
        .connect_client(&bootstrap_url, &contract)
        .await
        .expect("connect user A");
    let user_b = admin
        .connect_new_local_user(
            &bootstrap_url,
            &contract,
            format!("state-user-{}", ulid::Ulid::new()),
            "state-user-password-2026",
        )
        .await
        .expect("register and connect user B");
    let user_a_id = user_a
        .authorization_context()
        .expect("user A context")
        .expect("user A authorization context")
        .context["principal"]["id"]
        .as_str()
        .expect("user A principal")
        .to_owned();
    let user_b_id = user_b
        .authorization_context()
        .expect("user B context")
        .expect("user B authorization context")
        .context["principal"]["id"]
        .as_str()
        .expect("user B principal")
        .to_owned();
    assert_ne!(user_a_id, user_b_id);

    let store_a = ValueStateStore::<_, Preferences>::new(&user_a, "preferences");
    let store_b = ValueStateStore::<_, Preferences>::new(&user_b, "preferences");
    store_a
        .put(&Preferences {
            theme: "user-a".into(),
        })
        .await
        .expect("write user A State");
    assert!(matches!(
        store_b.get().await.expect("read user B State"),
        StateGetResult::Missing { .. }
    ));
    store_b
        .put(&Preferences {
            theme: "user-b".into(),
        })
        .await
        .expect("write user B State");
    assert!(matches!(
        store_a.get().await.expect("reread user A State"),
        StateGetResult::Found { entry, .. } if entry.value.theme == "user-a"
    ));
}

#[tokio::test]
async fn state_exact_resource_permission_is_required() {
    let runtime = trellis_test::TrellisTestRuntime::start(
        trellis_test::TrellisTestRuntimeOptions::repo_platform(),
    )
    .await
    .expect("start live Trellis runtime");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(10))
        .await
        .expect("bootstrap URL");
    let contract = trellis_test::TrellisTestContract::from_manifest_json(CLIENT_CONTRACT_JSON)
        .expect("State client contract");
    let mut admin = runtime.admin();
    let allowed = admin
        .connect_client(&bootstrap_url, &contract)
        .await
        .expect("connect State client with exact resource atom");
    assert!(matches!(
        ValueStateStore::<_, Preferences>::new(&allowed, "preferences")
            .get()
            .await
            .expect("State.Get succeeds with exact resource atom"),
        StateGetResult::Missing { .. }
    ));
    let context = allowed
        .authorization_context()
        .expect("read caller context")
        .expect("caller context");
    let user_id = context.context["principal"]["id"]
        .as_str()
        .expect("user principal id");
    let participant_id = context.context["participant"]["id"]
        .as_str()
        .expect("participant id");
    let sqlite = runtime.control_plane_sqlite();
    let rows = sqlite
        .query(
            "SELECT effective_grant_set_json FROM auth_materialized_authorities WHERE subject_id = ?1 AND participant_id = ?2",
            params![user_id, participant_id],
        )
        .expect("read materialized State grants");
    assert_eq!(rows.len(), 1);
    let mut grants: serde_json::Value = serde_json::from_str(
        rows[0]["effective_grant_set_json"]
            .as_str()
            .expect("materialized grants JSON"),
    )
    .expect("parse materialized grants");
    let permissions = grants["permissions"]
        .as_array_mut()
        .expect("grant permissions");
    let before = permissions.len();
    permissions.retain(|permission| {
        !(permission["action"] == "read"
            && permission["target"]["kind"] == "participantResource"
            && permission["target"]["participant"] == participant_id
            && permission["target"]["resource"] == "state"
            && permission["target"]["name"] == "preferences")
    });
    assert_eq!(permissions.len() + 1, before);
    assert!(permissions.iter().any(|permission| {
        permission["action"] == "call"
            && permission["target"]["kind"] == "apiSurface"
            && permission["target"]["name"] == "State.Get"
    }));
    let updated = serde_json::to_string(&grants).expect("encode reduced grants");
    assert_eq!(
        sqlite
            .execute(
                "UPDATE auth_materialized_authorities SET effective_grant_set_json = ?1 WHERE subject_id = ?2 AND participant_id = ?3",
                params![&updated, user_id, participant_id],
            )
            .expect("remove exact State resource atom")
            .rows_affected,
        1
    );
    allowed
        .refresh_authorization_context()
        .await
        .expect("issue fresh context without exact resource atom");
    let result = ValueStateStore::<_, Preferences>::new(&allowed, "preferences")
        .get()
        .await;
    assert!(format!("{result:?}").contains("AuthError"), "{result:?}");
}

#[tokio::test]
async fn state_admin_inspect_and_delete_state() {
    let (runtime, caller) = client().await;
    let value_store = ValueStateStore::<_, Preferences>::new(&caller, "preferences");
    value_store
        .put(&Preferences {
            theme: "admin".into(),
        })
        .await
        .expect("write admin target");
    let context = caller
        .authorization_context()
        .expect("read caller context")
        .expect("caller context");
    let user_id = context.context["principal"]["id"]
        .as_str()
        .expect("user principal id")
        .to_owned();
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(2))
        .await
        .expect("bootstrap URL");
    let mut admin = runtime.admin();
    let contract = trellis_test::TrellisTestContract::from_manifest_json(CLIENT_CONTRACT_JSON)
        .expect("client contract");
    let contract = runtime
        .scoped_contract(&contract)
        .expect("scope client contract");
    let target = StateAdminGetRequest::UserApp {
        contract_digest: contract.digest().to_owned(),
        contract_id: contract.id().to_owned(),
        key: None,
        store: "preferences".into(),
        user_id: user_id.clone(),
    };
    assert!(matches!(
        admin
            .state_admin_get(&bootstrap_url, &target)
            .await
            .expect("admin get"),
        StateAdminGetResponse::Variant2 { .. }
    ));
    let wrong_digest = admin
        .state_admin_get(
            &bootstrap_url,
            &StateAdminGetRequest::UserApp {
                contract_digest: "sha256:wrong".into(),
                contract_id: contract.id().to_owned(),
                key: None,
                store: "preferences".into(),
                user_id: user_id.clone(),
            },
        )
        .await;
    let error = format!("{wrong_digest:?}");
    assert!(error.contains("ValidationError"), "{error}");
    assert!(error.contains("/contractDigest"), "{error}");
    let listed = admin
        .state_admin_list(
            &bootstrap_url,
            &StateAdminListRequest::UserApp {
                contract_digest: contract.digest().to_owned(),
                contract_id: contract.id().to_owned(),
                limit: 10,
                offset: None,
                prefix: None,
                store: "drafts".into(),
                user_id: user_id.clone(),
            },
        )
        .await
        .expect("admin list");
    assert_eq!(listed.count, 0);
    assert!(
        admin
            .state_admin_delete(
                &bootstrap_url,
                &StateAdminDeleteRequest::UserApp {
                    contract_digest: contract.digest().to_owned(),
                    contract_id: contract.id().to_owned(),
                    expected_revision: None,
                    key: None,
                    store: "preferences".into(),
                    user_id,
                },
            )
            .await
            .expect("admin delete")
            .deleted
    );
    assert!(matches!(
        value_store.get().await.expect("read admin-deleted value"),
        StateGetResult::Missing { .. }
    ));
}

fn raw_value_state_key(caller: &trellis_rs::generated::Caller, store: &str) -> String {
    let context = caller
        .authorization_context()
        .expect("read caller context")
        .expect("caller context");
    let user_id = context.context["principal"]["id"]
        .as_str()
        .expect("user principal id");
    let participant_id = context.context["participant"]["id"]
        .as_str()
        .expect("participant id");
    let namespace = trellis_protocol::digest_json(&serde_json::json!({
        "scope": "userApp",
        "ownerId": user_id,
        "contractId": participant_id,
    }))
    .expect("State namespace digest");
    format!("value.{namespace}.{}", URL_SAFE_NO_PAD.encode(store))
}

#[tokio::test]
async fn state_admin_deletes_corrupt_state_entry() {
    let (runtime, caller) = client().await;
    let context = caller
        .authorization_context()
        .expect("read caller context")
        .expect("caller context");
    let user_id = context.context["principal"]["id"]
        .as_str()
        .expect("user principal id");
    let key = raw_value_state_key(&caller, "preferences");
    let contract = runtime
        .scoped_contract(
            &trellis_test::TrellisTestContract::from_manifest_json(CLIENT_CONTRACT_JSON)
                .expect("client contract"),
        )
        .expect("scope client contract");
    runtime
        .seed_raw_state_entry(trellis_test::TrellisRawStateEntry {
            key: key.clone(),
            value: serde_json::json!({
                "value": {"compact": true},
                "stateVersion": "v1",
                "writerContractDigest": contract.digest(),
                "updatedAt": "2026-08-09T12:34:56.789Z"
            }),
        })
        .await
        .expect("seed corrupt State entry");
    let store = ValueStateStore::<_, serde_json::Value>::new(&caller, "preferences");
    let corrupt = store.get().await;
    assert!(format!("{corrupt:?}").contains("UnexpectedError"));
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(2))
        .await
        .expect("bootstrap URL");
    assert!(
        runtime
            .admin()
            .state_admin_delete(
                &bootstrap_url,
                &StateAdminDeleteRequest::UserApp {
                    contract_digest: contract.digest().to_owned(),
                    contract_id: contract.id().to_owned(),
                    expected_revision: None,
                    key: None,
                    store: "preferences".into(),
                    user_id: user_id.to_owned(),
                },
            )
            .await
            .expect("admin delete corrupt State entry")
            .deleted
    );
    assert!(matches!(
        store
            .get()
            .await
            .expect("read cleaned schema-invalid entry"),
        StateGetResult::Missing { .. }
    ));
}

#[tokio::test]
async fn state_malformed_envelope_and_unknown_version_are_unexpected() {
    let (runtime, caller) = client().await;
    let context = caller
        .authorization_context()
        .expect("read caller context")
        .expect("caller context");
    let user_id = context.context["principal"]["id"]
        .as_str()
        .expect("user principal id")
        .to_owned();
    let key = raw_value_state_key(&caller, "preferences");
    let bootstrap_url = runtime
        .wait_for_bootstrap_url(Duration::from_secs(2))
        .await
        .expect("bootstrap URL");
    let contract = runtime
        .scoped_contract(
            &trellis_test::TrellisTestContract::from_manifest_json(CLIENT_CONTRACT_JSON)
                .expect("client contract"),
        )
        .expect("scope client contract");
    let target = |contract_digest: String| StateAdminDeleteRequest::UserApp {
        contract_digest,
        contract_id: contract.id().to_owned(),
        expected_revision: None,
        key: None,
        store: "preferences".into(),
        user_id: user_id.clone(),
    };

    runtime
        .seed_raw_state_entry(trellis_test::TrellisRawStateEntry {
            key: key.clone(),
            value: serde_json::json!({
                "value": {"theme": "malformed"},
                "writerContractDigest": contract.digest(),
                "updatedAt": "2026-08-09T12:34:56.789Z"
            }),
        })
        .await
        .expect("seed malformed State envelope");
    let store = ValueStateStore::<_, Preferences>::new(&caller, "preferences");
    let malformed = store.get().await;
    assert!(format!("{malformed:?}").contains("UnexpectedError"));
    assert!(
        runtime
            .admin()
            .state_admin_delete(&bootstrap_url, &target(contract.digest().to_owned()))
            .await
            .expect("admin delete malformed envelope")
            .deleted
    );
    assert!(matches!(
        store.get().await.expect("read cleaned malformed entry"),
        StateGetResult::Missing { .. }
    ));

    runtime
        .seed_raw_state_entry(trellis_test::TrellisRawStateEntry {
            key: key.clone(),
            value: serde_json::json!({
                "value": {"theme": "unknown"},
                "stateVersion": "preferences.unknown",
                "writerContractDigest": contract.digest(),
                "updatedAt": "2026-08-09T12:34:56.789Z"
            }),
        })
        .await
        .expect("seed unknown State version");
    let unknown = store.get().await;
    assert!(format!("{unknown:?}").contains("UnexpectedError"));
    assert!(
        runtime
            .admin()
            .state_admin_delete(&bootstrap_url, &target(contract.digest().to_owned()))
            .await
            .expect("admin delete unknown State version")
            .deleted
    );
    assert!(matches!(
        store
            .get()
            .await
            .expect("read cleaned unknown-version entry"),
        StateGetResult::Missing { .. }
    ));

    runtime
        .seed_raw_state_entry(trellis_test::TrellisRawStateEntry {
            key,
            value: serde_json::json!({
                "value": {"theme": "invalid-writer"},
                "stateVersion": "v1",
                "writerContractDigest": "invalid-writer-digest",
                "updatedAt": "2026-08-09T12:34:56.789Z"
            }),
        })
        .await
        .expect("seed invalid writer digest");
    let invalid_writer = store.get().await;
    assert!(format!("{invalid_writer:?}").contains("UnexpectedError"));
    assert!(
        runtime
            .admin()
            .state_admin_delete(&bootstrap_url, &target(contract.digest().to_owned()))
            .await
            .expect("admin delete invalid-writer envelope")
            .deleted
    );
    assert!(matches!(
        store
            .get()
            .await
            .expect("read cleaned invalid-writer entry"),
        StateGetResult::Missing { .. }
    ));
}
