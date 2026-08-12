//! Contract authoring and native API/participant artifacts for Trellis.
//!
//! This crate owns authoring validation, canonicalization, and native artifact
//! construction. It intentionally stays transport-agnostic so the runtime and
//! generators can share one contract source of truth.
//!
//! Generators use [`LoadedApi`] to produce SDK-local `TRELLIS.md` files for
//! AI agents. Those files should be treated as package-local summaries of the
//! canonical API: contract id, owned RPC/event/feed/operation names,
//! used dependency surfaces, and current TypeScript/Rust facade forms.

mod api_authoring;
mod authoring_model;
mod canonical;
mod error;
mod native_artifacts;
mod pagination;
mod source_parser;

pub use api_authoring::{
    contract_capability_namespace, event, feed, global_capability_name, job_queue, kv, operation,
    rpc, schema_ref, state, store, use_contract,
};
pub use authoring_model::{
    ApiRenderModel, ContractCapabilities, ContractCapabilityMetadata, ContractDocs,
    ContractErrorDecl, ContractErrorRef, ContractEvent, ContractEventConsumerGroup,
    ContractEventConsumerOrdering, ContractEventConsumerReplay, ContractExports, ContractFeed,
    ContractJobQueueResource, ContractKind, ContractKvResource, ContractOperation,
    ContractOperationSignal, ContractOperationTransfer, ContractOperationTransferDirection,
    ContractResources, ContractRpcMethod, ContractRpcTransfer, ContractRpcTransferDirection,
    ContractSchemaRef, ContractStateKind, ContractStateStore, ContractStoreResource,
    ContractUseFeed, ContractUseOperation, ContractUsePubSub, ContractUseRef, ContractUseRpc,
    ContractUses, FeedCapabilities, JobKeyConcurrencyDescriptor, JobKeyConcurrencyStalePolicy,
    JobQueueDepthDescriptor, JobQueueWhenFullPolicy, LoadedApi, LoadedParticipant,
    OperationCapabilities, ParticipantOperationUseRenderModel, ParticipantRenderModel,
    ParticipantUseRenderModel, ParticipantUsesRenderModel, PubSubCapabilities, RpcCapabilities,
    API_FORMAT_V1,
};
pub use canonical::{canonicalize_json, digest_json, sha256_base64url};
pub use error::ContractsError;
pub use native_artifacts::{ApiBuilder, ContractArtifacts, ContractBuilder};
pub use pagination::{PageRequest, PageResponse};
pub use source_parser::{
    load_json_value, load_participant_source, load_sdk_source, source_paths_in_dir,
};
pub use trellis_protocol::ApiArtifactV1;

#[cfg(test)]
mod tests;
