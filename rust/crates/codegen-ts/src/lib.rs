//! TypeScript SDK generation from canonical Trellis contract manifests.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use serde_json::Value;
use trellis_protocol::{ApiArtifact, ParticipantArtifact};

mod projection;
use projection::{ApiInput, ApiProjection};

/// Errors returned while generating a TypeScript SDK package.
#[derive(thiserror::Error, Debug)]
pub enum CodegenTsError {
    #[error("protocol error: {0}")]
    Protocol(#[from] trellis_protocol::ProtocolError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid generated TypeScript in {path}: {message}")]
    InvalidTypeScript { path: PathBuf, message: String },

    #[error("generated TypeScript export name collision: {0}")]
    ExportNameCollision(String),

    #[error("generated TypeScript output path must stay within the package: {0}")]
    InvalidOutputPath(PathBuf),

    #[error("missing generated reference: {0}")]
    MissingReference(String),
}

fn project_api(api: &ApiArtifact) -> Result<ApiInput, CodegenTsError> {
    let value = api.normalized_value()?;
    let mut render_model = serde_json::from_value::<ApiProjection>(value.clone())?;
    for (name, error) in &mut render_model.errors {
        error.error_type.clone_from(name);
    }
    Ok(ApiInput {
        render_model,
        subjects: api.derived_subjects()?,
        digest: api.digest()?,
        value,
    })
}

/// One generated TypeScript SDK source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTsSource {
    pub path: PathBuf,
    pub contents: String,
}

/// Render one ordinary ESM package into an empty caller-owned staging directory.
/// Publish the directory only after this function succeeds.
pub fn generate_ts_package(
    apis: &BTreeMap<&str, &ApiArtifact>,
    participants: &[ParticipantArtifact],
    out_dir: &Path,
    name: &str,
) -> Result<(), CodegenTsError> {
    let mut sources = vec![GeneratedTsSource {
        path: "package.json".into(),
        contents: serde_json::to_string_pretty(&serde_json::json!({
            "name": name,
            "version": "0.0.0",
            "private": true,
            "trellisGenerated": true,
            "description": "Generated Trellis APIs and participants.",
            "type": "module",
            "exports": {".": {"types": "./index.d.ts", "import": "./index.js"}},
            "dependencies": {"@qlever-llc/trellis": format!("^{}", env!("CARGO_PKG_VERSION"))}
        }))?,
    }];
    for (namespace, ids) in [
        ("apis", apis.keys().copied().collect::<Vec<_>>()),
        (
            "participants",
            participants.iter().map(ParticipantArtifact::id).collect(),
        ),
    ] {
        let mut names = BTreeMap::new();
        let mut exports = String::new();
        for id in ids {
            let stem = sdk_output_stem(id);
            let module = lower_camel_ident(&stem);
            if let Some(previous) = names.insert(module.clone(), id) {
                return Err(CodegenTsError::ExportNameCollision(format!(
                    "{namespace}/{module}: '{previous}' and '{id}'"
                )));
            }
            exports.push_str(&format!("export * as {module} from \"./{stem}/mod.ts\";\n"));
        }
        if exports.is_empty() {
            exports.push_str("export {};\n");
        }
        sources.push(GeneratedTsSource {
            path: PathBuf::from(namespace).join("index.ts"),
            contents: exports,
        });
    }
    sources.push(GeneratedTsSource {
        path: "index.ts".into(),
        contents: "export * as apis from \"./apis/index.ts\";\nexport * as participants from \"./participants/index.ts\";\n".into(),
    });
    for (id, api) in apis {
        let directory = PathBuf::from("apis").join(sdk_output_stem(id));
        for mut source in collect_ts_sdk_sources(api)? {
            source.path = directory.join(source.path);
            sources.push(source);
        }
    }
    for participant in participants {
        let value = participant.normalized_value()?;
        let owned = value["implements"]
            .as_object()
            .and_then(|entries| entries.values().next())
            .and_then(|entry| entry["api"].as_str())
            .and_then(|id| apis.get(id))
            .ok_or_else(|| {
                CodegenTsError::ExportNameCollision(format!(
                    "participant '{}' has no available implemented API",
                    participant.id()
                ))
            })?;
        sources.push(GeneratedTsSource {
            path: PathBuf::from("participants")
                .join(sdk_output_stem(participant.id()))
                .join("mod.ts"),
            contents: render_ts_participant(participant, owned, apis)?,
        });
    }
    let mut emitted = Vec::new();
    for source in sources {
        if source
            .path
            .extension()
            .is_some_and(|extension| extension == "ts")
        {
            emitted.extend(emit_typescript(&source)?);
        } else {
            emitted.push(source);
        }
    }
    write_ts_sdk_sources(out_dir, &emitted)
}

/// Validate source paths and write a package into a caller-owned staging directory.
pub fn write_ts_sdk_sources(
    out_dir: &Path,
    sources: &[GeneratedTsSource],
) -> Result<(), CodegenTsError> {
    for source in sources {
        if source.path.is_absolute()
            || source
                .path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(CodegenTsError::InvalidOutputPath(source.path.clone()));
        }
    }
    for source in sources {
        write_generated_file(&out_dir.join(&source.path), &source.contents)?;
    }

    Ok(())
}

fn render_ts_participant(
    participant: &ParticipantArtifact,
    owned: &ApiArtifact,
    referenced: &BTreeMap<&str, &ApiArtifact>,
) -> Result<String, CodegenTsError> {
    let participant_value = participant.normalized_value()?;
    let participant_digest = participant.digest()?;
    let referenced_ids = participant_value["implements"]
        .as_object()
        .into_iter()
        .chain(participant_value["uses"]["required"].as_object())
        .chain(participant_value["uses"]["optional"].as_object())
        .flat_map(|entries| entries.values())
        .filter_map(|entry| entry["api"].as_str())
        .collect::<BTreeSet<_>>();
    let referenced = referenced
        .values()
        .copied()
        .filter(|api| api.id() != owned.id() && referenced_ids.contains(api.id()))
        .map(project_api)
        .collect::<Result<Vec<_>, _>>()?;
    let owned = project_api(owned)?;
    let mut loaded_apis = vec![&owned];
    let mut apis = BTreeMap::from([(owned.render_model.id.clone(), owned.value.clone())]);
    for api in &referenced {
        if referenced_ids.contains(api.render_model.id.as_str()) {
            apis.insert(api.render_model.id.clone(), api.value.clone());
            loaded_apis.push(api);
        }
    }
    let aliases = apis
        .keys()
        .enumerate()
        .map(|(index, id)| (id.clone(), format!("Api{index}")))
        .collect::<BTreeMap<_, _>>();
    let metadata_aliases = loaded_apis
        .iter()
        .flat_map(|api| {
            let alias = &aliases[&api.render_model.id];
            public_schema_type_aliases(api, &public_schema_exports(api))
                .into_iter()
                .map(move |schema| SchemaTypeAlias {
                    key: schema.key,
                    type_name: format!("{alias}.{}", schema.type_name),
                    schema: schema.schema,
                })
        })
        .collect::<Vec<_>>();
    let owned_alias = &aliases[&owned.render_model.id];
    let mut lines = vec![
        "// Generated from canonical Trellis participant artifacts.".to_owned(),
        "import {".to_owned(),
        "  PARTICIPANT_EVENT_CONSUMERS_METADATA,".to_owned(),
        "  PARTICIPANT_JOBS_METADATA,".to_owned(),
        "  PARTICIPANT_KV_METADATA,".to_owned(),
        "  PARTICIPANT_RUNTIME,".to_owned(),
        "  PARTICIPANT_STATE_METADATA,".to_owned(),
        "  PARTICIPANT_STORE_METADATA,".to_owned(),
        "  runtimeApiFromActions,".to_owned(),
        "} from \"@qlever-llc/trellis/generated\";".to_owned(),
    ];
    for (id, alias) in &aliases {
        lines.push(format!(
            "import * as {alias} from \"../../apis/{}/mod.ts\";",
            sdk_output_stem(id)
        ));
    }
    lines.push("function typeOnly<T>(): T { return undefined as T; }".to_owned());

    let owned_actions = api_action_expressions(&owned.value, owned_alias);
    let (required_actions, optional_actions) =
        selected_action_expressions(&participant_value, &aliases, &apis)?;
    let all_actions = owned_actions
        .iter()
        .chain(&required_actions)
        .chain(&optional_actions)
        .cloned()
        .collect::<BTreeSet<_>>();
    for (name, actions) in [
        ("owned", owned_actions.clone()),
        (
            "used",
            required_actions.union(&optional_actions).cloned().collect(),
        ),
        ("all", all_actions),
    ] {
        lines.push(format!(
            "const __{name}Actions = [{}] as const;",
            actions
                .iter()
                .map(|action| format!("{action} as typeof {action}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    lines.push(String::new());
    lines.push("const __participant = {".to_owned());
    lines.push(format!("  id: {},", js_string(participant.id())));
    lines.push(format!("  digest: {},", js_string(&participant_digest)));
    lines.push(format!(
        "  artifact: {} as const,",
        serde_json::to_string(&participant_value)?
    ));
    lines.push(format!(
        "  api: {owned_alias}.API as typeof {owned_alias}.API,"
    ));
    lines.push(format!(
        "  apiDigest: {owned_alias}.API_DIGEST as typeof {owned_alias}.API_DIGEST,"
    ));
    lines.push(format!(
        "  referencedApis: [{}] as const,",
        aliases
            .iter()
            .filter(|(id, _)| *id != &owned.render_model.id)
            .map(|(_, alias)| format!("{alias}.API as typeof {alias}.API"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    for expression in owned_surface_entries(&owned.value, owned_alias) {
        lines.push(format!("  {expression},"));
    }
    lines.push("} as const;".to_owned());
    lines.push("const __metadata = {".to_owned());
    lines.push("  PARTICIPANT_RUNTIME: {".to_owned());
    lines.push(format!(
        "    ownedApi: runtimeApiFromActions(__ownedActions, {}) as ReturnType<typeof runtimeApiFromActions<typeof __ownedActions>> ,",
        serde_json::to_string(
            &participant_value["implements"]["self"]
                .get("operationTransfers")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}))
        )?
    ));
    lines.push("    usedApi: runtimeApiFromActions(__usedActions) as ReturnType<typeof runtimeApiFromActions<typeof __usedActions>> ,".to_owned());
    lines.push(format!(
        "    api: runtimeApiFromActions(__allActions, {}) as ReturnType<typeof runtimeApiFromActions<typeof __allActions>> ,",
        serde_json::to_string(
            &participant_value["implements"]["self"]
                .get("operationTransfers")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}))
        )?
    ));
    lines.push(format!(
        "    actions: [{}],",
        owned_actions
            .iter()
            .chain(&required_actions)
            .map(|action| format!("{{ action: {action} as typeof {action}, optional: false }}"))
            .chain(
                optional_actions.iter().map(|action| format!(
                    "{{ action: {action} as typeof {action}, optional: true }}"
                ))
            )
            .collect::<Vec<_>>()
            .join(", ")
    ));
    lines.push("  },".to_owned());
    let mut metadata_symbols = vec!["PARTICIPANT_RUNTIME"];
    metadata_symbols.extend(insert_participant_metadata(
        &mut lines,
        &participant_value,
        &metadata_aliases,
    )?);
    lines.push("} as const;".to_owned());
    lines.push("export const participant: typeof __participant & {".to_owned());
    for symbol in &metadata_symbols {
        lines.push(format!(
            "  readonly [{symbol}]: typeof __metadata.{symbol};"
        ));
    }
    lines.push("} = { ...__participant,".to_owned());
    for symbol in &metadata_symbols {
        lines.push(format!("  [{symbol}]: __metadata.{symbol},"));
    }
    lines.extend([
        "};".to_owned(),
        String::new(),
        "export default participant;".to_owned(),
        String::new(),
    ]);
    Ok(lines.join("\n"))
}

fn sdk_output_stem(id: &str) -> String {
    let stem = id.split('@').next().unwrap_or(id).replace('.', "-");
    stem.strip_prefix("trellis-").unwrap_or(&stem).to_owned()
}

fn api_action_expressions(api: &Value, alias: &str) -> BTreeSet<String> {
    let mut actions = BTreeSet::new();
    for section in ["rpc", "operations", "feeds"] {
        for name in api[section]
            .as_object()
            .into_iter()
            .flat_map(|value| value.keys())
        {
            actions.insert(format!("{alias}.ACTIONS[{}]", js_string(name)));
        }
    }
    for name in api["events"]
        .as_object()
        .into_iter()
        .flat_map(|value| value.keys())
    {
        let descriptor = format!("{alias}.ACTIONS[{}]", js_string(name));
        actions.insert(format!("{descriptor}[\"publish\"]"));
        actions.insert(format!("{descriptor}[\"subscribe\"]"));
    }
    actions
}

fn owned_surface_entries(api: &Value, alias: &str) -> BTreeSet<String> {
    ["rpc", "operations", "events", "feeds"]
        .into_iter()
        .flat_map(|section| {
            api[section]
                .as_object()
                .into_iter()
                .flat_map(|value| value.keys())
        })
        .map(|name| {
            let symbol = key_to_pascal(name);
            format!(
                "{symbol}: {alias}.ACTIONS[{0}] as typeof {alias}.ACTIONS[{0}]",
                js_string(name)
            )
        })
        .collect()
}

fn selected_action_expressions(
    participant: &Value,
    aliases: &BTreeMap<String, String>,
    apis: &BTreeMap<String, Value>,
) -> Result<(BTreeSet<String>, BTreeSet<String>), CodegenTsError> {
    let mut required = BTreeSet::new();
    let mut optional = BTreeSet::new();
    for (category, selected) in [("required", &mut required), ("optional", &mut optional)] {
        for used in participant["uses"][category]
            .as_object()
            .into_iter()
            .flat_map(|value| value.values())
        {
            let api_id = used["api"].as_str().expect("validated API use");
            let alias = aliases
                .get(api_id)
                .ok_or_else(|| CodegenTsError::MissingReference(format!("API '{api_id}'")))?;
            let api = apis
                .get(api_id)
                .ok_or_else(|| CodegenTsError::MissingReference(format!("API '{api_id}'")))?;
            for name in used["operations"]["control"]
                .as_object()
                .into_iter()
                .flat_map(|entries| entries.keys())
            {
                selected.insert(format!(
                    "{alias}.ACTIONS[{}]",
                    js_string(descriptor_name(api, "operations", name)?)
                ));
            }
            for (section, suffix) in [("rpc", ""), ("operations", ""), ("feeds", "")] {
                for actions in used[section]
                    .as_object()
                    .into_iter()
                    .flat_map(|value| value.values())
                {
                    for name in actions
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                    {
                        selected.insert(format!(
                            "{alias}.ACTIONS[{}]{suffix}",
                            js_string(descriptor_name(api, section, name)?)
                        ));
                    }
                }
            }
            for (action, names) in used["events"]
                .as_object()
                .into_iter()
                .flat_map(|value| value.iter())
            {
                for name in names
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    selected.insert(format!(
                        "{alias}.ACTIONS[{}][\"{action}\"]",
                        js_string(descriptor_name(api, "events", name)?)
                    ));
                }
            }
        }
    }
    Ok((required, optional))
}

fn descriptor_name<'a>(
    api: &Value,
    section: &str,
    selected: &'a str,
) -> Result<&'a str, CodegenTsError> {
    api[section]
        .as_object()
        .filter(|entries| entries.contains_key(selected))
        .map(|_| selected)
        .ok_or_else(|| {
            CodegenTsError::MissingReference(format!("{} {section}.{selected}", api["id"]))
        })
}

fn insert_participant_metadata(
    lines: &mut Vec<String>,
    participant: &Value,
    aliases: &[SchemaTypeAlias],
) -> Result<Vec<&'static str>, CodegenTsError> {
    let schemas = participant["schemas"].as_object();
    let mut symbols = Vec::new();
    for (section, symbol) in [
        ("state", "PARTICIPANT_STATE_METADATA"),
        ("jobQueues", "PARTICIPANT_JOBS_METADATA"),
    ] {
        let Some(entries) = participant[section].as_object() else {
            continue;
        };
        symbols.push(symbol);
        lines.push(format!("  {symbol}: {{"));
        for (name, entry) in entries {
            let schema_name = entry["schema"]["schema"]
                .as_str()
                .or_else(|| entry["payload"]["schema"].as_str())
                .expect("validated participant schema reference");
            let schemas = schemas.ok_or_else(|| {
                CodegenTsError::MissingReference(format!("participant schema '{schema_name}'"))
            })?;
            let schema = resolve_participant_schema(schemas, schema_name)?;
            let ty = participant_schema_type(schema_name, schema, aliases);
            if section == "state" {
                lines.push(format!("    {}: {{ kind: {}, value: typeOnly<{ty}>() as {ty}, schema: {} as const, stateVersion: {}, acceptedVersions: {{}} }},", js_string(name), js_string(entry["kind"].as_str().expect("state kind")), serde_json::to_string(schema)?, js_string(entry["stateVersion"].as_str().unwrap_or("v1"))));
            } else {
                lines.push(format!(
                    "    {}: {{ payload: typeOnly<{ty}>() as {ty}, result: typeOnly<unknown>() as unknown }},",
                    js_string(name)
                ));
            }
        }
        lines.push("  },".to_owned());
    }
    if let Some(resources) = participant["resources"].as_object() {
        for (section, symbol) in [
            ("kv", "PARTICIPANT_KV_METADATA"),
            ("store", "PARTICIPANT_STORE_METADATA"),
        ] {
            let Some(entries) = resources.get(section).and_then(Value::as_object) else {
                continue;
            };
            symbols.push(symbol);
            lines.push(format!("  {symbol}: {{"));
            for (name, entry) in entries {
                if section == "kv" {
                    let schema_name = entry["schema"]["schema"].as_str().expect("KV schema");
                    let schemas = schemas.ok_or_else(|| {
                        CodegenTsError::MissingReference(format!(
                            "participant schema '{schema_name}'"
                        ))
                    })?;
                    let schema = resolve_participant_schema(schemas, schema_name)?;
                    let ty = participant_schema_type(schema_name, schema, aliases);
                    lines.push(format!("    {}: {{ required: true, value: typeOnly<{ty}>() as {ty}, schema: {} as const }},", js_string(name), serde_json::to_string(schema)?));
                } else {
                    lines.push(format!("    {}: {{ required: true }},", js_string(name)));
                }
            }
            lines.push("  },".to_owned());
        }
    }
    if let Some(consumers) = participant["eventConsumers"].as_object() {
        let consumers = consumers
            .iter()
            .map(|(name, consumer)| {
                let mut consumer = consumer
                    .as_object()
                    .expect("validated event consumer")
                    .clone();
                let mut events = consumer
                    .remove("events")
                    .expect("validated consumer events")
                    .as_object()
                    .expect("consumer events map")
                    .clone();
                if let Some(events) = events.remove("self") {
                    consumer.insert("self".to_owned(), events);
                }
                if !events.is_empty() {
                    consumer.insert("uses".to_owned(), Value::Object(events));
                }
                (name.clone(), Value::Object(consumer))
            })
            .collect::<serde_json::Map<String, Value>>();
        symbols.push("PARTICIPANT_EVENT_CONSUMERS_METADATA");
        lines.push(format!(
            "  PARTICIPANT_EVENT_CONSUMERS_METADATA: {} as const,",
            serde_json::to_string(&consumers)?
        ));
    }
    Ok(symbols)
}

fn resolve_participant_schema<'a>(
    schemas: &'a serde_json::Map<String, Value>,
    name: &str,
) -> Result<&'a Value, CodegenTsError> {
    let mut name = name;
    let mut visited = BTreeSet::new();
    loop {
        if !visited.insert(name) {
            return Err(CodegenTsError::MissingReference(format!(
                "cyclic participant schema '{name}'"
            )));
        }
        let schema = schemas.get(name).ok_or_else(|| {
            CodegenTsError::MissingReference(format!("participant schema '{name}'"))
        })?;
        match schema.get("$ref").and_then(Value::as_str) {
            Some(reference) => {
                name = reference
                    .strip_prefix("#/schemas/")
                    .ok_or_else(|| CodegenTsError::MissingReference(reference.to_owned()))?
            }
            None => return Ok(schema),
        }
    }
}

fn participant_schema_type(name: &str, schema: &Value, aliases: &[SchemaTypeAlias]) -> String {
    let mut matches = aliases
        .iter()
        .filter(|alias| name == alias.key && *schema == alias.schema);
    match (matches.next(), matches.next()) {
        (Some(alias), None) => alias.type_name.clone(),
        _ => schema_to_ts_with_aliases(schema, &[], None),
    }
}

/// Render the source modules for one API without writing files or package metadata.
pub fn collect_ts_sdk_sources(api: &ApiArtifact) -> Result<Vec<GeneratedTsSource>, CodegenTsError> {
    let loaded = project_api(api)?;
    validate_public_export_names(&loaded)?;
    Ok(vec![
        GeneratedTsSource {
            path: PathBuf::from("descriptors.ts"),
            contents: render_descriptors_ts(&loaded),
        },
        GeneratedTsSource {
            path: PathBuf::from("types.ts"),
            contents: render_wire_types_ts(&loaded),
        },
        GeneratedTsSource {
            path: PathBuf::from("schemas.ts"),
            contents: render_schemas_ts(&loaded),
        },
        GeneratedTsSource {
            path: PathBuf::from("api.ts"),
            contents: render_api_ts(&loaded)?,
        },
        GeneratedTsSource {
            path: PathBuf::from("mod.ts"),
            contents: render_mod_ts(),
        },
        GeneratedTsSource {
            path: PathBuf::from("TRELLIS.md"),
            contents: render_trellis_md(&loaded),
        },
    ])
}

#[derive(Debug, Clone)]
struct PublicSchemaExport {
    key: String,
    const_name: String,
    type_name: Option<String>,
}

#[derive(Debug, Clone)]
struct SchemaTypeAlias {
    key: String,
    type_name: String,
    schema: Value,
}

fn render_api_ts(loaded: &ApiInput) -> Result<String, CodegenTsError> {
    let source_reference = &loaded.render_model.id;
    Ok(format!(
        "// Generated from {}\n\nexport const API_ID = {} as const;\nexport const API_DIGEST = {} as const;\nexport const API = {} as const;\n",
        escape_js_string(source_reference),
        js_string(&loaded.render_model.id),
        js_string(&loaded.digest),
        serde_json::to_string(&loaded.value)?,
    ))
}

fn render_wire_types_ts(loaded: &ApiInput) -> String {
    let source_reference = &loaded.render_model.id;
    let public_schema_exports = public_schema_exports(loaded);
    let schema_type_aliases = public_schema_type_aliases(loaded, &public_schema_exports);
    let schema_const_names = public_schema_exports
        .iter()
        .map(|export| (export.key.as_str(), export.const_name.as_str()))
        .collect::<BTreeMap<_, _>>();
    let error_schema_imports = loaded
        .render_model
        .errors
        .values()
        .filter_map(|error| error.schema.as_ref())
        .map(|schema| {
            schema_const_names
                .get(schema.schema.as_str())
                .expect("missing public schema export for error schema")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let mut lines = vec![format!(
        "// Generated from {}",
        escape_js_string(source_reference)
    )];

    if !loaded.render_model.errors.is_empty() {
        lines.extend([
            format!(
                "import type {{ SerializableErrorData }} from {};",
                js_string("@qlever-llc/trellis/generated")
            ),
            format!(
                "import {{ TrellisError }} from {};",
                js_string("@qlever-llc/trellis/generated")
            ),
        ]);
    }
    if !error_schema_imports.is_empty() {
        lines.push(format!(
            "import {{ {} }} from \"./schemas.ts\";",
            error_schema_imports
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    lines.push(String::new());

    for export in &public_schema_exports {
        if let Some(type_name) = &export.type_name {
            lines.push(format!(
                "export type {type_name} = {};",
                schema_to_ts_with_aliases(
                    resolve_schema_ref(loaded, &export.key),
                    &schema_type_aliases,
                    Some(&export.key),
                )
            ));
            lines.push(String::new());
        }
    }

    for (key, rpc) in &loaded.render_model.rpc {
        let base = key_to_pascal(key);
        lines.push(format!(
            "export type {base}Input = {};",
            schema_to_ts_with_aliases(
                resolve_schema_ref(loaded, &rpc.input.schema),
                &schema_type_aliases,
                None,
            )
        ));
        lines.push(format!(
            "export type {base}Output = {};",
            schema_to_ts_with_aliases(
                resolve_schema_ref(loaded, &rpc.output.schema),
                &schema_type_aliases,
                None,
            )
        ));
        lines.push(String::new());
    }

    for (key, operation) in &loaded.render_model.operations {
        let base = key_to_pascal(key);
        lines.push(format!(
            "export type {base}Input = {};",
            schema_to_ts_with_aliases(
                resolve_schema_ref(loaded, &operation.input.schema),
                &schema_type_aliases,
                None,
            )
        ));
        for (suffix, schema_ref) in [
            ("Progress", operation.progress.as_ref()),
            ("Update", operation.update.as_ref()),
            ("Output", operation.output.as_ref()),
        ] {
            if let Some(schema_ref) = schema_ref {
                lines.push(format!(
                    "export type {base}{suffix} = {};",
                    schema_to_ts_with_aliases(
                        resolve_schema_ref(loaded, &schema_ref.schema),
                        &schema_type_aliases,
                        None,
                    )
                ));
            }
        }
        for (signal_name, signal) in &operation.signals {
            lines.push(format!(
                "export type {base}{}Signal = {};",
                key_to_pascal(signal_name),
                schema_to_ts_with_aliases(
                    resolve_schema_ref(loaded, &signal.input.schema),
                    &schema_type_aliases,
                    None,
                )
            ));
        }
        lines.push(String::new());
    }

    for (key, event) in &loaded.render_model.events {
        let base = key_to_pascal(key);
        lines.push(format!(
            "export type {base}Event = {};",
            schema_to_ts_with_aliases(
                resolve_schema_ref(loaded, &event.event.schema),
                &schema_type_aliases,
                None,
            )
        ));
        lines.push(String::new());
    }

    for (key, feed) in &loaded.render_model.feeds {
        let base = key_to_pascal(key);
        lines.push(format!(
            "export type {base}Input = {};",
            schema_to_ts_with_aliases(
                resolve_schema_ref(loaded, &feed.input.schema),
                &schema_type_aliases,
                None,
            )
        ));
        lines.push(format!(
            "export type {base}Event = {};",
            schema_to_ts_with_aliases(
                resolve_schema_ref(loaded, &feed.event.schema),
                &schema_type_aliases,
                None,
            )
        ));
        lines.push(String::new());
    }

    for error in loaded.render_model.errors.values() {
        let base = key_to_pascal(&error.error_type);
        let data_type = format!("{base}Data");
        let ts_type = error
            .schema
            .as_ref()
            .map(|schema| {
                format!(
                    "SerializableErrorData & ({})",
                    schema_to_ts_with_aliases(
                        resolve_schema_ref(loaded, &schema.schema),
                        &schema_type_aliases,
                        None,
                    )
                )
            })
            .unwrap_or_else(|| "SerializableErrorData".to_string());
        lines.push(format!("export type {data_type} = {ts_type};"));
        lines.push(format!(
            "export class {base} extends TrellisError<{data_type}> {{"
        ));
        if let Some(schema) = &error.schema {
            lines.push(format!(
                "  static readonly schema: typeof {0} = {0};",
                schema_const_names
                    .get(schema.schema.as_str())
                    .expect("missing public schema export for error schema")
            ));
        }
        lines.push(format!(
            "  override readonly name = {} as const;",
            js_string(&error.error_type)
        ));
        lines.push(format!("  readonly data: {data_type};"));
        lines.push(format!("  constructor(data: {data_type}) {{"));
        lines.push("    super(data.message, {".to_string());
        lines.push("      id: data.id,".to_string());
        lines.push(
            "      ...(data.context !== undefined ? { context: data.context } : {}),".to_string(),
        );
        lines.push("    });".to_string());
        lines.push("    this.data = data;".to_string());
        lines.push("  }".to_string());
        lines.push(format!(
            "  static fromSerializable(data: {data_type}): {base} {{"
        ));
        lines.push(format!("    return new {base}(data);"));
        lines.push("  }".to_string());
        lines.push(format!("  override toSerializable(): {data_type} {{"));
        lines.push("    return this.data;".to_string());
        lines.push("  }".to_string());
        lines.push("}".to_string());
        lines.push(String::new());
    }

    if !lines.iter().any(|line| line.starts_with("export ")) {
        lines.push("export {};".to_owned());
    }
    format!("{}\n", lines.join("\n"))
}

fn render_schemas_ts(loaded: &ApiInput) -> String {
    let source_reference = &loaded.render_model.id;
    let public_schema_exports = public_schema_exports(loaded);
    let mut lines = vec![format!(
        "// Generated from {}",
        escape_js_string(source_reference)
    )];

    for export in public_schema_exports {
        lines.push(format!(
            "export const {} = {} as const;",
            export.const_name,
            serde_json::to_string(resolve_schema_ref(loaded, &export.key)).unwrap()
        ));
        lines.push(String::new());
    }

    if !lines.iter().any(|line| line.starts_with("export ")) {
        lines.push("export {};".to_owned());
    }
    format!(
        "{}
",
        lines.join(
            "
"
        )
    )
}

fn render_descriptors_ts(loaded: &ApiInput) -> String {
    let source_reference = &loaded.render_model.id;
    let trellis_runtime_import = "@qlever-llc/trellis/generated";
    let public_schema_exports = public_schema_exports(loaded);
    let schema_const_names = public_schema_exports
        .iter()
        .map(|export| (export.key.as_str(), export.const_name.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut api_schema_imports = BTreeSet::new();
    for rpc in loaded.render_model.rpc.values() {
        api_schema_imports.insert(rpc.input.schema.as_str());
        api_schema_imports.insert(rpc.output.schema.as_str());
    }
    for operation in loaded.render_model.operations.values() {
        api_schema_imports.insert(operation.input.schema.as_str());
        if let Some(progress) = &operation.progress {
            api_schema_imports.insert(progress.schema.as_str());
        }
        if let Some(update) = &operation.update {
            api_schema_imports.insert(update.schema.as_str());
        }
        if let Some(output) = &operation.output {
            api_schema_imports.insert(output.schema.as_str());
        }
        for signal in operation.signals.values() {
            api_schema_imports.insert(signal.input.schema.as_str());
        }
    }
    for event in loaded.render_model.events.values() {
        api_schema_imports.insert(event.event.schema.as_str());
    }
    for feed in loaded.render_model.feeds.values() {
        api_schema_imports.insert(feed.input.schema.as_str());
        api_schema_imports.insert(feed.event.schema.as_str());
    }
    for error in loaded.render_model.errors.values() {
        if let Some(schema) = &error.schema {
            api_schema_imports.insert(schema.schema.as_str());
        }
    }
    let uses_types_as_value = !loaded.render_model.errors.is_empty();
    let owner_id = "API_ID";
    let source_export = "API";
    let digest_export = "API_DIGEST";
    let mut lines = vec![
        format!("// Generated from {}", escape_js_string(source_reference)),
        format!(
            "import {{ eventActions, feedAction, operationAction, rpcAction, schema }} from {};",
            js_string(trellis_runtime_import)
        ),
        if uses_types_as_value {
            "import * as Types from \"./types.ts\";".to_string()
        } else {
            "import type * as Types from \"./types.ts\";".to_string()
        },
        format!(
            "import {{ {source_export} as ACTION_ARTIFACT, {digest_export} as ACTION_DIGEST }} from \"./api.ts\";"
        ),
        String::new(),
        "const ACTION_SOURCE = { api: ACTION_ARTIFACT, apiDigest: ACTION_DIGEST } as const;"
            .to_string(),
        String::new(),
        format!(
            "const {owner_id} = {} as const;",
            js_string(&loaded.render_model.id)
        ),
    ];

    if !api_schema_imports.is_empty() {
        lines.insert(
            3,
            format!(
                "import {{ {} }} from \"./schemas.ts\";",
                public_schema_exports
                    .iter()
                    .filter(|export| api_schema_imports.contains(export.key.as_str()))
                    .map(|export| export.const_name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }

    for (key, rpc) in &loaded.render_model.rpc {
        let base = key_to_pascal(key);
        lines.push(String::new());
        lines.push(format!("const __{base}Descriptor = {{"));
        lines.push(format!(
            "  subject: {},",
            js_string(&loaded.subjects.rpc[key])
        ));
        lines.push(format!(
            "  permission: {},",
            permission_literal(&loaded.render_model.id, &rpc.version, "rpc", key, "call",)
        ));
        lines.push(format!(
            "  input: schema<Types.{base}Input>({}) as ReturnType<typeof schema<Types.{base}Input>> ,",
            schema_const_names
                .get(rpc.input.schema.as_str())
                .expect("missing public schema export for rpc input")
        ));
        lines.push(format!(
            "  output: schema<Types.{base}Output>({}) as ReturnType<typeof schema<Types.{base}Output>> ,",
            schema_const_names
                .get(rpc.output.schema.as_str())
                .expect("missing public schema export for rpc output")
        ));
        if rpc.transfer.is_some() {
            lines.push("  transfer: { direction: \"receive\" },".to_string());
        }
        let capabilities = capability_names(&loaded.value, "rpc", key, "call");
        lines.push(format!(
            "  callerCapabilities: {} as const,",
            serde_json::to_string(&capabilities).unwrap()
        ));
        if let Some(errors) = &rpc.errors {
            if !errors.is_empty() {
                let error_types = errors
                    .iter()
                    .map(|error| error.error_type.clone())
                    .collect::<Vec<_>>();
                lines.push(format!(
                    "  errors: {} as const,",
                    serde_json::to_string(&error_types).unwrap()
                ));
                lines.push(format!(
                    "  declaredErrorTypes: {} as const,",
                    serde_json::to_string(&error_types).unwrap()
                ));
            }
        }
        let local_runtime_errors = rpc
            .errors
            .as_ref()
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| {
                        loaded
                            .render_model
                            .errors
                            .iter()
                            .find(|(_, decl)| decl.error_type == value.error_type)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if !local_runtime_errors.is_empty() {
            lines.push("  runtimeErrors: [".to_string());
            for (_error_name, error_decl) in local_runtime_errors {
                let base = key_to_pascal(&error_decl.error_type);
                lines.push("    {".to_string());
                lines.push(format!(
                    "      type: {},",
                    js_string(&error_decl.error_type)
                ));
                if let Some(schema) = &error_decl.schema {
                    lines.push(format!(
                        "      schema: schema<Types.{base}Data>({}) as ReturnType<typeof schema<Types.{base}Data>> ,",
                        schema_const_names
                            .get(schema.schema.as_str())
                            .expect("missing public schema export for error schema")
                    ));
                }
                lines.push(format!(
                    "      fromSerializable: Types.{base}.fromSerializable as typeof Types.{base}.fromSerializable,"
                ));
                lines.push("    },".to_string());
            }
            lines.push("  ] as const,".to_string());
        }
        lines.push(format!("}} as const;\nexport const {base}: ReturnType<typeof rpcAction<typeof {owner_id}, {0}, typeof __{base}Descriptor>> = rpcAction({owner_id}, {0}, __{base}Descriptor, {1}, ACTION_SOURCE);", js_string(key), js_string(&base)));
    }

    for (key, operation) in &loaded.render_model.operations {
        let base = key_to_pascal(key);
        lines.push(String::new());
        lines.push(format!("const __{base}Descriptor = {{"));
        lines.push(format!(
            "  subject: {},",
            js_string(&loaded.subjects.operations[key])
        ));
        lines.push("  permissions: {".to_string());
        lines.push(format!(
            "    invoke: {},",
            permission_literal(
                &loaded.render_model.id,
                &operation.version,
                "operation",
                key,
                "invoke",
            )
        ));
        lines.push(format!(
            "    observe: {},",
            permission_literal(
                &loaded.render_model.id,
                &operation.version,
                "operation",
                key,
                "observe",
            )
        ));
        lines.push(format!(
            "    cancel: {},",
            permission_literal(
                &loaded.render_model.id,
                &operation.version,
                "operation",
                key,
                "cancel",
            )
        ));
        lines.push("    control: {".to_string());
        for signal_name in operation.signals.keys() {
            lines.push(format!(
                "      {}: {},",
                js_string(signal_name),
                permission_literal(
                    &loaded.render_model.id,
                    &operation.version,
                    "operation",
                    &format!("{key}.{signal_name}"),
                    "control",
                )
            ));
        }
        lines.push("    },".to_string());
        lines.push("  },".to_string());
        lines.push(format!(
            "  input: schema<Types.{base}Input>({}) as ReturnType<typeof schema<Types.{base}Input>> ,",
            schema_const_names
                .get(operation.input.schema.as_str())
                .expect("missing public schema export for operation input")
        ));
        if let Some(progress) = &operation.progress {
            lines.push(format!(
                "  progress: schema<Types.{base}Progress>({}) as ReturnType<typeof schema<Types.{base}Progress>> ,",
                schema_const_names
                    .get(progress.schema.as_str())
                    .expect("missing public schema export for operation progress")
            ));
        }
        if let Some(update) = &operation.update {
            lines.push(format!(
                "  update: schema<Types.{base}Update>({}) as ReturnType<typeof schema<Types.{base}Update>> ,",
                schema_const_names
                    .get(update.schema.as_str())
                    .expect("missing public schema export for operation update")
            ));
        }
        if let Some(output) = &operation.output {
            lines.push(format!(
                "  output: schema<Types.{base}Output>({}) as ReturnType<typeof schema<Types.{base}Output>> ,",
                schema_const_names
                    .get(output.schema.as_str())
                    .expect("missing public schema export for operation output")
            ));
        }
        if !operation.signals.is_empty() {
            lines.push("  signals: {".to_string());
            for (signal_name, signal) in &operation.signals {
                let signal_base = format!("{base}{}", key_to_pascal(signal_name));
                lines.push(format!("    {}: {{", js_string(signal_name)));
                lines.push(format!(
                    "      input: schema<Types.{signal_base}Signal>({}) as ReturnType<typeof schema<Types.{signal_base}Signal>> ,",
                    schema_const_names
                        .get(signal.input.schema.as_str())
                        .expect("missing public schema export for operation signal input")
                ));
                lines.push("    },".to_string());
            }
            lines.push("  },".to_string());
        }
        if let Some(transfer) = &operation.transfer {
            lines.push("  transfer: {".to_string());
            lines.push(format!(
                "    direction: {},",
                js_string(&transfer.direction)
            ));
            lines.push("  },".to_string());
        }
        let caller = capability_names(&loaded.value, "operation", key, "invoke");
        let observe = capability_names(&loaded.value, "operation", key, "observe");
        let cancel = capability_names(&loaded.value, "operation", key, "cancel");
        let control = capability_names(&loaded.value, "operation", key, "control");
        lines.push(format!(
            "  callerCapabilities: {} as const,",
            serde_json::to_string(&caller).unwrap()
        ));
        lines.push(format!(
            "  observeCapabilities: {} as const,",
            serde_json::to_string(&observe).unwrap()
        ));
        lines.push(format!(
            "  cancelCapabilities: {} as const,",
            serde_json::to_string(&cancel).unwrap()
        ));
        lines.push(format!(
            "  controlCapabilities: {} as const,",
            serde_json::to_string(&control).unwrap()
        ));
        // Emit errors, declaredErrorTypes, runtimeErrors for operations (mirroring RPC)
        if !operation.errors.is_empty() {
            let error_types = operation
                .errors
                .iter()
                .map(|error| error.error_type.clone())
                .collect::<Vec<_>>();
            lines.push(format!(
                "  errors: {} as const,",
                serde_json::to_string(&error_types).unwrap()
            ));
            lines.push(format!(
                "  declaredErrorTypes: {} as const,",
                serde_json::to_string(&error_types).unwrap()
            ));
        }
        let local_runtime_errors = operation
            .errors
            .iter()
            .filter_map(|value| {
                loaded
                    .render_model
                    .errors
                    .iter()
                    .find(|(_, decl)| decl.error_type == value.error_type)
            })
            .collect::<Vec<_>>();
        if !local_runtime_errors.is_empty() {
            lines.push("  runtimeErrors: [".to_string());
            for (_error_name, error_decl) in local_runtime_errors {
                let base = key_to_pascal(&error_decl.error_type);
                lines.push("    {".to_string());
                lines.push(format!(
                    "      type: {},",
                    js_string(&error_decl.error_type)
                ));
                if let Some(schema) = &error_decl.schema {
                    lines.push(format!(
                        "      schema: schema<Types.{base}Data>({}) as ReturnType<typeof schema<Types.{base}Data>> ,",
                        schema_const_names
                            .get(schema.schema.as_str())
                            .expect("missing public schema export for error schema")
                    ));
                }
                lines.push(format!(
                    "      fromSerializable: Types.{base}.fromSerializable as typeof Types.{base}.fromSerializable,"
                ));
                lines.push("    },".to_string());
            }
            lines.push("  ] as const,".to_string());
        }
        if let Some(cancelable) = operation.cancel {
            lines.push(format!(
                "  cancel: {},",
                if cancelable { "true" } else { "false" }
            ));
        }
        lines.push(format!("}} as const;\nObject.freeze(__{base}Descriptor.permissions.control);\nObject.freeze(__{base}Descriptor.permissions);\nexport const {base}: ReturnType<typeof operationAction<typeof {owner_id}, {0}, typeof __{base}Descriptor>> = operationAction({owner_id}, {0}, __{base}Descriptor, {1}, ACTION_SOURCE);", js_string(key), js_string(&base)));
    }

    for (key, event) in &loaded.render_model.events {
        let base = key_to_pascal(key);
        lines.push(String::new());
        lines.push(format!("const __{base}Descriptor = {{"));
        lines.push(format!(
            "  subject: {},",
            js_string(&loaded.subjects.events[key].template)
        ));
        lines.push(format!(
            "  publishPermission: {},",
            permission_literal(
                &loaded.render_model.id,
                &event.version,
                "event",
                key,
                "publish",
            )
        ));
        lines.push(format!(
            "  subscribePermission: {},",
            permission_literal(
                &loaded.render_model.id,
                &event.version,
                "event",
                key,
                "subscribe",
            )
        ));
        if let Some(params) = &event.params {
            if !params.is_empty() {
                lines.push(format!(
                    "  params: {} as const,",
                    serde_json::to_string(params).unwrap()
                ));
            }
        }
        lines.push(format!(
            "  event: schema<Types.{base}Event>({}) as ReturnType<typeof schema<Types.{base}Event>> ,",
            schema_const_names
                .get(event.event.schema.as_str())
                .expect("missing public schema export for event schema")
        ));
        let publish = capability_names(&loaded.value, "event", key, "publish");
        let subscribe = capability_names(&loaded.value, "event", key, "subscribe");
        lines.push(format!(
            "  publishCapabilities: {} as const,",
            serde_json::to_string(&publish).unwrap()
        ));
        lines.push(format!(
            "  subscribeCapabilities: {} as const,",
            serde_json::to_string(&subscribe).unwrap()
        ));
        lines.push(format!("}} as const;\nexport const {base}: ReturnType<typeof eventActions<typeof {owner_id}, {0}, typeof __{base}Descriptor, true>> = eventActions({owner_id}, {0}, __{base}Descriptor, {1}, true, ACTION_SOURCE);", js_string(key), js_string(&base)));
    }

    for (key, feed) in &loaded.render_model.feeds {
        let base = key_to_pascal(key);
        lines.push(String::new());
        lines.push(format!("const __{base}Descriptor = {{"));
        lines.push(format!(
            "  subject: {},",
            js_string(&loaded.subjects.feeds[key])
        ));
        lines.push(format!(
            "  permission: {},",
            permission_literal(
                &loaded.render_model.id,
                &feed.version,
                "feed",
                key,
                "subscribe",
            )
        ));
        lines.push(format!(
            "  input: schema<Types.{base}Input>({}) as ReturnType<typeof schema<Types.{base}Input>> ,",
            schema_const_names
                .get(feed.input.schema.as_str())
                .expect("missing public schema export for feed input")
        ));
        lines.push(format!(
            "  event: schema<Types.{base}Event>({}) as ReturnType<typeof schema<Types.{base}Event>> ,",
            schema_const_names
                .get(feed.event.schema.as_str())
                .expect("missing public schema export for feed event")
        ));
        let subscribe = capability_names(&loaded.value, "feed", key, "subscribe");
        lines.push(format!(
            "  subscribeCapabilities: {} as const,",
            serde_json::to_string(&subscribe).unwrap()
        ));
        lines.push(format!("}} as const;\nexport const {base}: ReturnType<typeof feedAction<typeof {owner_id}, {0}, typeof __{base}Descriptor>> = feedAction({owner_id}, {0}, __{base}Descriptor, {1}, ACTION_SOURCE);", js_string(key), js_string(&base)));
    }
    lines.push(String::new());
    lines.push("export const ACTIONS = {".to_owned());
    for key in loaded.render_model.rpc.keys() {
        lines.push(format!(
            "  {}: {1} as typeof {1},",
            js_string(key),
            key_to_pascal(key)
        ));
    }
    for key in loaded.render_model.operations.keys() {
        lines.push(format!(
            "  {}: {1} as typeof {1},",
            js_string(key),
            key_to_pascal(key)
        ));
    }
    for key in loaded.render_model.events.keys() {
        lines.push(format!(
            "  {}: {1} as typeof {1},",
            js_string(key),
            key_to_pascal(key)
        ));
    }
    for key in loaded.render_model.feeds.keys() {
        lines.push(format!(
            "  {}: {1} as typeof {1},",
            js_string(key),
            key_to_pascal(key)
        ));
    }
    lines.push("} as const;".to_owned());
    lines.push(String::new());
    format!(
        "{}
",
        lines.join(
            "
"
        )
    )
}

fn permission_literal(
    api_id: &str,
    api_version: &str,
    surface_kind: &str,
    surface_name: &str,
    action: &str,
) -> String {
    format!(
        "Object.freeze({{ apiId: {0}, apiVersion: {1}, surfaceKind: {2}, surfaceName: {3}, action: {4} }}) as {{ readonly apiId: {0}; readonly apiVersion: {1}; readonly surfaceKind: {2}; readonly surfaceName: {3}; readonly action: {4} }}",
        js_string(api_id),
        js_string(api_version),
        js_string(surface_kind),
        js_string(surface_name),
        js_string(action),
    )
}

fn lower_camel_ident(value: &str) -> String {
    let pascal = key_to_pascal(value);
    let mut chars = pascal.chars();
    match chars.next() {
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
        None => "_".to_string(),
    }
}

fn render_trellis_md(loaded: &ApiInput) -> String {
    let mut lines = vec![
        format!("# Generated by Trellis.\n\nAPI guide: `{}`.", loaded.render_model.id),
        String::new(),
        "This file is generated for AI agents and out-of-tree Trellis services.".to_string(),
        String::new(),
        "## Global Trellis Context".to_string(),
        String::new(),
        "- llms.txt: https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms.txt".to_string(),
        "- llms-full.txt: https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms-full.txt".to_string(),
        String::new(),
        "## API Module".to_string(),
        String::new(),
        format!("- module: `apis.{}`", lower_camel_ident(&sdk_output_stem(&loaded.render_model.id))),
        format!("- API id: `{}`", loaded.render_model.id),
        String::new(),
        "## Consumer Vocabulary".to_string(),
        String::new(),
        "Declare selected actions in the local native `.trellis` participant's `use` block.".to_string(),
    ];

    push_ts_owned_surfaces(&mut lines, loaded);
    lines.extend([
        String::new(),
        "The module's `API` value contains its canonical API metadata.".to_string(),
        String::new(),
    ]);
    lines.join("\n")
}

fn push_ts_owned_surfaces(lines: &mut Vec<String>, loaded: &ApiInput) {
    let connected_path = |key: &str| {
        let (group, leaf) = key.split_once('.').unwrap_or((key, key));
        format!("{}.{}", lower_camel_ident(group), lower_camel_ident(leaf))
    };
    let has_public_rpc = !loaded.render_model.rpc.is_empty();
    for key in loaded.render_model.rpc.keys() {
        let descriptor = key_to_pascal(key);
        let connected = connected_path(key);
        lines.push(format!(
            "- RPC `{key}`: descriptor `{descriptor}`, connected call `client.rpc.{connected}(input)`"
        ));
    }
    for (key, event) in &loaded.render_model.events {
        let descriptor = key_to_pascal(key);
        let connected = connected_path(key);
        lines.push(format!(
            "- Event `{key}`: subscribe descriptor `{descriptor}.subscribe`, connected listener `client.event.{connected}.listen(handler)`"
        ));
        if event
            .capabilities
            .as_ref()
            .is_some_and(|capabilities| capabilities.publish.is_some())
        {
            lines.push(format!(
                "- Event `{key}` delegated publish: `{descriptor}.publish`, connected publisher `client.event.{connected}.publish(event)`"
            ));
        }
    }
    for key in loaded.render_model.feeds.keys() {
        let descriptor = key_to_pascal(key);
        let connected = connected_path(key);
        lines.push(format!(
            "- Feed `{key}`: descriptor `{descriptor}`, connected subscribe `client.feed.{connected}(input)`"
        ));
    }
    for key in loaded.render_model.operations.keys() {
        let descriptor = key_to_pascal(key);
        let connected = connected_path(key);
        lines.push(format!(
            "- Operation `{key}`: descriptor `{descriptor}`, connected call `client.operation.{connected}.start(input)`"
        ));
    }
    if !has_public_rpc
        && loaded.render_model.events.is_empty()
        && loaded.render_model.feeds.is_empty()
        && loaded.render_model.operations.is_empty()
    {
        lines.push("- No owned RPC, event, feed, or operation surfaces.".to_string());
    }
}

fn write_generated_file(path: &Path, contents: &str) -> Result<(), CodegenTsError> {
    let contents = format!("{}\n", contents.trim_end());
    if path.extension().is_some_and(|extension| extension == "ts") {
        validate_typescript(path, &contents)?;
    }
    write_if_changed(path, &contents)
}

fn validate_typescript(path: &Path, contents: &str) -> Result<(), CodegenTsError> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, contents, SourceType::ts()).parse();
    if parsed.errors.is_empty() {
        return Ok(());
    }

    let message = parsed
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ");
    Err(CodegenTsError::InvalidTypeScript {
        path: path.to_path_buf(),
        message,
    })
}

fn emit_typescript(source: &GeneratedTsSource) -> Result<[GeneratedTsSource; 2], CodegenTsError> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source.contents, SourceType::ts()).parse();
    let mut program = parsed.program;
    let mut errors = parsed.errors;
    if errors.is_empty() {
        // Rewrite only AST-identified module literals, never canonical schema strings.
        let mut executable_source = source.contents.clone();
        for literal in program
            .body
            .iter()
            .rev()
            .filter_map(|statement| {
                statement
                    .as_module_declaration()
                    .and_then(|module| module.source())
            })
            .filter(|literal| literal.value.starts_with("./") || literal.value.starts_with("../"))
        {
            if let Some(stem) = literal.value.strip_suffix(".ts") {
                executable_source.replace_range(
                    literal.span.start as usize..literal.span.end as usize,
                    &js_string(&format!("{stem}.js")),
                );
            }
        }
        let executable_source = allocator.alloc_str(&executable_source);
        let executable = Parser::new(&allocator, executable_source, SourceType::ts()).parse();
        errors.extend(executable.errors);
        program = executable.program;
        let declarations =
            IsolatedDeclarations::new(&allocator, IsolatedDeclarationsOptions::default())
                .build(&program);
        errors.extend(declarations.errors);
        let declaration_source = Codegen::new().build(&declarations.program).code;
        let semantic = SemanticBuilder::new().build(&program);
        errors.extend(semantic.errors);
        if errors.is_empty() {
            let transformed =
                Transformer::new(&allocator, &source.path, &TransformOptions::default())
                    .build_with_scoping(semantic.semantic.into_scoping(), &mut program);
            errors.extend(transformed.errors);
            if errors.is_empty() {
                return Ok([
                    GeneratedTsSource {
                        path: source.path.with_extension("js"),
                        contents: format!(
                            "// Generated by Trellis. Do not edit.\n// @ts-self-types=\"./{}\"\n{}",
                            source
                                .path
                                .with_extension("d.ts")
                                .file_name()
                                .expect("generated module filename")
                                .to_string_lossy(),
                            Codegen::new().build(&program).code
                        ),
                    },
                    GeneratedTsSource {
                        path: source.path.with_extension("d.ts"),
                        contents: format!(
                            "// Generated by Trellis. Do not edit.\n{declaration_source}"
                        ),
                    },
                ]);
            }
        }
    }
    Err(CodegenTsError::InvalidTypeScript {
        path: source.path.clone(),
        message: errors
            .into_iter()
            .map(|error| format!("{:?}", error.with_source_code(source.contents.clone())))
            .collect::<Vec<_>>()
            .join("\n"),
    })
}

fn write_if_changed(path: &Path, contents: &str) -> Result<(), CodegenTsError> {
    if fs::read_to_string(path).ok().as_deref() == Some(contents) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

fn js_string(value: &str) -> String {
    serde_json::to_string(value).expect("js string")
}

fn capability_names(api: &Value, surface: &str, name: &str, action: &str) -> Vec<String> {
    let mut names = api
        .get("capabilities")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|capabilities| capabilities.iter())
        .filter(|(_, capability)| {
            capability
                .get("allows")
                .and_then(Value::as_array)
                .is_some_and(|allows| {
                    allows.iter().any(|permission| {
                        permission.get("action").and_then(Value::as_str) == Some(action)
                            && permission.pointer("/target/kind").and_then(Value::as_str)
                                == Some("apiSurface")
                            && permission
                                .pointer("/target/surface")
                                .and_then(Value::as_str)
                                == Some(surface)
                            && permission.pointer("/target/name").and_then(Value::as_str)
                                == Some(name)
                    })
                })
        })
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn escape_js_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('$', "\\$")
}

fn resolve_schema_ref<'a>(loaded: &'a ApiInput, schema_name: &str) -> &'a Value {
    loaded
        .render_model
        .schemas
        .get(schema_name)
        .unwrap_or_else(|| panic!("missing schema '{schema_name}' in manifest"))
}

fn key_to_pascal(value: &str) -> String {
    value
        .split('.')
        .map(to_pascal_case_token)
        .collect::<Vec<_>>()
        .join("")
}

fn to_pascal_case_token(value: &str) -> String {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn schema_to_ts_with_aliases(
    schema: &Value,
    aliases: &[SchemaTypeAlias],
    excluded_alias_key: Option<&str>,
) -> String {
    if let Some(alias) = aliases
        .iter()
        .find(|alias| Some(alias.key.as_str()) != excluded_alias_key && alias.schema == *schema)
    {
        return alias.type_name.clone();
    }

    match schema {
        Value::Bool(true) => "unknown".to_string(),
        Value::Bool(false) => "never".to_string(),
        Value::Object(object) => {
            if let Some(value) = object.get("const") {
                return serde_json::to_string(value).unwrap_or_else(|_| "unknown".to_string());
            }

            if let Some(Value::Array(values)) = object.get("enum") {
                if !values.is_empty() {
                    return values
                        .iter()
                        .map(|value| {
                            serde_json::to_string(value).unwrap_or_else(|_| "unknown".to_string())
                        })
                        .collect::<Vec<_>>()
                        .join(" | ");
                }
            }

            for (key, operator) in [("allOf", "&"), ("oneOf", "|"), ("anyOf", "|")] {
                if let Some(Value::Array(values)) = object.get(key) {
                    if !values.is_empty() {
                        return format!(
                            "({})",
                            values
                                .iter()
                                .map(|value| {
                                    schema_to_ts_with_aliases(value, aliases, excluded_alias_key)
                                })
                                .collect::<Vec<_>>()
                                .join(&format!(" {operator} "))
                        );
                    }
                }
            }

            if let Some(Value::Array(types)) = object.get("type") {
                if !types.is_empty() {
                    return format!(
                        "({})",
                        types
                            .iter()
                            .map(|value| match value {
                                Value::String(type_name) => {
                                    let mut clone = object.clone();
                                    clone.insert(
                                        "type".to_string(),
                                        Value::String(type_name.clone()),
                                    );
                                    schema_to_ts_with_aliases(
                                        &Value::Object(clone),
                                        aliases,
                                        excluded_alias_key,
                                    )
                                }
                                _ => "unknown".to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(" | ")
                    );
                }
            }

            match object.get("type").and_then(Value::as_str) {
                Some("string") => "string".to_string(),
                Some("number") | Some("integer") => "number".to_string(),
                Some("boolean") => "boolean".to_string(),
                Some("null") => "null".to_string(),
                Some("array") => render_array_ts(object, aliases, excluded_alias_key),
                Some("object") => render_object_ts(object, aliases, excluded_alias_key),
                _ => {
                    if object.contains_key("properties") {
                        render_object_ts(object, aliases, excluded_alias_key)
                    } else {
                        "unknown".to_string()
                    }
                }
            }
        }
        Value::Null | Value::Number(_) | Value::String(_) | Value::Array(_) => {
            "unknown".to_string()
        }
    }
}

fn render_array_ts(
    object: &serde_json::Map<String, Value>,
    aliases: &[SchemaTypeAlias],
    excluded_alias_key: Option<&str>,
) -> String {
    match object.get("items") {
        Some(Value::Array(values)) => format!(
            "[{}]",
            values
                .iter()
                .map(|value| schema_to_ts_with_aliases(value, aliases, excluded_alias_key))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Some(value) => format!(
            "Array<{}>",
            schema_to_ts_with_aliases(value, aliases, excluded_alias_key)
        ),
        None => "unknown[]".to_string(),
    }
}

fn render_object_ts(
    object: &serde_json::Map<String, Value>,
    aliases: &[SchemaTypeAlias],
    excluded_alias_key: Option<&str>,
) -> String {
    let required = object
        .get("required")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut lines = Vec::new();
    if let Some(Value::Object(properties)) = object.get("properties") {
        for (key, value) in properties {
            let optional = if required.iter().any(|required_key| required_key == key) {
                ""
            } else {
                "?"
            };
            let safe_key = if is_safe_js_ident(key) {
                key.clone()
            } else {
                js_string(key)
            };
            lines.push(format!(
                "{safe_key}{optional}: {};",
                schema_to_ts_with_aliases(value, aliases, excluded_alias_key)
            ));
        }
    }

    if let Some(Value::Object(pattern_properties)) = object.get("patternProperties") {
        if pattern_properties.len() == 1 {
            let value = pattern_properties
                .values()
                .next()
                .expect("single pattern property value");
            lines.push(format!(
                "[k: string]: {};",
                schema_to_ts_with_aliases(value, aliases, excluded_alias_key)
            ));
        }
    }

    match object.get("additionalProperties") {
        Some(Value::Bool(true)) => lines.push("[k: string]: unknown;".to_string()),
        Some(value @ Value::Object(_)) => {
            lines.push(format!(
                "[k: string]: {};",
                schema_to_ts_with_aliases(value, aliases, excluded_alias_key)
            ));
        }
        _ => {}
    }

    format!("{{ {} }}", lines.join(" "))
}

fn is_safe_js_ident(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first == '_' || first == '$' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn render_mod_ts() -> String {
    [
        "export * from \"./api.ts\";",
        "export * from \"./descriptors.ts\";",
        "export * from \"./types.ts\";",
        "export * from \"./schemas.ts\";",
        "",
    ]
    .join("\n")
}

fn validate_public_export_names(loaded: &ApiInput) -> Result<(), CodegenTsError> {
    let mut values = BTreeSet::new();
    let mut types = BTreeSet::new();
    let insert = |namespace: &str,
                  names: &mut BTreeSet<String>,
                  name: String|
     -> Result<(), CodegenTsError> {
        if names.insert(name.clone()) {
            Ok(())
        } else {
            Err(CodegenTsError::ExportNameCollision(format!(
                "{} {namespace} export '{name}'",
                loaded.render_model.id
            )))
        }
    };

    for name in loaded.render_model.rpc.keys() {
        let base = key_to_pascal(name);
        insert("value", &mut values, base.clone())?;
        insert("type", &mut types, format!("{base}Input"))?;
        insert("type", &mut types, format!("{base}Output"))?;
    }
    for (name, operation) in &loaded.render_model.operations {
        let base = key_to_pascal(name);
        insert("value", &mut values, base.clone())?;
        insert("type", &mut types, format!("{base}Input"))?;
        if operation.progress.is_some() {
            insert("type", &mut types, format!("{base}Progress"))?;
        }
        if operation.update.is_some() {
            insert("type", &mut types, format!("{base}Update"))?;
        }
        if operation.output.is_some() {
            insert("type", &mut types, format!("{base}Output"))?;
        }
        for signal in operation.signals.keys() {
            insert(
                "type",
                &mut types,
                format!("{base}{}Signal", key_to_pascal(signal)),
            )?;
        }
    }
    for name in loaded.render_model.events.keys() {
        let base = key_to_pascal(name);
        insert("value", &mut values, base.clone())?;
        insert("type", &mut types, format!("{base}Event"))?;
    }
    for name in loaded.render_model.feeds.keys() {
        let base = key_to_pascal(name);
        insert("value", &mut values, base.clone())?;
        insert("type", &mut types, format!("{base}Input"))?;
        insert("type", &mut types, format!("{base}Event"))?;
    }
    for name in public_schema_keys(loaded) {
        let base = key_to_pascal(&name);
        insert("value", &mut values, format!("{base}Schema"))?;
        if loaded.render_model.exports.schemas.contains(&name) {
            insert("type", &mut types, base)?;
        }
    }
    for error in loaded.render_model.errors.values() {
        let base = key_to_pascal(&error.error_type);
        insert("value", &mut values, base.clone())?;
        insert("type", &mut types, base.clone())?;
        insert("type", &mut types, format!("{base}Data"))?;
    }
    Ok(())
}

fn public_schema_exports(loaded: &ApiInput) -> Vec<PublicSchemaExport> {
    let exported_schema_keys = exported_schema_keys(loaded);
    let mut used_const_names = BTreeSet::new();
    let mut used_type_names = generated_type_names(loaded);

    public_schema_keys(loaded)
        .into_iter()
        .map(|key| {
            let base_name = key_to_pascal(&key);
            let const_name =
                unique_export_name(&format!("{base_name}Schema"), &mut used_const_names);
            let type_name = if exported_schema_keys.contains(&key)
                && used_type_names.insert(base_name.clone())
            {
                Some(base_name)
            } else {
                None
            };

            PublicSchemaExport {
                key,
                const_name,
                type_name,
            }
        })
        .collect()
}

fn public_schema_type_aliases(
    loaded: &ApiInput,
    exports: &[PublicSchemaExport],
) -> Vec<SchemaTypeAlias> {
    exports
        .iter()
        .filter_map(|export| {
            Some(SchemaTypeAlias {
                key: export.key.clone(),
                type_name: export.type_name.clone()?,
                schema: resolve_schema_ref(loaded, &export.key).clone(),
            })
        })
        .collect()
}

fn public_schema_keys(loaded: &ApiInput) -> BTreeSet<String> {
    let mut keys = exported_schema_keys(loaded);

    for rpc in loaded.render_model.rpc.values() {
        keys.insert(rpc.input.schema.clone());
        keys.insert(rpc.output.schema.clone());
    }

    for operation in loaded.render_model.operations.values() {
        keys.insert(operation.input.schema.clone());
        if let Some(progress) = &operation.progress {
            keys.insert(progress.schema.clone());
        }
        if let Some(update) = &operation.update {
            keys.insert(update.schema.clone());
        }
        if let Some(output) = &operation.output {
            keys.insert(output.schema.clone());
        }
        for signal in operation.signals.values() {
            keys.insert(signal.input.schema.clone());
        }
    }

    for event in loaded.render_model.events.values() {
        keys.insert(event.event.schema.clone());
    }

    for feed in loaded.render_model.feeds.values() {
        keys.insert(feed.input.schema.clone());
        keys.insert(feed.event.schema.clone());
    }

    for error in loaded.render_model.errors.values() {
        if let Some(schema) = &error.schema {
            keys.insert(schema.schema.clone());
        }
    }

    keys
}

fn exported_schema_keys(loaded: &ApiInput) -> BTreeSet<String> {
    loaded
        .render_model
        .exports
        .schemas
        .iter()
        .cloned()
        .collect()
}

fn generated_type_names(loaded: &ApiInput) -> BTreeSet<String> {
    let mut names = BTreeSet::new();

    for key in loaded.render_model.rpc.keys() {
        let base = key_to_pascal(key);
        names.insert(format!("{base}Input"));
        names.insert(format!("{base}Output"));
    }

    for (key, operation) in &loaded.render_model.operations {
        let base = key_to_pascal(key);
        names.insert(format!("{base}Input"));
        if operation.progress.is_some() {
            names.insert(format!("{base}Progress"));
        }
        if operation.update.is_some() {
            names.insert(format!("{base}Update"));
        }
        if operation.output.is_some() {
            names.insert(format!("{base}Output"));
        }
        for signal_name in operation.signals.keys() {
            names.insert(format!("{base}{}Signal", key_to_pascal(signal_name)));
        }
    }

    for key in loaded.render_model.events.keys() {
        let base = key_to_pascal(key);
        names.insert(format!("{base}Event"));
    }

    for key in loaded.render_model.feeds.keys() {
        let base = key_to_pascal(key);
        names.insert(format!("{base}Input"));
        names.insert(format!("{base}Event"));
    }

    for error in loaded.render_model.errors.values() {
        let base = key_to_pascal(&error.error_type);
        names.insert(base.clone());
        names.insert(format!("{base}Data"));
    }

    names
}

fn unique_export_name(base: &str, used: &mut BTreeSet<String>) -> String {
    if used.insert(base.to_string()) {
        return base.to_string();
    }

    let mut index = 2;
    loop {
        let candidate = format!("{base}{index}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("trellis-codegen-ts-{label}-{nanos}"))
    }

    #[test]
    fn generated_state_api_emits_native_javascript_and_declarations() {
        let api = trellis_protocol::parse_api(
            &serde_json::from_str(include_str!("../../runtime-apis/src/trellis.state@v1.json"))
                .unwrap(),
        )
        .unwrap();
        let sources = collect_ts_sdk_sources(&api).unwrap();
        let guide = &sources
            .iter()
            .find(|source| source.path == Path::new("TRELLIS.md"))
            .unwrap()
            .contents;
        assert!(guide.contains("client.rpc.state.get(input)"));
        assert!(guide.contains("client.rpc.state.adminGet(input)"));
        for source in sources
            .iter()
            .filter(|source| source.path.extension().is_some_and(|ext| ext == "ts"))
        {
            emit_typescript(source).unwrap();
        }
    }

    #[test]
    fn source_writer_rejects_paths_outside_package() {
        let root = unique_temp_dir("invalid-output-path");
        let error = write_ts_sdk_sources(
            &root.join("sdk"),
            &[GeneratedTsSource {
                path: PathBuf::from("../escape.ts"),
                contents: "export {};\n".to_string(),
            }],
        )
        .unwrap_err();

        assert!(matches!(error, CodegenTsError::InvalidOutputPath(_)));
        assert!(!root.join("escape.ts").exists());
    }

    #[test]
    fn generated_exact_identity_consumer_type_checks() {
        let root = unique_temp_dir("exact-identity");
        let api = trellis_protocol::parse_api(&serde_json::json!({
            "format": "trellis.api.v1", "id": "identity@v1", "version": "1.0.0",
            "displayName": "Identity", "description": "Exact identity regression.",
            "schemas": {
                "Row": {"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]},
                "Count": {"type": "integer"}
            },
            "exports": {"schemas": ["Row"]},
            "operations": {"Work": {"version": "v1", "input": {"schema": "Count"}, "output": {"schema": "Row"}, "signals": {"Update": {"input": {"schema": "Row"}}}}},
            "errors": {"Failed": {}},
            "rpc": {
                "A.Get": {"version": "v1", "input": {"schema": "Row"}, "output": {"schema": "Row"}},
                "Get": {"version": "v1", "input": {"schema": "Count"}, "output": {"schema": "Count"}, "errors": ["Failed"]}
            }
        })).unwrap();
        let owned = trellis_protocol::parse_api(&serde_json::json!({
            "format": "trellis.api.v1", "id": "consumer@v1", "version": "1.0.0",
            "displayName": "Consumer", "description": "Exact identity consumer."
        }))
        .unwrap();
        let participant = trellis_protocol::parse_participant(&serde_json::json!({
            "format": "trellis.participant.v1", "id": "identity@v1", "kind": "service",
            "displayName": "Identity", "description": "Exact identity regression.",
            "implements": {"self": {"api": owned.id(), "apiDigest": owned.digest().unwrap()}},
            "uses": {"required": {"remote": {"api": api.id(), "apiDigest": api.digest().unwrap(), "rpc": {"call": ["Get"]}, "operations": {"control": {"Work": ["Update"]}}}}},
            "schemas": {"NotRow": {"type": "integer"}},
            "resources": {"kv": {"count": {"purpose": "Count", "schema": {"schema": "NotRow"}}}}
        })).unwrap();
        generate_ts_package(
            &BTreeMap::from([(api.id(), &api), (owned.id(), &owned)]),
            std::slice::from_ref(&participant),
            &root.join("sdk"),
            "identity-trellis",
        )
        .unwrap();
        let package: Value =
            serde_json::from_str(&fs::read_to_string(root.join("sdk/package.json")).unwrap())
                .unwrap();
        assert_eq!(package["trellisGenerated"], true);
        let mut invalid = participant.normalized_value().unwrap();
        invalid["uses"]["required"]["remote"]["rpc"]["call"] = serde_json::json!(["invented.Get"]);
        let invalid = trellis_protocol::parse_participant(&invalid).unwrap();
        let error = generate_ts_package(
            &BTreeMap::from([(api.id(), &api), (owned.id(), &owned)]),
            &[invalid],
            &root.join("invalid"),
            "identity-trellis",
        )
        .unwrap_err();
        assert!(matches!(error, CodegenTsError::MissingReference(_)));
        assert!(!root.join("invalid").exists());
        let mut collision = api.normalized_value().unwrap();
        collision["schemas"]["WorkInput"] = serde_json::json!({"type": "string"});
        collision["exports"]["schemas"] = serde_json::json!(["WorkInput"]);
        let collision = trellis_protocol::parse_api(&collision).unwrap();
        let error = generate_ts_package(
            &BTreeMap::from([(collision.id(), &collision)]),
            &[],
            &root.join("collision"),
            "identity-trellis",
        )
        .unwrap_err();
        assert!(matches!(error, CodegenTsError::ExportNameCollision(_)));
        assert!(!root.join("collision").exists());
        fs::write(
            root.join("consumer.ts"),
            r#"
import { participants, apis } from "./sdk/index.d.ts";
import { PARTICIPANT_RUNTIME, PARTICIPANT_KV_METADATA } from "@qlever-llc/trellis/generated";
const participant = participants.identity.participant;
const name: keyof typeof participant[typeof PARTICIPANT_RUNTIME]["usedApi"]["rpc"] = "Get";
// @ts-expect-error An unselected suffix match must not appear.
const wrong: keyof typeof participant[typeof PARTICIPANT_RUNTIME]["usedApi"]["rpc"] = "A.Get";
const count: number = participant[PARTICIPANT_KV_METADATA].count.value;
const operation: keyof typeof participant[typeof PARTICIPANT_RUNTIME]["usedApi"]["operations"] = "Work";
const output: apis.identity.GetOutput = 1;
const row: apis.identity.Row = { id: "row" };
void [name, wrong, count, output, row, operation];
"#,
        )
        .unwrap();
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let output = std::process::Command::new("deno")
            .args(["check", "--no-lock", "-c"])
            .arg(repo.join("ts/deno.json"))
            .arg(root.join("consumer.ts"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_runtime_consumer_type_checks() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let output = std::process::Command::new("deno")
            .current_dir(repo)
            .args([
                "check",
                "-c",
                "ts/integration/deno.json",
                "ts/integration/runtime_test.ts",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn invalid_generated_ts_is_rejected_before_write() {
        let root = unique_temp_dir("invalid-ts-before-write");
        let target = root.join("out").join("broken.ts");

        let err = write_generated_file(&target, "export const broken = ;\n").unwrap_err();

        assert!(matches!(err, CodegenTsError::InvalidTypeScript { .. }));
        assert!(!target.exists());

        if root.exists() {
            fs::remove_dir_all(root).unwrap();
        }
    }
}
