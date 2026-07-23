//! Generated Rust SDK crate for one Trellis contract.
const _: () = crate::generated::assert_abi(1);
/// Typed outbound adapters.
pub mod client;
/// Embedded contract identity and manifest.
pub mod contract;
/// Event descriptors.
pub mod events;
/// Feed descriptors.
pub mod feeds;
/// Job descriptors.
pub mod jobs;
/// Operation descriptors.
pub mod operations;
/// RPC descriptors and declared errors.
pub mod rpc;
/// JSON Schema constants.
pub mod schemas;
/// Generated wire types.
pub mod types;
pub use client::EventlogClient;
pub use contract::{contract_manifest, CONTRACT_DIGEST, CONTRACT_ID, CONTRACT_JSON, CONTRACT_NAME};
pub use feeds::*;
pub use rpc::*;
pub use types::*;
