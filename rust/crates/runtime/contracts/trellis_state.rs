//! Rust source for the `trellis.state@v1` contract manifest.

use serde_json::{json, Value};
use trellis_contracts::{
    ApiArtifact, ApiBuilder, ContractArtifacts, ContractBuilder, ContractKind, ContractsError,
};

const AUTH_ERROR: &str = "AuthError";
const UNEXPECTED_ERROR: &str = "UnexpectedError";
const VALIDATION_ERROR: &str = "ValidationError";

/// Build the canonical State service API artifact.
pub fn api_artifact() -> Result<ApiArtifact, ContractsError> {
    ApiBuilder::authoring(
        "trellis.state@v1",
        "1.0.0",
        "Trellis State",
        "Trellis-managed app state for authenticated app and device participants.",
    )
    .docs_with_summary(
        "Participant state storage APIs.",
        "Provides authenticated read, write, list, delete, and admin inspection APIs for Trellis-managed participant state.",
    )
    .schema("JsonValue", json!({}))
    .schema("StateAdminDeleteRequest", admin_request("delete"))
    .schema("StateAdminDeleteResponse", delete_response())
    .schema("StateAdminGetRequest", admin_request("get"))
    .schema("StateAdminGetResponse", get_response())
    .schema("StateAdminListRequest", admin_request("list"))
    .schema("StateAdminListResponse", list_response())
    .schema("StateDeleteRequest", delete_request())
    .schema("StateDeleteResponse", delete_response())
    .schema("StateEntry", state_entry())
    .schema("StateGetRequest", get_request())
    .schema("StateGetResponse", get_response())
    .schema("StateListRequest", list_request())
    .schema("StateListResponse", list_response())
    .schema("StateMigrationRequired", migration_required())
    .schema("StatePutRequest", put_request())
    .schema("StatePutResponse", put_response())
    .rpc(
        "State.Admin.Delete",
        state_rpc(
            "State.Admin.Delete",
            "StateAdminDeleteRequest",
            "StateAdminDeleteResponse",
        )
        .with_call_capabilities(["admin"])
        .docs_with_summary(
            "Admin delete a state value.",
            "Deletes one state value across participants for authorized administrators.",
        ),
    )
    .rpc(
        "State.Admin.Get",
        state_rpc(
            "State.Admin.Get",
            "StateAdminGetRequest",
            "StateAdminGetResponse",
        )
        .with_call_capabilities(["admin"])
        .docs_with_summary(
            "Admin read a state value.",
            "Returns one state value across participants for authorized administrators.",
        ),
    )
    .rpc(
        "State.Admin.List",
        state_rpc(
            "State.Admin.List",
            "StateAdminListRequest",
            "StateAdminListResponse",
        )
        .with_call_capabilities(["admin"])
        .docs_with_summary(
            "Admin list state values.",
            "Lists state values across participants for authorized administrators.",
        ),
    )
    .rpc(
        "State.Delete",
        state_rpc("State.Delete", "StateDeleteRequest", "StateDeleteResponse")
            .docs_with_summary(
                "Delete a state value.",
                "Deletes one state value from the caller's authorized scope.",
            ),
    )
    .rpc(
        "State.Get",
        state_rpc("State.Get", "StateGetRequest", "StateGetResponse").docs_with_summary(
            "Read a state value.",
            "Returns one state value in the caller's authorized scope.",
        ),
    )
    .rpc(
        "State.List",
        state_rpc("State.List", "StateListRequest", "StateListResponse").docs_with_summary(
            "List state values.",
            "Lists state values visible to the caller for the requested scope and prefix.",
        ),
    )
    .rpc(
        "State.Put",
        state_rpc("State.Put", "StatePutRequest", "StatePutResponse").docs_with_summary(
            "Write a state value.",
            "Creates or replaces one state value in an authorized scope.",
        ),
    )
    .build()
}

/// Build the native State participant and API artifacts.
pub fn contract_artifacts() -> Result<ContractArtifacts, ContractsError> {
    let api = api_artifact()?.normalized_value()?;
    ContractBuilder::from_api("trellis.state@v1", api, ContractKind::Service)?.build()
}

fn state_rpc(name: &str, input: &str, output: &str) -> trellis_contracts::ContractRpcMethod {
    trellis_contracts::rpc("v1", format!("rpc.v1.{name}"), input, output).with_error_types([
        AUTH_ERROR,
        UNEXPECTED_ERROR,
        VALIDATION_ERROR,
    ])
}

fn string() -> Value {
    json!({ "type": "string", "minLength": 1 })
}

fn state_entry() -> Value {
    json!({
        "type": "object",
        "required": ["value", "revision", "updatedAt"],
        "properties": {
            "value": {},
            "revision": string(),
            "updatedAt": { "type": "string", "format": "date-time" },
            "expiresAt": { "type": "string", "format": "date-time" },
            "key": string()
        }
    })
}

fn migration_required() -> Value {
    json!({
        "type": "object",
        "required": [
            "migrationRequired", "entry", "stateVersion", "currentStateVersion",
            "writerContractDigest"
        ],
        "properties": {
            "migrationRequired": { "type": "boolean", "const": true },
            "entry": state_entry(),
            "stateVersion": string(),
            "currentStateVersion": string(),
            "writerContractDigest": string()
        }
    })
}

fn get_request() -> Value {
    json!({
        "type": "object",
        "required": ["store"],
        "properties": { "store": string(), "key": string() }
    })
}

fn get_response() -> Value {
    json!({ "anyOf": [
        {
            "type": "object",
            "required": ["found"],
            "properties": { "found": { "type": "boolean", "const": false } }
        },
        {
            "type": "object",
            "required": ["found", "entry"],
            "properties": {
                "found": { "type": "boolean", "const": true },
                "entry": state_entry()
            }
        },
        migration_required()
    ] })
}

fn put_request() -> Value {
    json!({
        "type": "object",
        "required": ["store", "value"],
        "properties": {
            "store": string(),
            "key": string(),
            "value": {},
            "ttlMs": { "type": "integer", "minimum": 1 },
            "expectedRevision": { "anyOf": [string(), { "type": "null" }] }
        }
    })
}

fn put_response() -> Value {
    json!({ "anyOf": [
        {
            "type": "object",
            "required": ["applied", "entry"],
            "properties": {
                "applied": { "type": "boolean", "const": true },
                "entry": state_entry()
            }
        },
        {
            "type": "object",
            "required": ["applied", "found"],
            "properties": {
                "applied": { "type": "boolean", "const": false },
                "found": { "type": "boolean" },
                "entry": { "anyOf": [state_entry(), migration_required()] }
            }
        }
    ] })
}

fn delete_request() -> Value {
    json!({
        "type": "object",
        "required": ["store"],
        "properties": {
            "store": string(),
            "key": string(),
            "expectedRevision": string()
        }
    })
}

fn delete_response() -> Value {
    json!({
        "type": "object",
        "required": ["deleted"],
        "properties": { "deleted": { "type": "boolean" } }
    })
}

fn list_request() -> Value {
    json!({
        "type": "object",
        "required": ["limit", "store"],
        "properties": {
            "store": string(),
            "prefix": string(),
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 0 }
        }
    })
}

fn list_response() -> Value {
    json!({
        "type": "object",
        "required": ["entries", "count", "offset", "limit"],
        "properties": {
            "entries": {
                "type": "array",
                "default": [],
                "items": { "anyOf": [state_entry(), migration_required()] }
            },
            "count": { "type": "integer", "minimum": 0 },
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 0 },
            "nextOffset": { "type": "integer", "minimum": 0 }
        }
    })
}

fn admin_request(operation: &str) -> Value {
    let mut user_properties = serde_json::Map::from_iter([
        (
            "scope".to_owned(),
            json!({ "type": "string", "const": "userApp" }),
        ),
        ("contractId".to_owned(), string()),
        ("contractDigest".to_owned(), string()),
        ("store".to_owned(), string()),
        ("userId".to_owned(), string()),
    ]);
    let mut device_properties = serde_json::Map::from_iter([
        (
            "scope".to_owned(),
            json!({ "type": "string", "const": "deviceApp" }),
        ),
        ("contractId".to_owned(), string()),
        ("contractDigest".to_owned(), string()),
        ("store".to_owned(), string()),
        ("deviceId".to_owned(), string()),
    ]);
    let mut user_required = vec!["scope", "contractId", "contractDigest", "store", "userId"];
    let mut device_required = vec!["scope", "contractId", "contractDigest", "store", "deviceId"];
    match operation {
        "get" => {
            user_properties.insert("key".to_owned(), string());
            device_properties.insert("key".to_owned(), string());
        }
        "delete" => {
            user_properties.insert("key".to_owned(), string());
            user_properties.insert("expectedRevision".to_owned(), string());
            device_properties.insert("key".to_owned(), string());
            device_properties.insert("expectedRevision".to_owned(), string());
        }
        "list" => {
            for properties in [&mut user_properties, &mut device_properties] {
                properties.insert("prefix".to_owned(), string());
                properties.insert(
                    "offset".to_owned(),
                    json!({ "type": "integer", "minimum": 0 }),
                );
                properties.insert(
                    "limit".to_owned(),
                    json!({ "type": "integer", "minimum": 0 }),
                );
            }
            user_required.insert(0, "limit");
            device_required.insert(0, "limit");
        }
        _ => unreachable!("known State admin operation"),
    }
    json!({ "anyOf": [
        { "type": "object", "required": user_required, "properties": user_properties },
        { "type": "object", "required": device_required, "properties": device_properties }
    ] })
}
