use std::collections::BTreeMap;

use serde_json::{json, Map, Value};
use trellis_protocol::{
    parse_api_v1, parse_participant_v1, resolve_participant_v1, ApiArtifactV1, GrantSetV1,
};

use crate::{digest_contract_value, parse_manifest, ContractsError};

/// Canonical protocol artifacts compiled from one contract authoring manifest.
#[derive(Clone, Debug)]
pub struct CompiledProtocolArtifacts {
    /// Canonical `trellis.api.v1` artifact owned by the contract.
    pub api: Value,
    /// Canonical `trellis.participant.v1` artifact for the runtime participant.
    pub participant: Value,
    /// Participant artifact digest used at bootstrap and authorization boundaries.
    pub participant_digest: String,
    /// Canonical participant needs digest.
    pub participant_needs_digest: String,
    /// Exact required permission atoms derived from selected API surfaces.
    pub required_grants: GrantSetV1,
    /// Exact optional permission atoms derived from selected API surfaces.
    pub optional_grants: GrantSetV1,
}

/// Compile contract authoring JSON into canonical API and participant artifacts.
///
/// `referenced_apis` is keyed by API ID and must contain every required or optional
/// contract reference selected by the participant.
///
/// # Errors
///
/// Returns an error when the contract, a referenced API, or the compiled protocol
/// artifacts fail validation or contextual resolution.
pub fn compile_protocol_artifacts(
    contract: &Value,
    referenced_apis: &BTreeMap<String, Value>,
) -> Result<CompiledProtocolArtifacts, ContractsError> {
    parse_manifest(contract.clone())?;
    let object = contract
        .as_object()
        .ok_or_else(|| invalid("contract must be an object".to_owned()))?;
    let contract_digest = digest_contract_value(contract)?;
    let api_value = compile_api(object, &contract_digest)?;
    let api = parse_api_v1(&api_value).map_err(|error| invalid(error.to_string()))?;
    let mut apis = BTreeMap::new();
    apis.insert(api.id().to_owned(), api.clone());
    for (id, value) in referenced_apis {
        let referenced = parse_api_v1(value).map_err(|error| invalid(error.to_string()))?;
        if referenced.id() != id {
            return Err(invalid(format!(
                "referenced API map key '{id}' does not match artifact id '{}'",
                referenced.id()
            )));
        }
        apis.insert(id.clone(), referenced);
    }
    let participant_value = compile_participant(object, &api, &apis)?;
    let participant =
        parse_participant_v1(&participant_value).map_err(|error| invalid(error.to_string()))?;
    let resolved =
        resolve_participant_v1(&participant, &apis).map_err(|error| invalid(error.to_string()))?;
    Ok(CompiledProtocolArtifacts {
        api: api
            .normalized_value()
            .map_err(|error| invalid(error.to_string()))?,
        participant: participant
            .normalized_value()
            .map_err(|error| invalid(error.to_string()))?,
        participant_digest: resolved.participant_digest().to_owned(),
        participant_needs_digest: resolved
            .needs()
            .digest()
            .map_err(|error| invalid(error.to_string()))?,
        required_grants: resolved.proposal().required().grant_set().clone(),
        optional_grants: resolved.proposal().optional().grant_set().clone(),
    })
}

fn compile_api(
    contract: &Map<String, Value>,
    contract_digest: &str,
) -> Result<Value, ContractsError> {
    let mut api = Map::new();
    api.insert(
        "format".to_owned(),
        Value::String("trellis.api.v1".to_owned()),
    );
    for field in [
        "id",
        "displayName",
        "description",
        "docs",
        "schemas",
        "exports",
    ] {
        copy(contract, &mut api, field);
    }
    normalize_embedded_schemas(&mut api);
    let schemas = api
        .entry("schemas".to_owned())
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .expect("compiled schemas are an object");
    if schemas.contains_key("TrellisContractArtifactIdentity") {
        return Err(invalid(
            "schema name 'TrellisContractArtifactIdentity' is reserved by the protocol artifact compiler"
                .to_owned(),
        ));
    }
    schemas.insert(
        "TrellisContractArtifactIdentity".to_owned(),
        json!({"type": "string", "const": contract_digest}),
    );
    for section in ["rpc", "operations", "events", "feeds", "state"] {
        if let Some(Value::Object(definitions)) = contract.get(section) {
            let mut lowered = definitions.clone();
            for definition in lowered.values_mut().filter_map(Value::as_object_mut) {
                definition.remove("subject");
                definition.remove("capabilities");
                if let Some(Value::Array(errors)) = definition.get_mut("errors") {
                    for error in errors {
                        if let Some(error_type) = error.get("type").and_then(Value::as_str) {
                            *error = Value::String(error_type.to_owned());
                        }
                    }
                }
            }
            if !lowered.is_empty() {
                api.insert(section.to_owned(), Value::Object(lowered));
            }
        }
    }
    let capabilities = compile_capabilities(contract);
    if !capabilities.is_empty() {
        api.insert("capabilities".to_owned(), Value::Object(capabilities));
    }
    if let Some(Value::Object(definitions)) = contract.get("errors") {
        let mut lowered = definitions.clone();
        for definition in lowered.values_mut().filter_map(Value::as_object_mut) {
            definition.remove("type");
        }
        if !lowered.is_empty() {
            api.insert("errors".to_owned(), Value::Object(lowered));
        }
    }
    let referenced_errors = ["rpc", "operations"]
        .into_iter()
        .filter_map(|section| api.get(section).and_then(Value::as_object))
        .flat_map(|definitions| definitions.values())
        .filter_map(|definition| definition.get("errors").and_then(Value::as_array))
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if !referenced_errors.is_empty() {
        let errors = api
            .entry("errors".to_owned())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .expect("compiled errors are an object");
        for error in referenced_errors {
            errors.entry(error).or_insert_with(|| json!({}));
        }
    }
    Ok(Value::Object(api))
}

fn compile_capabilities(contract: &Map<String, Value>) -> Map<String, Value> {
    let api_id = contract
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let mut allows = BTreeMap::<String, Vec<Value>>::new();
    for (section, directions, surface) in [
        ("rpc", &[("call", "call")][..], "rpc"),
        (
            "operations",
            &[
                ("call", "invoke"),
                ("observe", "observe"),
                ("cancel", "cancel"),
            ][..],
            "operation",
        ),
        (
            "events",
            &[("publish", "publish"), ("subscribe", "subscribe")][..],
            "event",
        ),
        ("feeds", &[("subscribe", "subscribe")][..], "feed"),
    ] {
        let Some(definitions) = contract.get(section).and_then(Value::as_object) else {
            continue;
        };
        let mut names = definitions.keys().collect::<Vec<_>>();
        names.sort();
        for name in names {
            let definition = &definitions[name];
            let capabilities = definition.get("capabilities").and_then(Value::as_object);
            for (direction, action) in directions {
                for capability in capabilities
                    .and_then(|value| value.get(*direction))
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    allows.entry(capability.to_owned()).or_default().push(json!({
                        "action": action,
                        "target": {"kind": "apiSurface", "api": api_id, "surface": surface, "name": name}
                    }));
                }
            }
            if section == "operations" {
                let mut signals = definition
                    .get("signals")
                    .and_then(Value::as_object)
                    .map(|value| value.keys().collect::<Vec<_>>())
                    .unwrap_or_default();
                signals.sort();
                for capability in capabilities
                    .and_then(|value| value.get("control"))
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    for signal in &signals {
                        allows.entry(capability.to_owned()).or_default().push(json!({
                            "action": "control",
                            "target": {"kind": "operationSignal", "api": api_id, "operation": name, "signal": signal}
                        }));
                    }
                }
            }
        }
    }
    allows
        .into_iter()
        .map(|(name, allows)| (name, json!({"allows": allows})))
        .collect()
}

fn compile_participant(
    contract: &Map<String, Value>,
    own_api: &ApiArtifactV1,
    apis: &BTreeMap<String, ApiArtifactV1>,
) -> Result<Value, ContractsError> {
    let mut participant = Map::new();
    participant.insert(
        "format".to_owned(),
        Value::String("trellis.participant.v1".to_owned()),
    );
    for field in [
        "id",
        "displayName",
        "description",
        "docs",
        "kind",
        "schemas",
    ] {
        copy(contract, &mut participant, field);
    }
    normalize_embedded_schemas(&mut participant);
    let api_digest = own_api
        .digest()
        .map_err(|error| invalid(error.to_string()))?;
    if ["rpc", "operations", "events", "feeds", "state"]
        .iter()
        .any(|section| contract.get(*section).is_some_and(nonempty_object))
    {
        let operation_transfers = contract
            .get("operations")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .filter_map(|(name, operation)| {
                let mut transfer = operation.get("transfer")?.as_object()?.clone();
                (transfer.get("direction").and_then(Value::as_str) == Some("send")).then(|| {
                    transfer.remove("direction");
                    (name.clone(), Value::Object(transfer))
                })
            })
            .collect::<Map<_, _>>();
        let mut implementation = json!({"api": own_api.id(), "apiDigest": api_digest});
        if !operation_transfers.is_empty() {
            implementation["operationTransfers"] = Value::Object(operation_transfers);
        }
        participant.insert("implements".to_owned(), json!({"self": implementation}));
    }
    if let Some(Value::Object(groups)) = contract.get("uses") {
        let mut uses = Map::new();
        for group in ["required", "optional"] {
            let Some(Value::Object(references)) = groups.get(group) else {
                continue;
            };
            let mut lowered = Map::new();
            for (alias, reference) in references {
                let reference = reference
                    .as_object()
                    .ok_or_else(|| invalid(format!("uses.{group}.{alias} must be an object")))?;
                let api_id = reference
                    .get("contract")
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid(format!("uses.{group}.{alias}.contract is required")))?;
                let api = apis.get(api_id).ok_or_else(|| {
                    invalid(format!("referenced API artifact '{api_id}' is required"))
                })?;
                let mut used = Map::new();
                used.insert("api".to_owned(), Value::String(api_id.to_owned()));
                used.insert(
                    "apiDigest".to_owned(),
                    Value::String(api.digest().map_err(|error| invalid(error.to_string()))?),
                );
                copy(reference, &mut used, "rpc");
                if let Some(Value::Object(operations)) = reference.get("operations") {
                    let calls = operations.get("call").cloned().unwrap_or_else(|| json!([]));
                    let api_value = api
                        .normalized_value()
                        .map_err(|error| invalid(error.to_string()))?;
                    let cancel = operations
                        .get("cancel")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter(|name| {
                            name.as_str().is_some_and(|name| {
                                api_value["operations"][name]["cancel"].as_bool() == Some(true)
                            })
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    let control = operations
                        .get("control")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(|name| {
                            let signals = api_value["operations"][name]["signals"]
                                .as_object()
                                .map(|signals| {
                                    let mut names = signals.keys().cloned().collect::<Vec<_>>();
                                    names.sort();
                                    names
                                })
                                .unwrap_or_default();
                            (name.to_owned(), json!(signals))
                        })
                        .collect::<Map<_, _>>();
                    used.insert(
                        "operations".to_owned(),
                        json!({
                            "invoke": calls.clone(),
                            "observe": calls,
                            "cancel": cancel,
                            "control": control,
                        }),
                    );
                }
                copy(reference, &mut used, "events");
                copy(reference, &mut used, "feeds");
                lowered.insert(alias.clone(), Value::Object(used));
            }
            if !lowered.is_empty() {
                uses.insert(group.to_owned(), Value::Object(lowered));
            }
        }
        if !uses.is_empty() {
            participant.insert("uses".to_owned(), Value::Object(uses));
        }
    }
    if let Some(resources) = contract.get("resources") {
        participant.insert("resources".to_owned(), resources.clone());
    }
    if let Some(Value::Object(jobs)) = contract.get("jobs") {
        let queues = jobs
            .get("queues")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_else(|| jobs.clone());
        participant.insert("jobQueues".to_owned(), Value::Object(queues));
    }
    if let Some(Value::Object(consumers)) = contract.get("eventConsumers") {
        let consumers = consumers
            .iter()
            .map(|(name, consumer)| {
                let mut consumer = consumer
                    .as_object()
                    .cloned()
                    .ok_or_else(|| invalid(format!("eventConsumers.{name} must be an object")))?;
                let mut events = consumer
                    .remove("uses")
                    .and_then(|value| value.as_object().cloned())
                    .unwrap_or_default();
                if let Some(owned) = consumer.remove("self") {
                    events.insert("self".to_owned(), owned);
                }
                consumer.insert("events".to_owned(), Value::Object(events));
                Ok((name.clone(), Value::Object(consumer)))
            })
            .collect::<Result<Map<_, _>, ContractsError>>()?;
        participant.insert("eventConsumers".to_owned(), Value::Object(consumers));
    }
    Ok(Value::Object(participant))
}

fn copy(source: &Map<String, Value>, target: &mut Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key) {
        if !value.is_null() && !matches!(value, Value::Object(map) if map.is_empty()) {
            target.insert(key.to_owned(), value.clone());
        }
    }
}

fn nonempty_object(value: &Value) -> bool {
    value.as_object().is_some_and(|object| !object.is_empty())
}

fn normalize_embedded_schemas(artifact: &mut Map<String, Value>) {
    if let Some(Value::Object(schemas)) = artifact.get_mut("schemas") {
        for schema in schemas.values_mut() {
            normalize_schema(schema);
        }
    }
}

fn normalize_schema(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                normalize_schema(value);
            }
        }
        Value::Object(object) => {
            if object.remove("patternProperties").is_some() {
                object.insert("additionalProperties".to_owned(), Value::Bool(true));
            }
            for value in object.values_mut() {
                normalize_schema(value);
            }
        }
        _ => {}
    }
}

fn invalid(details: String) -> ContractsError {
    ContractsError::SchemaValidation {
        kind: "compiled protocol artifacts",
        details,
    }
}
