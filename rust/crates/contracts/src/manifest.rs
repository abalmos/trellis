use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::{
    canonicalize_json, sha256_base64url, validate_manifest, ContractManifest, ContractsError,
    LoadedManifest, CONTRACT_FORMAT_V1,
};

/// Load an arbitrary JSON value from disk.
pub fn load_json_value(path: impl AsRef<Path>) -> Result<Value, ContractsError> {
    let path = path.as_ref();
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

/// Parse and validate one contract manifest JSON value.
pub fn parse_manifest(value: Value) -> Result<ContractManifest, ContractsError> {
    reject_unsupported_contract_fields(&value)?;
    validate_manifest(&value)?;
    let manifest: ContractManifest = serde_json::from_value(value)?;
    validate_schema_refs(&manifest)?;
    validate_event_subjects(&manifest)?;
    Ok(manifest)
}

fn validate_event_subjects(manifest: &ContractManifest) -> Result<(), ContractsError> {
    for (name, event) in &manifest.events {
        let pointers = subject_template_pointers(&event.subject).map_err(|details| {
            ContractsError::SchemaValidation {
                kind: "contract",
                details: format!("Event '{name}' {details}"),
            }
        })?;
        if event.params.as_deref().unwrap_or_default() != pointers {
            return Err(ContractsError::SchemaValidation {
                kind: "contract",
                details: format!(
                    "Event '{name}' params must list subject template pointers in order"
                ),
            });
        }

        let schema = manifest.schemas.get(&event.event.schema).ok_or_else(|| {
            ContractsError::SchemaValidation {
                kind: "contract",
                details: format!(
                    "Event '{name}' references unknown schema '{}'",
                    event.event.schema
                ),
            }
        })?;
        for pointer in pointers {
            let segments = pointer[1..]
                .split('/')
                .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
                .collect::<Vec<_>>();
            let mut schemas = Vec::new();
            if !collect_pointer_schemas(schema, &segments, &mut schemas) {
                return Err(ContractsError::SchemaValidation {
                    kind: "contract",
                    details: format!(
                        "Invalid event subject param pointer '{pointer}' for event '{name}' (path not found in schema)"
                    ),
                });
            }
            if schemas.iter().any(|schema| !is_tokenable_schema(schema)) {
                return Err(ContractsError::SchemaValidation {
                    kind: "contract",
                    details: format!(
                        "Invalid event subject param pointer '{pointer}' for event '{name}' (must resolve to string/number schema with safe integer bounds)"
                    ),
                });
            }
        }
    }
    Ok(())
}

fn subject_template_pointers(subject: &str) -> Result<Vec<String>, String> {
    let mut pointers = Vec::new();
    let mut rest = subject;
    while let Some(open) = rest.find('{') {
        if rest[..open].contains('}') {
            return Err("subject has malformed template token".to_string());
        }
        let placeholder = &rest[open + 1..];
        let close = placeholder
            .find('}')
            .ok_or_else(|| "subject has malformed template token".to_string())?;
        let pointer = &placeholder[..close];
        if pointer.contains(['{', '}']) || !is_json_pointer(pointer) {
            return Err(format!(
                "subject template token '{pointer}' must be a JSON Pointer"
            ));
        }
        pointers.push(pointer.to_string());
        rest = &placeholder[close + 1..];
    }
    if rest.contains('}') {
        return Err("subject has malformed template token".to_string());
    }
    Ok(pointers)
}

fn is_json_pointer(pointer: &str) -> bool {
    if !pointer.starts_with('/') {
        return false;
    }
    let bytes = pointer.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'~' {
            index += 1;
            if index == bytes.len() || !matches!(bytes[index], b'0' | b'1') {
                return false;
            }
        }
        index += 1;
    }
    true
}

fn collect_pointer_schemas<'a>(
    schema: &'a Value,
    segments: &[String],
    resolved: &mut Vec<&'a Value>,
) -> bool {
    if segments.is_empty() {
        resolved.push(schema);
        return true;
    }
    let Some(object) = schema.as_object() else {
        return false;
    };

    let mut found = false;
    if let Some(property) = object
        .get("properties")
        .and_then(Value::as_object)
        .and_then(|properties| properties.get(&segments[0]))
    {
        found |= collect_pointer_schemas(property, &segments[1..], resolved);
    }
    if let Some(branches) = object.get("allOf").and_then(Value::as_array) {
        for branch in branches {
            found |= collect_pointer_schemas(branch, segments, resolved);
        }
    }
    for keyword in ["anyOf", "oneOf"] {
        let Some(branches) = object.get(keyword).and_then(Value::as_array) else {
            continue;
        };
        let mut branch_schemas = Vec::new();
        let every_branch = branches
            .iter()
            .all(|branch| collect_pointer_schemas(branch, segments, &mut branch_schemas));
        if every_branch {
            found = true;
        }
        resolved.extend(branch_schemas);
    }
    found
}

fn is_tokenable_schema(schema: &Value) -> bool {
    let Some(object) = schema.as_object() else {
        return false;
    };
    for keyword in ["allOf", "anyOf", "oneOf"] {
        if let Some(branches) = object.get(keyword).and_then(Value::as_array) {
            if !branches.is_empty() {
                return branches.iter().all(is_tokenable_schema);
            }
        }
    }
    if let Some(constant) = object.get("const") {
        return is_safe_token_value(constant);
    }
    if let Some(values) = object.get("enum").and_then(Value::as_array) {
        return !values.is_empty() && values.iter().all(is_safe_token_value);
    }

    match object.get("type") {
        Some(Value::String(kind)) => is_tokenable_type(kind, object),
        Some(Value::Array(kinds)) => {
            !kinds.is_empty()
                && kinds.iter().all(|kind| {
                    kind.as_str()
                        .is_some_and(|kind| is_tokenable_type(kind, object))
                })
        }
        _ => false,
    }
}

fn is_tokenable_type(kind: &str, schema: &serde_json::Map<String, Value>) -> bool {
    match kind {
        "string" | "number" => true,
        "integer" => {
            schema
                .get("minimum")
                .and_then(Value::as_f64)
                .is_some_and(|minimum| minimum >= -9_007_199_254_740_991_f64)
                && schema
                    .get("maximum")
                    .and_then(Value::as_f64)
                    .is_some_and(|maximum| maximum <= 9_007_199_254_740_991_f64)
        }
        _ => false,
    }
}

fn is_safe_token_value(value: &Value) -> bool {
    match value {
        Value::String(_) => true,
        Value::Number(number) if number.is_i64() => number
            .as_i64()
            .is_some_and(|value| value.unsigned_abs() <= 9_007_199_254_740_991),
        Value::Number(number) if number.is_u64() => number
            .as_u64()
            .is_some_and(|value| value <= 9_007_199_254_740_991),
        Value::Number(_) => true,
        _ => false,
    }
}

fn reject_unsupported_contract_fields(value: &Value) -> Result<(), ContractsError> {
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    if object.contains_key("subjects") {
        return Err(ContractsError::SchemaValidation {
            kind: "contract",
            details: "Contract subjects are not supported in v1".to_string(),
        });
    }
    if let Some(resources) = object.get("resources").and_then(Value::as_object) {
        if resources.contains_key("jobs") {
            return Err(ContractsError::SchemaValidation {
                kind: "contract",
                details: "/resources/jobs is not supported in v1".to_string(),
            });
        }
        if resources.contains_key("stream") || resources.contains_key("streams") {
            return Err(ContractsError::SchemaValidation {
                kind: "contract",
                details: "/resources/streams is not supported in v1".to_string(),
            });
        }
    }

    let Some(uses) = object.get("uses").and_then(Value::as_object) else {
        return Ok(());
    };
    for group in ["required", "optional"] {
        let Some(grouped_uses) = uses.get(group).and_then(Value::as_object) else {
            continue;
        };
        for (alias, use_ref) in grouped_uses {
            if use_ref
                .as_object()
                .is_some_and(|use_object| use_object.contains_key("subjects"))
            {
                return Err(ContractsError::SchemaValidation {
                    kind: "contract",
                    details: format!("Contract uses '{alias}' declares unsupported subjects"),
                });
            }
        }
    }
    Ok(())
}

/// Parse untrusted manifest JSON and return the normalized current v1 shape.
pub fn normalize_manifest_value(value: Value) -> Result<Value, ContractsError> {
    let manifest = parse_manifest(value)?;
    Ok(serde_json::to_value(manifest)?)
}

/// Load, validate, canonicalize, and digest one manifest file.
pub fn load_manifest(path: impl AsRef<Path>) -> Result<LoadedManifest, ContractsError> {
    let path = path.as_ref();
    let raw_value = load_json_value(path)?;
    let manifest = parse_manifest(raw_value)?;
    let value = serde_json::to_value(&manifest)?;
    let canonical = canonicalize_json(&value)?;
    let digest = digest_contract_value(&value)?;

    Ok(LoadedManifest {
        path: path.to_path_buf(),
        value,
        manifest,
        canonical,
        digest,
    })
}

fn object(value: Option<&Value>) -> Option<&serde_json::Map<String, Value>> {
    value.and_then(Value::as_object)
}

fn array(value: Option<&Value>) -> Option<&Vec<Value>> {
    value.and_then(Value::as_array)
}

fn schema_ref(value: Option<&Value>) -> Option<String> {
    object(value)
        .and_then(|value| value.get("schema"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn collect_schema_ref(reachable: &mut std::collections::BTreeSet<String>, value: Option<&Value>) {
    if let Some(schema) = schema_ref(value) {
        reachable.insert(schema);
    }
}

fn collect_reachable_schema_names(contract: &Value) -> std::collections::BTreeSet<String> {
    let mut reachable = std::collections::BTreeSet::new();

    for store in object(contract.get("state"))
        .map(|value| value.values())
        .into_iter()
        .flatten()
    {
        collect_schema_ref(
            &mut reachable,
            object(Some(store)).and_then(|value| value.get("schema")),
        );
        for accepted in object(object(Some(store)).and_then(|value| value.get("acceptedVersions")))
            .map(|value| value.values())
            .into_iter()
            .flatten()
        {
            collect_schema_ref(&mut reachable, Some(accepted));
        }
    }

    for method in object(contract.get("rpc"))
        .map(|value| value.values())
        .into_iter()
        .flatten()
    {
        let method = object(Some(method));
        collect_schema_ref(&mut reachable, method.and_then(|value| value.get("input")));
        collect_schema_ref(&mut reachable, method.and_then(|value| value.get("output")));
        for error in array(method.and_then(|value| value.get("errors")))
            .into_iter()
            .flatten()
        {
            let Some(error_type) = object(Some(error))
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            let declaration = object(contract.get("errors"))
                .and_then(|errors| {
                    errors.values().find(|declaration| {
                        object(Some(declaration))
                            .and_then(|value| value.get("type"))
                            .and_then(Value::as_str)
                            == Some(error_type)
                    })
                })
                .and_then(|value| object(Some(value)));
            collect_schema_ref(
                &mut reachable,
                declaration.and_then(|value| value.get("schema")),
            );
        }
    }

    for operation in object(contract.get("operations"))
        .map(|value| value.values())
        .into_iter()
        .flatten()
    {
        let operation = object(Some(operation));
        collect_schema_ref(
            &mut reachable,
            operation.and_then(|value| value.get("input")),
        );
        collect_schema_ref(
            &mut reachable,
            operation.and_then(|value| value.get("update")),
        );
        collect_schema_ref(
            &mut reachable,
            operation.and_then(|value| value.get("progress")),
        );
        collect_schema_ref(
            &mut reachable,
            operation.and_then(|value| value.get("output")),
        );
        for signal in object(operation.and_then(|value| value.get("signals")))
            .map(|value| value.values())
            .into_iter()
            .flatten()
        {
            collect_schema_ref(
                &mut reachable,
                object(Some(signal)).and_then(|value| value.get("input")),
            );
        }
        // NEW: collect error schemas (mirror RPC error collection)
        for error in array(operation.and_then(|value| value.get("errors")))
            .into_iter()
            .flatten()
        {
            let Some(error_type) = object(Some(error))
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            let declaration = object(contract.get("errors"))
                .and_then(|errors| {
                    errors.values().find(|declaration| {
                        object(Some(declaration))
                            .and_then(|value| value.get("type"))
                            .and_then(Value::as_str)
                            == Some(error_type)
                    })
                })
                .and_then(|value| object(Some(value)));
            collect_schema_ref(
                &mut reachable,
                declaration.and_then(|value| value.get("schema")),
            );
        }
    }

    for event in object(contract.get("events"))
        .map(|value| value.values())
        .into_iter()
        .flatten()
    {
        collect_schema_ref(
            &mut reachable,
            object(Some(event)).and_then(|value| value.get("event")),
        );
    }

    for feed in object(contract.get("feeds"))
        .map(|value| value.values())
        .into_iter()
        .flatten()
    {
        let feed = object(Some(feed));
        collect_schema_ref(&mut reachable, feed.and_then(|value| value.get("input")));
        collect_schema_ref(&mut reachable, feed.and_then(|value| value.get("event")));
    }

    for job in object(contract.get("jobs"))
        .map(|value| value.values())
        .into_iter()
        .flatten()
    {
        let job = object(Some(job));
        collect_schema_ref(&mut reachable, job.and_then(|value| value.get("payload")));
        collect_schema_ref(&mut reachable, job.and_then(|value| value.get("update")));
        collect_schema_ref(&mut reachable, job.and_then(|value| value.get("result")));
    }

    for resource in object(contract.get("resources"))
        .and_then(|value| object(value.get("kv")))
        .map(|value| value.values())
        .into_iter()
        .flatten()
    {
        collect_schema_ref(
            &mut reachable,
            object(Some(resource)).and_then(|value| value.get("schema")),
        );
    }

    reachable
}

fn sorted_unique_strings(value: &Value) -> Option<Value> {
    let values = value.as_array()?;
    let mut unique = std::collections::BTreeSet::new();
    for value in values {
        unique.insert(value.as_str()?.to_string());
    }
    Some(Value::Array(
        unique.into_iter().map(Value::String).collect(),
    ))
}

fn insert_sorted_list(
    target: &mut serde_json::Map<String, Value>,
    key: &str,
    source: Option<&Value>,
) {
    if let Some(sorted) = source.and_then(sorted_unique_strings) {
        target.insert(key.to_string(), sorted);
    }
}

fn project_reachable_schemas(contract: &Value) -> Option<Value> {
    let reachable = collect_reachable_schema_names(contract);
    let schemas = object(contract.get("schemas"))?;
    if reachable.is_empty() {
        return None;
    }
    let projected = schemas
        .iter()
        .filter(|(name, _)| reachable.contains(*name))
        .map(|(name, schema)| (name.clone(), schema.clone()))
        .collect::<serde_json::Map<_, _>>();
    (!projected.is_empty()).then_some(Value::Object(projected))
}

fn project_rpc_declared_errors(contract: &Value) -> Option<Value> {
    let errors = object(contract.get("errors"))?;
    let mut declared = std::collections::BTreeSet::new();
    for method in object(contract.get("rpc"))
        .map(|value| value.values())
        .into_iter()
        .flatten()
    {
        for error in array(object(Some(method)).and_then(|value| value.get("errors")))
            .into_iter()
            .flatten()
        {
            if let Some(error_type) = object(Some(error))
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str)
            {
                declared.insert(error_type.to_string());
            }
        }
    }
    let projected = errors
        .iter()
        .filter(|(_, error)| {
            object(Some(error))
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str)
                .is_some_and(|error_type| declared.contains(error_type))
        })
        .map(|(name, error)| (name.clone(), error.clone()))
        .collect::<serde_json::Map<_, _>>();
    (!projected.is_empty()).then_some(Value::Object(projected))
}

fn project_operation_declared_errors(contract: &Value) -> Option<Value> {
    let errors = object(contract.get("errors"))?;
    let mut declared = std::collections::BTreeSet::new();
    for operation in object(contract.get("operations"))
        .map(|value| value.values())
        .into_iter()
        .flatten()
    {
        for error in array(object(Some(operation)).and_then(|value| value.get("errors")))
            .into_iter()
            .flatten()
        {
            if let Some(error_type) = object(Some(error))
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str)
            {
                declared.insert(error_type.to_string());
            }
        }
    }
    let projected = errors
        .iter()
        .filter(|(_, error)| {
            object(Some(error))
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str)
                .is_some_and(|error_type| declared.contains(error_type))
        })
        .map(|(name, error)| (name.clone(), error.clone()))
        .collect::<serde_json::Map<_, _>>();
    (!projected.is_empty()).then_some(Value::Object(projected))
}

fn project_resources(resources: Option<&Value>) -> Option<Value> {
    let resources = object(resources)?;
    let mut projected = serde_json::Map::new();
    if let Some(kv) = resources.get("kv") {
        projected.insert("kv".to_string(), project_map_without_docs(kv));
    }
    if let Some(store) = resources.get("store") {
        projected.insert("store".to_string(), project_map_without_docs(store));
    }
    (!projected.is_empty()).then_some(Value::Object(projected))
}

fn project_map_without_docs(value: &Value) -> Value {
    let Some(entries) = value.as_object() else {
        return value.clone();
    };
    Value::Object(
        entries
            .iter()
            .map(|(name, entry)| {
                let mut entry = entry.clone();
                if let Some(entry) = entry.as_object_mut() {
                    entry.remove("docs");
                }
                (name.clone(), entry)
            })
            .collect(),
    )
}

fn project_use_refs(uses: Option<&Value>) -> Option<Value> {
    let uses = object(uses)?;
    let mut projected_uses = serde_json::Map::new();
    for (alias, use_ref) in uses {
        let Some(use_ref) = use_ref.as_object() else {
            continue;
        };
        let mut projected = serde_json::Map::new();
        if let Some(contract) = use_ref.get("contract") {
            projected.insert("contract".to_string(), contract.clone());
        }
        if let Some(call) = object(use_ref.get("rpc")).and_then(|value| value.get("call")) {
            let mut rpc = serde_json::Map::new();
            insert_sorted_list(&mut rpc, "call", Some(call));
            if !rpc.is_empty() {
                projected.insert("rpc".to_string(), Value::Object(rpc));
            }
        }
        if let Some(call) = object(use_ref.get("operations")).and_then(|value| value.get("call")) {
            let mut operations = serde_json::Map::new();
            insert_sorted_list(&mut operations, "call", Some(call));
            if !operations.is_empty() {
                projected.insert("operations".to_string(), Value::Object(operations));
            }
        }
        let events = object(use_ref.get("events"));
        let mut projected_events = serde_json::Map::new();
        insert_sorted_list(
            &mut projected_events,
            "publish",
            events.and_then(|value| value.get("publish")),
        );
        insert_sorted_list(
            &mut projected_events,
            "subscribe",
            events.and_then(|value| value.get("subscribe")),
        );
        if !projected_events.is_empty() {
            projected.insert("events".to_string(), Value::Object(projected_events));
        }
        let feeds = object(use_ref.get("feeds"));
        let mut projected_feeds = serde_json::Map::new();
        insert_sorted_list(
            &mut projected_feeds,
            "subscribe",
            feeds.and_then(|value| value.get("subscribe")),
        );
        if !projected_feeds.is_empty() {
            projected.insert("feeds".to_string(), Value::Object(projected_feeds));
        }
        projected_uses.insert(alias.clone(), Value::Object(projected));
    }
    Some(Value::Object(projected_uses))
}

fn project_uses(uses: Option<&Value>) -> Option<Value> {
    let uses = object(uses)?;
    let required = project_use_refs(uses.get("required"));
    let optional = omit_required_use_aliases(project_use_refs(uses.get("optional")), &required);

    let mut grouped = serde_json::Map::new();
    insert_if_present(&mut grouped, "required", required);
    insert_if_present(&mut grouped, "optional", optional);
    (!grouped.is_empty()).then_some(Value::Object(grouped))
}

fn omit_required_use_aliases(optional: Option<Value>, required: &Option<Value>) -> Option<Value> {
    let Some(Value::Object(mut optional)) = optional else {
        return optional;
    };
    let Some(Value::Object(required)) = required else {
        return Some(Value::Object(optional));
    };
    for alias in required.keys() {
        optional.remove(alias);
    }
    (!optional.is_empty()).then_some(Value::Object(optional))
}

fn project_capabilities(capabilities: Option<&Value>, keys: &[&str]) -> Option<Value> {
    let capabilities = object(capabilities)?;
    let mut projected = serde_json::Map::new();
    for key in keys {
        insert_sorted_list(&mut projected, key, capabilities.get(*key));
    }
    (!projected.is_empty()).then_some(Value::Object(projected))
}

fn remove_docs(projected: &mut serde_json::Map<String, Value>) {
    projected.remove("docs");
}

fn remove_signal_docs(projected: &mut serde_json::Map<String, Value>) {
    let Some(signals) = projected.get_mut("signals").and_then(Value::as_object_mut) else {
        return;
    };
    for signal in signals.values_mut() {
        if let Some(signal) = signal.as_object_mut() {
            signal.remove("docs");
        }
    }
}

fn project_rpc(rpc: Option<&Value>) -> Option<Value> {
    let rpc = object(rpc)?;
    let mut projected_rpc = serde_json::Map::new();
    for (name, method) in rpc {
        let Some(method_object) = method.as_object() else {
            continue;
        };
        let mut projected = method_object.clone();
        remove_docs(&mut projected);
        if let Some(capabilities) =
            project_capabilities(method_object.get("capabilities"), &["call"])
        {
            projected.insert("capabilities".to_string(), capabilities);
        }
        if let Some(errors) = array(method_object.get("errors")) {
            let sorted = sorted_unique_strings(&Value::Array(
                errors
                    .iter()
                    .filter_map(|error| {
                        object(Some(error))
                            .and_then(|value| value.get("type"))
                            .cloned()
                    })
                    .collect(),
            ));
            if let Some(Value::Array(types)) = sorted {
                projected.insert(
                    "errors".to_string(),
                    Value::Array(
                        types
                            .into_iter()
                            .map(|error_type| {
                                let mut error = serde_json::Map::new();
                                error.insert("type".to_string(), error_type);
                                Value::Object(error)
                            })
                            .collect(),
                    ),
                );
            }
        }
        projected_rpc.insert(name.clone(), Value::Object(projected));
    }
    Some(Value::Object(projected_rpc))
}

fn project_operations(operations: Option<&Value>) -> Option<Value> {
    let operations = object(operations)?;
    let mut projected_operations = serde_json::Map::new();
    for (name, operation) in operations {
        let Some(operation_object) = operation.as_object() else {
            continue;
        };
        let mut projected = operation_object.clone();
        remove_docs(&mut projected);
        remove_signal_docs(&mut projected);
        if let Some(capabilities) = project_capabilities(
            operation_object.get("capabilities"),
            &["call", "observe", "cancel", "control"],
        ) {
            projected.insert("capabilities".to_string(), capabilities);
        }
        if let Some(errors) = array(operation_object.get("errors")) {
            let sorted = sorted_unique_strings(&Value::Array(
                errors
                    .iter()
                    .filter_map(|error| {
                        object(Some(error))
                            .and_then(|value| value.get("type"))
                            .cloned()
                    })
                    .collect(),
            ));
            if let Some(Value::Array(types)) = sorted {
                projected.insert(
                    "errors".to_string(),
                    Value::Array(
                        types
                            .into_iter()
                            .map(|error_type| {
                                let mut error = serde_json::Map::new();
                                error.insert("type".to_string(), error_type);
                                Value::Object(error)
                            })
                            .collect(),
                    ),
                );
            }
        }
        projected_operations.insert(name.clone(), Value::Object(projected));
    }
    Some(Value::Object(projected_operations))
}

fn project_events(events: Option<&Value>) -> Option<Value> {
    let events = object(events)?;
    let mut projected_events = serde_json::Map::new();
    for (name, event) in events {
        let Some(event_object) = event.as_object() else {
            continue;
        };
        let mut projected = event_object.clone();
        remove_docs(&mut projected);
        if let Some(capabilities) =
            project_capabilities(event_object.get("capabilities"), &["publish", "subscribe"])
        {
            projected.insert("capabilities".to_string(), capabilities);
        }
        projected_events.insert(name.clone(), Value::Object(projected));
    }
    Some(Value::Object(projected_events))
}

fn project_feeds(feeds: Option<&Value>) -> Option<Value> {
    let feeds = object(feeds)?;
    let mut projected_feeds = serde_json::Map::new();
    for (name, feed) in feeds {
        let Some(feed_object) = feed.as_object() else {
            continue;
        };
        let mut projected = feed_object.clone();
        remove_docs(&mut projected);
        if let Some(capabilities) =
            project_capabilities(feed_object.get("capabilities"), &["subscribe"])
        {
            projected.insert("capabilities".to_string(), capabilities);
        }
        projected_feeds.insert(name.clone(), Value::Object(projected));
    }
    Some(Value::Object(projected_feeds))
}

fn insert_if_present(target: &mut serde_json::Map<String, Value>, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        target.insert(key.to_string(), value);
    }
}

/// Build the canonical semantic projection used for Trellis contract identity.
///
/// This projection is language-neutral and intentionally differs from the full
/// manifest: display-only metadata and unknown extension fields are excluded,
/// while runtime authority metadata such as top-level capabilities is included.
pub fn project_contract_digest_manifest(contract: &Value) -> Value {
    let mut projected = serde_json::Map::new();
    for key in ["format", "id", "kind"] {
        if let Some(value) = contract.get(key) {
            projected.insert(key.to_string(), value.clone());
        }
    }
    if let Some(capabilities) = contract.get("capabilities") {
        projected.insert("capabilities".to_string(), capabilities.clone());
    }
    insert_if_present(
        &mut projected,
        "schemas",
        project_reachable_schemas(contract),
    );
    if let Some(state) = contract.get("state") {
        projected.insert("state".to_string(), project_map_without_docs(state));
    }
    insert_if_present(&mut projected, "uses", project_uses(contract.get("uses")));
    insert_if_present(&mut projected, "rpc", project_rpc(contract.get("rpc")));
    insert_if_present(
        &mut projected,
        "operations",
        project_operations(contract.get("operations")),
    );
    insert_if_present(
        &mut projected,
        "events",
        project_events(contract.get("events")),
    );
    insert_if_present(
        &mut projected,
        "feeds",
        project_feeds(contract.get("feeds")),
    );
    let rpc_declared = project_rpc_declared_errors(contract);
    let operation_declared = project_operation_declared_errors(contract);
    let merged = match (rpc_declared, operation_declared) {
        (Some(rpc_errors), None) => Some(rpc_errors),
        (None, Some(op_errors)) => Some(op_errors),
        (Some(rpc_errors), Some(op_errors)) => {
            let mut merged = rpc_errors.as_object().cloned().unwrap_or_default();
            if let Some(obj) = op_errors.as_object() {
                merged.extend(obj.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
            (!merged.is_empty()).then_some(Value::Object(merged))
        }
        (None, None) => None,
    };
    insert_if_present(&mut projected, "errors", merged);
    if let Some(jobs) = contract.get("jobs") {
        projected.insert("jobs".to_string(), project_map_without_docs(jobs));
    }
    if let Some(event_consumers) = contract.get("eventConsumers") {
        projected.insert(
            "eventConsumers".to_string(),
            project_map_without_docs(event_consumers),
        );
    }
    insert_if_present(
        &mut projected,
        "resources",
        project_resources(contract.get("resources")),
    );
    Value::Object(projected)
}

/// Compute the v1 contract digest for a JSON manifest value.
pub fn digest_contract_value(contract: &Value) -> Result<String, ContractsError> {
    let normalized = normalize_manifest_value(contract.clone())?;
    Ok(sha256_base64url(&canonicalize_json(
        &project_contract_digest_manifest(&normalized),
    )?))
}

/// Parse and compute the v1 contract digest for a JSON manifest string.
pub fn digest_contract_json(contract_json: &str) -> Result<String, ContractsError> {
    let contract: Value = serde_json::from_str(contract_json)?;
    digest_contract_value(&contract)
}

/// Collect contract manifest candidates from one directory.
pub fn manifest_paths_in_dir(dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, ContractsError> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !(entry.file_type()?.is_file() && is_manifest_candidate_path(&path)) {
            continue;
        }

        let value = load_json_value(&path)?;
        if value
            .get("format")
            .and_then(Value::as_str)
            .is_some_and(|format| format == CONTRACT_FORMAT_V1)
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn is_manifest_candidate_path(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "json")
        && path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| stem.contains('@'))
}

fn validate_schema_refs(manifest: &ContractManifest) -> Result<(), ContractsError> {
    for (name, rpc) in &manifest.rpc {
        assert_schema_ref_exists(manifest, &rpc.input.schema, &format!("rpc '{name}' input"))?;
        assert_schema_ref_exists(
            manifest,
            &rpc.output.schema,
            &format!("rpc '{name}' output"),
        )?;
    }

    for (name, operation) in &manifest.operations {
        assert_schema_ref_exists(
            manifest,
            &operation.input.schema,
            &format!("operation '{name}' input"),
        )?;
        if let Some(update) = &operation.update {
            assert_schema_ref_exists(
                manifest,
                &update.schema,
                &format!("operation '{name}' update"),
            )?;
        }
        if let Some(progress) = &operation.progress {
            assert_schema_ref_exists(
                manifest,
                &progress.schema,
                &format!("operation '{name}' progress"),
            )?;
        }
        if let Some(output) = &operation.output {
            assert_schema_ref_exists(
                manifest,
                &output.schema,
                &format!("operation '{name}' output"),
            )?;
        }
        for (signal_name, signal) in &operation.signals {
            assert_schema_ref_exists(
                manifest,
                &signal.input.schema,
                &format!("operation '{name}' signal '{signal_name}' input"),
            )?;
        }
    }

    for (name, event) in &manifest.events {
        assert_schema_ref_exists(manifest, &event.event.schema, &format!("event '{name}'"))?;
    }

    for (name, feed) in &manifest.feeds {
        assert_schema_ref_exists(
            manifest,
            &feed.input.schema,
            &format!("feed '{name}' input"),
        )?;
        assert_schema_ref_exists(
            manifest,
            &feed.event.schema,
            &format!("feed '{name}' event"),
        )?;
    }

    for (name, state) in &manifest.state {
        assert_schema_ref_exists(manifest, &state.schema.schema, &format!("state '{name}'"))?;
        for (version, schema) in &state.accepted_versions {
            assert_schema_ref_exists(
                manifest,
                &schema.schema,
                &format!("state '{name}' accepted version '{version}'"),
            )?;
        }
    }

    for (name, error) in &manifest.errors {
        if let Some(schema) = &error.schema {
            assert_schema_ref_exists(manifest, &schema.schema, &format!("error '{name}'"))?;
        }
    }

    for (queue_type, queue) in &manifest.jobs {
        assert_schema_ref_exists(
            manifest,
            &queue.payload.schema,
            &format!("jobs queue '{queue_type}' payload"),
        )?;
        if let Some(update) = &queue.update {
            assert_schema_ref_exists(
                manifest,
                &update.schema,
                &format!("jobs queue '{queue_type}' update"),
            )?;
        }
        if let Some(result) = &queue.result {
            assert_schema_ref_exists(
                manifest,
                &result.schema,
                &format!("jobs queue '{queue_type}' result"),
            )?;
        }
        validate_job_key_concurrency(queue_type, &queue.key_concurrency)?;
    }

    validate_event_consumers(manifest)?;

    for (alias, kv) in &manifest.resources.kv {
        assert_schema_ref_exists(
            manifest,
            &kv.schema.schema,
            &format!("resources.kv.{alias}"),
        )?;
    }

    Ok(())
}

fn validate_job_key_concurrency(
    queue_type: &str,
    key_concurrency: &Option<crate::JobKeyConcurrencyDescriptor>,
) -> Result<(), ContractsError> {
    let Some(key_concurrency) = key_concurrency else {
        return Ok(());
    };
    if key_concurrency.key.is_empty() {
        return Err(ContractsError::SchemaValidation {
            kind: "contract",
            details: format!(
                "jobs queue '{queue_type}' keyConcurrency.key must contain at least one segment"
            ),
        });
    }
    for segment in &key_concurrency.key {
        if segment.is_empty() {
            return Err(ContractsError::SchemaValidation {
                kind: "contract",
                details: format!(
                    "jobs queue '{queue_type}' keyConcurrency.key segments must be non-empty strings"
                ),
            });
        }
        if segment.starts_with('/') {
            validate_json_pointer_syntax(queue_type, segment)?;
        }
    }
    if let (Some(interval), Some(ttl)) = (
        key_concurrency.heartbeat_interval_ms,
        key_concurrency.heartbeat_ttl_ms,
    ) {
        if ttl <= interval {
            return Err(ContractsError::SchemaValidation {
                kind: "contract",
                details: format!(
                    "jobs queue '{queue_type}' keyConcurrency.heartbeatTtlMs must exceed heartbeatIntervalMs"
                ),
            });
        }
    }
    Ok(())
}

fn validate_json_pointer_syntax(queue_type: &str, pointer: &str) -> Result<(), ContractsError> {
    let bytes = pointer.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'~' {
            let escaped = bytes.get(index + 1).copied();
            if !matches!(escaped, Some(b'0' | b'1')) {
                return Err(ContractsError::SchemaValidation {
                    kind: "contract",
                    details: format!(
                        "jobs queue '{queue_type}' keyConcurrency.key pointer segment '{pointer}' has invalid JSON Pointer escape at offset {index}; use '~0' for '~' and '~1' for '/'"
                    ),
                });
            }
            index += 1;
        }
        index += 1;
    }
    Ok(())
}

fn validate_event_consumers(manifest: &ContractManifest) -> Result<(), ContractsError> {
    for (group_name, group) in &manifest.event_consumers {
        if group.uses.values().all(Vec::is_empty) && group.self_events.is_empty() {
            return Err(ContractsError::SchemaValidation {
                kind: "contract",
                details: format!(
                    "eventConsumers.{group_name}: must declare at least one dependency or self event"
                ),
            });
        }
        for (use_alias, events) in &group.uses {
            if events.is_empty() {
                return Err(ContractsError::SchemaValidation {
                    kind: "contract",
                    details: format!(
                        "eventConsumers.{group_name}: use alias '{use_alias}' must declare events"
                    ),
                });
            }
            let Some(use_ref) = manifest.uses.get(use_alias) else {
                return Err(ContractsError::SchemaValidation {
                    kind: "contract",
                    details: format!(
                        "eventConsumers.{group_name}: unknown use alias '{use_alias}'"
                    ),
                });
            };
            for event in events {
                if !use_ref
                    .events
                    .as_ref()
                    .and_then(|events| events.subscribe.as_ref())
                    .is_some_and(|events| events.iter().any(|name| name == event))
                {
                    return Err(ContractsError::SchemaValidation {
                        kind: "contract",
                        details: format!(
                            "eventConsumers.{group_name}: event '{event}' is not subscribed through use alias '{use_alias}'"
                        ),
                    });
                }
            }
        }
        for event in &group.self_events {
            if !manifest.events.contains_key(event) {
                return Err(ContractsError::SchemaValidation {
                    kind: "contract",
                    details: format!("eventConsumers.{group_name}: unknown owned event '{event}'"),
                });
            }
        }
    }
    Ok(())
}

fn assert_schema_ref_exists(
    manifest: &ContractManifest,
    schema_name: &str,
    context: &str,
) -> Result<(), ContractsError> {
    if manifest.schemas.contains_key(schema_name) {
        Ok(())
    } else {
        Err(ContractsError::SchemaValidation {
            kind: "contract",
            details: format!("{context}: unknown schema '{schema_name}'"),
        })
    }
}
