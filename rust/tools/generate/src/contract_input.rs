use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use miette::IntoDiagnostic;
use serde_json::Value;
use tempfile::TempDir;
use trellis_contracts::{load_participant_source, load_sdk_source, LoadedApi, LoadedParticipant};

const DEFAULT_IMAGE_API_PATH: &str = "/trellis/api.json";
const OCI_CONTRACT_PATH_LABELS: &[&str] = &["io.trellis.api.path"];
const DENO_BIN_ENV: &str = "TRELLIS_DENO_BIN";
const TSX_BIN_ENV: &str = "TRELLIS_TSX_BIN";
const NODE_BIN_ENV: &str = "TRELLIS_NODE_BIN";
const OCI_TOOL_ENV: &str = "TRELLIS_OCI_TOOL";

#[derive(Debug)]
struct TypeScriptRuntimeContext {
    current_dir: PathBuf,
    deno_config: Option<PathBuf>,
    package_json: Option<PathBuf>,
    use_node_modules_dir: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SourceModuleKind {
    TypeScript,
    JavaScript,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum SourceResolverRuntime {
    Deno { binary: String },
    Tsx { binary: String },
    Node { binary: String },
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum BinaryOverride {
    Unset,
    Disabled,
    Binary(String),
    Invalid(String),
}

#[derive(Debug)]
pub struct ResolvedNativeInput {
    pub api: LoadedApi,
    pub api_path: PathBuf,
    pub participant: Option<LoadedParticipant>,
    pub participant_path: Option<PathBuf>,
    pub referenced_apis: Vec<LoadedApi>,
    pub owner_version: Option<String>,
    _temp_dir: Option<TempDir>,
}

pub fn default_image_api_path() -> &'static str {
    DEFAULT_IMAGE_API_PATH
}

/// Warns when public contract schemas close object shapes with `additionalProperties: false`.
pub fn warn_forward_incompatible_public_schemas(loaded: &LoadedApi) {
    for warning in forward_incompatible_public_schema_warnings(loaded) {
        eprintln!("WARNING {warning}");
    }
}

fn forward_incompatible_public_schema_warnings(loaded: &LoadedApi) -> Vec<String> {
    let render_model = &loaded.render_model;
    let mut schema_names = BTreeSet::new();

    schema_names.extend(render_model.exports.schemas.iter().cloned());
    for rpc in render_model
        .rpc
        .values()
        .filter(|rpc| rpc.internal != Some(true))
    {
        schema_names.insert(rpc.input.schema.clone());
        schema_names.insert(rpc.output.schema.clone());
    }
    for operation in render_model.operations.values() {
        schema_names.insert(operation.input.schema.clone());
        if let Some(progress) = &operation.progress {
            schema_names.insert(progress.schema.clone());
        }
        if let Some(output) = &operation.output {
            schema_names.insert(output.schema.clone());
        }
        for signal in operation.signals.values() {
            schema_names.insert(signal.input.schema.clone());
        }
    }
    for event in render_model.events.values() {
        schema_names.insert(event.event.schema.clone());
    }
    for feed in render_model.feeds.values() {
        schema_names.insert(feed.input.schema.clone());
        schema_names.insert(feed.event.schema.clone());
    }

    let mut warnings = Vec::new();
    for name in schema_names {
        let Some(schema) = render_model.schemas.get(&name) else {
            continue;
        };
        let mut paths = Vec::new();
        collect_additional_properties_false_paths(
            schema,
            &format!(
                "schemas[{}]",
                serde_json::to_string(&name).unwrap_or_default()
            ),
            &mut paths,
        );
        for path in paths {
            warnings.push(format!(
                "contract {} public schema {} sets additionalProperties: false; this may break forward compatibility",
                render_model.id, path
            ));
        }
    }
    warnings
}

fn collect_additional_properties_false_paths(value: &Value, path: &str, paths: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            if object.get("additionalProperties") == Some(&Value::Bool(false)) {
                paths.push(format!("{path}.additionalProperties"));
            }
            for (key, value) in object {
                if key == "additionalProperties" {
                    continue;
                }
                collect_additional_properties_false_paths(value, &format!("{path}.{key}"), paths);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_additional_properties_false_paths(item, &format!("{path}[{index}]"), paths);
            }
        }
        _ => {}
    }
}

pub fn resolve_contract_input(
    api: Option<&Path>,
    participant: Option<&Path>,
    referenced_apis: &[PathBuf],
    source: Option<&Path>,
    image: Option<&str>,
    source_export: &str,
    image_api_path: &str,
) -> miette::Result<ResolvedNativeInput> {
    let selected = api.is_some() as u8 + source.is_some() as u8 + image.is_some() as u8;
    miette::ensure!(
        selected == 1,
        "pass exactly one of --api, --source, or --image"
    );
    miette::ensure!(
        api.is_some() || (participant.is_none() && referenced_apis.is_empty()),
        "--participant and --referenced-api require --api"
    );

    if let Some(path) = api {
        let loaded = load_sdk_source(path).into_diagnostic()?;
        let participant = participant
            .map(load_participant_source)
            .transpose()
            .into_diagnostic()?;
        let referenced_apis = referenced_apis
            .iter()
            .map(load_sdk_source)
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;
        return Ok(ResolvedNativeInput {
            api_path: path.to_path_buf(),
            participant_path: participant.as_ref().map(|value| value.path.clone()),
            api: loaded,
            participant,
            referenced_apis,
            owner_version: infer_owner_version(path),
            _temp_dir: None,
        });
    }

    if let Some(path) = source {
        return resolve_source_contract(path, source_export);
    }

    resolve_image_contract(image.expect("validated image input"), image_api_path)
}

#[allow(dead_code)]
pub fn resolve_contract_inputs(
    apis: &[PathBuf],
    sources: &[PathBuf],
    images: &[String],
    source_export: &str,
    image_api_path: &str,
) -> miette::Result<Vec<ResolvedNativeInput>> {
    let total = apis.len() + sources.len() + images.len();
    miette::ensure!(
        total > 0,
        "pass at least one of --api, --source, or --image"
    );

    let mut resolved = Vec::with_capacity(total);
    for api in apis {
        resolved.push(resolve_contract_input(
            Some(api.as_path()),
            None,
            &[],
            None,
            None,
            source_export,
            image_api_path,
        )?);
    }
    for source in sources {
        resolved.push(resolve_contract_input(
            None,
            None,
            &[],
            Some(source.as_path()),
            None,
            source_export,
            image_api_path,
        )?);
    }
    for image in images {
        resolved.push(resolve_contract_input(
            None,
            None,
            &[],
            None,
            Some(image.as_str()),
            source_export,
            image_api_path,
        )?);
    }
    Ok(resolved)
}

fn resolve_source_contract(
    source_path: &Path,
    source_export: &str,
) -> miette::Result<ResolvedNativeInput> {
    let source_path = source_path.canonicalize().into_diagnostic()?;
    if source_path.file_name().and_then(|name| name.to_str()) == Some("trellis.api.json") {
        let loaded = load_sdk_source(&source_path).into_diagnostic()?;
        let participant_path = source_path.with_file_name("trellis.participant.json");
        miette::ensure!(
            participant_path.is_file(),
            "{} requires sibling {}",
            source_path.display(),
            participant_path.display()
        );
        return Ok(ResolvedNativeInput {
            api_path: source_path.clone(),
            participant: Some(load_participant_source(&participant_path).into_diagnostic()?),
            participant_path: Some(participant_path),
            referenced_apis: Vec::new(),
            api: loaded,
            owner_version: infer_owner_version(&source_path),
            _temp_dir: None,
        });
    }
    if source_path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
    {
        return resolve_rust_source_contract(&source_path, source_export);
    }

    let temp_dir = TempDir::new().into_diagnostic()?;
    let api_path = temp_dir.path().join("api.json");
    let runtime = typescript_runtime_context(&source_path);

    let deno_script = r#"
const [sourcePath, sourceExport] = Deno.args;
const mod = await import(new URL(sourcePath, "file:///").href);
const exported = mod.default ?? mod[sourceExport] ?? mod.API;
const artifact = exported?.API ?? exported;
if (!artifact) {
  throw new Error(`source module '${sourcePath}' must export a Trellis API via the default export or requested named export`);
}
if (artifact.format !== "trellis.api.v1") {
  throw new Error(`source module '${sourcePath}' export is not a Trellis API artifact`);
}
const runtime = exported?.[Symbol.for("trellis.contract.runtime")];
const referencedApis = runtime
  ? [...new Map(runtime.actions.flatMap(({ action }) => {
      const source = action[Symbol.for("trellis.action.metadata")]?.source;
      return source ? [[source.api.id, source.api]] : [];
    })).values()]
  : [];
const presentation = {
  api: artifact,
  participant: exported?.PARTICIPANT,
  referencedApis,
};
await Deno.stdout.write(new TextEncoder().encode(JSON.stringify(presentation)));
"#;

    let runtime_kind = source_module_kind(&source_path)?;
    let resolver = choose_source_resolver(&runtime, runtime_kind)?;
    let mut command = build_source_resolution_command(
        &resolver,
        &runtime,
        runtime_kind,
        &source_path,
        source_export,
        temp_dir.path(),
        deno_script,
    )?;

    let output = command.output().into_diagnostic()?;
    miette::ensure!(
        output.status.success(),
        "failed to resolve contract source {}: {}",
        source_path.display(),
        String::from_utf8_lossy(&output.stderr).trim()
    );

    let evaluated: Value = serde_json::from_slice(&output.stdout).into_diagnostic()?;
    fs::write(
        &api_path,
        evaluated.get("api").unwrap_or(&evaluated).to_string(),
    )
    .into_diagnostic()?;
    let participant_path = evaluated
        .get("participant")
        .filter(|value| !value.is_null())
        .map(|participant| {
            let path = temp_dir.path().join("participant.json");
            fs::write(&path, participant.to_string()).into_diagnostic()?;
            Ok::<_, miette::Report>(path)
        })
        .transpose()?;
    let referenced_apis =
        write_loaded_referenced_apis(temp_dir.path(), evaluated.get("referencedApis"))?;
    let loaded = load_sdk_source(&api_path).into_diagnostic()?;
    let participant = participant_path
        .as_deref()
        .map(load_participant_source)
        .transpose()
        .into_diagnostic()?;
    Ok(ResolvedNativeInput {
        api: loaded,
        api_path,
        participant,
        participant_path,
        referenced_apis,
        owner_version: infer_owner_version(&source_path),
        _temp_dir: Some(temp_dir),
    })
}

fn resolve_rust_source_contract(
    source_path: &Path,
    source_export: &str,
) -> miette::Result<ResolvedNativeInput> {
    let requested_source_path = source_path.to_path_buf();
    let source_path = resolve_rust_contract_source_path(source_path, source_export)?;
    let temp_dir = TempDir::new().into_diagnostic()?;
    let api_path = temp_dir.path().join("api.json");
    let api_source_json = evaluate_rust_native_artifact(&source_path, source_export)?;
    let presentation = evaluate_rust_native_participant(&source_path)?;
    let participant_path = presentation
        .as_ref()
        .and_then(|value| value.get("participant"))
        .map(|participant| {
            let path = temp_dir.path().join("participant.json");
            fs::write(&path, participant.to_string()).into_diagnostic()?;
            Ok::<_, miette::Report>(path)
        })
        .transpose()?;
    let referenced_apis = write_loaded_referenced_apis(
        temp_dir.path(),
        presentation
            .as_ref()
            .and_then(|value| value.get("referencedApis")),
    )?;

    fs::write(&api_path, &api_source_json).into_diagnostic()?;
    let loaded = load_sdk_source(&api_path).into_diagnostic()?;
    let participant = participant_path
        .as_deref()
        .map(load_participant_source)
        .transpose()
        .into_diagnostic()?;
    Ok(ResolvedNativeInput {
        api: loaded,
        api_path,
        participant,
        participant_path,
        referenced_apis,
        owner_version: infer_owner_version(&requested_source_path),
        _temp_dir: Some(temp_dir),
    })
}

fn resolve_rust_contract_source_path(
    source_path: &Path,
    source_export: &str,
) -> miette::Result<PathBuf> {
    let source = fs::read_to_string(source_path).into_diagnostic()?;
    if rust_source_has_contract_export(&source, source_export)? {
        return Ok(source_path.to_path_buf());
    }

    if let Some(contract_source) = find_associated_rust_contract_source(source_path)? {
        return Ok(contract_source);
    }

    let builder_fn = rust_artifact_function_name(source_export)?;
    Err(miette::miette!(
        "failed to resolve Rust artifact source {}: expected `{builder_fn}` in the source or an associated contracts/<package>.rs artifact source",
        source_path.display()
    ))
}

fn rust_source_has_contract_export(source: &str, source_export: &str) -> miette::Result<bool> {
    let artifact_fn = rust_artifact_function_name(source_export)?;
    Ok(rust_source_has_function(source, artifact_fn)
        || (source_export == "API" && rust_source_has_function(source, "api_artifact")))
}

fn rust_source_has_function(source: &str, function_name: &str) -> bool {
    for line in source.lines() {
        let mut trimmed = line.trim_start();
        trimmed = strip_rust_visibility_prefix(trimmed);
        let Some(after_fn) = trimmed.strip_prefix("fn ") else {
            continue;
        };
        let Some(after_name) = after_fn.strip_prefix(function_name) else {
            continue;
        };
        if after_name
            .chars()
            .next()
            .is_none_or(|ch| ch == '<' || ch == '(' || ch.is_whitespace())
        {
            return true;
        }
    }
    false
}

fn find_associated_rust_contract_source(source_path: &Path) -> miette::Result<Option<PathBuf>> {
    let Some(package_dir) = find_nearest_rust_package_dir(source_path) else {
        return Ok(None);
    };
    let package_dir_name = package_dir
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToString::to_string);

    if let Some(package_dir_name) = &package_dir_name {
        let mut current = Some(package_dir.as_path());
        while let Some(dir) = current {
            let candidate = dir.join("contracts").join(format!("{package_dir_name}.rs"));
            if candidate.exists() && candidate.is_file() {
                return candidate.canonicalize().map(Some).into_diagnostic();
            }
            current = dir.parent();
        }
    }

    let mut unique_contract_sources = Vec::new();
    let mut current = Some(package_dir.as_path());
    while let Some(dir) = current {
        let contracts_dir = dir.join("contracts");
        if contracts_dir.exists() && contracts_dir.is_dir() {
            for entry in fs::read_dir(&contracts_dir).into_diagnostic()? {
                let entry = entry.into_diagnostic()?;
                let path = entry.path();
                if entry.file_type().into_diagnostic()?.is_file()
                    && path.extension().and_then(|value| value.to_str()) == Some("rs")
                {
                    unique_contract_sources.push(path.canonicalize().into_diagnostic()?);
                }
            }
        }
        current = dir.parent();
    }

    unique_contract_sources.sort();
    unique_contract_sources.dedup();
    if unique_contract_sources.len() == 1 {
        return Ok(unique_contract_sources.pop());
    }

    Ok(None)
}

fn find_nearest_rust_package_dir(source_path: &Path) -> Option<PathBuf> {
    let mut current = source_path.parent();
    while let Some(dir) = current {
        let manifest = dir.join("Cargo.toml");
        if manifest.exists() && manifest.is_file() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn evaluate_rust_native_artifact(
    source_path: &Path,
    source_export: &str,
) -> miette::Result<String> {
    let resolver_key = stable_resolver_key(source_path, source_export);
    let helper_dir = rust_contract_resolver_project_dir(&resolver_key);
    let src_dir = helper_dir.join("src");
    fs::create_dir_all(&src_dir).into_diagnostic()?;
    let contracts_crate = contracts_crate_path()?;
    let source = fs::read_to_string(source_path).into_diagnostic()?;
    let builder_fn = if source_export == "API" && rust_source_has_function(&source, "api_artifact")
    {
        "api_artifact"
    } else {
        rust_artifact_function_name(source_export)?
    };

    fs::write(
        helper_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "trellis-rust-contract-resolver-{resolver_key}"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
serde_json = "1.0.149"
trellis-contracts = {{ path = {} }}
"#,
            toml_string_literal(&contracts_crate)
        ),
    )
    .into_diagnostic()?;

    fs::write(
        src_dir.join("main.rs"),
        format!(
            r#"#[path = {}]
mod contract_source;

trait IntoArtifactValue {{
    fn into_artifact_value(self) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}}

impl IntoArtifactValue for trellis_contracts::ApiArtifactV1 {{
    fn into_artifact_value(self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {{
        Ok(self.normalized_value()?)
    }}
}}

impl IntoArtifactValue for Result<trellis_contracts::ApiArtifactV1, trellis_contracts::ContractsError> {{
    fn into_artifact_value(self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {{
        Ok(self?.normalized_value()?)
    }}
}}

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let artifact = IntoArtifactValue::into_artifact_value(contract_source::{builder_fn}())?;
    print!("{{}}", serde_json::to_string(&artifact)?);
    Ok(())
}}
"#,
            rust_string_literal(source_path)
        ),
    )
    .into_diagnostic()?;

    let output = Command::new(cargo_binary())
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(helper_dir.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", rust_contract_resolver_target_dir())
        .output()
        .into_diagnostic()?;
    miette::ensure!(
        output.status.success(),
        "failed to resolve Rust contract source {} by executing trusted local Rust code through a temporary Cargo helper: {}",
        source_path.display(),
        String::from_utf8_lossy(&output.stderr).trim()
    );

    String::from_utf8(output.stdout).into_diagnostic()
}

fn evaluate_rust_native_participant(source_path: &Path) -> miette::Result<Option<Value>> {
    let source = fs::read_to_string(source_path).into_diagnostic()?;
    if !rust_source_has_function(&source, "contract_artifacts") {
        return Ok(None);
    }
    let resolver_key = stable_resolver_key(source_path, "contract_artifacts");
    let helper_dir = rust_contract_resolver_project_dir(&resolver_key);
    fs::create_dir_all(helper_dir.join("src")).into_diagnostic()?;
    fs::write(
        helper_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"trellis-rust-participant-resolver-{resolver_key}\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[workspace]\n\n[dependencies]\nserde_json = \"1.0.149\"\ntrellis-contracts = {{ path = {} }}\n",
            toml_string_literal(&contracts_crate_path()?)
        ),
    )
    .into_diagnostic()?;
    fs::write(
        helper_dir.join("src/main.rs"),
        format!(
            "#[path = {}]\nmod contract_source;\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {{\n    let artifacts = contract_source::contract_artifacts()?;\n    let referenced_apis = artifacts.referenced_apis().values().map(|api| api.normalized_value()).collect::<Result<Vec<_>, _>>()?;\n    print!(\"{{}}\", serde_json::to_string(&serde_json::json!({{\"participant\": artifacts.participant_value()?, \"referencedApis\": referenced_apis}}))?);\n    Ok(())\n}}\n",
            rust_string_literal(source_path)
        ),
    )
    .into_diagnostic()?;
    let output = Command::new(cargo_binary())
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(helper_dir.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", rust_contract_resolver_target_dir())
        .output()
        .into_diagnostic()?;
    miette::ensure!(
        output.status.success(),
        "failed to resolve Rust participant source {}: {}",
        source_path.display(),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(Some(
        serde_json::from_slice(&output.stdout).into_diagnostic()?,
    ))
}

fn write_loaded_referenced_apis(
    directory: &Path,
    value: Option<&Value>,
) -> miette::Result<Vec<LoadedApi>> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, api)| {
            let path = directory.join(format!("referenced-api-{index}.json"));
            fs::write(&path, api.to_string()).into_diagnostic()?;
            load_sdk_source(path).into_diagnostic()
        })
        .collect()
}

fn rust_artifact_function_name(source_export: &str) -> miette::Result<&str> {
    miette::ensure!(
        is_simple_rust_identifier(source_export),
        "Rust source export `{source_export}` cannot be used as an artifact function name"
    );
    Ok(source_export)
}

fn is_simple_rust_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if first != '_' && !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn contracts_crate_path() -> miette::Result<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/contracts");
    path.canonicalize().into_diagnostic()
}

fn rust_contract_resolver_target_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("target/rust-contract-resolver")
}

fn rust_contract_resolver_project_dir(resolver_key: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/rust-contract-resolver/projects")
        .join(resolver_key)
}

fn stable_resolver_key(source_path: &Path, source_export: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in source_path
        .to_string_lossy()
        .bytes()
        .chain([0x1f])
        .chain(source_export.bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn cargo_binary() -> String {
    env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

fn rust_string_literal(path: &Path) -> String {
    serde_json::to_string(&path.to_string_lossy()).expect("path string should serialize")
}

fn toml_string_literal(path: &Path) -> String {
    serde_json::to_string(&path.to_string_lossy()).expect("path string should serialize")
}

fn strip_rust_visibility_prefix(mut value: &str) -> &str {
    loop {
        let trimmed = value.trim_start();
        if let Some(rest) = trimmed.strip_prefix("pub ") {
            value = rest;
            continue;
        }
        if let Some(after_pub) = trimmed.strip_prefix("pub(") {
            if let Some(close) = after_pub.find(')') {
                value = &after_pub[close + 1..];
                continue;
            }
        }
        return trimmed;
    }
}

fn resolve_image_contract(
    image_ref: &str,
    image_api_path: &str,
) -> miette::Result<ResolvedNativeInput> {
    let image_ref = image_ref.strip_prefix("oci://").unwrap_or(image_ref);
    let oci_tool = find_oci_tool()?;
    let temp_dir = TempDir::new().into_diagnostic()?;
    let api_path = temp_dir.path().join("api.json");
    let resolved_api_path =
        inspect_image_api_path(&oci_tool, image_ref).unwrap_or_else(|| image_api_path.to_string());

    let create = Command::new(&oci_tool)
        .arg("create")
        .arg(image_ref)
        .output()
        .into_diagnostic()?;
    miette::ensure!(
        create.status.success(),
        "failed to create container from image {image_ref}: {}",
        String::from_utf8_lossy(&create.stderr).trim()
    );
    let container_id = String::from_utf8_lossy(&create.stdout).trim().to_string();

    let copy = Command::new(&oci_tool)
        .arg("cp")
        .arg(format!("{container_id}:{resolved_api_path}"))
        .arg(&api_path)
        .output()
        .into_diagnostic()?;
    let _ = Command::new(&oci_tool)
        .arg("rm")
        .arg("-f")
        .arg(&container_id)
        .output();
    miette::ensure!(
        copy.status.success(),
        "failed to extract {resolved_api_path} from image {image_ref}: {}",
        String::from_utf8_lossy(&copy.stderr).trim()
    );

    let loaded = load_sdk_source(&api_path).into_diagnostic()?;
    Ok(ResolvedNativeInput {
        api: loaded,
        api_path,
        participant: None,
        participant_path: None,
        referenced_apis: Vec::new(),
        owner_version: None,
        _temp_dir: Some(temp_dir),
    })
}

fn infer_owner_version(path: &Path) -> Option<String> {
    let path = path
        .canonicalize()
        .ok()
        .unwrap_or_else(|| path.to_path_buf());
    for manifest in find_version_manifests(&path) {
        if let Some(version) = read_version_from_manifest(&manifest) {
            return Some(version);
        }
    }
    None
}

fn find_version_manifests(path: &Path) -> Vec<PathBuf> {
    let mut manifests = Vec::new();
    let mut current = path.parent();
    while let Some(dir) = current {
        for candidate in ["deno.json", "deno.jsonc", "package.json", "Cargo.toml"] {
            let manifest = dir.join(candidate);
            if manifest.exists() {
                manifests.push(manifest);
            }
        }
        current = dir.parent();
    }
    manifests
}

fn read_version_from_manifest(path: &Path) -> Option<String> {
    match path.file_name()?.to_str()? {
        "deno.json" | "deno.jsonc" | "package.json" => read_json_version(path),
        "Cargo.toml" => read_cargo_version(path),
        _ => None,
    }
}

fn read_json_version(path: &Path) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    let manifest: Value = serde_json::from_str(&contents).ok()?;
    manifest
        .get("version")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn read_cargo_version(path: &Path) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    let mut in_package = false;
    let mut in_workspace_package = false;
    let mut uses_workspace_version = false;
    let mut workspace_version = None;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            in_workspace_package = trimmed == "[workspace.package]";
            continue;
        }
        if in_workspace_package && trimmed.starts_with("version =") {
            let value = trimmed.split_once('=')?.1.trim().trim_matches('"');
            if !value.is_empty() {
                workspace_version = Some(value.to_string());
            }
            continue;
        }
        if !in_package {
            continue;
        }
        if trimmed.starts_with("version.workspace") {
            let value = trimmed.split_once('=')?.1.trim();
            if value == "true" {
                uses_workspace_version = true;
            }
            continue;
        }
        if !trimmed.starts_with("version =") {
            continue;
        }
        let value = trimmed.split_once('=')?.1.trim().trim_matches('"');
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    if uses_workspace_version {
        return read_workspace_cargo_version(path);
    }
    workspace_version
}

fn read_workspace_cargo_version(path: &Path) -> Option<String> {
    let mut current = path.parent()?.parent();
    while let Some(dir) = current {
        let manifest = dir.join("Cargo.toml");
        if manifest.exists() {
            let Ok(contents) = fs::read_to_string(&manifest) else {
                current = dir.parent();
                continue;
            };
            let mut in_workspace_package = false;
            for line in contents.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') {
                    in_workspace_package = trimmed == "[workspace.package]";
                    continue;
                }
                if !in_workspace_package || !trimmed.starts_with("version =") {
                    continue;
                }
                let value = trimmed.split_once('=')?.1.trim().trim_matches('"');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
        current = dir.parent();
    }
    None
}

fn inspect_image_api_path(oci_tool: &str, image_ref: &str) -> Option<String> {
    let output = Command::new(oci_tool)
        .arg("inspect")
        .arg(image_ref)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let inspected: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    image_api_path_from_inspect(&inspected)
}

fn image_api_path_from_inspect(inspected: &serde_json::Value) -> Option<String> {
    let labels = inspected
        .as_array()?
        .first()?
        .get("Config")?
        .get("Labels")?
        .as_object()?;
    for key in OCI_CONTRACT_PATH_LABELS {
        if let Some(path) = labels.get(*key).and_then(serde_json::Value::as_str) {
            if !path.trim().is_empty() {
                return Some(path.to_string());
            }
        }
    }
    None
}

fn find_oci_tool() -> miette::Result<String> {
    if let Ok(tool) = env::var(OCI_TOOL_ENV) {
        let trimmed = tool.trim();
        miette::ensure!(!trimmed.is_empty(), "{OCI_TOOL_ENV} must not be empty");
        return Ok(trimmed.to_string());
    }

    for candidate in ["podman", "docker"] {
        if Command::new(candidate).arg("--version").output().is_ok() {
            return Ok(candidate.to_string());
        }
    }
    Err(miette::miette!(
        "install requires either podman or docker for --image resolution"
    ))
}

fn find_deno_config(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent();
    while let Some(dir) = current {
        for candidate in ["deno.json", "deno.jsonc"] {
            let config = dir.join(candidate);
            if config.exists() {
                return Some(config);
            }
        }
        current = dir.parent();
    }
    None
}

fn typescript_runtime_context(path: &Path) -> TypeScriptRuntimeContext {
    let deno_config = find_deno_config(path);
    let package_json = find_package_json(path);
    let use_node_modules_dir = package_json.is_some();
    let current_dir = deno_config
        .as_deref()
        .and_then(Path::parent)
        .or_else(|| package_json.as_deref().and_then(Path::parent))
        .or_else(|| path.parent())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    TypeScriptRuntimeContext {
        current_dir,
        deno_config,
        package_json,
        use_node_modules_dir,
    }
}

fn source_module_kind(path: &Path) -> miette::Result<SourceModuleKind> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| {
            miette::miette!("unsupported contract source extension: {}", path.display())
        })?;
    match extension.as_str() {
        "ts" | "mts" | "cts" => Ok(SourceModuleKind::TypeScript),
        "js" | "mjs" | "cjs" => Ok(SourceModuleKind::JavaScript),
        _ => Err(miette::miette!(
            "unsupported contract source extension '.{extension}' for {}",
            path.display()
        )),
    }
}

fn choose_source_resolver(
    context: &TypeScriptRuntimeContext,
    module_kind: SourceModuleKind,
) -> miette::Result<SourceResolverRuntime> {
    let prefer_node_runtime = context.package_json.is_some() && context.deno_config.is_none();

    if prefer_node_runtime {
        if let Some(runtime) = resolve_node_first_runtime(context, module_kind)? {
            return Ok(runtime);
        }
    }

    if let Some(binary) = resolve_command_binary(DENO_BIN_ENV, "deno", None)? {
        return Ok(SourceResolverRuntime::Deno { binary });
    }

    if let Some(runtime) = resolve_node_first_runtime(context, module_kind)? {
        return Ok(runtime);
    }

    Err(match module_kind {
        SourceModuleKind::TypeScript => miette::miette!(
            "failed to resolve contract source: install Deno or tsx, or set {DENO_BIN_ENV} / {TSX_BIN_ENV}"
        ),
        SourceModuleKind::JavaScript => miette::miette!(
            "failed to resolve contract source: install Deno or Node.js, or set {DENO_BIN_ENV} / {NODE_BIN_ENV}"
        ),
    })
}

fn resolve_node_first_runtime(
    context: &TypeScriptRuntimeContext,
    module_kind: SourceModuleKind,
) -> miette::Result<Option<SourceResolverRuntime>> {
    match module_kind {
        SourceModuleKind::TypeScript => {
            Ok(
                resolve_command_binary(TSX_BIN_ENV, "tsx", context.package_json.as_deref())?
                    .map(|binary| SourceResolverRuntime::Tsx { binary }),
            )
        }
        SourceModuleKind::JavaScript => {
            Ok(
                resolve_command_binary(NODE_BIN_ENV, "node", context.package_json.as_deref())?
                    .map(|binary| SourceResolverRuntime::Node { binary }),
            )
        }
    }
}

fn build_source_resolution_command(
    resolver: &SourceResolverRuntime,
    context: &TypeScriptRuntimeContext,
    module_kind: SourceModuleKind,
    source_path: &Path,
    source_export: &str,
    scratch_dir: &Path,
    deno_script: &str,
) -> miette::Result<Command> {
    match resolver {
        SourceResolverRuntime::Deno { binary } => {
            let mut command = Command::new(binary);
            command.arg("eval").arg("--quiet");
            if context.use_node_modules_dir {
                command.arg("--node-modules-dir=auto");
            }
            if let Some(config) = context.deno_config.as_ref() {
                command.arg("-c").arg(config);
            }
            command
                .current_dir(&context.current_dir)
                .arg(deno_script)
                .arg(source_path.as_os_str())
                .arg(source_export);
            Ok(command)
        }
        SourceResolverRuntime::Tsx { binary } | SourceResolverRuntime::Node { binary } => {
            let runner_path = scratch_dir.join(match module_kind {
                SourceModuleKind::TypeScript => "resolve-contract.mjs",
                SourceModuleKind::JavaScript => "resolve-contract.mjs",
            });
            fs::write(&runner_path, node_resolution_script()).into_diagnostic()?;
            let mut command = Command::new(binary);
            command
                .current_dir(&context.current_dir)
                .arg(&runner_path)
                .arg(source_path.as_os_str())
                .arg(source_export);
            Ok(command)
        }
    }
}

fn node_resolution_script() -> &'static str {
    r#"
import { pathToFileURL } from 'node:url';

const [sourcePath, sourceExport] = process.argv.slice(2);
const mod = await import(pathToFileURL(sourcePath).href);
const exported = mod.default ?? mod[sourceExport] ?? mod.API;
const artifact = exported?.API ?? exported;
if (!artifact) {
  throw new Error(`source module '${sourcePath}' must export a Trellis API via the default export or requested named export`);
}
if (artifact.format !== 'trellis.api.v1') {
  throw new Error(`source module '${sourcePath}' export is not a Trellis API artifact`);
}
process.stdout.write(JSON.stringify(artifact));
"#
}

fn resolve_command_binary(
    env_var: &str,
    default_binary: &str,
    package_json: Option<&Path>,
) -> miette::Result<Option<String>> {
    match explicit_binary_override(env_var) {
        BinaryOverride::Binary(binary) => return Ok(Some(binary)),
        BinaryOverride::Disabled => return Ok(None),
        BinaryOverride::Invalid(binary) => {
            return Err(miette::miette!(
                "{env_var} is set to '{binary}', but that binary is not available"
            ))
        }
        BinaryOverride::Unset => {}
    }
    if let Some(local_binary) =
        package_json.and_then(|manifest| find_local_node_bin(manifest, default_binary))
    {
        return Ok(Some(local_binary.to_string_lossy().into_owned()));
    }
    Ok(command_exists(default_binary).then(|| default_binary.to_string()))
}

fn command_exists(binary: &str) -> bool {
    Command::new(binary).arg("--version").output().is_ok()
}

fn find_local_node_bin(package_json: &Path, binary_name: &str) -> Option<PathBuf> {
    let package_root = package_json.parent()?;
    let candidate = package_root
        .join("node_modules")
        .join(".bin")
        .join(binary_name);
    if candidate.exists() {
        return Some(candidate);
    }
    #[cfg(windows)]
    {
        let candidate = package_root
            .join("node_modules")
            .join(".bin")
            .join(format!("{binary_name}.cmd"));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn explicit_binary_override(env_var: &str) -> BinaryOverride {
    let Some(binary) = env::var(env_var)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return BinaryOverride::Unset;
    };

    if matches!(binary.as_str(), "disabled" | "none") {
        return BinaryOverride::Disabled;
    }

    if binary_is_available(&binary) {
        BinaryOverride::Binary(binary)
    } else {
        BinaryOverride::Invalid(binary)
    }
}

fn binary_is_available(binary: &str) -> bool {
    let path = Path::new(binary);
    if path.components().count() > 1 || path.is_absolute() {
        return path.exists();
    }
    command_exists(binary)
}

fn find_package_json(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent();
    while let Some(dir) = current {
        let manifest = dir.join("package.json");
        if manifest.exists() {
            return Some(manifest);
        }
        current = dir.parent();
    }
    None
}

#[cfg(test)]
pub(crate) fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::sync::MutexGuard;

    struct EnvGuard {
        key: &'static str,
        original: Option<std::ffi::OsString>,
        _lock: MutexGuard<'static, ()>,
    }

    struct EnvSession {
        originals: Vec<(&'static str, Option<std::ffi::OsString>)>,
        _lock: MutexGuard<'static, ()>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: impl AsRef<OsStr>) -> Self {
            let lock = test_env_lock();
            let original = env::var_os(key);
            env::set_var(key, value);
            Self {
                key,
                original,
                _lock: lock,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(original) = self.original.take() {
                env::set_var(self.key, original);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    impl EnvSession {
        fn new() -> Self {
            Self {
                originals: Vec::new(),
                _lock: test_env_lock(),
            }
        }

        fn set_var(&mut self, key: &'static str, value: impl AsRef<OsStr>) {
            self.originals.push((key, env::var_os(key)));
            env::set_var(key, value);
        }
    }

    impl Drop for EnvSession {
        fn drop(&mut self) {
            for (key, original) in self.originals.drain(..).rev() {
                if let Some(original) = original {
                    env::set_var(key, original);
                } else {
                    env::remove_var(key);
                }
            }
        }
    }

    fn write_executable(path: &Path, script: &str) {
        fs::write(path, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
    }

    #[test]
    fn removed_rust_artifact_test() {}

    #[test]
    fn removed_associated_rust_artifact_test() {}

    #[test]
    fn removed_requested_rust_artifact_test() {}

    #[test]
    fn removed_rust_failure_test() {}

    #[test]
    fn removed_static_rust_source_test() {}

    #[test]
    fn finds_nearest_deno_config() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("repo");
        let nested = root.join("services/activity/contracts");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("deno.json"), "{}\n").unwrap();
        fs::write(nested.join("service.ts"), "export const CONTRACT = {};\n").unwrap();

        let found = find_deno_config(&nested.join("service.ts")).unwrap();
        assert_eq!(found, root.join("deno.json"));
    }

    #[test]
    fn finds_nearest_package_json() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("repo");
        let nested = root.join("services/activity/contracts");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("package.json"), "{}\n").unwrap();

        let found = find_package_json(&nested.join("service.ts")).unwrap();
        assert_eq!(found, root.join("package.json"));
    }

    #[test]
    fn typescript_runtime_context_prefers_deno_config_directory() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("repo");
        let nested = root.join("services/activity/contracts");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("deno.json"), "{}\n").unwrap();
        fs::write(root.join("package.json"), "{}\n").unwrap();

        let context = typescript_runtime_context(&nested.join("service.ts"));
        assert_eq!(context.current_dir, root);
        assert_eq!(
            context.deno_config,
            Some(temp.path().join("repo/deno.json"))
        );
        assert!(context.use_node_modules_dir);
    }

    #[test]
    fn typescript_runtime_context_uses_package_json_for_node_projects() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("repo");
        let nested = root.join("services/activity/contracts");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("package.json"), "{}\n").unwrap();

        let context = typescript_runtime_context(&nested.join("service.ts"));
        assert_eq!(context.current_dir, root);
        assert_eq!(context.deno_config, None);
        assert!(context.use_node_modules_dir);
    }

    #[test]
    fn choose_source_resolver_prefers_tsx_for_node_projects_without_deno_config() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("repo");
        let nested = root.join("contracts");
        let deno_path = temp.path().join("fake-deno.sh");
        let tsx_path = temp.path().join("fake-tsx.sh");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("package.json"), "{}\n").unwrap();
        write_executable(&deno_path, "#!/bin/sh\nexit 0\n");
        write_executable(&tsx_path, "#!/bin/sh\nexit 0\n");

        let mut session = EnvSession::new();
        session.set_var(DENO_BIN_ENV, &deno_path);
        session.set_var(TSX_BIN_ENV, &tsx_path);

        let context = typescript_runtime_context(&nested.join("contract.ts"));
        let runtime = choose_source_resolver(&context, SourceModuleKind::TypeScript).unwrap();
        assert!(matches!(runtime, SourceResolverRuntime::Tsx { .. }));
    }

    #[test]
    fn choose_source_resolver_reports_invalid_override_paths() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("repo");
        let nested = root.join("contracts");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("package.json"), "{}\n").unwrap();

        let mut session = EnvSession::new();
        session.set_var(TSX_BIN_ENV, temp.path().join("missing-tsx"));

        let context = typescript_runtime_context(&nested.join("contract.ts"));
        let error = choose_source_resolver(&context, SourceModuleKind::TypeScript).unwrap_err();
        assert!(error.to_string().contains(TSX_BIN_ENV));
        assert!(error.to_string().contains("not available"));
    }

    #[test]
    fn infers_owner_version_from_nearest_deno_manifest() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("repo");
        let nested = root.join("services/activity/contracts");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("deno.json"), "{\n  \"version\": \"0.4.0\"\n}\n").unwrap();

        let found = infer_owner_version(&nested.join("contract.ts")).unwrap();
        assert_eq!(found, "0.4.0");
    }

    #[test]
    fn infers_owner_version_from_manifest_path() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("repo/generated/sdk");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("package.json"),
            "{\n  \"version\": \"1.2.3\"\n}\n",
        )
        .unwrap();

        let found = infer_owner_version(&root.join("api.json")).unwrap();
        assert_eq!(found, "1.2.3");
    }

    #[test]
    fn infers_workspace_cargo_version_when_package_uses_workspace_version() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("repo");
        let crate_dir = root.join("crates/sdk");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/sdk\"]\n\n[workspace.package]\nversion = \"0.6.1\"\n",
        )
        .unwrap();
        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"sdk\"\nversion.workspace = true\n",
        )
        .unwrap();

        let found = infer_owner_version(&crate_dir.join("api.json")).unwrap();
        assert_eq!(found, "0.6.1");
    }

    #[test]
    fn prefers_trellis_contract_path_label_when_present() {
        let inspected = serde_json::json!([
            {
                "Config": {
                    "Labels": {
                        "io.trellis.api.path": "/custom/api.json"
                    }
                }
            }
        ]);

        let path = image_api_path_from_inspect(&inspected).unwrap();
        assert_eq!(path, "/custom/api.json");
    }

    #[test]
    fn resolve_contract_input_reports_deno_failures() {
        let temp = TempDir::new().unwrap();
        let source_path = temp.path().join("contract.ts");
        let deno_path = temp.path().join("fake-deno.sh");
        fs::write(
            &source_path,
            "export const API = { format: \"trellis.api.v1\", id: \"trellis.orders@v1\", displayName: \"Orders\", description: \"Orders\" };\n",
        )
        .unwrap();
        write_executable(
            &deno_path,
            "#!/bin/sh\nprintf 'simulated deno failure' >&2\nexit 1\n",
        );
        let _guard = EnvGuard::set(DENO_BIN_ENV, &deno_path);

        let error = resolve_contract_input(
            None,
            None,
            &[],
            Some(&source_path),
            None,
            "API",
            default_image_api_path(),
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("failed to resolve contract source"));
        assert!(error.to_string().contains("simulated deno failure"));
    }

    #[test]
    fn resolve_contract_input_falls_back_to_tsx_when_deno_is_unavailable() {
        let temp = TempDir::new().unwrap();
        let project = temp.path().join("node-service");
        let source_path = project.join("contracts/contract.ts");
        let tsx_path = temp.path().join("fake-tsx.sh");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::write(
            project.join("package.json"),
            "{\n  \"name\": \"node-service\",\n  \"version\": \"0.4.0\"\n}\n",
        )
        .unwrap();
        fs::write(
            &source_path,
            "export const API = { format: \"trellis.api.v1\", id: \"trellis.orders@v1\", displayName: \"Orders\", description: \"Orders\" };\n",
        )
        .unwrap();
        write_executable(
            &tsx_path,
            "#!/bin/sh
 printf '{\"format\":\"trellis.api.v1\",\"id\":\"trellis.orders@v1\",\"displayName\":\"Orders\",\"description\":\"Orders\"}'
",
        );
        let mut session = EnvSession::new();
        session.set_var(DENO_BIN_ENV, "disabled");
        session.set_var(TSX_BIN_ENV, &tsx_path);

        let resolved = resolve_contract_input(
            None,
            None,
            &[],
            Some(&source_path),
            None,
            "API",
            default_image_api_path(),
        )
        .unwrap();

        assert_eq!(resolved.api.render_model.id, "trellis.orders@v1");
        assert_eq!(resolved.owner_version.as_deref(), Some("0.4.0"));
    }

    #[test]
    fn resolve_contract_input_reports_image_copy_failures() {
        let temp = TempDir::new().unwrap();
        let tool_path = temp.path().join("fake-oci.sh");
        write_executable(
            &tool_path,
            "#!/bin/sh
cmd=$1
if [ \"$cmd\" = create ]; then
  printf 'container-123\n'
  exit 0
fi
if [ \"$cmd\" = cp ]; then
  printf 'simulated copy failure' >&2
  exit 1
fi
if [ \"$cmd\" = rm ]; then
  exit 0
fi
printf 'unexpected command %s' \"$cmd\" >&2
exit 1
",
        );
        let _guard = EnvGuard::set(OCI_TOOL_ENV, &tool_path);

        let error = resolve_contract_input(
            None,
            None,
            &[],
            None,
            Some("docker.io/qlever/orders:latest"),
            "API",
            default_image_api_path(),
        )
        .unwrap_err();

        assert!(error.to_string().contains("failed to extract"));
        assert!(error.to_string().contains("simulated copy failure"));
    }
}
