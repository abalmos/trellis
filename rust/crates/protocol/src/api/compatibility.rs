use std::collections::{BTreeMap, BTreeSet};

use jsonptr::PointerBuf;
use serde::Serialize;

use super::{
    schema_compatibility::{prove_schema_equivalent, prove_schema_subset, SchemaRelation},
    ApiArtifact, ErrorDefinition, OperationDefinition, SchemaReference,
};
use crate::{
    canonicalize_json,
    identifiers::compare_protocol_strings,
    subjects::{
        derive_event_subject, derive_event_wildcard_subject, derive_feed_subject,
        derive_operation_subject, derive_rpc_subject,
    },
    ProtocolError,
};

/// Directional compatibility report for replacing one API artifact with another.
///
/// `compatible` means clients accepted against `previous` remain supported by
/// `candidate`; it makes no claim in the reverse direction. Issues are findings
/// about replacement safety, not authoring-validation errors.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCompatibilityReport {
    /// Whether the candidate can safely replace the previous artifact.
    pub compatible: bool,
    /// Deterministically ordered incompatibilities.
    pub issues: Vec<ApiCompatibilityIssue>,
}

/// One API replacement incompatibility.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCompatibilityIssue {
    /// Stable machine-readable issue category.
    pub code: ApiCompatibilityIssueCode,
    /// RFC 6901 pointer to the affected API member.
    pub path: String,
    /// Human-readable diagnostic.
    pub message: String,
}

/// Stable categories emitted by API replacement comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApiCompatibilityIssueCode {
    /// The artifacts are from different API lineages.
    ApiIdMismatch,
    /// A previously available surface is absent.
    SurfaceRemoved,
    /// A derived transport subject changed.
    SubjectChanged,
    /// A non-schema surface contract changed.
    DescriptorChanged,
    /// A schema relation was proven incompatible.
    SchemaIncompatible,
    /// The limited verifier could not prove the schema relation.
    SchemaRelationUnknown,
    /// A previously exported schema is absent.
    ExportRemoved,
    /// A previously declared capability is absent.
    CapabilityRemoved,
    /// A capability's normalized permission atoms changed.
    CapabilityChanged,
}

/// Compare a validated candidate API as a directional replacement for a previous API.
///
/// Both artifacts are expected to be validated members of the same exact API
/// lineage and major version. The comparison examines every publicly reachable
/// schema, including references reached through surfaces and exported schemas.
/// Object input schemas may add optional fields, but existing fields must remain
/// monotonic. Constructs whose relation cannot be proven safely, including
/// `oneOf`, object-sensitive `uniqueItems`, `maxContains`, and dynamic references,
/// produce conservative findings rather than optimistic compatibility.
///
/// Issues are ordered by API identity, RPC, operations, events, feeds, state,
/// exports, and capabilities. Authored names within each section use Trellis
/// UTF-16 code-unit ordering; checks within one descriptor use the fixed order
/// exercised by the compatibility conformance vectors.
///
/// # Errors
///
/// Returns a [`ProtocolError`] if either validated artifact cannot be projected,
/// digested, or converted to its derived transport subjects.
pub fn compare_api_replacement(
    previous: &ApiArtifact,
    candidate: &ApiArtifact,
) -> Result<ApiCompatibilityReport, ProtocolError> {
    if previous.digest()? == candidate.digest()? {
        return Ok(ApiCompatibilityReport {
            compatible: true,
            issues: Vec::new(),
        });
    }

    if previous.id != candidate.id {
        return Ok(report(vec![issue(
            ApiCompatibilityIssueCode::ApiIdMismatch,
            pointer(["id"]),
            format!(
                "candidate API id '{}' does not match previous id '{}'",
                candidate.id, previous.id
            ),
        )]));
    }

    let mut issues = Vec::new();
    let mut checked_errors = BTreeSet::new();
    compare_rpc(previous, candidate, &mut checked_errors, &mut issues)?;
    compare_operations(previous, candidate, &mut checked_errors, &mut issues)?;
    compare_events(previous, candidate, &mut issues)?;
    compare_feeds(previous, candidate, &mut issues)?;
    compare_state(previous, candidate, &mut issues)?;
    compare_exports(previous, candidate, &mut issues)?;
    compare_capabilities(previous, candidate, &mut issues);
    Ok(report(issues))
}

fn compare_rpc(
    previous: &ApiArtifact,
    candidate: &ApiArtifact,
    checked_errors: &mut BTreeSet<String>,
    issues: &mut Vec<ApiCompatibilityIssue>,
) -> Result<(), ProtocolError> {
    for (name, old) in sorted_entries(&previous.rpc) {
        let Some(new) = candidate.rpc.get(name) else {
            issues.push(removed("rpc", name));
            continue;
        };
        if derive_rpc_subject(&old.version, name)? != derive_rpc_subject(&new.version, name)? {
            issues.push(changed_subject("rpc", name));
        }
        compare_field("rpc", name, "version", &old.version, &new.version, issues);
        compare_field(
            "rpc",
            name,
            "internal",
            &old.internal,
            &new.internal,
            issues,
        );
        compare_field(
            "rpc",
            name,
            "transfer",
            &old.transfer,
            &new.transfer,
            issues,
        );
        compare_field("rpc", name, "errors", &old.errors, &new.errors, issues);
        compare_schema_ref(
            previous,
            &old.input,
            candidate,
            &new.input,
            pointer(["rpc", name, "input"]),
            issues,
        )?;
        compare_schema_ref(
            candidate,
            &new.output,
            previous,
            &old.output,
            pointer(["rpc", name, "output"]),
            issues,
        )?;
        compare_referenced_errors(previous, candidate, &old.errors, checked_errors, issues)?;
    }
    Ok(())
}

fn compare_operations(
    previous: &ApiArtifact,
    candidate: &ApiArtifact,
    checked_errors: &mut BTreeSet<String>,
    issues: &mut Vec<ApiCompatibilityIssue>,
) -> Result<(), ProtocolError> {
    for (name, old) in sorted_entries(&previous.operations) {
        let Some(new) = candidate.operations.get(name) else {
            issues.push(removed("operations", name));
            continue;
        };
        if derive_operation_subject(&old.version, name)?
            != derive_operation_subject(&new.version, name)?
        {
            issues.push(changed_subject("operations", name));
        }
        compare_field(
            "operations",
            name,
            "version",
            &old.version,
            &new.version,
            issues,
        );
        compare_field(
            "operations",
            name,
            "transfer",
            &old.transfer,
            &new.transfer,
            issues,
        );
        compare_field(
            "operations",
            name,
            "errors",
            &old.errors,
            &new.errors,
            issues,
        );
        if old.cancel && !new.cancel {
            issues.push(descriptor_changed("operations", name, "cancel"));
        }
        compare_schema_ref(
            previous,
            &old.input,
            candidate,
            &new.input,
            pointer(["operations", name, "input"]),
            issues,
        )?;
        for field in ["progress", "update", "output"] {
            compare_optional_output(previous, old, candidate, new, name, field, issues)?;
        }
        for (signal, old_signal) in sorted_entries(&old.signals) {
            let Some(new_signal) = new.signals.get(signal) else {
                issues.push(issue(
                    ApiCompatibilityIssueCode::SurfaceRemoved,
                    pointer(["operations", name, "signals", signal]),
                    format!("operation signal '{signal}' was removed"),
                ));
                continue;
            };
            compare_schema_ref(
                previous,
                &old_signal.input,
                candidate,
                &new_signal.input,
                pointer(["operations", name, "signals", signal, "input"]),
                issues,
            )?;
        }
        compare_referenced_errors(previous, candidate, &old.errors, checked_errors, issues)?;
    }
    Ok(())
}

fn compare_optional_output(
    previous: &ApiArtifact,
    old: &OperationDefinition,
    candidate: &ApiArtifact,
    new: &OperationDefinition,
    name: &str,
    field: &str,
    issues: &mut Vec<ApiCompatibilityIssue>,
) -> Result<(), ProtocolError> {
    let (old, new) = match field {
        "progress" => (old.progress.as_ref(), new.progress.as_ref()),
        "update" => (old.update.as_ref(), new.update.as_ref()),
        "output" => (old.output.as_ref(), new.output.as_ref()),
        _ => unreachable!("fixed operation schema field"),
    };
    match (old, new) {
        (Some(old), Some(new)) => compare_schema_ref(
            candidate,
            new,
            previous,
            old,
            pointer(["operations", name, field]),
            issues,
        ),
        (None, None) => Ok(()),
        _ => {
            issues.push(descriptor_changed("operations", name, field));
            Ok(())
        }
    }
}

fn compare_events(
    previous: &ApiArtifact,
    candidate: &ApiArtifact,
    issues: &mut Vec<ApiCompatibilityIssue>,
) -> Result<(), ProtocolError> {
    for (name, old) in sorted_entries(&previous.events) {
        let Some(new) = candidate.events.get(name) else {
            issues.push(removed("events", name));
            continue;
        };
        let old_subject = derive_event_subject(&old.version, name)?;
        let new_subject = derive_event_subject(&new.version, name)?;
        let old_wildcard = derive_event_wildcard_subject(&old.version, name, old.params.len())?;
        let new_wildcard = derive_event_wildcard_subject(&new.version, name, new.params.len())?;
        if old_subject != new_subject || old_wildcard != new_wildcard {
            issues.push(changed_subject("events", name));
        }
        compare_field(
            "events",
            name,
            "version",
            &old.version,
            &new.version,
            issues,
        );
        compare_field("events", name, "params", &old.params, &new.params, issues);
        compare_field("events", name, "class", &old.class, &new.class, issues);
        compare_schema_ref(
            candidate,
            &new.event,
            previous,
            &old.event,
            pointer(["events", name, "event"]),
            issues,
        )?;
    }
    Ok(())
}

fn compare_feeds(
    previous: &ApiArtifact,
    candidate: &ApiArtifact,
    issues: &mut Vec<ApiCompatibilityIssue>,
) -> Result<(), ProtocolError> {
    for (name, old) in sorted_entries(&previous.feeds) {
        let Some(new) = candidate.feeds.get(name) else {
            issues.push(removed("feeds", name));
            continue;
        };
        if derive_feed_subject(&old.version, name)? != derive_feed_subject(&new.version, name)? {
            issues.push(changed_subject("feeds", name));
        }
        compare_field("feeds", name, "version", &old.version, &new.version, issues);
        compare_schema_ref(
            previous,
            &old.input,
            candidate,
            &new.input,
            pointer(["feeds", name, "input"]),
            issues,
        )?;
        compare_schema_ref(
            candidate,
            &new.event,
            previous,
            &old.event,
            pointer(["feeds", name, "event"]),
            issues,
        )?;
    }
    Ok(())
}

fn compare_state(
    previous: &ApiArtifact,
    candidate: &ApiArtifact,
    issues: &mut Vec<ApiCompatibilityIssue>,
) -> Result<(), ProtocolError> {
    for (name, old) in sorted_entries(&previous.state) {
        let Some(new) = candidate.state.get(name) else {
            issues.push(removed("state", name));
            continue;
        };
        compare_field("state", name, "kind", &old.kind, &new.kind, issues);
        compare_field(
            "state",
            name,
            "stateVersion",
            &old.state_version,
            &new.state_version,
            issues,
        );
        compare_equivalent_schema_ref(
            previous,
            &old.schema,
            candidate,
            &new.schema,
            pointer(["state", name, "schema"]),
            issues,
        )?;
        for (version, old_schema) in sorted_entries(&old.accepted_versions) {
            let path = pointer(["state", name, "acceptedVersions", version]);
            let Some(new_schema) = new.accepted_versions.get(version) else {
                issues.push(issue(
                    ApiCompatibilityIssueCode::DescriptorChanged,
                    path,
                    format!("accepted state version '{version}' was removed"),
                ));
                continue;
            };
            compare_equivalent_schema_ref(
                previous, old_schema, candidate, new_schema, path, issues,
            )?;
        }
    }
    Ok(())
}

fn compare_exports(
    previous: &ApiArtifact,
    candidate: &ApiArtifact,
    issues: &mut Vec<ApiCompatibilityIssue>,
) -> Result<(), ProtocolError> {
    for name in &previous.exports.schemas {
        let path = pointer(["exports", "schemas", name]);
        if !candidate.exports.schemas.contains(name) {
            issues.push(issue(
                ApiCompatibilityIssueCode::ExportRemoved,
                path,
                format!("exported schema '{name}' was removed"),
            ));
            continue;
        }
        if canonicalize_json(&previous.schemas[name])?
            != canonicalize_json(&candidate.schemas[name])?
        {
            issues.push(issue(
                ApiCompatibilityIssueCode::DescriptorChanged,
                path,
                format!("exported schema '{name}' changed"),
            ));
        }
    }
    Ok(())
}

fn compare_capabilities(
    previous: &ApiArtifact,
    candidate: &ApiArtifact,
    issues: &mut Vec<ApiCompatibilityIssue>,
) {
    for (name, old) in sorted_entries(&previous.capabilities) {
        let path = pointer(["capabilities", name]);
        match candidate.capabilities.get(name) {
            None => issues.push(issue(
                ApiCompatibilityIssueCode::CapabilityRemoved,
                path,
                format!("capability '{name}' was removed"),
            )),
            Some(new) if old != new => issues.push(issue(
                ApiCompatibilityIssueCode::CapabilityChanged,
                path,
                format!("capability '{name}' permissions changed"),
            )),
            Some(_) => {}
        }
    }
}

fn compare_referenced_errors(
    previous: &ApiArtifact,
    candidate: &ApiArtifact,
    names: &[String],
    checked: &mut BTreeSet<String>,
    issues: &mut Vec<ApiCompatibilityIssue>,
) -> Result<(), ProtocolError> {
    for name in names {
        if !checked.insert(name.clone()) {
            continue;
        }
        let old = &previous.errors[name];
        let Some(new) = candidate.errors.get(name) else {
            issues.push(issue(
                ApiCompatibilityIssueCode::SchemaIncompatible,
                pointer(["errors", name]),
                format!("referenced error '{name}' was removed"),
            ));
            continue;
        };
        compare_error_schema(previous, old, candidate, new, name, issues)?;
    }
    Ok(())
}

fn compare_error_schema(
    previous: &ApiArtifact,
    old: &ErrorDefinition,
    candidate: &ApiArtifact,
    new: &ErrorDefinition,
    name: &str,
    issues: &mut Vec<ApiCompatibilityIssue>,
) -> Result<(), ProtocolError> {
    let path = pointer(["errors", name, "schema"]);
    match (&old.schema, &new.schema) {
        (None, None) | (None, Some(_)) => Ok(()),
        (Some(_), None) => {
            issues.push(issue(
                ApiCompatibilityIssueCode::SchemaIncompatible,
                path,
                format!("referenced error '{name}' no longer has a constrained schema"),
            ));
            Ok(())
        }
        (Some(old), Some(new)) => compare_schema_ref(candidate, new, previous, old, path, issues),
    }
}

fn compare_schema_ref(
    sub_artifact: &ApiArtifact,
    sub: &SchemaReference,
    super_artifact: &ApiArtifact,
    super_: &SchemaReference,
    path: String,
    issues: &mut Vec<ApiCompatibilityIssue>,
) -> Result<(), ProtocolError> {
    push_schema_relation(
        prove_schema_subset(
            &sub_artifact.schemas[&sub.schema],
            &super_artifact.schemas[&super_.schema],
        )?,
        path,
        issues,
    );
    Ok(())
}

fn compare_equivalent_schema_ref(
    left_artifact: &ApiArtifact,
    left: &SchemaReference,
    right_artifact: &ApiArtifact,
    right: &SchemaReference,
    path: String,
    issues: &mut Vec<ApiCompatibilityIssue>,
) -> Result<(), ProtocolError> {
    push_schema_relation(
        prove_schema_equivalent(
            &left_artifact.schemas[&left.schema],
            &right_artifact.schemas[&right.schema],
        )?,
        path,
        issues,
    );
    Ok(())
}

fn push_schema_relation(
    relation: SchemaRelation,
    path: String,
    issues: &mut Vec<ApiCompatibilityIssue>,
) {
    let (code, message) = match relation {
        SchemaRelation::Subset => return,
        SchemaRelation::Incompatible => (
            ApiCompatibilityIssueCode::SchemaIncompatible,
            "schema is not directionally compatible",
        ),
        SchemaRelation::Unknown => (
            ApiCompatibilityIssueCode::SchemaRelationUnknown,
            "schema relation is outside the supported conservative subset",
        ),
    };
    issues.push(issue(code, path, message));
}

fn compare_field<T: PartialEq>(
    section: &str,
    name: &str,
    field: &str,
    old: &T,
    new: &T,
    issues: &mut Vec<ApiCompatibilityIssue>,
) {
    if old != new {
        issues.push(descriptor_changed(section, name, field));
    }
}

fn sorted_entries<T>(map: &BTreeMap<String, T>) -> Vec<(&str, &T)> {
    let mut entries = map
        .iter()
        .map(|(name, value)| (name.as_str(), value))
        .collect::<Vec<_>>();
    entries.sort_by(|(left, _), (right, _)| compare_protocol_strings(left, right));
    entries
}

fn report(issues: Vec<ApiCompatibilityIssue>) -> ApiCompatibilityReport {
    ApiCompatibilityReport {
        compatible: issues.is_empty(),
        issues,
    }
}

fn removed(section: &str, name: &str) -> ApiCompatibilityIssue {
    issue(
        ApiCompatibilityIssueCode::SurfaceRemoved,
        pointer([section, name]),
        format!("{section} surface '{name}' was removed"),
    )
}

fn changed_subject(section: &str, name: &str) -> ApiCompatibilityIssue {
    issue(
        ApiCompatibilityIssueCode::SubjectChanged,
        pointer([section, name, "subject"]),
        format!("derived subject for {section} surface '{name}' changed"),
    )
}

fn descriptor_changed(section: &str, name: &str, field: &str) -> ApiCompatibilityIssue {
    issue(
        ApiCompatibilityIssueCode::DescriptorChanged,
        pointer([section, name, field]),
        format!("{section} surface '{name}' changed '{field}'"),
    )
}

fn issue(
    code: ApiCompatibilityIssueCode,
    path: String,
    message: impl Into<String>,
) -> ApiCompatibilityIssue {
    ApiCompatibilityIssue {
        code,
        path,
        message: message.into(),
    }
}

fn pointer<'a>(tokens: impl IntoIterator<Item = &'a str>) -> String {
    PointerBuf::from_tokens(tokens).to_string()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use serde::Deserialize;
    use serde_json::{json, Map, Value};

    use super::*;
    use crate::parse_api;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Vector {
        name: String,
        previous: Value,
        candidate: Value,
        compatible: bool,
        issues: Vec<ExpectedIssue>,
    }

    #[derive(Deserialize)]
    struct ExpectedIssue {
        code: String,
        path: String,
    }

    #[derive(Deserialize)]
    struct Fixture {
        vectors: Vec<Vector>,
    }

    #[test]
    fn shared_api_compatibility_vectors_pass() {
        let fixture = fixture();
        assert!(fixture.vectors.len() >= 32);
        for vector in fixture.vectors {
            let previous = parse_api(&vector.previous)
                .unwrap_or_else(|error| panic!("{} previous: {error}", vector.name));
            let candidate = parse_api(&vector.candidate)
                .unwrap_or_else(|error| panic!("{} candidate: {error}", vector.name));
            let report = compare_api_replacement(&previous, &candidate)
                .unwrap_or_else(|error| panic!("{} comparison: {error}", vector.name));
            assert_eq!(report.compatible, vector.compatible, "{}", vector.name);
            let actual = report
                .issues
                .iter()
                .map(|issue| {
                    (
                        serde_json::to_value(issue.code)
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_owned(),
                        issue.path.clone(),
                    )
                })
                .collect::<Vec<_>>();
            let expected = vector
                .issues
                .iter()
                .map(|issue| (issue.code.clone(), issue.path.clone()))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{}", vector.name);
            for issue in &report.issues {
                jsonptr::Pointer::parse(&issue.path)
                    .unwrap_or_else(|error| panic!("{} invalid path: {error}", vector.name));
            }
        }
    }

    #[test]
    fn comparison_ignores_input_object_order() {
        let vector = fixture()
            .vectors
            .into_iter()
            .find(|vector| vector.name == "deterministic-section-and-utf16-order")
            .unwrap();
        let previous = parse_api(&vector.previous).unwrap();
        let reordered = parse_api(&reverse_object(vector.previous)).unwrap();
        let candidate = parse_api(&vector.candidate).unwrap();
        assert_eq!(
            compare_api_replacement(&previous, &candidate).unwrap(),
            compare_api_replacement(&reordered, &candidate).unwrap()
        );
    }

    #[test]
    fn report_serialization_is_stable() {
        let report = ApiCompatibilityReport {
            compatible: false,
            issues: vec![issue(
                ApiCompatibilityIssueCode::SurfaceRemoved,
                "/rpc/Example.Get".to_owned(),
                "rpc surface 'Example.Get' was removed",
            )],
        };
        assert_eq!(
            serde_json::to_value(report).unwrap(),
            json!({
                "compatible": false,
                "issues": [{
                    "code": "surface-removed",
                    "path": "/rpc/Example.Get",
                    "message": "rpc surface 'Example.Get' was removed"
                }]
            })
        );
    }

    fn fixture() -> Fixture {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../conformance/api-compatibility/vectors.json");
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    fn reverse_object(value: Value) -> Value {
        match value {
            Value::Object(object) => Value::Object(object.into_iter().rev().collect::<Map<_, _>>()),
            value => value,
        }
    }
}
