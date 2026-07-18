use std::collections::{BTreeMap, BTreeSet};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use crate::{
    canonicalize_json, digest_json,
    identifiers::{
        api_error, sort_deduplicate, validate_api_id, validate_logical_name,
        validate_nonempty_text, validate_protocol_identifier, validate_version,
    },
    schema_profile::{validate_api_structure, validate_embedded_schema},
    subjects::{
        derive_event_subject, derive_event_wildcard_subject, derive_feed_subject,
        derive_operation_subject, derive_rpc_subject, DerivedApiSubjectsV1, DerivedEventSubjectsV1,
    },
    ApiSurfaceKindV1, CapabilityDefinitionV1, ConsentMetadataV1, PermissionActionV1, ProtocolError,
};

/// The first canonical Trellis API artifact format.
pub const API_FORMAT_V1: &str = "trellis.api.v1";

/// Draft 2020-12 authoring schema for `trellis.api.v1` artifacts.
pub const API_SCHEMA_V1_JSON: &str = include_str!("../schemas/trellis.api.v1.schema.json");

/// One validated, normalized `trellis.api.v1` artifact.
///
/// The `lineage@vN` identifier is the API-level identity. Surface-local
/// versions independently control derived NATS subjects. Human-facing text is
/// retained in normalized values but omitted from [`Self::digest_projection`].
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
/// This version applies the closed authoring schema as part of parsing. It also
/// validates identifiers, references, surface invariants, and embedded schemas.
///
/// # Errors
///
/// Returns [`ProtocolError::ApiValidation`] with an RFC 6901 path when the
/// artifact or an embedded schema violates the API profile, or
/// [`ProtocolError::Json`] when the value cannot be decoded.
pub fn parse_api_v1(value: &Value) -> Result<ApiArtifactV1, ProtocolError> {
    validate_api_structure(value)?;
    let wire: WireApiArtifactV1 =
        serde_json::from_value(value.clone()).map_err(|error| api_error("", error.to_string()))?;
    ApiArtifactV1::from_wire(wire)
}

impl ApiArtifactV1 {
    /// Return the stable versioned API lineage identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Serialize the normalized supported API shape, including human fields.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Json`] if the validated artifact cannot be
    /// represented as a JSON value.
    pub fn normalized_value(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::to_value(self)?)
    }

    /// Render the complete normalized API artifact as RFC 8785 JSON.
    ///
    /// # Errors
    ///
    /// Returns a serialization or canonicalization [`ProtocolError`].
    pub fn canonical_json(&self) -> Result<String, ProtocolError> {
        canonicalize_json(&self.normalized_value()?)
    }

    /// Build the machine-visible API identity projection.
    ///
    /// Display names, descriptions, documentation, consent wording, and
    /// surface documentation are excluded. Machine schemas, exports, errors,
    /// surfaces, versions, transfer behavior, and capability permissions are
    /// included.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Json`] if a supported machine field cannot be
    /// represented as JSON.
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
    ///
    /// # Errors
    ///
    /// Returns a serialization or canonicalization [`ProtocolError`].
    pub fn digest(&self) -> Result<String, ProtocolError> {
        digest_json(&self.digest_projection()?)
    }

    /// Derive every public communication subject owned by this API.
    ///
    /// RPCs use `rpc`, operations use `operations`, events use `events`, and
    /// feeds use `feed`. Event wildcard subjects append one `*` token for each
    /// authored event parameter.
    ///
    /// # Errors
    ///
    /// Returns an API validation error if a stored version or logical name
    /// cannot be converted to a canonical NATS subject.
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
                let signal_path = format!("{path}/signals/{signal}");
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
                validate_protocol_identifier(
                    &format!("{path}/acceptedVersions/{version}"),
                    version,
                    api_error,
                )?;
                require_schema(
                    &wire.schemas,
                    reference,
                    &format!("{path}/acceptedVersions/{version}"),
                )?;
            }
            if let Some(docs) = &definition.docs {
                validate_docs(&format!("{path}/docs"), docs)?;
            }
        }

        for (capability_name, capability) in &wire.capabilities {
            let path = member_path("capabilities", capability_name);
            validate_protocol_identifier(&path, capability_name, api_error)?;
            for (index, atom) in capability.allows().iter().enumerate() {
                let atom_path = format!("{path}/allows/{index}");
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
                    if atom.action() == PermissionActionV1::Control && operation.signals.is_empty()
                    {
                        return Err(api_error(
                            atom_path,
                            "control permission requires at least one operation signal",
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
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
#[serde(deny_unknown_fields)]
struct DocumentationV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    markdown: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
struct TransferDefinitionV1 {
    direction: TransferDirectionV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ErrorDefinitionV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<SchemaReferenceV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
struct OperationSignalV1 {
    input: SchemaReferenceV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
    format!("/{section}/{}", name.replace('~', "~0").replace('/', "~1"))
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
    use crate::schema_profile::{validate_api_meta_schema, validate_api_structure};

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
    fn api_meta_schema_and_shared_vectors_agree() {
        validate_api_meta_schema().unwrap();
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../conformance/api/vectors.json");
        let fixture: Fixture = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let mut digests = BTreeMap::new();
        let mut normalized_values = BTreeMap::new();

        for vector in &fixture.vectors {
            assert_eq!(
                validate_api_structure(&vector.input).is_ok(),
                vector.schema_valid,
                "meta-schema result for {}",
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
}
