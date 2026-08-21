//! TypeScript SDK generation from canonical Trellis contract manifests.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::Value;
use trellis_contracts::{load_sdk_source, ApiSurfaceKindV1, LoadedApi, PermissionActionV1};

/// Errors returned while generating a TypeScript SDK package.
#[derive(thiserror::Error, Debug)]
pub enum CodegenTsError {
    #[error("contracts error: {0}")]
    Contracts(#[from] trellis_contracts::ContractsError),

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
    let parent = opts.out_dir.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let stem = opts
        .out_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("sdk");
    let staging = parent.join(format!(".{stem}.tmp-{}-{nonce}", std::process::id()));
    let backup = parent.join(format!(".{stem}.old-{}-{nonce}", std::process::id()));
    fs::create_dir(&staging)?;

    for source in collect_ts_sdk_sources(opts)? {
        write_generated_file(&staging.join(source.path), &source.contents)?;
    }

    if opts.out_dir.exists() {
        fs::rename(&opts.out_dir, &backup)?;
    }
    if let Err(error) = fs::rename(&staging, &opts.out_dir) {
        if backup.exists() {
            let _ = fs::rename(&backup, &opts.out_dir);
        }
        return Err(error.into());
    }
    if backup.exists() {
        fs::remove_dir_all(backup)?;
    }

    Ok(())
}

/// Render all files that make up a TypeScript SDK package without writing them.
pub fn collect_ts_sdk_sources(
    opts: &GenerateTsSdkOpts,
) -> Result<Vec<GeneratedTsSource>, CodegenTsError> {
    let loaded = load_sdk_source(&opts.api_path)?;
    validate_public_export_names(&loaded)?;
    Ok(vec![
        GeneratedTsSource {
            path: PathBuf::from("deno.json"),
            contents: format!(
                "{}\n",
                serde_json::to_string_pretty(&deno_json(opts, &loaded)?)?
            ),
        },
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

fn deno_json(
    opts: &GenerateTsSdkOpts,
    _loaded: &LoadedApi,
) -> Result<serde_json::Map<String, Value>, CodegenTsError> {
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

fn render_api_ts(opts: &GenerateTsSdkOpts, loaded: &LoadedApi) -> Result<String, CodegenTsError> {
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

fn native_api_source(loaded: &LoadedApi) -> Result<(Value, String), CodegenTsError> {
    Ok((loaded.value.clone(), loaded.digest.clone()))
}

fn render_wire_types_ts(opts: &GenerateTsSdkOpts, loaded: &LoadedApi) -> String {
    let source_reference =
        api_source_reference(&opts.api_path, opts.runtime_deps.repo_root.as_deref());
    let trellis_import = trellis_runtime_import(opts);
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
                js_string(&trellis_contracts_import(opts))
            ),
            format!(
                "import {{ TrellisError }} from {};",
                js_string(&format!("{trellis_import}/errors"))
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
        if !is_public_rpc(rpc) {
            continue;
        }
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

    format!("{}\n", lines.join("\n"))
}

fn render_schemas_ts(opts: &GenerateTsSdkOpts, loaded: &LoadedApi) -> String {
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

    format!(
        "{}
",
        lines.join(
            "
"
        )
    )
}

fn render_descriptors_ts(opts: &GenerateTsSdkOpts, loaded: &LoadedApi) -> String {
    let source_reference =
        api_source_reference(&opts.api_path, opts.runtime_deps.repo_root.as_deref());
    let trellis_contracts_import = trellis_contracts_import(opts);
    let public_schema_exports = public_schema_exports(loaded);
    let schema_const_names = public_schema_exports
        .iter()
        .map(|export| (export.key.as_str(), export.const_name.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut api_schema_imports = BTreeSet::new();
    for rpc in loaded
        .render_model
        .rpc
        .values()
        .filter(|rpc| is_public_rpc(rpc))
    {
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
            js_string(&trellis_contracts_import)
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
        if !is_public_rpc(rpc) {
            continue;
        }
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
        let capabilities = loaded.api.capability_names_for_surface(
            ApiSurfaceKindV1::Rpc,
            key,
            PermissionActionV1::Call,
        );
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
        let caller = loaded.api.capability_names_for_surface(
            ApiSurfaceKindV1::Operation,
            key,
            PermissionActionV1::Invoke,
        );
        let observe = loaded.api.capability_names_for_surface(
            ApiSurfaceKindV1::Operation,
            key,
            PermissionActionV1::Observe,
        );
        let cancel = loaded.api.capability_names_for_surface(
            ApiSurfaceKindV1::Operation,
            key,
            PermissionActionV1::Cancel,
        );
        let control = loaded.api.capability_names_for_surface(
            ApiSurfaceKindV1::Operation,
            key,
            PermissionActionV1::Control,
        );
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
        let publish = loaded.api.capability_names_for_surface(
            ApiSurfaceKindV1::Event,
            key,
            PermissionActionV1::Publish,
        );
        let subscribe = loaded.api.capability_names_for_surface(
            ApiSurfaceKindV1::Event,
            key,
            PermissionActionV1::Subscribe,
        );
        lines.push(format!(
            "  publishCapabilities: {} as const,",
            serde_json::to_string(&publish).unwrap()
        ));
        lines.push(format!(
            "  subscribeCapabilities: {} as const,",
            serde_json::to_string(&subscribe).unwrap()
        ));
        let delegated_publish = !publish.is_empty();
        lines.push(format!(
            "}}, {}, {}, ACTION_SOURCE);",
            js_string(&base),
            if delegated_publish { "true" } else { "false" }
        ));
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
        let subscribe = loaded.api.capability_names_for_surface(
            ApiSurfaceKindV1::Feed,
            key,
            PermissionActionV1::Subscribe,
        );
        lines.push(format!(
            "  subscribeCapabilities: {} as const,",
            serde_json::to_string(&subscribe).unwrap()
        ));
        lines.push(format!("}}, {}, ACTION_SOURCE);", js_string(&base)));
    }
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

fn is_public_rpc(rpc: &trellis_contracts::ContractRpcMethod) -> bool {
    rpc.internal != Some(true)
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

fn trellis_contracts_import(_opts: &GenerateTsSdkOpts) -> String {
    "@qlever-llc/trellis/contracts".to_string()
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

fn render_readme(opts: &GenerateTsSdkOpts, loaded: &LoadedApi) -> String {
    let import_specifier = sdk_readme_import_specifier(&opts.package_name);
    let descriptors = descriptor_export_names(loaded);
    let imports = if descriptors.is_empty() {
        "// This package exports schemas and wire types only.".to_string()
    } else {
        format!(
            "import {{ {} }} from \"{}\";",
            descriptors.join(", "),
            import_specifier
        )
    };
    format!(
        "# {}\n\nPortable Trellis consumer SDK for contract `{}`.\n\n## Usage\n\n```ts\nimport {{ defineAppContract }} from \"@qlever-llc/trellis\";\n{}\n\nexport default defineAppContract(() => ({{\n  id: \"example.app@v1\",\n  displayName: \"Example App\",\n  description: \"User-facing app for the example deployment.\",\n  uses: [{}],\n}}));\n```\n\n## Contents\n\n- `descriptors.ts`: owned RPC, operation, event, and feed action descriptors\n- `types.ts`: portable wire types and declared error classes\n- `schemas.ts`: reachable and explicitly exported JSON Schemas\n- `api.ts`: canonical native API entrypoint\n- `TRELLIS.md`: generated package guidance\n",
        opts.package_name,
        loaded.render_model.id,
        imports,
        descriptors.join(", ")
    )
}

fn render_trellis_md(opts: &GenerateTsSdkOpts, loaded: &LoadedApi) -> String {
    let mut lines = vec![
        format!("# Trellis Contract Guide: {}", loaded.render_model.id),
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
        format!("- contract id: `{}`", loaded.render_model.id),
        String::new(),
        "## Consumer Vocabulary".to_string(),
        String::new(),
        "Import direct action descriptors and list them in the local participant contract's `uses` array.".to_string(),
    ];

    push_ts_owned_surfaces(&mut lines, loaded);
    lines.extend([
        String::new(),
        "The canonical API is available from the package's `./api` entrypoint.".to_string(),
        String::new(),
    ]);
    lines.join("\n")
}

fn push_ts_owned_surfaces(lines: &mut Vec<String>, loaded: &LoadedApi) {
    let has_public_rpc = loaded.render_model.rpc.values().any(is_public_rpc);
    for (key, rpc) in &loaded.render_model.rpc {
        if !is_public_rpc(rpc) {
            continue;
        }
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

fn descriptor_export_names(loaded: &LoadedApi) -> Vec<String> {
    loaded
        .render_model
        .rpc
        .iter()
        .filter(|(_, rpc)| is_public_rpc(rpc))
        .map(|(name, _)| key_to_pascal(name))
        .chain(
            loaded
                .render_model
                .operations
                .keys()
                .map(|name| key_to_pascal(name)),
        )
        .chain(
            loaded
                .render_model
                .events
                .keys()
                .map(|name| key_to_pascal(name)),
        )
        .chain(
            loaded
                .render_model
                .feeds
                .keys()
                .map(|name| key_to_pascal(name)),
        )
        .collect()
}

fn write_generated_file(path: &Path, contents: &str) -> Result<(), CodegenTsError> {
    if path.extension().is_some_and(|extension| extension == "ts") {
        validate_typescript(path, contents)?;
    }
    write_if_changed(path, contents)
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

fn escape_js_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('$', "\\$")
}

fn resolve_schema_ref<'a>(loaded: &'a LoadedApi, schema_name: &str) -> &'a Value {
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

fn sdk_readme_import_specifier(package_name: &str) -> String {
    if let Some(trimmed) = package_name.strip_prefix("@qlever-llc/trellis-sdk-") {
        format!("@qlever-llc/trellis/sdk/{trimmed}")
    } else {
        package_name.to_string()
    }
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

fn render_mod_ts(_opts: &GenerateTsSdkOpts, _loaded: &LoadedApi) -> String {
    [
        "export * from \"./descriptors.ts\";",
        "export * from \"./types.ts\";",
        "export * from \"./schemas.ts\";",
        "",
    ]
    .join("\n")
}

fn validate_public_export_names(loaded: &LoadedApi) -> Result<(), CodegenTsError> {
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
                "{namespace} export '{name}'"
            )))
        }
    };

    for (name, rpc) in &loaded.render_model.rpc {
        if !is_public_rpc(rpc) {
            continue;
        }
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

fn public_schema_exports(loaded: &LoadedApi) -> Vec<PublicSchemaExport> {
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
    loaded: &LoadedApi,
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

fn public_schema_keys(loaded: &LoadedApi) -> BTreeSet<String> {
    let mut keys = exported_schema_keys(loaded);

    for rpc in loaded
        .render_model
        .rpc
        .values()
        .filter(|rpc| is_public_rpc(rpc))
    {
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

fn exported_schema_keys(loaded: &LoadedApi) -> BTreeSet<String> {
    loaded
        .render_model
        .exports
        .schemas
        .iter()
        .cloned()
        .collect()
}

fn generated_type_names(loaded: &LoadedApi) -> BTreeSet<String> {
    let mut names = BTreeSet::new();

    for (key, rpc) in &loaded.render_model.rpc {
        if !is_public_rpc(rpc) {
            continue;
        }
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
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("trellis-codegen-ts-{label}-{nanos}"))
    }

    #[test]
    fn protocol_api_generation_uses_api_identity() {
        let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../generated/protocol/apis/trellis.auth@v1.json");
        let sources = collect_ts_sdk_sources(&GenerateTsSdkOpts {
            api_path: manifest_path,
            out_dir: unique_temp_dir("protocol-api"),
            package_name: "@example/auth".to_owned(),
            package_version: "0.1.0".to_owned(),
            runtime_deps: TsRuntimeDeps {
                source: TsRuntimeSource::Registry,
                version: "0.11.0".to_owned(),
                repo_root: None,
            },
        })
        .unwrap();
        let source = |path: &str| {
            sources
                .iter()
                .find(|source| source.path == Path::new(path))
                .unwrap()
                .contents
                .as_str()
        };

        assert!(source("api.ts").contains("export const API_ID"));
        assert!(!source("api.ts").contains("CONTRACT_ID"));
    }

    fn minimal_manifest(contract_id: &str) -> Value {
        json!({
            "format": "trellis.api.v1",
            "id": contract_id,
            "displayName": "Test Contract",
            "description": "Fixture contract",
            "schemas": {},
            "rpc": {},
            "operations": {},
            "events": {}
        })
    }

    fn sample_opts_and_loaded(
        package_name: &str,
        contract_id: &str,
    ) -> (GenerateTsSdkOpts, LoadedApi, PathBuf) {
        let root = unique_temp_dir("manifest");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("contract.json");
        fs::write(
            &manifest_path,
            serde_json::to_string(&json!({
                "format": "trellis.api.v1",
                "id": contract_id,
                "displayName": "Example Contract",
                "description": "Example contract for SDK generation tests.",
                "schemas": {
                    "PingInput": {
                        "type": "object",
                        "properties": {}
                    },
                    "PingOutput": {
                        "type": "object",
                        "properties": {
                            "ok": { "type": "boolean" }
                        },
                        "required": ["ok"]
                    },
                    "ProcessInput": {
                        "type": "object",
                        "properties": {
                            "amount": { "type": "number" }
                        },
                        "required": ["amount"]
                    },
                    "ProcessProgress": {
                        "type": "object",
                        "properties": {
                            "step": { "type": "string" }
                        },
                        "required": ["step"]
                    },
                    "ProcessOutput": {
                        "type": "object",
                        "properties": {
                            "ok": { "type": "boolean" }
                        },
                        "required": ["ok"]
                    },
                    "ProcessContinue": {
                        "type": "object",
                        "properties": {
                            "confirmed": { "type": "boolean" }
                        },
                        "required": ["confirmed"]
                    },
                    "FeedInput": {
                        "type": "object",
                        "properties": {
                            "siteId": { "type": "string" }
                        },
                        "required": ["siteId"]
                    },
                    "FeedEvent": {
                        "type": "object",
                        "properties": {
                            "message": { "type": "string" }
                        },
                        "required": ["message"]
                    }
                },
                "rpc": {
                    "Example.Ping": {
                        "version": "v1",
                        "input": { "schema": "PingInput" },
                        "output": { "schema": "PingOutput" }
                    }
                },
                "operations": {
                    "Example.Process": {
                        "version": "v1",
                        "input": { "schema": "ProcessInput" },
                        "progress": { "schema": "ProcessProgress" },
                        "output": { "schema": "ProcessOutput" },
                        "signals": {
                            "continue": {
                                "input": { "schema": "ProcessContinue" }
                            }
                        },
                        "cancel": true
                    }
                },
                "events": {},
                "feeds": {
                    "Example.Live": {
                        "version": "v1",
                        "input": { "schema": "FeedInput" },
                        "event": { "schema": "FeedEvent" },
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let opts = GenerateTsSdkOpts {
            api_path: manifest_path.clone(),
            out_dir: root.join("out"),
            package_name: package_name.to_string(),
            package_version: "0.4.0".to_string(),
            runtime_deps: TsRuntimeDeps {
                source: TsRuntimeSource::Registry,
                version: "0.4.0".to_string(),
                repo_root: None,
            },
        };
        let loaded = load_sdk_source(&manifest_path).unwrap();
        (opts, loaded, root)
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

    #[test]
    fn registry_mode_emits_npm_imports() {
        let root = unique_temp_dir("registry-mode-npm-imports");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("trellis.core@v1.json");
        fs::write(
            &manifest_path,
            serde_json::to_string(&minimal_manifest("trellis.core@v1")).unwrap(),
        )
        .unwrap();
        let opts = GenerateTsSdkOpts {
            api_path: PathBuf::from("generated/protocol/apis/trellis.core@v1.json"),
            out_dir: PathBuf::from("generated/packages/jsr/trellis-core"),
            package_name: "@qlever-llc/trellis-sdk-core".to_string(),
            package_version: "0.4.0".to_string(),
            runtime_deps: TsRuntimeDeps {
                source: TsRuntimeSource::Registry,
                version: "0.2.3".to_string(),
                repo_root: None,
            },
        };
        let loaded = load_sdk_source(&manifest_path).unwrap();
        let deno = deno_json(&opts, &loaded).unwrap();

        let imports = deno.get("imports").and_then(Value::as_object).unwrap();
        assert_eq!(
            imports.get("@qlever-llc/trellis").unwrap(),
            "jsr:@qlever-llc/trellis@^0.2.3"
        );
        assert_eq!(imports.len(), 1);
        assert!(deno.get("extends").is_none());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_sdk_deno_json_includes_web_and_deno_libs() {
        let root = unique_temp_dir("sdk-deno-libs");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("trellis.core@v1.json");
        fs::write(
            &manifest_path,
            serde_json::to_string(&minimal_manifest("trellis.core@v1")).unwrap(),
        )
        .unwrap();
        let opts = GenerateTsSdkOpts {
            api_path: manifest_path.clone(),
            out_dir: root.join("generated/packages/jsr/trellis-core"),
            package_name: "@qlever-llc/trellis-sdk-core".to_string(),
            package_version: "0.4.0".to_string(),
            runtime_deps: TsRuntimeDeps {
                source: TsRuntimeSource::Registry,
                version: "0.4.0".to_string(),
                repo_root: None,
            },
        };
        let loaded = load_sdk_source(&manifest_path).unwrap();
        let deno = deno_json(&opts, &loaded).unwrap();
        let compiler_options = deno
            .get("compilerOptions")
            .and_then(Value::as_object)
            .unwrap();

        assert_eq!(
            compiler_options.get("lib").unwrap(),
            &json!(["dom", "dom.iterable", "dom.asynciterable", "deno.ns"])
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn jsr_package_generation_does_not_emit_npm_build_scripts() {
        let (opts, _loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-core", "trellis.core@v1");

        generate_ts_sdk(&opts).unwrap();

        let deno = fs::read_to_string(opts.out_dir.join("deno.json")).unwrap();
        assert!(!deno.contains("build:npm"));
        assert!(!opts.out_dir.join("scripts/build_npm.ts").exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn collect_ts_sdk_sources_returns_rendered_package_files() {
        let (opts, _loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-demo", "trellis.demo@v1");

        let sources = collect_ts_sdk_sources(&opts).unwrap();
        let paths = sources
            .iter()
            .map(|source| source.path.as_path())
            .collect::<Vec<_>>();

        assert!(paths.contains(&Path::new("mod.ts")));
        assert!(paths.contains(&Path::new("descriptors.ts")));
        assert!(paths.contains(&Path::new("api.ts")));
        assert!(paths.contains(&Path::new("types.ts")));
        assert!(paths.contains(&Path::new("schemas.ts")));
        assert!(!paths.contains(&Path::new("contract.ts")));
        assert!(paths.contains(&Path::new("api.ts")));
        assert!(!paths.contains(&Path::new("owned_api.ts")));
        assert!(!paths.contains(&Path::new("client.ts")));
        assert!(paths.contains(&Path::new("README.md")));
        assert!(paths.contains(&Path::new("TRELLIS.md")));
        assert!(sources
            .iter()
            .any(|source| source.path == Path::new("mod.ts")
                && source.contents.contains("./descriptors.ts")
                && !source.contents.contains("./api.ts")));
        assert!(sources.iter().any(|source| source.path == Path::new("TRELLIS.md")
            && source.contents.contains("client.examplePing(input)")
            && source.contents.contains("https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms.txt")));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn local_mode_derives_extends_from_repo_root() {
        let repo_root = unique_temp_dir("repo-root");
        let out_dir = repo_root.join("generated/packages/jsr/auth");
        fs::create_dir_all(repo_root.join("ts")).unwrap();
        fs::create_dir_all(&out_dir).unwrap();
        fs::write(repo_root.join("ts/deno.json"), "{}\n").unwrap();

        let manifest_path = repo_root.join("generated/protocol/apis/trellis.auth@v1.json");
        fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        fs::write(
            &manifest_path,
            serde_json::to_string(&minimal_manifest("trellis.auth@v1")).unwrap(),
        )
        .unwrap();
        let opts = GenerateTsSdkOpts {
            api_path: repo_root.join("generated/protocol/apis/trellis.auth@v1.json"),
            out_dir: out_dir.clone(),
            package_name: "@qlever-llc/trellis-sdk-auth".to_string(),
            package_version: "0.4.0".to_string(),
            runtime_deps: TsRuntimeDeps {
                source: TsRuntimeSource::Local,
                version: "0.4.0".to_string(),
                repo_root: Some(repo_root.clone()),
            },
        };
        let loaded = load_sdk_source(&manifest_path).unwrap();
        let deno = deno_json(&opts, &loaded).unwrap();

        assert_eq!(
            deno.get("extends").and_then(Value::as_str),
            Some("../../../../ts/deno.json")
        );
        assert!(deno.get("imports").is_none());

        fs::remove_dir_all(repo_root).unwrap();
    }

    #[test]
    fn local_mode_emits_package_runtime_imports() {
        let repo_root = unique_temp_dir("repo-root-local-imports");
        let out_dir = repo_root.join("workspaces/demo/generated/packages/jsr/auth");
        fs::create_dir_all(repo_root.join("ts/packages/trellis")).unwrap();
        fs::create_dir_all(&out_dir).unwrap();
        fs::write(repo_root.join("ts/deno.json"), "{}\n").unwrap();

        let (mut opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-auth", "trellis.auth@v1");
        opts.out_dir = out_dir.clone();
        opts.runtime_deps = TsRuntimeDeps {
            source: TsRuntimeSource::Local,
            version: "0.4.0".to_string(),
            repo_root: Some(repo_root.clone()),
        };

        let owned_api = render_descriptors_ts(&opts, &loaded);
        let contract = render_api_ts(&opts, &loaded).unwrap();
        let types = render_wire_types_ts(&opts, &loaded);

        assert!(owned_api.contains("@qlever-llc/trellis/contracts"));
        assert!(!contract.contains("@qlever-llc/trellis"));
        assert!(!owned_api.contains("ts/packages/trellis"));
        assert!(!contract.contains("ts/packages/trellis"));
        assert!(!types.contains("ts/packages/trellis"));

        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(repo_root).unwrap();
    }

    #[test]
    fn generated_sdk_emits_direct_action_descriptors() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-auth", "trellis.auth@v1");
        let descriptors = render_descriptors_ts(&opts, &loaded);
        assert!(
            descriptors.contains("export const ExamplePing = rpcAction(API_ID, \"Example.Ping\"")
        );
        assert!(descriptors.contains("subject: \"rpc.v1.Example.Ping\""));
        assert!(descriptors
            .contains("export const ExampleProcess = operationAction(API_ID, \"Example.Process\""));
        assert!(
            descriptors.contains("export const ExampleLive = feedAction(API_ID, \"Example.Live\"")
        );
        assert!(descriptors.contains("subject: \"feed.v1.Example.Live\""));
        assert!(descriptors.contains(
            "\"continue\": Object.freeze({ apiId: \"trellis.auth@v1\", apiVersion: \"v1\", surfaceKind: \"operation\", surfaceName: \"Example.Process.continue\", action: \"control\" }),"
        ));
        assert!(!descriptors.contains("OWNED_API"));
        assert!(descriptors.contains("API_DIGEST"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_owned_api_emits_literal_capability_arrays_for_all_surfaces() {
        let root = unique_temp_dir("literal-capability-arrays");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("trellis.demo@v1.json");
        fs::write(
            &manifest_path,
            serde_json::to_string(&json!({
                "format": "trellis.api.v1",
                "id": "trellis.demo@v1",
                "displayName": "Demo",
                "description": "Capability literal fixture.",
                "capabilities": {
                    "trellis.demo::rpc.read": {"allows": [{"action":"call","target":{"kind":"apiSurface","api":"trellis.demo@v1","surface":"rpc","name":"Demo.Get"}}]},
                    "trellis.demo::operation.run": {"allows": [{"action":"invoke","target":{"kind":"apiSurface","api":"trellis.demo@v1","surface":"operation","name":"Demo.Run"}}]},
                    "trellis.demo::operation.observe": {"allows": [{"action":"observe","target":{"kind":"apiSurface","api":"trellis.demo@v1","surface":"operation","name":"Demo.Run"}}]},
                    "trellis.demo::operation.cancel": {"allows": [{"action":"cancel","target":{"kind":"apiSurface","api":"trellis.demo@v1","surface":"operation","name":"Demo.Run"}}]},
                    "trellis.demo::operation.control": {"allows": [{"action":"control","target":{"kind":"operationSignal","api":"trellis.demo@v1","operation":"Demo.Run","signal":"continue"}}]},
                    "trellis.demo::event.publish": {"allows": [{"action":"publish","target":{"kind":"apiSurface","api":"trellis.demo@v1","surface":"event","name":"Demo.Updated"}}]},
                    "trellis.demo::event.subscribe": {"allows": [{"action":"subscribe","target":{"kind":"apiSurface","api":"trellis.demo@v1","surface":"event","name":"Demo.Updated"}}]},
                    "trellis.demo::feed.subscribe": {"allows": [{"action":"subscribe","target":{"kind":"apiSurface","api":"trellis.demo@v1","surface":"feed","name":"Demo.Live"}}]}
                },
                "schemas": {
                    "Empty": { "type": "object", "properties": {} },
                    "Result": { "type": "object", "properties": { "ok": { "type": "boolean" } } }
                },
                "rpc": {
                    "Demo.Get": {
                        "version": "v1",
                        "input": { "schema": "Empty" },
                        "output": { "schema": "Result" }
                    }
                },
                "operations": {
                    "Demo.Run": {
                        "version": "v1",
                        "input": { "schema": "Empty" },
                        "progress": { "schema": "Result" },
                        "output": { "schema": "Result" },
                        "signals": {"continue": {"input": {"schema": "Empty"}}},
                        "cancel": true
                    }
                },
                "events": {
                    "Demo.Updated": {
                        "version": "v1",
                        "event": { "schema": "Result" }
                    }
                },
                "feeds": {
                    "Demo.Live": {
                        "version": "v1",
                        "input": { "schema": "Empty" },
                        "event": { "schema": "Result" }
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();
        let opts = GenerateTsSdkOpts {
            api_path: manifest_path.clone(),
            out_dir: root.join("out"),
            package_name: "@qlever-llc/trellis-sdk-demo".to_string(),
            package_version: "0.4.0".to_string(),
            runtime_deps: TsRuntimeDeps {
                source: TsRuntimeSource::Registry,
                version: "0.4.0".to_string(),
                repo_root: None,
            },
        };
        let loaded = load_sdk_source(&manifest_path).unwrap();
        let owned_api = render_descriptors_ts(&opts, &loaded);

        assert!(owned_api.contains("callerCapabilities: [\"trellis.demo::rpc.read\"] as const,"));
        assert!(
            owned_api.contains("callerCapabilities: [\"trellis.demo::operation.run\"] as const,")
        );
        assert!(owned_api
            .contains("observeCapabilities: [\"trellis.demo::operation.observe\"] as const,"));
        assert!(owned_api
            .contains("cancelCapabilities: [\"trellis.demo::operation.cancel\"] as const,"));
        assert!(owned_api.contains(
            "permission: Object.freeze({ apiId: \"trellis.demo@v1\", apiVersion: \"v1\", surfaceKind: \"rpc\", surfaceName: \"Demo.Get\", action: \"call\" }),"
        ));
        assert!(owned_api.contains(
            "invoke: Object.freeze({ apiId: \"trellis.demo@v1\", apiVersion: \"v1\", surfaceKind: \"operation\", surfaceName: \"Demo.Run\", action: \"invoke\" }),"
        ));
        assert!(owned_api.contains(
            "publishPermission: Object.freeze({ apiId: \"trellis.demo@v1\", apiVersion: \"v1\", surfaceKind: \"event\", surfaceName: \"Demo.Updated\", action: \"publish\" }),"
        ));
        assert!(owned_api.contains(
            "subscribePermission: Object.freeze({ apiId: \"trellis.demo@v1\", apiVersion: \"v1\", surfaceKind: \"event\", surfaceName: \"Demo.Updated\", action: \"subscribe\" }),"
        ));
        assert!(owned_api.contains(
            "permission: Object.freeze({ apiId: \"trellis.demo@v1\", apiVersion: \"v1\", surfaceKind: \"feed\", surfaceName: \"Demo.Live\", action: \"subscribe\" }),"
        ));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn internal_rpcs_are_absent_from_consumer_descriptors() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-core", "trellis.core@v1");
        let descriptors = render_descriptors_ts(&opts, &loaded);
        assert!(!descriptors.contains("TrellisBindingsGet"));
        assert!(descriptors.contains("ExamplePing"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_manifest_is_tooling_only() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-core", "trellis.core@v1");
        let manifest = render_api_ts(&opts, &loaded).unwrap();
        let mod_ts = render_mod_ts(&opts, &loaded);
        let types = render_wire_types_ts(&opts, &loaded);
        assert!(manifest.contains("export const API"));
        assert!(manifest.contains("export const API_DIGEST"));
        assert!(!manifest.contains("export const CONTRACT"));
        assert!(!manifest.contains("sdk"));
        assert!(!manifest.contains("use"));
        assert!(!mod_ts.contains("manifest"));
        assert!(!mod_ts.contains("contract"));
        assert!(!types.contains("Handler"));
        assert!(!types.contains("Client"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_sdk_omits_participant_facade_artifacts() {
        let (opts, _loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-demo", "trellis.demo@v1");
        let sources = collect_ts_sdk_sources(&opts).unwrap();
        let paths = sources
            .iter()
            .map(|source| source.path.as_path())
            .collect::<BTreeSet<_>>();
        assert!(paths.contains(Path::new("descriptors.ts")));
        assert!(paths.contains(Path::new("api.ts")));
        assert!(paths.contains(Path::new("api.ts")));
        assert!(!paths.contains(Path::new("owned_api.ts")));
        assert!(!paths.contains(Path::new("client.ts")));
        assert!(!paths.contains(Path::new("contract.ts")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_sdk_does_not_render_used_contract_surfaces() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-demo", "trellis.demo@v1");
        let descriptors = render_descriptors_ts(&opts, &loaded);
        assert!(!descriptors.contains("dependency"));
        assert!(!descriptors.contains("USED_API"));
        assert!(!descriptors.contains("ClientUse"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_sdk_has_no_dependency_sdk_imports() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-demo", "trellis.demo@v1");
        for source in [
            render_descriptors_ts(&opts, &loaded),
            render_wire_types_ts(&opts, &loaded),
            render_mod_ts(&opts, &loaded),
        ] {
            assert!(!source.contains("../"));
            assert!(!source.contains("@trellis-sdk/"));
            assert!(!source.contains("/sdk/auth"));
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_event_types_are_portable_payloads() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-demo", "trellis.demo@v1");
        let types = render_wire_types_ts(&opts, &loaded);
        assert!(types.contains("Event"));
        assert!(!types.contains("EventMessage"));
        assert!(!types.contains("EventHandler"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_event_publication_requires_explicit_capabilities() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-auth", "trellis.auth@v1");
        let descriptors = render_descriptors_ts(&opts, &loaded);
        assert!(descriptors.contains("eventActions"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_consumer_types_omit_service_private_jobs() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-demo", "trellis.demo@v1");
        let types = render_wire_types_ts(&opts, &loaded);
        assert!(!types.contains("JobHandler"));
        assert!(!types.contains("ContractJobs"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_manifest_preserves_jobs_for_tooling() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-demo", "trellis.demo@v1");
        let manifest = render_api_ts(&opts, &loaded).unwrap();
        assert!(manifest.contains("export const API"));
        assert!(!manifest.contains("CONTRACT"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_types_emit_typed_pattern_properties() {
        let root = unique_temp_dir("typed-pattern-properties");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("contract.json");
        let manifest = serde_json::from_str::<Value>(
            r#"{
                "format": "trellis.api.v1",
                "id": "trellis.core@v1",
                "displayName": "Trellis Core",
                "description": "Core contract.",
                "schemas": {
                    "BindingsGetInput": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    },
                    "BindingsGetOutput": {
                        "type": "object",
                        "properties": {
                            "binding": {
                                "type": "object",
                                "required": ["resources"],
                                "properties": {
                                    "resources": {
                                        "type": "object",
                                        "required": ["streams"],
                                        "properties": {
                                            "streams": {
                                                "type": "object",
                                                "additionalProperties": true
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        "required": ["binding"]
                    }
                },
                "rpc": {
                    "Trellis.Bindings.Get": {
                        "version": "v1",
                        "input": { "schema": "BindingsGetInput" },
                        "output": { "schema": "BindingsGetOutput" }
                    }
                },
                "events": {}
            }"#,
        )
        .unwrap();
        fs::write(&manifest_path, serde_json::to_string(&manifest).unwrap()).unwrap();

        let opts = GenerateTsSdkOpts {
            api_path: manifest_path.clone(),
            out_dir: root.join("out"),
            package_name: "@qlever-llc/trellis-sdk-core".to_string(),
            package_version: "0.4.0".to_string(),
            runtime_deps: TsRuntimeDeps {
                source: TsRuntimeSource::Registry,
                version: "0.4.0".to_string(),
                repo_root: None,
            },
        };
        let loaded = load_sdk_source(&manifest_path).unwrap();

        let rendered = render_wire_types_ts(&opts, &loaded);

        assert!(rendered.contains("streams:"));
        assert!(rendered.contains("streams: { [k: string]: unknown; };"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_readme_uses_direct_descriptors() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-audit", "acme.audit@v1");
        let readme = render_readme(&opts, &loaded);

        assert!(readme.contains("import { defineAppContract } from \"@qlever-llc/trellis\";"));
        assert!(readme.contains("from \"@qlever-llc/trellis/sdk/audit\";"));
        assert!(readme.contains("displayName: \"Example App\""));
        assert!(readme.contains("description: \"User-facing app for the example deployment.\""));
        assert!(readme.contains("uses: ["));
        assert!(readme.contains("TRELLIS.md"));
        assert!(readme.contains("descriptors.ts"));
        assert!(!readme.contains("mergeApis"));
        assert!(!readme.contains("createClient(nc, auth, [api] as const)"));
        assert!(!readme.contains("dependency.use"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_sdk_emits_local_error_classes_and_runtime_descriptors() {
        let root = unique_temp_dir("generated-sdk-local-errors");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("contract.json");
        let manifest = serde_json::from_str::<Value>(
            r#"{
                "format": "trellis.api.v1",
                "id": "example.local-errors@v1",
                "displayName": "Local Errors",
                "description": "Local error sdk test.",
                "schemas": {
                    "Empty": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    },
                    "NotFoundErrorData": {
                        "type": "object",
                        "required": ["id", "type", "message", "resource"],
                        "properties": {
                            "id": { "type": "string" },
                            "type": { "const": "NotFoundError" },
                            "message": { "type": "string" },
                            "resource": { "type": "string" }
                        }
                    }
                },
                "errors": {
                    "WorkspaceMissing": {
                        "schema": { "schema": "NotFoundErrorData" }
                    },
                    "UnexpectedError": {}
                },
                "rpc": {
                    "Example.Get": {
                        "version": "v1",
                        "input": { "schema": "Empty" },
                        "output": { "schema": "Empty" },
                        "errors": ["WorkspaceMissing", "UnexpectedError"]
                    }
                },
                "events": {}
            }"#,
        )
        .unwrap();
        fs::write(&manifest_path, serde_json::to_string(&manifest).unwrap()).unwrap();

        let opts = GenerateTsSdkOpts {
            api_path: manifest_path.clone(),
            out_dir: root.join("out"),
            package_name: "@qlever-llc/trellis-sdk-local-errors".to_string(),
            package_version: "0.4.0".to_string(),
            runtime_deps: TsRuntimeDeps {
                source: TsRuntimeSource::Registry,
                version: "0.4.0".to_string(),
                repo_root: None,
            },
        };
        let loaded = load_sdk_source(&manifest_path).unwrap();

        let types = render_wire_types_ts(&opts, &loaded);
        let schemas = render_schemas_ts(&opts, &loaded);
        let owned_api = render_descriptors_ts(&opts, &loaded);

        assert!(types.contains(
            "import type { SerializableErrorData } from \"@qlever-llc/trellis/contracts\";"
        ));
        assert!(types.contains("import { TrellisError } from \"@qlever-llc/trellis/errors\";"));
        assert!(!types.contains("Handler"));
        assert!(!types.contains("RpcHandlerContext"));
        assert!(types.contains("NotFoundError"));
        assert!(types.contains("type: \"NotFoundError\";"));
        assert!(types.contains("resource: string;"));
        assert!(types
            .contains("export class WorkspaceMissing extends TrellisError<WorkspaceMissingData>"));
        assert!(types.contains("static readonly schema = NotFoundErrorDataSchema;"));
        assert!(
            types.contains("static fromSerializable(data: WorkspaceMissingData): WorkspaceMissing")
        );
        assert!(schemas.contains("export const EmptySchema = "));
        assert!(schemas.contains("export const NotFoundErrorDataSchema = "));
        assert!(!schemas.contains("SCHEMAS"));
        assert!(owned_api.contains("runtimeErrors: ["));
        assert!(owned_api.contains("import * as Types from \"./types.ts\";"));
        assert!(owned_api.contains("type: \"WorkspaceMissing\""));
        assert!(owned_api.contains("NotFoundErrorDataSchema"));
        assert!(owned_api.contains("fromSerializable: Types.WorkspaceMissing.fromSerializable"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_types_emit_operation_types() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-core", "trellis.core@v1");
        let types = render_wire_types_ts(&opts, &loaded);

        assert!(types.contains("export type ExampleProcessInput = { amount: number; };"));
        assert!(types.contains("export type ExampleProcessProgress = { step: string; };"));
        assert!(types.contains("export type ExampleProcessOutput = { ok: boolean; };"));
        assert!(!types.contains("OperationHandler"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_types_omit_per_method_handler_aliases() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-demo", "trellis.demo@v1");
        let types = render_wire_types_ts(&opts, &loaded);
        assert!(!types.contains("Handler"));
        assert!(!types.contains("HandlerClient"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_schemas_include_operations() {
        let (opts, loaded, root) =
            sample_opts_and_loaded("@qlever-llc/trellis-sdk-core", "trellis.core@v1");
        let schemas = render_schemas_ts(&opts, &loaded);

        assert!(schemas.contains("export const PingInputSchema = "));
        assert!(schemas.contains("export const PingOutputSchema = "));
        assert!(schemas.contains("export const ProcessProgressSchema = "));
        assert!(!schemas.contains("SCHEMAS"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generated_public_schema_exports_follow_surface_and_exports_config() {
        let root = unique_temp_dir("public-schema-exports");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("contract.json");
        fs::write(
            &manifest_path,
            serde_json::to_string(&json!({
                "format": "trellis.api.v1",
                "id": "example.schemas@v1",
                "displayName": "Schema Exports",
                "description": "Schema exports test.",
                "schemas": {
                    "PingInput": {
                        "type": "object",
                        "properties": {}
                    },
                    "PingOutput": {
                        "type": "object",
                        "properties": {
                            "ok": { "type": "boolean" },
                            "shared": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string" }
                                },
                                "required": ["name"]
                            }
                        },
                        "required": ["ok", "shared"]
                    },
                    "SharedModel": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" }
                        },
                        "required": ["name"]
                    },
                    "NotFoundErrorData": {
                        "type": "object",
                        "required": ["id", "type", "message", "resource"],
                        "properties": {
                            "id": { "type": "string" },
                            "type": { "const": "NotFoundError" },
                            "message": { "type": "string" },
                            "resource": { "type": "string" }
                        }
                    },
                    "InternalOnly": {
                        "type": "object",
                        "properties": {
                            "value": { "type": "string" }
                        },
                        "required": ["value"]
                    }
                },
                "exports": {
                    "schemas": ["SharedModel", "NotFoundErrorData"]
                },
                "errors": {
                    "WorkspaceMissing": {
                        "schema": { "schema": "NotFoundErrorData" }
                    }
                },
                "rpc": {
                    "Example.Ping": {
                        "version": "v1",
                        "input": { "schema": "PingInput" },
                        "output": { "schema": "PingOutput" }
                    }
                },
                "events": {}
            }))
            .unwrap(),
        )
        .unwrap();

        let opts = GenerateTsSdkOpts {
            api_path: manifest_path.clone(),
            out_dir: root.join("out"),
            package_name: "@qlever-llc/trellis-sdk-schema-exports".to_string(),
            package_version: "0.4.0".to_string(),
            runtime_deps: TsRuntimeDeps {
                source: TsRuntimeSource::Registry,
                version: "0.4.0".to_string(),
                repo_root: None,
            },
        };
        let loaded = load_sdk_source(&manifest_path).unwrap();

        let types = render_wire_types_ts(&opts, &loaded);
        let schemas = render_schemas_ts(&opts, &loaded);

        assert!(schemas.contains("export const PingInputSchema = "));
        assert!(schemas.contains("export const SharedModelSchema = "));
        assert!(schemas.contains("export const NotFoundErrorDataSchema = "));
        assert!(!schemas.contains("InternalOnlySchema"));
        assert!(types.contains("export type SharedModel = { name: string; };"));
        assert!(types.contains("shared: SharedModel;"), "{types}");
        assert_eq!(
            types
                .match_indices("export type NotFoundErrorData = ")
                .count(),
            1
        );

        fs::remove_dir_all(root).unwrap();
    }
}
