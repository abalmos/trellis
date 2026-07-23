#![allow(missing_docs)]

#[path = "integration/support/mod.rs"]
mod support;

#[path = "integration/rpc.rs"]
mod rpc;

#[path = "integration/events.rs"]
mod events;

#[path = "integration/operations.rs"]
mod operations;

#[path = "integration/feeds.rs"]
mod feeds;

#[path = "integration/state.rs"]
mod state;

#[path = "integration/transfer.rs"]
mod transfer;

#[path = "integration/resources.rs"]
mod resources;

#[path = "integration/jobs.rs"]
mod jobs;

#[path = "integration/health.rs"]
mod health;

#[path = "integration/runtime_ownership.rs"]
mod runtime_ownership;

#[path = "integration/event_consumers.rs"]
mod event_consumers;

#[path = "integration/prepared_events.rs"]
mod prepared_events;

fn generated_caller(client: &trellis_rs::generated::Caller) -> &trellis_rs::generated::Caller {
    client
}

fn wire<T: serde::de::DeserializeOwned, S: serde::Serialize>(value: S) -> T {
    serde_json::from_value(serde_json::to_value(value).expect("serialize test wire value"))
        .expect("deserialize test wire value")
}

#[test]
fn rust_integration_manifest_conforms_to_shared_matrix() {
    support::cases::assert_rust_manifest_conforms_to_matrix();
}

#[test]
fn rust_service_integration_manifest_conforms_to_shared_matrix() {
    support::cases::assert_rust_service_manifest_conforms_to_matrix();
}
