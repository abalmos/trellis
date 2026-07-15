use jsonschema::Draft;
use serde_json::Value;

use crate::{identifiers::api_error, ProtocolError, API_SCHEMA_V1_JSON};

const DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

#[cfg(test)]
pub(crate) fn validate_api_meta_schema() -> Result<(), ProtocolError> {
    let schema: Value = serde_json::from_str(API_SCHEMA_V1_JSON)?;
    jsonschema::meta::options()
        .validate(&schema)
        .map_err(|error| api_error(error.instance_path().to_string(), error.to_string()))
}

pub(crate) fn validate_api_structure(value: &Value) -> Result<(), ProtocolError> {
    let schema: Value = serde_json::from_str(API_SCHEMA_V1_JSON)?;
    let validator = jsonschema::options()
        .with_draft(Draft::Draft202012)
        .build(&schema)
        .map_err(|error| api_error(error.instance_path().to_string(), error.to_string()))?;
    if let Some(error) = validator.iter_errors(value).next() {
        return Err(api_error(
            error.instance_path().to_string(),
            error.to_string(),
        ));
    }
    Ok(())
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
