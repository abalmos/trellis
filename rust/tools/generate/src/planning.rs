use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use miette::IntoDiagnostic;
use serde_json::Value;
use trellis_contracts::ContractKind;

use crate::artifacts::{
    current_generator_fingerprint, default_rust_crate_name_from_id, detect_output_root,
    detect_runtime_source, generated_artifacts_are_fresh, generated_artifacts_metadata,
    generated_artifacts_metadata_path, required_owner_version, sdk_output_stem,
    trellis_package_version, ts_package_name_from_id, write_contract_outputs,
    write_contract_shell_outputs, write_participant_facade_outputs,
};
use crate::cli::{PackageTarget, RuntimeSource};
use crate::contract_input;
use crate::discovery::{discover_contract_metadata, DiscoveredContractSource, SourceLanguage};
use crate::output;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoAction {
    Generate,
    Verify,
}

#[derive(Debug, Clone)]
pub struct AutoPlanEntry {
    pub discovered: DiscoveredContractSource,
    pub contract_id: String,
    pub contract_kind: ContractKind,
    pub action: AutoAction,
    pub out_manifest: Option<PathBuf>,
    pub jsr_out: Option<PathBuf>,
    pub npm_out: Option<PathBuf>,
    pub cargo_out: Option<PathBuf>,
    pub cargo_participant_out: Option<PathBuf>,
    pub runtime_source: RuntimeSource,
    pub runtime_repo_root: Option<PathBuf>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AutoExecutionSummary {
    pub generated: usize,
    pub verified: usize,
    pub skipped: usize,
}

pub fn build_auto_plan(
    discovered: Vec<DiscoveredContractSource>,
    shared_output_root: Option<&Path>,
    prefix: &str,
) -> miette::Result<Vec<AutoPlanEntry>> {
    build_auto_plan_with_targets(discovered, shared_output_root, prefix, None)
}

pub fn build_auto_plan_with_targets(
    discovered: Vec<DiscoveredContractSource>,
    shared_output_root: Option<&Path>,
    prefix: &str,
    requested_targets: Option<&[PackageTarget]>,
) -> miette::Result<Vec<AutoPlanEntry>> {
    let mut plan = Vec::new();
    for contract in discovered {
        let (contract_id, contract_kind) = discover_contract_metadata(&contract)?;
        let action = action_for_discovered_kind(&contract, &contract_kind);
        let output_root = shared_output_root
            .map(Path::to_path_buf)
            .unwrap_or_else(|| detect_output_root(&contract.project_root));
        let runtime_repo_root = detect_runtime_repo_root(&output_root);
        let runtime_source = if runtime_repo_root.is_some() {
            RuntimeSource::Local
        } else {
            RuntimeSource::Registry
        };
        let (out_manifest, jsr_out, npm_out, cargo_out, cargo_participant_out) = match action {
            AutoAction::Generate => {
                let sdk_stem = sdk_output_stem(&contract_id);
                let targets = targets_for_entry(&contract, &contract_kind, requested_targets);
                let jsr_package_root = resolve_jsr_package_root(
                    &output_root,
                    &contract.project_root,
                    runtime_repo_root.as_deref(),
                );
                let out_manifest = if !targets.is_empty() {
                    Some(
                        output_root
                            .join(if contract.language == SourceLanguage::Protocol {
                                "generated/apis"
                            } else {
                                "generated/contracts/manifests"
                            })
                            .join(format!("{}.json", contract_id)),
                    )
                } else {
                    None
                };
                let jsr_out = if targets.contains(&PackageTarget::Jsr) {
                    Some(jsr_package_root.join(&sdk_stem))
                } else {
                    None
                };
                let npm_out = if targets.contains(&PackageTarget::Npm) {
                    Some(output_root.join("generated/packages/npm").join(&sdk_stem))
                } else {
                    None
                };
                let cargo_out = if targets.contains(&PackageTarget::Cargo) {
                    Some(output_root.join("generated/packages/cargo").join(&sdk_stem))
                } else {
                    None
                };
                let cargo_participant_out =
                    if matches!(contract.language, crate::discovery::SourceLanguage::Rust) {
                        Some(
                            output_root
                                .join("generated/packages/cargo-participants")
                                .join(&sdk_stem),
                        )
                    } else {
                        None
                    };
                (
                    out_manifest,
                    jsr_out,
                    npm_out,
                    cargo_out,
                    cargo_participant_out,
                )
            }
            AutoAction::Verify => (None, None, None, None, None),
        };
        let action = if matches!(action, AutoAction::Generate)
            && out_manifest.is_none()
            && jsr_out.is_none()
            && npm_out.is_none()
            && cargo_out.is_none()
            && cargo_participant_out.is_none()
        {
            AutoAction::Verify
        } else {
            action
        };
        plan.push(AutoPlanEntry {
            discovered: contract,
            contract_id,
            contract_kind,
            action,
            out_manifest,
            jsr_out,
            npm_out,
            cargo_out,
            cargo_participant_out,
            runtime_source,
            runtime_repo_root,
        });
    }
    sort_auto_plan(&mut plan, prefix);
    Ok(plan)
}

fn targets_for_entry(
    contract: &DiscoveredContractSource,
    kind: &ContractKind,
    requested_targets: Option<&[PackageTarget]>,
) -> Vec<PackageTarget> {
    let defaults = if matches!(kind, ContractKind::Service) {
        vec![
            PackageTarget::Manifest,
            PackageTarget::Jsr,
            PackageTarget::Npm,
            PackageTarget::Cargo,
        ]
    } else if matches!(kind, ContractKind::App)
        && !matches!(contract.language, crate::discovery::SourceLanguage::Rust)
    {
        vec![
            PackageTarget::Manifest,
            PackageTarget::Jsr,
            PackageTarget::Npm,
        ]
    } else if matches!(contract.language, crate::discovery::SourceLanguage::Rust) {
        vec![PackageTarget::Manifest, PackageTarget::Cargo]
    } else {
        Vec::new()
    };

    match requested_targets {
        Some(targets) => defaults
            .into_iter()
            .filter(|target| targets.contains(target))
            .collect(),
        None => defaults,
    }
}

fn sort_auto_plan(plan: &mut Vec<AutoPlanEntry>, prefix: &str) {
    let mut remaining = plan.clone();
    remaining.sort_by(compare_auto_plan_entries);

    let mut sorted = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let next = remaining
            .iter()
            .position(|entry| {
                local_jsr_package_dependencies(entry, plan, prefix)
                    .into_iter()
                    .all(|dependency| {
                        sorted
                            .iter()
                            .any(|candidate: &AutoPlanEntry| candidate.contract_id == dependency)
                            || !remaining
                                .iter()
                                .any(|candidate| candidate.contract_id == dependency)
                    })
            })
            .unwrap_or(0);
        sorted.push(remaining.remove(next));
    }

    *plan = sorted;
}

fn compare_auto_plan_entries(left: &AutoPlanEntry, right: &AutoPlanEntry) -> std::cmp::Ordering {
    auto_plan_rank(left)
        .cmp(&auto_plan_rank(right))
        .then_with(|| {
            left.discovered
                .source_path
                .cmp(&right.discovered.source_path)
        })
}

fn local_jsr_package_dependencies(
    entry: &AutoPlanEntry,
    plan: &[AutoPlanEntry],
    prefix: &str,
) -> Vec<String> {
    if entry.discovered.language != crate::discovery::SourceLanguage::TypeScript {
        return Vec::new();
    }
    let Ok(source) = fs::read_to_string(&entry.discovered.source_path) else {
        return Vec::new();
    };

    plan.iter()
        .filter(|candidate| candidate.contract_id != entry.contract_id)
        .filter(|candidate| candidate.jsr_out.is_some())
        .filter_map(|candidate| {
            let package_name = ts_package_name_from_id(&candidate.contract_id, prefix);
            source_imports_specifier(&source, &package_name).then(|| candidate.contract_id.clone())
        })
        .collect()
}

fn source_imports_specifier(source: &str, specifier: &str) -> bool {
    let double_quoted = format!("from \"{specifier}\"");
    let single_quoted = format!("from '{specifier}'");
    let dynamic_double_quoted = format!("import(\"{specifier}\")");
    let dynamic_single_quoted = format!("import('{specifier}')");
    source.contains(&double_quoted)
        || source.contains(&single_quoted)
        || source.contains(&dynamic_double_quoted)
        || source.contains(&dynamic_single_quoted)
}

fn resolve_jsr_package_root(
    output_root: &Path,
    project_root: &Path,
    runtime_repo_root: Option<&Path>,
) -> PathBuf {
    if runtime_repo_root == Some(output_root) {
        return output_root.join("generated/packages/jsr");
    }

    find_nested_workspace_root(project_root, output_root)
        .map(|workspace_root| workspace_root.join("generated/packages/jsr"))
        .unwrap_or_else(|| output_root.join("generated/packages/jsr"))
}

fn detect_runtime_repo_root(output_root: &Path) -> Option<PathBuf> {
    if matches!(detect_runtime_source(output_root), RuntimeSource::Local) {
        return Some(output_root.to_path_buf());
    }

    let mut current = output_root.parent();
    while let Some(dir) = current {
        if matches!(detect_runtime_source(dir), RuntimeSource::Local) {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn find_nested_workspace_root(project_root: &Path, output_root: &Path) -> Option<PathBuf> {
    let mut current = Some(project_root);
    while let Some(dir) = current {
        if !dir.starts_with(output_root) {
            break;
        }
        if dir != output_root && has_workspace_manifest(dir) {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn has_workspace_manifest(dir: &Path) -> bool {
    ["deno.json", "deno.jsonc", "package.json"]
        .into_iter()
        .any(|name| manifest_declares_workspace(&dir.join(name)))
}

fn manifest_declares_workspace(path: &Path) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<Value>(&contents) else {
        return false;
    };

    value.get("workspace").map(Value::is_array).unwrap_or(false)
        || value
            .get("workspaces")
            .map(|workspaces| workspaces.is_array() || workspaces.is_object())
            .unwrap_or(false)
}

pub fn execute_auto_plan(
    plan: &[AutoPlanEntry],
    title: Option<&str>,
    show_title: bool,
    force: bool,
    prefix: &str,
) -> miette::Result<AutoExecutionSummary> {
    if show_title {
        output::print_section("Run");
    } else if let Some(title) = title {
        output::print_title(title);
    }

    let generator_fingerprint = current_generator_fingerprint();
    let mut summary = AutoExecutionSummary::default();
    write_auto_plan_shells(plan, prefix, generator_fingerprint)?;
    for entry in plan {
        let resolved = contract_input::resolve_contract_input(
            None,
            Some(entry.discovered.source_path.as_path()),
            None,
            "CONTRACT",
            contract_input::default_image_contract_path(),
        )?;
        contract_input::warn_forward_incompatible_public_schemas(&resolved.loaded);
        match entry.action {
            AutoAction::Generate => {
                let artifact_version = required_owner_version(
                    &resolved,
                    "generate contract artifacts from local discovery",
                )?;
                let package_name = ts_package_name_from_id(&resolved.loaded.manifest.id, prefix);
                let crate_name = default_rust_crate_name_from_id(&resolved.loaded.manifest.id);
                let out_manifest = entry.out_manifest.as_ref().ok_or_else(|| {
                    miette::miette!("missing manifest output for generated contract")
                })?;
                let metadata = generated_artifacts_metadata(
                    &resolved,
                    &artifact_version,
                    entry.runtime_source,
                    &trellis_package_version(),
                    entry.jsr_out.is_some(),
                    entry.npm_out.is_some(),
                    entry.cargo_out.is_some(),
                    &package_name,
                    &crate_name,
                    generator_fingerprint,
                );
                if !force
                    && entry.cargo_participant_out.is_none()
                    && generated_artifacts_are_fresh(
                        &metadata,
                        out_manifest,
                        entry.jsr_out.as_deref(),
                        entry.npm_out.as_deref(),
                        entry.cargo_out.as_deref(),
                    )
                {
                    output::print_success(&format!(
                        "artifacts already up to date for {}",
                        resolved.loaded.manifest.id
                    ));
                    summary.skipped += 1;
                    continue;
                }
                print_auto_entry(entry);
                write_contract_outputs(
                    &resolved,
                    artifact_version.clone(),
                    out_manifest,
                    entry.jsr_out.as_deref(),
                    entry.npm_out.as_deref(),
                    entry.cargo_out.as_deref(),
                    &package_name,
                    &crate_name,
                    entry.runtime_source,
                    entry.runtime_repo_root.clone(),
                    generator_fingerprint,
                    "generated contract artifacts",
                )?;
                if entry.discovered.language == SourceLanguage::Protocol {
                    let generated_root = out_manifest
                        .parent()
                        .and_then(Path::parent)
                        .ok_or_else(|| miette::miette!("API output has no generated root"))?;
                    let legacy_manifest = generated_root
                        .join("contracts/manifests")
                        .join(format!("{}.json", resolved.loaded.manifest.id));
                    if legacy_manifest.exists() {
                        fs::remove_file(&legacy_manifest).into_diagnostic()?;
                    }
                }
                if let Some(cargo_participant_out) = &entry.cargo_participant_out {
                    match participant_alias_mappings(entry, plan) {
                        Ok(mappings) => write_participant_facade_outputs(
                            out_manifest,
                            cargo_participant_out,
                            &format!(
                                "trellis-participant-{}",
                                sdk_output_stem(&resolved.loaded.manifest.id)
                            ),
                            &artifact_version,
                            entry.runtime_source,
                            entry.runtime_repo_root.clone(),
                            Some(crate_name.clone()),
                            entry.cargo_out.clone(),
                            mappings,
                        )?,
                        Err(error) => {
                            remove_stale_participant_facade_outputs(cargo_participant_out)?;
                            return Err(error);
                        }
                    }
                }
                summary.generated += 1;
            }
            AutoAction::Verify => {
                if show_title {
                    print_auto_entry(entry);
                }
                output::print_success(&format!("verified {}", resolved.loaded.manifest.id));
                summary.verified += 1;
            }
        }
    }
    Ok(summary)
}

fn write_auto_plan_shells(
    plan: &[AutoPlanEntry],
    prefix: &str,
    generator_fingerprint: &str,
) -> miette::Result<()> {
    for entry in plan {
        if !matches!(entry.action, AutoAction::Generate) {
            continue;
        }
        if shell_outputs_are_not_needed(entry, generator_fingerprint) {
            continue;
        }
        let package_name = ts_package_name_from_id(&entry.contract_id, prefix);
        let crate_name = default_rust_crate_name_from_id(&entry.contract_id);
        write_contract_shell_outputs(
            &entry.contract_id,
            "0.0.0-shell",
            entry.out_manifest.as_deref(),
            entry.jsr_out.as_deref(),
            entry.npm_out.as_deref(),
            entry.cargo_out.as_deref(),
            &package_name,
            &crate_name,
            entry.runtime_source,
            entry.runtime_repo_root.clone(),
        )?;
    }
    Ok(())
}

fn shell_outputs_are_not_needed(entry: &AutoPlanEntry, generator_fingerprint: &str) -> bool {
    let Some(out_manifest) = &entry.out_manifest else {
        return false;
    };
    generated_artifacts_metadata_matches_generator(out_manifest, generator_fingerprint)
        && ts_shell_key_outputs_exist(entry.jsr_out.as_deref())
        && npm_shell_key_outputs_exist(entry.npm_out.as_deref())
        && rust_shell_key_outputs_exist(entry.cargo_out.as_deref())
}

fn generated_artifacts_metadata_matches_generator(
    out_manifest: &Path,
    generator_fingerprint: &str,
) -> bool {
    let Ok(contents) = fs::read_to_string(generated_artifacts_metadata_path(out_manifest)) else {
        return false;
    };
    let Ok(metadata) = serde_json::from_str::<Value>(&contents) else {
        return false;
    };
    metadata
        .get("generator_fingerprint")
        .and_then(Value::as_str)
        == Some(generator_fingerprint)
}

fn ts_shell_key_outputs_exist(ts_out: Option<&Path>) -> bool {
    let Some(ts_out) = ts_out else {
        return true;
    };
    ts_out.join("mod.ts").exists()
        && ts_out.join("descriptors.ts").exists()
        && ts_out.join("types.ts").exists()
        && ts_out.join("schemas.ts").exists()
        && ts_out.join("manifest.ts").exists()
}

fn npm_shell_key_outputs_exist(npm_out: Option<&Path>) -> bool {
    let Some(npm_out) = npm_out else {
        return true;
    };
    npm_out.join("package.json").exists()
}

fn rust_shell_key_outputs_exist(rust_out: Option<&Path>) -> bool {
    let Some(rust_out) = rust_out else {
        return true;
    };
    rust_out.join("Cargo.toml").exists() && rust_out.join("src/lib.rs").exists()
}

pub fn discover_summary_lines(plan: &[AutoPlanEntry]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_project_root: Option<&Path> = None;
    for entry in plan {
        let project_root = entry.discovered.project_root.as_path();
        if current_project_root != Some(project_root) {
            current_project_root = Some(project_root);
            lines.push(project_root.display().to_string());
        }
        lines.push(format!(
            "  {}  {}  {}",
            entry.contract_id,
            contract_kind_label(&entry.contract_kind),
            action_label(entry.action)
        ));
    }
    lines
}

pub fn action_for_kind(kind: &ContractKind) -> AutoAction {
    #[allow(unreachable_patterns)]
    match kind {
        ContractKind::Service | ContractKind::App => AutoAction::Generate,
        ContractKind::Device | ContractKind::Agent => AutoAction::Verify,
        _ => unreachable!("portal contract kind has been removed"),
    }
}

fn action_for_discovered_kind(
    contract: &DiscoveredContractSource,
    kind: &ContractKind,
) -> AutoAction {
    if matches!(contract.language, crate::discovery::SourceLanguage::Rust)
        && matches!(kind, ContractKind::Device | ContractKind::Agent)
    {
        AutoAction::Generate
    } else {
        action_for_kind(kind)
    }
}

fn participant_alias_mappings(
    entry: &AutoPlanEntry,
    plan: &[AutoPlanEntry],
) -> miette::Result<Vec<trellis_codegen_rust::ParticipantAliasMapping>> {
    let local_manifest = entry
        .out_manifest
        .as_ref()
        .ok_or_else(|| miette::miette!("missing participant manifest output"))?;
    let loaded = trellis_contracts::load_manifest(local_manifest).into_diagnostic()?;
    let mut mappings = Vec::new();
    for (alias, use_ref) in loaded.manifest.uses.iter() {
        if let Some(mapped) = plan.iter().find(|candidate| {
            candidate.contract_id == use_ref.contract && candidate.cargo_out.is_some()
        }) {
            let manifest_path = mapped
                .out_manifest
                .as_ref()
                .ok_or_else(|| {
                    miette::miette!("missing mapped manifest for {}", mapped.contract_id)
                })?
                .clone();
            mappings.push(trellis_codegen_rust::ParticipantAliasMapping {
                alias: alias.clone(),
                crate_name: default_rust_crate_name_from_id(&mapped.contract_id),
                manifest_path,
                crate_path: mapped.cargo_out.clone(),
                cargo_dependency: None,
            });
            continue;
        }

        if !trellis_codegen_rust::participant_use_requires_mapping(&loaded, alias, use_ref) {
            continue;
        }

        if let Some(mapping) = built_in_rust_alias_mapping(entry, alias, &use_ref.contract) {
            mappings.push(mapping);
            continue;
        }

        if let Some(mapping) = external_rust_alias_mapping(entry, alias, &use_ref.contract)? {
            mappings.push(mapping);
            continue;
        }

        return Err(miette::miette!(
            "Rust participant alias '{}' requires an explicit SDK mapping for contract '{}'",
            alias,
            use_ref.contract
        ));
    }
    Ok(mappings)
}

fn external_rust_alias_mapping(
    entry: &AutoPlanEntry,
    alias: &str,
    contract_id: &str,
) -> miette::Result<Option<trellis_codegen_rust::ParticipantAliasMapping>> {
    let Some(cargo_manifest) = nearest_cargo_manifest(&entry.discovered.source_path) else {
        return Ok(None);
    };
    let contents = fs::read_to_string(&cargo_manifest).into_diagnostic()?;
    let mut manifest = contents.parse::<toml::Table>().into_diagnostic()?;
    let Some(dependency_key) = manifest
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("metadata"))
        .and_then(toml::Value::as_table)
        .and_then(|metadata| metadata.get("trellis"))
        .and_then(toml::Value::as_table)
        .and_then(|trellis| trellis.get("sdk-mappings"))
        .and_then(toml::Value::as_table)
        .and_then(|mappings| mappings.get(alias))
        .and_then(toml::Value::as_str)
    else {
        return Ok(None);
    };
    let dependency_key = dependency_key.to_string();
    let dependency = manifest
        .get_mut("dependencies")
        .and_then(toml::Value::as_table_mut)
        .and_then(|dependencies| dependencies.get_mut(&dependency_key))
        .ok_or_else(|| {
            miette::miette!(
                "Rust SDK mapping '{}' names missing Cargo dependency '{}'",
                alias,
                dependency_key
            )
        })?;

    let package_name = dependency
        .as_table()
        .and_then(|table| table.get("package"))
        .and_then(toml::Value::as_str)
        .unwrap_or(&dependency_key)
        .to_string();
    if let Some(path) = dependency
        .as_table_mut()
        .and_then(|table| table.get_mut("path"))
        .and_then(|path| path.as_str().map(ToOwned::to_owned))
    {
        let package_root = cargo_manifest.parent().unwrap_or(Path::new(".")).join(path);
        let package_root = fs::canonicalize(&package_root).unwrap_or(package_root);
        if let Some(table) = dependency.as_table_mut() {
            table.insert(
                "path".to_string(),
                toml::Value::String(package_root.display().to_string()),
            );
        }
    }
    let dependency_spec = dependency.to_string();

    let metadata = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            cargo_manifest.to_string_lossy().as_ref(),
        ])
        .output()
        .into_diagnostic()?;
    if !metadata.status.success() {
        return Err(miette::miette!(
            "cargo metadata failed while resolving Rust SDK mapping '{}': {}",
            alias,
            String::from_utf8_lossy(&metadata.stderr).trim()
        ));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout).into_diagnostic()?;
    let package = metadata
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .and_then(|packages| {
            packages.iter().find(|package| {
                package.get("name").and_then(serde_json::Value::as_str)
                    == Some(package_name.as_str())
                    && package
                        .pointer("/metadata/trellis/contract-id")
                        .and_then(serde_json::Value::as_str)
                        == Some(contract_id)
            })
        })
        .ok_or_else(|| {
            miette::miette!(
                "Cargo dependency '{}' does not expose Trellis SDK metadata for contract '{}'",
                dependency_key,
                contract_id
            )
        })?;
    let package_manifest = PathBuf::from(
        package
            .get("manifest_path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| miette::miette!("resolved SDK package is missing manifest_path"))?,
    );
    let contract_manifest = package
        .pointer("/metadata/trellis/contract-manifest")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("contract.json");
    let manifest_path = package_manifest
        .parent()
        .unwrap_or(Path::new("."))
        .join(contract_manifest);

    Ok(Some(trellis_codegen_rust::ParticipantAliasMapping {
        alias: alias.to_string(),
        crate_name: dependency_key,
        manifest_path,
        crate_path: None,
        cargo_dependency: Some(dependency_spec),
    }))
}

fn nearest_cargo_manifest(source_path: &Path) -> Option<PathBuf> {
    source_path
        .parent()
        .into_iter()
        .flat_map(Path::ancestors)
        .map(|directory| directory.join("Cargo.toml"))
        .find(|manifest| manifest.exists())
}

fn built_in_rust_alias_mapping(
    entry: &AutoPlanEntry,
    alias: &str,
    contract_id: &str,
) -> Option<trellis_codegen_rust::ParticipantAliasMapping> {
    if !contract_id.starts_with("trellis.") {
        return None;
    }

    let repo_root = entry.runtime_repo_root.as_ref()?;
    let sdk_root = repo_root
        .join("generated/packages/cargo")
        .join(sdk_output_stem(contract_id));
    let manifest_path = repo_root
        .join("generated/contracts/manifests")
        .join(format!("{contract_id}.json"));
    if !sdk_root.join("Cargo.toml").exists() || !manifest_path.exists() {
        return None;
    }

    Some(trellis_codegen_rust::ParticipantAliasMapping {
        alias: alias.to_string(),
        crate_name: default_rust_crate_name_from_id(contract_id),
        manifest_path,
        crate_path: Some(sdk_root),
        cargo_dependency: None,
    })
}

fn remove_stale_participant_facade_outputs(out: &Path) -> miette::Result<()> {
    if out.exists() {
        fs::remove_dir_all(out).into_diagnostic()?;
    }
    Ok(())
}

pub fn contract_kind_label(kind: &ContractKind) -> &'static str {
    #[allow(unreachable_patterns)]
    match kind {
        ContractKind::Service => "service",
        ContractKind::App => "app",
        ContractKind::Device => "device",
        ContractKind::Agent => "agent",
        _ => unreachable!("portal contract kind has been removed"),
    }
}

fn print_auto_entry(entry: &AutoPlanEntry) {
    output::print_section(&format!(
        "{} {}",
        action_label(entry.action),
        entry.contract_id
    ));
    output::print_detail("kind", contract_kind_label(&entry.contract_kind));
    output::print_detail("source", entry.discovered.source_path.display().to_string());
    if let Some(out_manifest) = &entry.out_manifest {
        output::print_detail("manifest", out_manifest.display().to_string());
    }
    if let Some(jsr_out) = &entry.jsr_out {
        output::print_detail("jsr package", jsr_out.display().to_string());
    }
    if let Some(npm_out) = &entry.npm_out {
        output::print_detail("npm package", npm_out.display().to_string());
    }
    if let Some(cargo_out) = &entry.cargo_out {
        output::print_detail("cargo package", cargo_out.display().to_string());
    }
    if let Some(cargo_participant_out) = &entry.cargo_participant_out {
        output::print_detail(
            "cargo participant",
            cargo_participant_out.display().to_string(),
        );
    }
}

fn action_label(action: AutoAction) -> &'static str {
    match action {
        AutoAction::Generate => "generate",
        AutoAction::Verify => "verify",
    }
}

fn auto_plan_rank(entry: &AutoPlanEntry) -> u8 {
    match (entry.action, &entry.contract_kind) {
        (AutoAction::Generate, ContractKind::Service) => 0,
        (AutoAction::Generate, ContractKind::App) => 1,
        (AutoAction::Verify, _) => 2,
        #[allow(unreachable_patterns)]
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::SourceLanguage;

    #[test]
    fn auto_plan_orders_local_jsr_package_imports_before_dependents() {
        let _env_lock = crate::contract_input::test_env_lock();
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let notifications = root.join("services/notifications/contracts");
        let sherpa = root.join("services/sherpa/contracts");
        fs::create_dir_all(&notifications).unwrap();
        fs::create_dir_all(&sherpa).unwrap();
        fs::write(root.join("deno.json"), "{}\n").unwrap();
        fs::write(
            notifications.join("notifications.ts"),
            concat!(
                "import { SherpaRunIngested } from \"@trellis-sdk/krishi-sherpa\";\n",
                "import { defineServiceContract } from \"@qlever-llc/trellis\";\n",
                "export const notifications = defineServiceContract(() => ({\n",
                "  id: \"krishi.notifications@v1\",\n",
                "  kind: \"service\",\n",
                "  displayName: \"Notifications\",\n",
                "  description: \"Notifications\",\n",
                "  uses: [SherpaRunIngested.subscribe],\n",
                "}));\n",
                "export default notifications;\n",
            ),
        )
        .unwrap();
        fs::write(
            sherpa.join("sherpa.ts"),
            concat!(
                "import { defineServiceContract } from \"@qlever-llc/trellis\";\n",
                "export const sherpa = defineServiceContract(() => ({\n",
                "  id: \"krishi.sherpa@v1\",\n",
                "  kind: \"service\",\n",
                "  displayName: \"Sherpa\",\n",
                "  description: \"Sherpa\",\n",
                "}));\n",
                "export default sherpa;\n",
            ),
        )
        .unwrap();

        let discovered = vec![
            DiscoveredContractSource {
                project_root: root.join("services/notifications"),
                manifest_path: root.join("deno.json"),
                language: SourceLanguage::TypeScript,
                source_path: notifications.join("notifications.ts"),
            },
            DiscoveredContractSource {
                project_root: root.join("services/sherpa"),
                manifest_path: root.join("deno.json"),
                language: SourceLanguage::TypeScript,
                source_path: sherpa.join("sherpa.ts"),
            },
        ];

        let plan = build_auto_plan(discovered, Some(root), "@trellis-sdk/").unwrap();

        assert_eq!(
            plan.iter()
                .map(|entry| entry.contract_id.as_str())
                .collect::<Vec<_>>(),
            vec!["krishi.sherpa@v1", "krishi.notifications@v1"]
        );
        assert_eq!(
            plan[0].jsr_out,
            Some(root.join("generated/packages/jsr/krishi-sherpa"))
        );
        assert_eq!(
            plan[0].npm_out,
            Some(root.join("generated/packages/npm/krishi-sherpa"))
        );
        assert_eq!(
            plan[0].cargo_out,
            Some(root.join("generated/packages/cargo/krishi-sherpa"))
        );
    }
}
