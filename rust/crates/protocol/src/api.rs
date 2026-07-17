use std::collections::{BTreeMap, BTreeSet};

use jsonptr::PointerBuf;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use crate::{
    canonicalize_json, digest_json,
    identifiers::{
        api_error, sort_deduplicate, validate_api_id, validate_logical_name,
        validate_nonempty_text, validate_protocol_identifier, validate_version,
    },
    schema_profile::{
        lint_api_authoring, validate_api_runtime_structure, validate_embedded_schema,
        validate_wire_schema_additive,
    },
    subjects::{
        derive_event_subject, derive_event_wildcard_subject, derive_feed_subject,
        derive_operation_subject, derive_rpc_subject, DerivedApiSubjectsV1, DerivedEventSubjectsV1,
    },
    ApiSurfaceKindV1, CapabilityDefinitionV1, ConsentMetadataV1, PermissionActionV1, ProtocolError,
};

mod compatibility;
mod schema_compatibility;

pub use compatibility::{
    compare_api_replacement_v1, ApiCompatibilityIssueCodeV1, ApiCompatibilityIssueV1,
    ApiCompatibilityReportV1,
};

/// The first canonical Trellis API artifact format.
pub const API_FORMAT_V1: &str = "trellis.api.v1";

/// Strict Draft 2020-12 schema for authoring `trellis.api.v1` artifacts.
pub const API_AUTHORING_SCHEMA_V1_JSON: &str =
    include_str!("../schemas/trellis.api.v1.schema.json");

/// Apply the strict, closed authoring lint to an API artifact.
///
/// Runtime parsing is intentionally tolerant of unknown object members; use
/// this lint in authoring tools when extensions should be reported.
pub fn lint_api_v1_authoring(value: &Value) -> Result<(), ProtocolError> {
    lint_api_authoring(value)?;
    parse_api_v1(value).map(|_| ())
}

/// One validated, normalized `trellis.api.v1` artifact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiArtifactV1 {
    format: String,
    id: String,
    display_name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    schemas: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "ExportsV1::is_empty")]
    exports: ExportsV1,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    errors: BTreeMap<String, ErrorDefinitionV1>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    rpc: BTreeMap<String, RpcDefinitionV1>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    operations: BTreeMap<String, OperationDefinitionV1>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    events: BTreeMap<String, EventDefinitionV1>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    feeds: BTreeMap<String, FeedDefinitionV1>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    state: BTreeMap<String, StateDefinitionV1>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    capabilities: BTreeMap<String, CapabilityDefinitionV1>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    consent: BTreeMap<String, ConsentMetadataV1>,
}

/// Validate and parse one raw `trellis.api.v1` JSON value.
///
/// Unknown object members are ignored and do not affect normalization,
/// compatibility, capabilities, or the semantic digest.
pub fn parse_api_v1(value: &Value) -> Result<ApiArtifactV1, ProtocolError> {
    validate_api_runtime_structure(value)?;
    let wire: WireApiArtifactV1 =
        serde_json::from_value(value.clone()).map_err(|error| api_error("", error.to_string()))?;
    ApiArtifactV1::from_wire(wire)
}

impl ApiArtifactV1 {
    /// Return the stable versioned API lineage identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Serialize the normalized supported API shape.
    pub fn normalized_value(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::to_value(self)?)
    }

    /// Render the complete normalized API artifact as RFC 8785 JSON.
    pub fn canonical_json(&self) -> Result<String, ProtocolError> {
        canonicalize_json(&self.normalized_value()?)
    }

    /// Build the machine-visible API identity projection.
    pub fn digest_projection(&self) -> Result<Value, ProtocolError> {
        let mut projection = Map::new();
        projection.insert("format".to_string(), Value::String(self.format.clone()));
        projection.insert("id".to_string(), Value::String(self.id.clone()));
        insert_nonempty(&mut projection, "schemas", &self.schemas)?;
        if !self.exports.is_empty() {
            projection.insert("exports".to_string(), serde_json::to_value(&self.exports)?);
        }
        insert_without_docs(&mut projection, "errors", &self.errors, false)?;
        insert_without_docs(&mut projection, "rpc", &self.rpc, false)?;
        insert_without_docs(&mut projection, "operations", &self.operations, true)?;
        insert_without_docs(&mut projection, "events", &self.events, false)?;
        insert_without_docs(&mut projection, "feeds", &self.feeds, false)?;
        insert_without_docs(&mut projection, "state", &self.state, false)?;
        insert_nonempty(&mut projection, "capabilities", &self.capabilities)?;
        Ok(Value::Object(projection))
    }

    /// Compute the content digest of the machine-visible API projection.
    pub fn digest(&self) -> Result<String, ProtocolError> {
        digest_json(&self.digest_projection()?)
    }

    /// Derive every public communication subject owned by this API.
    pub fn derived_subjects(&self) -> Result<DerivedApiSubjectsV1, ProtocolError> {
        let rpc = self
            .rpc
            .iter()
            .map(|(name, definition)| {
                Ok((name.clone(), derive_rpc_subject(&definition.version, name)?))
            })
            .collect::<Result<_, ProtocolError>>()?;
        let operations = self
            .operations
            .iter()
            .map(|(name, definition)| {
                Ok((
                    name.clone(),
                    derive_operation_subject(&definition.version, name)?,
                ))
            })
            .collect::<Result<_, ProtocolError>>()?;
        let events = self
            .events
            .iter()
            .map(|(name, definition)| {
                Ok((
                    name.clone(),
                    DerivedEventSubjectsV1 {
                        base: derive_event_subject(&definition.version, name)?,
                        wildcard: derive_event_wildcard_subject(
                            &definition.version,
                            name,
                            definition.params.len(),
                        )?,
                    },
                ))
            })
            .collect::<Result<_, ProtocolError>>()?;
        let feeds = self
            .feeds
            .iter()
            .map(|(name, definition)| {
                Ok((
                    name.clone(),
                    derive_feed_subject(&definition.version, name)?,
                ))
            })
            .collect::<Result<_, ProtocolError>>()?;
        Ok(DerivedApiSubjectsV1 {
            rpc,
            operations,
            events,
            feeds,
        })
    }

    fn from_wire(mut wire: WireApiArtifactV1) -> Result<Self, ProtocolError> {
        if wire.format != API_FORMAT_V1 {
            return Err(api_error(
                "/format",
                format!("must equal '{API_FORMAT_V1}'"),
            ));
        }
        validate_api_id("/id", &wire.id, api_error)?;
        validate_nonempty_text("/displayName", &wire.display_name, api_error)?;
        validate_nonempty_text("/description", &wire.description, api_error)?;
        if let Some(docs) = &wire.docs {
            validate_docs("/docs", docs)?;
        }

        for (name, schema) in &wire.schemas {
            validate_protocol_identifier(&member_path("schemas", name), name, api_error)?;
            validate_embedded_schema(name, schema)?;
        }
        for name in &wire.exports.schemas {
            validate_protocol_identifier("/exports/schemas", name, api_error)?;
            require_key(&wire.schemas, name, "/exports/schemas", "schema")?;
        }
        sort_deduplicate(&mut wire.exports.schemas);

        for (name, definition) in &wire.errors {
            let path = member_path("errors", name);
            validate_protocol_identifier(&path, name, api_error)?;
            validate_optional_schema_ref(
                &wire.schemas,
                definition.schema.as_ref(),
                &format!("{path}/schema"),
            )?;
            if let Some(docs) = &definition.docs {
                validate_docs(&format!("{path}/docs"), docs)?;
            }
        }

        for (name, definition) in &mut wire.rpc {
            let path = member_path("rpc", name);
            validate_surface(&path, name, &definition.version)?;
            require_schema(&wire.schemas, &definition.input, &format!("{path}/input"))?;
            require_schema(&wire.schemas, &definition.output, &format!("{path}/output"))?;
            validate_error_refs(
                &wire.errors,
                &mut definition.errors,
                &format!("{path}/errors"),
            )?;
            if definition
                .transfer
                .as_ref()
                .is_some_and(|transfer| transfer.direction != TransferDirectionV1::Receive)
            {
                return Err(api_error(
                    format!("{path}/transfer/direction"),
                    "RPC transfer direction must be 'receive'",
                ));
            }
            if let Some(docs) = &definition.docs {
                validate_docs(&format!("{path}/docs"), docs)?;
            }
        }

        for (name, definition) in &mut wire.operations {
            let path = member_path("operations", name);
            validate_surface(&path, name, &definition.version)?;
            require_schema(&wire.schemas, &definition.input, &format!("{path}/input"))?;
            for (field, reference) in [
                ("progress", definition.progress.as_ref()),
                ("update", definition.update.as_ref()),
                ("output", definition.output.as_ref()),
            ] {
                validate_optional_schema_ref(&wire.schemas, reference, &format!("{path}/{field}"))?;
            }
            validate_error_refs(
                &wire.errors,
                &mut definition.errors,
                &format!("{path}/errors"),
            )?;
            if definition
                .transfer
                .as_ref()
                .is_some_and(|transfer| transfer.direction != TransferDirectionV1::Send)
            {
                return Err(api_error(
                    format!("{path}/transfer/direction"),
                    "operation transfer direction must be 'send'",
                ));
            }
            for (signal, descriptor) in &definition.signals {
                let signal_path = pointer(["operations", name, "signals", signal]);
                validate_protocol_identifier(&signal_path, signal, api_error)?;
                require_schema(
                    &wire.schemas,
                    &descriptor.input,
                    &format!("{signal_path}/input"),
                )?;
                if let Some(docs) = &descriptor.docs {
                    validate_docs(&format!("{signal_path}/docs"), docs)?;
                }
            }
            if let Some(docs) = &definition.docs {
                validate_docs(&format!("{path}/docs"), docs)?;
            }
        }

        for (name, definition) in &wire.events {
            let path = member_path("events", name);
            validate_surface(&path, name, &definition.version)?;
            require_schema(&wire.schemas, &definition.event, &format!("{path}/event"))?;
            let mut pointers = BTreeSet::new();
            for (index, pointer) in definition.params.iter().enumerate() {
                let pointer_path = format!("{path}/params/{index}");
                let parsed = jsonptr::Pointer::parse(pointer)
                    .map_err(|error| api_error(&pointer_path, error.to_string()))?;
                if parsed.is_root() {
                    return Err(api_error(
                        pointer_path,
                        "event subject parameters must not use the root pointer",
                    ));
                }
                if !pointers.insert(pointer) {
                    return Err(api_error(
                        pointer_path,
                        "event parameter pointers must be unique",
                    ));
                }
            }
            if let Some(docs) = &definition.docs {
                validate_docs(&format!("{path}/docs"), docs)?;
            }
        }

        for (name, definition) in &wire.feeds {
            let path = member_path("feeds", name);
            validate_surface(&path, name, &definition.version)?;
            require_schema(&wire.schemas, &definition.input, &format!("{path}/input"))?;
            require_schema(&wire.schemas, &definition.event, &format!("{path}/event"))?;
            if let Some(docs) = &definition.docs {
                validate_docs(&format!("{path}/docs"), docs)?;
            }
        }

        for (name, definition) in &wire.state {
            let path = member_path("state", name);
            validate_protocol_identifier(&path, name, api_error)?;
            require_schema(&wire.schemas, &definition.schema, &format!("{path}/schema"))?;
            if let Some(version) = &definition.state_version {
                validate_protocol_identifier(&format!("{path}/stateVersion"), version, api_error)?;
            }
            for (version, reference) in &definition.accepted_versions {
                let version_path = pointer(["state", name, "acceptedVersions", version]);
                validate_protocol_identifier(&version_path, version, api_error)?;
                require_schema(&wire.schemas, reference, &version_path)?;
            }
            if let Some(docs) = &definition.docs {
                validate_docs(&format!("{path}/docs"), docs)?;
            }
        }

        let mut public_schemas = wire.exports.schemas.iter().collect::<BTreeSet<_>>();
        for definition in wire.rpc.values() {
            public_schemas.insert(&definition.input.schema);
            public_schemas.insert(&definition.output.schema);
            for error in &definition.errors {
                if let Some(schema) = wire.errors[error].schema.as_ref() {
                    public_schemas.insert(&schema.schema);
                }
            }
        }
        for definition in wire.operations.values() {
            public_schemas.insert(&definition.input.schema);
            public_schemas.extend(
                [
                    definition.progress.as_ref(),
                    definition.update.as_ref(),
                    definition.output.as_ref(),
                ]
                .into_iter()
                .flatten()
                .map(|reference| &reference.schema),
            );
            public_schemas.extend(
                definition
                    .signals
                    .values()
                    .map(|signal| &signal.input.schema),
            );
            for error in &definition.errors {
                if let Some(schema) = wire.errors[error].schema.as_ref() {
                    public_schemas.insert(&schema.schema);
                }
            }
        }
        for definition in wire.events.values() {
            public_schemas.insert(&definition.event.schema);
        }
        for definition in wire.feeds.values() {
            public_schemas.insert(&definition.input.schema);
            public_schemas.insert(&definition.event.schema);
        }
        for definition in wire.state.values() {
            public_schemas.insert(&definition.schema.schema);
            public_schemas.extend(
                definition
                    .accepted_versions
                    .values()
                    .map(|reference| &reference.schema),
            );
        }
        for name in public_schemas {
            validate_wire_schema_additive(name, &wire.schemas[name])?;
        }

        for (capability_name, capability) in &wire.capabilities {
            let path = member_path("capabilities", capability_name);
            validate_protocol_identifier(&path, capability_name, api_error)?;
            for (index, atom) in capability.allows().iter().enumerate() {
                let atom_path = format!("{path}/allows/{index}");
                if let Some((api, operation, signal)) = atom.target().as_operation_signal() {
                    if api != wire.id {
                        return Err(api_error(
                            atom_path,
                            format!("capability target API '{api}' must equal '{}'", wire.id),
                        ));
                    }
                    let Some(operation_definition) = wire.operations.get(operation) else {
                        return Err(api_error(
                            atom_path,
                            format!("capability targets missing operation '{operation}'"),
                        ));
                    };
                    if !operation_definition.signals.contains_key(signal) {
                        return Err(api_error(
                            atom_path,
                            format!(
                                "capability targets missing signal '{signal}' on operation '{operation}'"
                            ),
                        ));
                    }
                    continue;
                }
                let Some((api, surface, name)) = atom.target().as_api_surface() else {
                    return Err(api_error(
                        atom_path,
                        "API capabilities cannot target participant resources",
                    ));
                };
                if api != wire.id {
                    return Err(api_error(
                        atom_path,
                        format!("capability target API '{api}' must equal '{}'", wire.id),
                    ));
                }
                let exists = match surface {
                    ApiSurfaceKindV1::Rpc => wire.rpc.contains_key(name),
                    ApiSurfaceKindV1::Operation => wire.operations.contains_key(name),
                    ApiSurfaceKindV1::Event => wire.events.contains_key(name),
                    ApiSurfaceKindV1::Feed => wire.feeds.contains_key(name),
                    ApiSurfaceKindV1::State => wire.state.contains_key(name),
                };
                if !exists {
                    return Err(api_error(
                        &atom_path,
                        format!("capability targets missing {surface:?} surface '{name}'"),
                    ));
                }
                if surface == ApiSurfaceKindV1::Operation {
                    let operation = &wire.operations[name];
                    if atom.action() == PermissionActionV1::Cancel && !operation.cancel {
                        return Err(api_error(
                            &atom_path,
                            "cancel permission requires a cancelable operation",
                        ));
                    }
                }
            }
        }
        for capability in wire.consent.keys() {
            validate_protocol_identifier(
                &member_path("consent", capability),
                capability,
                api_error,
            )?;
            require_key(
                &wire.capabilities,
                capability,
                &member_path("consent", capability),
                "capability",
            )?;
        }

        let artifact = Self {
            format: wire.format,
            id: wire.id,
            display_name: wire.display_name,
            description: wire.description,
            docs: wire.docs,
            schemas: wire.schemas,
            exports: wire.exports,
            errors: wire.errors,
            rpc: wire.rpc,
            operations: wire.operations,
            events: wire.events,
            feeds: wire.feeds,
            state: wire.state,
            capabilities: wire.capabilities,
            consent: wire.consent,
        };
        artifact.validate_subject_collisions()?;
        Ok(artifact)
    }

    fn validate_subject_collisions(&self) -> Result<(), ProtocolError> {
        let subjects = self.derived_subjects()?;
        validate_unique_subjects("/rpc", subjects.rpc.values())?;
        validate_unique_subjects("/operations", subjects.operations.values())?;
        validate_event_subjects(&subjects.events)?;
        validate_unique_subjects("/feeds", subjects.feeds.values())
    }
}

impl<'de> Deserialize<'de> for ApiArtifactV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        parse_api_v1(&value).map_err(D::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireApiArtifactV1 {
    format: String,
    id: String,
    display_name: String,
    description: String,
    #[serde(default)]
    docs: Option<DocumentationV1>,
    #[serde(default)]
    schemas: BTreeMap<String, Value>,
    #[serde(default)]
    exports: ExportsV1,
    #[serde(default)]
    errors: BTreeMap<String, ErrorDefinitionV1>,
    #[serde(default)]
    rpc: BTreeMap<String, RpcDefinitionV1>,
    #[serde(default)]
    operations: BTreeMap<String, OperationDefinitionV1>,
    #[serde(default)]
    events: BTreeMap<String, EventDefinitionV1>,
    #[serde(default)]
    feeds: BTreeMap<String, FeedDefinitionV1>,
    #[serde(default)]
    state: BTreeMap<String, StateDefinitionV1>,
    #[serde(default)]
    capabilities: BTreeMap<String, CapabilityDefinitionV1>,
    #[serde(default)]
    consent: BTreeMap<String, ConsentMetadataV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct DocumentationV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    markdown: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct ExportsV1 {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    schemas: Vec<String>,
}

impl ExportsV1 {
    fn is_empty(&self) -> bool {
        self.schemas.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct SchemaReferenceV1 {
    schema: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum TransferDirectionV1 {
    Send,
    Receive,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct TransferDefinitionV1 {
    direction: TransferDirectionV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct ErrorDefinitionV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<SchemaReferenceV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct RpcDefinitionV1 {
    version: String,
    input: SchemaReferenceV1,
    output: SchemaReferenceV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transfer: Option<TransferDefinitionV1>,
    #[serde(default, skip_serializing_if = "is_false")]
    internal: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct OperationSignalV1 {
    input: SchemaReferenceV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct OperationDefinitionV1 {
    version: String,
    input: SchemaReferenceV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<SchemaReferenceV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    update: Option<SchemaReferenceV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<SchemaReferenceV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transfer: Option<TransferDefinitionV1>,
    #[serde(default, skip_serializing_if = "is_false")]
    cancel: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    signals: BTreeMap<String, OperationSignalV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum EventClassV1 {
    #[default]
    Domain,
    Audit,
    Control,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct EventDefinitionV1 {
    version: String,
    event: SchemaReferenceV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    params: Vec<String>,
    #[serde(default, skip_serializing_if = "is_domain")]
    class: EventClassV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct FeedDefinitionV1 {
    version: String,
    input: SchemaReferenceV1,
    event: SchemaReferenceV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum StateKindV1 {
    Value,
    Map,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StateDefinitionV1 {
    kind: StateKindV1,
    schema: SchemaReferenceV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_version: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    accepted_versions: BTreeMap<String, SchemaReferenceV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

fn validate_docs(path: &str, docs: &DocumentationV1) -> Result<(), ProtocolError> {
    validate_nonempty_text(&format!("{path}/markdown"), &docs.markdown, api_error)?;
    if let Some(summary) = &docs.summary {
        validate_nonempty_text(&format!("{path}/summary"), summary, api_error)?;
    }
    Ok(())
}

fn validate_surface(path: &str, name: &str, version: &str) -> Result<(), ProtocolError> {
    validate_logical_name(path, name, api_error)?;
    validate_version(&format!("{path}/version"), version, api_error)
}

fn require_schema(
    schemas: &BTreeMap<String, Value>,
    reference: &SchemaReferenceV1,
    path: &str,
) -> Result<(), ProtocolError> {
    validate_protocol_identifier(&format!("{path}/schema"), &reference.schema, api_error)?;
    require_key(schemas, &reference.schema, path, "schema")
}

fn validate_optional_schema_ref(
    schemas: &BTreeMap<String, Value>,
    reference: Option<&SchemaReferenceV1>,
    path: &str,
) -> Result<(), ProtocolError> {
    if let Some(reference) = reference {
        require_schema(schemas, reference, path)?;
    }
    Ok(())
}

fn validate_error_refs(
    errors: &BTreeMap<String, ErrorDefinitionV1>,
    references: &mut Vec<String>,
    path: &str,
) -> Result<(), ProtocolError> {
    for reference in references.iter() {
        validate_protocol_identifier(path, reference, api_error)?;
        require_key(errors, reference, path, "error")?;
    }
    sort_deduplicate(references);
    Ok(())
}

fn require_key<T>(
    map: &BTreeMap<String, T>,
    name: &str,
    path: &str,
    kind: &str,
) -> Result<(), ProtocolError> {
    if map.contains_key(name) {
        Ok(())
    } else {
        Err(api_error(
            path,
            format!("references unknown {kind} '{name}'"),
        ))
    }
}

fn member_path(section: &str, name: &str) -> String {
    pointer([section, name])
}

fn pointer<'a>(tokens: impl IntoIterator<Item = &'a str>) -> String {
    PointerBuf::from_tokens(tokens).to_string()
}

fn insert_nonempty<T: Serialize>(
    projection: &mut Map<String, Value>,
    key: &str,
    map: &BTreeMap<String, T>,
) -> Result<(), ProtocolError> {
    if !map.is_empty() {
        projection.insert(key.to_string(), serde_json::to_value(map)?);
    }
    Ok(())
}

fn insert_without_docs<T: Serialize>(
    projection: &mut Map<String, Value>,
    key: &str,
    map: &BTreeMap<String, T>,
    operation_signals: bool,
) -> Result<(), ProtocolError> {
    if map.is_empty() {
        return Ok(());
    }
    let mut value = serde_json::to_value(map)?;
    if let Some(definitions) = value.as_object_mut() {
        for definition in definitions.values_mut().filter_map(Value::as_object_mut) {
            definition.remove("docs");
            if operation_signals {
                if let Some(signals) = definition.get_mut("signals").and_then(Value::as_object_mut)
                {
                    for signal in signals.values_mut().filter_map(Value::as_object_mut) {
                        signal.remove("docs");
                    }
                }
            }
        }
    }
    projection.insert(key.to_string(), value);
    Ok(())
}

fn validate_unique_subjects<'a>(
    path: &str,
    subjects: impl IntoIterator<Item = &'a String>,
) -> Result<(), ProtocolError> {
    let mut seen = BTreeSet::new();
    for subject in subjects {
        if !seen.insert(subject) {
            return Err(api_error(
                path,
                format!("derived subject collision at '{subject}'"),
            ));
        }
    }
    Ok(())
}

fn validate_event_subjects(
    events: &BTreeMap<String, DerivedEventSubjectsV1>,
) -> Result<(), ProtocolError> {
    let events = events.iter().collect::<Vec<_>>();
    for (index, (left_name, left)) in events.iter().enumerate() {
        for (right_name, right) in &events[index + 1..] {
            if event_patterns_overlap(&left.wildcard, &right.wildcard) {
                return Err(api_error(
                    "/events",
                    format!(
                        "event '{left_name}' pattern '{}' overlaps event '{right_name}' pattern '{}'",
                        left.wildcard, right.wildcard
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn event_patterns_overlap(left: &str, right: &str) -> bool {
    let left = left.split('.').collect::<Vec<_>>();
    let right = right.split('.').collect::<Vec<_>>();
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| *left == "*" || right == "*" || *left == right)
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_domain(value: &EventClassV1) -> bool {
    *value == EventClassV1::Domain
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs, path::PathBuf};

    use serde::Deserialize;
    use serde_json::json;

    use super::*;
    use crate::schema_profile::{lint_api_authoring, validate_api_meta_schema};

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    struct Vector {
        name: String,
        schema_valid: bool,
        valid: bool,
        input: Value,
        normalized: Option<Value>,
        digest_projection: Option<Value>,
        digest: Option<String>,
        subjects: Option<Value>,
        same_normalized_as: Option<String>,
        same_digest_as: Option<String>,
        different_digest_from: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Fixture {
        vectors: Vec<Vector>,
    }

    #[test]
    fn api_authoring_schema_and_shared_vectors_agree() {
        validate_api_meta_schema().unwrap();
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../conformance/api/vectors.json");
        let fixture: Fixture = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let mut digests = BTreeMap::new();
        let mut normalized_values = BTreeMap::new();

        for vector in &fixture.vectors {
            assert_eq!(
                lint_api_authoring(&vector.input).is_ok(),
                vector.schema_valid,
                "authoring lint result for {}",
                vector.name
            );
            let parsed = parse_api_v1(&vector.input);
            assert_eq!(
                parsed.is_ok(),
                vector.valid,
                "typed result for {}: {parsed:?}",
                vector.name
            );
            assert_eq!(
                serde_json::from_value::<ApiArtifactV1>(vector.input.clone()).is_ok(),
                vector.valid,
                "direct deserialization result for {}",
                vector.name
            );
            let Ok(api) = parsed else {
                continue;
            };
            let normalized = api.normalized_value().unwrap();
            if let Some(expected) = &vector.normalized {
                assert_eq!(&normalized, expected, "{}", vector.name);
                assert_eq!(
                    api.canonical_json().unwrap(),
                    canonicalize_json(expected).unwrap(),
                    "{}",
                    vector.name
                );
            }
            if let Some(expected) = &vector.digest_projection {
                assert_eq!(
                    &api.digest_projection().unwrap(),
                    expected,
                    "{}",
                    vector.name
                );
            }
            let digest = api.digest().unwrap();
            if let Some(expected) = &vector.digest {
                assert_eq!(&digest, expected, "{}", vector.name);
            }
            if let Some(expected) = &vector.subjects {
                assert_eq!(
                    serde_json::to_value(api.derived_subjects().unwrap()).unwrap(),
                    *expected,
                    "{}",
                    vector.name
                );
            }
            normalized_values.insert(vector.name.clone(), normalized);
            digests.insert(vector.name.clone(), digest);
        }

        for vector in &fixture.vectors {
            let Some(digest) = digests.get(&vector.name) else {
                continue;
            };
            if let Some(other) = &vector.same_normalized_as {
                assert_eq!(
                    &normalized_values[&vector.name], &normalized_values[other],
                    "{} and {other}",
                    vector.name
                );
            }
            if let Some(other) = &vector.same_digest_as {
                assert_eq!(digest, &digests[other], "{} and {other}", vector.name);
            }
            if let Some(other) = &vector.different_digest_from {
                assert_ne!(digest, &digests[other], "{} and {other}", vector.name);
            }
        }
    }

    #[test]
    fn api_semantic_errors_remain_api_validation_errors() {
        let value = json!({
            "format": API_FORMAT_V1,
            "id": " documents@v1",
            "displayName": "Documents",
            "description": "Invalid API identifier."
        });
        match parse_api_v1(&value).unwrap_err() {
            ProtocolError::ApiValidation { path, .. } => assert_eq!(path, "/id"),
            error => panic!("expected API validation error, received {error:?}"),
        }
    }

    #[test]
    fn api_authored_nested_keys_are_json_pointer_encoded() {
        let signal = json!({
            "format": API_FORMAT_V1,
            "id": "example@v1",
            "displayName": "Example",
            "description": "Example API.",
            "schemas": { "Any": true },
            "operations": {
                "Op": {
                    "version": "v1",
                    "input": { "schema": "Any" },
                    "signals": {
                        "sig/~": { "input": { "schema": "Missing" } }
                    }
                }
            }
        });
        assert_api_error(&signal, "/operations/Op/signals/sig~1~0/input");

        let accepted_version = json!({
            "format": API_FORMAT_V1,
            "id": "example@v1",
            "displayName": "Example",
            "description": "Example API.",
            "schemas": { "Any": true },
            "state": {
                "S": {
                    "kind": "value",
                    "schema": { "schema": "Any" },
                    "acceptedVersions": {
                        "v/~": { "schema": "Missing" }
                    }
                }
            }
        });
        assert_api_error(&accepted_version, "/state/S/acceptedVersions/v~1~0");
    }

    #[test]
    fn wire_schemas_reject_closed_object_keywords() {
        for (schema, expected_path) in [
            (
                json!({ "type": "object", "additionalProperties": false }),
                "/additionalProperties",
            ),
            (
                json!({ "type": "object", "additionalProperties": { "type": "string" } }),
                "/additionalProperties",
            ),
            (
                json!({ "type": "object", "unevaluatedProperties": false }),
                "/unevaluatedProperties",
            ),
            (
                json!({ "type": "object", "not": { "required": ["futureField"] } }),
                "/not",
            ),
            (
                json!({ "type": "object", "if": { "required": ["futureField"] }, "then": false }),
                "/if",
            ),
            (
                json!({ "type": "object", "dependentSchemas": { "futureField": false } }),
                "/dependentSchemas",
            ),
            (
                json!({ "type": "object", "dependentRequired": { "futureField": ["other"] } }),
                "/dependentRequired",
            ),
            (json!({ "const": { "fixed": true } }), "/const"),
            (json!({ "enum": [{ "fixed": true }] }), "/enum"),
            (
                json!({ "allOf": [true, { "type": "object", "additionalProperties": false }] }),
                "/allOf/1/additionalProperties",
            ),
            (
                json!({
                    "type": "object",
                    "properties": {
                        "nested": { "type": "object", "additionalProperties": false }
                    }
                }),
                "/properties/nested/additionalProperties",
            ),
        ] {
            let value = json!({
                "format": API_FORMAT_V1,
                "id": "example@v1",
                "displayName": "Example",
                "description": "Example API.",
                "schemas": { "Input": schema },
                "rpc": {
                    "Example.Get": {
                        "version": "v1",
                        "input": { "schema": "Input" },
                        "output": { "schema": "Input" }
                    }
                }
            });
            let error = parse_api_v1(&value).expect_err("closed wire schema must fail");
            assert_schema_profile(error, "Input", expected_path);
        }

        let private = json!({
            "format": API_FORMAT_V1,
            "id": "example@v1",
            "displayName": "Example",
            "description": "Example API.",
            "schemas": {
                "Public": true,
                "Private": { "type": "object", "additionalProperties": false }
            },
            "rpc": {
                "Example.Get": {
                    "version": "v1",
                    "input": { "schema": "Public" },
                    "output": { "schema": "Public" }
                }
            }
        });
        assert!(parse_api_v1(&private).is_ok());
    }

    #[test]
    fn every_api_wire_schema_reference_requires_additive_objects() {
        let base = json!({
            "format": API_FORMAT_V1,
            "id": "example@v1",
            "displayName": "Example",
            "description": "Example API.",
            "schemas": {
                "Export": true, "RpcInput": true, "RpcOutput": true, "RpcError": true,
                "OpInput": true, "OpProgress": true, "OpUpdate": true, "OpOutput": true,
                "OpSignal": true, "OpError": true, "Event": true, "FeedInput": true,
                "FeedEvent": true, "State": true, "StateV1": true
            },
            "exports": { "schemas": ["Export"] },
            "errors": {
                "RpcFailure": { "schema": { "schema": "RpcError" } },
                "OpFailure": { "schema": { "schema": "OpError" } }
            },
            "rpc": {
                "Example.Get": {
                    "version": "v1", "input": { "schema": "RpcInput" },
                    "output": { "schema": "RpcOutput" }, "errors": ["RpcFailure"]
                }
            },
            "operations": {
                "Example.Run": {
                    "version": "v1", "input": { "schema": "OpInput" },
                    "progress": { "schema": "OpProgress" }, "update": { "schema": "OpUpdate" },
                    "output": { "schema": "OpOutput" }, "errors": ["OpFailure"],
                    "signals": { "approve": { "input": { "schema": "OpSignal" } } }
                }
            },
            "events": { "Example.Changed": { "version": "v1", "event": { "schema": "Event" } } },
            "feeds": {
                "Example.Watch": {
                    "version": "v1", "input": { "schema": "FeedInput" },
                    "event": { "schema": "FeedEvent" }
                }
            },
            "state": {
                "Settings": {
                    "kind": "value", "schema": { "schema": "State" },
                    "acceptedVersions": { "v1": { "schema": "StateV1" } }
                }
            }
        });
        assert!(parse_api_v1(&base).is_ok());

        for name in [
            "Export",
            "RpcInput",
            "RpcOutput",
            "RpcError",
            "OpInput",
            "OpProgress",
            "OpUpdate",
            "OpOutput",
            "OpSignal",
            "OpError",
            "Event",
            "FeedInput",
            "FeedEvent",
            "State",
            "StateV1",
        ] {
            let mut value = base.clone();
            value["schemas"][name] = json!({ "type": "object", "additionalProperties": false });
            let error = parse_api_v1(&value).expect_err("closed wire schema must fail");
            assert_schema_profile(error, name, "/additionalProperties");
        }
    }

    #[test]
    fn runtime_extensions_do_not_change_api_semantics() {
        let base = json!({
            "format": API_FORMAT_V1,
            "id": "example@v1",
            "displayName": "Example",
            "description": "Example API.",
            "schemas": { "Any": true },
            "rpc": {
                "Example.Get": {
                    "version": "v1",
                    "input": { "schema": "Any" },
                    "output": { "schema": "Any" }
                }
            },
            "capabilities": {
                "read": {
                    "allows": [{
                        "target": {
                            "kind": "apiSurface",
                            "api": "example@v1",
                            "surface": "rpc",
                            "name": "Example.Get"
                        },
                        "action": "call"
                    }]
                }
            },
            "consent": {
                "read": {
                    "title": "Read",
                    "description": "Read records.",
                    "consequence": "Records are visible."
                }
            }
        });
        let mut extended = base.clone();
        extended["extension"] = json!(true);
        extended["rpc"]["Example.Get"]["extension"] = json!(true);
        extended["capabilities"]["read"]["extension"] = json!(true);
        extended["capabilities"]["read"]["allows"][0]["extension"] = json!(true);
        extended["capabilities"]["read"]["allows"][0]["target"]["extension"] = json!(true);
        extended["consent"]["read"]["extension"] = json!(true);

        assert!(lint_api_v1_authoring(&extended).is_err());
        let base = parse_api_v1(&base).unwrap();
        let extended = parse_api_v1(&extended).unwrap();
        assert_eq!(
            extended.normalized_value().unwrap(),
            base.normalized_value().unwrap()
        );
        assert_eq!(extended.digest().unwrap(), base.digest().unwrap());
    }

    fn assert_api_error(value: &Value, expected_path: &str) {
        match parse_api_v1(value).unwrap_err() {
            ProtocolError::ApiValidation { path, .. } => assert_eq!(path, expected_path),
            error => panic!("expected API validation error, received {error:?}"),
        }
    }

    fn assert_schema_profile(error: ProtocolError, expected_schema: &str, expected_path: &str) {
        let ProtocolError::SchemaProfile { schema, path, .. } = error else {
            panic!("expected schema profile error, got {error:?}")
        };
        assert_eq!(schema, expected_schema);
        assert_eq!(path, expected_path);
    }
}
