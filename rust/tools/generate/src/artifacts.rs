use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use trellis_codegen_rust::{
    default_sdk_stem, rust_sdk_cargo_manifest_is_valid, GenerateRustParticipantFacadeOpts,
    GenerateRustSdkOpts, RustRuntimeDeps, RustRuntimeSource as CodegenRustRuntimeSource,
};
use trellis_codegen_ts::{
    collect_ts_sdk_sources, render_ts_sdk_config, write_ts_sdk_sources, GenerateTsSdkOpts,
    TsRuntimeDeps, TsRuntimeSource as CodegenTsRuntimeSource,
};
use trellis_contracts::{canonicalize_json, ApiBuilder, ContractBuilder};

use crate::contract_input::{self, ResolvedNativeInput};
use crate::model::{ContractInput, RuntimeSource};
use crate::output;

const TRELLIS_DENO_JSON: &str = include_str!("../../../../ts/packages/trellis/deno.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedArtifactsMetadata {
    pub schema_version: u8,
    pub contract_id: String,
    pub api_version: String,
    pub api_digest: String,
    pub participant_digest: Option<String>,
    pub api_sdk_version: String,
    pub participant_version: Option<String>,
    pub runtime_source: RuntimeSource,
    pub runtime_version: String,
    pub has_jsr_package: bool,
    pub has_cargo_package: bool,
    pub package_name: String,
    pub crate_name: String,
    pub model_fingerprint: String,
    pub ts_codegen_fingerprint: String,
    pub rust_codegen_fingerprint: String,
}

#[derive(Debug, Clone, Copy)]
pub struct GeneratorFingerprints {
    pub model: &'static str,
    pub ts: &'static str,
    pub rust: &'static str,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TargetFreshness {
    pub api: bool,
    pub jsr: bool,
    pub cargo: bool,
    pub participant: bool,
}

impl TargetFreshness {
    pub fn all(self) -> bool {
        self.api && self.jsr && self.cargo && self.participant
    }
}

pub(crate) struct ContractOutputPlan<'a> {
    pub api_sdk_version: &'a str,
    pub participant_version: Option<&'a str>,
    pub runtime_version: &'a str,
    pub out_api: &'a Path,
    pub ts_out: Option<&'a Path>,
    pub rust_out: Option<&'a Path>,
    pub package_name: &'a str,
    pub crate_name: &'a str,
    pub runtime_source: RuntimeSource,
    pub runtime_repo_root: Option<&'a Path>,
    pub fingerprints: GeneratorFingerprints,
}

impl GeneratedArtifactsMetadata {
    const SCHEMA_VERSION: u8 = 7;
}

pub fn detect_output_root(project_root: &Path) -> PathBuf {
    let mut current = Some(project_root);
    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return dir.to_path_buf();
        }
        current = dir.parent();
    }
    project_root.to_path_buf()
}

pub fn detect_runtime_source(output_root: &Path) -> RuntimeSource {
    if output_root.join("rust/Cargo.toml").exists()
        && output_root.join("ts/packages/trellis").exists()
    {
        RuntimeSource::Local
    } else {
        RuntimeSource::Registry
    }
}

pub fn sdk_output_stem(contract_id: &str) -> String {
    match contract_id {
        "trellis.core@v1" => "trellis-core".to_string(),
        _ => default_sdk_stem(contract_id),
    }
}

pub fn resolve_contract(args: &ContractInput) -> miette::Result<ResolvedNativeInput> {
    let resolved = contract_input::resolve_contract_input(
        args.api.as_deref(),
        args.participant.as_deref(),
        &args.referenced_api,
        args.source.as_deref(),
        args.image.as_deref(),
        &args.source_export,
        &args.image_api_path,
    )?;
    contract_input::warn_forward_incompatible_public_schemas(&resolved.api);
    Ok(resolved)
}

pub(crate) fn write_contract_outputs(
    resolved: &ResolvedNativeInput,
    plan: &ContractOutputPlan<'_>,
    freshness: TargetFreshness,
) -> miette::Result<()> {
    let api = native_api_artifact(resolved)?;
    if let Some(parent) = plan.out_api.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }
    if !freshness.api {
        write_if_changed(plan.out_api, &format!("{}\n", api.json))?;
    }

    let ts_sources = match (plan.ts_out, freshness.jsr) {
        (Some(ts_out), false) => Some(
            collect_ts_sdk_sources(&GenerateTsSdkOpts {
                api_path: plan.out_api.to_path_buf(),
                out_dir: ts_out.to_path_buf(),
                package_name: plan.package_name.to_string(),
                package_version: plan.api_sdk_version.to_string(),
                runtime_deps: ts_runtime_deps(
                    RuntimeSource::Registry,
                    plan.runtime_version.to_owned(),
                    plan.runtime_repo_root.map(Path::to_path_buf),
                ),
            })
            .into_diagnostic()?,
        ),
        _ => None,
    };

    if let Some(ts_out) = plan.ts_out.filter(|_| !freshness.jsr) {
        let parent = ts_out.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).into_diagnostic()?;
        let staging = tempfile::Builder::new()
            .prefix(".trellis-jsr-")
            .tempdir_in(parent)
            .into_diagnostic()?;
        let staged_out = staging.path().join("package");
        let local_opts = GenerateTsSdkOpts {
            api_path: plan.out_api.to_path_buf(),
            out_dir: ts_out.to_path_buf(),
            package_name: plan.package_name.to_string(),
            package_version: plan.api_sdk_version.to_string(),
            runtime_deps: ts_runtime_deps(
                plan.runtime_source,
                plan.runtime_version.to_owned(),
                plan.runtime_repo_root.map(Path::to_path_buf),
            ),
        };
        let mut sources = ts_sources.clone().expect("rendered TypeScript sources");
        let config = render_ts_sdk_config(&local_opts).into_diagnostic()?;
        let config_path = config.path.clone();
        *sources
            .iter_mut()
            .find(|source| source.path == config_path)
            .expect("rendered SDK contains deno.json") = config;
        write_ts_sdk_sources(&staged_out, &sources).into_diagnostic()?;
        rewrite_local_generated_ts_sdk_imports(
            &staged_out,
            ts_out,
            plan.runtime_source,
            plan.runtime_repo_root,
        )?;
        format_generated_typescript_artifacts(&staged_out, plan.runtime_repo_root)?;
        copy_embedded_trellis_owned_ts_sdk(
            &resolved.api.render_model.id,
            &staged_out,
            plan.runtime_source,
            plan.runtime_repo_root,
        )?;
        install_staged_directory(&staged_out, ts_out, staging.path())?;
    }

    if let Some(rust_out) = plan.rust_out.filter(|_| !freshness.cargo) {
        trellis_codegen_rust::generate_rust_sdk(&GenerateRustSdkOpts {
            api_path: plan.out_api.to_path_buf(),
            out_dir: rust_out.to_path_buf(),
            crate_name: plan.crate_name.to_string(),
            crate_version: plan.api_sdk_version.to_string(),
            runtime_deps: rust_runtime_deps(
                plan.runtime_source,
                plan.runtime_version.to_owned(),
                plan.runtime_repo_root.map(Path::to_path_buf),
            ),
        })
        .into_diagnostic()?;
        copy_embedded_trellis_owned_rust_sdk(
            &resolved.api.render_model.id,
            rust_out,
            plan.runtime_source,
            plan.runtime_repo_root,
        )?;
    }

    output::print_success(&format!(
        "generated contract artifacts for {}",
        resolved.api.render_model.id
    ));
    output::print_detail("api", plan.out_api.display().to_string());
    output::print_detail("digest", &api.digest);
    Ok(())
}

/// Generate consumer-local SDKs from one canonical API artifact.
pub fn generate_installed_api(
    api_path: &Path,
    materialized_api: &Path,
    ts_out: Option<&Path>,
    rust_out: Option<&Path>,
) -> miette::Result<bool> {
    let resolved = resolve_contract(&ContractInput {
        api: Some(api_path.to_path_buf()),
        participant: None,
        referenced_api: Vec::new(),
        source: None,
        image: None,
        source_export: "API".to_owned(),
        image_api_path: "/trellis/api.json".to_owned(),
    })?;
    let version = resolved.api.api.version().to_owned();
    let id = &resolved.api.render_model.id;
    let package_name = format!("@trellis/{}", id.split('@').next().unwrap_or(id));
    let crate_name = default_rust_crate_name_from_id(id);
    let runtime_version = trellis_package_version();
    let plan = ContractOutputPlan {
        api_sdk_version: &version,
        participant_version: None,
        runtime_version: &runtime_version,
        out_api: materialized_api,
        ts_out,
        rust_out,
        package_name: &package_name,
        crate_name: &crate_name,
        runtime_source: RuntimeSource::Registry,
        runtime_repo_root: None,
        fingerprints: current_generator_fingerprints(),
    };
    let metadata = generated_artifacts_metadata(&resolved, &native_api_digest(&resolved)?, &plan);
    let freshness = generated_artifacts_are_fresh(&metadata, materialized_api, ts_out, rust_out);
    if freshness.all() {
        return Ok(false);
    }
    write_contract_outputs(&resolved, &plan, freshness)?;
    write_generated_artifacts_metadata(materialized_api, &metadata)?;
    Ok(true)
}

/// Check whether consumer-local SDK outputs match one exact locked API.
pub fn installed_api_is_fresh(
    id: &str,
    version: &str,
    digest: &str,
    materialized_api: &Path,
    ts_out: Option<&Path>,
    rust_out: Option<&Path>,
) -> bool {
    let package_name = format!("@trellis/{}", id.split('@').next().unwrap_or(id));
    let crate_name = default_rust_crate_name_from_id(id);
    let runtime_version = trellis_package_version();
    let plan = ContractOutputPlan {
        api_sdk_version: version,
        participant_version: None,
        runtime_version: &runtime_version,
        out_api: materialized_api,
        ts_out,
        rust_out,
        package_name: &package_name,
        crate_name: &crate_name,
        runtime_source: RuntimeSource::Registry,
        runtime_repo_root: None,
        fingerprints: current_generator_fingerprints(),
    };
    let expected = generated_artifacts_metadata_from_parts(id, version, digest, None, &plan);
    generated_artifacts_are_fresh(&expected, materialized_api, ts_out, rust_out).all()
}

struct NativeApiOutput {
    json: String,
    digest: String,
}

pub fn native_api_digest(resolved: &ResolvedNativeInput) -> miette::Result<String> {
    Ok(native_api_artifact(resolved)?.digest)
}

fn native_api_artifact(resolved: &ResolvedNativeInput) -> miette::Result<NativeApiOutput> {
    let api = ApiBuilder::new(resolved.api.value.clone())
        .build()
        .into_diagnostic()?;
    Ok(NativeApiOutput {
        json: serde_json::to_string(&api.normalized_value().into_diagnostic()?)
            .into_diagnostic()?,
        digest: api.digest().into_diagnostic()?,
    })
}

fn rewrite_local_generated_ts_sdk_imports(
    ts_out: &Path,
    logical_ts_out: &Path,
    runtime_source: RuntimeSource,
    runtime_repo_root: Option<&Path>,
) -> miette::Result<()> {
    if !matches!(runtime_source, RuntimeSource::Local) {
        return Ok(());
    }

    let repo_root = runtime_repo_root.ok_or_else(|| {
        miette::miette!("local generated TypeScript imports require a runtime repository root")
    })?;
    let depth = logical_ts_out
        .strip_prefix(repo_root)
        .into_diagnostic()?
        .components()
        .count();
    let errors_import = format!("{}ts/packages/trellis/errors/index.ts", "../".repeat(depth));
    for entry in fs::read_dir(ts_out).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();
        if !path.is_file() || path.extension().is_none_or(|extension| extension != "ts") {
            continue;
        }
        let contents = fs::read_to_string(&path).into_diagnostic()?;
        let rewritten = contents.replace(
            "from \"@qlever-llc/trellis/errors\"",
            &format!("from \"{errors_import}\""),
        );
        if rewritten != contents {
            write_if_changed(&path, &rewritten)?;
        }
    }

    Ok(())
}

pub(crate) fn write_participant_facade_outputs(
    protocol_participant_out: &Path,
    opts: GenerateRustParticipantFacadeOpts,
) -> miette::Result<()> {
    let (participant_json, participant_digest) = trellis_codegen_rust::native_participant_artifact(
        &opts.api_path,
        &opts.participant_path,
        &opts.alias_mappings,
    )
    .into_diagnostic()?;
    if let Some(parent) = protocol_participant_out.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }
    write_if_changed(protocol_participant_out, &format!("{participant_json}\n"))?;
    trellis_codegen_rust::generate_rust_participant_facade(&opts).into_diagnostic()?;
    output::print_detail("rust participant", opts.out_dir.display().to_string());
    output::print_detail(
        "participant",
        protocol_participant_out.display().to_string(),
    );
    output::print_detail("participant digest", participant_digest);
    Ok(())
}

pub fn write_protocol_participant(
    resolved: &ResolvedNativeInput,
    protocol_participant_out: &Path,
) -> miette::Result<()> {
    let participant = resolved
        .participant
        .as_ref()
        .ok_or_else(|| miette::miette!("missing resolved participant"))?;
    let referenced_apis = resolved
        .referenced_apis
        .iter()
        .map(|api| (api.render_model.id.clone(), api.value.clone()))
        .collect();
    let artifacts =
        ContractBuilder::from_native(resolved.api.value.clone(), participant.value.clone())
            .referenced_apis(referenced_apis)
            .build()
            .into_diagnostic()?;
    let participant_json =
        canonicalize_json(&artifacts.participant_value().into_diagnostic()?).into_diagnostic()?;
    let participant_digest = artifacts.participant_digest().into_diagnostic()?;
    if let Some(parent) = protocol_participant_out.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }
    write_if_changed(protocol_participant_out, &format!("{participant_json}\n"))?;
    output::print_detail(
        "participant",
        protocol_participant_out.display().to_string(),
    );
    output::print_detail("participant digest", participant_digest);
    Ok(())
}

fn install_staged_directory(
    staged: &Path,
    destination: &Path,
    backup_root: &Path,
) -> miette::Result<()> {
    let backup = backup_root.join("previous");
    if destination.exists() {
        fs::rename(destination, &backup).into_diagnostic()?;
    }
    if let Err(error) = fs::rename(staged, destination) {
        if backup.exists() {
            let _ = fs::rename(&backup, destination);
        }
        return Err(error).into_diagnostic();
    }
    Ok(())
}

pub(crate) fn generated_artifacts_metadata(
    resolved: &ResolvedNativeInput,
    api_digest: &str,
    plan: &ContractOutputPlan<'_>,
) -> GeneratedArtifactsMetadata {
    generated_artifacts_metadata_from_parts(
        &resolved.api.render_model.id,
        resolved.api.api.version(),
        api_digest,
        resolved
            .participant
            .as_ref()
            .map(|participant| participant.digest.as_str()),
        plan,
    )
}

pub(crate) fn generated_artifacts_metadata_from_parts(
    contract_id: &str,
    api_version: &str,
    api_digest: &str,
    participant_digest: Option<&str>,
    plan: &ContractOutputPlan<'_>,
) -> GeneratedArtifactsMetadata {
    GeneratedArtifactsMetadata {
        schema_version: GeneratedArtifactsMetadata::SCHEMA_VERSION,
        contract_id: contract_id.to_owned(),
        api_version: api_version.to_owned(),
        api_digest: api_digest.to_owned(),
        participant_digest: participant_digest.map(str::to_owned),
        api_sdk_version: plan.api_sdk_version.to_string(),
        participant_version: plan.participant_version.map(str::to_owned),
        runtime_source: plan.runtime_source,
        runtime_version: plan.runtime_version.to_string(),
        has_jsr_package: plan.ts_out.is_some(),
        has_cargo_package: plan.rust_out.is_some(),
        package_name: plan.package_name.to_string(),
        crate_name: plan.crate_name.to_string(),
        model_fingerprint: plan.fingerprints.model.to_string(),
        ts_codegen_fingerprint: plan.fingerprints.ts.to_string(),
        rust_codegen_fingerprint: plan.fingerprints.rust.to_string(),
    }
}

pub fn generated_artifacts_are_fresh(
    expected: &GeneratedArtifactsMetadata,
    out_api: &Path,
    ts_out: Option<&Path>,
    rust_out: Option<&Path>,
) -> TargetFreshness {
    let Some(existing) = read_generated_artifacts_metadata(out_api) else {
        return TargetFreshness::default();
    };
    let common = existing.schema_version == expected.schema_version
        && existing.contract_id == expected.contract_id
        && existing.api_version == expected.api_version
        && existing.api_digest == expected.api_digest
        && existing.api_sdk_version == expected.api_sdk_version
        && existing.runtime_source == expected.runtime_source
        && existing.runtime_version == expected.runtime_version
        && existing.package_name == expected.package_name
        && existing.crate_name == expected.crate_name
        && existing.model_fingerprint == expected.model_fingerprint;
    let api = common && out_api.exists();
    TargetFreshness {
        api,
        jsr: ts_out.is_none()
            || (api
                && existing.has_jsr_package
                && existing.ts_codegen_fingerprint == expected.ts_codegen_fingerprint
                && ts_key_outputs_exist(ts_out)
                && embedded_trellis_owned_ts_sdk_key_outputs_exist(expected, out_api)),
        cargo: rust_out.is_none()
            || (api
                && existing.has_cargo_package
                && existing.rust_codegen_fingerprint == expected.rust_codegen_fingerprint
                && rust_key_outputs_exist(rust_out, expected, out_api)),
        participant: existing.participant_digest == expected.participant_digest
            && existing.participant_version == expected.participant_version,
    }
}

fn read_generated_artifacts_metadata(out_api: &Path) -> Option<GeneratedArtifactsMetadata> {
    let contents = fs::read_to_string(generated_artifacts_metadata_path(out_api)).ok()?;
    serde_json::from_str(&contents).ok()
}

pub(crate) fn write_generated_artifacts_metadata(
    out_api: &Path,
    metadata: &GeneratedArtifactsMetadata,
) -> miette::Result<()> {
    write_if_changed(
        &generated_artifacts_metadata_path(out_api),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(metadata).into_diagnostic()?
        ),
    )
}

pub fn generated_artifacts_metadata_path(out_api: &Path) -> PathBuf {
    out_api.with_extension("trellis-artifacts.json")
}

fn ts_key_outputs_exist(ts_out: Option<&Path>) -> bool {
    let Some(ts_out) = ts_out else {
        return true;
    };
    ts_out.join("mod.ts").exists()
        && ts_out.join("descriptors.ts").exists()
        && ts_out.join("types.ts").exists()
        && ts_out.join("schemas.ts").exists()
        && ts_out.join("api.ts").exists()
}

fn embedded_trellis_owned_ts_sdk_key_outputs_exist(
    expected: &GeneratedArtifactsMetadata,
    out_api: &Path,
) -> bool {
    if !matches!(expected.runtime_source, RuntimeSource::Local) {
        return true;
    }
    let Some(module) = embedded_trellis_owned_rust_sdk_module(&expected.contract_id) else {
        return true;
    };
    let repo_root = detect_output_root(out_api.parent().unwrap_or(out_api));
    let embedded_dir = repo_root
        .join("ts/packages/trellis/internal_sdk/generated")
        .join(module);
    embedded_dir.join("mod.ts").exists() && embedded_dir.join("descriptors.ts").exists()
}

fn rust_key_outputs_exist(
    rust_out: Option<&Path>,
    expected: &GeneratedArtifactsMetadata,
    out_api: &Path,
) -> bool {
    let Some(rust_out) = rust_out else {
        return true;
    };
    let cargo_toml = rust_out.join("Cargo.toml");
    let source = if manifest_is_api(out_api) {
        "api.rs"
    } else {
        "contract.rs"
    };
    cargo_toml.exists()
        && rust_out.join("src").join(source).exists()
        && embedded_trellis_owned_rust_sdk_key_outputs_exist(expected, out_api)
        && rust_sdk_cargo_manifest_is_valid(
            &cargo_toml,
            &expected.crate_name,
            &expected.api_sdk_version,
        )
}

fn embedded_trellis_owned_rust_sdk_key_outputs_exist(
    expected: &GeneratedArtifactsMetadata,
    out_api: &Path,
) -> bool {
    if !matches!(expected.runtime_source, RuntimeSource::Local) {
        return true;
    }
    let Some(module) = embedded_trellis_owned_rust_sdk_module(&expected.contract_id) else {
        return true;
    };
    let repo_root = detect_output_root(out_api.parent().unwrap_or(out_api));
    let embedded_dir = repo_root
        .join("rust/crates/trellis/src/internal_sdk/generated")
        .join(module);
    let source = if manifest_is_api(out_api) {
        "api.rs"
    } else {
        "contract.rs"
    };
    embedded_dir.join("mod.rs").exists() && embedded_dir.join(source).exists()
}

fn manifest_is_api(path: &Path) -> bool {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str::<Value>(&contents).ok())
        .and_then(|value| {
            value
                .get("format")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .as_deref()
        == Some("trellis.api.v1")
}

pub fn copy_embedded_trellis_owned_rust_sdk(
    contract_id: &str,
    rust_out: &Path,
    runtime_source: RuntimeSource,
    runtime_repo_root: Option<&Path>,
) -> miette::Result<()> {
    if !matches!(runtime_source, RuntimeSource::Local) {
        return Ok(());
    }
    let Some(module) = embedded_trellis_owned_rust_sdk_module(contract_id) else {
        return Ok(());
    };
    let Some(repo_root) = runtime_repo_root else {
        return Ok(());
    };
    let src_dir = rust_out.join("src");
    let dest_dir = repo_root
        .join("rust/crates/trellis/src/internal_sdk/generated")
        .join(module);
    let source_digest = embedded_sdk_source_digest(&src_dir, "rs", "rust-v1")?;
    let digest_path = dest_dir.join(".trellis-source.digest");
    if fs::read_to_string(&digest_path).ok().as_deref() == Some(&source_digest) {
        return Ok(());
    }
    fs::create_dir_all(&dest_dir).into_diagnostic()?;
    let mut expected = BTreeSet::new();
    let mut changed = false;

    for entry in fs::read_dir(&src_dir).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();
        if !path.is_file() || path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let file_name = path.file_name().ok_or_else(|| {
            miette::miette!(
                "generated Rust SDK source path has no file name: {}",
                path.display()
            )
        })?;
        let is_root = file_name == "lib.rs";
        let contents = fs::read_to_string(&path).into_diagnostic()?;
        let dest_name = if is_root {
            OsString::from("mod.rs")
        } else {
            file_name.to_os_string()
        };
        let dest_path = dest_dir.join(dest_name);
        expected.insert(dest_path.clone());
        let rewritten = rewrite_embedded_rust_sdk_source(&contents, is_root);
        changed |= fs::read_to_string(&dest_path).ok().as_deref() != Some(&rewritten);
        write_if_changed(&dest_path, &rewritten)?;
    }
    for entry in fs::read_dir(&dest_dir).into_diagnostic()? {
        let path = entry.into_diagnostic()?.path();
        if path.extension().is_some_and(|extension| extension == "rs") && !expected.contains(&path)
        {
            fs::remove_file(path).into_diagnostic()?;
            changed = true;
        }
    }
    if changed {
        format_rust_files(&dest_dir)?;
    }
    write_if_changed(&digest_path, &source_digest)?;
    Ok(())
}

fn copy_embedded_trellis_owned_ts_sdk(
    contract_id: &str,
    ts_out: &Path,
    runtime_source: RuntimeSource,
    runtime_repo_root: Option<&Path>,
) -> miette::Result<()> {
    if !matches!(runtime_source, RuntimeSource::Local) {
        return Ok(());
    }
    let Some(module) = embedded_trellis_owned_rust_sdk_module(contract_id) else {
        return Ok(());
    };
    let Some(repo_root) = runtime_repo_root else {
        return Ok(());
    };
    let dest_dir = repo_root
        .join("ts/packages/trellis/internal_sdk/generated")
        .join(module);
    let source_digest = embedded_sdk_source_digest(ts_out, "ts", "typescript-v1")?;
    let digest_path = dest_dir.join(".trellis-source.digest");
    if fs::read_to_string(&digest_path).ok().as_deref() == Some(&source_digest) {
        return Ok(());
    }
    fs::create_dir_all(&dest_dir).into_diagnostic()?;
    let mut expected = BTreeSet::new();
    let mut changed = false;

    for entry in fs::read_dir(ts_out).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();
        if !path.is_file() || path.extension().is_none_or(|extension| extension != "ts") {
            continue;
        }
        let file_name = path.file_name().ok_or_else(|| {
            miette::miette!(
                "generated TypeScript SDK source path has no file name: {}",
                path.display()
            )
        })?;
        let contents = fs::read_to_string(&path).into_diagnostic()?;
        let contents = rewrite_embedded_trellis_owned_ts_sdk_source(&contents);
        let dest_path = dest_dir.join(file_name);
        expected.insert(dest_path.clone());
        changed |= fs::read_to_string(&dest_path).ok().as_deref() != Some(&contents);
        write_if_changed(&dest_path, &contents)?;
    }
    for entry in fs::read_dir(&dest_dir).into_diagnostic()? {
        let path = entry.into_diagnostic()?.path();
        if path.extension().is_some_and(|extension| extension == "ts") && !expected.contains(&path)
        {
            fs::remove_file(path).into_diagnostic()?;
            changed = true;
        }
    }
    if changed {
        format_generated_typescript_artifacts(&dest_dir, Some(repo_root))?;
    }
    write_if_changed(&digest_path, &source_digest)?;
    Ok(())
}

fn embedded_sdk_source_digest(
    source_dir: &Path,
    extension: &str,
    rewrite_version: &str,
) -> miette::Result<String> {
    let mut paths = fs::read_dir(source_dir)
        .into_diagnostic()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|candidate| candidate == extension)
        })
        .collect::<Vec<_>>();
    paths.sort();
    let mut hasher = Sha256::new();
    hasher.update(rewrite_version.as_bytes());
    for path in paths {
        hasher.update(
            path.file_name()
                .expect("generated SDK file name")
                .as_encoded_bytes(),
        );
        hasher.update(fs::read(path).into_diagnostic()?);
    }
    let digest = hasher
        .finalize()
        .iter()
        .fold(String::with_capacity(65), |mut digest, byte| {
            write!(digest, "{byte:02x}").expect("writing to a string cannot fail");
            digest
        });
    Ok(format!("{digest}\n"))
}

fn rewrite_embedded_trellis_owned_ts_sdk_source(contents: &str) -> String {
    contents
        .replace(
            "from \"@qlever-llc/trellis/contracts\"",
            "from \"../../../contracts.ts\"",
        )
        .replace(
            "from \"../../../../ts/packages/trellis/errors/index.ts\"",
            "from \"../../../errors/index.ts\"",
        )
        .replace(
            "from \"../../../../../../../../ts/packages/trellis/errors/index.ts\"",
            "from \"../../../errors/index.ts\"",
        )
        .replace(
            "from \"@qlever-llc/trellis/errors\"",
            "from \"../../../errors/index.ts\"",
        )
        .replace("from \"@qlever-llc/trellis\"", "from \"../../../index.ts\"")
}

fn rewrite_embedded_rust_sdk_source(contents: &str, is_root: bool) -> String {
    let rewritten = if is_root {
        contents.replace("crate::", "self::")
    } else {
        contents.replace("crate::", "super::")
    };
    rewritten
        .replace("trellis_rs::", "crate::")
        .replace("trellis_client::", "crate::client::")
        .replace("trellis_contracts::", "crate::contracts::")
}

pub fn format_generated_typescript_artifacts(
    path: &Path,
    runtime_repo_root: Option<&Path>,
) -> miette::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let Some(config) = runtime_repo_root
        .map(|root| root.join("ts/deno.json"))
        .filter(|config| config.exists())
    else {
        return Ok(());
    };

    let mut command = crate::timings::command("deno");
    command.arg("fmt").arg("-c").arg(config);
    command.arg(path);

    let output = command.output().into_diagnostic()?;
    if output.status.success() {
        return Ok(());
    }

    Err(miette::miette!(
        "deno fmt failed for {}\nstdout:\n{}\nstderr:\n{}",
        path.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn format_rust_files(root: &Path) -> miette::Result<()> {
    let mut paths = fs::read_dir(root)
        .into_diagnostic()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .collect::<Vec<_>>();
    paths.sort();
    if paths.is_empty() {
        return Ok(());
    }
    let output = crate::timings::command("rustfmt")
        .args(["--edition", "2021"])
        .args(paths)
        .output()
        .into_diagnostic()?;
    if !output.status.success() {
        return Err(miette::miette!(
            "rustfmt failed for {}\nstderr:\n{}",
            root.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn write_if_changed(path: &Path, contents: &str) -> miette::Result<()> {
    if fs::read_to_string(path).ok().as_deref() == Some(contents) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }
    fs::write(path, contents).into_diagnostic()?;
    Ok(())
}

pub fn current_generator_fingerprints() -> GeneratorFingerprints {
    GeneratorFingerprints {
        model: env!("TRELLIS_MODEL_FINGERPRINT"),
        ts: env!("TRELLIS_TS_CODEGEN_FINGERPRINT"),
        rust: env!("TRELLIS_RUST_CODEGEN_FINGERPRINT"),
    }
}

pub fn required_owner_version(
    resolved: &ResolvedNativeInput,
    action: &str,
) -> miette::Result<String> {
    resolved.owner_version.clone().ok_or_else(|| {
        miette::miette!(
            "cannot {action}: no owning workspace version could be inferred from the contract input; use a source file or a manifest located under a versioned workspace"
        )
    })
}

pub fn ts_package_name_from_id(contract_id: &str, prefix: &str) -> String {
    let stem = contract_id
        .split('@')
        .next()
        .unwrap_or("trellis-sdk")
        .replace('.', "-");

    format!("{prefix}{stem}")
}

pub fn default_rust_crate_name_from_id(contract_id: &str) -> String {
    trellis_codegen_rust::default_sdk_crate_name(contract_id)
}

fn embedded_trellis_owned_rust_sdk_module(contract_id: &str) -> Option<&'static str> {
    match contract_id {
        "trellis.auth@v1" => Some("auth"),
        "trellis.core@v1" => Some("core"),
        "trellis.health@v1" => Some("health"),
        "trellis.eventlog@v1" => Some("eventlog"),
        "trellis.jobs@v1" => Some("jobs"),
        "trellis.state@v1" => Some("state"),
        _ => None,
    }
}

pub fn trellis_package_version() -> String {
    let manifest: serde_json::Value = serde_json::from_str(TRELLIS_DENO_JSON)
        .expect("bundled Trellis Deno package manifest must be valid JSON");
    manifest
        .get("version")
        .and_then(serde_json::Value::as_str)
        .expect("bundled Trellis Deno package manifest must have a version")
        .to_string()
}

pub fn rust_runtime_deps(
    source: RuntimeSource,
    version: String,
    repo_root: Option<PathBuf>,
) -> RustRuntimeDeps {
    RustRuntimeDeps {
        source: match source {
            RuntimeSource::Registry => CodegenRustRuntimeSource::Registry,
            RuntimeSource::Local => CodegenRustRuntimeSource::Local,
        },
        version,
        repo_root,
    }
}

pub fn ts_runtime_deps(
    source: RuntimeSource,
    version: String,
    repo_root: Option<PathBuf>,
) -> TsRuntimeDeps {
    TsRuntimeDeps {
        source: match source {
            RuntimeSource::Registry => CodegenTsRuntimeSource::Registry,
            RuntimeSource::Local => CodegenTsRuntimeSource::Local,
        },
        version,
        repo_root,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::model::RuntimeSource;

    use super::{
        format_rust_files, generated_artifacts_are_fresh, rewrite_embedded_rust_sdk_source,
        rewrite_embedded_trellis_owned_ts_sdk_source, rewrite_local_generated_ts_sdk_imports,
        trellis_package_version, ts_package_name_from_id, write_generated_artifacts_metadata,
        GeneratedArtifactsMetadata,
    };

    #[test]
    fn generated_ts_package_names_use_private_default_namespace() {
        assert_eq!(
            ts_package_name_from_id("trellis.demo-service@v1", "@trellis-sdk/"),
            "@trellis-sdk/trellis-demo-service",
        );
    }

    #[test]
    fn participant_digest_invalidates_participant_target_only() {
        let temp = tempfile::tempdir().unwrap();
        let out_api = temp.path().join("api.json");
        fs::write(&out_api, "{}\n").unwrap();
        let expected = GeneratedArtifactsMetadata {
            schema_version: GeneratedArtifactsMetadata::SCHEMA_VERSION,
            contract_id: "trellis.test@v1".to_string(),
            api_version: "1.0.0".to_string(),
            api_digest: "api".to_string(),
            participant_digest: Some("new".to_string()),
            api_sdk_version: "1.0.0".to_string(),
            participant_version: None,
            runtime_source: RuntimeSource::Registry,
            runtime_version: "1.0.0".to_string(),
            has_jsr_package: false,
            has_cargo_package: false,
            package_name: "@test/sdk".to_string(),
            crate_name: "test_sdk".to_string(),
            model_fingerprint: "model".to_string(),
            ts_codegen_fingerprint: "ts".to_string(),
            rust_codegen_fingerprint: "rust".to_string(),
        };
        let mut existing = expected.clone();
        existing.participant_digest = Some("old".to_string());
        write_generated_artifacts_metadata(&out_api, &existing).unwrap();

        let freshness = generated_artifacts_are_fresh(&expected, &out_api, None, None);
        assert!(freshness.api);
        assert!(!freshness.participant);
        assert!(!freshness.all());
    }

    #[test]
    fn local_import_rewrite_uses_logical_output_depth() {
        let temp = tempfile::tempdir().unwrap();
        let staged = temp.path().join(".staging/package");
        fs::create_dir_all(&staged).unwrap();
        fs::write(
            staged.join("types.ts"),
            "import { TrellisError } from \"@qlever-llc/trellis/errors\";\n",
        )
        .unwrap();
        rewrite_local_generated_ts_sdk_imports(
            &staged,
            &temp.path().join("generated/packages/jsr/test"),
            RuntimeSource::Local,
            Some(temp.path()),
        )
        .unwrap();

        let rewritten = fs::read_to_string(staged.join("types.ts")).unwrap();
        assert!(rewritten.contains("../../../../ts/packages/trellis/errors/index.ts"));
        assert!(!rewritten.contains(".staging"));
    }

    #[test]
    fn generated_ts_package_names_apply_prefix() {
        assert_eq!(
            ts_package_name_from_id("trellis.demo-service@v1", "@example/"),
            "@example/trellis-demo-service",
        );
        assert_eq!(
            ts_package_name_from_id("trellis.demo-service@v1", "example-sdk-"),
            "example-sdk-trellis-demo-service",
        );
    }

    #[test]
    fn generated_ts_package_names_keep_trellis_owned_contracts_private() {
        assert_eq!(
            ts_package_name_from_id("trellis.core@v1", "@example/"),
            "@example/trellis-core",
        );
    }

    #[test]
    fn trellis_package_version_comes_from_bundled_package_metadata() {
        assert_ne!(trellis_package_version(), "0.0.0");
    }

    #[test]
    fn embedded_trellis_owned_ts_sdk_uses_package_relative_imports() {
        let source = concat!(
            "import { rpcAction, schema } from \"@qlever-llc/trellis/contracts\";\n",
            "import { TrellisError } from \"@qlever-llc/trellis/errors\";\n",
        );

        let rewritten = rewrite_embedded_trellis_owned_ts_sdk_source(source);

        assert!(rewritten.contains("from \"../../../contracts.ts\""));
        assert!(rewritten.contains("from \"../../../errors/index.ts\""));
        assert!(!rewritten.contains("from \"@qlever-llc/trellis"));
    }

    #[test]
    fn embedded_rust_sdk_copy_is_formatted_after_rewrite() {
        let temp = tempfile::tempdir().expect("create tempdir");
        let dest = temp.path().join("rpc.rs");
        let rewritten = rewrite_embedded_rust_sdk_source(
            "use trellis_rs::service::OperationFailureLike;\npub fn client( )->trellis_client::Result<()> { todo!() }\n",
            false,
        );
        fs::write(&dest, rewritten).expect("write rewritten Rust SDK source");
        format_rust_files(temp.path()).expect("format rewritten Rust SDK source");
        let formatted = fs::read_to_string(dest).expect("read formatted Rust SDK source");

        assert_eq!(
            formatted,
            "use crate::service::OperationFailureLike;\npub fn client() -> crate::client::Result<()> {\n    todo!()\n}\n"
        );
    }
}
