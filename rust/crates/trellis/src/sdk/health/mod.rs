//! Generated Rust SDK crate for one Trellis API.
const _: () = crate::generated::assert_abi(1);
/// Embedded API identity and artifact.
pub mod api;
/// Typed outbound adapters.
pub mod client;
/// Event descriptors.
pub mod events;
/// Feed descriptors.
pub mod feeds;
/// Operation descriptors.
pub mod operations;
/// RPC descriptors and declared errors.
pub mod rpc;
/// JSON Schema constants.
pub mod schemas;
/// Generated wire types.
pub mod types;
pub use api::{api_artifact, API_DIGEST, API_ID, API_JSON, API_NAME};
pub use client::HealthClient;
pub use events::*;
pub use feeds::*;
pub use rpc::*;
pub use types::*;
