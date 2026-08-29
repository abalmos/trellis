use std::ffi::OsString;
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
    GeneratedTsSource, TsRuntimeDeps, TsRuntimeSource as CodegenTsRuntimeSource,
};
use trellis_contracts::{canonicalize_json, ApiBuilder, ContractBuilder};

use crate::cli::{ContractInputArgs, RuntimeSource};
use crate::contract_input::{self, ResolvedNativeInput};
use crate::output;

const TRELLIS_DENO_JSON: &str = include_str!("../../../../ts/packages/trellis/deno.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedArtifactsMetadata {
    pub schema_version: u8,
    pub contract_id: String,
    pub api_version: String,
    pub api_digest: String,
    pub participant_digest: Option<String>,
    pub artifact_version: String,
    pub runtime_source: RuntimeSource,
    pub jsr_runtime_version: String,
    pub has_jsr_package: bool,
    pub has_npm_package: bool,
    pub has_cargo_package: bool,
    pub package_name: String,
    pub crate_name: String,
    pub model_fingerprint: String,
    pub ts_codegen_fingerprint: String,
    pub npm_packaging_fingerprint: String,
    pub npm_toolchain_fingerprint: Option<String>,
    pub rust_codegen_fingerprint: String,
}

#[derive(Debug, Clone, Copy)]
pub struct GeneratorFingerprints {
    pub model: &'static str,
    pub ts: &'static str,
    pub npm: &'static str,
    pub rust: &'static str,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TargetFreshness {
    pub api: bool,
    pub jsr: bool,
    pub npm: bool,
    pub cargo: bool,
    pub participant: bool,
}

impl TargetFreshness {
    pub fn all(self) -> bool {
        self.api && self.jsr && self.npm && self.cargo && self.participant
    }
}

pub struct NpmTsSources {
    pub root_dir: PathBuf,
}

pub(crate) struct ContractOutputPlan<'a> {
    pub artifact_version: &'a str,
    pub out_api: &'a Path,
    pub ts_out: Option<&'a Path>,
    pub npm_out: Option<&'a Path>,
    pub rust_out: Option<&'a Path>,
    pub package_name: &'a str,
    pub crate_name: &'a str,
    pub runtime_source: RuntimeSource,
    pub runtime_repo_root: Option<&'a Path>,
    pub fingerprints: GeneratorFingerprints,
}

pub struct ContractShellOutputPlan<'a> {
    pub contract_id: &'a str,
    pub artifact_version: &'a str,
    pub out_api: Option<&'a Path>,
    pub ts_out: Option<&'a Path>,
    pub npm_out: Option<&'a Path>,
    pub rust_out: Option<&'a Path>,
    pub package_name: &'a str,
    pub crate_name: &'a str,
    pub runtime_source: RuntimeSource,
    pub runtime_repo_root: Option<&'a Path>,
}

pub(crate) struct NpmPackageManifest<'a> {
    pub package_name: &'a str,
    pub package_version: &'a str,
    pub trellis_runtime_version: &'a str,
    pub contract_id: &'a str,
}

pub(crate) struct NpmPackageBuild<'a> {
    pub src_dir: &'a Path,
    pub npm_out: &'a Path,
    pub manifest: NpmPackageManifest<'a>,
    pub runtime_repo_root: Option<&'a Path>,
}

impl GeneratedArtifactsMetadata {
    const SCHEMA_VERSION: u8 = 5;
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

pub fn resolve_contract(args: &ContractInputArgs) -> miette::Result<ResolvedNativeInput> {
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

    let ts_sources = if (plan.ts_out.is_some() && !freshness.jsr)
        || (plan.npm_out.is_some() && !freshness.npm)
    {
        Some(
            collect_ts_sdk_sources(&GenerateTsSdkOpts {
                api_path: plan.out_api.to_path_buf(),
                out_dir: plan
                    .ts_out
                    .or(plan.npm_out)
                    .expect("checked TypeScript output")
                    .to_path_buf(),
                package_name: plan.package_name.to_string(),
                package_version: plan.artifact_version.to_string(),
                runtime_deps: ts_runtime_deps(
                    RuntimeSource::Registry,
                    trellis_package_version(),
                    plan.runtime_repo_root.map(Path::to_path_buf),
                ),
            })
            .into_diagnostic()?,
        )
    } else {
        None
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
            package_version: plan.artifact_version.to_string(),
            runtime_deps: ts_runtime_deps(
                plan.runtime_source,
                trellis_package_version(),
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

    if let Some(npm_out) = plan.npm_out.filter(|_| !freshness.npm) {
        let staging_dir = tempfile::tempdir().into_diagnostic()?;
        let npm_sources = stage_npm_ts_sources_from_sources(
            &resolved.api.render_model.id,
            staging_dir.path(),
            ts_sources.as_deref().expect("rendered TypeScript sources"),
        )?;
        build_npm_package_from_ts_sources(&NpmPackageBuild {
            src_dir: &npm_sources.root_dir,
            npm_out,
            manifest: NpmPackageManifest {
                package_name: plan.package_name,
                package_version: plan.artifact_version,
                trellis_runtime_version: &trellis_package_version(),
                contract_id: &resolved.api.render_model.id,
            },
            runtime_repo_root: plan.runtime_repo_root,
        })?;
    }

    if let Some(rust_out) = plan.rust_out.filter(|_| !freshness.cargo) {
        trellis_codegen_rust::generate_rust_sdk(&GenerateRustSdkOpts {
            api_path: plan.out_api.to_path_buf(),
            out_dir: rust_out.to_path_buf(),
            crate_name: plan.crate_name.to_string(),
            crate_version: plan.artifact_version.to_string(),
            runtime_deps: rust_runtime_deps(
                plan.runtime_source,
                plan.artifact_version.to_string(),
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

struct NativeApiOutput {
    json: String,
    digest: String,
}

pub fn native_api_digest(resolved: &ResolvedNativeInput) -> miette::Result<String> {
    Ok(native_api_artifact(resolved)?.digest)
}

pub fn native_api_json(resolved: &ResolvedNativeInput) -> miette::Result<String> {
    Ok(native_api_artifact(resolved)?.json)
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

pub fn write_contract_shell_outputs(plan: &ContractShellOutputPlan<'_>) -> miette::Result<()> {
    if let Some(ts_out) = plan.ts_out {
        write_ts_sdk_shell(
            plan.contract_id,
            plan.artifact_version,
            ts_out,
            plan.package_name,
            plan.runtime_source,
            plan.runtime_repo_root.map(Path::to_path_buf),
        )?;
    }

    if let Some(npm_out) = plan.npm_out {
        write_npm_package_shell(npm_out, plan.package_name, plan.artifact_version)?;
    }

    if let Some(rust_out) = plan.rust_out {
        write_rust_sdk_shell(
            plan.contract_id,
            plan.artifact_version,
            rust_out,
            plan.crate_name,
            plan.runtime_source,
            plan.runtime_repo_root.map(Path::to_path_buf),
        )?;
    }

    if let Some(out_api) = plan.out_api {
        if let Some(parent) = out_api.parent() {
            fs::create_dir_all(parent).into_diagnostic()?;
        }
        let metadata_path = generated_artifacts_metadata_path(out_api);
        if metadata_path.exists() {
            fs::remove_file(metadata_path).into_diagnostic()?;
        }
    }

    Ok(())
}

fn write_npm_package_shell(
    out: &Path,
    package_name: &str,
    package_version: &str,
) -> miette::Result<()> {
    fs::create_dir_all(out.join("esm")).into_diagnostic()?;
    write_if_changed(
        &out.join("package.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "name": package_name,
                "version": package_version,
                "type": "module",
                "exports": {
                    ".": {
                        "types": "./esm/mod.d.ts",
                        "import": "./esm/mod.js"
                    }
                },
                "types": "./esm/mod.d.ts"
            }))
            .into_diagnostic()?
        ),
    )?;
    write_if_changed(
        &out.join("esm/mod.js"),
        "export const API_DIGEST = \"shell\";\n",
    )?;
    write_if_changed(
        &out.join("esm/mod.d.ts"),
        "export declare const API_DIGEST = \"shell\";\n",
    )
}

fn write_ts_sdk_shell(
    contract_id: &str,
    artifact_version: &str,
    out: &Path,
    package_name: &str,
    runtime_source: RuntimeSource,
    runtime_repo_root: Option<PathBuf>,
) -> miette::Result<()> {
    if out.exists() {
        fs::remove_dir_all(out).into_diagnostic()?;
    }
    fs::create_dir_all(out).into_diagnostic()?;
    let runtime_deps =
        ts_runtime_deps(runtime_source, trellis_package_version(), runtime_repo_root);
    let deno = ts_shell_deno_json(package_name, artifact_version, &runtime_deps);
    write_if_changed(
        &out.join("deno.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&deno).into_diagnostic()?
        ),
    )?;
    write_if_changed(&out.join("mod.ts"), "export * from \"./api.ts\";\nexport * from \"./descriptors.ts\";\nexport * from \"./types.ts\";\nexport * from \"./schemas.ts\";\n")?;
    write_if_changed(&out.join("descriptors.ts"), "")?;
    write_if_changed(&out.join("types.ts"), "")?;
    write_if_changed(&out.join("schemas.ts"), "")?;
    write_if_changed(
        &out.join("api.ts"),
        &format!(
            "export const API_ID = {} as const;\nexport const API_DIGEST = \"shell\" as const;\n",
            js_string(contract_id)
        ),
    )?;
    Ok(())
}

fn write_rust_sdk_shell(
    contract_id: &str,
    artifact_version: &str,
    out: &Path,
    crate_name: &str,
    runtime_source: RuntimeSource,
    runtime_repo_root: Option<PathBuf>,
) -> miette::Result<()> {
    fs::create_dir_all(out.join("src")).into_diagnostic()?;
    let deps = rust_runtime_deps(
        runtime_source,
        artifact_version.to_string(),
        runtime_repo_root,
    );
    let opts = GenerateRustSdkOpts {
        api_path: out.join("shell.api.json"),
        out_dir: out.to_path_buf(),
        crate_name: crate_name.to_string(),
        crate_version: artifact_version.to_string(),
        runtime_deps: deps,
    };

    write_if_changed(
        &out.join("Cargo.toml"),
        &render_rust_shell_cargo_toml(contract_id, &opts),
    )?;
    write_if_changed(
        &out.join("src/lib.rs"),
        &render_rust_shell_lib_rs(contract_id),
    )?;
    Ok(())
}

fn render_rust_shell_cargo_toml(contract_id: &str, opts: &GenerateRustSdkOpts) -> String {
    let publish_line = if is_trellis_owned_sdk_contract(contract_id) {
        "publish = false\n"
    } else {
        ""
    };
    format!(
        "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2021\"\nlicense = \"Apache-2.0\"\n{}\n[dependencies]\nserde = {{ version = \"1.0\", features = [\"derive\"] }}\nserde_json = \"1.0\"\n{}\n",
        opts.crate_name,
        opts.crate_version,
        publish_line,
        rust_runtime_deps_lines(&opts.runtime_deps).join("\n")
    )
}

fn is_trellis_owned_sdk_contract(contract_id: &str) -> bool {
    matches!(
        contract_id,
        "trellis.auth@v1"
            | "trellis.core@v1"
            | "trellis.health@v1"
            | "trellis.eventlog@v1"
            | "trellis.jobs@v1"
            | "trellis.state@v1"
    )
}

fn rust_runtime_deps_lines(deps: &RustRuntimeDeps) -> Vec<String> {
    match deps.source {
        CodegenRustRuntimeSource::Registry => vec![
            format!("trellis-contracts = \"{}\"", deps.version),
            format!("trellis-rs = \"{}\"", deps.version),
        ],
        CodegenRustRuntimeSource::Local => {
            let repo_root = deps
                .repo_root
                .as_ref()
                .expect("local Rust SDK shell requires repo root");
            vec![
                format!(
                    "trellis-contracts = {{ path = {} }}",
                    string_literal(
                        &repo_root
                            .join("rust/crates/contracts")
                            .display()
                            .to_string()
                    )
                ),
                format!(
                    "trellis-rs = {{ path = {} }}",
                    string_literal(&repo_root.join("rust/crates/trellis").display().to_string())
                ),
            ]
        }
    }
}

fn render_rust_shell_lib_rs(contract_id: &str) -> String {
    format!(
        "//! Temporary generated Rust SDK shell used during `trellis-generate prepare`.\n\npub const API_ID: &str = {};\npub const API_DIGEST: &str = \"shell\";\npub const API_JSON: &str = \"{{}}\";\n",
        string_literal(contract_id),
    )
}

fn ts_shell_deno_json(
    package_name: &str,
    package_version: &str,
    runtime_deps: &TsRuntimeDeps,
) -> serde_json::Map<String, serde_json::Value> {
    let mut root = serde_json::Map::new();
    if let Some(extends) = ts_shell_extends(runtime_deps) {
        root.insert("extends".to_string(), serde_json::Value::String(extends));
    } else {
        let mut imports = serde_json::Map::new();
        imports.insert(
            "@qlever-llc/trellis".to_string(),
            serde_json::Value::String(format!("jsr:@qlever-llc/trellis@^{}", runtime_deps.version)),
        );
        root.insert("imports".to_string(), serde_json::Value::Object(imports));
    }
    root.insert(
        "name".to_string(),
        serde_json::Value::String(package_name.to_string()),
    );
    root.insert(
        "version".to_string(),
        serde_json::Value::String(package_version.to_string()),
    );
    root.insert(
        "exports".to_string(),
        serde_json::json!({ ".": "./mod.ts", "./api": "./api.ts" }),
    );
    root.insert(
        "compilerOptions".to_string(),
        serde_json::json!({
            "lib": ["dom", "dom.iterable", "dom.asynciterable", "deno.ns"],
            "strict": true,
            "verbatimModuleSyntax": true
        }),
    );
    root
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

fn ts_shell_extends(runtime_deps: &TsRuntimeDeps) -> Option<String> {
    let repo_root = runtime_deps.repo_root.as_ref()?;
    if !matches!(runtime_deps.source, CodegenTsRuntimeSource::Local) {
        return None;
    }
    Some(
        repo_root
            .join("ts/deno.json")
            .to_string_lossy()
            .replace('\\', "/"),
    )
}

fn js_string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a string cannot fail")
}

fn string_literal(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a string cannot fail")
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

pub(crate) fn build_npm_package_from_ts_sources(plan: &NpmPackageBuild<'_>) -> miette::Result<()> {
    let npm_out = if plan.npm_out.is_absolute() {
        plan.npm_out.to_path_buf()
    } else {
        std::env::current_dir()
            .into_diagnostic()?
            .join(plan.npm_out)
    };
    let parent = npm_out.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).into_diagnostic()?;
    let staging = tempfile::Builder::new()
        .prefix(".trellis-npm-")
        .tempdir_in(parent)
        .into_diagnostic()?;
    let staged_out = staging.path().join("package");
    fs::create_dir(&staged_out).into_diagnostic()?;
    let esm_dir = staged_out.join("esm");
    write_if_changed(
        &plan.src_dir.join("tsconfig.json"),
        &render_npm_tsconfig(&staged_out),
    )?;
    write_if_changed(
        &plan.src_dir.join("package.json"),
        "{\n  \"type\": \"module\"\n}\n",
    )?;
    let tsc = resolve_tsc_bin()?;
    let output = crate::timings::command(&tsc)
        .arg("-p")
        .arg(plan.src_dir.join("tsconfig.json"))
        .current_dir(plan.src_dir)
        .output()
        .into_diagnostic()?;
    if !output.status.success() {
        return Err(miette::miette!(
            "npm package TypeScript build failed for {} using {:?}\nstdout:\n{}\nstderr:\n{}",
            plan.manifest.package_name,
            tsc,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    write_npm_package_json(&staged_out, &plan.manifest)?;
    if let Ok(readme) = fs::read_to_string(plan.src_dir.join("README.md")) {
        write_if_changed(&staged_out.join("README.md"), &readme)?;
    }
    if !esm_dir.join("mod.js").exists() || !esm_dir.join("mod.d.ts").exists() {
        return Err(miette::miette!(
            "npm package TypeScript build for {} did not emit esm/mod.js and esm/mod.d.ts",
            plan.manifest.package_name
        ));
    }
    format_generated_npm_artifacts(&staged_out, plan.runtime_repo_root)?;

    install_staged_directory(&staged_out, &npm_out, staging.path())
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

pub fn stage_npm_ts_sources(
    contract_id: &str,
    api_path: &Path,
    staging_root: &Path,
    package_name: &str,
    package_version: &str,
) -> miette::Result<NpmTsSources> {
    let root_dir = staging_root.join(sdk_output_stem(contract_id));
    fs::create_dir_all(&root_dir).into_diagnostic()?;
    let opts = GenerateTsSdkOpts {
        api_path: api_path.to_path_buf(),
        out_dir: root_dir.clone(),
        package_name: package_name.to_string(),
        package_version: package_version.to_string(),
        runtime_deps: ts_runtime_deps(RuntimeSource::Registry, trellis_package_version(), None),
    };
    let sources = collect_ts_sdk_sources(&opts).into_diagnostic()?;
    stage_npm_ts_sources_from_sources(contract_id, staging_root, &sources)
}

fn stage_npm_ts_sources_from_sources(
    contract_id: &str,
    staging_root: &Path,
    sources: &[GeneratedTsSource],
) -> miette::Result<NpmTsSources> {
    let root_dir = staging_root.join(sdk_output_stem(contract_id));
    fs::create_dir_all(&root_dir).into_diagnostic()?;
    for source in sources {
        if source
            .path
            .extension()
            .is_some_and(|extension| extension == "ts")
        {
            write_if_changed(
                &root_dir.join(&source.path),
                &rewrite_npm_ts_imports(&source.contents),
            )?;
        } else if source.path == Path::new("README.md") {
            write_if_changed(&root_dir.join(&source.path), &source.contents)?;
        }
    }
    Ok(NpmTsSources { root_dir })
}

fn render_npm_tsconfig(npm_out: &Path) -> String {
    let out_dir = npm_out.join("esm").to_string_lossy().replace('\\', "/");
    format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({
            "compilerOptions": {
                "declaration": true,
                "emitDeclarationOnly": false,
                "module": "NodeNext",
                "moduleResolution": "NodeNext",
                "noCheck": true,
                "outDir": out_dir,
                "rootDir": ".",
                "skipLibCheck": true,
                "strict": true,
                "target": "ES2022",
                "verbatimModuleSyntax": true
            },
            "include": ["./*.ts"]
        }))
        .expect("npm tsconfig json")
    )
}

fn write_npm_package_json(npm_out: &Path, manifest: &NpmPackageManifest<'_>) -> miette::Result<()> {
    let trellis_dependency = format!("^{}", manifest.trellis_runtime_version);
    let mut peer_dependencies = serde_json::Map::new();
    peer_dependencies.insert(
        "@qlever-llc/trellis".to_string(),
        serde_json::Value::String(trellis_dependency.clone()),
    );
    write_if_changed(
        &npm_out.join("package.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "name": manifest.package_name,
                "version": manifest.package_version,
                "description": format!("Generated Trellis SDK for contract {}", manifest.contract_id),
                "type": "module",
                "license": "Apache-2.0",
                "homepage": "https://github.com/Qlever-LLC/trellis#readme",
                "bugs": {
                    "url": "https://github.com/Qlever-LLC/trellis/issues"
                },
                "repository": {
                    "type": "git",
                    "url": "https://github.com/Qlever-LLC/trellis"
                },
                "publishConfig": {
                    "access": "public"
                },
                "exports": {
                    ".": {
                        "types": "./esm/mod.d.ts",
                        "import": "./esm/mod.js"
                    }
                },
                "types": "./esm/mod.d.ts",
                "peerDependencies": peer_dependencies,
                "devDependencies": peer_dependencies
            }))
            .into_diagnostic()?
        ),
    )
}

fn rewrite_npm_ts_imports(contents: &str) -> String {
    contents.replace(".ts\"", ".js\"").replace(".ts'", ".js'")
}

fn npm_toolchain_fingerprint() -> Option<String> {
    let binary = std::env::var_os("TRELLIS_TSC_BIN")
        .filter(|binary| !binary.is_empty())
        .or_else(|| find_tsc_in_node_modules(&std::env::current_dir().ok()?))
        .and_then(resolve_binary_path)
        .or_else(|| resolve_binary_path(OsString::from("tsc")))?;
    let binary = binary.canonicalize().unwrap_or(binary);
    let mut digest = Sha256::new();
    digest.update(fs::read(&binary).ok()?);
    if let Some(package) = binary
        .ancestors()
        .map(|directory| directory.join("package.json"))
        .find(|path| path.is_file())
    {
        digest.update(fs::read(package).ok()?);
    }
    Some(
        digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}

fn resolve_binary_path(binary: OsString) -> Option<PathBuf> {
    let path = PathBuf::from(&binary);
    if path.components().count() > 1 {
        return path.is_file().then_some(path);
    }
    std::env::split_paths(&std::env::var_os("PATH")?).find_map(|directory| {
        let candidate = directory.join(&binary);
        candidate.is_file().then_some(candidate)
    })
}

fn resolve_tsc_bin() -> miette::Result<OsString> {
    if let Some(bin) = std::env::var_os("TRELLIS_TSC_BIN") {
        if !bin.is_empty() {
            return binary_is_available(&bin).then_some(bin).ok_or_else(|| {
                miette::miette!(
                    "TRELLIS_TSC_BIN is set, but the configured TypeScript compiler is not available"
                )
            });
        }
    }

    let current = std::env::current_dir().into_diagnostic()?;
    if let Some(tsc) = find_tsc_in_node_modules(&current) {
        return Ok(tsc);
    }

    let tsc = OsString::from("tsc");
    binary_is_available(&tsc).then_some(tsc).ok_or_else(|| {
        miette::miette!(
            "npm package generation requires the TypeScript compiler `tsc`; install TypeScript in the Node project, make `tsc` available on PATH, or set TRELLIS_TSC_BIN"
        )
    })
}

fn find_tsc_in_node_modules(start: &Path) -> Option<OsString> {
    let mut current = start.to_path_buf();
    loop {
        for candidate in [
            current.join("node_modules/.bin/tsc"),
            current.join("ts/node_modules/.bin/tsc"),
        ] {
            if candidate.exists() {
                return Some(candidate.into_os_string());
            }
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn binary_is_available(binary: &OsString) -> bool {
    crate::timings::command(binary)
        .arg("--version")
        .output()
        .is_ok()
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
        artifact_version: plan.artifact_version.to_string(),
        runtime_source: plan.runtime_source,
        jsr_runtime_version: trellis_package_version(),
        has_jsr_package: plan.ts_out.is_some(),
        has_npm_package: plan.npm_out.is_some(),
        has_cargo_package: plan.rust_out.is_some(),
        package_name: plan.package_name.to_string(),
        crate_name: plan.crate_name.to_string(),
        model_fingerprint: plan.fingerprints.model.to_string(),
        ts_codegen_fingerprint: plan.fingerprints.ts.to_string(),
        npm_packaging_fingerprint: plan.fingerprints.npm.to_string(),
        npm_toolchain_fingerprint: plan.npm_out.and_then(|_| npm_toolchain_fingerprint()),
        rust_codegen_fingerprint: plan.fingerprints.rust.to_string(),
    }
}

pub fn generated_artifacts_are_fresh(
    expected: &GeneratedArtifactsMetadata,
    out_api: &Path,
    ts_out: Option<&Path>,
    npm_out: Option<&Path>,
    rust_out: Option<&Path>,
) -> TargetFreshness {
    let Some(existing) = read_generated_artifacts_metadata(out_api) else {
        return TargetFreshness::default();
    };
    let common = existing.schema_version == expected.schema_version
        && existing.contract_id == expected.contract_id
        && existing.api_version == expected.api_version
        && existing.api_digest == expected.api_digest
        && existing.artifact_version == expected.artifact_version
        && existing.runtime_source == expected.runtime_source
        && existing.jsr_runtime_version == expected.jsr_runtime_version
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
        npm: npm_out.is_none()
            || (api
                && existing.has_npm_package
                && existing.ts_codegen_fingerprint == expected.ts_codegen_fingerprint
                && existing.npm_packaging_fingerprint == expected.npm_packaging_fingerprint
                && existing.npm_toolchain_fingerprint == expected.npm_toolchain_fingerprint
                && npm_key_outputs_exist(npm_out)),
        cargo: rust_out.is_none()
            || (api
                && existing.has_cargo_package
                && existing.rust_codegen_fingerprint == expected.rust_codegen_fingerprint
                && rust_key_outputs_exist(rust_out, expected, out_api)),
        participant: existing.participant_digest == expected.participant_digest,
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
    out_api.with_extension("trellis-generate.json")
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

fn npm_key_outputs_exist(npm_out: Option<&Path>) -> bool {
    let Some(npm_out) = npm_out else {
        return true;
    };
    npm_out.join("package.json").exists()
        && npm_out.join("esm/mod.js").exists()
        && npm_out.join("esm/mod.d.ts").exists()
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
        .join("ts/packages/trellis/sdk/_generated")
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
            &expected.artifact_version,
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
    let embedded_dir = repo_root.join("rust/crates/trellis/src/sdk").join(module);
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
    let dest_dir = repo_root.join("rust/crates/trellis/src/sdk").join(module);
    if dest_dir.exists() {
        fs::remove_dir_all(&dest_dir).into_diagnostic()?;
    }
    fs::create_dir_all(&dest_dir).into_diagnostic()?;

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
        let rewritten = rewrite_embedded_rust_sdk_source(&contents, is_root);
        write_if_changed(&dest_path, &rewritten)?;
    }
    format_rust_files(&dest_dir)?;
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
        .join("ts/packages/trellis/sdk/_generated")
        .join(module);
    if dest_dir.exists() {
        fs::remove_dir_all(&dest_dir).into_diagnostic()?;
    }
    fs::create_dir_all(&dest_dir).into_diagnostic()?;

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
        write_if_changed(&dest_dir.join(file_name), &contents)?;
    }
    format_generated_typescript_artifacts(&dest_dir, Some(repo_root))?;
    Ok(())
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

fn format_generated_npm_artifacts(
    npm_out: &Path,
    runtime_repo_root: Option<&Path>,
) -> miette::Result<()> {
    let Some(config) = runtime_repo_root
        .map(|root| root.join("ts/deno.json"))
        .filter(|config| config.exists())
    else {
        return Ok(());
    };
    let mut paths = Vec::new();
    let mut javascript = Vec::new();
    collect_npm_format_paths(npm_out, &mut paths, &mut javascript)?;
    if !javascript.is_empty() {
        let staging = tempfile::tempdir().into_diagnostic()?;
        let mut staged = Vec::new();
        for source in javascript {
            let mut relative = source
                .strip_prefix(npm_out)
                .into_diagnostic()?
                .to_path_buf();
            relative.set_extension("ts");
            let destination = staging.path().join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).into_diagnostic()?;
            }
            fs::copy(&source, &destination).into_diagnostic()?;
            staged.push((source, destination));
        }
        let output = crate::timings::command("deno")
            .arg("fmt")
            .arg("-c")
            .arg(&config)
            .arg(staging.path())
            .output()
            .into_diagnostic()?;
        miette::ensure!(
            output.status.success(),
            "deno fmt failed for JavaScript in {}\nstdout:\n{}\nstderr:\n{}",
            npm_out.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        for (source, formatted) in staged {
            fs::copy(formatted, source).into_diagnostic()?;
        }
    }
    if paths.is_empty() {
        return Ok(());
    }
    paths.sort();
    let output = crate::timings::command("deno")
        .arg("fmt")
        .arg("-c")
        .arg(config)
        .args(paths)
        .output()
        .into_diagnostic()?;
    miette::ensure!(
        output.status.success(),
        "deno fmt failed for {}\nstdout:\n{}\nstderr:\n{}",
        npm_out.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn collect_npm_format_paths(
    dir: &Path,
    paths: &mut Vec<PathBuf>,
    javascript: &mut Vec<PathBuf>,
) -> miette::Result<()> {
    for entry in fs::read_dir(dir).into_diagnostic()? {
        let path = entry.into_diagnostic()?.path();
        if path.is_dir() {
            collect_npm_format_paths(&path, paths, javascript)?;
        } else if path.extension().is_some_and(|extension| extension == "js") {
            javascript.push(path);
        } else if matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("ts" | "json" | "md")
        ) {
            paths.push(path);
        }
    }
    Ok(())
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
        npm: env!("TRELLIS_NPM_PACKAGING_FINGERPRINT"),
        rust: env!("TRELLIS_RUST_CODEGEN_FINGERPRINT"),
    }
}

pub fn infer_artifact_version(
    resolved: &ResolvedNativeInput,
    explicit: Option<String>,
    action: &str,
) -> miette::Result<String> {
    explicit.or(resolved.owner_version.clone()).ok_or_else(|| {
        miette::miette!(
            "cannot {action}: no version could be inferred; pass --artifact-version when using --api or --image"
        )
    })
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

pub fn default_ts_package_name_from_id(contract_id: &str) -> String {
    ts_package_name_from_id(contract_id, "@trellis-sdk/")
}

pub fn ts_package_name_from_id(contract_id: &str, prefix: &str) -> String {
    let stem = contract_id
        .split('@')
        .next()
        .unwrap_or("trellis-sdk")
        .replace('.', "-");

    match stem.as_str() {
        "trellis-auth" => "@qlever-llc/trellis/sdk/auth".to_string(),
        "trellis-core" => "@qlever-llc/trellis/sdk/core".to_string(),
        "trellis-health" => "@qlever-llc/trellis/sdk/health".to_string(),
        "trellis-eventlog" => "@qlever-llc/trellis/sdk/eventlog".to_string(),
        "trellis-jobs" => "@qlever-llc/trellis/sdk/jobs".to_string(),
        "trellis-state" => "@qlever-llc/trellis/sdk/state".to_string(),
        other => format!("{prefix}{other}"),
    }
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

    use crate::cli::RuntimeSource;

    use super::{
        find_tsc_in_node_modules, format_rust_files, generated_artifacts_are_fresh,
        generated_artifacts_metadata_path, render_npm_tsconfig, rewrite_embedded_rust_sdk_source,
        rewrite_embedded_trellis_owned_ts_sdk_source, rewrite_local_generated_ts_sdk_imports,
        rewrite_npm_ts_imports, trellis_package_version, ts_package_name_from_id,
        write_contract_shell_outputs, write_generated_artifacts_metadata, ContractShellOutputPlan,
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
            artifact_version: "1.0.0".to_string(),
            runtime_source: RuntimeSource::Registry,
            jsr_runtime_version: "1.0.0".to_string(),
            has_jsr_package: false,
            has_npm_package: false,
            has_cargo_package: false,
            package_name: "@test/sdk".to_string(),
            crate_name: "test_sdk".to_string(),
            model_fingerprint: "model".to_string(),
            ts_codegen_fingerprint: "ts".to_string(),
            npm_packaging_fingerprint: "npm".to_string(),
            npm_toolchain_fingerprint: None,
            rust_codegen_fingerprint: "rust".to_string(),
        };
        let mut existing = expected.clone();
        existing.participant_digest = Some("old".to_string());
        write_generated_artifacts_metadata(&out_api, &existing).unwrap();

        let freshness = generated_artifacts_are_fresh(&expected, &out_api, None, None, None);
        assert!(freshness.api);
        assert!(!freshness.participant);
        assert!(!freshness.all());
    }

    #[test]
    fn npm_toolchain_change_invalidates_npm_target() {
        let temp = tempfile::tempdir().unwrap();
        let out_api = temp.path().join("api.json");
        let npm_out = temp.path().join("npm");
        fs::create_dir_all(npm_out.join("esm")).unwrap();
        for path in ["package.json", "esm/mod.js", "esm/mod.d.ts"] {
            fs::write(npm_out.join(path), "{}\n").unwrap();
        }
        fs::write(&out_api, "{}\n").unwrap();
        let expected = GeneratedArtifactsMetadata {
            schema_version: GeneratedArtifactsMetadata::SCHEMA_VERSION,
            contract_id: "trellis.test@v1".to_string(),
            api_version: "1.0.0".to_string(),
            api_digest: "api".to_string(),
            participant_digest: None,
            artifact_version: "1.0.0".to_string(),
            runtime_source: RuntimeSource::Registry,
            jsr_runtime_version: "1.0.0".to_string(),
            has_jsr_package: false,
            has_npm_package: true,
            has_cargo_package: false,
            package_name: "@test/sdk".to_string(),
            crate_name: "test_sdk".to_string(),
            model_fingerprint: "model".to_string(),
            ts_codegen_fingerprint: "ts".to_string(),
            npm_packaging_fingerprint: "npm".to_string(),
            npm_toolchain_fingerprint: Some("new".to_string()),
            rust_codegen_fingerprint: "rust".to_string(),
        };
        let mut existing = expected.clone();
        existing.npm_toolchain_fingerprint = Some("old".to_string());
        write_generated_artifacts_metadata(&out_api, &existing).unwrap();

        assert!(
            !generated_artifacts_are_fresh(&expected, &out_api, None, Some(&npm_out), None,).npm
        );
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
    fn generated_ts_package_names_keep_trellis_owned_contracts_canonical() {
        assert_eq!(
            ts_package_name_from_id("trellis.core@v1", "@example/"),
            "@qlever-llc/trellis/sdk/core",
        );
    }

    #[test]
    fn trellis_package_version_comes_from_bundled_package_metadata() {
        assert_ne!(trellis_package_version(), "0.0.0");
    }

    #[test]
    fn npm_tsconfig_uses_node_esm_declaration_output() {
        let tsconfig = render_npm_tsconfig(std::path::Path::new("/tmp/npm-out"));

        assert!(tsconfig.contains("\"module\": \"NodeNext\""));
        assert!(tsconfig.contains("\"declaration\": true"));
        assert!(tsconfig.contains("/tmp/npm-out/esm"));
        assert!(!tsconfig.contains("deno"));
        assert!(!tsconfig.contains("dnt"));
    }

    #[test]
    fn tsc_lookup_finds_ts_workspace_node_modules_from_repo_root() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let tsc = temp.path().join("ts/node_modules/.bin/tsc");
        fs::create_dir_all(tsc.parent().expect("tsc parent")).expect("create tsc parent");
        fs::write(&tsc, "#!/bin/sh\n").expect("write tsc");

        assert_eq!(
            find_tsc_in_node_modules(temp.path()),
            Some(tsc.into_os_string())
        );
    }

    #[test]
    fn npm_source_rewrite_changes_local_ts_specifiers_to_js() {
        let source = "export * from \"./types.ts\";\nimport { rpcAction } from '@qlever-llc/trellis/contracts';\n";
        let rewritten = rewrite_npm_ts_imports(source);

        assert!(rewritten.contains("./types.js"));
        assert!(rewritten.contains("@qlever-llc/trellis"));
        assert!(!rewritten.contains(".ts\""));
        assert!(!rewritten.contains(".ts'"));
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

    #[test]
    fn contract_shell_outputs_create_empty_typescript_vocabulary() {
        let temp = tempfile::tempdir().expect("create tempdir");
        let manifest = temp.path().join("generated/protocol/apis/demo@v1.json");
        let metadata = generated_artifacts_metadata_path(&manifest);
        fs::create_dir_all(metadata.parent().expect("metadata parent"))
            .expect("create metadata dir");
        fs::write(&metadata, "{}\n").expect("write stale metadata");
        let ts_out = temp.path().join("generated/packages/jsr/demo");

        write_contract_shell_outputs(&ContractShellOutputPlan {
            contract_id: "demo@v1",
            artifact_version: "0.0.0-shell",
            out_api: Some(&manifest),
            ts_out: Some(&ts_out),
            npm_out: None,
            rust_out: None,
            package_name: "@trellis-sdk/demo",
            crate_name: "trellis_sdk_demo",
            runtime_source: RuntimeSource::Registry,
            runtime_repo_root: None,
        })
        .expect("write shell outputs");

        let shell_manifest = fs::read_to_string(ts_out.join("api.ts")).expect("read API shell");
        let deno = fs::read_to_string(ts_out.join("deno.json")).expect("read deno shell config");
        assert_eq!(
            fs::read_to_string(ts_out.join("descriptors.ts")).unwrap(),
            ""
        );
        assert_eq!(fs::read_to_string(ts_out.join("types.ts")).unwrap(), "");
        assert_eq!(fs::read_to_string(ts_out.join("schemas.ts")).unwrap(), "");
        assert!(shell_manifest.contains("API_ID"));
        assert!(shell_manifest.contains("API_DIGEST"));
        assert!(!ts_out.join("contract.ts").exists());
        assert!(ts_out.join("api.ts").exists());
        assert!(!ts_out.join("owned_api.ts").exists());
        assert!(!ts_out.join("client.ts").exists());
        assert!(deno.contains(r#""lib": ["#));
        assert!(deno.contains(r#""dom""#));
        assert!(deno.contains(r#""deno.ns""#));
        assert!(!metadata.exists());
    }

    #[test]
    fn contract_shell_outputs_create_rust_sdk_crate_shell() {
        let temp = tempfile::tempdir().expect("create tempdir");
        let rust_out = temp.path().join("generated/packages/cargo/demo");

        write_contract_shell_outputs(&ContractShellOutputPlan {
            contract_id: "demo@v1",
            artifact_version: "0.0.0-shell",
            out_api: None,
            ts_out: None,
            npm_out: None,
            rust_out: Some(&rust_out),
            package_name: "@trellis-sdk/demo",
            crate_name: "trellis_sdk_demo",
            runtime_source: RuntimeSource::Registry,
            runtime_repo_root: None,
        })
        .expect("write shell outputs");

        let cargo = fs::read_to_string(rust_out.join("Cargo.toml")).expect("read cargo shell");
        let lib = fs::read_to_string(rust_out.join("src/lib.rs")).expect("read lib shell");
        assert!(cargo.contains("name = \"trellis_sdk_demo\""));
        assert!(!cargo.contains("publish = false"));
        assert!(cargo.contains("trellis-contracts"));
        assert!(lib.contains("pub const API_ID: &str = \"demo@v1\""));
    }

    #[test]
    fn trellis_owned_contract_shell_outputs_non_publishable_rust_sdk_crate_shell() {
        let temp = tempfile::tempdir().expect("create tempdir");
        let rust_out = temp.path().join("generated/packages/cargo/trellis-core");

        write_contract_shell_outputs(&ContractShellOutputPlan {
            contract_id: "trellis.core@v1",
            artifact_version: "0.0.0-shell",
            out_api: None,
            ts_out: None,
            npm_out: None,
            rust_out: Some(&rust_out),
            package_name: "@trellis-sdk/core",
            crate_name: "trellis_sdk_core",
            runtime_source: RuntimeSource::Registry,
            runtime_repo_root: None,
        })
        .expect("write shell outputs");

        let cargo = fs::read_to_string(rust_out.join("Cargo.toml")).expect("read cargo shell");
        assert!(cargo.contains("publish = false"));
    }
}
