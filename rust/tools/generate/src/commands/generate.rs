use std::fs;

use miette::IntoDiagnostic;

use crate::artifacts::{
    build_npm_package_from_ts_sources, current_generator_fingerprints,
    default_rust_crate_name_from_id, format_generated_typescript_artifacts,
    generated_artifacts_are_fresh, generated_artifacts_metadata, infer_artifact_version,
    native_api_digest, native_api_json, resolve_contract, rust_runtime_deps, stage_npm_ts_sources,
    trellis_package_version, ts_package_name_from_id, ts_runtime_deps, write_contract_outputs,
    ContractOutputPlan, NpmPackageBuild, NpmPackageManifest,
};
use crate::cli::{
    GenerateAllArgs, GenerateApiArgs, GenerateCargoPackageArgs, GenerateJsrPackageArgs,
    GenerateNpmPackageArgs,
};
use crate::output;
use trellis_codegen_rust::GenerateRustSdkOpts;
use trellis_codegen_ts::GenerateTsSdkOpts;

pub fn api(args: &GenerateApiArgs) -> miette::Result<()> {
    let resolved = resolve_contract(&args.contract)?;
    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }
    fs::write(&args.out, format!("{}\n", native_api_json(&resolved)?)).into_diagnostic()?;
    output::print_success(&format!(
        "generated canonical API for {}",
        resolved.api.render_model.id
    ));
    output::print_detail("api", args.out.display().to_string());
    output::print_detail("digest", &native_api_digest(&resolved)?);
    Ok(())
}

pub fn jsr_package(args: &GenerateJsrPackageArgs) -> miette::Result<()> {
    let resolved = resolve_contract(&args.contract)?;
    let package_name = args
        .package_name
        .clone()
        .unwrap_or_else(|| ts_package_name_from_id(&resolved.api.render_model.id, &args.prefix));
    let artifact_version = infer_artifact_version(
        &resolved,
        args.artifact_version.clone(),
        "generate a JSR package",
    )?;
    trellis_codegen_ts::generate_ts_sdk(&GenerateTsSdkOpts {
        api_path: resolved.api_path.clone(),
        out_dir: args.out.clone(),
        package_name,
        package_version: artifact_version.clone(),
        runtime_deps: ts_runtime_deps(
            args.runtime_source,
            trellis_package_version(),
            args.runtime_repo_root.clone(),
        ),
    })
    .into_diagnostic()?;
    format_generated_typescript_artifacts(&args.out, args.runtime_repo_root.as_deref())?;
    output::print_success(&format!("generated JSR package at {}", args.out.display()));
    Ok(())
}

pub fn npm_package(args: &GenerateNpmPackageArgs) -> miette::Result<()> {
    let resolved = resolve_contract(&args.contract)?;
    let package_name = args
        .package_name
        .clone()
        .unwrap_or_else(|| ts_package_name_from_id(&resolved.api.render_model.id, &args.prefix));
    let artifact_version = infer_artifact_version(
        &resolved,
        args.artifact_version.clone(),
        "generate an npm package",
    )?;
    let staging = tempfile::tempdir().into_diagnostic()?;
    let npm_sources = stage_npm_ts_sources(
        &resolved.api.render_model.id,
        &resolved.api_path,
        staging.path(),
        &package_name,
        &artifact_version,
    )?;
    build_npm_package_from_ts_sources(&NpmPackageBuild {
        src_dir: &npm_sources.root_dir,
        npm_out: &args.out,
        manifest: NpmPackageManifest {
            package_name: &package_name,
            package_version: &artifact_version,
            trellis_runtime_version: &trellis_package_version(),
            contract_id: &resolved.api.render_model.id,
        },
        runtime_repo_root: None,
    })?;
    output::print_success(&format!("generated npm package at {}", args.out.display()));
    Ok(())
}

pub fn cargo_package(args: &GenerateCargoPackageArgs) -> miette::Result<()> {
    let resolved = resolve_contract(&args.contract)?;
    let crate_name = args
        .crate_name
        .clone()
        .unwrap_or_else(|| default_rust_crate_name_from_id(&resolved.api.render_model.id));
    let artifact_version = infer_artifact_version(
        &resolved,
        args.artifact_version.clone(),
        "generate a Cargo package",
    )?;
    trellis_codegen_rust::generate_rust_sdk(&GenerateRustSdkOpts {
        api_path: resolved.api_path.clone(),
        out_dir: args.out.clone(),
        crate_name,
        crate_version: artifact_version.clone(),
        runtime_deps: rust_runtime_deps(
            args.runtime_source,
            artifact_version,
            args.runtime_repo_root.clone(),
        ),
    })
    .into_diagnostic()?;
    crate::artifacts::copy_embedded_trellis_owned_rust_sdk(
        &resolved.api.render_model.id,
        &args.out,
        args.runtime_source,
        args.runtime_repo_root.as_deref(),
    )?;
    output::print_success(&format!(
        "generated Cargo package at {}",
        args.out.display()
    ));
    Ok(())
}

pub fn all(args: &GenerateAllArgs, force: bool) -> miette::Result<()> {
    let resolved = resolve_contract(&args.contract)?;
    let artifact_version = infer_artifact_version(
        &resolved,
        args.artifact_version.clone(),
        "generate all artifacts",
    )?;
    let package_name = args
        .package_name
        .clone()
        .unwrap_or_else(|| ts_package_name_from_id(&resolved.api.render_model.id, &args.prefix));
    let crate_name = args
        .crate_name
        .clone()
        .unwrap_or_else(|| default_rust_crate_name_from_id(&resolved.api.render_model.id));
    let fingerprints = current_generator_fingerprints();
    let output_plan = ContractOutputPlan {
        artifact_version: &artifact_version,
        runtime_version: &artifact_version,
        out_api: &args.out_api,
        ts_out: args.jsr_out.as_deref(),
        npm_out: args.npm_out.as_deref(),
        rust_out: args.cargo_out.as_deref(),
        package_name: &package_name,
        crate_name: &crate_name,
        runtime_source: args.runtime_source,
        runtime_repo_root: args.runtime_repo_root.as_deref(),
        fingerprints,
    };
    let metadata =
        generated_artifacts_metadata(&resolved, &native_api_digest(&resolved)?, &output_plan);
    let freshness = if force {
        Default::default()
    } else {
        generated_artifacts_are_fresh(
            &metadata,
            &args.out_api,
            args.jsr_out.as_deref(),
            args.npm_out.as_deref(),
            args.cargo_out.as_deref(),
        )
    };
    if freshness.all() {
        output::print_success(&format!(
            "artifacts already up to date for {}",
            resolved.api.render_model.id
        ));
        return Ok(());
    }
    write_contract_outputs(&resolved, &output_plan, freshness)?;
    crate::artifacts::write_generated_artifacts_metadata(&args.out_api, &metadata)
}
