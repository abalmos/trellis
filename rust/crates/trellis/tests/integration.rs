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

#[path = "integration/transfer.rs"]
mod transfer;

#[path = "integration/resources.rs"]
mod resources;

#[path = "integration/jobs.rs"]
mod jobs;

#[path = "integration/health.rs"]
mod health;

#[path = "integration/idl_demo.rs"]
mod idl_demo;

#[path = "integration/runtime_ownership.rs"]
mod runtime_ownership;

#[path = "integration/event_consumers.rs"]
mod event_consumers;

#[path = "integration/prepared_events.rs"]
mod prepared_events;

#[path = "integration/state.rs"]
mod state;

#[path = "integration/cli.rs"]
mod cli;

#[path = "integration/auth.rs"]
mod auth;

#[path = "integration/authority_plan.rs"]
mod authority_plan;

fn generated_caller(client: &trellis_rs::generated::Caller) -> &trellis_rs::generated::Caller {
    client
}

fn wire<T: serde::de::DeserializeOwned, S: serde::Serialize>(value: S) -> T {
    serde_json::from_value(serde_json::to_value(value).expect("serialize test wire value"))
        .expect("deserialize test wire value")
}
