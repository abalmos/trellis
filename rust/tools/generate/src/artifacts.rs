use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use trellis_codegen_rust::{
    default_sdk_stem, rust_sdk_cargo_manifest_is_valid, GenerateRustParticipantFacadeOpts,
    GenerateRustSdkOpts, ParticipantAliasMapping, RustRuntimeDeps,
    RustRuntimeSource as CodegenRustRuntimeSource,
};
use trellis_codegen_ts::{
    collect_ts_sdk_sources, GenerateTsSdkOpts, TsRuntimeDeps,
    TsRuntimeSource as CodegenTsRuntimeSource,
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
    pub api_digest: String,
    pub artifact_version: String,
    pub runtime_source: RuntimeSource,
    pub jsr_runtime_version: String,
    pub has_jsr_package: bool,
    pub has_npm_package: bool,
    pub has_cargo_package: bool,
    pub package_name: String,
    pub crate_name: String,
    pub generator_fingerprint: String,
}

pub struct NpmTsSources {
    pub root_dir: PathBuf,
    pub dependency_packages: BTreeSet<String>,
}

impl GeneratedArtifactsMetadata {
    const SCHEMA_VERSION: u8 = 2;
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

#[expect(
    clippy::too_many_arguments,
    reason = "the output matrix is the generator command boundary"
)]
pub fn write_contract_outputs(
    resolved: &ResolvedNativeInput,
    artifact_version: String,
    out_api: &Path,
    ts_out: Option<&Path>,
    npm_out: Option<&Path>,
    rust_out: Option<&Path>,
    package_name: &str,
    crate_name: &str,
    runtime_source: RuntimeSource,
    runtime_repo_root: Option<PathBuf>,
    generator_fingerprint: &str,
    success_message: &str,
) -> miette::Result<()> {
    let api = native_api_artifact(resolved)?;
    let metadata = generated_artifacts_metadata(
        resolved,
        &api.digest,
        &artifact_version,
        runtime_source,
        &trellis_package_version(),
        ts_out.is_some(),
        npm_out.is_some(),
        rust_out.is_some(),
        package_name,
        crate_name,
        generator_fingerprint,
    );
    if let Some(parent) = out_api.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }
    write_if_changed(out_api, &format!("{}\n", api.json))?;

    if let Some(ts_out) = ts_out {
        trellis_codegen_ts::generate_ts_sdk(&GenerateTsSdkOpts {
            api_path: out_api.to_path_buf(),
            out_dir: ts_out.to_path_buf(),
            package_name: package_name.to_string(),
            package_version: artifact_version.clone(),
            runtime_deps: ts_runtime_deps(
                runtime_source,
                trellis_package_version(),
                runtime_repo_root.clone(),
            ),
        })
        .into_diagnostic()?;
        rewrite_local_generated_ts_sdk_imports(
            ts_out,
            runtime_source,
            runtime_repo_root.as_deref(),
        )?;
        format_generated_typescript_artifacts(ts_out, runtime_repo_root.as_deref())?;
        copy_embedded_trellis_owned_ts_sdk(
            &resolved.api.render_model.id,
            ts_out,
            runtime_source,
            runtime_repo_root.as_deref(),
        )?;
    }

    if let Some(npm_out) = npm_out {
        let staging_dir = tempfile::tempdir().into_diagnostic()?;
        let npm_sources = stage_npm_ts_sources(
            &resolved.api.render_model.id,
            out_api,
            staging_dir.path(),
            package_name,
            &artifact_version,
        )?;
        build_npm_package_from_ts_sources(
            &npm_sources.root_dir,
            npm_out,
            package_name,
            &artifact_version,
            &trellis_package_version(),
            &resolved.api.render_model.id,
            &npm_sources.dependency_packages,
            runtime_repo_root.as_deref(),
        )?;
    }

    if let Some(rust_out) = rust_out {
        trellis_codegen_rust::generate_rust_sdk(&GenerateRustSdkOpts {
            api_path: out_api.to_path_buf(),
            out_dir: rust_out.to_path_buf(),
            crate_name: crate_name.to_string(),
            crate_version: artifact_version.clone(),
            runtime_deps: rust_runtime_deps(
                runtime_source,
                artifact_version.clone(),
                runtime_repo_root.clone(),
            ),
        })
        .into_diagnostic()?;
        copy_embedded_trellis_owned_rust_sdk(
            &resolved.api.render_model.id,
            rust_out,
            runtime_source,
            runtime_repo_root.as_deref(),
        )?;
    }

    write_generated_artifacts_metadata(out_api, &metadata)?;

    output::print_success(&format!(
        "{} for {}",
        success_message, resolved.api.render_model.id
    ));
    output::print_detail("api", out_api.display().to_string());
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

#[expect(
    clippy::too_many_arguments,
    reason = "shell generation mirrors the optional output matrix"
)]
pub fn write_contract_shell_outputs(
    contract_id: &str,
    artifact_version: &str,
    out_api: Option<&Path>,
    ts_out: Option<&Path>,
    npm_out: Option<&Path>,
    rust_out: Option<&Path>,
    package_name: &str,
    crate_name: &str,
    runtime_source: RuntimeSource,
    runtime_repo_root: Option<PathBuf>,
) -> miette::Result<()> {
    if let Some(ts_out) = ts_out {
        write_ts_sdk_shell(
            contract_id,
            artifact_version,
            ts_out,
            package_name,
            runtime_source,
            runtime_repo_root.clone(),
        )?;
    }

    if let Some(npm_out) = npm_out {
        write_npm_package_shell(npm_out, package_name, artifact_version)?;
    }

    if let Some(rust_out) = rust_out {
        write_rust_sdk_shell(
            contract_id,
            artifact_version,
            rust_out,
            crate_name,
            runtime_source,
            runtime_repo_root,
        )?;
    }

    if let Some(out_api) = out_api {
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
    runtime_source: RuntimeSource,
    runtime_repo_root: Option<&Path>,
) -> miette::Result<()> {
    if !matches!(runtime_source, RuntimeSource::Local) {
        return Ok(());
    }

    let repo_root = runtime_repo_root.ok_or_else(|| {
        miette::miette!("local generated TypeScript imports require a runtime repository root")
    })?;
    let depth = ts_out
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

#[expect(
    clippy::too_many_arguments,
    reason = "participant generation receives the resolved CLI output settings"
)]
pub fn write_participant_facade_outputs(
    api_path: &Path,
    participant_path: &Path,
    protocol_participant_out: &Path,
    rust_participant_out: &Path,
    crate_name: &str,
    crate_version: &str,
    runtime_source: RuntimeSource,
    runtime_repo_root: Option<PathBuf>,
    owned_sdk_crate_name: Option<String>,
    owned_sdk_path: Option<PathBuf>,
    alias_mappings: Vec<ParticipantAliasMapping>,
) -> miette::Result<()> {
    let (participant_json, participant_digest) = trellis_codegen_rust::native_participant_artifact(
        api_path,
        participant_path,
        &alias_mappings,
    )
    .into_diagnostic()?;
    if let Some(parent) = protocol_participant_out.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }
    write_if_changed(protocol_participant_out, &format!("{participant_json}\n"))?;
    trellis_codegen_rust::generate_rust_participant_facade(&GenerateRustParticipantFacadeOpts {
        api_path: api_path.to_path_buf(),
        participant_path: participant_path.to_path_buf(),
        out_dir: rust_participant_out.to_path_buf(),
        crate_name: crate_name.to_string(),
        crate_version: crate_version.to_string(),
        runtime_deps: rust_runtime_deps(
            runtime_source,
            crate_version.to_string(),
            runtime_repo_root,
        ),
        owned_sdk_crate_name,
        owned_sdk_path,
        alias_mappings,
    })
    .into_diagnostic()?;
    output::print_detail(
        "rust participant",
        rust_participant_out.display().to_string(),
    );
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

#[expect(
    clippy::too_many_arguments,
    reason = "npm package assembly receives one value per package manifest field"
)]
pub fn build_npm_package_from_ts_sources(
    src_dir: &Path,
    npm_out: &Path,
    package_name: &str,
    package_version: &str,
    trellis_runtime_version: &str,
    contract_id: &str,
    dependency_packages: &BTreeSet<String>,
    runtime_repo_root: Option<&Path>,
) -> miette::Result<()> {
    let npm_out = if npm_out.is_absolute() {
        npm_out.to_path_buf()
    } else {
        std::env::current_dir().into_diagnostic()?.join(npm_out)
    };
    if npm_out.exists() {
        fs::remove_dir_all(&npm_out).into_diagnostic()?;
    }
    fs::create_dir_all(&npm_out).into_diagnostic()?;
    let esm_dir = npm_out.join("esm");
    write_if_changed(
        &src_dir.join("tsconfig.json"),
        &render_npm_tsconfig(&npm_out),
    )?;
    write_if_changed(
        &src_dir.join("package.json"),
        "{\n  \"type\": \"module\"\n}\n",
    )?;
    let tsc = resolve_tsc_bin()?;
    let output = Command::new(&tsc)
        .arg("-p")
        .arg(src_dir.join("tsconfig.json"))
        .current_dir(src_dir)
        .output()
        .into_diagnostic()?;
    if !output.status.success() {
        return Err(miette::miette!(
            "npm package TypeScript build failed for {} using {:?}\nstdout:\n{}\nstderr:\n{}",
            package_name,
            tsc,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    write_npm_package_json(
        &npm_out,
        package_name,
        package_version,
        trellis_runtime_version,
        contract_id,
        dependency_packages,
    )?;
    if let Ok(readme) = fs::read_to_string(src_dir.join("README.md")) {
        write_if_changed(&npm_out.join("README.md"), &readme)?;
    }
    if !esm_dir.join("mod.js").exists() || !esm_dir.join("mod.d.ts").exists() {
        return Err(miette::miette!(
            "npm package TypeScript build for {} did not emit esm/mod.js and esm/mod.d.ts",
            package_name
        ));
    }
    format_generated_npm_artifacts(&npm_out, runtime_repo_root)?;
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
    for source in collect_ts_sdk_sources(&opts).into_diagnostic()? {
        if source
            .path
            .extension()
            .is_some_and(|extension| extension == "ts")
        {
            write_if_changed(
                &root_dir.join(source.path),
                &rewrite_npm_ts_imports(&source.contents),
            )?;
        } else if source.path == Path::new("README.md") {
            write_if_changed(&root_dir.join(source.path), &source.contents)?;
        }
    }
    Ok(NpmTsSources {
        root_dir,
        dependency_packages: BTreeSet::new(),
    })
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

fn write_npm_package_json(
    npm_out: &Path,
    package_name: &str,
    package_version: &str,
    trellis_runtime_version: &str,
    contract_id: &str,
    _dependency_packages: &BTreeSet<String>,
) -> miette::Result<()> {
    let trellis_dependency = format!("^{}", trellis_runtime_version);
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
                "name": package_name,
                "version": package_version,
                "description": format!("Generated Trellis SDK for contract {contract_id}"),
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
    Command::new(binary).arg("--version").output().is_ok()
}

#[expect(
    clippy::too_many_arguments,
    reason = "metadata construction records the complete generated output matrix"
)]
pub fn generated_artifacts_metadata(
    resolved: &ResolvedNativeInput,
    api_digest: &str,
    artifact_version: &str,
    runtime_source: RuntimeSource,
    jsr_runtime_version: &str,
    has_jsr_package: bool,
    has_npm_package: bool,
    has_cargo_package: bool,
    package_name: &str,
    crate_name: &str,
    generator_fingerprint: &str,
) -> GeneratedArtifactsMetadata {
    GeneratedArtifactsMetadata {
        schema_version: GeneratedArtifactsMetadata::SCHEMA_VERSION,
        contract_id: resolved.api.render_model.id.clone(),
        api_digest: api_digest.to_owned(),
        artifact_version: artifact_version.to_string(),
        runtime_source,
        jsr_runtime_version: jsr_runtime_version.to_string(),
        has_jsr_package,
        has_npm_package,
        has_cargo_package,
        package_name: package_name.to_string(),
        crate_name: crate_name.to_string(),
        generator_fingerprint: generator_fingerprint.to_string(),
    }
}

pub fn generated_artifacts_are_fresh(
    expected: &GeneratedArtifactsMetadata,
    out_api: &Path,
    ts_out: Option<&Path>,
    npm_out: Option<&Path>,
    rust_out: Option<&Path>,
) -> bool {
    let Some(existing) = read_generated_artifacts_metadata(out_api) else {
        return false;
    };
    existing == *expected
        && out_api.exists()
        && ts_key_outputs_exist(ts_out)
        && embedded_trellis_owned_ts_sdk_key_outputs_exist(expected, out_api)
        && npm_key_outputs_exist(npm_out)
        && rust_key_outputs_exist(rust_out, expected, out_api)
}

fn read_generated_artifacts_metadata(out_api: &Path) -> Option<GeneratedArtifactsMetadata> {
    let contents = fs::read_to_string(generated_artifacts_metadata_path(out_api)).ok()?;
    serde_json::from_str(&contents).ok()
}

fn write_generated_artifacts_metadata(
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
        let rewritten = rewrite_embedded_rust_sdk_source(&contents, is_root, module);
        let formatted = format_embedded_rust_sdk_source(&dest_path, &rewritten)?;
        write_if_changed(&dest_path, &formatted)?;
    }
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

fn rewrite_embedded_rust_sdk_source(contents: &str, is_root: bool, module: &str) -> String {
    let rewritten = if is_root {
        contents.replace("crate::", "self::")
    } else {
        contents.replace("crate::", "super::")
    };
    let rewritten = rewritten
        .replace("trellis_rs::", "crate::")
        .replace("trellis_client::", "crate::client::")
        .replace("trellis_contracts::", "crate::contracts::");
    if is_root && module == "jobs" {
        rewritten.replace(
            "/// Job descriptors.\npub mod jobs;",
            "/// Job descriptors.\n#[expect(\n    clippy::module_inception,\n    reason = \"generated SDK modules mirror contract surface names\"\n)]\npub mod jobs;",
        )
    } else {
        rewritten
    }
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

    let mut command = Command::new("deno");
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
    if !npm_out.exists() {
        return Ok(());
    }

    for path in npm_format_paths(npm_out)? {
        let Some(ext) = npm_format_extension(&path) else {
            continue;
        };
        let contents = fs::read_to_string(&path).into_diagnostic()?;
        let formatted = format_text_with_deno(&path, &config, ext, &contents)?;
        write_if_changed(&path, &formatted)?;
    }
    Ok(())
}

fn npm_format_paths(root: &Path) -> miette::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_npm_format_paths(root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_npm_format_paths(dir: &Path, paths: &mut Vec<PathBuf>) -> miette::Result<()> {
    for entry in fs::read_dir(dir).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();
        if path.is_dir() {
            collect_npm_format_paths(&path, paths)?;
        } else if npm_format_extension(&path).is_some() {
            paths.push(path);
        }
    }
    Ok(())
}

fn npm_format_extension(path: &Path) -> Option<&'static str> {
    let extension = path.extension()?.to_str()?;
    match extension {
        // Generated npm .js files can still contain type-only syntax, so format
        // them with the TypeScript parser rather than Deno's stricter JS parser.
        "js" | "ts" => Some("ts"),
        "json" => Some("json"),
        "md" => Some("md"),
        _ => None,
    }
}

fn format_text_with_deno(
    path: &Path,
    config: &Path,
    extension: &str,
    contents: &str,
) -> miette::Result<String> {
    let mut child = Command::new("deno")
        .arg("fmt")
        .arg("--quiet")
        .arg("-c")
        .arg(config)
        .arg("--ext")
        .arg(extension)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .into_diagnostic()?;

    child
        .stdin
        .take()
        .expect("deno fmt stdin should be piped")
        .write_all(contents.as_bytes())
        .into_diagnostic()?;

    let output = child.wait_with_output().into_diagnostic()?;
    if !output.status.success() {
        return Err(miette::miette!(
            "deno fmt failed for {}\nstderr:\n{}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn format_embedded_rust_sdk_source(path: &Path, contents: &str) -> miette::Result<String> {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2021"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .into_diagnostic()?;

    child
        .stdin
        .take()
        .expect("rustfmt stdin should be piped")
        .write_all(contents.as_bytes())
        .into_diagnostic()?;

    let output = child.wait_with_output().into_diagnostic()?;
    if !output.status.success() {
        return Err(miette::miette!(
            "rustfmt failed for {}\nstderr:\n{}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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

pub fn current_generator_fingerprint() -> &'static str {
    env!("TRELLIS_GENERATE_FINGERPRINT")
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
        find_tsc_in_node_modules, format_embedded_rust_sdk_source,
        generated_artifacts_metadata_path, render_npm_tsconfig, rewrite_embedded_rust_sdk_source,
        rewrite_embedded_trellis_owned_ts_sdk_source, rewrite_npm_ts_imports,
        trellis_package_version, ts_package_name_from_id, write_contract_shell_outputs,
    };

    #[test]
    fn generated_ts_package_names_use_private_default_namespace() {
        assert_eq!(
            ts_package_name_from_id("trellis.demo-service@v1", "@trellis-sdk/"),
            "@trellis-sdk/trellis-demo-service",
        );
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
            "core",
        );
        let formatted = format_embedded_rust_sdk_source(&dest, &rewritten)
            .expect("format rewritten Rust SDK source");

        assert_eq!(
            formatted,
            "use crate::service::OperationFailureLike;\npub fn client() -> crate::client::Result<()> {\n    todo!()\n}\n"
        );
    }

    #[test]
    fn embedded_jobs_sdk_preserves_module_inception_expectation() {
        let rewritten =
            rewrite_embedded_rust_sdk_source("/// Job descriptors.\npub mod jobs;\n", true, "jobs");

        assert!(rewritten.contains("#[expect(\n    clippy::module_inception,"));
        assert!(rewritten.contains("pub mod jobs;"));
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

        write_contract_shell_outputs(
            "demo@v1",
            "0.0.0-shell",
            Some(&manifest),
            Some(&ts_out),
            None,
            None,
            "@trellis-sdk/demo",
            "trellis_sdk_demo",
            RuntimeSource::Registry,
            None,
        )
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

        write_contract_shell_outputs(
            "demo@v1",
            "0.0.0-shell",
            None,
            None,
            None,
            Some(&rust_out),
            "@trellis-sdk/demo",
            "trellis_sdk_demo",
            RuntimeSource::Registry,
            None,
        )
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

        write_contract_shell_outputs(
            "trellis.core@v1",
            "0.0.0-shell",
            None,
            None,
            None,
            Some(&rust_out),
            "@trellis-sdk/core",
            "trellis_sdk_core",
            RuntimeSource::Registry,
            None,
        )
        .expect("write shell outputs");

        let cargo = fs::read_to_string(rust_out.join("Cargo.toml")).expect("read cargo shell");
        assert!(cargo.contains("publish = false"));
    }
}
