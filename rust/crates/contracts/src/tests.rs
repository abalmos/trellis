use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};

use crate::{
    canonicalize_json, digest_json, event, feed, job_queue, kv, operation, rpc, schema_ref, state,
    store, use_contract, ContractArtifacts, ContractBuilder, ContractCapabilityMetadata,
    ContractEvent, ContractEventConsumerGroup, ContractEventConsumerOrdering,
    ContractEventConsumerReplay, ContractKind, ContractSchemaRef, ContractStateKind,
    JobKeyConcurrencyDescriptor, JobKeyConcurrencyStalePolicy, JobQueueDepthDescriptor,
    JobQueueWhenFullPolicy,
};

#[test]
fn api_form_authoring_preserves_participant_event_consumers() {
    let participant = ContractBuilder::authoring(
        "example.consumer@v1",
        "Consumer",
        "Consumes an event.",
        ContractKind::Service,
    )
    .schema("Event", serde_json::json!({"type": "object"}))
    .event(
        "Updated",
        ContractEvent {
            version: "v1".to_owned(),
            event: ContractSchemaRef {
                schema: "Event".to_owned(),
            },
            subject: String::new(),
            params: None,
            capabilities: None,
            docs: None,
        },
    )
    .event_consumer(
        "ingest",
        ContractEventConsumerGroup {
            self_events: vec!["Updated".to_owned()],
            ordering: ContractEventConsumerOrdering::Strict,
            max_deliver: Some(2),
            replay: ContractEventConsumerReplay::New,
            uses: Default::default(),
            ack_wait_ms: None,
            backoff_ms: None,
            docs: None,
        },
    )
    .build()
    .expect("build native artifacts")
    .participant_value()
    .expect("normalize participant");
    assert_eq!(participant["eventConsumers"]["ingest"]["maxDeliver"], 2);
    assert_eq!(
        participant["eventConsumers"]["ingest"]["events"]["self"][0],
        "Updated"
    );
}

#[test]
fn canonicalize_matches_shared_conformance_vectors() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../conformance/canonical-json/vectors.json");
    let fixtures: Vec<Value> =
        serde_json::from_str(&fs::read_to_string(fixture_path).unwrap()).unwrap();

    for fixture in fixtures {
        if fixture.get("error").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        let input = fixture.get("input").cloned().unwrap();
        assert_eq!(canonicalize_json(&input).unwrap(), fixture["canonical"]);
        assert_eq!(digest_json(&input).unwrap(), fixture["digest"]);
    }
}

#[test]
fn native_authoring_matches_shared_vectors() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../conformance/native-authoring/vectors.json");
    let fixture: Value = serde_json::from_str(&fs::read_to_string(&fixture_path).unwrap()).unwrap();
    let authored = representative_native_cases();
    let expected = fixture["cases"].as_array().unwrap();
    assert_eq!(expected.len(), authored.len());
    for ((name, artifacts), vector) in authored.iter().zip(expected) {
        assert_eq!(vector["name"], *name);
        let actual = native_case(name, artifacts);
        assert_eq!(&actual, vector, "{name}");
    }
}

fn representative_native_cases() -> Vec<(&'static str, ContractArtifacts)> {
    let dependency = ContractBuilder::authoring(
        "conformance.dependency@v1",
        "Dependency",
        "Conformance dependency.",
        ContractKind::Service,
    )
    .schema("Payload", object_schema())
    .rpc("Dependency.Call", rpc("v1", "", "Payload", "Payload"))
    .event("Dependency.Changed", event("v1", "", "Payload"))
    .build()
    .unwrap();
    let dependency_api = dependency.api_value().unwrap();
    let optional_dependency = ContractBuilder::authoring(
        "conformance.optional-dependency@v1",
        "Optional dependency",
        "Optional conformance dependency.",
        ContractKind::Service,
    )
    .schema("Payload", object_schema())
    .rpc("Optional.Call", rpc("v1", "", "Payload", "Payload"))
    .build()
    .unwrap();
    let optional_dependency_api = optional_dependency.api_value().unwrap();

    let minimal = ContractBuilder::authoring(
        "conformance.minimal-app@v1",
        "Minimal app",
        "Minimal native app.",
        ContractKind::App,
    )
    .build()
    .unwrap();
    let service = ContractBuilder::authoring(
        "conformance.service@v1",
        "Conformance service",
        "Representative native service.",
        ContractKind::Service,
    )
    .schema("Payload", object_schema())
    .schema("OldPayload", object_schema())
    .capability(
        "use",
        ContractCapabilityMetadata {
            display_name: "Use service".into(),
            description: "Use representative surfaces.".into(),
            consequence: Some("Runs work.".into()),
        },
    )
    .rpc(
        "Service.Call",
        rpc("v1", "", "Payload", "Payload")
            .with_error_types(["UnexpectedError"])
            .with_call_capabilities(["use"]),
    )
    .operation(
        "Service.Run",
        operation("v1", "", "Payload", Some("Payload"), Some("Payload"))
            .with_update_schema("Payload")
            .cancel(true)
            .signal("continue", "Payload")
            .with_call_capabilities(["use"])
            .with_observe_capabilities(["use"])
            .with_cancel_capabilities(["use"])
            .with_control_capabilities(["use"])
            .with_transfer(
                "objects",
                "/id",
                None::<String>,
                None::<String>,
                Some(1000),
                Some(1024),
            ),
    )
    .event(
        "Service.Changed",
        event("v1", "", "Payload").with_params(["/id"]),
    )
    .feed("Service.Live", feed("v1", "", "Payload", "Payload"))
    .state(
        "settings",
        state(ContractStateKind::Value, "Payload")
            .state_version("v2")
            .accepted_version("v1", "OldPayload"),
    )
    .job_queue(
        "work",
        job_queue(schema_ref("Payload"), Some(schema_ref("Payload")))
            .key_concurrency(JobKeyConcurrencyDescriptor {
                key: vec!["/id".into()],
                max_active: Some(1),
                heartbeat_interval_ms: Some(1000),
                heartbeat_ttl_ms: Some(3000),
                stale_policy: Some(JobKeyConcurrencyStalePolicy::FailStale),
            })
            .queue_policy(JobQueueDepthDescriptor {
                max_queued_per_key: Some(2),
                when_full: Some(JobQueueWhenFullPolicy::ReplaceOldest),
            }),
    )
    .kv_resource("cache", kv("Conformance cache", "Payload"))
    .store_resource("objects", store("Conformance objects"))
    .use_ref(
        "conformance.dependency@v1",
        use_contract("conformance.dependency@v1")
            .with_rpc_call(["Dependency.Call"])
            .with_event_subscribe(["Dependency.Changed"]),
    )
    .use_ref(
        "trellis.state@v1",
        use_contract("trellis.state@v1").with_rpc_call([
            "State.Delete",
            "State.Get",
            "State.List",
            "State.Put",
        ]),
    )
    .optional_use_ref(
        "conformance.optional-dependency@v1",
        use_contract("conformance.optional-dependency@v1").with_rpc_call(["Optional.Call"]),
    )
    .event_consumer(
        "changes",
        ContractEventConsumerGroup {
            uses: [(
                "conformance.dependency@v1".into(),
                vec!["Dependency.Changed".into()],
            )]
            .into(),
            self_events: vec!["Service.Changed".into()],
            replay: ContractEventConsumerReplay::All,
            ordering: ContractEventConsumerOrdering::Strict,
            ack_wait_ms: Some(1000),
            max_deliver: Some(2),
            backoff_ms: None,
            docs: None,
        },
    )
    .referenced_api("conformance.dependency@v1", dependency_api.clone())
    .referenced_api(
        "conformance.optional-dependency@v1",
        optional_dependency_api,
    )
    .referenced_api(
        "trellis.state@v1",
        serde_json::from_str(include_str!(
            "../../../../generated/protocol/apis/trellis.state@v1.json"
        ))
        .unwrap(),
    )
    .build()
    .unwrap();
    let device = ContractBuilder::authoring(
        "conformance.device@v1",
        "Device",
        "Native device.",
        ContractKind::Device,
    )
    .use_ref(
        "conformance.dependency@v1",
        use_contract("conformance.dependency@v1").with_rpc_call(["Dependency.Call"]),
    )
    .referenced_api("conformance.dependency@v1", dependency_api.clone())
    .build()
    .unwrap();
    let agent = ContractBuilder::authoring(
        "conformance.agent@v1",
        "Agent",
        "Native agent.",
        ContractKind::Agent,
    )
    .use_ref(
        "conformance.dependency@v1",
        use_contract("conformance.dependency@v1").with_event_subscribe(["Dependency.Changed"]),
    )
    .referenced_api("conformance.dependency@v1", dependency_api)
    .build()
    .unwrap();

    vec![
        ("minimal app", minimal),
        ("RPC + declared error", clone_artifacts(&service)),
        (
            "operation progress/update/cancel/signal",
            clone_artifacts(&service),
        ),
        ("events params + feeds", clone_artifacts(&service)),
        ("State acceptedVersions", clone_artifacts(&service)),
        ("Jobs queue/key concurrency", clone_artifacts(&service)),
        ("KV/store resources", clone_artifacts(&service)),
        ("required use", clone_artifacts(&service)),
        ("optional use", clone_artifacts(&service)),
        (
            "self + dependency event consumers",
            clone_artifacts(&service),
        ),
        ("capability allows + consent", clone_artifacts(&service)),
        ("operation transfer", service),
        ("device", device),
        ("agent", agent),
    ]
}

fn object_schema() -> Value {
    json!({"type":"object","required":["id"],"properties":{"id":{"type":"string"}}})
}

fn clone_artifacts(artifacts: &ContractArtifacts) -> ContractArtifacts {
    ContractBuilder::from_native(
        artifacts.api_value().unwrap(),
        artifacts.participant_value().unwrap(),
    )
    .referenced_apis(
        artifacts
            .referenced_apis()
            .iter()
            .map(|(id, api)| (id.clone(), api.normalized_value().unwrap()))
            .collect(),
    )
    .build()
    .unwrap()
}

fn native_case(name: &str, artifacts: &ContractArtifacts) -> Value {
    json!({
        "name": name,
        "api": artifacts.api_value().unwrap(),
        "apiCanonicalJson": artifacts.api().canonical_json().unwrap(),
        "apiDigest": artifacts.api_digest().unwrap(),
        "participant": artifacts.participant_value().unwrap(),
        "participantCanonicalJson": artifacts.participant().canonical_json().unwrap(),
        "participantDigest": artifacts.participant_digest().unwrap(),
        "participantNeeds": artifacts.resolved().needs(),
        "participantNeedsDigest": artifacts.participant_needs_digest().unwrap(),
        "requiredGrants": artifacts.required_grants(),
        "optionalGrants": artifacts.optional_grants(),
    })
}

#[test]
fn native_builder_keeps_protocol_subjects_derived() {
    let api = json!({
        "format": "trellis.api.v1",
        "id": "example.api@v1",
        "displayName": "Example",
        "description": "Example API.",
        "schemas": {
            "Input": {"type": "object"},
            "Output": {"type": "object"}
        },
        "rpc": {
            "Example.Get": {
                "version": "v1",
                "input": {"schema": "Input"},
                "output": {"schema": "Output"}
            }
        }
    });
    let participant = json!({
        "format": "trellis.participant.v1",
        "id": "example.service@v1",
        "displayName": "Example Service",
        "description": "Example service.",
        "kind": "service",
        "implements": {
            "self": {
                "api": "example.api@v1",
                "apiDigest": trellis_protocol::parse_api(&api)
                    .unwrap()
                    .digest()
                    .unwrap()
            }
        }
    });
    let artifacts = ContractBuilder::from_native(api, participant)
        .build()
        .expect("native artifacts should build");
    assert!(
        artifacts.api().normalized_value().unwrap()["rpc"]["Example.Get"]
            .get("subject")
            .is_none()
    );
}
