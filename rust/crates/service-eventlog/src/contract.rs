//! Runtime adapter for hosting the standard Trellis Event Log API.

use trellis_rs::sdk::eventlog::contract as generated_contract;
use trellis_rs::service::{BootstrapContractRef, GeneratedServiceContract};

use crate::wire::{
    EventLogConsumersInspectRpc, EventLogConsumersQueryRpc, EventLogInspectRpc, EventLogMetricsRpc,
    EventLogQueryRpc,
};
use trellis_rs::service::RpcDescriptor;

/// Runtime service name for the Event Log host.
pub const SERVICE_NAME: &str = "trellis-service-eventlog";
/// Exact RPC subjects served by the Event Log service.
pub const EVENTLOG_RPC_SUBJECTS: &[&str] = &[
    <EventLogQueryRpc as RpcDescriptor>::SUBJECT,
    <EventLogInspectRpc as RpcDescriptor>::SUBJECT,
    <EventLogMetricsRpc as RpcDescriptor>::SUBJECT,
    <EventLogConsumersQueryRpc as RpcDescriptor>::SUBJECT,
    <EventLogConsumersInspectRpc as RpcDescriptor>::SUBJECT,
];

pub use generated_contract::{contract_manifest, CONTRACT_DIGEST, CONTRACT_ID, CONTRACT_JSON};

/// Generated contract marker used by the Rust Trellis service runtime facade.
#[derive(Debug, Clone, Copy)]
pub struct EventLogContract;

impl GeneratedServiceContract for EventLogContract {
    const CONTRACT_ID: &'static str = generated_contract::CONTRACT_ID;
    const CONTRACT_DIGEST: &'static str = generated_contract::CONTRACT_DIGEST;
    const CONTRACT_JSON: &'static str = generated_contract::CONTRACT_JSON;
}

/// Return the contract id/digest pair expected by the Event Log service.
pub fn expected_contract() -> BootstrapContractRef {
    BootstrapContractRef {
        id: CONTRACT_ID.to_string(),
        digest: CONTRACT_DIGEST.to_string(),
    }
}
