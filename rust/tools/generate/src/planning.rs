use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use miette::IntoDiagnostic;
use serde_json::Value;
use trellis_contracts::ContractKind;

use crate::artifacts::{
    current_generator_fingerprints, default_rust_crate_name_from_id, detect_output_root,
    detect_runtime_source, generated_artifacts_are_fresh, generated_artifacts_metadata,
    generated_artifacts_metadata_from_parts, native_api_digest, required_owner_version,
    rust_runtime_deps, sdk_output_stem, ts_package_name_from_id, write_contract_outputs,
    write_generated_artifacts_metadata, write_participant_facade_outputs,
    write_protocol_participant, ContractOutputPlan,
};
use crate::contract_input;
use crate::discovery::{discover_contract_kind, DiscoveredContractSource};
use crate::model::{PackageTarget, RuntimeSource};
use crate::output;
use crate::resolution_cache::{CacheMissReason, CachedContractResolution, ResolutionCache};
use trellis_codegen_rust::GenerateRustParticipantFacadeOpts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoAction {
    Generate,
    Verify,
}

#[derive(Debug, Clone)]
pub struct AutoPlanEntry {
    pub discovered: DiscoveredContractSource,
    pub resolved: Option<Arc<contract_input::ResolvedNativeInput>>,
    pub(crate) cached_resolution: Option<Arc<CachedContractResolution>>,
    pub(crate) previous_participant_id: Option<String>,
    pub(crate) local_dependencies: Vec<String>,
    pub contract_id: String,
    pub contract_kind: ContractKind,
    pub action: AutoAction,
    pub out_api: Option<PathBuf>,
    pub jsr_out: Option<PathBuf>,
    pub cargo_out: Option<PathBuf>,
    pub cargo_participant_out: Option<PathBuf>,
    pub protocol_participant_out: Option<PathBuf>,
    pub runtime_source: RuntimeSource,
    pub runtime_repo_root: Option<PathBuf>,
}

#[derive(Debug, Default, Clone)]
pub struct AutoExecutionSummary {
    pub generated: usize,
    pub verified: usize,
    pub skipped: usize,
    pub owned_api_paths: Vec<PathBuf>,
}

pub fn build_auto_plan(
    discovered: Vec<DiscoveredContractSource>,
    shared_output_root: Option<&Path>,
) -> miette::Result<Vec<AutoPlanEntry>> {
    build_auto_plan_with_targets(discovered, shared_output_root, None, &BTreeMap::new())
}

pub fn build_auto_plan_with_targets(
    discovered: Vec<DiscoveredContractSource>,
    shared_output_root: Option<&Path>,
    requested_targets: Option<&[PackageTarget]>,
    locked_api_digests: &BTreeMap<String, String>,
) -> miette::Result<Vec<AutoPlanEntry>> {
    let mut plan = Vec::new();
    for contract in discovered {
        let previous_participant_id =
            ResolutionCache::for_contract(&contract).previous_participant_id(&contract);
        let cached_resolution = load_cached_resolution(&contract);
        let (contract_id, contract_kind, resolved) = if let Some(cached) = &cached_resolution {
            (
                cached.contract_id().to_string(),
                cached.contract_kind().clone(),
                None,
            )
        } else {
            let (resolved, contract_kind) = resolve_and_cache(&contract, &[], locked_api_digests)?;
            let contract_id = resolved.api.render_model.id.clone();
            (contract_id, contract_kind, Some(Arc::new(resolved)))
        };
        validate_output_identity("API", &contract_id)?;
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
        let (out_api, jsr_out, cargo_out, cargo_participant_out, protocol_participant_out) =
            match action {
                AutoAction::Generate => {
                    let sdk_stem = sdk_output_stem(&contract_id);
                    let participant_id = cached_resolution
                        .as_deref()
                        .and_then(CachedContractResolution::participant_id)
                        .map(str::to_owned)
                        .or_else(|| {
                            resolved.as_deref().and_then(|resolved| {
                                resolved
                                    .participant
                                    .as_ref()
                                    .map(|participant| participant.participant.id().to_owned())
                            })
                        })
                        .or(contract
                            .source_path
                            .with_file_name("trellis.participant.json")
                            .is_file()
                            .then(|| {
                                trellis_contracts::load_participant_source(
                                    contract
                                        .source_path
                                        .with_file_name("trellis.participant.json"),
                                )
                                .map(|participant| participant.participant.id().to_owned())
                                .map_err(|error| miette::miette!(error.to_string()))
                            })
                            .transpose()?);
                    let targets = targets_for_entry(&contract, &contract_kind, requested_targets);
                    let jsr_package_root = resolve_jsr_package_root(
                        &output_root,
                        &contract.project_root,
                        runtime_repo_root.as_deref(),
                    );
                    let out_api = if !targets.is_empty() {
                        Some(
                            output_root
                                .join("generated/protocol/apis")
                                .join(format!("{}.json", contract_id)),
                        )
                    } else {
                        None
                    };
                    let jsr_out = if targets.contains(&PackageTarget::TypeScript) {
                        Some(jsr_package_root.join(&sdk_stem))
                    } else {
                        None
                    };
                    let cargo_out = if targets.contains(&PackageTarget::Cargo) {
                        Some(output_root.join("generated/packages/cargo").join(&sdk_stem))
                    } else {
                        None
                    };
                    let cargo_participant_out =
                        if matches!(contract.language, crate::discovery::SourceLanguage::Rust)
                            && targets.contains(&PackageTarget::Cargo)
                        {
                            Some(
                                output_root
                                    .join("generated/packages/cargo-participants")
                                    .join(sdk_output_stem(participant_id.as_deref().ok_or_else(
                                        || miette::miette!("Rust participant identity is missing"),
                                    )?)),
                            )
                        } else {
                            None
                        };
                    let protocol_participant_out = participant_id.as_ref().map(|participant_id| {
                        output_root
                            .join("generated/protocol/participants")
                            .join(format!("{participant_id}.json"))
                    });
                    if let Some(participant_id) = &participant_id {
                        validate_output_identity("participant", participant_id)?;
                    }
                    (
                        out_api,
                        jsr_out,
                        cargo_out,
                        cargo_participant_out,
                        protocol_participant_out,
                    )
                }
                AutoAction::Verify => (None, None, None, None, None),
            };
        let action = if matches!(action, AutoAction::Generate)
            && out_api.is_none()
            && jsr_out.is_none()
            && cargo_out.is_none()
            && cargo_participant_out.is_none()
            && protocol_participant_out.is_none()
        {
            AutoAction::Verify
        } else {
            action
        };
        plan.push(AutoPlanEntry {
            discovered: contract,
            resolved,
            cached_resolution,
            previous_participant_id,
            local_dependencies: Vec::new(),
            contract_id,
            contract_kind,
            action,
            out_api,
            jsr_out,
            cargo_out,
            cargo_participant_out,
            protocol_participant_out,
            runtime_source,
            runtime_repo_root,
        });
    }
    let mut outputs = BTreeMap::new();
    for entry in &plan {
        for path in [
            entry.out_api.as_ref(),
            entry.jsr_out.as_ref(),
            entry.cargo_out.as_ref(),
            entry.cargo_participant_out.as_ref(),
            entry.protocol_participant_out.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(existing) = outputs.insert(path, &entry.discovered.source_path) {
                return Err(miette::miette!(
                    "contract outputs collide at {} for {} and {}",
                    path.display(),
                    existing.display(),
                    entry.discovered.source_path.display()
                ));
            }
        }
    }
    sort_auto_plan(&mut plan)?;
    Ok(plan)
}

/// Reject identities that cannot safely name generated filesystem outputs.
pub fn validate_output_identity(kind: &str, id: &str) -> miette::Result<()> {
    miette::ensure!(
        !id.contains(['/', '\\']) && !id.contains("..") && !id.chars().any(char::is_whitespace),
        "{kind} id {id:?} cannot be used as a generated output name"
    );
    Ok(())
}

fn evaluate_discovered_contract(
    contract: &DiscoveredContractSource,
) -> miette::Result<contract_input::ResolvedNativeInput> {
    crate::timings::resolve(&contract.source_path, || {
        contract_input::resolve_contract_input(
            None,
            None,
            &[],
            Some(contract.source_path.as_path()),
            None,
            "API",
            contract_input::default_image_api_path(),
        )
    })
}

fn load_cached_resolution(
    contract: &DiscoveredContractSource,
) -> Option<Arc<CachedContractResolution>> {
    match ResolutionCache::for_contract(contract).load(contract) {
        Ok(cached) => Some(Arc::new(cached)),
        Err(reason) => {
            crate::timings::resolution_cache_miss(cache_miss_reason(reason));
            None
        }
    }
}

fn cache_miss_reason(reason: CacheMissReason) -> &'static str {
    match reason {
        CacheMissReason::Missing => "missing",
        CacheMissReason::InvalidSchema => "invalid schema",
        CacheMissReason::InvalidFingerprint => "invalid fingerprint",
        CacheMissReason::InputChanged => "input changed",
        CacheMissReason::Corrupt => "corrupt",
    }
}

fn resolve_and_cache(
    contract: &DiscoveredContractSource,
    local_dependencies: &[String],
    current_api_digests: &BTreeMap<String, String>,
) -> miette::Result<(contract_input::ResolvedNativeInput, ContractKind)> {
    let resolved = evaluate_discovered_contract(contract)?;
    for referenced in &resolved.referenced_apis {
        let id = &referenced.render_model.id;
        let digest = &referenced.digest;
        miette::ensure!(
            current_api_digests.get(id) == Some(digest),
            "referenced API '{id}' with digest '{digest}' is absent from trellis.lock or differs from its locked digest ({:?})",
            current_api_digests.get(id)
        );
    }
    let contract_kind = discover_contract_kind(contract, &resolved)?;
    let mut dependencies: Vec<String> = resolved
        .referenced_apis
        .iter()
        .map(|api| api.render_model.id.clone())
        .collect();
    dependencies.extend(local_dependencies.iter().cloned());
    dependencies.sort();
    dependencies.dedup();
    let _ = ResolutionCache::for_contract(contract).store(
        contract,
        &resolved,
        &contract_kind,
        dependencies,
        current_api_digests,
    );
    Ok((resolved, contract_kind))
}

fn targets_for_entry(
    contract: &DiscoveredContractSource,
    kind: &ContractKind,
    requested_targets: Option<&[PackageTarget]>,
) -> Vec<PackageTarget> {
    let defaults = if matches!(kind, ContractKind::Service) {
        vec![
            PackageTarget::Api,
            PackageTarget::TypeScript,
            PackageTarget::Cargo,
        ]
    } else if matches!(kind, ContractKind::App)
        && !matches!(contract.language, crate::discovery::SourceLanguage::Rust)
    {
        vec![PackageTarget::Api, PackageTarget::TypeScript]
    } else if matches!(contract.language, crate::discovery::SourceLanguage::Rust) {
        vec![PackageTarget::Api, PackageTarget::Cargo]
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

fn sort_auto_plan(plan: &mut Vec<AutoPlanEntry>) -> miette::Result<()> {
    let mut remaining = plan.clone();
    remaining.sort_by(compare_auto_plan_entries);
    let dependencies = remaining
        .iter()
        .map(|entry| {
            let dependencies = local_api_dependencies(entry, plan);
            (entry.contract_id.clone(), dependencies)
        })
        .collect::<BTreeMap<_, _>>();
    let api_digests = remaining
        .iter()
        .filter_map(|entry| {
            entry
                .resolved
                .as_deref()
                .map(|resolved| resolved.api.digest.clone())
                .or_else(|| {
                    entry
                        .cached_resolution
                        .as_deref()
                        .map(|cached| cached.api_digest().to_string())
                })
                .map(|digest| (entry.contract_id.clone(), digest))
        })
        .collect::<BTreeMap<_, _>>();
    for entry in &mut remaining {
        entry.local_dependencies = dependencies[&entry.contract_id].clone();
        let needs_update = entry
            .cached_resolution
            .as_deref()
            .is_none_or(|cached| cached.dependencies() != entry.local_dependencies);
        if needs_update {
            if let Some(cached) = ResolutionCache::for_contract(&entry.discovered)
                .update_dependencies(
                    &entry.discovered,
                    entry.local_dependencies.clone(),
                    &api_digests,
                )
            {
                entry.cached_resolution = Some(Arc::new(cached));
            }
        }
    }

    let mut sorted = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let Some(next) = remaining.iter().position(|entry| {
            dependencies[&entry.contract_id].iter().all(|dependency| {
                sorted
                    .iter()
                    .any(|candidate: &AutoPlanEntry| candidate.contract_id == dependency.as_str())
                    || !remaining
                        .iter()
                        .any(|candidate| candidate.contract_id == dependency.as_str())
            })
        }) else {
            return Err(miette::miette!(
                "contract dependency cycle: {}",
                remaining
                    .iter()
                    .map(|entry| entry.contract_id.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ));
        };
        sorted.push(remaining.remove(next));
    }

    *plan = sorted;
    Ok(())
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

fn local_api_dependencies(entry: &AutoPlanEntry, plan: &[AutoPlanEntry]) -> Vec<String> {
    let mut dependencies = entry
        .cached_resolution
        .as_deref()
        .map(CachedContractResolution::dependencies)
        .unwrap_or_default()
        .to_vec();
    dependencies.extend(
        entry
            .resolved
            .as_deref()
            .into_iter()
            .flat_map(|resolved| &resolved.referenced_apis)
            .map(|api| api.render_model.id.clone()),
    );
    dependencies.sort();
    dependencies.dedup();
    dependencies.retain(|dependency| {
        plan.iter()
            .any(|candidate| candidate.contract_id == *dependency)
    });
    dependencies
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
    locked_api_digests: BTreeMap<String, String>,
    title: Option<&str>,
    show_title: bool,
    force: bool,
    prefix: &str,
    runtime_version: Option<&str>,
) -> miette::Result<AutoExecutionSummary> {
    let default_runtime_version = crate::artifacts::trellis_package_version();
    let runtime_version = runtime_version.unwrap_or(&default_runtime_version);
    if show_title {
        output::print_section("Run");
    } else if let Some(title) = title {
        output::print_title(title);
    }

    let fingerprints = current_generator_fingerprints();
    let mut summary = AutoExecutionSummary {
        owned_api_paths: plan
            .iter()
            .filter_map(|entry| entry.out_api.clone())
            .collect(),
        ..AutoExecutionSummary::default()
    };
    let mut current_api_digests = locked_api_digests;
    let mut participant_outputs = plan
        .iter()
        .filter_map(|entry| {
            entry
                .protocol_participant_out
                .as_ref()
                .map(|path| (path.clone(), entry.discovered.source_path.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    cleanup_legacy_protocol_outputs(plan)?;
    for entry in plan {
        if let Some(cached) = entry
            .cached_resolution
            .as_deref()
            .filter(|cached| {
                cached.projection_is_current() && cached.references_match(&current_api_digests)
            })
            .filter(|_| !force && matches!(entry.action, AutoAction::Generate))
        {
            let package_name = ts_package_name_from_id(&entry.contract_id, prefix);
            let crate_name = default_rust_crate_name_from_id(&entry.contract_id);
            let out_api = entry
                .out_api
                .as_ref()
                .ok_or_else(|| miette::miette!("missing manifest output for generated contract"))?;
            let participant_version = if entry.cargo_participant_out.is_some() {
                Some(cached.owner_version().ok_or_else(|| {
                    miette::miette!("participant facade requires an owning workspace version")
                })?)
            } else {
                None
            };
            let output_plan = ContractOutputPlan {
                api_sdk_version: cached.api_version(),
                participant_version,
                runtime_version,
                out_api,
                ts_out: entry.jsr_out.as_deref(),
                rust_out: entry.cargo_out.as_deref(),
                package_name: &package_name,
                crate_name: &crate_name,
                runtime_source: entry.runtime_source,
                runtime_repo_root: entry.runtime_repo_root.as_deref(),
                fingerprints,
            };
            let metadata = generated_artifacts_metadata_from_parts(
                &entry.contract_id,
                cached.api_version(),
                cached.generated_api_digest(),
                cached.participant_digest(),
                &output_plan,
            );
            let freshness = generated_artifacts_are_fresh(
                &metadata,
                out_api,
                entry.jsr_out.as_deref(),
                entry.cargo_out.as_deref(),
            );
            let participant_fresh = freshness.participant
                && freshness.cargo
                && cargo_participant_key_outputs_exist(entry.cargo_participant_out.as_deref());
            let protocol_participant_fresh =
                cached.protocol_participant_is_fresh(entry.protocol_participant_out.as_deref());
            if freshness.all() && participant_fresh && protocol_participant_fresh {
                cached.emit_warnings();
                crate::timings::target("api", true, false);
                crate::timings::target("jsr", entry.jsr_out.is_some(), false);
                crate::timings::target("cargo", entry.cargo_out.is_some(), false);
                crate::timings::target(
                    "participant-facade",
                    entry.cargo_participant_out.is_some(),
                    false,
                );
                crate::timings::target(
                    "participant-json",
                    entry.protocol_participant_out.is_some()
                        && cached.participant_digest().is_some(),
                    false,
                );
                crate::timings::resolution_cache_hit();
                current_api_digests
                    .insert(entry.contract_id.clone(), cached.api_digest().to_string());
                output::print_success(&format!(
                    "artifacts already up to date for {}",
                    entry.contract_id
                ));
                summary.skipped += 1;
                continue;
            }
        }
        let deferred;
        let resolved = if let Some(resolved) = entry.resolved.as_deref() {
            resolved
        } else if let Some(cached) = entry
            .cached_resolution
            .as_deref()
            .filter(|_| !force)
            .filter(|cached| cached.references_match(&current_api_digests))
        {
            match cached.rehydrate() {
                Ok(cached) => {
                    if !entry
                        .cached_resolution
                        .as_deref()
                        .is_some_and(CachedContractResolution::projection_is_current)
                    {
                        let _ = ResolutionCache::for_contract(&entry.discovered).store(
                            &entry.discovered,
                            &cached,
                            &entry.contract_kind,
                            entry.local_dependencies.clone(),
                            &current_api_digests,
                        );
                    }
                    crate::timings::resolution_cache_hit();
                    deferred = cached;
                    &deferred
                }
                Err(_) => {
                    crate::timings::resolution_cache_miss("corrupt");
                    deferred = resolve_and_cache(
                        &entry.discovered,
                        &entry.local_dependencies,
                        &current_api_digests,
                    )?
                    .0;
                    &deferred
                }
            }
        } else {
            if entry.cached_resolution.is_some() {
                crate::timings::resolution_cache_miss(if force {
                    "forced"
                } else {
                    "dependency changed"
                });
            }
            deferred = resolve_and_cache(
                &entry.discovered,
                &entry.local_dependencies,
                &current_api_digests,
            )?
            .0;
            &deferred
        };
        miette::ensure!(
            resolved.api.render_model.id == entry.contract_id,
            "contract identity changed between discovery ({}) and resolution ({}) for {}",
            entry.contract_id,
            resolved.api.render_model.id,
            entry.discovered.source_path.display()
        );
        if let Some(participant) = &resolved.participant {
            miette::ensure!(
                participant.render_model.kind == entry.contract_kind,
                "contract kind changed between discovery ({:?}) and resolution ({:?}) for {}",
                entry.contract_kind,
                participant.render_model.kind,
                entry.discovered.source_path.display()
            );
        }
        contract_input::warn_forward_incompatible_public_schemas(&resolved.api);
        current_api_digests.insert(entry.contract_id.clone(), resolved.api.digest.clone());
        match entry.action {
            AutoAction::Generate => {
                let participant_version = if entry.cargo_participant_out.is_some() {
                    Some(required_owner_version(
                        resolved,
                        "generate a participant facade from local discovery",
                    )?)
                } else {
                    None
                };
                let package_name = ts_package_name_from_id(&resolved.api.render_model.id, prefix);
                let crate_name = default_rust_crate_name_from_id(&resolved.api.render_model.id);
                let out_api = entry.out_api.as_ref().ok_or_else(|| {
                    miette::miette!("missing manifest output for generated contract")
                })?;
                if let Some(participant) = &resolved.participant {
                    validate_output_identity("participant", participant.participant.id())?;
                }
                let protocol_participant_out = match (
                    entry.protocol_participant_out.as_ref(),
                    resolved.participant.as_ref(),
                ) {
                    (Some(path), _) => Some(path.clone()),
                    (None, Some(participant)) => Some(protocol_participant_output_path(
                        out_api,
                        participant.participant.id(),
                    )?),
                    (None, None) => None,
                };
                if let Some(path) = &protocol_participant_out {
                    if let Some(existing) = participant_outputs
                        .insert(path.clone(), entry.discovered.source_path.clone())
                    {
                        miette::ensure!(
                            existing == entry.discovered.source_path,
                            "participant outputs collide at {} for {} and {}",
                            path.display(),
                            existing.display(),
                            entry.discovered.source_path.display()
                        );
                    }
                }
                let output_plan = ContractOutputPlan {
                    api_sdk_version: resolved.api.api.version(),
                    participant_version: participant_version.as_deref(),
                    runtime_version,
                    out_api,
                    ts_out: entry.jsr_out.as_deref(),
                    rust_out: entry.cargo_out.as_deref(),
                    package_name: &package_name,
                    crate_name: &crate_name,
                    runtime_source: entry.runtime_source,
                    runtime_repo_root: entry.runtime_repo_root.as_deref(),
                    fingerprints,
                };
                let metadata = generated_artifacts_metadata(
                    resolved,
                    &native_api_digest(resolved)?,
                    &output_plan,
                );
                let freshness = if force {
                    Default::default()
                } else {
                    generated_artifacts_are_fresh(
                        &metadata,
                        out_api,
                        entry.jsr_out.as_deref(),
                        entry.cargo_out.as_deref(),
                    )
                };
                let participant_fresh = freshness.participant
                    && freshness.cargo
                    && cargo_participant_key_outputs_exist(entry.cargo_participant_out.as_deref());
                let protocol_participant_fresh = match (
                    resolved.participant.as_ref(),
                    protocol_participant_out.as_deref(),
                ) {
                    (Some(expected), Some(path)) => {
                        protocol_participant_output_is_fresh(expected, path)?
                    }
                    (None, None) => true,
                    _ => false,
                };
                crate::timings::target("api", true, !freshness.api);
                crate::timings::target("jsr", entry.jsr_out.is_some(), !freshness.jsr);
                crate::timings::target("cargo", entry.cargo_out.is_some(), !freshness.cargo);
                crate::timings::target(
                    "participant-facade",
                    entry.cargo_participant_out.is_some(),
                    !participant_fresh,
                );
                crate::timings::target(
                    "participant-json",
                    protocol_participant_out.is_some() && resolved.participant.is_some(),
                    !protocol_participant_fresh,
                );
                if freshness.all() && participant_fresh && protocol_participant_fresh {
                    output::print_success(&format!(
                        "artifacts already up to date for {}",
                        resolved.api.render_model.id
                    ));
                    summary.skipped += 1;
                    continue;
                }
                print_auto_entry(entry);
                write_contract_outputs(resolved, &output_plan, freshness)?;
                if let Some(cargo_participant_out) = entry
                    .cargo_participant_out
                    .as_ref()
                    .filter(|_| !participant_fresh)
                {
                    let participant_source = resolved
                        .participant_path
                        .as_deref()
                        .unwrap_or(&resolved.api.path);
                    let mappings = participant_alias_mappings(entry, plan, participant_source)?;
                    write_participant_facade_outputs(
                        protocol_participant_out.as_deref().ok_or_else(|| {
                            miette::miette!("missing protocol participant output")
                        })?,
                        GenerateRustParticipantFacadeOpts {
                            api_path: resolved.api_path.clone(),
                            participant_path: participant_source.to_path_buf(),
                            out_dir: cargo_participant_out.clone(),
                            crate_name: format!(
                                "trellis-participant-{}",
                                sdk_output_stem(
                                    &resolved
                                        .participant
                                        .as_ref()
                                        .ok_or_else(|| {
                                            miette::miette!(
                                                "participant facade requires a participant artifact"
                                            )
                                        })?
                                        .render_model
                                        .id
                                )
                            ),
                            crate_version: participant_version.clone().ok_or_else(|| {
                                miette::miette!(
                                    "participant facade requires an owning workspace version"
                                )
                            })?,
                            runtime_deps: rust_runtime_deps(
                                entry.runtime_source,
                                runtime_version.to_owned(),
                                entry.runtime_repo_root.clone(),
                            ),
                            owned_sdk_crate_name: Some(crate_name.clone()),
                            owned_sdk_path: entry.cargo_out.clone(),
                            alias_mappings: mappings,
                        },
                    )?;
                } else if let Some(protocol_participant_out) = protocol_participant_out
                    .as_deref()
                    .filter(|_| !protocol_participant_fresh)
                {
                    write_protocol_participant(resolved, protocol_participant_out)?;
                }
                write_generated_artifacts_metadata(out_api, &metadata)?;
                if !freshness.api {
                    crate::timings::installed(out_api)?;
                }
                for (path, generated) in [
                    (entry.jsr_out.as_deref(), !freshness.jsr),
                    (entry.cargo_out.as_deref(), !freshness.cargo),
                    (entry.cargo_participant_out.as_deref(), !participant_fresh),
                    (
                        protocol_participant_out.as_deref(),
                        resolved.participant.is_some() && !protocol_participant_fresh,
                    ),
                ] {
                    if generated {
                        if let Some(path) = path {
                            crate::timings::installed(path)?;
                        }
                    }
                }
                summary.generated += 1;
            }
            AutoAction::Verify => {
                if show_title {
                    print_auto_entry(entry);
                }
                output::print_success(&format!("verified {}", resolved.api.render_model.id));
                summary.verified += 1;
            }
        }
    }
    for (current, source) in &participant_outputs {
        let entry = plan
            .iter()
            .find(|entry| entry.discovered.source_path == *source)
            .expect("participant output source belongs to the execution plan");
        let legacy = current
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!("{}.json", entry.contract_id));
        if legacy != *current && !participant_outputs.contains_key(&legacy) {
            remove_participant_output_if_owned(
                &legacy,
                &entry.contract_id,
                Some(&entry.contract_id),
            )?;
        }
    }
    Ok(summary)
}

fn protocol_participant_output_path(
    out_api: &Path,
    participant_id: &str,
) -> miette::Result<PathBuf> {
    let protocol_root = out_api.parent().and_then(Path::parent).ok_or_else(|| {
        miette::miette!("generated API output is outside generated/protocol/apis")
    })?;
    Ok(protocol_root
        .join("participants")
        .join(format!("{participant_id}.json")))
}

fn cargo_participant_key_outputs_exist(path: Option<&Path>) -> bool {
    path.is_none_or(|path| path.join("Cargo.toml").exists() && path.join("src/lib.rs").exists())
}

fn protocol_participant_output_is_fresh(
    expected: &trellis_contracts::LoadedParticipant,
    output: &Path,
) -> miette::Result<bool> {
    Ok(fs::read_to_string(output)
        .is_ok_and(|existing| existing == format!("{}\n", expected.canonical)))
}

fn cleanup_legacy_protocol_outputs(plan: &[AutoPlanEntry]) -> miette::Result<()> {
    let current_participant_outputs = plan
        .iter()
        .filter_map(|entry| entry.protocol_participant_out.clone())
        .collect::<BTreeSet<_>>();
    let current_facade_outputs = plan
        .iter()
        .filter_map(|entry| entry.cargo_participant_out.clone())
        .collect::<BTreeSet<_>>();
    for entry in plan {
        if let Some(current) = &entry.protocol_participant_out {
            for (legacy_id, expected_id, expected_self_api) in [
                (
                    Some(entry.contract_id.as_str()),
                    entry.contract_id.as_str(),
                    Some(entry.contract_id.as_str()),
                ),
                (
                    entry.previous_participant_id.as_deref(),
                    entry.previous_participant_id.as_deref().unwrap_or(""),
                    None,
                ),
            ] {
                let Some(legacy_id) = legacy_id else { continue };
                let legacy = current
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(format!("{legacy_id}.json"));
                if legacy != *current && !current_participant_outputs.contains(&legacy) {
                    remove_participant_output_if_owned(&legacy, expected_id, expected_self_api)?;
                }
            }
        }
        if let Some(current) = &entry.cargo_participant_out {
            for (legacy_id, expected_id) in [
                (Some(entry.contract_id.as_str()), entry.contract_id.as_str()),
                (
                    entry.previous_participant_id.as_deref(),
                    entry.previous_participant_id.as_deref().unwrap_or(""),
                ),
            ] {
                let Some(legacy_id) = legacy_id else { continue };
                let legacy = current
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(sdk_output_stem(legacy_id));
                if legacy != *current && !current_facade_outputs.contains(&legacy) {
                    remove_facade_output_if_owned(&legacy, expected_id)?;
                }
            }
        }
    }
    let roots = plan
        .iter()
        .filter_map(|entry| entry.out_api.as_deref())
        .filter_map(|path| {
            path.parent()?
                .parent()?
                .parent()?
                .parent()
                .map(Path::to_path_buf)
        })
        .collect::<BTreeSet<_>>();
    for root in roots {
        for legacy in [
            root.join("generated/apis"),
            root.join("generated/contracts"),
        ] {
            if legacy.exists() {
                fs::remove_dir_all(legacy).into_diagnostic()?;
            }
        }
    }
    Ok(())
}

fn remove_participant_output_if_owned(
    path: &Path,
    expected_id: &str,
    expected_self_api: Option<&str>,
) -> miette::Result<()> {
    let owned = fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .is_some_and(|value| {
            value.get("id").and_then(|id| id.as_str()) == Some(expected_id)
                && expected_self_api.is_none_or(|api_id| {
                    value
                        .pointer("/implements/self/api")
                        .and_then(|id| id.as_str())
                        == Some(api_id)
                })
        });
    if owned {
        fs::remove_file(path).into_diagnostic()?;
    }
    Ok(())
}

fn remove_facade_output_if_owned(path: &Path, expected_id: &str) -> miette::Result<()> {
    let owned = fs::read_to_string(path.join("src/contract.rs")).is_ok_and(|source| {
        source.contains(&format!("pub const CONTRACT_ID: &str = {expected_id:?};"))
    });
    if owned {
        fs::remove_dir_all(path).into_diagnostic()?;
    }
    Ok(())
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
    match kind {
        ContractKind::Service | ContractKind::App => AutoAction::Generate,
        ContractKind::Device | ContractKind::Agent => AutoAction::Verify,
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
    source_path: &Path,
) -> miette::Result<Vec<trellis_codegen_rust::ParticipantAliasMapping>> {
    let local_manifest = source_path;
    let loaded = trellis_contracts::load_participant_source(local_manifest).into_diagnostic()?;
    let uses = loaded
        .render_model
        .uses
        .iter()
        .map(|(alias, used)| (alias.clone(), used.clone()))
        .collect::<Vec<_>>();
    let mut mappings = Vec::new();
    for (alias, use_ref) in &uses {
        if !trellis_codegen_rust::participant_use_requires_mapping(&loaded, alias, use_ref) {
            continue;
        }
        if let Some(mapped) = plan
            .iter()
            .find(|candidate| candidate.contract_id == use_ref.api && candidate.cargo_out.is_some())
        {
            let manifest_path = mapped
                .out_api
                .as_ref()
                .ok_or_else(|| {
                    miette::miette!("missing mapped manifest for {}", mapped.contract_id)
                })?
                .clone();
            mappings.push(trellis_codegen_rust::ParticipantAliasMapping {
                alias: alias.clone(),
                crate_name: default_rust_crate_name_from_id(&mapped.contract_id),
                api_path: manifest_path,
                crate_path: mapped.cargo_out.clone(),
                cargo_dependency: None,
            });
            continue;
        }

        if let Some(mapping) = installed_rust_alias_mapping(entry, alias, &use_ref.api)? {
            mappings.push(mapping);
            continue;
        }

        return Err(miette::miette!(
            "Rust participant alias '{}' requires contract '{}' in trellis.lock",
            alias,
            use_ref.api
        ));
    }
    Ok(mappings)
}

fn installed_rust_alias_mapping(
    entry: &AutoPlanEntry,
    alias: &str,
    contract_id: &str,
) -> miette::Result<Option<trellis_codegen_rust::ParticipantAliasMapping>> {
    let Some(root) = entry
        .discovered
        .source_path
        .ancestors()
        .find(|directory| directory.join(".trellis").is_dir())
    else {
        return Ok(None);
    };
    let crate_path = root
        .join(".trellis/generated/rust/packages")
        .join(sdk_output_stem(contract_id));
    if !crate_path.join("Cargo.toml").is_file() {
        return Ok(None);
    }
    let api_root = root.join(".trellis/apis").join(contract_id);
    let mut installed = fs::read_dir(&api_root)
        .into_diagnostic()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("trellis.api.json"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    installed.sort();
    miette::ensure!(
        installed.len() == 1,
        "installed API '{}' must have exactly one materialized release",
        contract_id
    );
    Ok(Some(trellis_codegen_rust::ParticipantAliasMapping {
        alias: alias.to_string(),
        crate_name: default_rust_crate_name_from_id(contract_id),
        api_path: installed.remove(0),
        crate_path: Some(crate_path),
        cargo_dependency: None,
    }))
}

pub fn contract_kind_label(kind: &ContractKind) -> &'static str {
    match kind {
        ContractKind::Service => "service",
        ContractKind::App => "app",
        ContractKind::Device => "device",
        ContractKind::Agent => "agent",
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
    if let Some(out_api) = &entry.out_api {
        output::print_detail("api", out_api.display().to_string());
    }
    if let Some(jsr_out) = &entry.jsr_out {
        output::print_detail("jsr package", jsr_out.display().to_string());
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
        (AutoAction::Generate, ContractKind::Device | ContractKind::Agent) => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::SourceLanguage;

    #[test]
    fn auto_plan_rejects_colliding_api_outputs() {
        let _env_lock = crate::contract_input::test_env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let trellis = repo_root.join("ts/packages/trellis/contract_support/mod.ts");
        fs::write(
            temp.path().join("deno.json"),
            serde_json::to_vec(&serde_json::json!({
                "imports": {
                    "@qlever-llc/result": format!(
                        "file://{}",
                        repo_root.join("ts/packages/result/mod.ts").display()
                    ),
                    "typebox": "npm:typebox@^1.1.33"
                }
            }))
            .unwrap(),
        )
        .unwrap();
        let source = format!(
            concat!(
                "import {{ defineServiceContract }} from \"file://{}\";\n",
                "export default defineServiceContract({{}}, () => ({{ ",
                "id: \"example.participant@v1\", apiId: \"example.api@v1\", ",
                "apiVersion: \"1.0.0\", displayName: \"Example\", ",
                "description: \"Example\" }}));\n",
            ),
            trellis.display(),
        );
        let first = temp.path().join("first.ts");
        let second = temp.path().join("second.ts");
        fs::write(&first, &source).unwrap();
        fs::write(&second, source).unwrap();
        let discovered = [first, second]
            .into_iter()
            .map(|source_path| DiscoveredContractSource {
                project_root: temp.path().to_path_buf(),
                manifest_path: temp.path().join("deno.json"),
                language: SourceLanguage::TypeScript,
                source_path,
            })
            .collect();

        let error = build_auto_plan(discovered, Some(temp.path()))
            .expect_err("duplicate API IDs must not share outputs");
        assert!(
            error.to_string().contains("contract outputs collide"),
            "{error}"
        );
    }

    #[test]
    fn rejects_identities_that_escape_generated_output_names() {
        assert!(validate_output_identity("API", "trellis.orders@v1").is_ok());
        assert!(validate_output_identity("API", "../orders@v1").is_err());
        assert!(validate_output_identity("participant", "trellis/orders@v1").is_err());
        assert!(validate_output_identity("participant", "trellis\\orders@v1").is_err());
    }

    #[test]
    fn protocol_participant_freshness_tracks_canonical_artifact() {
        let temp = tempfile::tempdir().unwrap();
        let expected_path = temp.path().join("expected.participant.json");
        let output_path = temp.path().join("generated.participant.json");
        let service = r#"{"format":"trellis.participant.v1","id":"example.fixture@v1","displayName":"Example","description":"Example participant","kind":"service"}"#;
        let app = r#"{"format":"trellis.participant.v1","id":"example.fixture@v1","displayName":"Example","description":"Example participant","kind":"app"}"#;
        fs::write(&expected_path, service).unwrap();
        let expected = trellis_contracts::load_participant_source(&expected_path).unwrap();
        fs::write(&output_path, format!("{}\n", expected.canonical)).unwrap();
        assert!(protocol_participant_output_is_fresh(&expected, &output_path).unwrap());

        fs::write(&output_path, app).unwrap();
        assert!(!protocol_participant_output_is_fresh(&expected, &output_path).unwrap());

        fs::write(
            &output_path,
            format!(
                "{}\n",
                expected
                    .canonical
                    .replace("Example participant", "Stale description")
            ),
        )
        .unwrap();
        assert!(!protocol_participant_output_is_fresh(&expected, &output_path).unwrap());

        fs::write(&output_path, "{").unwrap();
        assert!(!protocol_participant_output_is_fresh(&expected, &output_path).unwrap());

        fs::remove_file(&output_path).unwrap();
        assert!(!protocol_participant_output_is_fresh(&expected, &output_path).unwrap());
    }
}
