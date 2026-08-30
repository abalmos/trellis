use trellis_rs::client::FeedDescriptor;
use trellis_rs::internal_sdk::jobs::api as generated_contract;
#[path = "../contracts/trellis_jobs.rs"]
mod jobs_contract_source;

#[test]
fn rust_api_builder_matches_generated_jobs_sdk() {
    let artifacts = jobs_contract_source::contract_artifacts().expect("jobs contract artifacts");
    assert_eq!(
        artifacts
            .api()
            .normalized_value()
            .expect("normalized jobs API"),
        generated_contract::api_artifact()
    );
    assert_eq!(artifacts.participant().id(), "trellis.jobs@v1");
}

#[test]
fn generated_jobs_watch_subject_matches_canonical_feed_subject() {
    let api = generated_contract::api_artifact();
    let watch = api
        .get("feeds")
        .and_then(|feeds| feeds.get("Jobs.Watch"))
        .expect("Jobs.Watch feed");

    assert!(watch.get("subject").is_none());
    assert_eq!(
        trellis_rs::internal_sdk::jobs::feeds::JobsWatchFeedDescriptor::SUBJECT,
        "feed.v1.Jobs.Watch"
    );
}

#[test]
fn generated_jobs_contract_uses_scoped_rpc_capability_names() {
    let api = generated_contract::api_artifact();
    let capabilities = api
        .get("capabilities")
        .and_then(serde_json::Value::as_object)
        .expect("jobs API capabilities");

    assert!(capabilities.contains_key("trellis.jobs::admin.read"));
    assert!(capabilities.contains_key("trellis.jobs::admin.mutate"));

    let jobs_cancel = capabilities
        .get("trellis.jobs::admin.mutate")
        .expect("jobs mutate capability");
    assert_eq!(
        jobs_cancel
            .get("allows")
            .and_then(serde_json::Value::as_array)
            .and_then(|allows| allows.first())
            .and_then(|allow| allow.get("target"))
            .and_then(|target| target.get("name"))
            .and_then(serde_json::Value::as_str),
        Some("Jobs.Cancel")
    );

    let rpc = api.get("rpc").expect("jobs API RPC");
    assert!(rpc.get("Jobs.List").is_none());
    assert!(rpc.get("Jobs.Get").is_none());

    let jobs_get = capabilities
        .get("trellis.jobs::admin.read")
        .expect("jobs read capability");
    assert!(jobs_get
        .get("allows")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|allows| {
            allows.iter().any(|allow| {
                allow
                    .get("target")
                    .and_then(|target| target.get("name"))
                    .and_then(serde_json::Value::as_str)
                    == Some("Jobs.Inspect")
            })
        }));
}

#[test]
fn generated_jobs_contract_omits_runtime_bootstrap_uses() {
    let api = generated_contract::api_artifact();
    assert!(api.get("uses").is_none());
}

#[test]
fn generated_jobs_contract_declares_full_job_state_set() {
    let api = generated_contract::api_artifact();
    let schema = api
        .get("schemas")
        .and_then(|schemas| schemas.get("JobState"))
        .expect("JobState schema");
    let states = schema
        .get("anyOf")
        .and_then(|value| value.as_array())
        .expect("JobState anyOf")
        .iter()
        .map(|variant| {
            variant
                .get("const")
                .and_then(|value| value.as_str())
                .expect("state const")
        })
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        states,
        [
            "active",
            "cancelled",
            "completed",
            "dead",
            "dismissed",
            "expired",
            "failed",
            "pending",
            "retry",
            "skipped",
            "stale",
        ]
        .into_iter()
        .collect()
    );
}
