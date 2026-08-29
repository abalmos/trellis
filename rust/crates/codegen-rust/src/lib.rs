//! Rust SDK generation from canonical Trellis contract manifests.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use serde_json::Value;
use trellis_contracts::{
    canonicalize_json, load_participant_source as load_native_participant, load_sdk_source,
    ContractBuilder, ContractKind, ContractsError, LoadedApi, LoadedParticipant,
    ParticipantUseRenderModel,
};

/// Errors returned while generating a Rust SDK crate.
#[derive(thiserror::Error, Debug)]
pub enum CodegenRustError {
    #[error("contracts error: {0}")]
    Contracts(#[from] trellis_contracts::ContractsError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("missing runtime repo root for local runtime source")]
    MissingRuntimeRepoRoot,

    #[error("participant mapping alias '{alias}' is not declared in the participant uses")]
    UnknownParticipantMappingAlias { alias: String },

    #[error("participant uses alias '{alias}' for contract '{contract}' requires an explicit alias mapping")]
    MissingParticipantMappingAlias { alias: String, contract: String },

    #[error("participant mapping alias '{alias}' targets contract '{actual_contract}', expected '{expected_contract}'")]
    InvalidParticipantMappingContract {
        alias: String,
        expected_contract: String,
        actual_contract: String,
    },

    #[error("participant mapping alias '{alias}' does not expose rpc '{key}'")]
    MissingMappedRpc { alias: String, key: String },

    #[error("participant mapping alias '{alias}' does not expose operation '{key}'")]
    MissingMappedOperation { alias: String, key: String },

    #[error("participant mapping alias '{alias}' does not expose event '{key}'")]
    MissingMappedEvent { alias: String, key: String },

    #[error("participant mapping alias '{alias}' cannot publish owner-only event '{key}'")]
    OwnerOnlyMappedEvent { alias: String, key: String },

    #[error("participant mapping alias '{alias}' does not expose feed '{key}'")]
    MissingMappedFeed { alias: String, key: String },

    #[error("workspace does not declare package '{package_name}'")]
    MissingWorkspacePackage { package_name: String },

    #[error("workspace member '{member}' is missing a [package].name declaration")]
    MissingWorkspaceMemberPackageName { member: String },

    #[error("invalid generated Rust source for {path}: {message}")]
    RustSyntax { path: String, message: String },

    #[error("failed to format generated Rust source for {path}: {message}")]
    RustFormat { path: String, message: String },

    #[error(
        "generated Rust identifier collision in {scope}: '{identifier}' comes from {originals:?}"
    )]
    IdentifierCollision {
        scope: String,
        identifier: String,
        originals: Vec<String>,
    },

    #[error("unsupported schema at {path}: {reason}")]
    UnsupportedSchema { path: String, reason: String },

    #[error("participant contract '{contract}' owns public surfaces but has no generated owner SDK mapping")]
    MissingOwnedSdk { contract: String },
}

/// Options for generating one Rust SDK crate.
#[derive(Debug, Clone)]
pub struct GenerateRustSdkOpts {
    /// Canonical native API to load.
    pub api_path: PathBuf,
    /// Directory where the crate will be written.
    pub out_dir: PathBuf,
    /// Cargo crate name for the generated SDK.
    pub crate_name: String,
    /// Crate version written into `Cargo.toml`.
    pub crate_version: String,
    /// How generated code should depend on Trellis runtime crates.
    pub runtime_deps: RustRuntimeDeps,
}

/// One explicit `uses` alias mapping for participant-facade generation.
#[derive(Debug, Clone)]
pub struct ParticipantAliasMapping {
    /// Local `uses` alias from the participant manifest.
    pub alias: String,
    /// Crate name that satisfies the alias at compile time.
    pub crate_name: String,
    /// Native dependency API; used to validate exposed RPCs/events.
    pub api_path: PathBuf,
    /// Optional local crate path override.
    ///
    /// When omitted, the generator assumes the dependency crate lives next to
    /// the provided API path.
    pub crate_path: Option<PathBuf>,
    /// Optional Cargo dependency value copied from the participant's manifest.
    ///
    /// Local generated SDK mappings use `crate_path`; independently published
    /// SDKs use this exact dependency value instead.
    pub cargo_dependency: Option<String>,
}

/// Options for generating one local Rust participant facade crate.
#[derive(Debug, Clone)]
pub struct GenerateRustParticipantFacadeOpts {
    /// Native API artifact implemented by the participant.
    pub api_path: PathBuf,
    /// Native participant artifact that owns the facade.
    pub participant_path: PathBuf,
    /// Output directory for the generated crate.
    pub out_dir: PathBuf,
    /// Cargo crate name for the facade crate.
    pub crate_name: String,
    /// Crate version written into `Cargo.toml`.
    pub crate_version: String,
    /// How generated code should depend on Trellis runtime crates.
    pub runtime_deps: RustRuntimeDeps,
    /// Optional owned SDK crate name to import from generated facade code.
    pub owned_sdk_crate_name: Option<String>,
    /// Optional path to the owned SDK crate used during local generation.
    pub owned_sdk_path: Option<PathBuf>,
    /// Explicit mappings for locally resolvable `uses` aliases declared by the participant.
    pub alias_mappings: Vec<ParticipantAliasMapping>,
}

/// Runtime dependency configuration for generated Rust SDKs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustRuntimeDeps {
    /// Whether dependencies come from crates.io or the local repo.
    pub source: RustRuntimeSource,
    /// Version string used for registry dependencies.
    pub version: String,
    /// Repo root required when `source` is `Local`.
    pub repo_root: Option<PathBuf>,
}

/// Where generated SDKs should resolve Trellis runtime crates from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustRuntimeSource {
    Registry,
    Local,
}

#[derive(Debug, Deserialize)]
struct WorkspaceManifest {
    workspace: WorkspaceSection,
}

#[derive(Debug, Deserialize)]
struct WorkspaceSection {
    members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PackageManifest {
    package: Option<PackageSection>,
}

#[derive(Debug, Deserialize)]
struct PackageSection {
    name: String,
}

fn workspace_package_dir(
    repo_root: &Path,
    package_name: &str,
) -> Result<PathBuf, CodegenRustError> {
    let workspace_manifest_path = repo_root.join("rust/Cargo.toml");
    let workspace_manifest: WorkspaceManifest =
        toml::from_str(&fs::read_to_string(&workspace_manifest_path)?)?;

    for member in workspace_manifest.workspace.members {
        let member_manifest_path = repo_root.join("rust").join(&member).join("Cargo.toml");
        let member_manifest: PackageManifest =
            toml::from_str(&fs::read_to_string(&member_manifest_path)?)?;
        let package = member_manifest.package.ok_or_else(|| {
            CodegenRustError::MissingWorkspaceMemberPackageName {
                member: member.clone(),
            }
        })?;
        if package.name == package_name {
            return Ok(member_manifest_path
                .parent()
                .expect("Cargo.toml should always have a parent directory")
                .to_path_buf());
        }
    }

    Err(CodegenRustError::MissingWorkspacePackage {
        package_name: package_name.to_string(),
    })
}

/// Derive the default Rust SDK crate name for a contract id.
pub fn default_sdk_crate_name(contract_id: &str) -> String {
    format!("trellis-sdk-{}", default_sdk_stem(contract_id))
}

/// Derive the default Rust SDK stem used for crate and facade naming.
pub fn default_sdk_stem(contract_id: &str) -> String {
    let stem = contract_id
        .split('@')
        .next()
        .unwrap_or("trellis-sdk")
        .replace('.', "-");
    stem.strip_prefix("trellis-").unwrap_or(&stem).to_string()
}

/// Generate a Rust SDK crate for one manifest.
pub fn generate_rust_sdk(opts: &GenerateRustSdkOpts) -> Result<(), CodegenRustError> {
    replace_generated_dir(&opts.out_dir, |staging_dir| {
        let mut staged = opts.clone();
        staged.out_dir = staging_dir.to_path_buf();
        generate_rust_sdk_into(&staged)?;
        format_generated_rust_files(staging_dir)
    })
}

fn generate_rust_sdk_into(opts: &GenerateRustSdkOpts) -> Result<(), CodegenRustError> {
    let loaded = load_sdk_source(&opts.api_path)?;
    let artifact_canonical = loaded.canonical.clone();
    let artifact_digest = loaded.digest.clone();
    validate_generated_identifiers(&loaded)?;
    validate_supported_schemas(&loaded)?;

    fs::create_dir_all(opts.out_dir.join("src"))?;
    let mut cargo_toml = render_cargo_toml(
        opts,
        !loaded.render_model.feeds.is_empty() || !loaded.render_model.events.is_empty(),
        is_trellis_owned_sdk_contract(&loaded.render_model.id),
    )?;
    cargo_toml.push_str(&format!(
        "\n[package.metadata.trellis]\napi-id = {}\napi-digest = {}\napi-artifact = \"api.json\"\n",
        string_literal(&loaded.render_model.id),
        string_literal(&artifact_digest),
    ));
    write_if_changed(&opts.out_dir.join("Cargo.toml"), &cargo_toml)?;
    write_if_changed(
        &opts.out_dir.join("api.json"),
        &format!("{artifact_canonical}\n"),
    )?;
    write_if_changed(
        &opts.out_dir.join("TRELLIS.md"),
        &render_rust_sdk_trellis_md(opts, &loaded),
    )?;
    write_if_changed(
        &opts.out_dir.join("README.md"),
        &format!(
            "# {}\n\nGenerated Rust SDK for `{}`.\n\nThis crate contains contract types and typed adapters. Connect through your generated participant facade.\n",
            opts.crate_name, loaded.render_model.id
        ),
    )?;
    write_rust_if_changed(
        &opts.out_dir.join("src").join("api.rs"),
        &render_api_rs(opts, &loaded, &artifact_canonical, &artifact_digest),
    )?;
    remove_if_exists(&opts.out_dir.join("src").join("contract.rs"))?;
    remove_if_exists(&opts.out_dir.join("contract.json"))?;
    write_rust_if_changed(
        &opts.out_dir.join("src").join("types.rs"),
        &render_types_rs(&loaded),
    )?;
    write_rust_if_changed(
        &opts.out_dir.join("src").join("rpc.rs"),
        &render_rpc_rs(&loaded),
    )?;
    write_rust_if_changed(
        &opts.out_dir.join("src").join("operations.rs"),
        &render_operations_rs(&loaded),
    )?;
    remove_if_exists(&opts.out_dir.join("src").join("jobs.rs"))?;
    write_rust_if_changed(
        &opts.out_dir.join("src").join("events.rs"),
        &render_events_rs(&loaded),
    )?;
    write_rust_if_changed(
        &opts.out_dir.join("src").join("feeds.rs"),
        &render_feeds_rs(&loaded),
    )?;
    write_rust_if_changed(
        &opts.out_dir.join("src").join("schemas.rs"),
        &render_schemas_rs(&loaded),
    )?;
    write_rust_if_changed(
        &opts.out_dir.join("src").join("client.rs"),
        &render_client_rs(&loaded),
    )?;
    remove_if_exists(&opts.out_dir.join("src").join("connect.rs"))?;
    remove_if_exists(&opts.out_dir.join("src").join("server.rs"))?;
    write_rust_if_changed(
        &opts.out_dir.join("src").join("lib.rs"),
        &render_lib_rs(&loaded),
    )?;

    Ok(())
}

fn is_trellis_owned_sdk_contract(contract_id: &str) -> bool {
    matches!(
        contract_id,
        "trellis.auth@v1"
            | "trellis.core@v1"
            | "trellis.health@v1"
            | "trellis.jobs@v1"
            | "trellis.state@v1"
    )
}

/// Validate the minimal generated Rust SDK manifest invariants used by freshness checks.
pub fn rust_sdk_cargo_manifest_is_valid(
    cargo_toml_path: &Path,
    crate_name: &str,
    crate_version: &str,
) -> bool {
    let Ok(contents) = fs::read_to_string(cargo_toml_path) else {
        return false;
    };
    let Ok(manifest) = contents.parse::<toml::Table>() else {
        return false;
    };
    let Some(package) = manifest.get("package").and_then(toml::Value::as_table) else {
        return false;
    };
    let Some(dependencies) = manifest.get("dependencies").and_then(toml::Value::as_table) else {
        return false;
    };

    package.get("name").and_then(toml::Value::as_str) == Some(crate_name)
        && package.get("version").and_then(toml::Value::as_str) == Some(crate_version)
        && ["serde", "serde_json", "trellis-rs", "trellis-contracts"]
            .into_iter()
            .all(|dependency| dependencies.contains_key(dependency))
}

fn generate_rust_participant_generated_sources(
    opts: &GenerateRustParticipantFacadeOpts,
) -> Result<(), CodegenRustError> {
    let api = load_sdk_source(&opts.api_path)?;
    let participant = load_participant_source(&opts.participant_path)?;
    let mappings = validate_participant_mappings(&participant, &opts.alias_mappings)?;

    fs::create_dir_all(opts.out_dir.join("src/uses"))?;
    write_rust_if_changed(
        &opts.out_dir.join("src/facade.rs"),
        &render_participant_facade_rs(&participant, &mappings),
    )?;
    write_rust_if_changed(
        &opts.out_dir.join("src/owned.rs"),
        &render_participant_owned_rs(
            &api,
            &participant,
            opts.owned_sdk_crate_name.as_deref(),
            mappings
                .iter()
                .map(|mapping| Ok((mapping.contract_id.clone(), mapping.manifest.value.clone())))
                .collect::<Result<std::collections::BTreeMap<_, _>, CodegenRustError>>()?,
        )?,
    )?;
    write_rust_if_changed(
        &opts.out_dir.join("src/schemas.rs"),
        &render_schemas_rs(&api),
    )?;
    write_rust_if_changed(
        &opts.out_dir.join("src/state.rs"),
        &render_participant_state_rs(&participant),
    )?;
    if participant.render_model.jobs.is_empty() {
        remove_if_exists(&opts.out_dir.join("src/jobs.rs"))?;
    } else {
        write_rust_if_changed(
            &opts.out_dir.join("src/jobs.rs"),
            &format!(
                "{}\n{}",
                render_participant_job_descriptors_rs(&participant),
                render_participant_jobs_facade_rs(&participant),
            ),
        )?;
    }
    if participant.render_model.event_consumers.is_empty() {
        remove_if_exists(&opts.out_dir.join("src/event_consumers.rs"))?;
    } else {
        write_rust_if_changed(
            &opts.out_dir.join("src/event_consumers.rs"),
            &render_participant_event_consumers_rs(
                &participant,
                &mappings,
                opts.owned_sdk_crate_name.as_deref(),
            ),
        )?;
    }
    write_rust_if_changed(
        &opts.out_dir.join("src/uses/mod.rs"),
        &render_participant_uses_mod_rs(&mappings),
    )?;

    for mapping in &mappings {
        write_rust_if_changed(
            &opts
                .out_dir
                .join("src/uses")
                .join(format!("{}.rs", key_to_snake(&mapping.alias))),
            &render_participant_use_alias_rs(mapping),
        )?;
    }

    Ok(())
}

/// Generate a complete local Rust participant-facade crate.
///
/// The facade crate wraps one owned participant contract plus explicit `uses`
/// alias mappings so local development can type-check the full integration.
pub fn generate_rust_participant_facade(
    opts: &GenerateRustParticipantFacadeOpts,
) -> Result<(), CodegenRustError> {
    replace_generated_dir(&opts.out_dir, |staging_dir| {
        let mut staged = opts.clone();
        staged.out_dir = staging_dir.to_path_buf();
        generate_rust_participant_facade_into(&staged)?;
        format_generated_rust_files(staging_dir)
    })
}

fn generate_rust_participant_facade_into(
    opts: &GenerateRustParticipantFacadeOpts,
) -> Result<(), CodegenRustError> {
    let api = load_sdk_source(&opts.api_path)?;
    let participant = load_participant_source(&opts.participant_path)?;
    if participant_requires_owned_sdk(&api, &participant)
        && (opts.owned_sdk_crate_name.is_none() || opts.owned_sdk_path.is_none())
    {
        return Err(CodegenRustError::MissingOwnedSdk {
            contract: participant.render_model.id.clone(),
        });
    }
    let mappings = validate_participant_mappings(&participant, &opts.alias_mappings)?;
    let native_artifacts = build_participant_artifacts(&api, &participant, &mappings)?;
    let apis_dir = opts.out_dir.join("apis");
    if apis_dir.exists() {
        fs::remove_dir_all(&apis_dir)?;
    }
    let legacy_contracts_dir = opts.out_dir.join("contracts");
    if legacy_contracts_dir.exists() {
        fs::remove_dir_all(legacy_contracts_dir)?;
    }
    fs::create_dir_all(opts.out_dir.join("src"))?;
    fs::create_dir_all(&apis_dir)?;
    write_if_changed(
        &opts.out_dir.join("Cargo.toml"),
        &render_participant_cargo_toml(opts, &mappings, !api.render_model.feeds.is_empty())?,
    )?;
    write_if_changed(
        &opts.out_dir.join("TRELLIS.md"),
        &render_rust_participant_trellis_md(opts, &api, &participant, &mappings),
    )?;
    for mapping in &mappings {
        fs::copy(
            &mapping.manifest.path,
            apis_dir.join(format!("{}.json", mapping.alias_ident)),
        )?;
    }
    remove_if_exists(&opts.out_dir.join("build.rs"))?;
    write_rust_if_changed(
        &opts.out_dir.join("src/lib.rs"),
        &render_participant_shim_lib_rs(&participant),
    )?;
    write_rust_if_changed(
        &opts.out_dir.join("src/connect.rs"),
        &render_participant_connect_rs(&participant, &mappings),
    )?;
    write_rust_if_changed(
        &opts.out_dir.join("src/contract.rs"),
        &render_participant_contract_rs(&participant, &native_artifacts)?,
    )?;
    generate_rust_participant_generated_sources(opts)?;

    Ok(())
}

fn build_participant_artifacts(
    api: &LoadedApi,
    participant: &LoadedParticipant,
    mappings: &[ValidatedParticipantAlias],
) -> Result<trellis_contracts::ContractArtifacts, CodegenRustError> {
    let mut referenced_apis = std::collections::BTreeMap::new();
    for mapping in mappings {
        let value = mapping.manifest.value.clone();
        let id = value["id"].as_str().expect("validated API id");
        referenced_apis.insert(id.to_owned(), value);
    }
    Ok(
        ContractBuilder::from_native(api.value.clone(), participant.value.clone())
            .referenced_apis(referenced_apis)
            .build()?,
    )
}

/// Build the canonical participant artifact for a generated participant facade.
pub fn native_participant_artifact(
    api_path: &Path,
    participant_path: &Path,
    alias_mappings: &[ParticipantAliasMapping],
) -> Result<(String, String), CodegenRustError> {
    let api = load_sdk_source(api_path)?;
    let participant = load_participant_source(participant_path)?;
    let mappings = validate_participant_mappings(&participant, alias_mappings)?;
    let artifacts = build_participant_artifacts(&api, &participant, &mappings)?;
    let participant = artifacts.participant_value()?;
    Ok((
        canonicalize_json(&participant)?,
        artifacts.participant_digest()?,
    ))
}

fn load_participant_source(participant_path: &Path) -> Result<LoadedParticipant, ContractsError> {
    load_native_participant(participant_path)
}

fn participant_requires_owned_sdk(api: &LoadedApi, participant: &LoadedParticipant) -> bool {
    !api.render_model.rpc.is_empty()
        || !api.render_model.operations.is_empty()
        || !api.render_model.events.is_empty()
        || !api.render_model.feeds.is_empty()
        || !participant.render_model.jobs.is_empty()
}

fn replace_generated_dir(
    out_dir: &Path,
    generate: impl FnOnce(&Path) -> Result<(), CodegenRustError>,
) -> Result<(), CodegenRustError> {
    let parent = out_dir.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let name = out_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("generated");
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let staging = parent.join(format!(".{name}.staging-{}-{nonce}", std::process::id()));
    let backup = parent.join(format!(".{name}.backup-{}-{nonce}", std::process::id()));
    remove_dir_if_exists(&staging)?;
    remove_dir_if_exists(&backup)?;

    if let Err(error) = generate(&staging) {
        remove_dir_if_exists(&staging)?;
        return Err(error);
    }

    if out_dir.exists() {
        fs::rename(out_dir, &backup)?;
    }
    if let Err(error) = fs::rename(&staging, out_dir) {
        if backup.exists() {
            fs::rename(&backup, out_dir)?;
        }
        return Err(CodegenRustError::Io(error));
    }
    remove_dir_if_exists(&backup)?;
    Ok(())
}

fn remove_dir_if_exists(path: &Path) -> Result<(), CodegenRustError> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn render_cargo_toml(
    opts: &GenerateRustSdkOpts,
    has_feeds: bool,
    publish_false: bool,
) -> Result<String, CodegenRustError> {
    let mut dependency_lines = runtime_dependency_lines(&opts.runtime_deps, &opts.out_dir)?;
    if has_feeds {
        dependency_lines.push("futures-util = \"0.3\"".to_string());
    }
    dependency_lines.sort();
    let description = format!("Generated Rust SDK crate for {}.", opts.crate_name);
    let publish_line = if publish_false {
        "publish = false\n"
    } else {
        ""
    };
    Ok(format!(
        "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2021\"\nlicense = \"Apache-2.0\"\nrepository = \"https://github.com/qlever-llc/trellis\"\ndescription = \"{}\"\nreadme = \"README.md\"\n{}\n[dependencies]\nserde = {{ version = \"1.0\", features = [\"derive\"] }}\nserde_json = \"1.0\"\n{}\n",
        opts.crate_name,
        opts.crate_version,
        description,
        publish_line,
        dependency_lines.join("\n"),
    ))
}

fn runtime_dependency_lines(
    runtime_deps: &RustRuntimeDeps,
    out_dir: &Path,
) -> Result<Vec<String>, CodegenRustError> {
    match runtime_deps.source {
        RustRuntimeSource::Registry => Ok(vec![
            format!("trellis-rs = \"{}\"", runtime_deps.version),
            format!("trellis-contracts = \"{}\"", runtime_deps.version),
        ]),
        RustRuntimeSource::Local => {
            let repo_root = runtime_deps
                .repo_root
                .as_ref()
                .ok_or(CodegenRustError::MissingRuntimeRepoRoot)?;
            let repo_root = fs::canonicalize(repo_root).unwrap_or_else(|_| repo_root.clone());
            let trellis_path = workspace_package_dir(&repo_root, "trellis-rs")?;
            let contracts_path = workspace_package_dir(&repo_root, "trellis-contracts")?;
            let trellis_path = relative_path(out_dir, &trellis_path);
            let contracts_path = relative_path(out_dir, &contracts_path);
            Ok(vec![
                format!(
                    "trellis-rs = {{ path = {} }}",
                    string_literal(&trellis_path.display().to_string())
                ),
                format!(
                    "trellis-contracts = {{ path = {} }}",
                    string_literal(&contracts_path.display().to_string())
                ),
            ])
        }
    }
}

fn render_rust_sdk_trellis_md(opts: &GenerateRustSdkOpts, loaded: &LoadedApi) -> String {
    let mut lines = vec![
        format!(
            "# Trellis API Guide: {}", loaded.render_model.id
        ),
        String::new(),
        "This file is generated for AI agents and out-of-tree Trellis services.".to_string(),
        String::new(),
        "## Global Trellis Context".to_string(),
        String::new(),
        "- llms.txt: https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms.txt".to_string(),
        "- llms-full.txt: https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms-full.txt".to_string(),
        String::new(),
        "## Crate".to_string(),
        String::new(),
        format!("- crate: `{}`", opts.crate_name),
        format!("- contract id: `{}`", loaded.render_model.id),
        String::new(),
        "## Rust Facades".to_string(),
        String::new(),
        "Owned surfaces:".to_string(),
    ];
    push_rust_owned_surfaces(&mut lines, loaded, "crate", false);
    lines.extend([
        String::new(),
        "Dependency surfaces are declared by participant packages.".to_string(),
    ]);
    push_rust_prepared_events(&mut lines);
    lines.join("\n") + "\n"
}

fn render_rust_participant_trellis_md(
    opts: &GenerateRustParticipantFacadeOpts,
    api: &LoadedApi,
    participant: &LoadedParticipant,
    mappings: &[ValidatedParticipantAlias],
) -> String {
    let mut lines = vec![
        format!("# Trellis Participant Guide: {}", participant.render_model.id),
        String::new(),
        "This file is generated for AI agents and out-of-tree Trellis services.".to_string(),
        String::new(),
        "## Global Trellis Context".to_string(),
        String::new(),
        "- llms.txt: https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms.txt".to_string(),
        "- llms-full.txt: https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms-full.txt".to_string(),
        String::new(),
        "## Crate".to_string(),
        String::new(),
        format!("- crate: `{}`", opts.crate_name),
        format!("- contract id: `{}`", participant.render_model.id),
        format!("- kind: `{:?}`", participant.render_model.kind),
        String::new(),
        "## Participant Facades".to_string(),
        String::new(),
        "Owned surfaces are available through `connected_service.service().owned()`, `connected_service.handle()`, and `connected_client.client().owned()`:".to_string(),
    ];
    push_rust_owned_surfaces(&mut lines, api, "owned_sdk", true);
    lines.extend([String::new(), "Mapped dependency aliases:".to_string()]);
    if mappings.is_empty() {
        lines.push("- No mapped dependency aliases.".to_string());
    } else {
        for mapping in mappings {
            lines.push(format!(
                "- alias `{}` -> crate `{}` contract `{}`",
                mapping.alias, mapping.crate_name, mapping.contract_id
            ));
            push_rust_used_mapping_surfaces(&mut lines, mapping);
        }
    }
    push_rust_prepared_events(&mut lines);
    lines.join("\n") + "\n"
}

fn push_rust_owned_surfaces(
    lines: &mut Vec<String>,
    loaded: &LoadedApi,
    crate_prefix: &str,
    include_service_handlers: bool,
) {
    for key in public_rpc_keys(loaded) {
        let base = key_to_pascal(key);
        let (group, method) = surface_group_and_method(key);
        let handler = if include_service_handlers {
            format!(", service handler `service.handle().rpc().{group}().{method}(handler)`")
        } else {
            String::new()
        };
        lines.push(format!("- RPC `{key}`: descriptor `{crate_prefix}::rpc::{base}Rpc`, low-level `trellis_client.call::<{crate_prefix}::rpc::{base}Rpc>(...)`, generated wrapper `.rpc().{group}().{method}(...)`{handler}"));
    }
    for key in loaded.render_model.events.keys() {
        let base = key_to_pascal(key);
        let (group, method) = surface_group_and_method(key);
        lines.push(format!("- Event `{key}`: `trellis_client.publish::<{crate_prefix}::events::{base}EventDescriptor>(...)`, generated wrapper `.event().{group}().{method}().publish(...)`, prepare with `trellis_client.prepare_event::<{crate_prefix}::events::{base}EventDescriptor>(...)`"));
    }
    for key in loaded.render_model.feeds.keys() {
        let base = key_to_pascal(key);
        let (group, method) = surface_group_and_method(key);
        let handler = if include_service_handlers {
            format!(", service handler `service.handle().feed().{group}().{method}(handler)`")
        } else {
            String::new()
        };
        lines.push(format!("- Feed `{key}`: `trellis_client.feed::<{crate_prefix}::feeds::{base}FeedDescriptor>(input)`, generated wrapper `.feed().{group}().{method}(...)`{handler}"));
    }
    for key in loaded.render_model.operations.keys() {
        let base = key_to_pascal(key);
        let (group, method) = surface_group_and_method(key);
        let provider = if include_service_handlers {
            format!(
                ", service provider `service.handle().operation().{group}().{method}(provider)`"
            )
        } else {
            String::new()
        };
        lines.push(format!("- Operation `{key}`: `trellis_client.operation::<{crate_prefix}::operations::{base}Operation>().start(...)`, generated wrapper `.operation().{group}().{method}().start(...)`{provider}"));
    }
    if public_rpc_keys(loaded).is_empty()
        && loaded.render_model.events.is_empty()
        && loaded.render_model.feeds.is_empty()
        && loaded.render_model.operations.is_empty()
    {
        lines.push("- No owned RPC, event, feed, or operation surfaces.".to_string());
    }
}

fn push_rust_used_mapping_surfaces(lines: &mut Vec<String>, mapping: &ValidatedParticipantAlias) {
    push_rust_use_ref_lines(lines, &mapping.use_ref, &mapping.crate_ident);
}

fn push_rust_use_ref_lines(
    lines: &mut Vec<String>,
    use_ref: &ParticipantUseRenderModel,
    crate_prefix: &str,
) {
    if let Some(rpc) = &use_ref.rpc {
        for key in rpc.call.as_deref().unwrap_or(&[]) {
            let base = key_to_pascal(key);
            let (group, method) = surface_group_and_method(key);
            lines.push(format!("  - RPC call `{key}`: `trellis_client.call::<{crate_prefix}::rpc::{base}Rpc>(...)` or generated wrapper `.rpc().{group}().{method}(...)`"));
        }
    }
    if let Some(operations) = &use_ref.operations {
        for key in operations.invoke.as_deref().unwrap_or(&[]) {
            let base = key_to_pascal(key);
            let (group, method) = surface_group_and_method(key);
            lines.push(format!("  - Operation call `{key}`: `trellis_client.operation::<{crate_prefix}::operations::{base}Operation>().start(...)` or generated wrapper `.operation().{group}().{method}().start(...)`"));
        }
    }
    if let Some(events) = &use_ref.events {
        for key in events.publish.as_deref().unwrap_or(&[]) {
            let base = key_to_pascal(key);
            let (group, method) = surface_group_and_method(key);
            lines.push(format!("  - Event publish `{key}`: `trellis_client.publish::<{crate_prefix}::events::{base}EventDescriptor>(...)` or generated wrapper `.event().{group}().{method}().publish(...)`"));
        }
        for key in events.subscribe.as_deref().unwrap_or(&[]) {
            let base = key_to_pascal(key);
            lines.push(format!("  - Event subscribe `{key}`: `trellis_client.subscribe::<{crate_prefix}::events::{base}EventDescriptor>(...)`"));
        }
    }
    if let Some(feeds) = &use_ref.feeds {
        for key in feeds.subscribe.as_deref().unwrap_or(&[]) {
            let base = key_to_pascal(key);
            let (group, method) = surface_group_and_method(key);
            lines.push(format!("  - Feed subscribe `{key}`: `trellis_client.feed::<{crate_prefix}::feeds::{base}FeedDescriptor>(input)` or generated wrapper `.feed().{group}().{method}(...)`"));
        }
    }
}

fn push_rust_prepared_events(lines: &mut Vec<String>) {
    lines.extend([
        String::new(),
        "Prepared events and outbox/inbox:".to_string(),
        "- Generated event structs are event bodies only; runtime metadata is separate from the body payload.".to_string(),
        "- `PreparedTrellisEvent` captures a validated subject, encoded body payload, preserved transport headers, event id, and event time.".to_string(),
        "- Published prepared events send the runtime event id from `event_id()` and `Trellis-Event-Time` from `event_time()`.".to_string(),
        "- Use `subscribe_messages::<Descriptor>(...)` and `EventMessage::event_id()` / `event_time()` when subscribers need metadata.".to_string(),
        "- Use `prepare_event::<Descriptor>(...)`, `publish_prepared(...)`, and `dispatch_outbox_once(...)` for durable publish flows.".to_string(),
        "- Runtime stores include `OutboxStore`, `InboxStore`, `SqliteOutboxStore`, `SqliteInboxStore`, `PostgresOutboxStore`, and `PostgresInboxStore`.".to_string(),
        String::new(),
    ]);
}

#[derive(Debug, Clone)]
struct ValidatedParticipantAlias {
    alias: String,
    alias_ident: String,
    crate_name: String,
    crate_ident: String,
    crate_path: PathBuf,
    cargo_dependency: Option<String>,
    contract_id: String,
    manifest: trellis_contracts::LoadedApi,
    use_ref: ParticipantUseRenderModel,
}

fn validate_participant_mappings(
    local: &LoadedParticipant,
    mappings: &[ParticipantAliasMapping],
) -> Result<Vec<ValidatedParticipantAlias>, CodegenRustError> {
    let mut validated = Vec::new();
    let mut mapped_aliases = std::collections::BTreeSet::new();
    let mut generated_aliases = std::collections::BTreeMap::<String, Vec<String>>::new();

    for mapping in mappings {
        generated_aliases
            .entry(rust_ident(&key_to_snake(&mapping.alias)))
            .or_default()
            .push(mapping.alias.clone());
        let use_ref = local.render_model.uses.get(&mapping.alias).ok_or_else(|| {
            CodegenRustError::UnknownParticipantMappingAlias {
                alias: mapping.alias.clone(),
            }
        })?;
        let manifest = load_sdk_source(&mapping.api_path)?;
        if manifest.render_model.id != use_ref.api {
            return Err(CodegenRustError::InvalidParticipantMappingContract {
                alias: mapping.alias.clone(),
                expected_contract: use_ref.api.clone(),
                actual_contract: manifest.render_model.id.clone(),
            });
        }

        if let Some(rpc) = &use_ref.rpc {
            for key in rpc.call.as_deref().unwrap_or(&[]) {
                if !manifest.render_model.rpc.contains_key(key) {
                    return Err(CodegenRustError::MissingMappedRpc {
                        alias: mapping.alias.clone(),
                        key: key.clone(),
                    });
                }
            }
        }
        if let Some(operations) = &use_ref.operations {
            for key in operations.invoke.as_deref().unwrap_or(&[]) {
                if !manifest.render_model.operations.contains_key(key) {
                    return Err(CodegenRustError::MissingMappedOperation {
                        alias: mapping.alias.clone(),
                        key: key.clone(),
                    });
                }
            }
        }
        if let Some(events) = &use_ref.events {
            for key in events.publish.as_deref().unwrap_or(&[]) {
                let Some(event) = manifest.render_model.events.get(key) else {
                    return Err(CodegenRustError::MissingMappedEvent {
                        alias: mapping.alias.clone(),
                        key: key.clone(),
                    });
                };
                if event
                    .capabilities
                    .as_ref()
                    .is_some_and(|capabilities| capabilities.publish.is_none())
                {
                    return Err(CodegenRustError::OwnerOnlyMappedEvent {
                        alias: mapping.alias.clone(),
                        key: key.clone(),
                    });
                }
            }
            for key in events.subscribe.as_deref().unwrap_or(&[]) {
                if !manifest.render_model.events.contains_key(key) {
                    return Err(CodegenRustError::MissingMappedEvent {
                        alias: mapping.alias.clone(),
                        key: key.clone(),
                    });
                }
            }
        }
        if let Some(feeds) = &use_ref.feeds {
            for key in feeds.subscribe.as_deref().unwrap_or(&[]) {
                if !manifest.render_model.feeds.contains_key(key) {
                    return Err(CodegenRustError::MissingMappedFeed {
                        alias: mapping.alias.clone(),
                        key: key.clone(),
                    });
                }
            }
        }
        validated.push(ValidatedParticipantAlias {
            alias: mapping.alias.clone(),
            alias_ident: rust_ident(&key_to_snake(&mapping.alias)),
            crate_name: mapping.crate_name.clone(),
            crate_ident: crate_ident(&mapping.crate_name),
            crate_path: mapping.crate_path.clone().unwrap_or_else(|| {
                mapping
                    .api_path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .to_path_buf()
            }),
            cargo_dependency: mapping.cargo_dependency.clone(),
            contract_id: manifest.render_model.id.clone(),
            manifest,
            use_ref: use_ref.clone(),
        });
        mapped_aliases.insert(mapping.alias.clone());
    }

    if let Some((identifier, originals)) = generated_aliases
        .into_iter()
        .find(|(_, originals)| originals.len() > 1)
    {
        return Err(CodegenRustError::IdentifierCollision {
            scope: "participant aliases".to_string(),
            identifier,
            originals,
        });
    }

    for (alias, use_ref) in local.render_model.uses.iter() {
        if !mapped_aliases.contains(alias)
            && participant_use_requires_mapping(local, alias, use_ref)
        {
            return Err(CodegenRustError::MissingParticipantMappingAlias {
                alias: alias.clone(),
                contract: use_ref.api.clone(),
            });
        }
    }

    validated.sort_by(|left, right| left.alias.cmp(&right.alias));
    Ok(validated)
}

fn validate_generated_identifiers(
    loaded: &trellis_contracts::LoadedApi,
) -> Result<(), CodegenRustError> {
    for (scope, keys) in [
        (
            "RPC types",
            loaded.render_model.rpc.keys().collect::<Vec<_>>(),
        ),
        (
            "operation types",
            loaded.render_model.operations.keys().collect::<Vec<_>>(),
        ),
        (
            "event types",
            loaded.render_model.events.keys().collect::<Vec<_>>(),
        ),
        (
            "feed types",
            loaded.render_model.feeds.keys().collect::<Vec<_>>(),
        ),
    ] {
        reject_identifier_collisions(
            scope,
            keys.into_iter()
                .map(|key| (key.to_string(), key_to_pascal(key))),
        )?;
    }

    reject_identifier_collisions(
        "RPC methods",
        loaded.render_model.rpc.keys().map(|key| {
            let (group, method) = surface_group_and_method(key);
            (key.clone(), format!("{group}::{method}"))
        }),
    )?;

    for (schema_name, schema) in &loaded.render_model.schemas {
        validate_schema_field_identifiers(schema, &format!("schemas.{schema_name}"))?;
    }
    Ok(())
}

fn validate_schema_field_identifiers(
    schema: &serde_json::Value,
    path: &str,
) -> Result<(), CodegenRustError> {
    if let Some(properties) = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
    {
        reject_identifier_collisions(
            path,
            properties
                .keys()
                .map(|field| (field.clone(), rust_ident(&rust_schema_field_base(field)))),
        )?;
        for (field, child) in properties {
            validate_schema_field_identifiers(child, &format!("{path}.properties.{field}"))?;
        }
    }
    for keyword in ["items", "additionalProperties"] {
        if let Some(child) = schema.get(keyword) {
            validate_schema_field_identifiers(child, &format!("{path}.{keyword}"))?;
        }
    }
    for keyword in ["allOf", "anyOf", "oneOf"] {
        if let Some(children) = schema.get(keyword).and_then(serde_json::Value::as_array) {
            for (index, child) in children.iter().enumerate() {
                validate_schema_field_identifiers(child, &format!("{path}.{keyword}[{index}]"))?;
            }
        }
    }
    Ok(())
}

fn reject_identifier_collisions(
    scope: impl Into<String>,
    identifiers: impl IntoIterator<Item = (String, String)>,
) -> Result<(), CodegenRustError> {
    let scope = scope.into();
    let mut grouped = std::collections::BTreeMap::<String, Vec<String>>::new();
    for (original, generated) in identifiers {
        grouped.entry(generated).or_default().push(original);
    }
    if let Some((identifier, originals)) = grouped
        .into_iter()
        .find(|(_, originals)| originals.len() > 1)
    {
        return Err(CodegenRustError::IdentifierCollision {
            scope,
            identifier,
            originals,
        });
    }
    Ok(())
}

fn validate_supported_schemas(
    loaded: &trellis_contracts::LoadedApi,
) -> Result<(), CodegenRustError> {
    for (name, schema) in &loaded.render_model.schemas {
        validate_supported_schema(schema, &format!("schemas.{name}"))?;
    }
    Ok(())
}

fn validate_supported_schema(
    schema: &serde_json::Value,
    path: &str,
) -> Result<(), CodegenRustError> {
    for keyword in ["anyOf", "oneOf"] {
        if let Some(variants) = schema.get(keyword).and_then(serde_json::Value::as_array) {
            let non_null = variants
                .iter()
                .filter(|variant| !is_null_schema(variant))
                .collect::<Vec<_>>();
            if non_null.len() > 1
                && !matches!(
                    union_base_type(schema),
                    Some("String" | "bool" | "i64" | "f64")
                )
                && string_enum_values(schema).is_none()
                && tagged_object_union(schema).is_none()
                && object_union_variants(schema).is_none()
            {
                return Err(CodegenRustError::UnsupportedSchema {
                    path: format!("{path}.{keyword}"),
                    reason: "ambiguous union cannot be represented faithfully".to_string(),
                });
            }
            for (index, variant) in variants.iter().enumerate() {
                validate_supported_schema(variant, &format!("{path}.{keyword}[{index}]"))?;
            }
        }
    }
    if let Some(properties) = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
    {
        for (field, child) in properties {
            validate_supported_schema(child, &format!("{path}.properties.{field}"))?;
        }
    }
    if let Some(items) = schema.get("items") {
        validate_supported_schema(items, &format!("{path}.items"))?;
    }
    Ok(())
}

/// Return whether a participant `uses` alias requires an explicit local SDK mapping.
pub fn participant_use_requires_mapping(
    local: &LoadedParticipant,
    alias: &str,
    use_ref: &ParticipantUseRenderModel,
) -> bool {
    !is_runtime_owned_baseline_use(local, alias, use_ref)
}

fn is_runtime_owned_baseline_use(
    local: &LoadedParticipant,
    alias: &str,
    use_ref: &ParticipantUseRenderModel,
) -> bool {
    if alias == "state"
        && use_ref.api == "trellis.state@v1"
        && !local.render_model.state.is_empty()
        && use_ref.operations.is_none()
        && use_ref.events.is_none()
    {
        return use_ref.rpc.as_ref().is_some_and(|rpc| {
            rpc.call.as_deref().unwrap_or(&[]).iter().all(|key| {
                matches!(
                    key.as_str(),
                    "State.Get" | "State.Put" | "State.Delete" | "State.List"
                )
            })
        });
    }

    false
}

fn render_participant_cargo_toml(
    opts: &GenerateRustParticipantFacadeOpts,
    mappings: &[ValidatedParticipantAlias],
    has_owned_feeds: bool,
) -> Result<String, CodegenRustError> {
    let mut dependency_lines =
        participant_runtime_dependency_lines(&opts.runtime_deps, &opts.out_dir)?;
    if has_owned_feeds
        || mappings.iter().any(|mapping| {
            let subscribes_events = mapping
                .use_ref
                .events
                .as_ref()
                .and_then(|events| events.subscribe.as_ref())
                .is_some_and(|subscribe| !subscribe.is_empty());
            let subscribes_feeds = mapping
                .use_ref
                .feeds
                .as_ref()
                .and_then(|feeds| feeds.subscribe.as_ref())
                .is_some_and(|subscribe| !subscribe.is_empty());
            subscribes_events || subscribes_feeds
        })
    {
        dependency_lines.push("futures-util = \"0.3\"".to_string());
    }
    if mappings.iter().any(|mapping| {
        mapping
            .use_ref
            .rpc
            .as_ref()
            .and_then(|rpc| rpc.call.as_ref())
            .is_some_and(|calls| {
                calls
                    .iter()
                    .any(|key| mapping.manifest.render_model.rpc[key].transfer.is_some())
            })
    }) {
        dependency_lines.push("tokio = { version = \"1\", features = [\"io-util\"] }".to_string());
    }
    if let (Some(crate_name), Some(path)) = (&opts.owned_sdk_crate_name, &opts.owned_sdk_path) {
        let path = relative_path(&opts.out_dir, path);
        dependency_lines.push(format!(
            "{} = {{ path = {} }}",
            crate_name,
            string_literal(&path.display().to_string())
        ));
    }
    for mapping in mappings {
        if let Some(dependency) = &mapping.cargo_dependency {
            dependency_lines.push(format!("{} = {dependency}", mapping.crate_name));
            continue;
        }
        let path = relative_path(&opts.out_dir, &mapping.crate_path);
        dependency_lines.push(format!(
            "{} = {{ path = {} }}",
            mapping.crate_name,
            string_literal(&path.display().to_string())
        ));
    }
    dependency_lines.sort();

    Ok(format!(
        "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2021\"\nlicense = \"Apache-2.0\"\npublish = false\n\n[dependencies]\nserde = {{ version = \"1.0\", features = [\"derive\"] }}\nserde_json = \"1.0\"\n{}\n",
        opts.crate_name,
        opts.crate_version,
        dependency_lines.join("\n")
    ))
}

fn participant_runtime_dependency_lines(
    runtime_deps: &RustRuntimeDeps,
    out_dir: &Path,
) -> Result<Vec<String>, CodegenRustError> {
    runtime_dependency_lines(runtime_deps, out_dir)
}

fn relative_path(from_dir: &Path, target: &Path) -> PathBuf {
    let from_dir = fs::canonicalize(from_dir).unwrap_or_else(|_| from_dir.to_path_buf());
    let target = fs::canonicalize(target).unwrap_or_else(|_| target.to_path_buf());
    let from = from_dir.components().collect::<Vec<_>>();
    let to = target.components().collect::<Vec<_>>();
    let common = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    let mut path = PathBuf::new();
    for _ in common..from.len() {
        path.push("..");
    }
    for component in &to[common..] {
        path.push(component.as_os_str());
    }
    path
}

fn render_participant_shim_lib_rs(loaded: &LoadedParticipant) -> String {
    let jobs_module = if loaded.render_model.jobs.is_empty() {
        ""
    } else {
        "pub mod jobs;\n"
    };
    let event_consumers_module = if loaded.render_model.event_consumers.is_empty() {
        ""
    } else {
        "pub mod event_consumers;\n"
    };
    let exports = match loaded.render_model.kind {
        ContractKind::Service => {
            "pub use connect::{connect, ConnectedService, Contract, ServiceConnectOptions};\npub use trellis_rs::service::{GeneratedServiceContract, ServiceHandlerContext, ServiceRuntimeError};"
        }
        ContractKind::App | ContractKind::Agent => {
            "pub use connect::{connect, ConnectOptions, ConnectedClient};"
        }
        ContractKind::Device => {
            "pub use connect::{connect, ConnectOptions, ConnectedClient, Contract};"
        }
    };
    format!(
        "//! Generated Rust participant facade crate.\n\nconst _: () = trellis_rs::generated::assert_abi(1);\n\npub mod connect;\npub mod contract;\n{event_consumers_module}{jobs_module}include!(\"facade.rs\");\n{exports}\n"
    )
}

fn render_participant_event_consumers_rs(
    loaded: &LoadedParticipant,
    mappings: &[ValidatedParticipantAlias],
    owned_sdk_crate_name: Option<&str>,
) -> String {
    let mut lines = vec![
        "//! Generated durable event-consumer facade.".to_string(),
        String::new(),
        "/// Durable event consumers declared by this participant contract.".to_string(),
        "pub struct EventConsumers<'a> { service: &'a crate::ConnectedService }".to_string(),
        "impl EventConsumers<'_> {".to_string(),
    ];
    for group in loaded.render_model.event_consumers.keys() {
        let method = rust_ident(&key_to_snake(group));
        let group_type = format!("{}Consumer", key_to_pascal(group));
        lines.push(format!("    /// Access the `{group}` consumer group.\n    pub fn {method}(&self) -> {group_type}<'_> {{ {group_type} {{ service: self.service }} }}"));
    }
    lines.extend([
        "}".to_string(),
        String::new(),
        "impl crate::ConnectedService {".to_string(),
        "    /// Access durable event consumers declared by this participant contract.".to_string(),
        "    pub fn event_consumers(&self) -> EventConsumers<'_> { EventConsumers { service: self } }"
            .to_string(),
        "}".to_string(),
        String::new(),
    ]);

    for (group, spec) in &loaded.render_model.event_consumers {
        let group_type = format!("{}Consumer", key_to_pascal(group));
        lines.push(format!(
            "/// Typed registrations for the `{group}` consumer group."
        ));
        lines.push(format!(
            "pub struct {group_type}<'a> {{ service: &'a crate::ConnectedService }}"
        ));
        lines.push(format!("impl {group_type}<'_> {{"));
        for key in &spec.self_events {
            let base = key_to_pascal(key);
            let method = rust_ident(&key_to_snake(key));
            let sdk = crate_ident(owned_sdk_crate_name.expect("self event requires owner SDK"));
            lines.push(render_event_consumer_method(
                group,
                &method,
                &sdk,
                &base,
                &format!("{sdk}::API_ID"),
            ));
        }
        for (alias, keys) in &spec.uses {
            let mapping = mappings
                .iter()
                .find(|mapping| mapping.alias == *alias)
                .expect("validated event consumer alias mapping");
            let sdk = crate_ident(&mapping.crate_name);
            for key in keys {
                let base = key_to_pascal(key);
                let method = rust_ident(&key_to_snake(key));
                let api_identity = format!("{sdk}::API_ID");
                lines.push(render_event_consumer_method(
                    group,
                    &method,
                    &sdk,
                    &base,
                    &api_identity,
                ));
            }
        }
        lines.extend(["}".to_string(), String::new()]);
    }
    format!("{}\n", lines.join("\n"))
}

fn render_event_consumer_method(
    group: &str,
    method: &str,
    sdk: &str,
    base: &str,
    api_identity: &str,
) -> String {
    format!("    /// Register a typed `{base}` event handler.\n    pub async fn {method}<F, Fut>(&self, handler: F) -> Result<trellis_rs::service::ServiceEventListenerHandle, crate::ServiceRuntimeError> where F: Fn({sdk}::{base}Event, trellis_rs::service::ServiceEventListenerContext) -> Fut + Send + Sync + 'static, Fut: std::future::Future<Output = Result<(), trellis_rs::service::ServerError>> + Send + 'static {{ self.service.inner.listen_event_with_api_id::<{sdk}::events::{base}EventDescriptor, _, _>({api_identity}, handler, trellis_rs::service::ServiceEventListenOptions {{ group: Some({group:?}.to_string()), ..Default::default() }}).await }}")
}

fn render_participant_jobs_facade_rs(loaded: &LoadedParticipant) -> String {
    let mut lines = vec![
        "// Generated service-private jobs facade.".to_string(),
        String::new(),
        "use trellis_rs::service::{ActiveJob, JobRef, JobsError};".to_string(),
        String::new(),
        "/// Service-private jobs declared by this participant contract.".to_string(),
        "pub struct Jobs<'a> { service: &'a mut crate::ConnectedService }".to_string(),
        "/// Cloneable service-private job submission facade.".to_string(),
        "#[derive(Clone)]".to_string(),
        "pub struct JobsClient { handle: trellis_rs::service::ServiceHandle }".to_string(),
        "impl<'a> Jobs<'a> {".to_string(),
    ];
    for key in loaded.render_model.jobs.keys() {
        let method = key_to_snake(key);
        let queue = format!("{}Queue", key_to_pascal(key));
        lines.push(format!(
            "    /// Access the `{key}` queue.\n    pub fn {method}(&mut self) -> {queue}<'_> {{ {queue} {{ service: self.service }} }}"
        ));
    }
    lines.extend([
        "}".to_string(),
        String::new(),
        "impl JobsClient {".to_string(),
    ]);
    for key in loaded.render_model.jobs.keys() {
        let method = key_to_snake(key);
        let queue = format!("{}QueueClient", key_to_pascal(key));
        lines.push(format!(
            "    /// Access the `{key}` queue for submission.\n    pub fn {method}(&self) -> {queue} {{ {queue} {{ handle: self.handle.clone() }} }}"
        ));
    }
    lines.extend([
        "}".to_string(),
        String::new(),
        "impl crate::ConnectedService {".to_string(),
        "    /// Access service-private jobs declared by this participant contract.".to_string(),
        "    pub fn jobs(&mut self) -> Jobs<'_> { Jobs { service: self } }".to_string(),
        "    /// Clone a service-private job submission facade for handlers and background tasks."
            .to_string(),
        "    pub fn jobs_client(&self) -> JobsClient { JobsClient { handle: self.inner.generated_handle() } }"
            .to_string(),
        "}".to_string(),
        String::new(),
    ]);
    for key in loaded.render_model.jobs.keys() {
        let base = key_to_pascal(key);
        let queue = format!("{base}Queue");
        let descriptor = format!("crate::jobs::{base}Job");
        lines.extend([
            format!("/// Typed `{key}` jobs queue."),
            format!("pub struct {queue}<'a> {{ service: &'a mut crate::ConnectedService }}"),
            format!("impl {queue}<'_> {{"),
            format!("    /// Submit one `{key}` job.\n    pub async fn submit(&self, payload: <{descriptor} as JobDescriptor>::Payload) -> Result<JobRef<<{descriptor} as JobDescriptor>::Payload, <{descriptor} as JobDescriptor>::Result>, JobsError> {{ self.service.inner.generated_submit_job::<{descriptor}>(payload).await }}"),
            format!("    /// Register the worker handler for `{key}`.\n    pub async fn handle<H, Fut, E>(&mut self, handler: H) -> Result<(), crate::ServiceRuntimeError> where H: Fn(ActiveJob<<{descriptor} as JobDescriptor>::Payload, <{descriptor} as JobDescriptor>::Result>) -> Fut + Clone + Send + Sync + 'static, Fut: std::future::Future<Output = Result<<{descriptor} as JobDescriptor>::Result, E>> + Send + 'static, E: ToString + Send + 'static {{ self.service.runtime_mut().register_generated_job_worker::<{descriptor}, _, _, _>(handler).await }}"),
            "}".to_string(),
            String::new(),
            format!("/// Cloneable `{key}` job submission queue."),
            "#[derive(Clone)]".to_string(),
            format!("pub struct {base}QueueClient {{ handle: trellis_rs::service::ServiceHandle }}"),
            format!("impl {base}QueueClient {{"),
            format!("    /// Submit one `{key}` job.\n    pub async fn submit(&self, payload: <{descriptor} as JobDescriptor>::Payload) -> Result<JobRef<<{descriptor} as JobDescriptor>::Payload, <{descriptor} as JobDescriptor>::Result>, JobsError> {{ self.handle.generated_submit_job::<{descriptor}>(payload).await }}"),
            "}".to_string(),
            String::new(),
        ]);
    }
    format!("{}\n", lines.join("\n"))
}

fn render_participant_connect_rs(
    loaded: &LoadedParticipant,
    mappings: &[ValidatedParticipantAlias],
) -> String {
    match loaded.render_model.kind {
        ContractKind::Service => render_service_participant_connect_rs(mappings),
        ContractKind::App | ContractKind::Agent => render_user_participant_connect_rs(mappings),
        ContractKind::Device => render_device_participant_connect_rs(mappings),
    }
}

fn render_service_participant_connect_rs(mappings: &[ValidatedParticipantAlias]) -> String {
    let mut source = r#"//! Service connection entry point for this participant.

use crate::Service;

/// Runtime and credential options; contract evidence comes from [`Contract`].
pub use trellis_rs::service::ServiceConnectOptions;

/// Exact generated participant and API evidence used for service bootstrap.
pub struct Contract;

impl trellis_rs::service::GeneratedServiceContract for Contract {
    const PARTICIPANT_ID: &'static str = crate::contract::CONTRACT_ID;
    const CONTRACT_DIGEST: &'static str = crate::contract::CONTRACT_DIGEST;
    const PARTICIPANT_NEEDS_DIGEST: &'static str = crate::contract::PARTICIPANT_NEEDS_DIGEST;
    const PARTICIPANT_JSON: &'static str = crate::contract::PARTICIPANT;
    const API_JSON: &'static str = crate::contract::API_JSON;
    const API_DIGEST: &'static str = crate::contract::API_DIGEST;
    const REFERENCED_API_ARTIFACTS: &'static [(&'static str, &'static str)] = crate::contract::REFERENCED_API_ARTIFACTS;
}

/// Connected service runtime for this participant contract.
pub struct ConnectedService {
    pub(crate) inner: trellis_rs::service::ConnectedServiceRuntime<Contract>,
}

impl ConnectedService {
    /// Connect this service through Trellis bootstrap.
    pub async fn connect(opts: ServiceConnectOptions<'_>) -> Result<Self, trellis_rs::service::ServiceRuntimeError> {
        Ok(Self { inner: trellis_rs::service::ConnectedServiceRuntime::<Contract>::connect(opts).await? })
    }

    /// Access contract-shaped owned and used surfaces.
    pub fn service(&self) -> Service<'_> { Service::new(self.inner.caller()) }

    /// Clone the service handle for providers that need resolved runtime resources.
    pub fn generated_handle(&self) -> trellis_rs::service::ServiceHandle { self.inner.generated_handle() }

    pub(crate) fn runtime_mut(&mut self) -> &mut trellis_rs::service::ConnectedServiceRuntime<Contract> { &mut self.inner }

    /// Run all registered providers and service-private workers until shutdown.
    pub async fn run(self) -> Result<(), trellis_rs::service::ServiceRuntimeError> { self.inner.run().await }
}

/// Connect this service through Trellis bootstrap.
pub async fn connect(opts: ServiceConnectOptions<'_>) -> Result<ConnectedService, trellis_rs::service::ServiceRuntimeError> {
    ConnectedService::connect(opts).await
}
"#
    .to_string();
    source.push_str("\nimpl ConnectedService {\n");
    source.push_str(&render_connected_alias_methods(mappings, true));
    source.push_str("}\n");
    source
}

fn render_user_participant_connect_rs(mappings: &[ValidatedParticipantAlias]) -> String {
    let mut source = r#"//! User-authenticated connection entry point for this participant.

use std::sync::Arc;

use trellis_rs::generated::{AuthorizationContextBundle, AuthorizationContextStore, Caller, TrellisClientError, UserAuthorizationContext, UserConnectOptions, UserSessionCredentials};

use crate::Client;

/// User-authenticated participant connection options.
pub struct ConnectOptions<'a> {
    trellis_url: &'a str,
    servers: &'a str,
    bootstrap_jwt: &'a str,
    session_id: &'a str,
    inbox_prefix: &'a str,
    session_key_seed_base64url: &'a str,
    authorization_context: AuthorizationContextBundle,
    timeout_ms: u64,
    authorization_context_store: Arc<dyn AuthorizationContextStore>,
}

impl<'a> ConnectOptions<'a> {
    /// Create user-authenticated connection options.
    pub fn new(trellis_url: &'a str, servers: &'a str, bootstrap_jwt: &'a str, session_id: &'a str, inbox_prefix: &'a str, session_key_seed_base64url: &'a str, authorization_context: AuthorizationContextBundle, timeout_ms: u64, authorization_context_store: Arc<dyn AuthorizationContextStore>) -> Self {
        Self { trellis_url, servers, bootstrap_jwt, session_id, inbox_prefix, session_key_seed_base64url, authorization_context, timeout_ms, authorization_context_store }
    }
}

/// Connected caller facade for this participant contract.
pub struct ConnectedClient { inner: Caller }

impl ConnectedClient {
    /// Access only the contract surfaces declared by this participant.
    pub fn client(&self) -> Client<'_> { Client::new(&self.inner) }
}

/// Connect this participant with a user-authenticated session.
pub async fn connect(opts: ConnectOptions<'_>) -> Result<ConnectedClient, TrellisClientError> {
    Ok(ConnectedClient {
        inner: Caller::connect_user(UserConnectOptions::new(
            opts.trellis_url,
            opts.servers,
            opts.inbox_prefix,
            opts.timeout_ms,
            UserSessionCredentials {
                bootstrap_jwt: opts.bootstrap_jwt,
                session_id: opts.session_id,
                session_key_seed_base64url: opts.session_key_seed_base64url,
                participant_digest: crate::contract::CONTRACT_DIGEST,
            },
            UserAuthorizationContext {
                bundle: opts.authorization_context,
                binding: format!("installation:{}", opts.trellis_url),
                store: opts.authorization_context_store,
            },
        )).await?,
    })
}
"#
    .to_string();
    source.push_str("\nimpl ConnectedClient {\n");
    source.push_str(&render_connected_alias_methods(mappings, false));
    source.push_str("}\n");
    source
}

fn render_device_participant_connect_rs(mappings: &[ValidatedParticipantAlias]) -> String {
    let mut source = r#"//! Activated-device connection entry point for this participant.

use std::sync::Arc;

use trellis_rs::generated::{AuthorizationContextStore, Caller, DeviceConnectOptions, TrellisClientError};

use crate::Client;

/// Exact generated participant and API evidence used for device bootstrap.
pub struct Contract;

impl trellis_rs::service::GeneratedServiceContract for Contract {
    const PARTICIPANT_ID: &'static str = crate::contract::CONTRACT_ID;
    const CONTRACT_DIGEST: &'static str = crate::contract::CONTRACT_DIGEST;
    const PARTICIPANT_NEEDS_DIGEST: &'static str = crate::contract::PARTICIPANT_NEEDS_DIGEST;
    const PARTICIPANT_JSON: &'static str = crate::contract::PARTICIPANT;
    const API_JSON: &'static str = crate::contract::API_JSON;
    const API_DIGEST: &'static str = crate::contract::API_DIGEST;
    const REFERENCED_API_ARTIFACTS: &'static [(&'static str, &'static str)] = crate::contract::REFERENCED_API_ARTIFACTS;
}

/// Activated-device participant connection options.
pub struct ConnectOptions<'a> {
    trellis_url: &'a str,
    deployment_id: &'a str,
    instance_id: &'a str,
    public_identity_key: &'a str,
    identity_seed_base64url: &'a str,
    timeout_ms: u64,
    authorization_context_store: Arc<dyn AuthorizationContextStore>,
}

impl<'a> ConnectOptions<'a> {
    /// Create activated-device connection options.
    pub fn new(trellis_url: &'a str, deployment_id: &'a str, instance_id: &'a str, public_identity_key: &'a str, identity_seed_base64url: &'a str, timeout_ms: u64, authorization_context_store: Arc<dyn AuthorizationContextStore>) -> Self {
        Self { trellis_url, deployment_id, instance_id, public_identity_key, identity_seed_base64url, timeout_ms, authorization_context_store }
    }
}

/// Connected caller facade for this participant contract.
pub struct ConnectedClient { inner: Caller }

impl ConnectedClient {
    /// Access only the contract surfaces declared by this participant.
    pub fn client(&self) -> Client<'_> { Client::new(&self.inner) }

    /// Connect using the exact ready bootstrap evidence returned by device activation.
    pub async fn connect_activated(
        options: trellis_rs::auth::DeviceActivationOptions<'_, Contract>,
        session: trellis_rs::auth::DeviceActivationSession,
    ) -> Result<Self, trellis_rs::auth::DeviceActivationError> {
        Ok(Self {
            inner: Caller::connect_device(options.into_connect_options(session)?).await?,
        })
    }
}

/// Connect this activated device.
pub async fn connect(opts: ConnectOptions<'_>) -> Result<ConnectedClient, TrellisClientError> {
    Ok(ConnectedClient {
        inner: Caller::connect_device(DeviceConnectOptions::<Contract>::new(
            opts.trellis_url,
            opts.deployment_id,
            opts.instance_id,
            opts.public_identity_key,
            opts.identity_seed_base64url,
            opts.authorization_context_store,
        ).with_timeout_ms(opts.timeout_ms)).await?,
    })
}
"#
    .to_string();
    source.push_str("\nimpl ConnectedClient {\n");
    source.push_str(&render_connected_alias_methods(mappings, false));
    source.push_str("}\n");
    source
}

fn render_connected_alias_methods(mappings: &[ValidatedParticipantAlias], service: bool) -> String {
    let receiver = if service {
        "crate::Service::new(self.inner.caller())"
    } else {
        "crate::Client::new(&self.inner)"
    };
    let mut methods = String::new();
    for mapping in mappings {
        let alias = &mapping.alias;
        let alias_ident = &mapping.alias_ident;
        methods.push_str(&format!(
            "    /// Access the `{alias}` dependency surface.\n    pub fn {alias_ident}(&self) -> crate::uses::{alias_ident}::Client<'_> {{ {receiver}.{alias_ident}() }}\n"
        ));
    }
    methods
}

fn render_participant_contract_rs(
    loaded: &LoadedParticipant,
    artifacts: &trellis_contracts::ContractArtifacts,
) -> Result<String, CodegenRustError> {
    let participant_json = artifacts.participant_value()?;
    let api_json = artifacts.api_value()?;
    let mut referenced = Vec::new();
    for api in artifacts.referenced_apis().values() {
        let value = api
            .normalized_value()
            .map_err(trellis_contracts::ContractsError::from)?;
        let json =
            serde_json::to_string(&value).map_err(trellis_contracts::ContractsError::from)?;
        let digest = api
            .digest()
            .map_err(trellis_contracts::ContractsError::from)?;
        referenced.push(format!(
            "    ({}, {}),",
            string_literal(&json),
            string_literal(&digest),
        ));
    }
    Ok(format!(
        "//! Native participant metadata for `{}`.\n\n/// Participant contract id.\npub const CONTRACT_ID: &str = {};\n\n/// Semantic participant artifact digest.\npub const CONTRACT_DIGEST: &str = {};\n\n/// Canonical participant artifact JSON.\npub const PARTICIPANT: &str = {};\n\n/// Authoritative participant-needs digest.\npub const PARTICIPANT_NEEDS_DIGEST: &str = {};\n\n/// Canonical owned API artifact JSON.\npub const API_JSON: &str = {};\n\n/// Semantic owned API digest.\npub const API_DIGEST: &str = {};\n\n/// Exact referenced API JSON and digest evidence.\npub const REFERENCED_API_ARTIFACTS: &[(&str, &str)] = &[\n{}];\n",
        loaded.render_model.id,
        string_literal(&loaded.render_model.id),
         string_literal(&artifacts.participant_digest()?),
         string_literal(
             &serde_json::to_string(&participant_json)
                 .map_err(trellis_contracts::ContractsError::from)?,
         ),
         string_literal(&artifacts.participant_needs_digest()?),
        string_literal(
            &serde_json::to_string(&api_json)
                .map_err(trellis_contracts::ContractsError::from)?,
        ),
        string_literal(&artifacts.api_digest()?),
        referenced.join("\n"),
    ))
}

fn render_participant_facade_rs(
    _loaded: &LoadedParticipant,
    mappings: &[ValidatedParticipantAlias],
) -> String {
    let mut lines = vec![
        "pub mod owned;".to_string(),
        String::new(),
        "pub mod schemas;".to_string(),
        String::new(),
        "pub mod state;".to_string(),
        String::new(),
        "pub mod uses;".to_string(),
    ];
    lines.extend([
        String::new(),
        "/// Contract-shaped outbound facade for this participant.".to_string(),
        "pub struct Client<'a> {".to_string(),
        "    inner: &'a trellis_rs::generated::Caller,".to_string(),
        "}".to_string(),
        String::new(),
        "/// Service-side facade for owned handlers plus outbound alias access.".to_string(),
        "pub struct Service<'a> {".to_string(),
        "    inner: &'a trellis_rs::generated::Caller,".to_string(),
        "}".to_string(),
        String::new(),
        "impl<'a> Client<'a> {".to_string(),
        "    /// Wrap an already connected low-level Trellis client.".to_string(),
        "    pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self { Self { inner } }"
            .to_string(),
        "    /// Access the participant's owned contract surface.".to_string(),
        "    pub fn owned(&self) -> owned::Client<'a> { owned::Client::new(self.inner) }"
            .to_string(),
        "    /// Access typed state stores declared by this participant.".to_string(),
        "    pub fn state(&self) -> state::State<'a> { state::State::new(self.inner) }".to_string(),
    ]);
    for mapping in mappings {
        lines.push(format!(
            "    /// Access the `{}` dependency alias facade.",
            mapping.alias
        ));
        lines.push(format!(
            "    pub fn {}(&self) -> uses::{}::Client<'a> {{ uses::{}::Client::new(self.inner) }}",
            mapping.alias_ident, mapping.alias_ident, mapping.alias_ident
        ));
    }
    lines.push("}".to_string());
    lines.push(String::new());
    lines.push("impl<'a> Service<'a> {".to_string());
    lines.push(
        "    /// Wrap an already connected low-level Trellis client for outbound service calls."
            .to_string(),
    );
    lines.push(
        "    pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self { Self { inner } }"
            .to_string(),
    );
    lines.push("    /// Access owned handler and publish helpers.".to_string());
    lines.push(
        "    pub fn owned(&self) -> owned::Service<'a> { owned::Service::new(self.inner) }"
            .to_string(),
    );
    for mapping in mappings {
        lines.push(format!(
            "    /// Access the `{}` dependency alias facade for outbound calls.",
            mapping.alias
        ));
        lines.push(format!(
            "    pub fn {}(&self) -> uses::{}::Client<'a> {{ uses::{}::Client::new(self.inner) }}",
            mapping.alias_ident, mapping.alias_ident, mapping.alias_ident
        ));
    }
    lines.push("}".to_string());
    lines.push(String::new());
    format!("{}\n", lines.join("\n"))
}

fn render_participant_owned_rs(
    loaded: &trellis_contracts::LoadedApi,
    participant: &LoadedParticipant,
    owned_sdk_crate_name: Option<&str>,
    referenced_apis: std::collections::BTreeMap<String, serde_json::Value>,
) -> Result<String, CodegenRustError> {
    if public_rpc_keys(loaded).is_empty()
        && loaded.render_model.operations.is_empty()
        && loaded.render_model.events.is_empty()
        && loaded.render_model.feeds.is_empty()
        && participant.render_model.jobs.is_empty()
        && participant.render_model.event_consumers.is_empty()
        && participant.render_model.resources.kv.is_empty()
        && participant.render_model.resources.store.is_empty()
    {
        let artifacts =
            ContractBuilder::from_native(loaded.value.clone(), participant.value.clone())
                .referenced_apis(referenced_apis)
                .build()?;
        return Ok(format!(
            "/// Owned facade for `{}`.\n/// Reusable native participant vocabulary for this participant.\npub struct OwnedContract;\n\nimpl OwnedContract {{\n    pub const PARTICIPANT_ID: &'static str = {};\n    pub const CONTRACT_DIGEST: &'static str = {};\n    pub const PARTICIPANT_NEEDS_DIGEST: &'static str = {};\n    pub const PARTICIPANT_JSON: &'static str = {};\n    pub const API_JSON: &'static str = {};\n    pub const API_DIGEST: &'static str = {};\n}}\n\npub struct Client<'a> {{ _inner: &'a trellis_rs::generated::Caller }}\nimpl<'a> Client<'a> {{ pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self {{ Self {{ _inner: inner }} }} }}\n\npub struct Service<'a> {{ _inner: &'a trellis_rs::generated::Caller }}\nimpl<'a> Service<'a> {{ pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self {{ Self {{ _inner: inner }} }} }}\n",
            participant.render_model.id,
            string_literal(artifacts.participant().id()),
            string_literal(&artifacts.participant_digest()?),
            string_literal(&artifacts.participant_needs_digest()?),
            string_literal(&canonicalize_json(&artifacts.participant_value()?)?),
            string_literal(&canonicalize_json(&artifacts.api_value()?)?),
            string_literal(&artifacts.api_digest()?),
        )
        .replace("PARTICIPANT_ID", "CONTRACT_ID")
        .replace("PARTICIPANT_JSON", "PARTICIPANT"));
    }

    let owned_sdk_crate_name = owned_sdk_crate_name.expect("owned sdk crate required");
    let owned_crate_ident = crate_ident(owned_sdk_crate_name);
    let owned_client_name = format!("{}Client", sdk_stem_pascal(loaded));
    let mut lines = vec![
        format!("/// Owned facade for `{}`.", participant.render_model.id),
        String::new(),
        format!("use {} as sdk;", owned_crate_ident),
        String::new(),
        "/// Reusable owned contract vocabulary for this participant.".to_string(),
        "pub struct OwnedContract;".to_string(),
        String::new(),
        "impl OwnedContract {".to_string(),
        "    pub const CONTRACT_ID: &'static str = crate::contract::CONTRACT_ID;".to_string(),
        "    pub const CONTRACT_DIGEST: &'static str = crate::contract::CONTRACT_DIGEST;".to_string(),
        "    pub const PARTICIPANT: &'static str = crate::contract::PARTICIPANT;".to_string(),
        "    pub const PARTICIPANT_NEEDS_DIGEST: &'static str = crate::contract::PARTICIPANT_NEEDS_DIGEST;".to_string(),
        "    pub const API_JSON: &'static str = sdk::API_JSON;".to_string(),
        "    pub const API_DIGEST: &'static str = sdk::API_DIGEST;".to_string(),
        "}".to_string(),
        String::new(),
        "pub struct Client<'a> { inner: sdk::".to_string() + &owned_client_name + "<'a> }",
        "impl<'a> Client<'a> {".to_string(),
        "    pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self { Self { inner: sdk::"
            .to_string()
            + &owned_client_name
            + "::new(inner) } }",
    ];
    for key in public_rpc_keys(loaded) {
        let method = key_to_snake(key);
        let base = key_to_pascal(key);
        let input_empty = is_empty_object_schema(resolve_schema_ref(
            loaded,
            &loaded.render_model.rpc[key].input.schema,
        ));
        let output_type = if is_empty_object_schema(resolve_schema_ref(
            loaded,
            &loaded.render_model.rpc[key].output.schema,
        )) {
            "sdk::rpc::Empty".to_string()
        } else {
            format!("sdk::{base}Response")
        };
        let error_type = rpc_call_error_type(loaded, key, "sdk::rpc");
        if input_empty {
            let (group, surface_method) = surface_group_and_method(key);
            lines.push(format!("    pub async fn {method}(&self) -> Result<{output_type}, trellis_rs::generated::CallError<{error_type}>> {{ self.inner.rpc().{group}().{surface_method}().await }}"));
        } else {
            let (group, surface_method) = surface_group_and_method(key);
            lines.push(format!("    pub async fn {method}(&self, input: &sdk::{base}Request) -> Result<{output_type}, trellis_rs::generated::CallError<{error_type}>> {{ self.inner.rpc().{group}().{surface_method}(input).await }}"));
        }
    }
    for key in loaded.render_model.events.keys() {
        let method = format!("publish_{}", key_to_snake(key));
        let base = key_to_pascal(key);
        let (group, surface_method) = surface_group_and_method(key);
        lines.push(format!("    pub async fn {method}(&self, event: &sdk::{base}Event) -> Result<(), trellis_rs::generated::TrellisClientError> {{ self.inner.event().{group}().{surface_method}().publish(event).await }}"));
    }
    for (key, feed) in &loaded.render_model.feeds {
        let method = key_to_snake(key);
        let base = key_to_pascal(key);
        if is_empty_object_schema(resolve_schema_ref(loaded, &feed.input.schema)) {
            let (group, surface_method) = surface_group_and_method(key);
            lines.push(format!("    pub async fn {method}(&self) -> Result<futures_util::stream::BoxStream<'static, Result<sdk::{base}Event, trellis_rs::generated::TrellisClientError>>, trellis_rs::generated::TrellisClientError> {{ self.inner.feed().{group}().{surface_method}().await }}"));
        } else {
            let (group, surface_method) = surface_group_and_method(key);
            lines.push(format!("    pub async fn {method}(&self, input: &sdk::{base}Input) -> Result<futures_util::stream::BoxStream<'static, Result<sdk::{base}Event, trellis_rs::generated::TrellisClientError>>, trellis_rs::generated::TrellisClientError> {{ self.inner.feed().{group}().{surface_method}(input).await }}"));
        }
    }
    lines.push("}".to_string());
    lines.push(String::new());
    lines.push("pub struct Service<'a> { inner: &'a trellis_rs::generated::Caller }".to_string());
    lines.push("impl<'a> Service<'a> {".to_string());
    lines.push(
        "    pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self { Self { inner } }"
            .to_string(),
    );
    lines.push("    pub fn client(&self) -> Client<'a> { Client::new(self.inner) }".to_string());
    for key in loaded.render_model.events.keys() {
        let method = format!("publish_{}", key_to_snake(key));
        let base = key_to_pascal(key);
        lines.push(format!("    pub async fn {method}(&self, publisher: &trellis_rs::service::EventPublisher, event: &sdk::{base}Event) -> Result<(), trellis_rs::service::ServerError> {{ publisher.publish::<sdk::events::{base}EventDescriptor>(event).await }}"));
    }
    lines.push("}".to_string());
    lines.push(String::new());
    if participant.render_model.kind == ContractKind::Service
        && !loaded.render_model.events.is_empty()
    {
        lines.push("/// Cloneable typed publisher for events owned by this service.".to_string());
        lines.push("#[derive(Clone)]".to_string());
        lines.push(
            "pub struct Publisher { inner: trellis_rs::service::EventPublisher }".to_string(),
        );
        lines.push("impl crate::ConnectedService {".to_string());
        lines.push("    /// Clone a typed publisher for owned events.\n    pub fn publisher(&self) -> Publisher { Publisher { inner: self.inner.event_publisher() } }".to_string());
        lines.push("}".to_string());
        lines.push("impl Publisher {".to_string());
        for key in loaded.render_model.events.keys() {
            let method = format!("publish_{}", key_to_snake(key));
            let base = key_to_pascal(key);
            lines.push(format!("    /// Publish `{key}`.\n    pub async fn {method}(&self, event: &sdk::{base}Event) -> Result<(), trellis_rs::service::ServerError> {{ self.inner.publish::<sdk::events::{base}EventDescriptor>(event).await }}"));
        }
        lines.push("}".to_string());
        lines.push(String::new());
    }
    if participant.render_model.kind == ContractKind::Service
        && (!public_rpc_keys(loaded).is_empty()
            || !loaded.render_model.operations.is_empty()
            || !loaded.render_model.events.is_empty()
            || !loaded.render_model.feeds.is_empty())
    {
        render_participant_owned_provider_surface(loaded, &mut lines);
        lines.push("impl crate::ConnectedService {".to_string());
        for key in public_rpc_keys(loaded) {
            let method = format!("register_{}", key_to_snake(key));
            let base = key_to_pascal(key);
            let input_type = if is_empty_object_schema(resolve_schema_ref(
                loaded,
                &loaded.render_model.rpc[key].input.schema,
            )) {
                "sdk::rpc::Empty".to_string()
            } else {
                format!("sdk::{base}Request")
            };
            let output_type = if is_empty_object_schema(resolve_schema_ref(
                loaded,
                &loaded.render_model.rpc[key].output.schema,
            )) {
                "sdk::rpc::Empty".to_string()
            } else {
                format!("sdk::{base}Response")
            };
            lines.push(format!("    fn {method}<F, Fut>(&mut self, handler: F) where F: Fn(trellis_rs::service::ServiceHandlerContext, {input_type}) -> Fut + Send + Sync + 'static, Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<{output_type}>> + Send + 'static {{ self.runtime_mut().register_rpc::<sdk::rpc::{base}Rpc, _, _>(handler); }}"));
        }
        for key in loaded.render_model.operations.keys() {
            let method = format!("register_{}_provider", key_to_snake(key));
            let base = key_to_pascal(key);
            lines.push(format!("    fn {method}<P>(&mut self, provider: P) where P: trellis_rs::service::ServiceOperationProvider<sdk::operations::{base}Operation> {{ self.runtime_mut().register_operation_provider::<sdk::operations::{base}Operation, _>(provider); }}"));
        }
        for key in loaded.render_model.events.keys() {
            let method = format!("publish_{}", key_to_snake(key));
            let base = key_to_pascal(key);
            lines.push(format!("    pub async fn {method}(&self, event: &sdk::{base}Event) -> Result<(), trellis_rs::service::ServerError> {{ self.inner.event_publisher().publish::<sdk::events::{base}EventDescriptor>(event).await }}"));
        }
        for (key, feed) in &loaded.render_model.feeds {
            let method = format!("register_{}", key_to_snake(key));
            let base = key_to_pascal(key);
            let input_type =
                if is_empty_object_schema(resolve_schema_ref(loaded, &feed.input.schema)) {
                    "sdk::rpc::Empty".to_string()
                } else {
                    format!("sdk::{base}Input")
                };
            lines.push(format!("    fn {method}<F, S>(&mut self, handler: F) where F: Fn(trellis_rs::service::ServiceHandlerContext, {input_type}) -> S + Send + Sync + 'static, S: futures_util::Stream<Item = Result<sdk::{base}Event, trellis_rs::service::ServerError>> + Send + 'static {{ self.runtime_mut().register_feed::<sdk::feeds::{base}FeedDescriptor, _, _>(handler); }}"));
        }
        lines.push("}".to_string());
        lines.push(String::new());
    }
    if participant.render_model.kind == ContractKind::Service {
        render_participant_resource_surfaces(participant, &mut lines);
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn render_participant_resource_surfaces(loaded: &LoadedParticipant, lines: &mut Vec<String>) {
    if !loaded.render_model.resources.kv.is_empty() {
        lines.extend([
            "/// Contract-declared key-value resources.".to_string(),
            "pub struct Kv<'a> { service: &'a crate::ConnectedService }".to_string(),
            "impl crate::ConnectedService {".to_string(),
            "    /// Access contract-declared key-value resources.".to_string(),
            "    pub fn kv(&self) -> Kv<'_> { Kv { service: self } }".to_string(),
            "}".to_string(),
            "impl<'a> Kv<'a> {".to_string(),
        ]);
        for name in loaded.render_model.resources.kv.keys() {
            let method = rust_ident(&key_to_snake(name));
            lines.push(format!("    /// Open the `{name}` key-value resource."));
            lines.push(format!("    pub async fn {method}(&self) -> Result<trellis_rs::service::KvHandle, trellis_rs::service::ServerError> {{ self.service.inner.kv_client({}).await }}", string_literal(name)));
        }
        lines.extend(["}".to_string(), String::new()]);
    }

    if !loaded.render_model.resources.store.is_empty() {
        lines.extend([
            "/// Contract-declared object-store resources.".to_string(),
            "pub struct Store<'a> { service: &'a crate::ConnectedService }".to_string(),
            "impl crate::ConnectedService {".to_string(),
            "    /// Access contract-declared object-store resources.".to_string(),
            "    pub fn store(&self) -> Store<'_> { Store { service: self } }".to_string(),
            "}".to_string(),
            "impl<'a> Store<'a> {".to_string(),
        ]);
        for name in loaded.render_model.resources.store.keys() {
            let method = rust_ident(&key_to_snake(name));
            lines.push(format!("    /// Open the `{name}` object-store resource."));
            lines.push(format!("    pub async fn {method}(&self) -> Result<trellis_rs::service::StoreHandle, trellis_rs::service::ServerError> {{ self.service.inner.store_client({}).await }}", string_literal(name)));
        }
        lines.extend(["}".to_string(), String::new()]);
    }
}

fn render_participant_owned_provider_surface(
    loaded: &trellis_contracts::LoadedApi,
    lines: &mut Vec<String>,
) {
    lines.extend([
        "impl crate::ConnectedService {".to_string(),
        "    pub fn handle(&mut self) -> ServiceHandle<'_> { ServiceHandle { service: self } }"
            .to_string(),
        "}".to_string(),
        String::new(),
        "pub struct ServiceHandle<'a> { service: &'a mut crate::ConnectedService }".to_string(),
        "impl<'a> ServiceHandle<'a> {".to_string(),
    ]);
    if !grouped_public_rpc_keys(loaded).is_empty() {
        lines.push("    pub fn rpc(&mut self) -> ProviderRpc<'_> { ProviderRpc { service: self.service } }".to_string());
    }
    if !loaded.render_model.feeds.is_empty() {
        lines.push("    pub fn feed(&mut self) -> ProviderFeed<'_> { ProviderFeed { service: self.service } }".to_string());
    }
    if !loaded.render_model.operations.is_empty() {
        lines.push("    pub fn operation(&mut self) -> ProviderOperation<'_> { ProviderOperation { service: self.service } }".to_string());
    }
    lines.extend(["}".to_string(), String::new()]);

    if !grouped_public_rpc_keys(loaded).is_empty() {
        lines.extend([
            "pub struct ProviderRpc<'a> { service: &'a mut crate::ConnectedService }".to_string(),
            "impl<'a> ProviderRpc<'a> {".to_string(),
        ]);
        for group in grouped_public_rpc_keys(loaded).keys() {
            let group_ty = format!("{}ProviderRpc", key_to_pascal(group));
            lines.push(format!("    pub fn {group}(&mut self) -> {group_ty}<'_> {{ {group_ty} {{ service: self.service }} }}"));
        }
        lines.extend(["}".to_string(), String::new()]);
        for (group, keys) in grouped_public_rpc_keys(loaded) {
            let group_ty = format!("{}ProviderRpc", key_to_pascal(&group));
            lines.push(format!(
                "pub struct {group_ty}<'a> {{ service: &'a mut crate::ConnectedService }}"
            ));
            lines.push(format!("impl<'a> {group_ty}<'a> {{"));
            for key in keys {
                let (_, method) = surface_group_and_method(key);
                let register = format!("register_{}", key_to_snake(key));
                let base = key_to_pascal(key);
                let rpc = &loaded.render_model.rpc[key];
                let input_type =
                    if is_empty_object_schema(resolve_schema_ref(loaded, &rpc.input.schema)) {
                        "sdk::rpc::Empty".to_string()
                    } else {
                        format!("sdk::{base}Request")
                    };
                let output_type =
                    if is_empty_object_schema(resolve_schema_ref(loaded, &rpc.output.schema)) {
                        "sdk::rpc::Empty".to_string()
                    } else {
                        format!("sdk::{base}Response")
                    };
                lines.push(format!("    pub fn {method}<F, Fut>(&mut self, handler: F) where F: Fn(trellis_rs::service::ServiceHandlerContext, {input_type}) -> Fut + Send + Sync + 'static, Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<{output_type}>> + Send + 'static {{ self.service.{register}(handler); }}"));
            }
            lines.extend(["}".to_string(), String::new()]);
        }
    }

    if !loaded.render_model.feeds.is_empty() {
        lines.extend([
            "pub struct ProviderFeed<'a> { service: &'a mut crate::ConnectedService }".to_string(),
            "impl<'a> ProviderFeed<'a> {".to_string(),
        ]);
        for group in grouped_keys(&loaded.render_model.feeds).keys() {
            let group_ty = format!("{}ProviderFeed", key_to_pascal(group));
            lines.push(format!("    pub fn {group}(&mut self) -> {group_ty}<'_> {{ {group_ty} {{ service: self.service }} }}"));
        }
        lines.extend(["}".to_string(), String::new()]);
        for (group, keys) in grouped_keys(&loaded.render_model.feeds) {
            let group_ty = format!("{}ProviderFeed", key_to_pascal(&group));
            lines.push(format!(
                "pub struct {group_ty}<'a> {{ service: &'a mut crate::ConnectedService }}"
            ));
            lines.push(format!("impl<'a> {group_ty}<'a> {{"));
            for key in keys {
                let (_, method) = surface_group_and_method(key);
                let register = format!("register_{}", key_to_snake(key));
                let base = key_to_pascal(key);
                let feed = &loaded.render_model.feeds[key];
                let input_type =
                    if is_empty_object_schema(resolve_schema_ref(loaded, &feed.input.schema)) {
                        "sdk::rpc::Empty".to_string()
                    } else {
                        format!("sdk::{base}Input")
                    };
                lines.push(format!("    pub fn {method}<F, S>(&mut self, handler: F) where F: Fn(trellis_rs::service::ServiceHandlerContext, {input_type}) -> S + Send + Sync + 'static, S: futures_util::Stream<Item = Result<sdk::{base}Event, trellis_rs::service::ServerError>> + Send + 'static {{ self.service.{register}(handler); }}"));
            }
            lines.extend(["}".to_string(), String::new()]);
        }
    }

    if !loaded.render_model.operations.is_empty() {
        lines.extend([
            "pub struct ProviderOperation<'a> { service: &'a mut crate::ConnectedService }"
                .to_string(),
            "impl<'a> ProviderOperation<'a> {".to_string(),
        ]);
        for group in grouped_keys(&loaded.render_model.operations).keys() {
            let group_ty = format!("{}ProviderOperation", key_to_pascal(group));
            lines.push(format!("    pub fn {group}(&mut self) -> {group_ty}<'_> {{ {group_ty} {{ service: self.service }} }}"));
        }
        lines.extend(["}".to_string(), String::new()]);
        for (group, keys) in grouped_keys(&loaded.render_model.operations) {
            let group_ty = format!("{}ProviderOperation", key_to_pascal(&group));
            lines.push(format!(
                "pub struct {group_ty}<'a> {{ service: &'a mut crate::ConnectedService }}"
            ));
            lines.push(format!("impl<'a> {group_ty}<'a> {{"));
            for key in keys {
                let (_, method) = surface_group_and_method(key);
                let register = format!("register_{}_provider", key_to_snake(key));
                let base = key_to_pascal(key);
                lines.push(format!("    pub fn {method}<P>(&mut self, provider: P) where P: trellis_rs::service::ServiceOperationProvider<sdk::operations::{base}Operation> {{ self.service.{register}(provider); }}"));
            }
            lines.extend(["}".to_string(), String::new()]);
        }
    }
}

fn render_participant_state_rs(loaded: &LoadedParticipant) -> String {
    let mut renderer = TypeRenderer::default();
    let mut stores = loaded.render_model.state.iter().collect::<Vec<_>>();
    stores.sort_by(|left, right| left.0.cmp(right.0));

    let schema_names = stores
        .iter()
        .map(|(_, store)| store.schema.schema.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mut used_type_names = std::collections::BTreeSet::new();
    let mut schema_type_names = std::collections::BTreeMap::new();
    for schema_name in schema_names {
        let base = state_type_name(&schema_name);
        let mut candidate = base.clone();
        let mut suffix = 2;
        while used_type_names.contains(&candidate) {
            candidate = format!("{base}{suffix}");
            suffix += 1;
        }
        used_type_names.insert(candidate.clone());
        schema_type_names.insert(schema_name, candidate);
    }

    for (_, store) in &stores {
        let type_name = schema_type_names
            .get(&store.schema.schema)
            .expect("state schema type name");
        renderer.render_named_type(
            type_name,
            loaded
                .render_model
                .schemas
                .get(&store.schema.schema)
                .expect("validated participant state schema"),
        );
    }

    let rendered = renderer.finish();
    let mut lines = vec![format!(
        "// Typed state store helpers for `{}`.",
        loaded.render_model.id
    )];

    if !rendered.is_empty() {
        lines.push(String::new());
        lines.push("use serde::{Deserialize, Serialize};".to_string());
        if rendered.iter().any(|line| line.contains("Value")) {
            lines.push("use serde_json::Value;".to_string());
        }
        if rendered.iter().any(|line| line.contains("BTreeMap<")) {
            lines.push("use std::collections::BTreeMap;".to_string());
        }
    }

    lines.push(String::new());
    lines.push("/// Typed access to state stores declared by this participant.".to_string());
    lines.push("pub struct State<'a> {".to_string());
    lines.push(if stores.is_empty() {
        "    _inner: &'a trellis_rs::generated::Caller,".to_string()
    } else {
        "    inner: &'a trellis_rs::generated::Caller,".to_string()
    });
    lines.push("}".to_string());
    lines.push(String::new());
    lines.push("impl<'a> State<'a> {".to_string());
    lines.push("    /// Wrap an already connected low-level Trellis client.".to_string());
    lines.push(if stores.is_empty() {
        "    pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self { Self { _inner: inner } }"
            .to_string()
    } else {
        "    pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self { Self { inner } }"
            .to_string()
    });

    for (name, store) in stores {
        let method_name = rust_ident(&key_to_snake(name));
        let ty = schema_type_names
            .get(&store.schema.schema)
            .expect("state schema type name");
        match &store.kind {
            trellis_contracts::ContractStateKind::Value => {
                lines.push(format!("    /// Access the `{name}` value state store."));
                lines.push(format!("    pub fn {method_name}(&self) -> trellis_rs::generated::ValueStateStore<'a, trellis_rs::generated::Caller, {ty}> {{"));
                lines.push(format!(
                    "        trellis_rs::generated::ValueStateStore::new(self.inner, {})",
                    string_literal(name)
                ));
                lines.push("    }".to_string());
            }
            trellis_contracts::ContractStateKind::Map => {
                lines.push(format!("    /// Access the `{name}` map state store."));
                lines.push(format!("    pub fn {method_name}(&self) -> trellis_rs::generated::MapStateStore<'a, trellis_rs::generated::Caller, {ty}> {{"));
                lines.push(format!(
                    "        trellis_rs::generated::MapStateStore::new(self.inner, {})",
                    string_literal(name)
                ));
                lines.push("    }".to_string());
            }
        }
    }

    lines.push("}".to_string());
    lines.push(String::new());
    lines.extend(rendered);
    format!("{}\n", lines.join("\n"))
}

fn state_type_name(schema_name: &str) -> String {
    let base = key_to_pascal(schema_name);
    if base == "State" {
        return "StateValue".to_string();
    }
    if base.ends_with("State") {
        base
    } else {
        format!("{base}State")
    }
}

fn render_participant_uses_mod_rs(mappings: &[ValidatedParticipantAlias]) -> String {
    let mut lines = vec![
        "//! Generated dependency alias facades.".to_string(),
        String::new(),
    ];
    for mapping in mappings {
        lines.push(format!("pub mod {};", mapping.alias_ident));
    }
    lines.push(String::new());
    format!("{}\n", lines.join("\n"))
}

fn render_participant_use_alias_rs(mapping: &ValidatedParticipantAlias) -> String {
    let remote_client_name = format!(
        "{}Client",
        sdk_stem_from_contract_id_pascal(&mapping.contract_id)
    );
    let has_download_transfer = mapping
        .use_ref
        .rpc
        .as_ref()
        .and_then(|rpc| rpc.call.as_ref())
        .is_some_and(|calls| {
            calls
                .iter()
                .any(|key| mapping.manifest.render_model.rpc[key].transfer.is_some())
        });
    let needs_transport = has_download_transfer
        || mapping
            .use_ref
            .operations
            .as_ref()
            .and_then(|operations| operations.invoke.as_ref())
            .is_some_and(|call| !call.is_empty())
        || mapping.use_ref.events.as_ref().is_some_and(|events| {
            events
                .publish
                .as_ref()
                .is_some_and(|publish| !publish.is_empty())
                || events
                    .subscribe
                    .as_ref()
                    .is_some_and(|subscribe| !subscribe.is_empty())
        })
        || mapping
            .use_ref
            .feeds
            .as_ref()
            .and_then(|feeds| feeds.subscribe.as_ref())
            .is_some_and(|subscribe| !subscribe.is_empty());
    let client_struct = if needs_transport {
        "pub struct Client<'a> { inner: sdk::".to_string()
            + &remote_client_name
            + "<'a>, transport: &'a trellis_rs::generated::Caller }"
    } else {
        "pub struct Client<'a> { inner: sdk::".to_string() + &remote_client_name + "<'a> }"
    };
    let client_new = if needs_transport {
        "    pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self { Self { inner: sdk::"
            .to_string()
            + &remote_client_name
            + "::new(inner), transport: inner } }"
    } else {
        "    pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self { Self { inner: sdk::"
            .to_string()
            + &remote_client_name
            + "::new(inner) } }"
    };
    let mut lines = vec![
        format!("/// Facade for the `{}` dependency alias.", mapping.alias),
        format!("use {} as sdk;", mapping.crate_ident),
        String::new(),
        client_struct,
        "impl<'a> Client<'a> {".to_string(),
        client_new,
        format!(
            "    pub const API_ID: &'static str = {};",
            string_literal(&mapping.contract_id)
        ),
    ];

    if let Some(rpc) = &mapping.use_ref.rpc {
        for key in rpc.call.as_deref().unwrap_or(&[]) {
            if mapping.manifest.render_model.rpc[key].internal == Some(true) {
                continue;
            }
            let method = key_to_snake(key);
            let base = key_to_pascal(key);
            let (group, leaf) = surface_group_and_method(key);
            let input_empty = is_empty_object_schema(resolve_schema_ref(
                &mapping.manifest,
                &mapping.manifest.render_model.rpc[key].input.schema,
            ));
            let output_type = if is_empty_object_schema(resolve_schema_ref(
                &mapping.manifest,
                &mapping.manifest.render_model.rpc[key].output.schema,
            )) {
                "sdk::Empty".to_string()
            } else {
                format!("sdk::{base}Response")
            };
            let error_type = rpc_call_error_type(&mapping.manifest, key, "sdk::rpc");
            if input_empty {
                lines.push(format!("    pub async fn {method}(&self) -> Result<{output_type}, trellis_rs::generated::CallError<{error_type}>> {{ self.inner.rpc().{group}().{leaf}().await }}"));
            } else {
                lines.push(format!("    pub async fn {method}(&self, input: &sdk::{base}Request) -> Result<{output_type}, trellis_rs::generated::CallError<{error_type}>> {{ self.inner.rpc().{group}().{leaf}(input).await }}"));
            }
        }
    }
    if has_download_transfer {
        lines.push(
            "    /// Download bytes from a transfer grant returned by this dependency.".to_string(),
        );
        lines.push("    pub async fn download_transfer(&self, grant: &trellis_rs::generated::DownloadTransferGrant) -> Result<Vec<u8>, trellis_rs::generated::TrellisClientError> { self.transport.download_transfer(grant).await }".to_string());
        lines.push(
            "    /// Stream bytes from a transfer grant into a caller-owned writer.".to_string(),
        );
        lines.push("    pub async fn download_transfer_into<W>(&self, grant: &trellis_rs::generated::DownloadTransferGrant, writer: &mut W) -> Result<trellis_rs::generated::FileInfo, trellis_rs::generated::TrellisClientError> where W: tokio::io::AsyncWrite + Unpin + Send + ?Sized { self.transport.download_transfer_into(grant, writer).await }".to_string());
        lines.push(
            "    /// Stream bytes into a writer with authenticated cancellation.".to_string(),
        );
        lines.push("    pub async fn download_transfer_into_with_cancel<W>(&self, grant: &trellis_rs::generated::DownloadTransferGrant, writer: &mut W, cancellation: &trellis_rs::generated::TransferCancellation) -> Result<trellis_rs::generated::FileInfo, trellis_rs::generated::TrellisClientError> where W: tokio::io::AsyncWrite + Unpin + Send + ?Sized { self.transport.download_transfer_into_with_cancel(grant, writer, cancellation).await }".to_string());
    }
    if let Some(operations) = &mapping.use_ref.operations {
        for key in operations.invoke.as_deref().unwrap_or(&[]) {
            let method = key_to_snake(key);
            let base = key_to_pascal(key);
            lines.push(format!("    pub fn {method}(&self) -> trellis_rs::generated::OperationInvoker<'a, trellis_rs::generated::Caller, sdk::operations::{base}Operation> {{ self.transport.operation::<sdk::operations::{base}Operation>() }}"));
        }
    }
    if let Some(events) = &mapping.use_ref.events {
        for key in events.publish.as_deref().unwrap_or(&[]) {
            let method = format!("publish_{}", key_to_snake(key));
            let base = key_to_pascal(key);
            lines.push(format!("    pub async fn {method}(&self, event: &sdk::{base}Event) -> Result<(), trellis_rs::generated::TrellisClientError> {{ self.transport.publish::<sdk::events::{base}EventDescriptor>(event).await }}"));
        }
        for key in events.subscribe.as_deref().unwrap_or(&[]) {
            let method = format!("subscribe_{}", key_to_snake(key));
            let base = key_to_pascal(key);
            lines.push(format!("    pub async fn {method}(&self) -> Result<futures_util::stream::BoxStream<'static, Result<sdk::{base}Event, trellis_rs::generated::TrellisClientError>>, trellis_rs::generated::TrellisClientError> {{ self.transport.subscribe::<sdk::events::{base}EventDescriptor>().await }}"));
        }
    }
    if let Some(feeds) = &mapping.use_ref.feeds {
        for key in feeds.subscribe.as_deref().unwrap_or(&[]) {
            let method = key_to_snake(key);
            let base = key_to_pascal(key);
            let input_empty = is_empty_object_schema(resolve_schema_ref(
                &mapping.manifest,
                &mapping.manifest.render_model.feeds[key].input.schema,
            ));
            if input_empty {
                lines.push(format!("    pub async fn {method}(&self) -> Result<futures_util::stream::BoxStream<'static, Result<sdk::{base}Event, trellis_rs::generated::TrellisClientError>>, trellis_rs::generated::TrellisClientError> {{ self.transport.feed::<sdk::feeds::{base}FeedDescriptor>(&sdk::rpc::Empty {{}}).await }}"));
            } else {
                lines.push(format!("    pub async fn {method}(&self, input: &sdk::{base}Input) -> Result<futures_util::stream::BoxStream<'static, Result<sdk::{base}Event, trellis_rs::generated::TrellisClientError>>, trellis_rs::generated::TrellisClientError> {{ self.transport.feed::<sdk::feeds::{base}FeedDescriptor>(input).await }}"));
            }
        }
    }
    lines.push("}".to_string());

    lines.push(String::new());
    format!("{}\n", lines.join("\n"))
}

fn render_api_rs(
    opts: &GenerateRustSdkOpts,
    loaded: &trellis_contracts::LoadedApi,
    artifact_canonical: &str,
    artifact_digest: &str,
) -> String {
    let api_name = manifest_display_name(loaded);
    let source_reference =
        api_source_reference(&opts.api_path, opts.runtime_deps.repo_root.as_deref());
    let rpc_metadata = loaded
        .render_model
        .rpc
        .keys()
        .map(|key| {
            format!(
                "    router.register_rpc_metadata::<super::rpc::{}Rpc>();",
                key_to_pascal(key)
            )
        })
        .chain(loaded.render_model.operations.keys().map(|key| {
            format!(
                "    router.register_operation_metadata::<super::operations::{}Operation>();",
                key_to_pascal(key)
            )
        }))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "//! API metadata for `{}`.\n//! Generated from {}\n\n/// Canonical Trellis API id.\npub const API_ID: &str = {};\n\n/// Stable digest for the canonical API JSON.\npub const API_DIGEST: &str = {};\n\n/// Human-readable API name.\npub const API_NAME: &str = {};\n\n/// Canonical API JSON embedded in the SDK crate.\npub const API_JSON: &str = {};\n\n/// Deserialize the embedded API artifact as JSON.\npub fn api_artifact() -> serde_json::Value {{\n    serde_json::from_str(API_JSON).expect(\"generated API JSON\")\n}}\n\n/// Register every generated RPC descriptor as router metadata.\npub fn register_rpc_metadata(router: &mut trellis_rs::service::Router) {{\n    router.set_api_id(API_ID);\n{}\n}}\n",
        loaded.render_model.id,
        source_reference,
        string_literal(&loaded.render_model.id),
         string_literal(artifact_digest),
        string_literal(&api_name),
         string_literal(artifact_canonical),
        rpc_metadata,
    )
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

fn render_types_rs(loaded: &trellis_contracts::LoadedApi) -> String {
    let mut renderer = TypeRenderer::default();
    let mut lines = vec![format!(
        "//! Shared request and response types for `{}`.",
        loaded.render_model.id
    )];

    for (key, rpc) in &loaded.render_model.rpc {
        if rpc.internal == Some(true) {
            continue;
        }
        let base = key_to_pascal(key);
        if !is_empty_object_schema(resolve_schema_ref(loaded, &rpc.input.schema)) {
            renderer.render_named_type(
                &format!("{base}Request"),
                resolve_schema_ref(loaded, &rpc.input.schema),
            );
        }
        if !is_empty_object_schema(resolve_schema_ref(loaded, &rpc.output.schema)) {
            renderer.render_named_type(
                &format!("{base}Response"),
                resolve_schema_ref(loaded, &rpc.output.schema),
            );
        }
    }

    for (key, operation) in &loaded.render_model.operations {
        let base = key_to_pascal(key);
        if !is_empty_object_schema(resolve_schema_ref(loaded, &operation.input.schema)) {
            renderer.render_named_type(
                &format!("{base}Input"),
                resolve_schema_ref(loaded, &operation.input.schema),
            );
        }
        if let Some(progress) = &operation.progress {
            if !is_empty_object_schema(resolve_schema_ref(loaded, &progress.schema)) {
                renderer.render_named_type(
                    &format!("{base}Progress"),
                    resolve_schema_ref(loaded, &progress.schema),
                );
            }
        }
        if let Some(update) = &operation.update {
            if !is_empty_object_schema(resolve_schema_ref(loaded, &update.schema)) {
                renderer.render_named_type(
                    &format!("{base}Update"),
                    resolve_schema_ref(loaded, &update.schema),
                );
            }
        }
        if let Some(output) = &operation.output {
            if !is_empty_object_schema(resolve_schema_ref(loaded, &output.schema)) {
                renderer.render_named_type(
                    &format!("{base}Output"),
                    resolve_schema_ref(loaded, &output.schema),
                );
            }
        }
    }

    for key in loaded.render_model.events.keys() {
        let base = key_to_pascal(key);
        renderer.render_named_type(
            &format!("{base}Event"),
            resolve_schema_ref(loaded, &loaded.render_model.events[key].event.schema),
        );
    }

    for (key, feed) in &loaded.render_model.feeds {
        let base = key_to_pascal(key);
        if !is_empty_object_schema(resolve_schema_ref(loaded, &feed.input.schema)) {
            renderer.render_named_type(
                &format!("{base}Input"),
                resolve_schema_ref(loaded, &feed.input.schema),
            );
        }
        renderer.render_named_type(
            &format!("{base}Event"),
            resolve_schema_ref(loaded, &feed.event.schema),
        );
    }

    for error in loaded.render_model.errors.values() {
        if let Some(schema) = &error.schema {
            renderer.render_named_type(
                &key_to_pascal(&schema.schema),
                resolve_schema_ref(loaded, &schema.schema),
            );
        }
    }

    for schema_name in &loaded.render_model.exports.schemas {
        renderer.render_named_type(
            key_to_pascal(schema_name).as_str(),
            resolve_schema_ref(loaded, schema_name),
        );
    }

    let rendered = renderer.finish();
    if rendered.is_empty() {
        lines.push(String::new());
        lines.push(
            "/// Marker emitted when this contract declares no shared wire types.".to_string(),
        );
        lines.push("#[doc(hidden)]".to_string());
        lines.push("pub struct GeneratedTypes;".to_string());
    } else {
        lines.push(String::new());
        lines.push("use serde::{Deserialize, Serialize};".to_string());
        if rendered.iter().any(|line| line.contains("Value")) {
            lines.push("use serde_json::Value;".to_string());
        }
        if rendered.iter().any(|line| line.contains("BTreeMap<")) {
            lines.push("use std::collections::BTreeMap;".to_string());
        }
        lines.push(String::new());
        lines.extend(rendered);
    }
    format!("{}\n", lines.join("\n"))
}

fn render_rpc_rs(loaded: &trellis_contracts::LoadedApi) -> String {
    let mut lines = vec![
        format!(
            "//! Typed RPC descriptors for `{}`.",
            loaded.render_model.id
        ),
        String::new(),
        "use serde::{Deserialize, Serialize};".to_string(),
        String::new(),
    ];

    if loaded
        .render_model
        .rpc
        .values()
        .any(|rpc| rpc.internal != Some(true))
    {
        lines.push("use trellis_rs::generated::RpcDescriptor;".to_string());
        lines.push(String::new());
    }

    lines.push("/// Empty request or response payload used by zero-argument RPCs.".to_string());
    lines.push(
        "#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]".to_string(),
    );
    lines.push("pub struct Empty {}".to_string());
    lines.push(String::new());

    for (key, rpc) in &loaded.render_model.rpc {
        if rpc.internal == Some(true) {
            continue;
        }
        let base = key_to_pascal(key);
        let input_type = if is_empty_object_schema(resolve_schema_ref(loaded, &rpc.input.schema)) {
            "Empty".to_string()
        } else {
            format!("crate::types::{base}Request")
        };
        let output_type = if is_empty_object_schema(resolve_schema_ref(loaded, &rpc.output.schema))
        {
            "Empty".to_string()
        } else {
            format!("crate::types::{base}Response")
        };
        let capabilities = capability_names(&loaded.value, "rpc", key, "call");
        let errors = rpc
            .errors
            .as_ref()
            .map(|values| {
                values
                    .iter()
                    .map(|value| value.error_type.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        lines.push(format!("/// Descriptor for `{key}`."));
        lines.push(format!("pub struct {base}Rpc;"));
        lines.push(String::new());
        let schema_base = key_to_schema_constant_base(key);
        lines.push(format!("impl RpcDescriptor for {base}Rpc {{"));
        lines.push(format!("    type Input = {input_type};"));
        lines.push(format!("    type Output = {output_type};"));
        lines.push(format!(
            "    const INPUT_SCHEMA_JSON: &'static str = crate::schemas::{schema_base}_INPUT_SCHEMA_JSON;"
        ));
        lines.push(format!(
            "    const OUTPUT_SCHEMA_JSON: &'static str = crate::schemas::{schema_base}_OUTPUT_SCHEMA_JSON;"
        ));
        lines.push(format!(
            "    const KEY: &'static str = {};",
            string_literal(key)
        ));
        lines.push(format!(
            "    const SUBJECT: &'static str = {};",
            string_literal(&loaded.subjects.rpc[key])
        ));
        lines.push(format!(
            "    const CALLER_CAPABILITIES: &'static [&'static str] = &[{}];",
            join_string_literals(&capabilities)
        ));
        lines.push(format!(
            "    const ERRORS: &'static [&'static str] = &[{}];",
            join_string_literals(&errors)
        ));
        lines.push("}".to_string());
        lines.push(String::new());

        if !errors.is_empty() {
            lines.push(format!("/// Errors declared by `{key}`."));
            lines.push("#[derive(Debug, Clone, PartialEq)]".to_string());
            lines.push(format!("pub enum {base}Error {{"));
            for error_type in &errors {
                let variant = key_to_pascal(error_type);
                let payload = declared_error_payload_type(loaded, error_type);
                lines.push(format!("    /// `{error_type}` error payload."));
                lines.push(format!("    {variant}({payload}),"));
            }
            lines.push("}".to_string());
            lines.push(String::new());
            lines.push(format!(
                "impl trellis_rs::generated::DeclaredError for {base}Error {{"
            ));
            lines.push("    fn decode(payload: &trellis_rs::generated::RemoteErrorPayload) -> Result<Option<Self>, serde_json::Error> {".to_string());
            lines.push("        match payload.error_type() {".to_string());
            for error_type in &errors {
                let variant = key_to_pascal(error_type);
                let payload_type = declared_error_payload_type(loaded, error_type);
                lines.push(format!(
                    "            Some({}) => payload.decode_declared::<{payload_type}>({}).map(|value| value.map(Self::{variant})),",
                    string_literal(error_type),
                    string_literal(error_type),
                ));
            }
            lines.push("            _ => Ok(None),".to_string());
            lines.push("        }".to_string());
            lines.push("    }".to_string());
            if errors.iter().any(|error| error == "AuthError") {
                lines.push("    fn auth_error_reason(&self) -> Option<&str> {".to_string());
                lines.push(
                    "        match self { Self::AuthError(payload) => Some(payload.reason.as_str()), _ => None }"
                        .to_string(),
                );
                lines.push("    }".to_string());
            }
            lines.push("}".to_string());
            lines.push(String::new());
        }
    }

    format!("{}\n", lines.join("\n"))
}

fn render_events_rs(loaded: &trellis_contracts::LoadedApi) -> String {
    let mut lines = vec![
        format!(
            "//! Typed event descriptors for `{}`.",
            loaded.render_model.id
        ),
        String::new(),
    ];

    if !loaded.render_model.events.is_empty() {
        lines.push("use trellis_rs::generated::EventDescriptor;".to_string());
        lines.push(String::new());
    }

    for key in loaded.render_model.events.keys() {
        let base = key_to_pascal(key);
        let publish = capability_names(&loaded.value, "event", key, "publish");
        let subscribe = capability_names(&loaded.value, "event", key, "subscribe");
        lines.push(format!("/// Descriptor for `{key}`."));
        lines.push(format!("pub struct {base}EventDescriptor;"));
        lines.push(String::new());
        lines.push(format!("impl EventDescriptor for {base}EventDescriptor {{"));
        lines.push(format!("    type Event = crate::types::{base}Event;"));
        lines.push(format!(
            "    const KEY: &'static str = {};",
            string_literal(key)
        ));
        lines.push(format!(
            "    const SUBJECT: &'static str = {};",
            string_literal(&loaded.subjects.events[key].base)
        ));
        lines.push(format!(
            "    const SUBSCRIBE_SUBJECT: &'static str = {};",
            string_literal(&loaded.subjects.events[key].wildcard)
        ));
        lines.push(format!(
            "    const EVENT_SCHEMA_JSON: &'static str = crate::schemas::{}_EVENT_SCHEMA_JSON;",
            key_to_schema_constant_base(key)
        ));
        lines.push(format!(
            "    const PUBLISH_CAPABILITIES: &'static [&'static str] = &[{}];",
            join_string_literals(&publish)
        ));
        lines.push(format!(
            "    const DELEGATED_PUBLISH: bool = {};",
            !publish.is_empty()
        ));
        lines.push(format!(
            "    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &[{}];",
            join_string_literals(&subscribe)
        ));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    format!("{}\n", lines.join("\n"))
}

fn render_feeds_rs(loaded: &trellis_contracts::LoadedApi) -> String {
    let mut lines = vec![
        format!(
            "//! Typed feed descriptors for `{}`.",
            loaded.render_model.id
        ),
        String::new(),
    ];

    if !loaded.render_model.feeds.is_empty() {
        lines.push("use trellis_rs::generated::FeedDescriptor;".to_string());
        lines.push(String::new());
    }

    for (key, feed) in &loaded.render_model.feeds {
        let base = key_to_pascal(key);
        let input_type = if is_empty_object_schema(resolve_schema_ref(loaded, &feed.input.schema)) {
            "crate::rpc::Empty".to_string()
        } else {
            format!("crate::types::{base}Input")
        };
        let event_type = format!("crate::types::{base}Event");
        let subscribe = capability_names(&loaded.value, "feed", key, "subscribe");

        lines.push(format!("/// Descriptor for `{key}`."));
        lines.push(format!("pub struct {base}FeedDescriptor;"));
        lines.push(String::new());
        let schema_base = key_to_schema_constant_base(key);
        lines.push(format!("impl FeedDescriptor for {base}FeedDescriptor {{"));
        lines.push(format!("    type Input = {input_type};"));
        lines.push(format!("    type Event = {event_type};"));
        lines.push(format!(
            "    const INPUT_SCHEMA_JSON: &'static str = crate::schemas::{schema_base}_INPUT_SCHEMA_JSON;"
        ));
        lines.push(format!(
            "    const EVENT_SCHEMA_JSON: &'static str = crate::schemas::{schema_base}_EVENT_SCHEMA_JSON;"
        ));
        lines.push(format!(
            "    const KEY: &'static str = {};",
            string_literal(key)
        ));
        lines.push(format!(
            "    const SUBJECT: &'static str = {};",
            string_literal(&loaded.subjects.feeds[key])
        ));
        lines.push(format!(
            "    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &[{}];",
            join_string_literals(&subscribe)
        ));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    format!("{}\n", lines.join("\n"))
}

fn render_operations_rs(loaded: &trellis_contracts::LoadedApi) -> String {
    let mut lines = vec![format!(
        "//! Typed operation descriptors for `{}`.",
        loaded.render_model.id
    )];

    if loaded.render_model.operations.is_empty() {
        lines.push(String::new());
        return format!("{}\n", lines.join("\n"));
    }

    lines.push(String::new());
    let mut operation_imports = vec!["OperationDescriptor"];
    if loaded
        .render_model
        .operations
        .values()
        .any(|operation| operation.update.is_some())
    {
        operation_imports.push("DeclaredOperationUpdates");
    }
    if loaded
        .render_model
        .operations
        .values()
        .any(|operation| operation.update.is_none())
    {
        operation_imports.push("NoOperationUpdates");
    }
    if loaded
        .render_model
        .operations
        .values()
        .any(|operation| operation.transfer.is_some())
    {
        operation_imports.push("TransferOperationDescriptor");
    }
    operation_imports.sort_unstable();
    lines.push(format!(
        "use trellis_rs::generated::{{{}}};",
        operation_imports.join(", ")
    ));
    if loaded
        .render_model
        .operations
        .values()
        .any(|op| !op.errors.is_empty())
    {
        lines.push("use trellis_rs::service::OperationFailureLike;".to_string());
    }
    lines.push(String::new());

    for (key, operation) in &loaded.render_model.operations {
        let base = key_to_pascal(key);
        let input_type =
            if is_empty_object_schema(resolve_schema_ref(loaded, &operation.input.schema)) {
                "crate::rpc::Empty".to_string()
            } else {
                format!("crate::types::{base}Input")
            };
        let progress_type = match &operation.progress {
            Some(progress)
                if !is_empty_object_schema(resolve_schema_ref(loaded, &progress.schema)) =>
            {
                format!("crate::types::{base}Progress")
            }
            _ => "crate::rpc::Empty".to_string(),
        };
        let output_type = match &operation.output {
            Some(output) if !is_empty_object_schema(resolve_schema_ref(loaded, &output.schema)) => {
                format!("crate::types::{base}Output")
            }
            _ => "crate::rpc::Empty".to_string(),
        };
        let caller = capability_names(&loaded.value, "operation", key, "invoke");
        let observe = capability_names(&loaded.value, "operation", key, "observe");
        let cancel = capability_names(&loaded.value, "operation", key, "cancel");
        let control = capability_names(&loaded.value, "operation", key, "control");
        let error_types: Vec<String> = operation
            .errors
            .iter()
            .map(|error| error.error_type.clone())
            .collect();

        lines.push(format!("/// Descriptor for `{key}`."));
        lines.push(format!("pub struct {base}Operation;"));
        lines.push(String::new());
        let schema_base = key_to_schema_constant_base(key);
        lines.push(format!("impl OperationDescriptor for {base}Operation {{"));
        lines.push(format!("    type Input = {input_type};"));
        lines.push(format!("    type Progress = {progress_type};"));
        lines.push(format!("    type Output = {output_type};"));
        let update_type = operation.update.as_ref().map_or_else(
            || "serde_json::Value".to_string(),
            |update| {
                if is_empty_object_schema(resolve_schema_ref(loaded, &update.schema)) {
                    "crate::rpc::Empty".to_string()
                } else {
                    format!("crate::types::{base}Update")
                }
            },
        );
        lines.push(format!("    type Update = {update_type};"));
        lines.push(format!(
            "    type UpdateEvidence = {};",
            if operation.update.is_some() {
                "DeclaredOperationUpdates"
            } else {
                "NoOperationUpdates"
            }
        ));
        lines.push(format!(
            "    type Error = {};",
            if error_types.is_empty() {
                "trellis_rs::service::OperationFailure".to_string()
            } else {
                format!("{base}OperationError")
            }
        ));
        lines.push(format!(
            "    const INPUT_SCHEMA_JSON: &'static str = crate::schemas::{schema_base}_INPUT_SCHEMA_JSON;"
        ));
        if operation.progress.is_some() {
            lines.push(format!(
                "    const PROGRESS_SCHEMA_JSON: Option<&'static str> = Some(crate::schemas::{schema_base}_PROGRESS_SCHEMA_JSON);"
            ));
        } else {
            lines.push("    const PROGRESS_SCHEMA_JSON: Option<&'static str> = None;".to_string());
        }
        lines.push(format!(
            "    const OUTPUT_SCHEMA_JSON: &'static str = crate::schemas::{schema_base}_OUTPUT_SCHEMA_JSON;"
        ));
        if operation.update.is_some() {
            lines.push(format!(
                "    const UPDATE_SCHEMA_JSON: Option<&'static str> = Some(crate::schemas::{schema_base}_UPDATE_SCHEMA_JSON);"
            ));
        } else {
            lines.push("    const UPDATE_SCHEMA_JSON: Option<&'static str> = None;".to_string());
        }
        lines.push(format!(
            "    const SIGNAL_INPUT_SCHEMAS_JSON: &'static str = crate::schemas::{schema_base}_SIGNAL_INPUT_SCHEMAS_JSON;"
        ));
        lines.push(format!(
            "    const ERRORS: &'static [&'static str] = &[{}];",
            join_string_literals(&error_types)
        ));
        lines.push(format!(
            "    const KEY: &'static str = {};",
            string_literal(key)
        ));
        lines.push(format!(
            "    const SUBJECT: &'static str = {};",
            string_literal(&loaded.subjects.operations[key])
        ));
        lines.push(format!(
            "    const CALLER_CAPABILITIES: &'static [&'static str] = &[{}];",
            join_string_literals(&caller)
        ));
        lines.push(format!(
            "    const OBSERVE_CAPABILITIES: &'static [&'static str] = &[{}];",
            join_string_literals(&observe)
        ));
        lines.push(format!(
            "    const CANCEL_CAPABILITIES: &'static [&'static str] = &[{}];",
            join_string_literals(&cancel)
        ));
        lines.push(format!(
            "    const CONTROL_CAPABILITIES: &'static [&'static str] = &[{}];",
            join_string_literals(&control)
        ));
        lines.push(format!(
            "    const CANCELABLE: bool = {};",
            operation.cancel.unwrap_or(false)
        ));
        lines.push("}".to_string());
        lines.push(String::new());

        // Emit typed error enum when the operation declares errors.
        if !error_types.is_empty() {
            lines.push(format!("/// Errors declared by `{key}`."));
            lines.push("#[derive(Debug, Clone, PartialEq)]".to_string());
            lines.push(format!("pub enum {base}OperationError {{"));
            for error_type in &error_types {
                let variant = key_to_pascal(error_type);
                let payload = declared_error_payload_type(loaded, error_type);
                lines.push(format!("    /// `{error_type}` failure."));
                lines.push(format!("    {variant}({payload}),"));
            }
            lines.push("}".to_string());
            lines.push(String::new());
            lines.push(format!(
                "impl trellis_rs::generated::DeclaredError for {base}OperationError {{"
            ));
            lines.push("    fn decode(payload: &trellis_rs::generated::RemoteErrorPayload) -> Result<Option<Self>, serde_json::Error> {".to_string());
            lines.push("        match payload.error_type() {".to_string());
            for error_type in &error_types {
                let variant = key_to_pascal(error_type);
                let payload_type = declared_error_payload_type(loaded, error_type);
                lines.push(format!(
                    "            Some({}) => payload.decode_declared::<{payload_type}>({}).map(|value| value.map(Self::{variant})),",
                    string_literal(error_type),
                    string_literal(error_type),
                ));
            }
            lines.push("            _ => Ok(None),".to_string());
            lines.push("        }".to_string());
            lines.push("    }".to_string());
            if error_types.iter().any(|error| error == "AuthError") {
                lines.push("    fn auth_error_reason(&self) -> Option<&str> {".to_string());
                lines.push(
                    "        match self { Self::AuthError(payload) => Some(payload.reason.as_str()), _ => None }"
                        .to_string(),
                );
                lines.push("    }".to_string());
            }
            lines.push("}".to_string());
            lines.push(String::new());
            lines.push(format!(
                "impl OperationFailureLike for {base}OperationError {{"
            ));
            lines.push("    fn error_type(&self) -> &str {".to_string());
            lines.push("        match self {".to_string());
            for error_type in &error_types {
                let variant = key_to_pascal(error_type);
                lines.push(format!(
                    "            Self::{variant}(..) => {},",
                    string_literal(error_type)
                ));
            }
            lines.push("        }".to_string());
            lines.push("    }".to_string());
            lines.push("    fn message(&self) -> String {".to_string());
            lines.push("        self.fields().remove(\"message\").and_then(|value| value.as_str().map(ToOwned::to_owned)).unwrap_or_else(|| self.error_type().to_string())".to_string());
            lines.push("    }".to_string());
            lines.push(
                "    fn fields(&self) -> serde_json::Map<String, serde_json::Value> {".to_string(),
            );
            lines.push("        let value = match self {".to_string());
            for error_type in &error_types {
                let variant = key_to_pascal(error_type);
                lines.push(format!(
                    "            Self::{variant}(payload) => serde_json::to_value(payload),"
                ));
            }
            lines.push("        };".to_string());
            lines.push("        value.ok().and_then(|value| value.as_object().cloned()).unwrap_or_default()".to_string());
            lines.push("    }".to_string());
            lines.push("}".to_string());
            lines.push(String::new());
        }

        if operation.transfer.is_some() {
            lines.push(format!(
                "impl TransferOperationDescriptor for {base}Operation {{}}"
            ));
            lines.push(String::new());
        }
    }

    format!("{}\n", lines.join("\n"))
}

fn render_participant_job_descriptors_rs(loaded: &LoadedParticipant) -> String {
    let mut lines = vec![
        format!(
            "//! Typed jobs descriptors for `{}`.",
            loaded.render_model.id
        ),
        String::new(),
    ];
    if !loaded.render_model.jobs.is_empty() {
        let mut renderer = TypeRenderer::default();
        for job in loaded.render_model.jobs.values() {
            for schema_name in [
                Some(job.payload.schema.as_str()),
                job.result.as_ref().map(|schema| schema.schema.as_str()),
                job.update.as_ref().map(|schema| schema.schema.as_str()),
            ]
            .into_iter()
            .flatten()
            {
                let schema = loaded
                    .render_model
                    .schemas
                    .get(schema_name)
                    .expect("validated job schema");
                if !is_empty_object_schema(schema) {
                    renderer.render_named_type(&key_to_pascal(schema_name), schema);
                }
            }
        }
        let rendered = renderer.finish();
        if !rendered.is_empty() {
            lines.push("use serde::{Deserialize, Serialize};".to_string());
            if rendered.iter().any(|line| line.contains("Value")) {
                lines.push("use serde_json::Value;".to_string());
            }
            lines.push(String::new());
            lines.extend(rendered);
            lines.push(String::new());
        }
        lines.push("use trellis_rs::service::JobDescriptor;".to_string());
    }
    if loaded
        .render_model
        .jobs
        .values()
        .any(|job| job.update.is_some())
    {
        lines.push("use trellis_rs::service::JobUpdateDescriptor;".to_string());
    }
    if !loaded.render_model.jobs.is_empty() {
        lines.push(String::new());
    }

    for (key, job) in &loaded.render_model.jobs {
        let base = key_to_pascal(key);
        let payload_type = if is_empty_object_schema(
            loaded
                .render_model
                .schemas
                .get(&job.payload.schema)
                .expect("validated job payload schema"),
        ) {
            "serde_json::Value".to_string()
        } else {
            format!("crate::jobs::{}", key_to_pascal(&job.payload.schema))
        };
        let result_type = job
            .result
            .as_ref()
            .map(|result| {
                if is_empty_object_schema(
                    loaded
                        .render_model
                        .schemas
                        .get(&result.schema)
                        .expect("validated job result schema"),
                ) {
                    "serde_json::Value".to_string()
                } else {
                    format!("crate::jobs::{}", key_to_pascal(&result.schema))
                }
            })
            .unwrap_or_else(|| "serde_json::Value".to_string());
        lines.push(format!("/// Descriptor for jobs queue `{key}`."));
        lines.push(format!("pub struct {base}Job;"));
        lines.push(String::new());
        lines.push(format!("impl JobDescriptor for {base}Job {{"));
        lines.push(format!("    type Payload = {payload_type};"));
        lines.push(format!("    type Result = {result_type};"));
        lines.push(format!(
            "    const QUEUE_TYPE: &'static str = {};",
            string_literal(key)
        ));
        let payload_schema = loaded
            .render_model
            .schemas
            .get(&job.payload.schema)
            .expect("validated job payload schema");
        lines.push(format!(
            "    const PAYLOAD_SCHEMA_JSON: &'static str = {};",
            string_literal(&canonicalize_json(payload_schema).expect("valid job payload schema"))
        ));
        match &job.result {
            Some(result) => {
                let schema = loaded
                    .render_model
                    .schemas
                    .get(&result.schema)
                    .expect("validated job result schema");
                lines.push(format!(
                    "    const RESULT_SCHEMA_JSON: Option<&'static str> = Some({});",
                    string_literal(&canonicalize_json(schema).expect("valid job result schema"))
                ));
            }
            None => {
                lines.push("    const RESULT_SCHEMA_JSON: Option<&'static str> = None;".to_string())
            }
        }
        lines.push("}".to_string());
        lines.push(String::new());

        if let Some(update) = &job.update {
            let update_type = if is_empty_object_schema(
                loaded
                    .render_model
                    .schemas
                    .get(&update.schema)
                    .expect("validated job update schema"),
            ) {
                "serde_json::Value".to_string()
            } else {
                format!("crate::jobs::{}", key_to_pascal(&update.schema))
            };
            lines.push(format!("impl JobUpdateDescriptor for {base}Job {{"));
            lines.push(format!("    type Update = {update_type};"));
            lines.push(format!(
                "    const UPDATE_SCHEMA: &'static str = {};",
                string_literal(&update.schema)
            ));
            let schema = loaded
                .render_model
                .schemas
                .get(&update.schema)
                .expect("validated job update schema");
            lines.push(format!(
                "    const UPDATE_SCHEMA_JSON: &'static str = {};",
                string_literal(&canonicalize_json(schema).expect("valid job update schema"))
            ));
            lines.push("}".to_string());
            lines.push(String::new());
        }
    }
    format!("{}\n", lines.join("\n"))
}

fn render_client_rs(loaded: &trellis_contracts::LoadedApi) -> String {
    let client_name = format!("{}Client", sdk_stem_pascal(loaded));
    let mut lines = vec![
        format!(
            "//! Thin typed client helpers for `{}`.",
            loaded.render_model.id
        ),
        String::new(),
        format!(
            "/// Typed API wrapper for the `{}` contract.",
            loaded.render_model.id
        ),
        format!("pub struct {client_name}<'a> {{"),
        "    inner: &'a trellis_rs::generated::Caller,".to_string(),
        "}".to_string(),
        String::new(),
        format!("impl<'a> {client_name}<'a> {{"),
        "    /// Wrap an already connected low-level Trellis client.".to_string(),
        "    pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self {".to_string(),
        "        Self { inner }".to_string(),
        "    }".to_string(),
        String::new(),
        "    /// Access typed RPC calls.".to_string(),
        "    pub fn rpc(&self) -> Rpc<'a> { Rpc { _inner: self.inner } }".to_string(),
        String::new(),
        "    /// Access typed events.".to_string(),
        "    pub fn event(&self) -> Event<'a> { Event { _inner: self.inner } }".to_string(),
        String::new(),
        "    /// Access typed feeds.".to_string(),
        "    pub fn feed(&self) -> Feed<'a> { Feed { _inner: self.inner } }".to_string(),
        String::new(),
        "    /// Access typed operations.".to_string(),
        "    pub fn operation(&self) -> Operation<'a> { Operation { _inner: self.inner } }"
            .to_string(),
        String::new(),
        "}".to_string(),
        String::new(),
    ];

    if !loaded.render_model.events.is_empty()
        || !loaded.render_model.feeds.is_empty()
        || !loaded.render_model.operations.is_empty()
    {
        lines.insert(
            2,
            "use trellis_rs::generated::TrellisClientError;".to_string(),
        );
        lines.insert(3, String::new());
    }

    render_client_rpc_surface(loaded, &mut lines);
    render_client_event_surface(loaded, &mut lines);
    render_client_feed_surface(loaded, &mut lines);
    render_client_operation_surface(loaded, &mut lines);

    format!("{}\n", lines.join("\n"))
}

fn surface_group_and_method(key: &str) -> (String, String) {
    let parts = key
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let group = parts.first().copied().unwrap_or(key);
    let tail = if parts.len() > 1 {
        parts[1..].join(".")
    } else {
        key.to_string()
    };
    (
        rust_ident(&key_to_snake(group)),
        rust_ident(&key_to_snake(&tail)),
    )
}

fn grouped_keys<'a, T>(
    items: &'a std::collections::BTreeMap<String, T>,
) -> std::collections::BTreeMap<String, Vec<&'a str>> {
    let mut groups = std::collections::BTreeMap::<String, Vec<&'a str>>::new();
    for key in items.keys() {
        groups
            .entry(surface_group_and_method(key).0)
            .or_default()
            .push(key.as_str());
    }
    groups
}

fn public_rpc_keys(loaded: &trellis_contracts::LoadedApi) -> Vec<&str> {
    loaded
        .render_model
        .rpc
        .iter()
        .filter_map(|(key, rpc)| (rpc.internal != Some(true)).then_some(key.as_str()))
        .collect()
}

fn grouped_public_rpc_keys(
    loaded: &trellis_contracts::LoadedApi,
) -> std::collections::BTreeMap<String, Vec<&str>> {
    let mut groups = std::collections::BTreeMap::<String, Vec<&str>>::new();
    for key in public_rpc_keys(loaded) {
        groups
            .entry(surface_group_and_method(key).0)
            .or_default()
            .push(key);
    }
    groups
}

fn render_client_rpc_surface(loaded: &trellis_contracts::LoadedApi, lines: &mut Vec<String>) {
    lines.extend([
        "/// Typed RPC surface.".to_string(),
        "pub struct Rpc<'a> { pub(crate) _inner: &'a trellis_rs::generated::Caller }".to_string(),
        "impl<'a> Rpc<'a> {".to_string(),
    ]);
    for group in grouped_public_rpc_keys(loaded).keys() {
        let group_ty = format!("{}Rpc", key_to_pascal(group));
        lines.push(format!("    /// Access the `{group}` RPC group."));
        lines.push(format!(
            "    pub fn {group}(&self) -> {group_ty}<'a> {{ {group_ty} {{ inner: self._inner }} }}"
        ));
    }
    lines.extend(["}".to_string(), String::new()]);

    for (group, keys) in grouped_public_rpc_keys(loaded) {
        let group_ty = format!("{}Rpc", key_to_pascal(&group));
        lines.push(format!("/// Typed RPC methods in the `{group}` group."));
        lines.push(format!(
            "pub struct {group_ty}<'a> {{ inner: &'a trellis_rs::generated::Caller }}"
        ));
        lines.push(format!("impl<'a> {group_ty}<'a> {{"));
        for key in keys {
            let rpc = &loaded.render_model.rpc[key];
            let base = key_to_pascal(key);
            let (_, method_name) = surface_group_and_method(key);
            let output_type =
                if is_empty_object_schema(resolve_schema_ref(loaded, &rpc.output.schema)) {
                    "crate::rpc::Empty".to_string()
                } else {
                    format!("crate::types::{base}Response")
                };
            let errors = rpc
                .errors
                .as_ref()
                .map(|errors| {
                    errors
                        .iter()
                        .map(|error| error.error_type.as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let error_type = if errors.is_empty() {
                "trellis_rs::generated::NoDeclaredError".to_string()
            } else {
                format!("crate::rpc::{base}Error")
            };
            lines.push(format!("    /// Call `{key}`."));
            if is_empty_object_schema(resolve_schema_ref(loaded, &rpc.input.schema)) {
                lines.push(format!(
                    "    pub async fn {method_name}(&self) -> Result<{output_type}, trellis_rs::generated::CallError<{error_type}>> {{"
                ));
                lines.push(format!(
                    "        self.inner.call_typed::<crate::rpc::{base}Rpc, {error_type}>(&crate::rpc::Empty {{}}).await"
                ));
            } else {
                lines.push(format!(
                    "    pub async fn {method_name}(&self, input: &crate::types::{base}Request) -> Result<{output_type}, trellis_rs::generated::CallError<{error_type}>> {{"
                ));
                lines.push(format!(
                    "        self.inner.call_typed::<crate::rpc::{base}Rpc, {error_type}>(input).await"
                ));
            }
            lines.push("    }".to_string());
            lines.push(String::new());
        }
        lines.extend(["}".to_string(), String::new()]);
    }
}

fn render_client_event_surface(loaded: &trellis_contracts::LoadedApi, lines: &mut Vec<String>) {
    lines.extend([
        "/// Typed event surface.".to_string(),
        "pub struct Event<'a> { pub(crate) _inner: &'a trellis_rs::generated::Caller }".to_string(),
        "impl<'a> Event<'a> {".to_string(),
    ]);
    for group in grouped_keys(&loaded.render_model.events).keys() {
        let group_ty = format!("{}Event", key_to_pascal(group));
        lines.push(format!("    /// Access the `{group}` event group."));
        lines.push(format!(
            "    pub fn {group}(&self) -> {group_ty}<'a> {{ {group_ty} {{ inner: self._inner }} }}"
        ));
    }
    lines.extend(["}".to_string(), String::new()]);

    for (group, keys) in grouped_keys(&loaded.render_model.events) {
        let group_ty = format!("{}Event", key_to_pascal(&group));
        let mut leaf_lines = Vec::new();
        lines.push(format!("/// Typed events in the `{group}` group."));
        lines.push(format!(
            "pub struct {group_ty}<'a> {{ inner: &'a trellis_rs::generated::Caller }}"
        ));
        lines.push(format!("impl<'a> {group_ty}<'a> {{"));
        for key in keys {
            let base = key_to_pascal(key);
            let (_, method_name) = surface_group_and_method(key);
            let leaf_ty = format!(
                "{}{}Event",
                key_to_pascal(&group),
                key_to_pascal(&method_name)
            );
            lines.push(format!("    /// Access `{key}`."));
            lines.push(format!(
                "    pub fn {method_name}(&self) -> {leaf_ty}<'a> {{ {leaf_ty} {{ inner: self.inner }} }}"
            ));
            lines.push(String::new());
            leaf_lines.push(format!("/// Typed `{key}` event operations."));
            leaf_lines.push(format!(
                "pub struct {leaf_ty}<'a> {{ inner: &'a trellis_rs::generated::Caller }}"
            ));
            leaf_lines.push(format!("impl<'a> {leaf_ty}<'a> {{"));
            leaf_lines.push(format!("    /// Publish `{key}`."));
            leaf_lines.push(format!(
                "    pub async fn publish(&self, event: &crate::types::{base}Event) -> Result<(), TrellisClientError> {{"
            ));
            leaf_lines.push(format!(
                "        self.inner.publish::<crate::events::{base}EventDescriptor>(event).await"
            ));
            leaf_lines.push("    }".to_string());
            leaf_lines.push(format!("    /// Listen for live `{key}` events."));
            leaf_lines.push(format!("    pub async fn listen<F, Fut>(&self, handler: F) -> Result<(), TrellisClientError> where F: Fn(crate::types::{base}Event) -> Fut, Fut: std::future::Future<Output = Result<(), TrellisClientError>> {{"));
            leaf_lines.push(format!(
                "        let mut stream = self.inner.subscribe::<crate::events::{base}EventDescriptor>().await?;"
            ));
            leaf_lines.push("        while let Some(event) = futures_util::StreamExt::next(&mut stream).await {".to_string());
            leaf_lines.push("            handler(event?).await?;".to_string());
            leaf_lines.push("        }".to_string());
            leaf_lines.push("        Ok(())".to_string());
            leaf_lines.push("    }".to_string());
            leaf_lines.extend(["}".to_string(), String::new()]);
        }
        lines.extend(["}".to_string(), String::new()]);
        lines.extend(leaf_lines);
    }
}

fn render_client_feed_surface(loaded: &trellis_contracts::LoadedApi, lines: &mut Vec<String>) {
    lines.extend([
        "/// Typed feed surface.".to_string(),
        "pub struct Feed<'a> { pub(crate) _inner: &'a trellis_rs::generated::Caller }".to_string(),
        "impl<'a> Feed<'a> {".to_string(),
    ]);
    for group in grouped_keys(&loaded.render_model.feeds).keys() {
        let group_ty = format!("{}Feed", key_to_pascal(group));
        lines.push(format!("    /// Access the `{group}` feed group."));
        lines.push(format!(
            "    pub fn {group}(&self) -> {group_ty}<'a> {{ {group_ty} {{ inner: self._inner }} }}"
        ));
    }
    lines.extend(["}".to_string(), String::new()]);

    for (group, keys) in grouped_keys(&loaded.render_model.feeds) {
        let group_ty = format!("{}Feed", key_to_pascal(&group));
        lines.push(format!("/// Typed feeds in the `{group}` group."));
        lines.push(format!(
            "pub struct {group_ty}<'a> {{ inner: &'a trellis_rs::generated::Caller }}"
        ));
        lines.push(format!("impl<'a> {group_ty}<'a> {{"));
        for key in keys {
            let feed = &loaded.render_model.feeds[key];
            let base = key_to_pascal(key);
            let (_, method_name) = surface_group_and_method(key);
            lines.push(format!("    /// Subscribe to `{key}`."));
            if is_empty_object_schema(resolve_schema_ref(loaded, &feed.input.schema)) {
                lines.push(format!("    pub async fn {method_name}(&self) -> Result<futures_util::stream::BoxStream<'static, Result<crate::types::{base}Event, TrellisClientError>>, TrellisClientError> {{"));
                lines.push(format!("        self.inner.feed::<crate::feeds::{base}FeedDescriptor>(&crate::rpc::Empty {{}}).await"));
            } else {
                lines.push(format!("    pub async fn {method_name}(&self, input: &crate::types::{base}Input) -> Result<futures_util::stream::BoxStream<'static, Result<crate::types::{base}Event, TrellisClientError>>, TrellisClientError> {{"));
                lines.push(format!(
                    "        self.inner.feed::<crate::feeds::{base}FeedDescriptor>(input).await"
                ));
            }
            lines.push("    }".to_string());
            lines.push(String::new());
        }
        lines.extend(["}".to_string(), String::new()]);
    }
}

fn render_client_operation_surface(loaded: &trellis_contracts::LoadedApi, lines: &mut Vec<String>) {
    lines.extend([
        "/// Typed operation surface.".to_string(),
        "pub struct Operation<'a> { pub(crate) _inner: &'a trellis_rs::generated::Caller }"
            .to_string(),
        "impl<'a> Operation<'a> {".to_string(),
    ]);
    for group in grouped_keys(&loaded.render_model.operations).keys() {
        let group_ty = format!("{}Operation", key_to_pascal(group));
        lines.push(format!("    /// Access the `{group}` operation group."));
        lines.push(format!(
            "    pub fn {group}(&self) -> {group_ty}<'a> {{ {group_ty} {{ inner: self._inner }} }}"
        ));
    }
    lines.extend(["}".to_string(), String::new()]);

    for (group, keys) in grouped_keys(&loaded.render_model.operations) {
        let group_ty = format!("{}Operation", key_to_pascal(&group));
        let mut leaf_lines = Vec::new();
        lines.push(format!("/// Typed operations in the `{group}` group."));
        lines.push(format!(
            "pub struct {group_ty}<'a> {{ inner: &'a trellis_rs::generated::Caller }}"
        ));
        lines.push(format!("impl<'a> {group_ty}<'a> {{"));
        for key in keys {
            let base = key_to_pascal(key);
            let (_, method_name) = surface_group_and_method(key);
            let leaf_ty = format!(
                "{}{}Operation",
                key_to_pascal(&group),
                key_to_pascal(&method_name)
            );
            lines.push(format!("    /// Access `{key}`."));
            lines.push(format!(
                "    pub fn {method_name}(&self) -> {leaf_ty}<'a> {{ {leaf_ty} {{ inner: self.inner }} }}"
            ));
            lines.push(String::new());
            leaf_lines.push(format!("/// Typed `{key}` operation controls."));
            leaf_lines.push(format!(
                "pub struct {leaf_ty}<'a> {{ inner: &'a trellis_rs::generated::Caller }}"
            ));
            leaf_lines.push(format!("impl<'a> {leaf_ty}<'a> {{"));
            leaf_lines.push(format!("    /// Start `{key}`."));
            leaf_lines.push(format!("    pub async fn start(&self, input: &crate::types::{base}Input) -> Result<trellis_rs::generated::OperationRef<'a, trellis_rs::generated::Caller, crate::operations::{base}Operation>, TrellisClientError> {{"));
            leaf_lines.push(format!(
                "        self.inner.operation::<crate::operations::{base}Operation>().start(input).await"
            ));
            leaf_lines.push("    }".to_string());
            leaf_lines.extend(["}".to_string(), String::new()]);
        }
        lines.extend(["}".to_string(), String::new()]);
        lines.extend(leaf_lines);
    }
}

fn write_if_changed(path: &Path, contents: &str) -> Result<(), CodegenRustError> {
    if fs::read_to_string(path).ok().as_deref() == Some(contents) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

fn remove_if_exists(path: &Path) -> Result<(), CodegenRustError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn write_rust_if_changed(path: &Path, contents: &str) -> Result<(), CodegenRustError> {
    let contents = format_generated_rust_source(path, contents)?;
    write_if_changed(path, &contents)
}

fn format_generated_rust_source(
    path: impl AsRef<Path>,
    contents: &str,
) -> Result<String, CodegenRustError> {
    let path = path.as_ref().display().to_string();
    let file = syn::parse_file(contents).map_err(|error| CodegenRustError::RustSyntax {
        path: path.clone(),
        message: error.to_string(),
    })?;
    Ok(prettyplease::unparse(&file))
}

fn format_generated_rust_files(root: &Path) -> Result<(), CodegenRustError> {
    fn collect(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                collect(&path, files)?;
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    collect(root, &mut files)?;
    files.sort();
    if files.is_empty() {
        return Ok(());
    }
    let output = Command::new("rustfmt")
        .args(["--edition", "2021"])
        .args(&files)
        .output()
        .map_err(|error| CodegenRustError::RustFormat {
            path: root.display().to_string(),
            message: format!("failed to start rustfmt: {error}"),
        })?;

    if !output.status.success() {
        return Err(CodegenRustError::RustFormat {
            path: root.display().to_string(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(())
}

fn key_to_pascal(value: &str) -> String {
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

fn rust_variant_ident(value: &str) -> String {
    let ident = key_to_pascal(value);
    if ident.is_empty() {
        "Value".to_string()
    } else if ident.starts_with(|character: char| character.is_ascii_digit()) {
        format!("V{ident}")
    } else {
        ident
    }
}

fn key_to_snake(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut prev_was_sep = false;
    for (index, ch) in chars.iter().copied().enumerate() {
        if ch.is_ascii_alphanumeric() {
            let prev = index.checked_sub(1).and_then(|i| chars.get(i)).copied();
            let next = chars.get(index + 1).copied();
            let starts_new_word = ch.is_ascii_uppercase()
                && !out.is_empty()
                && !prev_was_sep
                && (prev.is_some_and(|value| value.is_ascii_lowercase() || value.is_ascii_digit())
                    || next.is_some_and(|value| value.is_ascii_lowercase()));

            if starts_new_word {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            prev_was_sep = false;
        } else if !out.is_empty() && !prev_was_sep {
            out.push('_');
            prev_was_sep = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

fn rust_ident(value: &str) -> String {
    match value {
        "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" | "false"
        | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move"
        | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super"
        | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while" | "async" | "await"
        | "dyn" | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override"
        | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "try" => format!("r#{value}"),
        _ => value.to_string(),
    }
}

fn rust_schema_type_segment(value: &str) -> String {
    key_to_pascal(value).replace("Nats", "Transport")
}

fn rust_schema_field_base(value: &str) -> String {
    match key_to_snake(value).as_str() {
        "nats" => "transport_rules".to_string(),
        "nats_servers" => "servers".to_string(),
        other => other.replace("nats", "transport"),
    }
}

#[derive(Default)]
struct TypeRenderer {
    rendered: std::collections::BTreeSet<String>,
    defs: Vec<String>,
    needs_optional_nullable_helper: bool,
}

impl TypeRenderer {
    fn render_named_type(&mut self, type_name: &str, schema: &serde_json::Value) {
        if self.rendered.contains(type_name) {
            return;
        }
        self.rendered.insert(type_name.to_string());

        if let Some(values) = string_enum_values(schema) {
            self.defs
                .push(format!("/// Generated schema type `{type_name}`."));
            self.defs
                .push("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]".to_string());
            self.defs.push(format!("pub enum {type_name} {{"));
            for value in values {
                let variant = rust_variant_ident(value);
                self.defs.push(format!("    /// The `{value}` wire value."));
                self.defs
                    .push(format!("    #[serde(rename = {})]", string_literal(value)));
                self.defs.push(format!("    {variant},"));
            }
            self.defs.push("}".to_string());
            self.defs.push(format!("impl {type_name} {{"));
            self.defs
                .push("    /// Return the contract wire value.".to_string());
            self.defs
                .push("    pub const fn as_str(&self) -> &'static str {".to_string());
            self.defs.push("        match self {".to_string());
            for value in string_enum_values(schema).expect("rendered string enum") {
                let variant = rust_variant_ident(value);
                self.defs.push(format!(
                    "            Self::{variant} => {},",
                    string_literal(value)
                ));
            }
            self.defs.push("        }".to_string());
            self.defs.push("    }".to_string());
            self.defs.push("}".to_string());
            self.defs
                .push(format!("impl AsRef<str> for {type_name} {{"));
            self.defs
                .push("    fn as_ref(&self) -> &str { self.as_str() }".to_string());
            self.defs.push("}".to_string());
            self.defs
                .push(format!("impl std::fmt::Display for {type_name} {{"));
            self.defs.push(
                "    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { formatter.write_str(self.as_str()) }"
                    .to_string(),
            );
            self.defs.push("}".to_string());
            self.defs
                .push(format!("impl PartialEq<&str> for {type_name} {{"));
            self.defs.push(
                "    fn eq(&self, other: &&str) -> bool { self.as_str() == *other }".to_string(),
            );
            self.defs.push("}".to_string());
            self.defs
                .push(format!("impl PartialEq<{type_name}> for &str {{"));
            self.defs.push(format!(
                "    fn eq(&self, other: &{type_name}) -> bool {{ *self == other.as_str() }}"
            ));
            self.defs.push("}".to_string());
            self.defs.push(String::new());
            return;
        }

        if let Some((tag, variants)) = tagged_object_union(schema) {
            let rendered = variants
                .into_iter()
                .map(|(tag_value, variant_schema)| {
                    let variant = rust_variant_ident(&tag_value);
                    let fields = self
                        .render_object_fields(
                            &format!("{type_name}{variant}"),
                            variant_schema,
                            Some(&tag),
                        )
                        .into_iter()
                        .map(|line| line.replacen("    pub ", "    ", 1))
                        .collect::<Vec<_>>();
                    (tag_value, variant, fields)
                })
                .collect::<Vec<_>>();
            self.defs
                .push(format!("/// Generated schema type `{type_name}`."));
            self.defs
                .push("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]".to_string());
            self.defs
                .push(format!("#[serde(tag = {})]", string_literal(&tag)));
            self.defs.push(format!("pub enum {type_name} {{"));
            for (tag_value, variant, fields) in rendered {
                self.defs
                    .push(format!("    /// The `{tag_value}` variant."));
                self.defs.push(format!(
                    "    #[serde(rename = {})]",
                    string_literal(&tag_value)
                ));
                self.defs.push(format!("    {variant} {{"));
                self.defs
                    .extend(fields.into_iter().map(|line| format!("    {line}")));
                self.defs.push("    },".to_string());
            }
            self.defs.push("}".to_string());
            self.defs.push(String::new());
            return;
        }

        if let Some(variants) = object_union_variants(schema) {
            let mut used_names = BTreeSet::new();
            let mut rendered = variants
                .iter()
                .enumerate()
                .map(|(index, variant_schema)| {
                    let mut variant = object_variant_name(variant_schema)
                        .unwrap_or_else(|| format!("Variant{}", index + 1));
                    if !used_names.insert(variant.clone()) {
                        variant.push_str(&(index + 1).to_string());
                        used_names.insert(variant.clone());
                    }
                    let fields = self
                        .render_object_fields(
                            &format!("{type_name}{variant}"),
                            variant_schema,
                            None,
                        )
                        .into_iter()
                        .map(|line| line.replacen("    pub ", "    ", 1))
                        .collect::<Vec<_>>();
                    let required = variant_schema
                        .get("required")
                        .and_then(serde_json::Value::as_array)
                        .map_or(0, Vec::len);
                    let properties = variant_schema
                        .get("properties")
                        .and_then(serde_json::Value::as_object)
                        .map_or(0, serde_json::Map::len);
                    (required, properties, index, variant, fields)
                })
                .collect::<Vec<_>>();
            rendered.sort_by(|left, right| {
                right
                    .0
                    .cmp(&left.0)
                    .then_with(|| right.1.cmp(&left.1))
                    .then_with(|| left.2.cmp(&right.2))
            });
            self.defs
                .push(format!("/// Generated schema type `{type_name}`."));
            self.defs
                .push("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]".to_string());
            self.defs.push("#[serde(untagged)]".to_string());
            self.defs.push(format!("pub enum {type_name} {{"));
            for (_, _, _, variant, fields) in rendered {
                self.defs.push(format!("    /// The `{variant}` variant."));
                self.defs.push(format!("    {variant} {{"));
                self.defs
                    .extend(fields.into_iter().map(|line| format!("    {line}")));
                self.defs.push("    },".to_string());
            }
            self.defs.push("}".to_string());
            self.defs.push(String::new());
            return;
        }

        if object_fields(schema).is_some() {
            let field_lines = self.render_object_fields(type_name, schema, None);
            self.defs
                .push(format!("/// Generated schema type `{type_name}`."));
            self.defs
                .push("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]".to_string());
            self.defs.push(format!("pub struct {type_name} {{"));
            self.defs.extend(field_lines);
            self.defs.push("}".to_string());
            self.defs.push(String::new());
            return;
        }

        let expr = self.scalar_or_container_expr(type_name, schema);
        self.defs
            .push(format!("/// Generated schema type `{type_name}`."));
        self.defs
            .push("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]".to_string());
        self.defs.push(format!(
            "pub struct {type_name}(#[doc = \"The wrapped wire value.\"] pub {expr});"
        ));
        self.defs.push(String::new());
    }

    fn render_object_fields(
        &mut self,
        type_name: &str,
        schema: &serde_json::Value,
        skip: Option<&str>,
    ) -> Vec<String> {
        let mut field_lines = Vec::new();
        for (field_name, field_schema) in object_fields(schema).into_iter().flatten() {
            if skip == Some(field_name.as_str()) {
                continue;
            }
            let rust_field_base = rust_schema_field_base(field_name);
            let rust_field = rust_ident(&rust_field_base);
            field_lines.push(format!("    /// The `{field_name}` wire field."));
            if rust_field_base != *field_name {
                field_lines.push(format!(
                    "    #[serde(rename = {})]",
                    string_literal(field_name)
                ));
            }
            let required = schema_required(schema, field_name);
            let field_type_name = format!("{type_name}{}", rust_schema_type_segment(field_name));
            let ty = self.type_expr(&field_type_name, field_schema);
            let nullable = schema_is_nullable(field_schema);
            if required && nullable {
                field_lines.push(format!("    pub {rust_field}: Option<{ty}>,"));
            } else if required {
                field_lines.push(format!("    pub {rust_field}: {ty},"));
            } else if nullable {
                self.needs_optional_nullable_helper = true;
                field_lines.push(
                    "    #[serde(default, deserialize_with = \"deserialize_optional_nullable\", skip_serializing_if = \"Option::is_none\")]"
                        .to_string(),
                );
                field_lines.push(format!("    pub {rust_field}: Option<Option<{ty}>>,"));
            } else {
                field_lines
                    .push("    #[serde(skip_serializing_if = \"Option::is_none\")]".to_string());
                field_lines.push(format!("    pub {rust_field}: Option<{ty}>,"));
            }
        }
        field_lines
    }

    fn type_expr(&mut self, type_name: &str, schema: &serde_json::Value) -> String {
        if object_fields(schema).is_some()
            || string_enum_values(schema).is_some()
            || tagged_object_union(schema).is_some()
            || object_union_variants(schema).is_some()
        {
            self.render_named_type(type_name, schema);
            return type_name.to_string();
        }

        self.scalar_or_container_expr(type_name, schema)
    }

    fn scalar_or_container_expr(&mut self, type_name: &str, schema: &serde_json::Value) -> String {
        if let Some(non_null) = single_non_null_variant(schema) {
            return self.scalar_or_container_expr(type_name, non_null);
        }
        if let Some(ty) = union_base_type(schema) {
            return ty.to_string();
        }

        if let Some(types) = schema.get("type").and_then(serde_json::Value::as_array) {
            let non_null = types
                .iter()
                .filter_map(serde_json::Value::as_str)
                .filter(|kind| *kind != "null")
                .collect::<Vec<_>>();
            if non_null.len() == 1 {
                let mut cloned = schema.clone();
                cloned["type"] = serde_json::Value::String(non_null[0].to_string());
                return self.scalar_or_container_expr(type_name, &cloned);
            }
            return "Value".to_string();
        }

        if schema.get("enum").is_some() || schema.get("const").is_some() {
            return literal_base_type(schema).unwrap_or("Value").to_string();
        }

        match schema.get("type").and_then(serde_json::Value::as_str) {
            Some("string") => "String".to_string(),
            Some("boolean") => "bool".to_string(),
            Some("integer") => "i64".to_string(),
            Some("number") => "f64".to_string(),
            Some("array") => {
                let item_schema = schema.get("items").unwrap_or(&serde_json::Value::Null);
                let item_name = format!("{type_name}Item");
                let item_type = self.type_expr(&item_name, item_schema);
                format!("Vec<{item_type}>")
            }
            Some("object") => {
                if let Some(value_schema) = object_map_value_schema(schema) {
                    let value_name = format!("{type_name}Value");
                    let value_type = self.type_expr(&value_name, value_schema);
                    return format!("BTreeMap<String, {value_type}>");
                }
                "BTreeMap<String, Value>".to_string()
            }
            _ => "Value".to_string(),
        }
    }

    fn finish(mut self) -> Vec<String> {
        if self.needs_optional_nullable_helper {
            self.defs.splice(
                0..0,
                [
                    "fn deserialize_optional_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>".to_string(),
                    "where".to_string(),
                    "    D: serde::Deserializer<'de>,".to_string(),
                    "    T: serde::Deserialize<'de>,".to_string(),
                    "{".to_string(),
                    "    Option::<T>::deserialize(deserializer).map(Some)".to_string(),
                    "}".to_string(),
                    String::new(),
                ],
            );
        }
        self.defs
    }
}

fn single_non_null_variant(schema: &serde_json::Value) -> Option<&serde_json::Value> {
    let variants = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))?
        .as_array()?;
    let non_null = variants
        .iter()
        .filter(|variant| !is_null_schema(variant))
        .collect::<Vec<_>>();
    (non_null.len() == 1 && non_null.len() != variants.len()).then_some(non_null[0])
}

fn is_null_schema(schema: &serde_json::Value) -> bool {
    schema.get("type").and_then(serde_json::Value::as_str) == Some("null")
}

fn schema_is_nullable(schema: &serde_json::Value) -> bool {
    schema
        .get("type")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|types| types.iter().any(|kind| kind.as_str() == Some("null")))
        || schema
            .get("anyOf")
            .or_else(|| schema.get("oneOf"))
            .and_then(serde_json::Value::as_array)
            .is_some_and(|variants| variants.iter().any(is_null_schema))
}

fn string_enum_values(schema: &serde_json::Value) -> Option<Vec<&str>> {
    if let Some(value) = schema.get("const").and_then(serde_json::Value::as_str) {
        return Some(vec![value]);
    }
    if let Some(values) = schema.get("enum").and_then(serde_json::Value::as_array) {
        let values = values
            .iter()
            .map(serde_json::Value::as_str)
            .collect::<Option<Vec<_>>>()?;
        return (!values.is_empty()).then_some(values);
    }
    let variants = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))?
        .as_array()?;
    let mut values = Vec::new();
    for variant in variants {
        if is_null_schema(variant) {
            continue;
        }
        values.extend(string_enum_values(variant)?);
    }
    (!values.is_empty()).then_some(values)
}

fn object_fields(
    schema: &serde_json::Value,
) -> Option<&serde_json::Map<String, serde_json::Value>> {
    let is_object = schema.get("type").and_then(serde_json::Value::as_str) == Some("object");
    if !is_object {
        return None;
    }
    schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .filter(|properties| !properties.is_empty())
}

fn schema_required(schema: &serde_json::Value, field_name: &str) -> bool {
    schema
        .get("required")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|required| {
            required
                .iter()
                .any(|value| value.as_str() == Some(field_name))
        })
}

fn object_map_value_schema(schema: &serde_json::Value) -> Option<&serde_json::Value> {
    if let Some(additional) = schema.get("additionalProperties") {
        if additional.as_bool() == Some(false) {
            return schema
                .get("patternProperties")
                .and_then(serde_json::Value::as_object)
                .and_then(single_map_schema_value);
        }
        return Some(additional);
    }

    schema
        .get("patternProperties")
        .and_then(serde_json::Value::as_object)
        .and_then(single_map_schema_value)
}

fn single_map_schema_value(
    schemas: &serde_json::Map<String, serde_json::Value>,
) -> Option<&serde_json::Value> {
    if schemas.len() == 1 {
        schemas.values().next()
    } else {
        None
    }
}

fn literal_base_type(schema: &serde_json::Value) -> Option<&'static str> {
    if let Some(value) = schema.get("const") {
        return match value {
            serde_json::Value::String(_) => Some("String"),
            serde_json::Value::Bool(_) => Some("bool"),
            serde_json::Value::Number(number) if number.is_i64() => Some("i64"),
            serde_json::Value::Number(_) => Some("f64"),
            _ => Some("Value"),
        };
    }

    let values = schema.get("enum")?.as_array()?;
    let first = values.first()?;
    match first {
        serde_json::Value::String(_) => Some("String"),
        serde_json::Value::Bool(_) => Some("bool"),
        serde_json::Value::Number(number) if number.is_i64() => Some("i64"),
        serde_json::Value::Number(_) => Some("f64"),
        _ => Some("Value"),
    }
}

fn union_base_type(schema: &serde_json::Value) -> Option<&'static str> {
    let variants = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))?
        .as_array()?;
    let mut ty = None;

    for variant in variants {
        if variant.get("type").and_then(serde_json::Value::as_str) == Some("null") {
            continue;
        }

        let variant_ty = literal_base_type(variant)?;
        if ty.is_some_and(|ty| ty != variant_ty) {
            return Some("Value");
        }
        ty = Some(variant_ty);
    }

    ty
}

fn tagged_object_union(
    schema: &serde_json::Value,
) -> Option<(String, Vec<(String, &serde_json::Value)>)> {
    let variants = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))?
        .as_array()?;
    let first = variants.first()?;
    let first_properties = object_fields(first)?;

    for (candidate, candidate_schema) in first_properties {
        let Some(first_value) = candidate_schema
            .get("const")
            .and_then(serde_json::Value::as_str)
        else {
            continue;
        };
        let mut tagged = vec![(first_value.to_string(), first)];
        for variant in variants.iter().skip(1) {
            let value = object_fields(variant)?
                .get(candidate)?
                .get("const")?
                .as_str()?;
            if tagged.iter().any(|(existing, _)| existing == value) {
                tagged.clear();
                break;
            }
            tagged.push((value.to_string(), variant));
        }
        if tagged.len() == variants.len() {
            return Some((candidate.clone(), tagged));
        }
    }
    None
}

fn object_union_variants(schema: &serde_json::Value) -> Option<&Vec<serde_json::Value>> {
    let variants = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))?
        .as_array()?;
    (!variants.is_empty()
        && variants
            .iter()
            .all(|variant| object_fields(variant).is_some()))
    .then_some(variants)
}

fn object_variant_name(schema: &serde_json::Value) -> Option<String> {
    let parts = object_fields(schema)?
        .values()
        .filter_map(|field| field.get("const").and_then(serde_json::Value::as_str))
        .map(rust_variant_ident)
        .collect::<String>();
    (!parts.is_empty()).then_some(parts)
}

fn join_string_literals(values: &[String]) -> String {
    values
        .iter()
        .map(|value| string_literal(value))
        .collect::<Vec<_>>()
        .join(", ")
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

fn string_literal(value: &str) -> String {
    serde_json::to_string(value).expect("string literal")
}

fn manifest_display_name(loaded: &trellis_contracts::LoadedApi) -> String {
    loaded.render_model.display_name.clone()
}

fn sdk_stem_pascal(loaded: &trellis_contracts::LoadedApi) -> String {
    sdk_stem_from_contract_id_pascal(&loaded.render_model.id)
}

fn sdk_stem_from_contract_id_pascal(contract_id: &str) -> String {
    default_sdk_stem(contract_id)
        .split('.')
        .flat_map(|segment| segment.split('-'))
        .map(key_to_pascal)
        .collect::<String>()
}

fn crate_ident(crate_name: &str) -> String {
    crate_name.replace('-', "_")
}

fn resolve_schema_ref<'a>(
    loaded: &'a trellis_contracts::LoadedApi,
    schema_name: &str,
) -> &'a serde_json::Value {
    loaded
        .render_model
        .schemas
        .get(schema_name)
        .unwrap_or_else(|| panic!("missing schema '{schema_name}' in manifest"))
}

fn schema_value_matches_types(
    loaded: &trellis_contracts::LoadedApi,
    schema_name: &str,
) -> serde_json::Value {
    let mut schema = resolve_schema_ref(loaded, schema_name).clone();
    inline_local_schema_refs(&mut schema, &loaded.render_model.schemas);
    schema
}

fn inline_local_schema_refs(
    value: &mut serde_json::Value,
    schemas: &std::collections::BTreeMap<String, serde_json::Value>,
) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(name) = object
                .get("$ref")
                .and_then(serde_json::Value::as_str)
                .and_then(|reference| reference.strip_prefix("#/schemas/"))
            {
                if let Some(schema) = schemas.get(name) {
                    *value = schema.clone();
                    inline_local_schema_refs(value, schemas);
                }
                return;
            }
            for nested in object.values_mut() {
                inline_local_schema_refs(nested, schemas);
            }
        }
        serde_json::Value::Array(values) => {
            for nested in values {
                inline_local_schema_refs(nested, schemas);
            }
        }
        _ => {}
    }
}

fn declared_error_payload_type(loaded: &trellis_contracts::LoadedApi, error_type: &str) -> String {
    if error_type == "AuthError" {
        return "trellis_rs::generated::AuthErrorPayload".to_string();
    }
    loaded
        .render_model
        .errors
        .values()
        .find(|error| error.error_type == error_type)
        .and_then(|error| error.schema.as_ref())
        .map(|schema| format!("crate::types::{}", key_to_pascal(&schema.schema)))
        .unwrap_or_else(|| "trellis_rs::generated::DeclaredErrorPayload".to_string())
}

fn rpc_call_error_type(loaded: &trellis_contracts::LoadedApi, key: &str, module: &str) -> String {
    if loaded.render_model.rpc[key]
        .errors
        .as_ref()
        .is_some_and(|errors| !errors.is_empty())
    {
        format!("{module}::{}Error", key_to_pascal(key))
    } else {
        "trellis_rs::generated::NoDeclaredError".to_string()
    }
}

fn is_empty_object_schema(schema: &serde_json::Value) -> bool {
    let Some(kind) = schema.get("type").and_then(serde_json::Value::as_str) else {
        return false;
    };
    if kind != "object" {
        return false;
    }

    let properties_empty = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .is_none_or(|properties| properties.is_empty());
    let required_empty = schema
        .get("required")
        .and_then(serde_json::Value::as_array)
        .is_none_or(|required| required.is_empty());

    properties_empty && required_empty
}

fn render_lib_rs(loaded: &trellis_contracts::LoadedApi) -> String {
    let client_name = format!("{}Client", sdk_stem_pascal(loaded));
    let operations_reexport = if loaded.render_model.operations.is_empty() {
        String::new()
    } else {
        "pub use operations::*;\n".to_string()
    };
    let events_reexport = if loaded.render_model.events.is_empty() {
        String::new()
    } else {
        "pub use events::*;\n".to_string()
    };
    let feeds_reexport = if loaded.render_model.feeds.is_empty() {
        String::new()
    } else {
        "pub use feeds::*;\n".to_string()
    };
    let feeds_module = if loaded.render_model.feeds.is_empty() {
        String::new()
    } else {
        "/// Feed descriptors.\npub mod feeds;\n".to_string()
    };
    format!(
        "//! Generated Rust SDK crate for one Trellis API.\n\nconst _: () = trellis_rs::generated::assert_abi(1);\n\n/// Typed outbound adapters.\npub mod client;\n/// Embedded API identity and artifact.\npub mod api;\n/// Event descriptors.\npub mod events;\n{feeds_module}/// Operation descriptors.\npub mod operations;\n/// RPC descriptors and declared errors.\npub mod rpc;\n/// JSON Schema constants.\npub mod schemas;\n/// Generated wire types.\npub mod types;\n\npub use client::{client_name};\npub use api::{{api_artifact, API_DIGEST, API_ID, API_JSON, API_NAME}};\n{events_reexport}{feeds_reexport}{operations_reexport}pub use rpc::*;\npub use types::*;\n"
    )
}

fn key_to_schema_constant_base(key: &str) -> String {
    key_to_snake(key).to_uppercase()
}

fn render_schemas_rs(loaded: &trellis_contracts::LoadedApi) -> String {
    use serde_json::Value;

    let mut lines = vec![
        format!(
            "//! JSON Schema constants for `{}`.",
            loaded.render_model.id
        ),
        String::new(),
    ];

    for (key, rpc) in &loaded.render_model.rpc {
        if rpc.internal == Some(true) {
            continue;
        }
        let base = key_to_schema_constant_base(key);
        let input_schema = schema_value_matches_types(loaded, &rpc.input.schema);
        let input_json = serde_json::to_string(&input_schema).expect("valid json");
        lines.push(format!(
            "pub const {}_INPUT_SCHEMA_JSON: &str = r#\"{}\"#;",
            base, input_json
        ));
        let output_schema = schema_value_matches_types(loaded, &rpc.output.schema);
        let output_json = serde_json::to_string(&output_schema).expect("valid json");
        lines.push(format!(
            "pub const {}_OUTPUT_SCHEMA_JSON: &str = r#\"{}\"#;",
            base, output_json
        ));
        lines.push(String::new());
    }

    for (key, operation) in &loaded.render_model.operations {
        let base = key_to_schema_constant_base(key);
        let input_schema = resolve_schema_ref(loaded, &operation.input.schema);
        let input_json = serde_json::to_string(input_schema).expect("valid json");
        lines.push(format!(
            "pub const {}_INPUT_SCHEMA_JSON: &str = r#\"{}\"#;",
            base, input_json
        ));
        if let Some(update) = &operation.update {
            let update_schema = resolve_schema_ref(loaded, &update.schema);
            let update_json = serde_json::to_string(update_schema).expect("valid json");
            lines.push(format!(
                "pub const {}_UPDATE_SCHEMA_JSON: &str = r#\"{}\"#;",
                base, update_json
            ));
        }
        match &operation.progress {
            Some(progress) => {
                let progress_schema = resolve_schema_ref(loaded, &progress.schema);
                let progress_json = serde_json::to_string(progress_schema).expect("valid json");
                lines.push(format!(
                    "pub const {}_PROGRESS_SCHEMA_JSON: &str = r#\"{}\"#;",
                    base, progress_json
                ));
            }
            None => {
                lines.push(format!(
                    "pub const {}_PROGRESS_SCHEMA_JSON: Option<&str> = None;",
                    base
                ));
            }
        }
        match &operation.output {
            Some(output) => {
                let output_schema = resolve_schema_ref(loaded, &output.schema);
                let output_json = serde_json::to_string(output_schema).expect("valid json");
                lines.push(format!(
                    "pub const {}_OUTPUT_SCHEMA_JSON: &str = r#\"{}\"#;",
                    base, output_json
                ));
            }
            None => {
                lines.push(format!(
                    "pub const {}_OUTPUT_SCHEMA_JSON: &str = r#\"{{}}\"#;",
                    base
                ));
            }
        }
        if !operation.signals.is_empty() {
            let mut signal_map = serde_json::Map::new();
            for (signal_name, signal) in &operation.signals {
                let signal_schema = resolve_schema_ref(loaded, &signal.input.schema);
                signal_map.insert(signal_name.clone(), signal_schema.clone());
            }
            let signal_json =
                serde_json::to_string(&Value::Object(signal_map)).expect("valid json");
            lines.push(format!(
                "pub const {}_SIGNAL_INPUT_SCHEMAS_JSON: &str = r#\"{}\"#;",
                base, signal_json
            ));
        } else {
            lines.push(format!(
                "pub const {}_SIGNAL_INPUT_SCHEMAS_JSON: &str = r#\"{{}}\"#;",
                base
            ));
        }
        lines.push(String::new());
    }

    for (key, event) in &loaded.render_model.events {
        let base = key_to_schema_constant_base(key);
        let event_json = serde_json::to_string(resolve_schema_ref(loaded, &event.event.schema))
            .expect("valid json");
        lines.push(format!(
            "pub const {}_EVENT_SCHEMA_JSON: &str = r#\"{}\"#;",
            base, event_json
        ));
        lines.push(String::new());
    }

    for (key, feed) in &loaded.render_model.feeds {
        let base = key_to_schema_constant_base(key);
        let input_schema = resolve_schema_ref(loaded, &feed.input.schema);
        let input_json = serde_json::to_string(input_schema).expect("valid json");
        lines.push(format!(
            "pub const {}_INPUT_SCHEMA_JSON: &str = r#\"{}\"#;",
            base, input_json
        ));
        let event_schema = resolve_schema_ref(loaded, &feed.event.schema);
        let event_json = serde_json::to_string(event_schema).expect("valid json");
        lines.push(format!(
            "pub const {}_EVENT_SCHEMA_JSON: &str = r#\"{}\"#;",
            base, event_json
        ));
        lines.push(String::new());
    }

    for schema_name in &loaded.render_model.exports.schemas {
        let base = key_to_schema_constant_base(schema_name);
        let schema = resolve_schema_ref(loaded, schema_name);
        let schema_json = serde_json::to_string(schema).expect("valid json");
        lines.push(format!(
            "pub const {}_SCHEMA_JSON: &str = r#\"{}\"#;",
            base, schema_json
        ));
        lines.push(String::new());
    }

    // Ensure at least one item exists so empty-contract modules compile.
    if lines.len() <= 2 {
        lines.push("pub const _SCHEMA_MODULE_LOADED: bool = true;".to_string());
    }

    let mut documented = Vec::with_capacity(lines.len() * 2);
    for line in lines {
        if let Some(name) = line
            .strip_prefix("pub const ")
            .and_then(|rest| rest.split_once(':').map(|(name, _)| name))
        {
            documented.push(format!("/// Generated JSON Schema constant `{name}`."));
        }
        documented.push(line);
    }
    format!("{}\n", documented.join("\n"))
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
        std::env::temp_dir().join(format!("trellis-codegen-rust-{label}-{nanos}"))
    }

    #[test]
    fn protocol_api_generation_uses_api_identity() {
        let out_dir = unique_temp_dir("protocol-api");
        fs::create_dir_all(&out_dir).unwrap();
        let api_path = write_sample_manifest(&out_dir);
        generate_rust_sdk(&GenerateRustSdkOpts {
            api_path,
            out_dir: out_dir.clone(),
            crate_name: "example-auth".to_owned(),
            crate_version: "0.1.0".to_owned(),
            runtime_deps: RustRuntimeDeps {
                source: RustRuntimeSource::Registry,
                version: "0.11.0".to_owned(),
                repo_root: None,
            },
        })
        .unwrap();

        assert!(out_dir.join("api.json").is_file());
        assert!(out_dir.join("src/api.rs").is_file());
        assert!(!out_dir.join("contract.json").exists());
        assert!(!out_dir.join("src/contract.rs").exists());
        let api = fs::read_to_string(out_dir.join("src/api.rs")).unwrap();
        assert!(api.contains("pub const API_ID"));
        fs::remove_dir_all(out_dir).unwrap();
    }

    fn write_sample_manifest(root: &Path) -> PathBuf {
        let manifest_path = root.join("trellis.core@v1.json");
        let manifest: serde_json::Value = serde_json::from_str(
            r#"{
                "format": "trellis.api.v1",
                "id": "trellis.core@v1",
                "version": "1.0.0",
                "displayName": "Trellis Core",
                "description": "Trellis core runtime surface.",
                "schemas": {
                    "InfoInput": {"type":"object","properties":{},"required":[]},
                    "InfoOutput": {"type":"object","properties":{"status":{"type":"string"}},"required":["status"]},
                    "ProcessInput": {"type":"object","properties":{"amount":{"type":"string"}},"required":["amount"]},
                    "ProcessProgress": {"type":"object","properties":{"step":{"type":"string"}},"required":["step"]},
                    "ProcessUpdate": {"type":"object","properties":{"message":{"type":"string"}},"required":["message"]},
                    "ProcessOutput": {"type":"object","properties":{"done":{"type":"boolean"}},"required":["done"]},
                    "AuthChangedEvent": {"type":"object","properties":{"status":{"type":"string"}},"required":["status"]},
                    "AuditFeedInput": {"type":"object","properties":{"since":{"type":"string"}},"required":["since"]},
                    "AuditFeedEvent": {"type":"object","properties":{"message":{"type":"string"}},"required":["message"]},
                    "ExternalCheckpoint": {"type":"object","properties":{"cursor":{"type":"string"}},"required":["cursor"]}
                },
                "capabilities": {
                    "auth.event.publish": {"allows":[{"action":"publish","target":{"kind":"apiSurface","api":"trellis.core@v1","surface":"event","name":"Auth.Changed"}}]},
                    "auth.event.subscribe": {"allows":[{"action":"subscribe","target":{"kind":"apiSurface","api":"trellis.core@v1","surface":"event","name":"Auth.Changed"}}]},
                    "audit.feed.subscribe": {"allows":[{"action":"subscribe","target":{"kind":"apiSurface","api":"trellis.core@v1","surface":"feed","name":"Audit.Feed"}}]}
                },
                "exports": {"schemas": ["ExternalCheckpoint"]},
                "rpc": {
                    "Trellis.Bindings.Get": {"version":"v1","input":{"schema":"ProcessInput"},"output":{"schema":"ProcessOutput"},"internal":true},
                    "Trellis.Info": {"version":"v1","input":{"schema":"InfoInput"},"output":{"schema":"InfoOutput"}}
                },
                "operations": {
                    "Trellis.Process": {"version":"v1","input":{"schema":"ProcessInput"},"progress":{"schema":"ProcessProgress"},"update":{"schema":"ProcessUpdate"},"output":{"schema":"ProcessOutput"},"transfer":{"direction":"send"},"cancel":true},
                    "Trellis.Audit": {"version":"v1","input":{"schema":"ProcessInput"},"progress":{"schema":"ProcessProgress"},"output":{"schema":"ProcessOutput"}}
                },
                "events": {
                    "Auth.Changed": {"version":"v1","event":{"schema":"AuthChangedEvent"}}
                },
                "feeds": {
                    "Audit.Feed": {"version":"v1","input":{"schema":"AuditFeedInput"},"event":{"schema":"AuditFeedEvent"}}
                }
            }"#,
        )
        .unwrap();

        let manifest = trellis_contracts::ApiBuilder::new(manifest)
            .build()
            .unwrap()
            .normalized_value()
            .unwrap();
        fs::write(
            &manifest_path,
            trellis_contracts::canonicalize_json(&manifest).unwrap(),
        )
        .unwrap();
        manifest_path
    }

    fn write_remote_manifest(root: &Path, file_name: &str, manifest: serde_json::Value) -> PathBuf {
        let manifest_path = root.join(file_name);
        let manifest = trellis_contracts::ApiBuilder::new(manifest)
            .build()
            .unwrap()
            .normalized_value()
            .unwrap();
        fs::write(
            &manifest_path,
            trellis_contracts::canonicalize_json(&manifest).unwrap(),
        )
        .unwrap();
        manifest_path
    }

    fn write_participant(root: &Path, file_name: &str, participant: serde_json::Value) -> PathBuf {
        let path = root.join(file_name);
        fs::write(
            &path,
            trellis_contracts::canonicalize_json(&participant).unwrap(),
        )
        .unwrap();
        trellis_contracts::load_participant_source(&path).unwrap();
        path
    }

    fn self_participant_value(api_path: &Path, kind: &str) -> serde_json::Value {
        let api = trellis_contracts::load_sdk_source(api_path).unwrap();
        let mut participant = json!({
            "format": "trellis.participant.v1",
            "id": api.render_model.id,
            "displayName": api.render_model.display_name,
            "description": api.render_model.description,
            "kind": kind,
            "implements": {
                "self": {"api": api.render_model.id, "apiDigest": api.digest}
            }
        });
        for field in ["schemas", "state"] {
            if let Some(value) = api.value.get(field) {
                participant[field] = value.clone();
            }
        }
        participant
    }

    fn write_self_participant(
        root: &Path,
        file_name: &str,
        api_path: &Path,
        kind: &str,
    ) -> PathBuf {
        write_participant(root, file_name, self_participant_value(api_path, kind))
    }

    fn add_required_use(
        participant: &mut serde_json::Value,
        alias: &str,
        api_path: &Path,
        selections: serde_json::Value,
    ) {
        let api = trellis_contracts::load_sdk_source(api_path).unwrap();
        let mut used = selections.as_object().cloned().unwrap();
        used.insert("api".to_string(), json!(api.render_model.id));
        used.insert("apiDigest".to_string(), json!(api.digest));
        participant["uses"]["required"][alias] = serde_json::Value::Object(used);
    }

    fn cargo_check(manifest_path: &Path) {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let output = std::process::Command::new(cargo)
            .arg("check")
            .arg("--manifest-path")
            .arg(manifest_path)
            .arg("--quiet")
            .env(
                "CARGO_TARGET_DIR",
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"),
            )
            .output()
            .expect("run cargo check");
        if !output.status.success() {
            panic!(
                "cargo check failed\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
    }

    #[test]
    fn cargo_toml_uses_registry_dependencies() {
        let cargo = render_cargo_toml(
            &GenerateRustSdkOpts {
                api_path: PathBuf::from("generated/protocol/apis/trellis.core@v1.json"),
                out_dir: PathBuf::from("generated/packages/cargo/trellis-core"),
                crate_name: "trellis-sdk-core".to_string(),
                crate_version: "0.1.0".to_string(),
                runtime_deps: RustRuntimeDeps {
                    source: RustRuntimeSource::Registry,
                    version: "0.1.0".to_string(),
                    repo_root: None,
                },
            },
            false,
            true,
        )
        .unwrap();

        assert!(cargo.contains("description = \"Generated Rust SDK crate for trellis-sdk-core.\""));
        assert!(cargo.contains("repository = \"https://github.com/qlever-llc/trellis\""));
        assert!(cargo.contains("trellis-rs = \"0.1.0\""));
        assert!(cargo.contains("trellis-contracts = \"0.1.0\""));
        assert!(cargo.contains("publish = false"));
        assert!(!cargo.contains("trellis-service"));
        assert!(!cargo.contains("path ="));
    }

    #[test]
    fn cargo_toml_uses_workspace_member_paths_for_local_runtime_deps() {
        let repo_root = unique_temp_dir("workspace-runtime-paths");
        fs::create_dir_all(repo_root.join("rust/crates/runtime-client")).unwrap();
        fs::create_dir_all(repo_root.join("rust/crates/runtime-contracts")).unwrap();
        fs::create_dir_all(repo_root.join("rust/crates/runtime-service")).unwrap();
        fs::create_dir_all(repo_root.join("rust/crates/sdk-generator")).unwrap();
        fs::write(
            repo_root.join("rust/Cargo.toml"),
            concat!(
                "[workspace]\n",
                "members = [\n",
                "  \"crates/runtime-client\",\n",
                "  \"crates/runtime-contracts\",\n",
                "  \"crates/runtime-service\",\n",
                "  \"crates/sdk-generator\",\n",
                "]\n",
            ),
        )
        .unwrap();
        fs::write(
            repo_root.join("rust/crates/runtime-client/Cargo.toml"),
            "[package]\nname = \"trellis-rs\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::write(
            repo_root.join("rust/crates/runtime-contracts/Cargo.toml"),
            "[package]\nname = \"trellis-contracts\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::write(
            repo_root.join("rust/crates/runtime-service/Cargo.toml"),
            "[package]\nname = \"trellis-service\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::write(
            repo_root.join("rust/crates/sdk-generator/Cargo.toml"),
            "[package]\nname = \"trellis-codegen-rust\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();

        let cargo = render_cargo_toml(
            &GenerateRustSdkOpts {
                api_path: PathBuf::from("generated/protocol/apis/trellis.core@v1.json"),
                out_dir: PathBuf::from("generated/packages/cargo/trellis-core"),
                crate_name: "trellis-sdk-core".to_string(),
                crate_version: "0.1.0".to_string(),
                runtime_deps: RustRuntimeDeps {
                    source: RustRuntimeSource::Local,
                    version: "0.1.0".to_string(),
                    repo_root: Some(repo_root.clone()),
                },
            },
            false,
            true,
        )
        .unwrap();

        assert!(cargo.contains(
            &repo_root
                .join("rust/crates/runtime-client")
                .display()
                .to_string()
        ));
        assert!(cargo.contains(
            &repo_root
                .join("rust/crates/runtime-contracts")
                .display()
                .to_string()
        ));
        assert!(!cargo.contains("trellis-service"));
        assert!(!cargo.contains("rust/crates/client"));
    }

    #[test]
    fn cargo_toml_integrity_check_accepts_generated_sdk_manifest() {
        let out_dir = unique_temp_dir("sdk-cargo-integrity-valid");
        fs::create_dir_all(&out_dir).unwrap();
        let manifest_path = write_sample_manifest(&out_dir);
        let sdk_out = out_dir.join("generated");

        generate_rust_sdk(&GenerateRustSdkOpts {
            api_path: manifest_path,
            out_dir: sdk_out.clone(),
            crate_name: "trellis-sdk-core".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps: RustRuntimeDeps {
                source: RustRuntimeSource::Registry,
                version: "0.1.0".to_string(),
                repo_root: None,
            },
        })
        .unwrap();

        assert!(rust_sdk_cargo_manifest_is_valid(
            &sdk_out.join("Cargo.toml"),
            "trellis-sdk-core",
            "0.1.0"
        ));
        let trellis_md = fs::read_to_string(sdk_out.join("TRELLIS.md")).unwrap();
        assert!(trellis_md.contains("# Trellis API Guide: trellis.core@v1"));
        assert!(trellis_md.contains("Trellis.Info"));
        assert!(trellis_md.contains(
            "https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms.txt"
        ));

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn cargo_toml_integrity_check_rejects_missing_required_dependencies() {
        let out_dir = unique_temp_dir("sdk-cargo-integrity-invalid");
        fs::create_dir_all(&out_dir).unwrap();
        let cargo_toml = out_dir.join("Cargo.toml");
        fs::write(
            &cargo_toml,
            concat!(
                "[package]\n",
                "name = \"trellis-sdk-core\"\n",
                "version = \"0.1.0\"\n",
                "edition = \"2021\"\n\n",
                "[dependencies]\n",
                "trellis = \"0.1.0\"\n",
            ),
        )
        .unwrap();

        assert!(!rust_sdk_cargo_manifest_is_valid(
            &cargo_toml,
            "trellis-sdk-core",
            "0.1.0"
        ));

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn generated_rust_source_validation_rejects_invalid_source() {
        let error = format_generated_rust_source("src/lib.rs", "pub fn broken(").unwrap_err();

        assert!(matches!(error, CodegenRustError::RustSyntax { path, .. } if path == "src/lib.rs"));
    }

    #[test]
    fn generated_rust_source_validation_formats_valid_source() {
        let formatted = format_generated_rust_source("src/lib.rs", "pub fn ok(){ }").unwrap();

        assert_eq!(formatted, "pub fn ok() {}\n");
    }

    #[test]
    fn invalid_generated_rust_is_rejected_before_write() {
        let out_dir = unique_temp_dir("invalid-rust-before-write");
        let target = out_dir.join("broken.rs");

        let error = write_rust_if_changed(&target, "pub fn broken(").unwrap_err();

        assert!(matches!(error, CodegenRustError::RustSyntax { .. }));
        assert!(!target.exists());

        if out_dir.exists() {
            fs::remove_dir_all(out_dir).unwrap();
        }
    }

    #[test]
    fn default_sdk_name_drops_duplicate_trellis_prefix() {
        assert_eq!(
            default_sdk_crate_name("trellis.core@v1"),
            "trellis-sdk-core"
        );
        assert_eq!(
            default_sdk_crate_name("trellis.auth@v1"),
            "trellis-sdk-auth"
        );
        assert_eq!(default_sdk_crate_name("graph@v1"), "trellis-sdk-graph");
    }

    #[test]
    fn key_to_snake_keeps_acronyms_together() {
        assert_eq!(key_to_snake("Jobs.ListDLQ"), "jobs_list_dlq");
        assert_eq!(key_to_snake("Jobs.ReplayDLQ"), "jobs_replay_dlq");
        assert_eq!(key_to_snake("HTTPServer"), "http_server");
    }

    #[test]
    fn generated_sdk_uses_contract_modules_shape() {
        let out_dir = unique_temp_dir("sdk-shape");
        fs::create_dir_all(&out_dir).unwrap();
        let manifest_path = write_sample_manifest(&out_dir);

        generate_rust_sdk(&GenerateRustSdkOpts {
            api_path: manifest_path,
            out_dir: out_dir.join("generated"),
            crate_name: "trellis-sdk-sample-service".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps: RustRuntimeDeps {
                source: RustRuntimeSource::Registry,
                version: "0.1.0".to_string(),
                repo_root: None,
            },
        })
        .unwrap();

        let lib_rs = fs::read_to_string(out_dir.join("generated/src/lib.rs")).unwrap();
        let api_rs = fs::read_to_string(out_dir.join("generated/src/api.rs")).unwrap();
        let types_rs = fs::read_to_string(out_dir.join("generated/src/types.rs")).unwrap();
        let rpc_rs = fs::read_to_string(out_dir.join("generated/src/rpc.rs")).unwrap();
        let operations_rs =
            fs::read_to_string(out_dir.join("generated/src/operations.rs")).unwrap();
        let events_rs = fs::read_to_string(out_dir.join("generated/src/events.rs")).unwrap();
        let feeds_rs = fs::read_to_string(out_dir.join("generated/src/feeds.rs")).unwrap();
        let client_rs = fs::read_to_string(out_dir.join("generated/src/client.rs")).unwrap();
        let cargo_toml = fs::read_to_string(out_dir.join("generated/Cargo.toml")).unwrap();

        assert!(lib_rs.contains("pub mod rpc;"));
        assert!(lib_rs.contains("pub mod operations;"));
        assert!(lib_rs.contains("pub mod events;"));
        assert!(lib_rs.contains("pub mod feeds;"));
        assert!(lib_rs.contains("pub use feeds::*;"));
        assert!(!lib_rs.contains("pub mod server;"));
        assert!(!lib_rs.contains("pub mod subjects;"));
        assert!(!out_dir.join("generated/src/server.rs").exists());
        assert!(cargo_toml.contains("publish = false"));
        assert!(!cargo_toml.contains("trellis-service"));
        assert!(api_rs.contains("//! Generated from"));
        assert!(api_rs.contains("pub const API_ID: &str = \"trellis.core@v1\";"));
        assert!(types_rs.contains("pub status: String,"));
        assert!(!types_rs.contains("TrellisBindingsGet"));
        assert!(types_rs.contains("pub struct TrellisProcessInput {"));
        assert!(types_rs.contains("pub struct TrellisProcessProgress {"));
        assert!(types_rs.contains("pub struct TrellisProcessOutput {"));
        assert!(types_rs.contains("pub struct AuthChangedEvent {"));
        assert!(types_rs.contains("pub struct AuditFeedInput {"));
        assert!(types_rs.contains("pub struct AuditFeedEvent {"));
        assert!(types_rs.contains("pub struct ExternalCheckpoint {"));
        assert!(types_rs.contains("pub status: String,"));
        assert!(rpc_rs.contains("pub struct TrellisInfoRpc;"));
        assert!(!rpc_rs.contains("TrellisBindingsGet"));
        assert!(rpc_rs.contains("type Input = Empty;"));
        assert!(operations_rs.contains("pub struct TrellisProcessOperation;"));
        assert!(operations_rs.contains("DeclaredOperationUpdates"));
        assert!(operations_rs.contains("NoOperationUpdates"));
        assert!(operations_rs.contains("TransferOperationDescriptor"));
        assert!(operations_rs.contains("impl OperationDescriptor for TrellisProcessOperation"));
        assert!(operations_rs.contains("type Update = crate::types::TrellisProcessUpdate;"));
        assert!(operations_rs.contains("type UpdateEvidence = DeclaredOperationUpdates;"));
        assert!(operations_rs.contains("TRELLIS_PROCESS_UPDATE_SCHEMA_JSON"));
        assert!(operations_rs.contains("type Update = serde_json::Value;"));
        assert!(operations_rs.contains("type UpdateEvidence = NoOperationUpdates;"));
        assert!(operations_rs.contains("const UPDATE_SCHEMA_JSON: Option<&'static str> = None;"));
        assert!(operations_rs
            .contains("impl TransferOperationDescriptor for TrellisProcessOperation {}"));
        assert!(operations_rs.contains("impl OperationDescriptor for TrellisAuditOperation"));
        assert!(!operations_rs
            .contains("impl TransferOperationDescriptor for TrellisAuditOperation {}"));
        assert!(!operations_rs.contains("ServerOperationDescriptor"));
        assert!(events_rs.contains("pub struct AuthChangedEventDescriptor;"));
        assert!(events_rs.contains(
            "const PUBLISH_CAPABILITIES: &'static [&'static str] = &[\"auth.event.publish\"];"
        ));
        assert!(events_rs.contains("\"auth.event.subscribe\""));
        assert!(feeds_rs.contains("\"audit.feed.subscribe\""));
        assert!(feeds_rs.contains("pub struct AuditFeedFeedDescriptor;"));
        assert!(feeds_rs.contains("impl FeedDescriptor for AuditFeedFeedDescriptor"));
        assert!(!feeds_rs.contains("ServerFeedDescriptor"));
        assert!(feeds_rs.contains("type Input = crate::types::AuditFeedInput;"));
        assert!(feeds_rs.contains("type Event = crate::types::AuditFeedEvent;"));
        assert!(feeds_rs.contains("const SUBJECT: &'static str ="));
        assert!(feeds_rs.contains("SUBSCRIBE_CAPABILITIES"));
        assert!(client_rs.contains("pub struct CoreClient<'a>"));
        assert!(!client_rs.contains("pub(crate) fn inner("));
        assert!(!client_rs.contains("#[allow(dead_code)]"));
        assert!(client_rs.contains("pub fn rpc(&self) -> Rpc<'a>"));
        assert!(client_rs.contains("pub fn trellis(&self) -> TrellisRpc<'a>"));
        assert!(client_rs.contains("pub async fn info("));
        assert!(!client_rs.contains("pub async fn bindings_get("));
        assert!(client_rs.contains("pub fn feed(&self) -> Feed<'a>"));
        assert!(client_rs.contains("pub fn audit(&self) -> AuditFeed<'a>"));
        assert!(client_rs.contains("pub async fn feed("));
        assert!(client_rs.contains("futures_util::stream::BoxStream"));
        assert!(client_rs.contains(".feed::<"));
        assert!(client_rs.contains("crate::feeds::AuditFeedFeedDescriptor"));
        assert!(client_rs.contains("pub fn operation(&self) -> Operation<'a>"));
        assert!(client_rs.contains("crate::operations::TrellisProcessOperation"));
        assert!(client_rs.contains(".subscribe::<"));
        assert!(!out_dir.join("generated/src/connect.rs").exists());
        assert!(!rpc_rs.contains("trellis_service::"));
        assert!(!operations_rs.contains("trellis_service::"));
        assert!(!events_rs.contains("trellis_service::"));
        assert!(!feeds_rs.contains("trellis_service::"));
        assert!(!client_rs.contains("trellis_service::"));

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn generated_event_only_sdk_omits_unused_imports() {
        let out_dir = unique_temp_dir("event-only-sdk-imports");
        fs::create_dir_all(&out_dir).unwrap();
        let manifest_path = write_remote_manifest(
            &out_dir,
            "example.events@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "example.events@v1",
                "version": "1.0.0",
                "displayName": "Example Events",
                "description": "Example event-only contract.",
                "schemas": {
                    "Updated": {
                        "type": "object",
                        "properties": { "service": { "type": "string" } },
                        "required": ["service"]
                    }
                },
                "events": {
                    "Example.Updated": {
                        "version": "v1",
                        "event": { "schema": "Updated" }
                    }
                }
            }),
        );

        generate_rust_sdk(&GenerateRustSdkOpts {
            api_path: manifest_path,
            out_dir: out_dir.join("generated"),
            crate_name: "trellis-sdk-health".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps: RustRuntimeDeps {
                source: RustRuntimeSource::Registry,
                version: "0.1.0".to_string(),
                repo_root: None,
            },
        })
        .unwrap();

        let rpc_rs = fs::read_to_string(out_dir.join("generated/src/rpc.rs")).unwrap();

        assert!(!rpc_rs.contains("trellis_rs::client::RpcDescriptor"));
        assert!(!rpc_rs.contains("trellis_service::RpcDescriptor"));
        assert!(!out_dir.join("generated/src/server.rs").exists());

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn generated_service_sdk_without_runtime_facade_omits_connect_surface() {
        let out_dir = unique_temp_dir("service-sdk-no-runtime-facade");
        fs::create_dir_all(&out_dir).unwrap();
        let manifest_path = write_sample_manifest(&out_dir);
        let sdk_out = out_dir.join("generated");
        fs::create_dir_all(sdk_out.join("src")).unwrap();
        fs::write(sdk_out.join("src/connect.rs"), "pub fn stale() {}\n").unwrap();

        generate_rust_sdk(&GenerateRustSdkOpts {
            api_path: manifest_path,
            out_dir: sdk_out.clone(),
            crate_name: "trellis-sdk-core".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps: RustRuntimeDeps {
                source: RustRuntimeSource::Registry,
                version: "0.1.0".to_string(),
                repo_root: None,
            },
        })
        .unwrap();

        let cargo_toml = fs::read_to_string(sdk_out.join("Cargo.toml")).unwrap();
        let lib_rs = fs::read_to_string(sdk_out.join("src/lib.rs")).unwrap();

        assert!(!cargo_toml.contains("trellis-service-runtime"));
        assert!(cargo_toml.contains("trellis-rs ="));
        assert!(!sdk_out.join("src/connect.rs").exists());
        assert!(!lib_rs.contains("pub mod connect"));
        assert!(!lib_rs.contains("pub use connect::"));
        assert!(!lib_rs.contains("trellis_service_runtime"));
        assert!(!lib_rs.contains("trellis_rs::service"));

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn generated_participant_facade_rejects_partial_alias_mappings() {
        let out_dir = unique_temp_dir("participant-partial-aliases");
        fs::create_dir_all(&out_dir).unwrap();

        let local_manifest = write_remote_manifest(
            &out_dir,
            "device@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "device@v1",
                "version": "1.0.0",
                "displayName": "Device",
                "description": "Device.",
            }),
        );
        let core_manifest = write_remote_manifest(
            &out_dir,
            "trellis.core@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "trellis.core@v1",
                "version": "1.0.0",
                "displayName": "Trellis Core",
                "description": "Core.",
                "schemas": {
                    "CatalogInput": {"type":"object","properties":{},"required":[]},
                    "CatalogOutput": {"type":"object","properties":{},"required":[]}
                },
                "rpc": {
                    "Core.Info": {
                        "version":"v1",
                        "input":{"schema":"CatalogInput"},
                        "output":{"schema":"CatalogOutput"}
                    }
                }
            }),
        );
        let mut participant = self_participant_value(&local_manifest, "device");
        add_required_use(
            &mut participant,
            "core",
            &core_manifest,
            json!({"rpc": {"call": ["Core.Info"]}}),
        );
        participant["uses"]["required"]["auth"] = json!({
            "api": "trellis.auth@v1",
            "apiDigest": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "rpc": {"call": ["Auth.Sessions.Me"]}
        });

        let error = generate_rust_participant_facade(&GenerateRustParticipantFacadeOpts {
            api_path: local_manifest.clone(),
            participant_path: write_participant(&out_dir, "device.participant.json", participant),
            out_dir: out_dir.join("facade"),
            crate_name: "device-participant".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps: RustRuntimeDeps {
                source: RustRuntimeSource::Registry,
                version: "0.1.0".to_string(),
                repo_root: None,
            },
            owned_sdk_crate_name: None,
            owned_sdk_path: None,
            alias_mappings: vec![ParticipantAliasMapping {
                alias: "core".to_string(),
                crate_name: "trellis-sdk-core".to_string(),
                api_path: core_manifest,
                crate_path: None,
                cargo_dependency: None,
            }],
        })
        .unwrap_err();

        assert!(matches!(
            error,
            CodegenRustError::MissingParticipantMappingAlias { alias, contract }
                if alias == "auth" && contract == "trellis.auth@v1"
        ));

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn generated_participant_facade_exposes_owned_and_used_aliases() {
        let out_dir = unique_temp_dir("participant-facade");
        fs::create_dir_all(&out_dir).unwrap();

        let local_manifest = write_remote_manifest(
            &out_dir,
            "audit@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "audit@v1",
                "version": "1.0.0",
                "displayName": "Audit",
                "description": "Audit service.",
                "schemas": {
                    "AuditListInput": {"type":"object","properties":{},"required":[]},
                    "AuditListOutput": {"type":"object","properties":{"items":{"type":"array","items":{"type":"string"}}},"required":["items"]},
                    "AuditFeedInput": {"type":"object","properties":{},"required":[]},
                    "AuditFeedEvent": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]}
                },
                "rpc": {
                    "Audit.List": {
                        "version": "v1",
                        "input": {"schema":"AuditListInput"},
                        "output": {"schema":"AuditListOutput"}
                    }
                },
                "feeds": {
                    "Audit.Feed": {
                        "version": "v1",
                        "input": {"schema":"AuditFeedInput"},
                        "event": {"schema":"AuditFeedEvent"}
                    }
                }
            }),
        );
        let core_manifest = write_remote_manifest(
            &out_dir,
            "trellis.core@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "trellis.core@v1",
                "version": "1.0.0",
                "displayName": "Trellis Core",
                "description": "Core.",
                "schemas": {
                    "CatalogInput": {"type":"object","properties":{},"required":[]},
                    "CatalogOutput": {"type":"object","properties":{},"required":[]},
                    "ContractGetInput": {"type":"object","properties":{"digest":{"type":"string"}},"required":["digest"]},
                    "ContractGetOutput": {"type":"object","properties":{},"required":[]}
                },
                "rpc": {
                    "Core.Info": {
                        "version":"v1",
                        "input":{"schema":"CatalogInput"},
                        "output":{"schema":"CatalogOutput"},
                        "transfer":{"direction":"receive"}
                    },
                    "Core.Api.Get": {
                        "version":"v1",
                        "input":{"schema":"ContractGetInput"},
                        "output":{"schema":"ContractGetOutput"}
                    }
                }
            }),
        );
        let auth_manifest = write_remote_manifest(
            &out_dir,
            "trellis.auth@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "trellis.auth@v1",
                "version": "1.0.0",
                "displayName": "Trellis Auth",
                "description": "Auth.",
                "schemas": {
                    "AuthSessionsMeInput": {"type":"object","properties":{},"required":[]},
                    "AuthSessionsMeOutput": {"type":"object","properties":{},"required":[]},
                    "AuthConnectionsOpenedEvent": {"type":"object","properties":{"user":{"type":"string"}},"required":["user"]}
                },
                "rpc": {
                    "Auth.Sessions.Me": {
                        "version":"v1",
                        "input":{"schema":"AuthSessionsMeInput"},
                        "output":{"schema":"AuthSessionsMeOutput"}
                    }
                },
                "events": {
                    "Auth.Connections.Opened": {
                        "version":"v1",
                        "event":{"schema":"AuthConnectionsOpenedEvent"}
                    }
                }
            }),
        );

        let owned_sdk_dir = out_dir.join("owned-sdk");
        let core_sdk_dir = out_dir.join("core-sdk");
        let auth_sdk_dir = out_dir.join("auth-sdk");
        let runtime_deps = RustRuntimeDeps {
            source: RustRuntimeSource::Local,
            version: "0.1.0".to_string(),
            repo_root: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")),
        };

        for (manifest_path, sdk_dir, crate_name) in [
            (&local_manifest, &owned_sdk_dir, "audit-sdk"),
            (&core_manifest, &core_sdk_dir, "trellis-sdk-core"),
            (&auth_manifest, &auth_sdk_dir, "trellis-sdk-auth"),
        ] {
            generate_rust_sdk(&GenerateRustSdkOpts {
                api_path: manifest_path.clone(),
                out_dir: sdk_dir.clone(),
                crate_name: crate_name.to_string(),
                crate_version: "0.1.0".to_string(),
                runtime_deps: runtime_deps.clone(),
            })
            .unwrap();
        }
        let mut participant = self_participant_value(&local_manifest, "service");
        add_required_use(
            &mut participant,
            "core",
            &core_manifest,
            json!({"rpc": {"call": ["Core.Info"]}}),
        );
        add_required_use(
            &mut participant,
            "auth",
            &auth_manifest,
            json!({
                "rpc": {"call": ["Auth.Sessions.Me"]},
                "events": {
                    "publish": ["Auth.Connections.Opened"],
                    "subscribe": ["Auth.Connections.Opened"]
                }
            }),
        );

        generate_rust_participant_facade(&GenerateRustParticipantFacadeOpts {
            api_path: local_manifest.clone(),
            participant_path: write_participant(&out_dir, "audit.participant.json", participant),
            out_dir: out_dir.join("facade"),
            crate_name: "audit-participant".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps,
            owned_sdk_crate_name: Some("audit-sdk".to_string()),
            owned_sdk_path: Some(owned_sdk_dir),
            alias_mappings: vec![
                ParticipantAliasMapping {
                    alias: "core".to_string(),
                    crate_name: "trellis-sdk-core".to_string(),
                    api_path: core_manifest,
                    crate_path: Some(core_sdk_dir),
                    cargo_dependency: None,
                },
                ParticipantAliasMapping {
                    alias: "auth".to_string(),
                    crate_name: "trellis-sdk-auth".to_string(),
                    api_path: auth_manifest,
                    crate_path: Some(auth_sdk_dir),
                    cargo_dependency: None,
                },
            ],
        })
        .unwrap();

        let cargo_toml = fs::read_to_string(out_dir.join("facade/Cargo.toml")).unwrap();
        let trellis_md = fs::read_to_string(out_dir.join("facade/TRELLIS.md")).unwrap();
        let lib_rs = fs::read_to_string(out_dir.join("facade/src/lib.rs")).unwrap();
        let connect_rs = fs::read_to_string(out_dir.join("facade/src/connect.rs")).unwrap();
        let contract_rs = fs::read_to_string(out_dir.join("facade/src/contract.rs")).unwrap();
        let owned_rs = fs::read_to_string(out_dir.join("facade/src/owned.rs")).unwrap();

        assert!(!cargo_toml.contains("build = \"build.rs\""));
        assert!(!cargo_toml.contains("\ntrellis-codegen-rust ="));
        assert!(cargo_toml.contains("serde = { version = \"1.0\", features = [\"derive\"] }"));
        assert!(cargo_toml.contains("trellis-rs = { path = "));
        assert!(cargo_toml.contains("trellis-contracts = { path = "));
        assert!(!cargo_toml.contains("trellis-service"));
        assert!(cargo_toml.contains("futures-util = \"0.3\""));
        assert!(cargo_toml.contains("tokio = { version = \"1\", features = [\"io-util\"] }"));
        assert!(!out_dir.join("facade/build.rs").exists());
        assert!(trellis_md.contains("# Trellis Participant Guide: audit@v1"));
        assert!(trellis_md
            .contains("alias `auth` -> crate `trellis-sdk-auth` contract `trellis.auth@v1`"));
        assert!(trellis_md.contains("Event publish `Auth.Connections.Opened`"));
        assert!(lib_rs.contains("include!(\"facade.rs\");"));
        assert!(lib_rs.contains("connect"));
        assert!(!lib_rs.contains("ConnectedClient"));
        assert!(lib_rs.contains("ConnectedService"));
        assert!(lib_rs.contains("ServiceConnectOptions"));
        assert!(lib_rs.contains("ServiceRuntimeError"));
        assert!(connect_rs.contains("pub struct Contract"));
        assert!(
            connect_rs.contains("impl trellis_rs::service::GeneratedServiceContract for Contract")
        );
        assert!(connect_rs
            .contains("const PARTICIPANT_ID: &'static str = crate::contract::CONTRACT_ID",));
        assert!(connect_rs.contains("pub struct ConnectedService"));
        assert!(connect_rs.contains("trellis_rs::service::ConnectedServiceRuntime<Contract>"));
        assert!(connect_rs.contains("pub fn generated_handle(&self)"));
        assert!(!connect_rs.contains("pub(crate) fn runtime(&self)"));
        assert!(!connect_rs
            .contains("pub fn raw(&self) -> &trellis_rs::service::ConnectedServiceRuntime"));
        assert!(!connect_rs.contains("pub fn raw_mut(&mut self)"));
        assert!(
            !connect_rs.contains("pub fn new(inner: trellis_rs::service::ConnectedServiceRuntime")
        );
        assert!(connect_rs.contains("pub use trellis_rs::service::ServiceConnectOptions"));
        assert!(connect_rs.contains("contract evidence comes from [`Contract`]"));
        assert!(!connect_rs.contains("opts.participant_"));
        assert!(!connect_rs.contains("opts.api_"));
        assert!(!connect_rs.contains("opts.referenced_api_artifacts"));
        assert!(connect_rs.contains("pub async fn connect("));
        assert!(connect_rs.contains("opts: ServiceConnectOptions<'_>"));
        assert!(connect_rs
            .contains("Result<ConnectedService, trellis_rs::service::ServiceRuntimeError>"));
        assert!(!connect_rs.contains("connect_user"));
        assert!(contract_rs.contains("pub const CONTRACT_DIGEST: &str = "));
        assert!(contract_rs.contains("pub const PARTICIPANT: &str = "));
        assert!(contract_rs.contains("pub const API_JSON: &str = "));
        assert!(owned_rs.contains("impl crate::ConnectedService"));
        assert!(owned_rs.contains("pub fn handle(&mut self) -> ServiceHandle<'_>"));
        assert!(owned_rs.contains("pub fn rpc(&mut self) -> ProviderRpc<'_>"));
        assert!(owned_rs.contains("pub fn audit(&mut self) -> AuditProviderRpc<'_>"));
        assert!(owned_rs.contains("pub fn list<F, Fut>(&mut self, handler: F)"));
        assert!(owned_rs.contains(".register_rpc::<sdk::rpc::AuditListRpc"));
        assert!(owned_rs.contains("runtime_mut()"));
        assert!(!owned_rs.contains("pub fn register_audit_list"));
        assert!(owned_rs.contains("pub fn feed<F, S>(&mut self, handler: F)"));
        assert!(owned_rs.contains(".register_feed::<sdk::feeds::AuditFeedFeedDescriptor"));
        assert!(!owned_rs.contains("pub fn register_audit_feed"));
        assert!(out_dir.join("facade/apis/core.json").exists());
        assert!(out_dir.join("facade/apis/auth.json").exists());
        fs::write(
            out_dir.join("facade/src/stream_compile.rs"),
            r#"#[allow(dead_code)]
async fn download_through_alias<W>(
    client: &crate::uses::core::Client<'_>,
    grant: &trellis_rs::generated::DownloadTransferGrant,
    writer: &mut W,
    cancellation: &trellis_rs::generated::TransferCancellation,
) -> Result<trellis_rs::generated::FileInfo, trellis_rs::generated::TrellisClientError>
where
    W: tokio::io::AsyncWrite + Unpin + Send + ?Sized,
{
    client.download_transfer_into(grant, writer).await?;
    client
        .download_transfer_into_with_cancel(grant, writer, cancellation)
        .await
}
"#,
        )
        .unwrap();
        fs::write(
            out_dir.join("facade/src/lib.rs"),
            format!("{}\nmod stream_compile;\n", lib_rs),
        )
        .unwrap();
        cargo_check(&out_dir.join("facade/Cargo.toml"));

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn generated_participant_facade_exposes_operation_only_service_registrations() {
        let out_dir = unique_temp_dir("participant-operation-only");
        fs::create_dir_all(&out_dir).unwrap();

        let local_manifest = write_remote_manifest(
            &out_dir,
            "ops@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "ops@v1",
                "version": "1.0.0",
                "displayName": "Ops",
                "description": "Operation-only service.",
                "schemas": {
                    "OpInput": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                    "OpOutput": {"type":"object","properties":{"ok":{"type":"boolean"}},"required":["ok"]}
                },
                "operations": {
                    "Op.Run": {
                        "version": "v1",
                        "input": {"schema":"OpInput"},
                        "output": {"schema":"OpOutput"}
                    }
                }
            }),
        );
        let owned_sdk_dir = out_dir.join("owned-sdk");
        fs::create_dir_all(&owned_sdk_dir).unwrap();

        generate_rust_participant_generated_sources(&GenerateRustParticipantFacadeOpts {
            api_path: local_manifest.clone(),
            participant_path: write_self_participant(
                &out_dir,
                "ops.participant.json",
                &local_manifest,
                "service",
            ),
            out_dir: out_dir.join("generated"),
            crate_name: "ops-participant".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps: RustRuntimeDeps {
                source: RustRuntimeSource::Registry,
                version: "0.1.0".to_string(),
                repo_root: None,
            },
            owned_sdk_crate_name: Some("ops-sdk".to_string()),
            owned_sdk_path: Some(owned_sdk_dir),
            alias_mappings: vec![],
        })
        .unwrap();

        let owned_rs = fs::read_to_string(out_dir.join("generated/src/owned.rs")).unwrap();
        assert!(owned_rs.contains("impl crate::ConnectedService"));
        assert!(owned_rs.contains("pub fn run<P>(&mut self, provider: P)"));
        assert!(!owned_rs.contains("pub fn register_op_run_provider"));
        assert!(owned_rs.contains("sdk::operations::OpRunOperation"));

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn generated_participant_facade_compiles_with_service_runtime() {
        let out_dir = unique_temp_dir("participant-compile");
        fs::create_dir_all(&out_dir).unwrap();

        let local_manifest = write_remote_manifest(
            &out_dir,
            "compile@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "compile@v1",
                "version": "1.0.0",
                "displayName": "Compile",
                "description": "Compile-test service.",
                "schemas": {
                    "PingInput": {"type":"object","properties":{"value":{"type":"string"}},"required":["value"]},
                    "PingOutput": {"type":"object","properties":{"value":{"type":"string"}},"required":["value"]},
                    "FeedInput": {"type":"object","properties":{},"required":[]},
                    "FeedEvent": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                    "WorkPayload": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                    "WorkResult": {"type":"object","properties":{"done":{"type":"boolean"}},"required":["done"]},
                    "ChangedEvent": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                    "ProcessInput": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                    "ProcessProgress": {"type":"object","properties":{"step":{"type":"string"}},"required":["step"]},
                    "ProcessOutput": {"type":"object","properties":{"ok":{"type":"boolean"}},"required":["ok"]},
                    "ProcessErrorData": {"type":"object","properties":{"type":{"const":"ProcessError"},"message":{"type":"string"},"id":{"type":"string"}},"required":["type","message","id"]},
                    "StateValue": {"type":"object","properties":{"requiredNullable":{"anyOf":[{"type":"string"},{"type":"null"}]},"optionalNullable":{"anyOf":[{"type":"string"},{"type":"null"}]}},"required":["requiredNullable"]},
                    "OpenRecord": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                    "Proof": {"type":"object","properties":{"token":{"type":"string"}},"required":["token"],"additionalProperties":false}
                },
                "errors": {
                    "ProcessError": {"schema":{"schema":"ProcessErrorData"}}
                },
                "rpc": {
                    "Compile.Ping": {
                        "version":"v1",
                        "input":{"schema":"PingInput"},
                        "output":{"schema":"PingOutput"},
                        "errors":["ProcessError"]
                    }
                },
                "feeds": {
                    "Compile.Feed": {
                        "version":"v1",
                        "input":{"schema":"FeedInput"},
                        "event":{"schema":"FeedEvent"}
                    }
                },
                "operations": {
                    "Compile.Process": {
                        "version":"v1",
                        "input":{"schema":"ProcessInput"},
                        "progress":{"schema":"ProcessProgress"},
                        "output":{"schema":"ProcessOutput"},
                        "errors":["ProcessError"],
                        "cancel":true
                    }
                },
                "state": {
                    "current": {"kind":"value","schema":{"schema":"StateValue"}},
                    "records": {"kind":"map","schema":{"schema":"OpenRecord"}}
                },
                "events": {
                    "Compile.Changed": {
                        "version": "v1",
                        "event": {"schema": "ChangedEvent"}
                    }
                }
            }),
        );
        let owned_sdk_dir = out_dir.join("owned-sdk");

        let runtime_deps = RustRuntimeDeps {
            source: RustRuntimeSource::Local,
            version: "0.1.0".to_string(),
            repo_root: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")),
        };

        generate_rust_sdk(&GenerateRustSdkOpts {
            api_path: local_manifest.clone(),
            out_dir: owned_sdk_dir.clone(),
            crate_name: "compile-sdk".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps: runtime_deps.clone(),
        })
        .unwrap();
        generate_rust_participant_facade(&GenerateRustParticipantFacadeOpts {
            api_path: local_manifest.clone(),
            participant_path: write_self_participant(
                &out_dir,
                "compile.participant.json",
                &local_manifest,
                "service",
            ),
            out_dir: out_dir.join("facade"),
            crate_name: "compile-participant".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps,
            owned_sdk_crate_name: Some("compile-sdk".to_string()),
            owned_sdk_path: Some(owned_sdk_dir),
            alias_mappings: vec![],
        })
        .unwrap();

        cargo_check(&out_dir.join("facade/Cargo.toml"));

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn generated_caller_facades_compile_with_kind_specific_connections() {
        for kind in ["app", "agent", "device"] {
            let out_dir = unique_temp_dir(&format!("participant-{kind}-compile"));
            fs::create_dir_all(&out_dir).unwrap();
            let manifest = write_remote_manifest(
                &out_dir,
                &format!("{kind}@v1.json"),
                json!({
                    "format": "trellis.api.v1",
                    "id": format!("fixture.{kind}@v1"),
                    "version": "1.0.0",
                    "displayName": format!("Fixture {kind}"),
                    "description": "Compile fixture.",
                    "schemas": {
                        "ChangedEvent": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]}
                    },
                    "events": {
                        "Fixture.Changed": {
                            "version":"v1",
                            "event":{"schema":"ChangedEvent"}
                        }
                    }
                }),
            );
            let facade = out_dir.join("facade");
            let owned_sdk = out_dir.join("owned-sdk");
            let runtime_deps = RustRuntimeDeps {
                source: RustRuntimeSource::Local,
                version: "0.1.0".to_string(),
                repo_root: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")),
            };
            generate_rust_sdk(&GenerateRustSdkOpts {
                api_path: manifest.clone(),
                out_dir: owned_sdk.clone(),
                crate_name: format!("fixture-{kind}-sdk"),
                crate_version: "0.1.0".to_string(),
                runtime_deps: runtime_deps.clone(),
            })
            .unwrap();
            generate_rust_participant_facade(&GenerateRustParticipantFacadeOpts {
                api_path: manifest.clone(),
                participant_path: write_self_participant(
                    &out_dir,
                    &format!("{kind}.participant.json"),
                    &manifest,
                    kind,
                ),
                out_dir: facade.clone(),
                crate_name: format!("fixture-{kind}-participant"),
                crate_version: "0.1.0".to_string(),
                runtime_deps,
                owned_sdk_crate_name: Some(format!("fixture-{kind}-sdk")),
                owned_sdk_path: Some(owned_sdk),
                alias_mappings: vec![],
            })
            .unwrap();

            let connect = fs::read_to_string(facade.join("src/connect.rs")).unwrap();
            assert!(connect.contains("pub async fn connect("));
            assert!(!connect.contains("connect_service"));
            if kind == "device" {
                let lib = fs::read_to_string(facade.join("src/lib.rs")).unwrap();
                assert!(lib.contains("ConnectedClient, Contract"));
                assert!(connect.contains(
                    "/// Exact generated participant and API evidence used for device bootstrap."
                ));
                assert!(connect
                    .contains("impl trellis_rs::service::GeneratedServiceContract for Contract"));
                assert!(connect.contains("DeviceConnectOptions::<Contract>::new"));
                assert!(connect.contains("DeviceActivationOptions<'_, Contract>"));
                assert!(connect.contains("options.into_connect_options(session)?"));
                assert!(!connect.contains("opts.participant_needs_digest"));
                assert!(!connect.contains("session_key_seed_base64url"));
            }

            cargo_check(&facade.join("Cargo.toml"));
        }
    }

    #[test]
    fn generated_participant_facade_exposes_typed_state_helpers() {
        let out_dir = unique_temp_dir("participant-state");
        fs::create_dir_all(&out_dir).unwrap();

        let local_manifest = write_remote_manifest(
            &out_dir,
            "device@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "device@v1",
                "version": "1.0.0",
                "displayName": "Device",
                "description": "Device.",
                "schemas": {
                    "SelectedSite": {
                        "type": "object",
                        "properties": { "siteId": { "type": "string" } },
                        "required": ["siteId"]
                    },
                    "DraftInspection": {
                        "type": "object",
                        "properties": { "title": { "type": "string" } },
                        "required": ["title"]
                    },
                    "State": {
                        "type": "object",
                        "properties": { "flag": { "type": "boolean" } },
                        "required": ["flag"]
                    },
                    "StateValue": {
                        "type": "object",
                        "properties": { "name": { "type": "string" } },
                        "required": ["name"]
                    },
                    "Foo": {
                        "type": "object",
                        "properties": { "one": { "type": "string" } },
                        "required": ["one"]
                    },
                    "FooState": {
                        "type": "object",
                        "properties": { "two": { "type": "string" } },
                        "required": ["two"]
                    }
                },
                "state": {
                    "selectedSite": {
                        "kind": "value",
                        "schema": { "schema": "SelectedSite" }
                    },
                    "draftInspections": {
                        "kind": "map",
                        "schema": { "schema": "DraftInspection" }
                    },
                    "currentState": {
                        "kind": "value",
                        "schema": { "schema": "State" }
                    },
                    "stateValue": {
                        "kind": "value",
                        "schema": { "schema": "StateValue" }
                    },
                    "foo": {
                        "kind": "value",
                        "schema": { "schema": "Foo" }
                    },
                    "fooState": {
                        "kind": "value",
                        "schema": { "schema": "FooState" }
                    }
                }
            }),
        );

        generate_rust_participant_generated_sources(&GenerateRustParticipantFacadeOpts {
            api_path: local_manifest.clone(),
            participant_path: write_self_participant(
                &out_dir,
                "state.participant.json",
                &local_manifest,
                "device",
            ),
            out_dir: out_dir.join("generated"),
            crate_name: "device-participant".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps: RustRuntimeDeps {
                source: RustRuntimeSource::Registry,
                version: "0.1.0".to_string(),
                repo_root: None,
            },
            owned_sdk_crate_name: None,
            owned_sdk_path: None,
            alias_mappings: vec![],
        })
        .unwrap();

        let facade_rs = fs::read_to_string(out_dir.join("generated/src/facade.rs")).unwrap();
        let state_rs = fs::read_to_string(out_dir.join("generated/src/state.rs")).unwrap();

        assert!(facade_rs.contains("pub mod state"));
        assert!(facade_rs.contains("pub fn state(&self) -> state::State<'a>"));
        assert!(state_rs.contains("pub struct SelectedSiteState {"));
        assert!(state_rs.contains("pub site_id: String,"));
        assert!(state_rs.contains("pub struct DraftInspectionState {"));
        assert!(state_rs.contains("pub struct StateValue {"));
        assert!(state_rs.contains("pub struct StateValueState {"));
        assert!(state_rs.contains("pub struct FooState {"));
        assert!(state_rs.contains("pub struct FooState2 {"));
        assert!(state_rs.contains("pub fn selected_site("));
        assert!(state_rs.contains("trellis_rs::generated::ValueStateStore<"));
        assert!(state_rs.contains("SelectedSiteState"));
        assert!(state_rs
            .contains("trellis_rs::generated::ValueStateStore::new(self.inner, \"selectedSite\")"));
        assert!(state_rs.contains("pub fn draft_inspections("));
        assert!(state_rs.contains("trellis_rs::generated::MapStateStore<"));
        assert!(state_rs.contains("DraftInspectionState"));
        assert!(state_rs.contains(
            "trellis_rs::generated::MapStateStore::new(self.inner, \"draftInspections\")"
        ));
        assert!(state_rs.contains("pub fn current_state("));
        assert!(state_rs.contains("StateValue"));
        assert!(state_rs.contains("pub fn state_value("));
        assert!(state_rs.contains("StateValueState"));
        assert!(state_rs.contains("pub fn foo("));
        assert!(state_rs.contains("FooState"));
        assert!(state_rs.contains("pub fn foo_state("));
        assert!(state_rs.contains("FooState2"));

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn generated_participant_alias_forwards_selected_operation_calls() {
        let out_dir = unique_temp_dir("participant-operation-alias");
        fs::create_dir_all(&out_dir).unwrap();

        let local_manifest = write_remote_manifest(
            &out_dir,
            "participant@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "participant@v1",
                "version": "1.0.0",
                "displayName": "Participant",
                "description": "Participant.",
                "schemas": {}
            }),
        );
        let evidence_manifest = write_remote_manifest(
            &out_dir,
            "evidence@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "evidence@v1",
                "version": "1.0.0",
                "displayName": "Evidence",
                "description": "Evidence.",
                "schemas": {
                    "UploadInput": {"type":"object","properties":{"path":{"type":"string"}},"required":["path"]},
                    "UploadProgress": {"type":"object","properties":{"bytes":{"type":"number"}},"required":["bytes"]},
                    "UploadOutput": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                    "DeleteInput": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                    "DeleteProgress": {"type":"object","properties":{},"required":[]},
                    "DeleteOutput": {"type":"object","properties":{},"required":[]},
                    "EvidenceUploadedEvent": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                    "EvidenceStreamInput": {"type":"object","properties":{},"required":[]},
                    "EvidenceStreamEvent": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]}
                },
                "operations": {
                    "Evidence.Upload": {
                        "version":"v1",
                        "input":{"schema":"UploadInput"},
                        "progress":{"schema":"UploadProgress"},
                        "output":{"schema":"UploadOutput"}
                    },
                    "Evidence.Delete": {
                        "version":"v1",
                        "input":{"schema":"DeleteInput"},
                        "progress":{"schema":"DeleteProgress"},
                        "output":{"schema":"DeleteOutput"}
                    }
                },
                "events": {
                    "Evidence.Uploaded": {
                        "version":"v1",
                        "event":{"schema":"EvidenceUploadedEvent"}
                    }
                },
                "feeds": {
                    "Evidence.Stream": {
                        "version":"v1",
                        "input":{"schema":"EvidenceStreamInput"},
                        "event":{"schema":"EvidenceStreamEvent"}
                    }
                }
            }),
        );
        let mut participant = self_participant_value(&local_manifest, "app");
        add_required_use(
            &mut participant,
            "evidence",
            &evidence_manifest,
            json!({
                "operations": {"invoke": ["Evidence.Upload"]},
                "events": {"subscribe": ["Evidence.Uploaded"]},
                "feeds": {"subscribe": ["Evidence.Stream"]}
            }),
        );

        generate_rust_participant_generated_sources(&GenerateRustParticipantFacadeOpts {
            api_path: local_manifest.clone(),
            participant_path: write_participant(&out_dir, "uses.participant.json", participant),
            out_dir: out_dir.join("generated"),
            crate_name: "participant".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps: RustRuntimeDeps {
                source: RustRuntimeSource::Registry,
                version: "0.1.0".to_string(),
                repo_root: None,
            },
            owned_sdk_crate_name: None,
            owned_sdk_path: None,
            alias_mappings: vec![ParticipantAliasMapping {
                alias: "evidence".to_string(),
                crate_name: "evidence-sdk".to_string(),
                api_path: evidence_manifest,
                crate_path: None,
                cargo_dependency: None,
            }],
        })
        .unwrap();

        let evidence_rs =
            fs::read_to_string(out_dir.join("generated/src/uses/evidence.rs")).unwrap();
        assert!(evidence_rs.contains("EvidenceUploadOperation"));
        assert!(evidence_rs.contains("trellis_rs::generated::OperationInvoker<"));
        assert!(evidence_rs.contains("sdk::operations::EvidenceUploadOperation"));
        assert!(evidence_rs.contains("self.transport"));
        assert!(evidence_rs.contains(".operation::<sdk::operations::EvidenceUploadOperation>()"));
        assert!(evidence_rs.contains("pub async fn subscribe_evidence_uploaded("));
        assert!(evidence_rs.contains(".subscribe::<"));
        assert!(evidence_rs.contains("sdk::events::EvidenceUploadedEventDescriptor"));
        assert!(evidence_rs.contains("pub async fn evidence_stream("));
        assert!(evidence_rs.contains(".feed::<sdk::feeds::EvidenceStreamFeedDescriptor>"));
        assert!(evidence_rs.contains("&sdk::rpc::Empty {}"));
        assert!(!evidence_rs.contains("evidence_delete"));

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn generated_participant_facade_rejects_missing_mapped_feed() {
        let out_dir = unique_temp_dir("participant-missing-feed");
        fs::create_dir_all(&out_dir).unwrap();

        let local_manifest = write_remote_manifest(
            &out_dir,
            "participant@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "participant@v1",
                "version": "1.0.0",
                "displayName": "Participant",
                "description": "Participant.",
            }),
        );
        let evidence_manifest = write_remote_manifest(
            &out_dir,
            "evidence@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "evidence@v1",
                "version": "1.0.0",
                "displayName": "Evidence",
                "description": "Evidence.",
                "feeds": {}
            }),
        );
        let mut participant = self_participant_value(&local_manifest, "service");
        add_required_use(
            &mut participant,
            "evidence",
            &evidence_manifest,
            json!({"feeds": {"subscribe": ["Evidence.Stream"]}}),
        );

        let error =
            generate_rust_participant_generated_sources(&GenerateRustParticipantFacadeOpts {
                api_path: local_manifest.clone(),
                participant_path: write_participant(
                    &out_dir,
                    "missing-feed.participant.json",
                    participant,
                ),
                out_dir: out_dir.join("generated"),
                crate_name: "participant".to_string(),
                crate_version: "0.1.0".to_string(),
                runtime_deps: RustRuntimeDeps {
                    source: RustRuntimeSource::Registry,
                    version: "0.1.0".to_string(),
                    repo_root: None,
                },
                owned_sdk_crate_name: None,
                owned_sdk_path: None,
                alias_mappings: vec![ParticipantAliasMapping {
                    alias: "evidence".to_string(),
                    crate_name: "evidence-sdk".to_string(),
                    api_path: evidence_manifest,
                    crate_path: None,
                    cargo_dependency: None,
                }],
            })
            .unwrap_err();

        assert!(matches!(
            error,
            CodegenRustError::MissingMappedFeed { alias, key }
                if alias == "evidence" && key == "Evidence.Stream"
        ));

        fs::remove_dir_all(out_dir).unwrap();
    }

    #[test]
    fn generated_operation_descriptor_includes_error_types() {
        let out_dir = unique_temp_dir("operation-descriptor-errors");
        fs::create_dir_all(&out_dir).unwrap();

        let manifest_path = write_remote_manifest(
            &out_dir,
            "ops@v1.json",
            json!({
                "format": "trellis.api.v1",
                "id": "ops@v1",
                "version": "1.0.0",
                "displayName": "Ops With Errors",
                "description": "Operation with declared errors.",
                "schemas": {
                    "Input": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                    "Progress": {"type":"object","properties":{"step":{"type":"string"}},"required":["step"]},
                    "Output": {"type":"object","properties":{"ok":{"type":"boolean"}},"required":["ok"]},
                    "NotFoundData": {
                        "type":"object",
                        "properties":{
                            "type":{"const":"NotFoundError"},
                            "message":{"type":"string"},
                            "id":{"type":"string"}
                        },
                        "required":["type","message","id"]
                    }
                },
                "errors": {
                    "NotFoundError": {
                        "schema": { "schema": "NotFoundData" }
                    }
                },
                "operations": {
                    "Example.Process": {
                        "version": "v1",
                        "input": { "schema": "Input" },
                        "progress": { "schema": "Progress" },
                        "output": { "schema": "Output" },
                        "errors": ["NotFoundError"]
                    }
                }
            }),
        );

        generate_rust_sdk(&GenerateRustSdkOpts {
            api_path: manifest_path.clone(),
            out_dir: out_dir.join("generated"),
            crate_name: "ops-sdk".to_string(),
            crate_version: "0.1.0".to_string(),
            runtime_deps: RustRuntimeDeps {
                source: RustRuntimeSource::Local,
                version: "0.1.0".to_string(),
                repo_root: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")),
            },
        })
        .unwrap();

        let operations_rs =
            fs::read_to_string(out_dir.join("generated/src/operations.rs")).unwrap();

        assert!(operations_rs.contains("impl OperationDescriptor for ExampleProcessOperation"));
        assert!(operations_rs.contains("type Update = serde_json::Value;"));
        assert!(operations_rs.contains("const UPDATE_SCHEMA_JSON: Option<&'static str> = None;"));
        assert!(operations_rs.contains("type Error = ExampleProcessOperationError;"));
        assert!(
            operations_rs.contains("const ERRORS: &'static [&'static str] = &[\"NotFoundError\"];")
        );
        assert!(operations_rs.contains("pub enum ExampleProcessOperationError {"));
        assert!(
            operations_rs.contains("impl OperationFailureLike for ExampleProcessOperationError {")
        );
        assert!(operations_rs.contains("NotFoundError(crate::types::NotFoundData),"));
        assert!(operations_rs.contains("impl trellis_rs::generated::DeclaredError"));
        assert!(!operations_rs.contains("Other"));
        assert!(operations_rs.contains("NoOperationUpdates, OperationDescriptor"));
        assert!(operations_rs.contains("use trellis_rs::service::OperationFailureLike;"));
        assert!(!operations_rs.contains("TransferOperationDescriptor"));

        let consumer_dir = out_dir.join("consumer");
        fs::create_dir_all(consumer_dir.join("src")).unwrap();
        fs::write(
            consumer_dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"ops-consumer\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nops-sdk = {{ path = {:?} }}\ntrellis-rs = {{ path = {:?} }}\n",
                out_dir.join("generated"),
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../trellis"),
            ),
        )
        .unwrap();
        fs::write(
            consumer_dir.join("src/main.rs"),
            r#"use ops_sdk::operations::ExampleProcessOperationError;
use trellis_rs::service::OperationFailureLike;

fn main() {
    let declared = ExampleProcessOperationError::NotFoundError(ops_sdk::NotFoundData {
        r#type: ops_sdk::NotFoundDataType::NotFoundError,
        message: "missing".to_string(),
        id: "order-1".to_string(),
    });
    assert_eq!(declared.message(), "missing");
    assert_eq!(declared.fields()["id"], "order-1");
}
"#,
        )
        .unwrap();
        cargo_check(&consumer_dir.join("Cargo.toml"));

        fs::remove_dir_all(out_dir).unwrap();
    }
}
