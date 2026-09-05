//! Generated Rust SDK crate for one Trellis API.
const _: () = trellis_rs::generated::assert_abi(1);
/// Embedded API identity and artifact.
pub mod api;
/// Typed outbound adapters.
pub mod client;
/// Event descriptors.
pub mod events;
/// Operation descriptors.
pub mod operations;
/// RPC descriptors and declared errors.
pub mod rpc;
/// JSON Schema constants.
pub mod schemas;
/// Generated wire types.
pub mod types;
pub use api::{api_artifact, API_DIGEST, API_ID, API_JSON, API_NAME};
pub use client::AuthClient;
pub use events::*;
pub use operations::*;
pub use rpc::*;
pub use types::*;
