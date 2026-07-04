//! Runtime adapter for hosting the standard Trellis Jobs API.

use trellis_rs::sdk::jobs::contract as generated_contract;
use trellis_rs::sdk::jobs::rpc::{
    JobsCancelRpc, JobsDismissDLQRpc, JobsGetKeyRpc, JobsHealthRpc, JobsInspectRpc, JobsListDLQRpc,
    JobsListServicesRpc, JobsQueryRpc, JobsReplayDLQRpc, JobsRetryRpc,
};
use trellis_rs::service::BootstrapContractRef;
use trellis_rs::service::GeneratedServiceContract;
use trellis_rs::service::RpcDescriptor;

/// Runtime service name for the Jobs admin host.
pub const SERVICE_NAME: &str = "trellis-service-jobs";
/// Exact RPC subjects served by the Jobs admin service.
pub const JOBS_RPC_SUBJECTS: &[&str] = &[
    <JobsHealthRpc as RpcDescriptor>::SUBJECT,
    <JobsListServicesRpc as RpcDescriptor>::SUBJECT,
    <JobsQueryRpc as RpcDescriptor>::SUBJECT,
    <JobsInspectRpc as RpcDescriptor>::SUBJECT,
    <JobsGetKeyRpc as RpcDescriptor>::SUBJECT,
    <JobsCancelRpc as RpcDescriptor>::SUBJECT,
    <JobsRetryRpc as RpcDescriptor>::SUBJECT,
    <JobsListDLQRpc as RpcDescriptor>::SUBJECT,
    <JobsReplayDLQRpc as RpcDescriptor>::SUBJECT,
    <JobsDismissDLQRpc as RpcDescriptor>::SUBJECT,
];

pub use generated_contract::{contract_manifest, CONTRACT_DIGEST, CONTRACT_ID, CONTRACT_JSON};
pub use trellis_rs::sdk::jobs::rpc;

/// Generated contract marker used by the Rust Trellis service runtime facade.
#[derive(Debug, Clone, Copy)]
pub struct JobsContract;

impl GeneratedServiceContract for JobsContract {
    const CONTRACT_ID: &'static str = generated_contract::CONTRACT_ID;
    const CONTRACT_DIGEST: &'static str = generated_contract::CONTRACT_DIGEST;
    const CONTRACT_JSON: &'static str = generated_contract::CONTRACT_JSON;
}

/// Return the contract id/digest pair expected by the Jobs admin service.
pub fn expected_contract() -> BootstrapContractRef {
    BootstrapContractRef {
        id: CONTRACT_ID.to_string(),
        digest: CONTRACT_DIGEST.to_string(),
    }
}
