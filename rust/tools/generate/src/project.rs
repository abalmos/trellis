use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use miette::IntoDiagnostic;

use crate::discovery::discover_contracts;
use crate::model::PackageTarget;
use crate::planning::{build_auto_plan_with_targets, execute_auto_plan, AutoExecutionSummary};

/// Generate one project's local API, SDK, participant, and facade artifacts.
pub fn generate_project(
    root: &Path,
    output_root: &Path,
    locked_api_digests: BTreeMap<String, String>,
) -> miette::Result<AutoExecutionSummary> {
    let discovered = discover_contracts(root)?;
    let plan = build_auto_plan_with_targets(
        discovered,
        Some(output_root),
        Some(&[
            PackageTarget::Api,
            PackageTarget::TypeScript,
            PackageTarget::Cargo,
        ]),
        &locked_api_digests,
    )?;
    let summary = if plan.is_empty() {
        AutoExecutionSummary::default()
    } else {
        let runtime_version = crate::artifacts::trellis_package_version();
        execute_auto_plan(
            &plan,
            locked_api_digests,
            None,
            false,
            false,
            "@trellis-sdk/",
            Some(&runtime_version),
        )?
    };

    for (directory, current) in [
        (
            output_root.join("generated/protocol/apis"),
            plan.iter()
                .filter_map(|entry| entry.out_api.clone())
                .flat_map(|path| {
                    [
                        path.clone(),
                        crate::artifacts::generated_artifacts_metadata_path(&path),
                    ]
                })
                .collect(),
        ),
        (
            output_root.join("generated/protocol/participants"),
            plan.iter()
                .filter_map(|entry| entry.protocol_participant_out.clone())
                .collect(),
        ),
        (
            output_root.join("generated/packages/jsr"),
            plan.iter()
                .filter_map(|entry| entry.jsr_out.clone())
                .collect(),
        ),
        (output_root.join("generated/packages/npm"), BTreeSet::new()),
        (
            output_root.join("generated/packages/cargo"),
            plan.iter()
                .filter_map(|entry| entry.cargo_out.clone())
                .collect(),
        ),
        (
            output_root.join("generated/packages/cargo-participants"),
            plan.iter()
                .filter_map(|entry| entry.cargo_participant_out.clone())
                .collect(),
        ),
    ] {
        prune_removed_outputs(&directory, &current)?;
    }

    Ok(summary)
}

fn prune_removed_outputs(directory: &Path, current: &BTreeSet<PathBuf>) -> miette::Result<()> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Ok(());
    };
    for entry in entries {
        let path = entry.into_diagnostic()?.path();
        if current.contains(&path) {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(path).into_diagnostic()?;
        } else {
            fs::remove_file(path).into_diagnostic()?;
        }
    }
    Ok(())
}
