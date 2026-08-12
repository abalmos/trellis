#![allow(missing_docs)]

use serde_json::json;
use std::collections::BTreeMap;
use trellis_contracts::{
    schema_ref, state, store, use_contract, ContractBuilder, ContractCapabilityMetadata,
    ContractKind, ContractStateKind, API_FORMAT_V1,
};

#[test]
fn builder_minimal_manifest_defaults_format_and_validates() {
    let artifacts = ContractBuilder::authoring(
        "example.contract@v1",
        "Example Contract",
        "Example contract description.",
        ContractKind::Service,
    )
    .build()
    .expect("builder should produce a valid minimal manifest");

    let api = artifacts.api_value().unwrap();
    assert_eq!(api["format"], API_FORMAT_V1);
    assert_eq!(api["id"], "example.contract@v1");
}

#[test]
fn builder_does_not_model_runtime_health_transport_as_a_contract_use() {
    let artifacts = ContractBuilder::authoring(
        "example.service@v1",
        "Example Service",
        "Example service description.",
        ContractKind::Service,
    )
    .build()
    .expect("builder should produce a valid service manifest");

    assert!(artifacts.participant_value().unwrap()["uses"].is_null());
}

#[test]
fn builder_does_not_add_baseline_health_to_health_contract_itself() {
    let artifacts = ContractBuilder::authoring(
        "trellis.health@v1",
        "Trellis Health",
        "Expose shared Trellis heartbeat events.",
        ContractKind::Service,
    )
    .build()
    .expect("health contract should build without self-use");

    assert!(artifacts.participant_value().unwrap()["uses"].is_null());
}

#[test]
fn builder_does_not_add_health_contract_use_for_devices() {
    let artifacts = ContractBuilder::authoring(
        "example.device@v1",
        "Example Device",
        "Example device manifest.",
        ContractKind::Device,
    )
    .schema("Preferences", json!({ "type": "object", "properties": {} }))
    .state(
        "preferences",
        state(ContractStateKind::Value, "Preferences"),
    )
    .build()
    .expect("builder should produce a valid device manifest");

    assert!(artifacts.participant_value().unwrap()["uses"].is_null());
}

#[test]
fn builder_preserves_explicit_health_use_without_implicit_publish() {
    let artifacts = ContractBuilder::authoring(
        "example.explicit-health@v1",
        "Example Explicit Health",
        "Example explicit health manifest.",
        ContractKind::Service,
    )
    .use_ref(
        "health",
        use_contract("trellis.health@v1").with_event_subscribe(["Health.StatusChanged"]),
    )
    .referenced_apis(referenced_api(
        "trellis.health@v1",
        "events",
        "Health.StatusChanged",
    ))
    .build()
    .expect("builder should produce a valid service manifest");

    let participant = artifacts.participant_value().unwrap();
    assert!(participant["uses"]["required"]["health"]["events"]["publish"].is_null());
    assert_eq!(
        participant["uses"]["required"]["health"]["events"]["subscribe"],
        json!(["Health.StatusChanged"])
    );
}

#[test]
fn builder_preserves_event_publish_and_subscribe_on_same_use() {
    let artifacts = ContractBuilder::authoring(
        "example.events-agent@v1",
        "Example Events Agent",
        "Example events agent manifest.",
        ContractKind::Agent,
    )
    .use_ref(
        "events",
        use_contract("example.events@v1")
            .with_event_publish(["Example.Changed"])
            .with_event_subscribe(["Example.Changed"]),
    )
    .referenced_apis(referenced_api(
        "example.events@v1",
        "events",
        "Example.Changed",
    ))
    .build()
    .expect("builder should preserve both event permissions");

    let participant = artifacts.participant_value().unwrap();
    let events = &participant["uses"]["required"]["events"]["events"];
    assert_eq!(events["publish"], json!(["Example.Changed"]));
    assert_eq!(events["subscribe"], json!(["Example.Changed"]));
}

#[test]
fn builder_supports_uses_rpc_kv_store_and_job_queue_resources() {
    let artifacts = ContractBuilder::authoring(
        "example.jobs@v1",
        "Example Jobs",
        "Example jobs manifest.",
        ContractKind::Service,
    )
    .schema(
        "JobsQueryRequest",
        json!({ "type": "object", "properties": {} }),
    )
    .schema(
        "JobsQueryResponse",
        json!({
            "type": "object",
            "required": ["ok"],
            "properties": {"ok": {"type": "boolean"}}
        }),
    )
    .schema(
        "CacheState",
        json!({
            "type": "object",
            "required": ["status"],
            "properties": {"status": {"type": "string"}}
        }),
    )
    .use_ref(
        "core",
        use_contract("trellis.core@v1").with_rpc_call(["Core.Info"]),
    )
    .capability(
        "admin.read",
        ContractCapabilityMetadata {
            display_name: "Read jobs".to_string(),
            description: "View jobs.".to_string(),
            consequence: None,
        },
    )
    .rpc(
        "Jobs.Query",
        trellis_contracts::rpc(
            "v1",
            "rpc.v1.Jobs.Query",
            "JobsQueryRequest",
            "JobsQueryResponse",
        )
        .with_call_capabilities(["admin.read", "service"])
        .with_error_types(["UnexpectedError"]),
    )
    .kv_resource(
        "cacheState",
        trellis_contracts::kv("Store projected cache state", "CacheState")
            .required(true)
            .history(1)
            .ttl_ms(0),
    )
    .store_resource("uploads", store("Temporary uploaded files"))
    .job_queue(
        "document-process",
        trellis_contracts::job_queue(
            schema_ref("JobsQueryRequest"),
            Some(schema_ref("JobsQueryResponse")),
        ),
    )
    .referenced_apis(referenced_api("trellis.core@v1", "rpc", "Core.Info"))
    .build()
    .expect("builder should produce a valid manifest");

    let api = artifacts.api_value().unwrap();
    let participant = artifacts.participant_value().unwrap();
    assert!(!participant["uses"]["required"]["core"].is_null());
    assert!(!api["rpc"]["Jobs.Query"].is_null());
    assert!(!api["capabilities"]["example.jobs::admin.read"].is_null());
    assert_eq!(
        api["capabilities"]["example.jobs::admin.read"]["allows"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(!participant["resources"]["kv"]["cacheState"].is_null());
    assert_eq!(
        participant["resources"]["kv"]["cacheState"]["schema"]["schema"],
        "CacheState"
    );
    assert!(!participant["resources"]["store"]["uploads"].is_null());
    assert!(!participant["jobQueues"]["document-process"].is_null());
}

#[test]
fn builder_rejects_local_capabilities_with_contract_namespace_prefix() {
    let error = ContractBuilder::authoring(
        "trellis.core@v1",
        "Trellis Core",
        "Trellis core manifest.",
        ContractKind::Service,
    )
    .schema("Empty", json!({ "type": "object", "properties": {} }))
    .capability(
        "trellis.core.catalog.read",
        ContractCapabilityMetadata {
            display_name: "Read catalog".to_string(),
            description: "Read catalog entries.".to_string(),
            consequence: None,
        },
    )
    .rpc(
        "Core.Info",
        trellis_contracts::rpc("v1", "rpc.v1.Core.Info", "Empty", "Empty")
            .with_call_capabilities(["trellis.core.catalog.read"]),
    )
    .build()
    .expect_err("namespace-prefixed local capability should be rejected");

    assert!(error
        .to_string()
        .contains("must not start with contract namespace prefix 'trellis.core.'"));

    let error = ContractBuilder::authoring(
        "trellis.core@v1",
        "Trellis Core",
        "Trellis core manifest.",
        ContractKind::Service,
    )
    .schema("Empty", json!({ "type": "object", "properties": {} }))
    .capability(
        "core.catalog.read",
        ContractCapabilityMetadata {
            display_name: "Read catalog".to_string(),
            description: "Read catalog entries.".to_string(),
            consequence: None,
        },
    )
    .rpc(
        "Core.Info",
        trellis_contracts::rpc("v1", "rpc.v1.Core.Info", "Empty", "Empty")
            .with_call_capabilities(["core.catalog.read"]),
    )
    .build()
    .expect_err("namespace-leaf-prefixed local capability should be rejected");

    assert!(error
        .to_string()
        .contains("must not start with contract namespace prefix 'core.'"));
}

#[test]
fn builder_supports_contract_local_error_declarations() {
    let artifacts = ContractBuilder::authoring(
        "example.errors@v1",
        "Example Errors",
        "Example error manifest.",
        ContractKind::Service,
    )
    .schema(
        "NotFoundErrorData",
        json!({
            "type": "object",
            "required": ["id", "type", "message", "resource"],
            "properties": {
                "id": { "type": "string" },
                "type": { "type": "string", "const": "NotFoundError" },
                "message": { "type": "string" },
                "resource": { "type": "string" },
                "context": { "type": "object", "patternProperties": { "^.*$": {} } },
                "traceId": { "type": "string" }
            }
        }),
    )
    .error("NotFoundError", "NotFoundError", "NotFoundErrorData")
    .build()
    .expect("builder should produce a valid manifest");

    let api = artifacts.api_value().unwrap();
    assert_eq!(
        api["errors"]["NotFoundError"]["schema"]["schema"],
        "NotFoundErrorData"
    );
}

#[test]
fn builder_supports_store_resources() {
    let artifacts = ContractBuilder::authoring(
        "example.store@v1",
        "Example Store",
        "Example store manifest.",
        ContractKind::Service,
    )
    .store_resource(
        "uploads",
        store("Temporary uploaded files")
            .required(true)
            .ttl_ms(0)
            .max_object_bytes(1_048_576)
            .max_total_bytes(2_097_152),
    )
    .build()
    .expect("builder should produce a valid manifest");

    let participant = artifacts.participant_value().unwrap();
    let uploads = &participant["resources"]["store"]["uploads"];
    assert_eq!(uploads["purpose"], "Temporary uploaded files");
    assert_eq!(uploads["maxObjectBytes"], 1_048_576);
    assert_eq!(uploads["maxTotalBytes"], 2_097_152);
}

#[test]
fn builder_supports_state_stores_exports_and_events() {
    let artifacts = ContractBuilder::authoring(
        "example.device@v1",
        "Example Device",
        "Example device manifest.",
        ContractKind::Device,
    )
    .schema("Preferences", json!({ "type": "object", "properties": {} }))
    .schema("Changed", json!({ "type": "object", "properties": {} }))
    .export_schema("Preferences")
    .state(
        "preferences",
        state(ContractStateKind::Value, "Preferences").state_version("preferences.v1"),
    )
    .event(
        "Preferences.Changed",
        trellis_contracts::event("v1", "events.v1.Preferences.Changed", "Changed"),
    )
    .build()
    .expect("builder should produce a valid state manifest");

    let api = artifacts.api_value().unwrap();
    let participant = artifacts.participant_value().unwrap();
    assert_eq!(api["exports"]["schemas"], json!(["Preferences"]));
    assert_eq!(
        participant["state"]["preferences"]["schema"]["schema"],
        "Preferences"
    );
    assert!(!api["events"]["Preferences.Changed"].is_null());
}

#[test]
fn builder_build_returns_validation_error_for_unknown_state_schema_ref() {
    let error = ContractBuilder::authoring(
        "example.device@v1",
        "Example Device",
        "Example device manifest.",
        ContractKind::Device,
    )
    .state("preferences", state(ContractStateKind::Value, "Missing"))
    .build()
    .expect_err("builder should reuse state schema validation");

    let message = error.to_string();
    assert!(message.contains("state"));
    assert!(message.contains("unknown schema"));
}

#[test]
fn builder_build_returns_validation_error_for_unknown_schema_ref() {
    let error = ContractBuilder::authoring(
        "example.contract@v1",
        "Example Contract",
        "Example contract description.",
        ContractKind::Service,
    )
    .schema("Present", json!({ "type": "object", "properties": {} }))
    .rpc(
        "Example.Call",
        trellis_contracts::rpc("v1", "rpc.v1.Example.Call", "Missing", "Present"),
    )
    .build()
    .expect_err("builder should reuse manifest schema validation");

    assert!(!error.to_string().is_empty());
}

#[test]
fn builder_build_returns_validation_error_for_unknown_kv_schema_ref() {
    let error = ContractBuilder::authoring(
        "example.kv@v1",
        "Example KV",
        "Example kv manifest.",
        ContractKind::Service,
    )
    .kv_resource(
        "cacheState",
        trellis_contracts::kv("Store projected cache state", "MissingState"),
    )
    .build()
    .expect_err("builder should reuse kv schema validation");

    assert!(!error.to_string().is_empty());
}

#[test]
fn builder_supports_owned_and_used_operations() {
    let artifacts = ContractBuilder::authoring(
        "example.operations@v1",
        "Example Operations",
        "Example operations manifest.",
        ContractKind::Service,
    )
    .schema(
        "CaptureRequest",
        json!({ "type": "object", "properties": {} }),
    )
    .schema(
        "CaptureProgress",
        json!({ "type": "object", "properties": {} }),
    )
    .schema(
        "CaptureResult",
        json!({ "type": "object", "properties": {} }),
    )
    .use_ref(
        "billing",
        use_contract("billing@v1").with_operation_call(["Billing.Refund"]),
    )
    .capability(
        "payments.capture",
        trellis_contracts::ContractCapabilityMetadata {
            display_name: "Capture payments".to_string(),
            description: "Start payment capture operations.".to_string(),
            consequence: None,
        },
    )
    .capability(
        "payments.read",
        trellis_contracts::ContractCapabilityMetadata {
            display_name: "Read payments".to_string(),
            description: "Read payment operation status.".to_string(),
            consequence: None,
        },
    )
    .capability(
        "payments.cancel",
        trellis_contracts::ContractCapabilityMetadata {
            display_name: "Cancel payments".to_string(),
            description: "Cancel payment operations.".to_string(),
            consequence: None,
        },
    )
    .capability(
        "payments.control",
        trellis_contracts::ContractCapabilityMetadata {
            display_name: "Control payments".to_string(),
            description: "Submit payment operation control signals.".to_string(),
            consequence: None,
        },
    )
    .operation(
        "Payments.Capture",
        trellis_contracts::operation(
            "v1",
            "operations.v1.Payments.Capture",
            "CaptureRequest",
            Some("CaptureProgress"),
            Some("CaptureResult"),
        )
        .with_call_capabilities(["payments.capture"])
        .with_observe_capabilities(["payments.read"])
        .with_cancel_capabilities(["payments.cancel"])
        .with_control_capabilities(["payments.control"])
        .signal("confirm", "CaptureRequest")
        .cancel(true),
    )
    .referenced_apis(referenced_api("billing@v1", "operations", "Billing.Refund"))
    .build()
    .expect("builder should produce a valid operation manifest");

    let api = artifacts.api_value().unwrap();
    let participant = artifacts.participant_value().unwrap();
    assert!(!participant["uses"]["required"]["billing"].is_null());
    assert!(!api["operations"]["Payments.Capture"].is_null());
    assert_eq!(
        api["capabilities"]["example.operations::payments.control"]["allows"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

fn referenced_api(
    id: &str,
    surface_kind: &str,
    surface_name: &str,
) -> BTreeMap<String, serde_json::Value> {
    let mut api = json!({
        "format": "trellis.api.v1",
        "id": id,
        "displayName": "Referenced API",
        "description": "Exact API evidence for builder tests.",
        "schemas": {
            "Empty": {"type": "object", "properties": {}}
        }
    });
    api[surface_kind] = match surface_kind {
        "rpc" => json!({surface_name: {
            "version": "v1",
            "input": {"schema": "Empty"},
            "output": {"schema": "Empty"}
        }}),
        "operations" => json!({surface_name: {
            "version": "v1",
            "input": {"schema": "Empty"},
            "progress": {"schema": "Empty"},
            "output": {"schema": "Empty"}
        }}),
        "events" => json!({surface_name: {
            "version": "v1",
            "event": {"schema": "Empty"}
        }}),
        _ => unreachable!(),
    };
    [(id.to_owned(), api)].into_iter().collect()
}

#[test]
fn builder_build_returns_validation_error_for_unknown_operation_schema_ref() {
    let error = ContractBuilder::authoring(
        "example.operations@v1",
        "Example Operations",
        "Example operations manifest.",
        ContractKind::Service,
    )
    .schema(
        "CaptureRequest",
        json!({ "type": "object", "properties": {} }),
    )
    .operation(
        "Payments.Capture",
        trellis_contracts::operation(
            "v1",
            "operations.v1.Payments.Capture",
            "CaptureRequest",
            Some("MissingProgress"),
            Some("MissingResult"),
        ),
    )
    .build()
    .expect_err("builder should reuse operation schema validation");

    let message = error.to_string();
    assert!(message.contains("operation"));
    assert!(message.contains("unknown schema"));
}
