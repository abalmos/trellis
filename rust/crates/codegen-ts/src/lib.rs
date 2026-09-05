//! TypeScript SDK generation from canonical Trellis contract manifests.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::Value;
use trellis_protocol::parse_api;

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

    #[error("missing manifest path file name")]
    MissingManifestFileName,

    #[error("missing runtime repo root for local runtime source")]
    MissingRuntimeRepoRoot,

    #[error("could not find a Deno config under runtime repo root")]
    MissingRuntimeConfig,

    #[error("invalid generated TypeScript in {path}: {message}")]
    InvalidTypeScript { path: PathBuf, message: String },

    #[error("generated TypeScript export name collision: {0}")]
    ExportNameCollision(String),

    #[error("generated TypeScript output path must stay within the package: {0}")]
    InvalidOutputPath(PathBuf),
}

fn load_sdk_source(path: impl AsRef<Path>) -> Result<ApiInput, CodegenTsError> {
    let path = path.as_ref().to_path_buf();
    let source = fs::read_to_string(&path)?;
    let value: Value = serde_json::from_str(&source)?;
    let api = parse_api(&value)?;
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

/// Options for generating one TypeScript SDK package.
#[derive(Debug, Clone)]
pub struct GenerateTsSdkOpts {
    pub api_path: PathBuf,
    pub out_dir: PathBuf,
    pub package_name: String,
    pub package_version: String,
    pub runtime_deps: TsRuntimeDeps,
}

/// Options for generating one TypeScript participant runtime module.
#[derive(Debug, Clone)]
pub struct GenerateTsParticipantOpts {
    /// Canonical participant artifact path.
    pub participant_path: PathBuf,
    /// Canonical artifact path for the implemented API.
    pub owned_api_path: PathBuf,
    /// Canonical artifact paths for referenced APIs.
    pub referenced_api_paths: Vec<PathBuf>,
    /// Generated participant package directory.
    pub out_dir: PathBuf,
}

/// One generated TypeScript SDK source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTsSource {
    pub path: PathBuf,
    pub contents: String,
}

/// Runtime dependency configuration for generated TypeScript SDKs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsRuntimeDeps {
    pub source: TsRuntimeSource,
    pub version: String,
    pub repo_root: Option<PathBuf>,
}

/// Where generated SDKs should resolve Trellis runtime packages from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsRuntimeSource {
    Registry,
    Local,
}

/// Generate a TypeScript SDK package for one native API.
pub fn generate_ts_sdk(opts: &GenerateTsSdkOpts) -> Result<(), CodegenTsError> {
    let sources = collect_ts_sdk_sources(opts)?;
    write_ts_sdk_sources(&opts.out_dir, &sources)
}

/// Generate runtime data for one TypeScript participant.
pub fn generate_ts_participant(opts: &GenerateTsParticipantOpts) -> Result<(), CodegenTsError> {
    let source = render_ts_participant(opts)?;
    write_ts_sdk_sources(
        &opts.out_dir,
        &[GeneratedTsSource {
            path: PathBuf::from("mod.ts"),
            contents: source,
        }],
    )
}

/// Atomically write a previously rendered TypeScript SDK package.
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
    let parent = out_dir.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let stem = out_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("sdk");
    let staging = parent.join(format!(".{stem}.tmp-{}-{nonce}", std::process::id()));
    let backup = parent.join(format!(".{stem}.old-{}-{nonce}", std::process::id()));
    fs::create_dir(&staging)?;

    for source in sources {
        write_generated_file(&staging.join(&source.path), &source.contents)?;
    }

    if out_dir.exists() {
        fs::rename(out_dir, &backup)?;
    }
    if let Err(error) = fs::rename(&staging, out_dir) {
        if backup.exists() {
            let _ = fs::rename(&backup, out_dir);
        }
        return Err(error.into());
    }
    if backup.exists() {
        fs::remove_dir_all(backup)?;
    }

    Ok(())
}

fn render_ts_participant(opts: &GenerateTsParticipantOpts) -> Result<String, CodegenTsError> {
    let participant: trellis_protocol::ParticipantArtifact =
        serde_json::from_slice(&fs::read(&opts.participant_path)?)?;
    let participant_value =
        participant
            .normalized_value()
            .map_err(|error| CodegenTsError::InvalidTypeScript {
                path: opts.participant_path.clone(),
                message: error.to_string(),
            })?;
    let participant_digest =
        participant
            .digest()
            .map_err(|error| CodegenTsError::InvalidTypeScript {
                path: opts.participant_path.clone(),
                message: error.to_string(),
            })?;
    let owned = load_sdk_source(&opts.owned_api_path)?;
    let mut loaded_apis = vec![&owned];
    let referenced = opts
        .referenced_api_paths
        .iter()
        .map(load_sdk_source)
        .collect::<Result<Vec<_>, _>>()?;
    let mut apis = BTreeMap::from([(owned.render_model.id.clone(), owned.value.clone())]);
    let referenced_ids = participant_value["implements"]
        .as_object()
        .into_iter()
        .chain(participant_value["uses"]["required"].as_object())
        .chain(participant_value["uses"]["optional"].as_object())
        .flat_map(|entries| entries.values())
        .filter_map(|entry| entry["api"].as_str())
        .collect::<BTreeSet<_>>();
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
        "} from \"@qlever-llc/trellis\";".to_owned(),
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
        selected_action_expressions(&participant_value, &aliases, &apis);
    let all_actions = owned_actions
        .iter()
        .chain(&required_actions)
        .chain(&optional_actions)
        .cloned()
        .collect::<BTreeSet<_>>();
    lines.push(String::new());
    lines.push("export const participant = {".to_owned());
    lines.push(format!("  id: {},", js_string(participant.id())));
    lines.push(format!("  digest: {},", js_string(&participant_digest)));
    lines.push(format!(
        "  artifact: {} as const,",
        serde_json::to_string(&participant_value)?
    ));
    lines.push(format!("  api: {owned_alias}.API,"));
    lines.push(format!("  apiDigest: {owned_alias}.API_DIGEST,"));
    lines.push(format!(
        "  referencedApis: [{}] as const,",
        aliases
            .iter()
            .filter(|(id, _)| *id != &owned.render_model.id)
            .map(|(_, alias)| format!("{alias}.API"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    for expression in owned_surface_entries(&owned.value, owned_alias) {
        lines.push(format!("  {expression},"));
    }
    lines.push("  [PARTICIPANT_RUNTIME]: {".to_owned());
    lines.push(format!(
        "    ownedApi: runtimeApiFromActions([{}], {}),",
        owned_actions.iter().cloned().collect::<Vec<_>>().join(", "),
        serde_json::to_string(
            &participant_value["implements"]["self"]
                .get("operationTransfers")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}))
        )?
    ));
    lines.push(format!(
        "    usedApi: runtimeApiFromActions([{}]),",
        required_actions
            .iter()
            .chain(&optional_actions)
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ")
    ));
    lines.push(format!(
        "    api: runtimeApiFromActions([{}], {}),",
        all_actions.iter().cloned().collect::<Vec<_>>().join(", "),
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
            .map(|action| format!("{{ action: {action}, optional: false }}"))
            .chain(
                optional_actions
                    .iter()
                    .map(|action| format!("{{ action: {action}, optional: true }}"))
            )
            .collect::<Vec<_>>()
            .join(", ")
    ));
    lines.push("  },".to_owned());
    insert_participant_metadata(&mut lines, &participant_value, &metadata_aliases)?;
    lines.extend([
        "} as const;".to_owned(),
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
        actions.insert(format!("{descriptor}.publish"));
        actions.insert(format!("{descriptor}.subscribe"));
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
            format!("{symbol}: {alias}.ACTIONS[{}]", js_string(name))
        })
        .collect()
}

fn selected_action_expressions(
    participant: &Value,
    aliases: &BTreeMap<String, String>,
    apis: &BTreeMap<String, Value>,
) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut required = BTreeSet::new();
    let mut optional = BTreeSet::new();
    for (category, selected) in [("required", &mut required), ("optional", &mut optional)] {
        for used in participant["uses"][category]
            .as_object()
            .into_iter()
            .flat_map(|value| value.values())
        {
            let api_id = used["api"].as_str().expect("validated API use");
            let alias = &aliases[api_id];
            let api = &apis[api_id];
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
                            js_string(descriptor_name(api, section, name))
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
                        "{alias}.ACTIONS[{}].{action}",
                        js_string(descriptor_name(api, "events", name))
                    ));
                }
            }
        }
    }
    (required, optional)
}

fn descriptor_name<'a>(api: &'a Value, section: &str, selected: &'a str) -> &'a str {
    api[section]
        .as_object()
        .and_then(|entries| {
            entries
                .keys()
                .find(|name| *name == selected || name.ends_with(&format!(".{selected}")))
        })
        .map(String::as_str)
        .unwrap_or(selected)
}

fn insert_participant_metadata(
    lines: &mut Vec<String>,
    participant: &Value,
    aliases: &[SchemaTypeAlias],
) -> Result<(), CodegenTsError> {
    let schemas = participant["schemas"].as_object();
    for (section, symbol) in [
        ("state", "PARTICIPANT_STATE_METADATA"),
        ("jobQueues", "PARTICIPANT_JOBS_METADATA"),
    ] {
        let Some(entries) = participant[section].as_object() else {
            continue;
        };
        lines.push(format!("  [{symbol}]: {{"));
        for (name, entry) in entries {
            let schema_name = entry["schema"]["schema"]
                .as_str()
                .or_else(|| entry["payload"]["schema"].as_str())
                .expect("validated participant schema reference");
            let schemas = schemas.expect("participant schemas");
            let schema = resolve_participant_schema(schemas, schema_name);
            let ty = participant_schema_type(schema_name, schema, aliases);
            if section == "state" {
                lines.push(format!("    {}: {{ kind: {}, value: typeOnly<{ty}>(), schema: {} as const, stateVersion: {}, acceptedVersions: {{}} }},", js_string(name), js_string(entry["kind"].as_str().expect("state kind")), serde_json::to_string(schema)?, js_string(entry["stateVersion"].as_str().unwrap_or("v1"))));
            } else {
                lines.push(format!(
                    "    {}: {{ payload: typeOnly<{ty}>(), result: typeOnly<unknown>() }},",
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
            lines.push(format!("  [{symbol}]: {{"));
            for (name, entry) in entries {
                if section == "kv" {
                    let schema_name = entry["schema"]["schema"].as_str().expect("KV schema");
                    let schemas = schemas.expect("participant schemas");
                    let schema = resolve_participant_schema(schemas, schema_name);
                    let ty = participant_schema_type(schema_name, schema, aliases);
                    lines.push(format!("    {}: {{ required: true, value: typeOnly<{ty}>(), schema: {} as const }},", js_string(name), serde_json::to_string(schema)?));
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
        lines.push(format!(
            "  [PARTICIPANT_EVENT_CONSUMERS_METADATA]: {} as const,",
            serde_json::to_string(&consumers)?
        ));
    }
    Ok(())
}

fn resolve_participant_schema<'a>(
    schemas: &'a serde_json::Map<String, Value>,
    name: &str,
) -> &'a Value {
    let schema = &schemas[name];
    schema
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|reference| reference.strip_prefix("#/schemas/"))
        .map(|name| resolve_participant_schema(schemas, name))
        .unwrap_or(schema)
}

fn participant_schema_type(name: &str, schema: &Value, aliases: &[SchemaTypeAlias]) -> String {
    aliases
        .iter()
        .filter(|alias| name.ends_with(&alias.key))
        .max_by_key(|alias| alias.key.len())
        .map(|alias| alias.type_name.clone())
        .unwrap_or_else(|| schema_to_ts_with_aliases(schema, aliases, None))
}

/// Render the package configuration for a TypeScript SDK.
pub fn render_ts_sdk_config(opts: &GenerateTsSdkOpts) -> Result<GeneratedTsSource, CodegenTsError> {
    Ok(GeneratedTsSource {
        path: PathBuf::from("deno.json"),
        contents: format!("{}\n", serde_json::to_string_pretty(&deno_json(opts)?)?),
    })
}

/// Render all files that make up a TypeScript SDK package without writing them.
pub fn collect_ts_sdk_sources(
    opts: &GenerateTsSdkOpts,
) -> Result<Vec<GeneratedTsSource>, CodegenTsError> {
    let loaded = load_sdk_source(&opts.api_path)?;
    validate_public_export_names(&loaded)?;
    Ok(vec![
        render_ts_sdk_config(opts)?,
        GeneratedTsSource {
            path: PathBuf::from("descriptors.ts"),
            contents: render_descriptors_ts(opts, &loaded),
        },
        GeneratedTsSource {
            path: PathBuf::from("types.ts"),
            contents: render_wire_types_ts(opts, &loaded),
        },
        GeneratedTsSource {
            path: PathBuf::from("schemas.ts"),
            contents: render_schemas_ts(opts, &loaded),
        },
        GeneratedTsSource {
            path: PathBuf::from("api.ts"),
            contents: render_api_ts(opts, &loaded)?,
        },
        GeneratedTsSource {
            path: PathBuf::from("mod.ts"),
            contents: render_mod_ts(opts, &loaded),
        },
        GeneratedTsSource {
            path: PathBuf::from("README.md"),
            contents: render_readme(opts, &loaded),
        },
        GeneratedTsSource {
            path: PathBuf::from("TRELLIS.md"),
            contents: render_trellis_md(opts, &loaded),
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

fn deno_json(opts: &GenerateTsSdkOpts) -> Result<serde_json::Map<String, Value>, CodegenTsError> {
    let mut root = serde_json::Map::new();
    let extends = resolved_extends(opts)?;

    if let Some(extends) = &extends {
        root.insert("extends".to_string(), Value::String(extends.clone()));
    }
    root.insert("name".to_string(), Value::String(opts.package_name.clone()));
    root.insert(
        "version".to_string(),
        Value::String(opts.package_version.clone()),
    );
    root.insert("publish".to_string(), Value::Bool(false));
    root.insert(
        "exports".to_string(),
        serde_json::json!({
            ".": "./mod.ts",
            "./api": "./api.ts"
        }),
    );
    if extends.is_none() {
        let mut imports = serde_json::Map::new();
        imports.insert(
            "@qlever-llc/trellis".to_string(),
            Value::String(format!(
                "jsr:@qlever-llc/trellis@^{}",
                opts.runtime_deps.version
            )),
        );
        root.insert("imports".to_string(), Value::Object(imports));
    }
    root.insert(
        "compilerOptions".to_string(),
        serde_json::json!({
            "strict": true,
            "lib": ["dom", "dom.iterable", "dom.asynciterable", "deno.ns"],
            "verbatimModuleSyntax": true
        }),
    );

    Ok(root)
}

fn render_api_ts(opts: &GenerateTsSdkOpts, loaded: &ApiInput) -> Result<String, CodegenTsError> {
    let source_reference =
        api_source_reference(&opts.api_path, opts.runtime_deps.repo_root.as_deref());
    let (api, digest) = native_api_source(loaded)?;
    Ok(format!(
        "// Generated from {}\n\nexport const API_ID = {} as const;\nexport const API_DIGEST = {} as const;\nexport const API = {} as const;\n",
        escape_js_string(&source_reference),
        js_string(&loaded.render_model.id),
        js_string(&digest),
        serde_json::to_string(&api)?,
    ))
}

fn native_api_source(loaded: &ApiInput) -> Result<(Value, String), CodegenTsError> {
    Ok((loaded.value.clone(), loaded.digest.clone()))
}

fn render_wire_types_ts(opts: &GenerateTsSdkOpts, loaded: &ApiInput) -> String {
    let source_reference =
        api_source_reference(&opts.api_path, opts.runtime_deps.repo_root.as_deref());
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
        escape_js_string(&source_reference)
    )];

    if !loaded.render_model.errors.is_empty() {
        lines.extend([
            format!(
                "import type {{ SerializableErrorData }} from {};",
                js_string(&trellis_runtime_import(opts))
            ),
            format!(
                "import {{ TrellisError }} from {};",
                js_string(&trellis_runtime_import(opts))
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
                "  static readonly schema = {};",
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

fn render_schemas_ts(opts: &GenerateTsSdkOpts, loaded: &ApiInput) -> String {
    let source_reference =
        api_source_reference(&opts.api_path, opts.runtime_deps.repo_root.as_deref());
    let public_schema_exports = public_schema_exports(loaded);
    let mut lines = vec![format!(
        "// Generated from {}",
        escape_js_string(&source_reference)
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

fn render_descriptors_ts(opts: &GenerateTsSdkOpts, loaded: &ApiInput) -> String {
    let source_reference =
        api_source_reference(&opts.api_path, opts.runtime_deps.repo_root.as_deref());
    let trellis_runtime_import = trellis_runtime_import(opts);
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
        format!("// Generated from {}", escape_js_string(&source_reference)),
        format!(
            "import {{ eventActions, feedAction, operationAction, rpcAction, schema }} from {};",
            js_string(&trellis_runtime_import)
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
        lines.push(format!(
            "export const {base} = rpcAction({owner_id}, {}, {{",
            js_string(key)
        ));
        lines.push(format!(
            "  subject: {},",
            js_string(&loaded.subjects.rpc[key])
        ));
        lines.push(format!(
            "  permission: {},",
            permission_literal(&loaded.render_model.id, &rpc.version, "rpc", key, "call",)
        ));
        lines.push(format!(
            "  input: schema<Types.{base}Input>({}),",
            schema_const_names
                .get(rpc.input.schema.as_str())
                .expect("missing public schema export for rpc input")
        ));
        lines.push(format!(
            "  output: schema<Types.{base}Output>({}),",
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
                        "      schema: schema<Types.{base}Data>({}),",
                        schema_const_names
                            .get(schema.schema.as_str())
                            .expect("missing public schema export for error schema")
                    ));
                }
                lines.push(format!(
                    "      fromSerializable: Types.{base}.fromSerializable,"
                ));
                lines.push("    },".to_string());
            }
            lines.push("  ] as const,".to_string());
        }
        lines.push(format!("}}, {}, ACTION_SOURCE);", js_string(&base)));
    }

    for (key, operation) in &loaded.render_model.operations {
        let base = key_to_pascal(key);
        lines.push(String::new());
        lines.push(format!(
            "export const {base} = operationAction({owner_id}, {}, {{",
            js_string(key)
        ));
        lines.push(format!(
            "  subject: {},",
            js_string(&loaded.subjects.operations[key])
        ));
        lines.push("  permissions: Object.freeze({".to_string());
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
        lines.push("    control: Object.freeze({".to_string());
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
        lines.push("    }),".to_string());
        lines.push("  }),".to_string());
        lines.push(format!(
            "  input: schema<Types.{base}Input>({}),",
            schema_const_names
                .get(operation.input.schema.as_str())
                .expect("missing public schema export for operation input")
        ));
        if let Some(progress) = &operation.progress {
            lines.push(format!(
                "  progress: schema<Types.{base}Progress>({}),",
                schema_const_names
                    .get(progress.schema.as_str())
                    .expect("missing public schema export for operation progress")
            ));
        }
        if let Some(update) = &operation.update {
            lines.push(format!(
                "  update: schema<Types.{base}Update>({}),",
                schema_const_names
                    .get(update.schema.as_str())
                    .expect("missing public schema export for operation update")
            ));
        }
        if let Some(output) = &operation.output {
            lines.push(format!(
                "  output: schema<Types.{base}Output>({}),",
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
                    "      input: schema<Types.{signal_base}Signal>({}),",
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
                        "      schema: schema<Types.{base}Data>({}),",
                        schema_const_names
                            .get(schema.schema.as_str())
                            .expect("missing public schema export for error schema")
                    ));
                }
                lines.push(format!(
                    "      fromSerializable: Types.{base}.fromSerializable,"
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
        lines.push(format!("}}, {}, ACTION_SOURCE);", js_string(&base)));
    }

    for (key, event) in &loaded.render_model.events {
        let base = key_to_pascal(key);
        lines.push(String::new());
        lines.push(format!(
            "export const {base} = eventActions({owner_id}, {}, {{",
            js_string(key)
        ));
        lines.push(format!(
            "  subject: {},",
            js_string(&loaded.subjects.events[key].base)
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
            "  event: schema<Types.{base}Event>({}),",
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
        lines.push(format!("}}, {}, true, ACTION_SOURCE);", js_string(&base)));
    }

    for (key, feed) in &loaded.render_model.feeds {
        let base = key_to_pascal(key);
        lines.push(String::new());
        lines.push(format!(
            "export const {base} = feedAction({owner_id}, {}, {{",
            js_string(key)
        ));
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
            "  input: schema<Types.{base}Input>({}),",
            schema_const_names
                .get(feed.input.schema.as_str())
                .expect("missing public schema export for feed input")
        ));
        lines.push(format!(
            "  event: schema<Types.{base}Event>({}),",
            schema_const_names
                .get(feed.event.schema.as_str())
                .expect("missing public schema export for feed event")
        ));
        let subscribe = capability_names(&loaded.value, "feed", key, "subscribe");
        lines.push(format!(
            "  subscribeCapabilities: {} as const,",
            serde_json::to_string(&subscribe).unwrap()
        ));
        lines.push(format!("}}, {}, ACTION_SOURCE);", js_string(&base)));
    }
    lines.push(String::new());
    lines.push("export const ACTIONS = {".to_owned());
    for key in loaded.render_model.rpc.keys() {
        lines.push(format!("  {}: {},", js_string(key), key_to_pascal(key)));
    }
    for key in loaded.render_model.operations.keys() {
        lines.push(format!("  {}: {},", js_string(key), key_to_pascal(key)));
    }
    for key in loaded.render_model.events.keys() {
        lines.push(format!("  {}: {},", js_string(key), key_to_pascal(key)));
    }
    for key in loaded.render_model.feeds.keys() {
        lines.push(format!("  {}: {},", js_string(key), key_to_pascal(key)));
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
        "Object.freeze({{ apiId: {}, apiVersion: {}, surfaceKind: {}, surfaceName: {}, action: {} }})",
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

fn resolved_extends(opts: &GenerateTsSdkOpts) -> Result<Option<String>, CodegenTsError> {
    match opts.runtime_deps.source {
        TsRuntimeSource::Registry => Ok(None),
        TsRuntimeSource::Local => {
            let repo_root = opts
                .runtime_deps
                .repo_root
                .as_ref()
                .ok_or(CodegenTsError::MissingRuntimeRepoRoot)?;
            let repo_root = repo_root.canonicalize()?;
            let runtime_config = runtime_config_path(&repo_root)?;
            let out_dir = opts
                .out_dir
                .canonicalize()
                .unwrap_or_else(|_| opts.out_dir.clone());
            Ok(Some(relative_path_string(&out_dir, &runtime_config)))
        }
    }
}

fn trellis_runtime_import(_opts: &GenerateTsSdkOpts) -> String {
    "@qlever-llc/trellis".to_string()
}

fn runtime_config_path(repo_root: &Path) -> Result<PathBuf, CodegenTsError> {
    let js_deno = repo_root.join("ts/deno.json");
    if js_deno.exists() {
        return Ok(js_deno);
    }

    let root_deno = repo_root.join("deno.json");
    if root_deno.exists() {
        return Ok(root_deno);
    }

    Err(CodegenTsError::MissingRuntimeConfig)
}

fn relative_path_string(from_dir: &Path, to_path: &Path) -> String {
    let from_components = from_dir.components().collect::<Vec<_>>();
    let to_components = to_path.components().collect::<Vec<_>>();
    let common_len = from_components
        .iter()
        .zip(&to_components)
        .take_while(|(left, right)| left == right)
        .count();

    let mut relative = PathBuf::new();
    for _ in common_len..from_components.len() {
        relative.push("..");
    }
    for component in &to_components[common_len..] {
        relative.push(component.as_os_str());
    }
    normalize_relative_path_string(relative.to_string_lossy().replace('\\', "/"))
}

fn api_source_reference(api_path: &Path, repo_root: Option<&Path>) -> String {
    let api_path = api_path
        .canonicalize()
        .unwrap_or_else(|_| api_path.to_path_buf());

    if let Some(repo_root) = repo_root {
        let repo_root = repo_root
            .canonicalize()
            .unwrap_or_else(|_| repo_root.to_path_buf());
        if let Ok(relative) = api_path.strip_prefix(&repo_root) {
            return normalize_relative_path_string(relative.to_string_lossy().replace('\\', "/"));
        }
    }

    normalize_relative_path_string(api_path.to_string_lossy().replace('\\', "/"))
}

fn normalize_relative_path_string(path: String) -> String {
    if path.is_empty() || path.starts_with("../") || path.starts_with("./") || path.starts_with('/')
    {
        return path;
    }
    format!("./{path}")
}

fn render_readme(opts: &GenerateTsSdkOpts, loaded: &ApiInput) -> String {
    let lineage = loaded
        .render_model
        .id
        .split_once('@')
        .map_or(loaded.render_model.id.as_str(), |(lineage, _)| lineage);
    let import_specifier = format!("@trellis/apis/{lineage}");
    format!(
        "# {}\n\nGenerated TypeScript SDK for API `{}`.\n\nImport descriptors from `{}` and declare their use in the consuming project's native `.trellis` participant.\n\n## Contents\n\n- `descriptors.ts`: RPC, operation, event, and feed action descriptors\n- `types.ts`: portable wire types and declared error classes\n- `schemas.ts`: reachable and explicitly exported JSON Schemas\n- `api.ts`: canonical native API entrypoint\n- `TRELLIS.md`: generated package guidance\n",
        opts.package_name,
        loaded.render_model.id,
        import_specifier,
    )
}

fn render_trellis_md(opts: &GenerateTsSdkOpts, loaded: &ApiInput) -> String {
    let mut lines = vec![
        format!("# Trellis API Guide: {}", loaded.render_model.id),
        String::new(),
        "This file is generated for AI agents and out-of-tree Trellis services.".to_string(),
        String::new(),
        "## Global Trellis Context".to_string(),
        String::new(),
        "- llms.txt: https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms.txt".to_string(),
        "- llms-full.txt: https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms-full.txt".to_string(),
        String::new(),
        "## Package".to_string(),
        String::new(),
        format!("- package: `{}`", opts.package_name),
        format!("- API id: `{}`", loaded.render_model.id),
        String::new(),
        "## Consumer Vocabulary".to_string(),
        String::new(),
        "Declare selected actions in the local native `.trellis` participant's `use` block.".to_string(),
    ];

    push_ts_owned_surfaces(&mut lines, loaded);
    lines.extend([
        String::new(),
        "The canonical API is available from the package's `./api` entrypoint.".to_string(),
        String::new(),
    ]);
    lines.join("\n")
}

fn push_ts_owned_surfaces(lines: &mut Vec<String>, loaded: &ApiInput) {
    let has_public_rpc = !loaded.render_model.rpc.is_empty();
    for key in loaded.render_model.rpc.keys() {
        let descriptor = key_to_pascal(key);
        let connected = lower_camel_ident(key);
        lines.push(format!(
            "- RPC `{key}`: descriptor `{descriptor}`, connected call `client.{connected}(input)`"
        ));
    }
    for (key, event) in &loaded.render_model.events {
        let descriptor = key_to_pascal(key);
        let connected = key_to_pascal(&lower_camel_ident(key));
        lines.push(format!(
            "- Event `{key}`: subscribe descriptor `{descriptor}.subscribe`, connected listener `client.on{connected}(handler)`"
        ));
        if event
            .capabilities
            .as_ref()
            .is_some_and(|capabilities| capabilities.publish.is_some())
        {
            lines.push(format!(
                "- Event `{key}` delegated publish: `{descriptor}.publish`, connected publisher `client.publish{connected}(event)`"
            ));
        }
    }
    for key in loaded.render_model.feeds.keys() {
        let descriptor = key_to_pascal(key);
        let connected = lower_camel_ident(key);
        lines.push(format!(
            "- Feed `{key}`: descriptor `{descriptor}`, connected subscribe `client.{connected}(input)`"
        ));
    }
    for key in loaded.render_model.operations.keys() {
        let descriptor = key_to_pascal(key);
        let connected = lower_camel_ident(key);
        lines.push(format!(
            "- Operation `{key}`: descriptor `{descriptor}`, connected call `client.{connected}(input).start()`"
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

/// Local module specifiers referenced by TypeScript source code.
#[derive(Debug, PartialEq, Eq)]
pub struct TypeScriptModuleDependencies {
    /// Static and literal dynamic module specifiers in stable order.
    pub specifiers: Vec<String>,
    /// Whether a computed dynamic import prevents complete static discovery.
    pub has_computed_dynamic_import: bool,
    /// Whether parse errors prevent complete static discovery.
    pub has_parse_errors: bool,
}

/// Parse TypeScript imports and re-exports for incremental input tracking.
pub fn typescript_module_dependencies(contents: &str) -> TypeScriptModuleDependencies {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, contents, SourceType::tsx()).parse();
    let has_parse_errors = !parsed.errors.is_empty();
    let mut specifiers = parsed
        .module_record
        .requested_modules
        .keys()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let mut has_computed_dynamic_import = false;
    for dynamic_import in &parsed.module_record.dynamic_imports {
        let source = &contents[dynamic_import.module_request.start as usize
            ..dynamic_import.module_request.end as usize];
        let literal = source
            .strip_prefix(['\'', '"'])
            .and_then(|source| source.strip_suffix(['\'', '"']));
        if let Some(literal) = literal {
            specifiers.push(literal.to_string());
        } else {
            has_computed_dynamic_import = true;
        }
    }
    specifiers.sort();
    specifiers.dedup();
    TypeScriptModuleDependencies {
        specifiers,
        has_computed_dynamic_import,
        has_parse_errors,
    }
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

#[cfg(test)]
mod path_tests {
    use super::{api_source_reference, relative_path_string};
    use std::path::Path;

    #[test]
    fn manifest_source_reference_uses_repo_relative_path() {
        assert_eq!(
            api_source_reference(
                Path::new("/repo/generated/protocol/apis/trellis.core@v1.json"),
                Some(Path::new("/repo")),
            ),
            "./generated/protocol/apis/trellis.core@v1.json"
        );
    }

    #[test]
    fn relative_path_string_is_normalized_without_dot_segments() {
        assert_eq!(
            relative_path_string(
                Path::new("/repo/generated/packages/jsr/trellis-core"),
                Path::new("/repo/ts/packages/contracts/npm"),
            ),
            "../../../../ts/packages/contracts/npm"
        );
    }
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

fn render_mod_ts(_opts: &GenerateTsSdkOpts, _loaded: &ApiInput) -> String {
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
    fn module_dependencies_include_static_and_literal_dynamic_imports() {
        let dependencies = typescript_module_dependencies(
            r#"
                import "./side-effect.ts";
                import { value } from "./value.ts";
                export { other } from "./other.ts";
                export * from "./all.ts";
                await import("./dynamic.ts");
            "#,
        );

        assert_eq!(
            dependencies.specifiers,
            [
                "./all.ts",
                "./dynamic.ts",
                "./other.ts",
                "./side-effect.ts",
                "./value.ts",
            ]
        );
        assert!(!dependencies.has_computed_dynamic_import);
    }

    #[test]
    fn module_dependencies_flag_computed_dynamic_imports() {
        let dependencies = typescript_module_dependencies("await import(`./${name}.ts`);");

        assert!(dependencies.specifiers.is_empty());
        assert!(dependencies.has_computed_dynamic_import);
    }

    #[test]
    fn module_dependencies_report_parse_errors() {
        assert!(typescript_module_dependencies("import {").has_parse_errors);
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
