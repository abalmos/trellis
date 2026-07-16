use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::{canonicalize_json, ProtocolError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SchemaRelation {
    Subset,
    Incompatible,
    Unknown,
}

pub(super) fn prove_schema_subset(
    sub_schema: &Value,
    super_schema: &Value,
) -> Result<SchemaRelation, ProtocolError> {
    if canonicalize_json(sub_schema)? == canonicalize_json(super_schema)? {
        return Ok(SchemaRelation::Subset);
    }

    compare(
        sub_schema,
        sub_schema,
        super_schema,
        super_schema,
        &mut BTreeSet::new(),
        &mut BTreeSet::new(),
    )
}

pub(super) fn prove_schema_equivalent(
    left: &Value,
    right: &Value,
) -> Result<SchemaRelation, ProtocolError> {
    if canonicalize_json(left)? == canonicalize_json(right)? {
        return Ok(SchemaRelation::Subset);
    }
    let left = normalize_for_equivalence(left, left, &mut BTreeSet::new());
    let right = normalize_for_equivalence(right, right, &mut BTreeSet::new());
    match (left, right) {
        (Ok(left), Ok(right)) => Ok(if canonicalize_json(&left)? == canonicalize_json(&right)? {
            SchemaRelation::Subset
        } else {
            SchemaRelation::Incompatible
        }),
        _ => Ok(SchemaRelation::Unknown),
    }
}

fn compare(
    sub_root: &Value,
    sub_schema: &Value,
    super_root: &Value,
    super_schema: &Value,
    sub_refs: &mut BTreeSet<String>,
    super_refs: &mut BTreeSet<String>,
) -> Result<SchemaRelation, ProtocolError> {
    if canonicalize_json(sub_schema)? == canonicalize_json(super_schema)? {
        return Ok(SchemaRelation::Subset);
    }

    let sub_schema = match resolve_local_ref(sub_root, sub_schema, sub_refs) {
        Ok(schema) => schema,
        Err(relation) => return Ok(relation),
    };
    let super_schema = match resolve_local_ref(super_root, super_schema, super_refs) {
        Ok(schema) => schema,
        Err(relation) => return Ok(relation),
    };

    if sub_schema == &Value::Bool(false) || is_unconstrained(super_schema) {
        return Ok(SchemaRelation::Subset);
    }
    if super_schema == &Value::Bool(false) {
        return Ok(SchemaRelation::Incompatible);
    }
    if has_unsupported_schema(sub_schema) || has_unsupported_schema(super_schema) {
        return Ok(SchemaRelation::Unknown);
    }
    if is_unconstrained(sub_schema) {
        return Ok(SchemaRelation::Incompatible);
    }

    match (sub_schema, super_schema) {
        (Value::Bool(true), Value::Object(_)) => Ok(SchemaRelation::Incompatible),
        (Value::Object(sub), Value::Object(super_)) => {
            compare_objects(sub_root, sub, super_root, super_, sub_refs, super_refs)
        }
        _ => Ok(SchemaRelation::Unknown),
    }
}

fn compare_objects(
    sub_root: &Value,
    sub: &Map<String, Value>,
    super_root: &Value,
    super_: &Map<String, Value>,
    sub_refs: &mut BTreeSet<String>,
    super_refs: &mut BTreeSet<String>,
) -> Result<SchemaRelation, ProtocolError> {
    if has_unsupported_keyword(sub) || has_unsupported_keyword(super_) {
        return Ok(SchemaRelation::Unknown);
    }

    let mut relation = compare_types(sub, super_);
    relation = relation.and(compare_values(sub, super_)?);

    let sub_types = type_set(sub.get("type"));
    if sub_types
        .as_ref()
        .is_none_or(|types| types.contains("object"))
    {
        relation = relation.and(compare_object_validation(
            sub_root, sub, super_root, super_, sub_refs, super_refs,
        )?);
    }
    Ok(relation)
}

fn compare_types(sub: &Map<String, Value>, super_: &Map<String, Value>) -> SchemaRelation {
    let Some(super_types) = type_set(super_.get("type")) else {
        return SchemaRelation::Subset;
    };
    let Some(sub_types) = type_set(sub.get("type")) else {
        return if explicit_value_refs(sub).is_some_and(|values| {
            values
                .into_iter()
                .all(|value| value_matches_types(value, &super_types))
        }) {
            SchemaRelation::Subset
        } else {
            SchemaRelation::Incompatible
        };
    };

    if sub_types.iter().all(|sub_type| {
        super_types.contains(sub_type) || (*sub_type == "integer" && super_types.contains("number"))
    }) {
        SchemaRelation::Subset
    } else {
        SchemaRelation::Incompatible
    }
}

fn value_matches_types(value: &Value, types: &BTreeSet<&str>) -> bool {
    let value_type = match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Object(_) => "object",
        Value::Array(_) => "array",
        Value::String(_) => "string",
        Value::Number(number) if number.as_f64().is_some_and(|value| value.fract() == 0.0) => {
            "integer"
        }
        Value::Number(_) => "number",
    };
    types.contains(value_type) || (value_type == "integer" && types.contains("number"))
}

fn explicit_value_refs(schema: &Map<String, Value>) -> Option<Vec<&Value>> {
    match (
        schema.get("const"),
        schema.get("enum").and_then(Value::as_array),
    ) {
        (Some(value), Some(values)) => Some(values.iter().filter(|item| *item == value).collect()),
        (Some(value), None) => Some(vec![value]),
        (None, Some(values)) => Some(values.iter().collect()),
        (None, None) => None,
    }
}

fn type_set(value: Option<&Value>) -> Option<BTreeSet<&str>> {
    match value {
        None => None,
        Some(Value::String(value)) => Some(BTreeSet::from([value.as_str()])),
        Some(Value::Array(values)) => Some(values.iter().filter_map(Value::as_str).collect()),
        Some(_) => Some(BTreeSet::new()),
    }
}

fn compare_values(
    sub: &Map<String, Value>,
    super_: &Map<String, Value>,
) -> Result<SchemaRelation, ProtocolError> {
    let sub_values = explicit_values(sub)?;
    let super_values = explicit_values(super_)?;
    match (sub_values, super_values) {
        (_, None) => Ok(SchemaRelation::Subset),
        (None, Some(_)) => Ok(SchemaRelation::Unknown),
        (Some(sub), Some(_)) if sub.is_empty() => Ok(SchemaRelation::Unknown),
        (Some(sub), Some(super_)) => Ok(if sub.is_subset(&super_) {
            SchemaRelation::Subset
        } else {
            SchemaRelation::Incompatible
        }),
    }
}

fn explicit_values(schema: &Map<String, Value>) -> Result<Option<BTreeSet<String>>, ProtocolError> {
    let constant = schema.get("const").map(canonicalize_json).transpose()?;
    let enumeration = schema
        .get("enum")
        .and_then(Value::as_array)
        .map(|values| values.iter().map(canonicalize_json).collect())
        .transpose()?;
    Ok(match (constant, enumeration) {
        (Some(constant), Some(enumeration)) => Some(
            BTreeSet::from([constant])
                .intersection(&enumeration)
                .cloned()
                .collect(),
        ),
        (Some(constant), None) => Some(BTreeSet::from([constant])),
        (None, enumeration) => enumeration,
    })
}

fn compare_object_validation(
    sub_root: &Value,
    sub: &Map<String, Value>,
    super_root: &Value,
    super_: &Map<String, Value>,
    sub_refs: &mut BTreeSet<String>,
    super_refs: &mut BTreeSet<String>,
) -> Result<SchemaRelation, ProtocolError> {
    let sub_required = string_set(sub.get("required"));
    let super_required = string_set(super_.get("required"));
    if !super_required.is_subset(&sub_required) {
        return Ok(SchemaRelation::Incompatible);
    }

    let empty_sub_properties = Map::new();
    let empty_super_properties = Map::new();
    let sub_properties = properties(sub).unwrap_or(&empty_sub_properties);
    let super_properties = properties(super_).unwrap_or(&empty_super_properties);
    let sub_additional = additional_properties(sub);
    let super_additional = additional_properties(super_);

    let mut relation = SchemaRelation::Subset;
    let property_names = sub_properties
        .keys()
        .chain(super_properties.keys())
        .collect::<BTreeSet<_>>();
    for name in property_names {
        let sub_property = sub_properties.get(name).unwrap_or(&sub_additional);
        let super_property = super_properties.get(name).unwrap_or(&super_additional);
        relation = relation.and(compare(
            sub_root,
            sub_property,
            super_root,
            super_property,
            &mut sub_refs.clone(),
            &mut super_refs.clone(),
        )?);
    }
    relation = relation.and(compare(
        sub_root,
        &sub_additional,
        super_root,
        &super_additional,
        &mut sub_refs.clone(),
        &mut super_refs.clone(),
    )?);
    Ok(relation)
}

fn properties(schema: &Map<String, Value>) -> Option<&Map<String, Value>> {
    schema.get("properties").and_then(Value::as_object)
}

fn string_set(value: Option<&Value>) -> BTreeSet<&str> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
}

fn additional_properties(schema: &Map<String, Value>) -> Value {
    schema
        .get("additionalProperties")
        .cloned()
        .unwrap_or(Value::Bool(true))
}

fn resolve_local_ref<'a>(
    root: &'a Value,
    schema: &'a Value,
    seen: &mut BTreeSet<String>,
) -> Result<&'a Value, SchemaRelation> {
    let mut schema = schema;
    loop {
        let Some(object) = schema.as_object() else {
            return Ok(schema);
        };
        let Some(reference) = object.get("$ref").and_then(Value::as_str) else {
            return Ok(schema);
        };
        if object.keys().any(|key| !ref_sibling_allowed(key)) || !seen.insert(reference.to_owned())
        {
            return Err(SchemaRelation::Unknown);
        }
        let pointer = reference.strip_prefix('#').ok_or(SchemaRelation::Unknown)?;
        jsonptr::Pointer::parse(pointer).map_err(|_| SchemaRelation::Unknown)?;
        schema = root.pointer(pointer).ok_or(SchemaRelation::Unknown)?;
    }
}

fn normalize_for_equivalence(
    root: &Value,
    schema: &Value,
    seen: &mut BTreeSet<String>,
) -> Result<Value, SchemaRelation> {
    let schema = resolve_local_ref(root, schema, seen)?;
    let Some(object) = schema.as_object() else {
        return Ok(schema.clone());
    };
    if has_unsupported_keyword(object)
        || object
            .get("additionalProperties")
            .is_some_and(|value| !value.is_boolean())
    {
        return Err(SchemaRelation::Unknown);
    }

    let mut normalized = Map::new();
    for (key, value) in object {
        if is_annotation(key) || matches!(key.as_str(), "$schema" | "$defs" | "$ref") {
            continue;
        }
        let value = if key == "properties" {
            Value::Object(
                value
                    .as_object()
                    .ok_or(SchemaRelation::Unknown)?
                    .iter()
                    .map(|(name, schema)| {
                        Ok((
                            name.clone(),
                            normalize_for_equivalence(root, schema, &mut seen.clone())?,
                        ))
                    })
                    .collect::<Result<_, SchemaRelation>>()?,
            )
        } else {
            value.clone()
        };
        normalized.insert(key.clone(), value);
    }
    Ok(Value::Object(normalized))
}

fn ref_sibling_allowed(key: &str) -> bool {
    key == "$ref" || key == "$defs" || key == "$schema" || is_annotation(key)
}

fn has_unsupported_keyword(schema: &Map<String, Value>) -> bool {
    schema
        .keys()
        .any(|key| !is_supported_keyword(key) && !is_annotation(key))
}

fn has_unsupported_schema(schema: &Value) -> bool {
    schema.as_object().is_some_and(has_unsupported_keyword)
}

fn is_supported_keyword(key: &str) -> bool {
    matches!(
        key,
        "$schema"
            | "$defs"
            | "$ref"
            | "type"
            | "const"
            | "enum"
            | "properties"
            | "required"
            | "additionalProperties"
    )
}

fn is_unconstrained(schema: &Value) -> bool {
    match schema {
        Value::Bool(true) => true,
        Value::Object(schema) => schema
            .keys()
            .all(|key| matches!(key.as_str(), "$schema" | "$defs") || is_annotation(key)),
        _ => false,
    }
}

fn is_annotation(key: &str) -> bool {
    matches!(
        key,
        "title"
            | "description"
            | "default"
            | "examples"
            | "$comment"
            | "deprecated"
            | "readOnly"
            | "writeOnly"
    )
}

impl SchemaRelation {
    fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Incompatible, _) | (_, Self::Incompatible) => Self::Incompatible,
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            _ => Self::Subset,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn supported_relations_are_directional_and_unknowns_fail_closed() {
        let open_old = json!({
            "type": "object",
            "properties": { "name": { "type": "string" } },
            "required": ["name"]
        });
        let widened_input = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "note": { "type": "string" }
            }
        });
        assert_eq!(
            prove_schema_subset(&open_old, &widened_input).unwrap(),
            SchemaRelation::Incompatible
        );
        assert_eq!(
            prove_schema_subset(&json!({ "type": "integer" }), &json!({ "type": "number" }))
                .unwrap(),
            SchemaRelation::Subset
        );
        assert_eq!(
            prove_schema_subset(&json!({ "type": "number" }), &json!({ "type": "integer" }))
                .unwrap(),
            SchemaRelation::Incompatible
        );
        assert_eq!(
            prove_schema_subset(&json!({ "anyOf": [true] }), &json!({ "type": "string" })).unwrap(),
            SchemaRelation::Unknown
        );
        assert_eq!(
            prove_schema_subset(&json!(true), &json!({ "description": "annotation" })).unwrap(),
            SchemaRelation::Subset
        );
        assert_eq!(
            prove_schema_subset(&json!(true), &json!({ "anyOf": [true] })).unwrap(),
            SchemaRelation::Unknown
        );
        assert_eq!(
            prove_schema_subset(&json!({ "const": "x" }), &json!({ "type": "string" })).unwrap(),
            SchemaRelation::Subset
        );
        assert_eq!(
            prove_schema_subset(
                &json!({ "const": "a", "enum": ["a", "b"] }),
                &json!({ "const": "a" })
            )
            .unwrap(),
            SchemaRelation::Subset
        );
        assert_ne!(
            prove_schema_subset(
                &json!({ "const": "a", "enum": ["b"] }),
                &json!({ "enum": ["a", "b"] })
            )
            .unwrap(),
            SchemaRelation::Subset
        );
    }

    #[test]
    fn local_refs_resolve_and_cycles_do_not_claim_compatibility() {
        let referenced = json!({
            "$defs": { "value": { "type": "integer" } },
            "$ref": "#/$defs/value"
        });
        assert_eq!(
            prove_schema_subset(&referenced, &json!({ "type": "number" })).unwrap(),
            SchemaRelation::Subset
        );

        let recursive =
            json!({ "$defs": { "node": { "$ref": "#/$defs/node" } }, "$ref": "#/$defs/node" });
        assert_eq!(
            prove_schema_subset(&recursive, &json!({ "type": "object" })).unwrap(),
            SchemaRelation::Unknown
        );
    }

    #[test]
    fn equivalence_resolves_refs_but_rejects_object_evolution() {
        let referenced = json!({
            "$defs": { "value": { "type": "string", "description": "left" } },
            "$ref": "#/$defs/value"
        });
        assert_eq!(
            prove_schema_equivalent(
                &referenced,
                &json!({ "type": "string", "description": "right" })
            )
            .unwrap(),
            SchemaRelation::Subset
        );
        assert_eq!(
            prove_schema_equivalent(
                &json!({ "type": "object" }),
                &json!({
                    "type": "object",
                    "properties": { "optional": { "type": "string" } }
                })
            )
            .unwrap(),
            SchemaRelation::Incompatible
        );
    }
}
