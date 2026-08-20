//! Rust source for the `trellis.core@v1` contract manifest.

use serde_json::{json, Value};
use trellis_contracts::{
    ApiArtifactV1, ApiBuilder, ContractArtifacts, ContractBuilder, ContractCapabilityMetadata,
    ContractKind, ContractsError,
};

const READ_CAPABILITY: &str = "authority.read";
const UNEXPECTED_ERROR: &str = "UnexpectedError";
const VALIDATION_ERROR: &str = "ValidationError";

/// Build the canonical Trellis Core service API artifact.
pub fn api_artifact() -> Result<ApiArtifactV1, ContractsError> {
    ApiBuilder::authoring(
        "trellis.core@v1",
        "Trellis Core",
        "Trellis runtime RPCs available to all connected participants.",
    )
    .docs_with_summary(
        "Runtime authority and binding APIs.",
        "Exposes runtime bindings and surface availability checks used by platform participants.",
    )
    .capability(
        READ_CAPABILITY,
        ContractCapabilityMetadata {
            display_name: "Read participant authority".to_string(),
            description: "Inspect native participant surface authority.".to_string(),
            consequence: None,
        },
    )
    .schema("TrellisSurfaceStatusRequest", status_request_schema())
    .schema("TrellisSurfaceStatusResponse", status_response_schema())
    .rpc(
        "Trellis.Surface.Status",
        trellis_contracts::rpc(
            "v1",
            "rpc.v1.Trellis.Surface.Status",
            "TrellisSurfaceStatusRequest",
            "TrellisSurfaceStatusResponse",
        )
        .with_call_capabilities([READ_CAPABILITY])
        .with_error_types([UNEXPECTED_ERROR, VALIDATION_ERROR])
        .docs_with_summary(
            "Inspect surface availability.",
            "Reports capability and deployment authority status for a contract-owned surface.",
        ),
    )
    .build()
}

/// Build the native Trellis Core participant and API artifacts.
pub fn contract_artifacts() -> Result<ContractArtifacts, ContractsError> {
    let api = api_artifact()?.normalized_value()?;
    ContractBuilder::from_api(api, ContractKind::Service)?.build()
}

fn status_request_schema() -> Value {
    json!({
        "type": "object",
        "required": ["contractId", "kind", "surface"],
        "properties": {
            "contractId": { "type": "string", "minLength": 1 },
            "kind": {
                "anyOf": [
                    { "const": "rpc", "type": "string" },
                    { "const": "operation", "type": "string" },
                    { "const": "event", "type": "string" },
                    { "const": "feed", "type": "string" }
                ]
            },
            "surface": { "type": "string", "minLength": 1 },
            "action": {
                "anyOf": [
                    { "const": "call", "type": "string" },
                    { "const": "publish", "type": "string" },
                    { "const": "subscribe", "type": "string" },
                    { "const": "observe", "type": "string" }
                ]
            }
        }
    })
}

fn status_response_schema() -> Value {
    json!({
        "type": "object",
        "required": ["status"],
        "properties": {
            "status": {
                "anyOf": [
                    {
                        "type": "object",
                        "required": ["state", "liveImplementer", "runtime"],
                        "properties": {
                            "state": { "const": "available", "type": "string" },
                            "liveImplementer": { "type": "boolean" },
                            "runtime": {
                                "anyOf": [
                                    { "const": "live", "type": "string" },
                                    { "const": "no_live_implementer", "type": "string" },
                                    { "const": "disabled", "type": "string" }
                                ]
                            }
                        }
                    },
                    {
                        "type": "object",
                        "required": ["state", "reason"],
                        "properties": {
                            "state": { "const": "unavailable", "type": "string" },
                            "reason": {
                                "anyOf": [
                                    { "const": "authority_unavailable", "type": "string" }
                                ]
                            }
                        }
                    },
                    {
                        "type": "object",
                        "required": ["state", "missingCapabilities"],
                        "properties": {
                            "state": { "const": "unauthorized", "type": "string" },
                            "missingCapabilities": {
                                "type": "array",
                                "items": { "type": "string" }
                            }
                        }
                    },
                    {
                        "type": "object",
                        "required": ["state", "contractId"],
                        "properties": {
                            "state": { "const": "unknown_contract", "type": "string" },
                            "contractId": { "type": "string", "minLength": 1 }
                        }
                    },
                    {
                        "type": "object",
                        "required": ["state", "contractId", "kind", "surface"],
                        "properties": {
                            "state": { "const": "unknown_surface", "type": "string" },
                            "contractId": { "type": "string", "minLength": 1 },
                            "kind": { "type": "string", "minLength": 1 },
                            "surface": { "type": "string", "minLength": 1 }
                        }
                    }
                ]
            }
        }
    })
}
