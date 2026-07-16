use jsonschema::Draft;
use serde_json::Value;

use crate::{
    identifiers::{api_error, participant_error},
    ProtocolError, API_AUTHORING_SCHEMA_V1_JSON, PARTICIPANT_AUTHORING_SCHEMA_V1_JSON,
};

const DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

#[cfg(test)]
pub(crate) fn validate_api_meta_schema() -> Result<(), ProtocolError> {
    let schema: Value = serde_json::from_str(API_AUTHORING_SCHEMA_V1_JSON)?;
    jsonschema::meta::options()
        .validate(&schema)
        .map_err(|error| api_error(error.instance_path().to_string(), error.to_string()))
}

pub(crate) fn lint_api_authoring(value: &Value) -> Result<(), ProtocolError> {
    validate_structure(API_AUTHORING_SCHEMA_V1_JSON, value, api_error)
}

#[cfg(test)]
pub(crate) fn validate_participant_meta_schema() -> Result<(), ProtocolError> {
    let schema: Value = serde_json::from_str(PARTICIPANT_AUTHORING_SCHEMA_V1_JSON)?;
    jsonschema::meta::options()
        .validate(&schema)
        .map_err(|error| participant_error(error.instance_path().to_string(), error.to_string()))
}

pub(crate) fn lint_participant_authoring(value: &Value) -> Result<(), ProtocolError> {
    validate_structure(
        PARTICIPANT_AUTHORING_SCHEMA_V1_JSON,
        value,
        participant_error,
    )
}

pub(crate) fn validate_api_runtime_structure(value: &Value) -> Result<(), ProtocolError> {
    validate_runtime_structure(API_AUTHORING_SCHEMA_V1_JSON, value, api_error)
}

pub(crate) fn validate_participant_runtime_structure(value: &Value) -> Result<(), ProtocolError> {
    validate_runtime_structure(
        PARTICIPANT_AUTHORING_SCHEMA_V1_JSON,
        value,
        participant_error,
    )
}

fn validate_runtime_structure(
    schema_json: &str,
    value: &Value,
    error: fn(String, String) -> ProtocolError,
) -> Result<(), ProtocolError> {
    let mut schema: Value = serde_json::from_str(schema_json)?;
    open_object_schemas(&mut schema);
    validate_structure_value(&schema, value, error)
}

fn open_object_schemas(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if map.get("additionalProperties") == Some(&Value::Bool(false)) {
                map.insert("additionalProperties".to_owned(), Value::Bool(true));
            }
            for value in map.values_mut() {
                open_object_schemas(value);
            }
        }
        Value::Array(values) => values.iter_mut().for_each(open_object_schemas),
        _ => {}
    }
}

fn validate_structure(
    schema_json: &str,
    value: &Value,
    error: fn(String, String) -> ProtocolError,
) -> Result<(), ProtocolError> {
    let schema: Value = serde_json::from_str(schema_json)?;
    validate_structure_value(&schema, value, error)
}

fn validate_structure_value(
    schema: &Value,
    value: &Value,
    error: fn(String, String) -> ProtocolError,
) -> Result<(), ProtocolError> {
    let validator = jsonschema::options()
        .with_draft(Draft::Draft202012)
        .build(schema)
        .map_err(|validation| {
            error(
                validation.instance_path().to_string(),
                validation.to_string(),
            )
        })?;
    if let Some(validation) = validator.iter_errors(value).next() {
        return Err(error(
            validation.instance_path().to_string(),
            validation.to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_public_schema(name: &str, schema: &Value) -> Result<(), ProtocolError> {
    validate_public_schema_inner(
        name,
        schema,
        schema,
        "",
        &mut std::collections::BTreeSet::new(),
    )
}

fn validate_public_schema_inner(
    name: &str,
    root: &Value,
    schema: &Value,
    path: &str,
    refs: &mut std::collections::BTreeSet<String>,
) -> Result<(), ProtocolError> {
    let Value::Object(map) = schema else {
        return Ok(());
    };

    for keyword in ["maxProperties", "patternProperties", "propertyNames"] {
        if map.contains_key(keyword) {
            return Err(schema_error(
                name,
                child_path(path, keyword),
                format!("public schemas must not use '{keyword}'"),
            ));
        }
    }
    for keyword in ["additionalProperties", "unevaluatedProperties"] {
        if map
            .get(keyword)
            .is_some_and(|value| value != &Value::Bool(true))
        {
            return Err(schema_error(
                name,
                child_path(path, keyword),
                format!("public schemas must leave '{keyword}' open"),
            ));
        }
    }

    for keyword in ["$ref", "$dynamicRef"] {
        if let Some(reference) = map.get(keyword).and_then(Value::as_str) {
            if refs.insert(reference.to_owned()) {
                if let Some(referenced) = resolve_local_schema(root, reference) {
                    validate_public_schema_inner(name, root, referenced, reference, refs)?;
                }
            }
        }
    }
    for keyword in [
        "contains",
        "contentSchema",
        "else",
        "if",
        "items",
        "not",
        "then",
        "unevaluatedItems",
    ] {
        if let Some(schema) = map.get(keyword) {
            validate_public_schema_inner(name, root, schema, &child_path(path, keyword), refs)?;
        }
    }
    for keyword in ["allOf", "anyOf", "oneOf", "prefixItems"] {
        if let Some(Value::Array(schemas)) = map.get(keyword) {
            for (index, schema) in schemas.iter().enumerate() {
                validate_public_schema_inner(
                    name,
                    root,
                    schema,
                    &format!("{}/{index}", child_path(path, keyword)),
                    refs,
                )?;
            }
        }
    }
    for keyword in ["dependentSchemas", "properties"] {
        if let Some(Value::Object(schemas)) = map.get(keyword) {
            for (key, schema) in schemas {
                validate_public_schema_inner(
                    name,
                    root,
                    schema,
                    &child_path(&child_path(path, keyword), key),
                    refs,
                )?;
            }
        }
    }
    Ok(())
}

fn resolve_local_schema<'a>(root: &'a Value, reference: &str) -> Option<&'a Value> {
    let fragment = reference.strip_prefix('#')?;
    if fragment.is_empty() || fragment.starts_with('/') {
        return root.pointer(fragment);
    }
    find_anchor(root, fragment)
}

fn find_anchor<'a>(value: &'a Value, anchor: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => {
            if ["$anchor", "$dynamicAnchor"]
                .into_iter()
                .any(|keyword| map.get(keyword).and_then(Value::as_str) == Some(anchor))
            {
                return Some(value);
            }
            map.values().find_map(|value| find_anchor(value, anchor))
        }
        Value::Array(values) => values.iter().find_map(|value| find_anchor(value, anchor)),
        _ => None,
    }
}

pub(crate) fn validate_embedded_schema(name: &str, schema: &Value) -> Result<(), ProtocolError> {
    validate_profile_keywords(name, schema, "")?;
    jsonschema::meta::options()
        .validate(schema)
        .map_err(|error| {
            schema_error(name, error.instance_path().to_string(), error.to_string())
        })?;
    jsonschema::options()
        .with_draft(Draft::Draft202012)
        .build(schema)
        .map_err(|error| {
            schema_error(name, error.instance_path().to_string(), error.to_string())
        })?;
    Ok(())
}

fn validate_profile_keywords(name: &str, value: &Value, path: &str) -> Result<(), ProtocolError> {
    let Value::Object(map) = value else {
        return Ok(());
    };

    if map.contains_key("$id") {
        return Err(schema_error(
            name,
            child_path(path, "$id"),
            "embedded schemas must not declare '$id'",
        ));
    }
    if let Some(dialect) = map.get("$schema") {
        if dialect.as_str() != Some(DRAFT_2020_12) {
            return Err(schema_error(
                name,
                child_path(path, "$schema"),
                "'$schema' must identify Draft 2020-12",
            ));
        }
    }
    for keyword in ["$ref", "$dynamicRef"] {
        if let Some(reference) = map.get(keyword) {
            if !reference
                .as_str()
                .is_some_and(|reference| reference.starts_with('#'))
            {
                return Err(schema_error(
                    name,
                    child_path(path, keyword),
                    format!("'{keyword}' must be a local fragment beginning with '#'"),
                ));
            }
        }
    }

    for keyword in [
        "additionalProperties",
        "contains",
        "contentSchema",
        "else",
        "if",
        "items",
        "not",
        "propertyNames",
        "then",
        "unevaluatedItems",
        "unevaluatedProperties",
    ] {
        if let Some(schema) = map.get(keyword) {
            validate_profile_keywords(name, schema, &child_path(path, keyword))?;
        }
    }
    for keyword in ["allOf", "anyOf", "oneOf", "prefixItems"] {
        if let Some(Value::Array(schemas)) = map.get(keyword) {
            for (index, schema) in schemas.iter().enumerate() {
                validate_profile_keywords(
                    name,
                    schema,
                    &format!("{}/{index}", child_path(path, keyword)),
                )?;
            }
        }
    }
    for keyword in [
        "$defs",
        "dependentSchemas",
        "patternProperties",
        "properties",
    ] {
        if let Some(Value::Object(schemas)) = map.get(keyword) {
            for (key, schema) in schemas {
                validate_profile_keywords(
                    name,
                    schema,
                    &child_path(&child_path(path, keyword), key),
                )?;
            }
        }
    }
    Ok(())
}

fn child_path(parent: &str, key: &str) -> String {
    format!("{parent}/{}", key.replace('~', "~0").replace('/', "~1"))
}

fn schema_error(
    schema: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ProtocolError {
    ProtocolError::SchemaProfile {
        schema: schema.into(),
        path: path.into(),
        message: message.into(),
    }
}
