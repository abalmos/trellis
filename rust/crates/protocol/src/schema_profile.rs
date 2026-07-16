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

pub(crate) fn validate_wire_schema_additive(
    name: &str,
    schema: &Value,
) -> Result<(), ProtocolError> {
    validate_wire_schema_additive_inner(
        name,
        schema,
        schema,
        "",
        &mut std::collections::BTreeSet::new(),
    )
}

fn validate_wire_schema_additive_inner(
    name: &str,
    root: &Value,
    schema: &Value,
    path: &str,
    refs: &mut std::collections::BTreeSet<String>,
) -> Result<(), ProtocolError> {
    let Value::Object(map) = schema else {
        return Ok(());
    };

    let object_capable = schema_can_validate_object(root, schema, &mut Default::default());

    for keyword in [
        "maxProperties",
        "propertyNames",
        "patternProperties",
        "dependentRequired",
        "dependentSchemas",
        "dependencies",
    ] {
        if object_capable && map.contains_key(keyword) {
            return Err(schema_error(
                name,
                child_path(path, keyword),
                format!("wire schemas that accept objects must not use '{keyword}'"),
            ));
        }
    }
    for keyword in ["additionalProperties", "unevaluatedProperties"] {
        if object_capable
            && map
                .get(keyword)
                .is_some_and(|value| value != &Value::Bool(true))
        {
            return Err(schema_error(
                name,
                child_path(path, keyword),
                format!("wire schemas that accept objects must leave '{keyword}' open"),
            ));
        }
    }
    if object_capable && map.get("const").is_some_and(Value::is_object) {
        return Err(schema_error(
            name,
            child_path(path, "const"),
            "wire schemas that accept objects must not use an object-valued 'const'",
        ));
    }
    if object_capable
        && map
            .get("enum")
            .and_then(Value::as_array)
            .is_some_and(|values| values.iter().any(Value::is_object))
    {
        return Err(schema_error(
            name,
            child_path(path, "enum"),
            "wire schemas that accept objects must not use object values in 'enum'",
        ));
    }
    for keyword in ["not", "if", "then", "else"] {
        if object_capable
            && map.get(keyword).is_some_and(|schema| {
                schema_can_validate_object(root, schema, &mut Default::default())
            })
        {
            return Err(schema_error(
                name,
                child_path(path, keyword),
                format!("wire schemas that accept objects must not use object-capable '{keyword}'"),
            ));
        }
    }

    for keyword in ["$ref", "$dynamicRef"] {
        if let Some(reference) = map.get(keyword).and_then(Value::as_str) {
            if refs.insert(reference.to_owned()) {
                if let Some((referenced, referenced_path)) = resolve_local_schema(root, reference) {
                    validate_wire_schema_additive_inner(
                        name,
                        root,
                        referenced,
                        &referenced_path,
                        refs,
                    )?;
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
            validate_wire_schema_additive_inner(
                name,
                root,
                schema,
                &child_path(path, keyword),
                refs,
            )?;
        }
    }
    for keyword in ["allOf", "anyOf", "oneOf", "prefixItems"] {
        if let Some(Value::Array(schemas)) = map.get(keyword) {
            for (index, schema) in schemas.iter().enumerate() {
                validate_wire_schema_additive_inner(
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
                validate_wire_schema_additive_inner(
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

fn schema_can_validate_object(
    root: &Value,
    schema: &Value,
    refs: &mut std::collections::BTreeSet<String>,
) -> bool {
    let Value::Object(map) = schema else {
        return schema == &Value::Bool(true);
    };
    if map.get("type").is_some_and(|value| match value {
        Value::String(value) => value != "object",
        Value::Array(values) => !values.iter().any(|value| value == "object"),
        _ => false,
    }) || map.get("const").is_some_and(|value| !value.is_object())
        || map
            .get("enum")
            .and_then(Value::as_array)
            .is_some_and(|values| !values.iter().any(Value::is_object))
    {
        return false;
    }
    if map
        .get("allOf")
        .and_then(Value::as_array)
        .is_some_and(|schemas| {
            schemas
                .iter()
                .any(|schema| !schema_can_validate_object(root, schema, &mut refs.clone()))
        })
        || ["anyOf", "oneOf"].into_iter().any(|keyword| {
            map.get(keyword)
                .and_then(Value::as_array)
                .is_some_and(|schemas| {
                    !schemas
                        .iter()
                        .any(|schema| schema_can_validate_object(root, schema, &mut refs.clone()))
                })
        })
    {
        return false;
    }
    for keyword in ["$ref", "$dynamicRef"] {
        if let Some(reference) = map.get(keyword).and_then(Value::as_str) {
            if !refs.insert(reference.to_owned()) {
                continue;
            }
            if let Some((referenced, _)) = resolve_local_schema(root, reference) {
                return schema_can_validate_object(root, referenced, refs);
            }
        }
    }
    true
}

fn resolve_local_schema<'a>(root: &'a Value, reference: &str) -> Option<(&'a Value, String)> {
    let fragment = reference.strip_prefix('#')?;
    if fragment.is_empty() || fragment.starts_with('/') {
        return root
            .pointer(fragment)
            .map(|schema| (schema, fragment.to_owned()));
    }
    find_anchor(root, fragment, "")
}

fn find_anchor<'a>(value: &'a Value, anchor: &str, path: &str) -> Option<(&'a Value, String)> {
    match value {
        Value::Object(map) => {
            if ["$anchor", "$dynamicAnchor"]
                .into_iter()
                .any(|keyword| map.get(keyword).and_then(Value::as_str) == Some(anchor))
            {
                return Some((value, path.to_owned()));
            }
            map.iter()
                .find_map(|(key, value)| find_anchor(value, anchor, &child_path(path, key)))
        }
        Value::Array(values) => values
            .iter()
            .enumerate()
            .find_map(|(index, value)| find_anchor(value, anchor, &format!("{path}/{index}"))),
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

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::*;

    #[test]
    fn additive_wire_profile_rejects_field_sensitive_object_schemas() {
        let cases = [
            (
                json!({"additionalProperties": false}),
                "/additionalProperties",
            ),
            (
                json!({"additionalProperties": {"type": "string"}}),
                "/additionalProperties",
            ),
            (
                json!({"unevaluatedProperties": false}),
                "/unevaluatedProperties",
            ),
            (
                json!({"unevaluatedProperties": {"type": "string"}}),
                "/unevaluatedProperties",
            ),
            (json!({"maxProperties": 1}), "/maxProperties"),
            (json!({"propertyNames": true}), "/propertyNames"),
            (
                json!({"patternProperties": {"^x": true}}),
                "/patternProperties",
            ),
            (
                json!({"dependentRequired": {"x": ["y"]}}),
                "/dependentRequired",
            ),
            (
                json!({"dependentSchemas": {"x": true}}),
                "/dependentSchemas",
            ),
            (json!({"dependencies": {"x": ["y"]}}), "/dependencies"),
            (json!({"not": {"required": ["x"]}}), "/not"),
            (json!({"if": {"required": ["x"]}}), "/if"),
            (json!({"then": {"required": ["x"]}}), "/then"),
            (json!({"else": {"required": ["x"]}}), "/else"),
            (json!({"const": {"x": 1}}), "/const"),
            (json!({"enum": ["ok", {"x": 1}]}), "/enum"),
            (
                json!({"properties": {"a/b~c": {"additionalProperties": false}}}),
                "/properties/a~1b~0c/additionalProperties",
            ),
            (
                json!({"items": {"additionalProperties": false}}),
                "/items/additionalProperties",
            ),
            (
                json!({"prefixItems": [{"additionalProperties": false}]}),
                "/prefixItems/0/additionalProperties",
            ),
            (
                json!({"contains": {"additionalProperties": false}}),
                "/contains/additionalProperties",
            ),
            (
                json!({"allOf": [{"additionalProperties": false}]}),
                "/allOf/0/additionalProperties",
            ),
            (
                json!({"anyOf": [{"type": "string"}, {"additionalProperties": false}]}),
                "/anyOf/1/additionalProperties",
            ),
            (
                json!({"oneOf": [{"type": "string"}, {"additionalProperties": false}]}),
                "/oneOf/1/additionalProperties",
            ),
            (
                json!({"$defs": {"closed": {"$anchor": "closed", "additionalProperties": false}}, "$ref": "#closed"}),
                "/$defs/closed/additionalProperties",
            ),
        ];

        for (schema, expected_path) in cases {
            let error = validate_wire_schema_additive("Payload", &schema).unwrap_err();
            let ProtocolError::SchemaProfile { schema, path, .. } = error else {
                panic!("expected schema profile error")
            };
            assert_eq!(schema, "Payload");
            assert_eq!(path, expected_path);
        }
    }

    #[test]
    fn additive_wire_profile_preserves_scalar_constraints_and_open_objects() {
        for schema in [
            json!({"type": "string", "const": "fixed"}),
            json!({"type": "string", "enum": ["a", "b"]}),
            json!({"type": "object"}),
            json!({"type": "object", "additionalProperties": true}),
            json!({"type": "object", "unevaluatedProperties": true}),
            Value::Bool(true),
        ] {
            validate_wire_schema_additive("Payload", &schema).unwrap();
        }
    }
}
