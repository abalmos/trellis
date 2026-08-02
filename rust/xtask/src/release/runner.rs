use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use miette::{miette, IntoDiagnostic, Result, WrapErr};

use super::plan::{
    lane_command_specs, release_lane_name, release_lane_waves, release_plan,
    validate_selected_stage_order, validate_stage_order, verify_command_specs, CommandSpec,
    ReleaseStage, StageId,
};
use super::versioning::{parse_release_tag, version_base};
use super::{ReleaseLane, INTEGRATION_LIVE_ARTIFACTS_MANIFEST};

pub(super) fn run_verify(
    repo_root: &Path,
    version: &str,
    since: &str,
    skip_integration: bool,
    keep_workdir: bool,
) -> Result<()> {
    version_base(version)?;
    parse_release_tag(since)?;
    check_workspace_lint_policy(repo_root)?;
    if skip_integration {
        println!(
            "WARNING: --skip-integration was set; release verification is incomplete until the JS and Rust integration suites pass."
        );
    }

    let before_prepare = working_tree_snapshot(repo_root)?;
    let total_started = Instant::now();
    let specs = verify_command_specs(version, since, skip_integration, keep_workdir);
    for stage in [StageId::ReleaseMetadata, StageId::Prepare] {
        let spec = specs
            .iter()
            .find(|spec| spec.id == stage)
            .ok_or_else(|| miette!("release command graph is missing stage {stage:?}"))?;
        run_checked_stage(repo_root, spec, "release verification command failed")?;
    }
    if working_tree_snapshot(repo_root)? != before_prepare {
        return Err(miette!(
            "repository preparation changed generated output; run prepare and commit the result"
        ));
    }

    let waves = release_lane_waves(&specs, &[StageId::ReleaseMetadata, StageId::Prepare])
        .map_err(|error| miette!("{error}"))?;
    for lanes in waves {
        run_release_lanes_parallel(repo_root, &specs, &lanes, keep_workdir)?;
    }
    println!(
        "Release verification passed in {}.",
        format_elapsed(total_started.elapsed())
    );
    Ok(())
}

fn run_release_lanes_parallel(
    repo_root: &Path,
    plan: &[ReleaseStage],
    lanes: &[ReleaseLane],
    keep_workdir: bool,
) -> Result<()> {
    std::thread::scope(|scope| {
        let handles = lanes
            .iter()
            .copied()
            .map(|lane| {
                scope.spawn(move || run_release_lane_stages(repo_root, lane, plan, keep_workdir))
            })
            .collect::<Vec<_>>();
        let mut failure = None;
        for handle in handles {
            let result = handle
                .join()
                .map_err(|_| miette!("release lane panicked"))
                .and_then(|result| result);
            if failure.is_none() {
                failure = result.err();
            }
        }
        failure.map_or(Ok(()), Err)
    })
}

pub(super) fn run_release_lane(
    repo_root: &Path,
    lane: ReleaseLane,
    keep_workdir: bool,
) -> Result<()> {
    let plan = release_plan(keep_workdir);
    validate_stage_order(&plan).map_err(|error| miette!("invalid release lane plan: {error}"))?;
    let selected = lane_command_specs(lane, keep_workdir);
    validate_selected_stage_order(&plan, lane, &selected)
        .map_err(|error| miette!("invalid {} release lane: {error}", release_lane_name(lane)))?;
    run_release_lane_stages(repo_root, lane, &plan, keep_workdir)
}

fn run_release_lane_stages(
    repo_root: &Path,
    lane: ReleaseLane,
    plan: &[ReleaseStage],
    _keep_workdir: bool,
) -> Result<()> {
    let specs = plan
        .iter()
        .filter(|stage| stage.lane == Some(lane))
        .cloned()
        .collect::<Vec<_>>();
    validate_selected_stage_order(plan, lane, &specs)
        .map_err(|error| miette!("invalid {} release lane: {error}", release_lane_name(lane)))?;
    if lane == ReleaseLane::Live {
        validate_live_artifact_manifest(repo_root, &specs)?;
    }
    let started = Instant::now();
    let mut index = 0;
    while index < specs.len() {
        if lane == ReleaseLane::Live && specs[index].id == StageId::LiveArtifactValidation {
            index += 1;
            continue;
        }
        let group = specs[index].parallel_group;
        let end = group.as_ref().map_or(index + 1, |group| {
            specs[index..]
                .iter()
                .take_while(|spec| spec.parallel_group.as_ref() == Some(group))
                .count()
                + index
        });
        if group.is_some() {
            let parallel_specs = specs[index..end].iter().collect::<Vec<_>>();
            run_parallel_commands(repo_root, &parallel_specs, "release lane command failed")?;
        } else {
            run_checked_stage(repo_root, &specs[index], "release lane command failed")?;
        }
        index = end;
    }
    println!(
        "{} release lane passed in {}.",
        release_lane_name(lane),
        format_elapsed(started.elapsed())
    );
    Ok(())
}

fn validate_live_artifact_manifest(repo_root: &Path, specs: &[ReleaseStage]) -> Result<()> {
    let manifest = repo_root.join(INTEGRATION_LIVE_ARTIFACTS_MANIFEST);
    if !manifest.is_file() {
        return Err(miette!(
            "release lane live requires the live-build artifact manifest at {}; run `release lane live-build` first",
            manifest.display()
        ));
    }
    let validation = specs
        .iter()
        .find(|spec| spec.id == StageId::LiveArtifactValidation)
        .ok_or_else(|| miette!("release lane live has no artifact validation stage"))?;
    run_checked_stage(
        repo_root,
        validation,
        "required live-build artifact manifest is missing or invalid",
    )
    .map_err(|error| {
        miette!(
            "required live-build artifact manifest at {} is invalid: {error}",
            manifest.display()
        )
    })
}

pub(super) fn working_tree_snapshot(repo_root: &Path) -> Result<Vec<u8>> {
    let mut snapshot = Command::new("git")
        .args(["diff", "--binary", "--no-ext-diff", "HEAD"])
        .current_dir(repo_root)
        .output()
        .into_diagnostic()
        .wrap_err("failed to inspect generated output diff")?;
    if !snapshot.status.success() {
        return Err(miette!("git diff failed with {}", snapshot.status));
    }
    let status = Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=all", "-z"])
        .current_dir(repo_root)
        .output()
        .into_diagnostic()
        .wrap_err("failed to inspect generated output status")?;
    if !status.status.success() {
        return Err(miette!("git status failed with {}", status.status));
    }
    snapshot.stdout.extend(&status.stdout);
    for entry in status.stdout.split(|byte| *byte == 0) {
        let Some(path) = entry.strip_prefix(b"?? ") else {
            continue;
        };
        let path = std::str::from_utf8(path)
            .into_diagnostic()
            .wrap_err("untracked generated-output path is not UTF-8")?;
        let contents = fs::read(repo_root.join(path))
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to read untracked file {path}"))?;
        snapshot
            .stdout
            .extend_from_slice(&(contents.len() as u64).to_le_bytes());
        snapshot.stdout.extend(contents);
    }
    Ok(snapshot.stdout)
}

fn run_parallel_commands(repo_root: &Path, specs: &[&ReleaseStage], context: &str) -> Result<()> {
    if specs.len() > 8 {
        return Err(miette!("parallel release stage exceeds eight commands"));
    }
    let mut children: Vec<(String, std::process::Child)> = Vec::with_capacity(specs.len());
    for spec in specs {
        println!("$ {}", command_text(&spec.command));
        let child = match Command::new(&spec.command.program)
            .args(&spec.command.args)
            .current_dir(repo_root)
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                for (_, mut child) in children {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                return Err(miette!(
                    "failed to run {}: {error}",
                    command_text(&spec.command)
                ));
            }
        };
        children.push((command_text(&spec.command), child));
    }
    let mut failure = None;
    for (command, mut child) in children {
        let status = child
            .wait()
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to wait for {command}"))?;
        if !status.success() && failure.is_none() {
            failure = Some(miette!("{context}: {command} exited with {status}"));
        }
    }
    failure.map_or(Ok(()), Err)
}

pub(super) fn format_elapsed(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

// The release guide owns the reason, owner, and removal condition for every
// private implementation crate in this exception registry.
const RUST_LINT_POLICY_EXCEPTIONS: &[&str] = &[
    "tools/generate",
    "crates/local-bootstrap",
    "crates/jobs",
    "crates/jobs-runtime",
    "crates/eventlog-runtime",
    "crates/trellis-test",
    "crates/codegen-ts",
    "crates/codegen-rust",
    "crates/generate-runner",
    "crates/cli",
];

pub(super) fn check_workspace_lint_policy(repo_root: &Path) -> Result<()> {
    let members = rust_workspace_members(repo_root)?;

    for member in &members {
        let manifest_path = repo_root.join("rust").join(member).join("Cargo.toml");
        let manifest = fs::read_to_string(&manifest_path)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to read {}", manifest_path.display()))?;
        let inherits = manifest.contains("[lints]\nworkspace = true")
            || manifest.contains("[lints]\r\nworkspace = true");
        if !inherits
            && !RUST_LINT_POLICY_EXCEPTIONS
                .iter()
                .any(|exception| *exception == member)
        {
            return Err(miette!(
                "Rust workspace member `{member}` must contain `[lints]` with `workspace = true` or be added to the documented release exception list"
            ));
        }
    }
    Ok(())
}

fn rust_workspace_members(repo_root: &Path) -> Result<Vec<String>> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
            "rust/Cargo.toml",
        ])
        .current_dir(repo_root)
        .output()
        .into_diagnostic()
        .wrap_err("failed to run cargo metadata")?;
    if !output.status.success() {
        return Err(miette!(
            "cargo metadata failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
        .into_diagnostic()
        .wrap_err("cargo metadata returned invalid JSON")?;
    let workspace_member_ids = metadata
        .get("workspace_members")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| miette!("cargo metadata omitted workspace members"))?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<BTreeSet<_>>();
    let rust_root = repo_root.join("rust");
    let mut members = metadata
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| miette!("cargo metadata omitted packages"))?
        .iter()
        .filter(|package| {
            package
                .get("id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|id| workspace_member_ids.contains(id))
        })
        .map(|package| {
            let manifest = package
                .get("manifest_path")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| miette!("cargo metadata package omitted manifest path"))?;
            let relative = Path::new(manifest)
                .strip_prefix(&rust_root)
                .map_err(|_| {
                    miette!("workspace member manifest is outside rust workspace: {manifest}")
                })?
                .parent()
                .ok_or_else(|| miette!("workspace member manifest has no parent: {manifest}"))?
                .to_str()
                .ok_or_else(|| miette!("workspace member path is not UTF-8: {manifest}"))?
                .to_owned();
            Ok(relative)
        })
        .collect::<Result<Vec<_>>>()?;
    members.sort();
    Ok(members)
}

fn run_checked_stage(repo_root: &Path, spec: &ReleaseStage, context: &str) -> Result<()> {
    run_checked_command(repo_root, &spec.command, context)
}

pub(super) fn run_checked_command(
    repo_root: &Path,
    spec: &CommandSpec,
    context: &str,
) -> Result<()> {
    println!("$ {}", command_text(spec));
    let status = Command::new(&spec.program)
        .args(&spec.args)
        .current_dir(repo_root)
        .status()
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to run {}", command_text(spec)))?;
    if status.success() {
        Ok(())
    } else {
        Err(miette!(
            "{context}: {} exited with {status}",
            command_text(spec)
        ))
    }
}

pub(super) fn run_output_command(
    repo_root: &Path,
    spec: &CommandSpec,
) -> Result<std::process::Output> {
    println!("$ {}", command_text(spec));
    Command::new(&spec.program)
        .args(&spec.args)
        .current_dir(repo_root)
        .output()
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to run {}", command_text(spec)))
}

pub(super) fn command_text(spec: &CommandSpec) -> String {
    std::iter::once(spec.program.as_str())
        .chain(spec.args.iter().map(String::as_str))
        .map(shell_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_word(word: &str) -> String {
    if word
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | '=' | ':'))
    {
        word.to_string()
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::time::Instant;

    use super::super::plan::{release_plan, CommandSpec, StageId};
    use super::{run_parallel_commands, run_release_lane_stages};
    use crate::release::ReleaseLane;

    #[cfg(unix)]
    #[test]
    fn spawn_failure_reaps_already_started_children() {
        let long_running =
            CommandSpec::new("sh", ["-c", "exec sleep 30"]).stage(StageId::DenoFormatting);
        let missing = CommandSpec::new(
            "/definitely/missing/trellis-release-command",
            Vec::<String>::new(),
        )
        .stage(StageId::RustWorkspaceFormatting);
        let specs = [&long_running, &missing];

        let started = Instant::now();
        let error = run_parallel_commands(Path::new("."), &specs, "parallel command failed")
            .expect_err("spawn failure should be returned");

        assert!(started.elapsed().as_secs() < 5);
        assert!(error.to_string().contains("definitely/missing"));
    }

    #[test]
    fn live_lane_rejects_missing_artifact_manifest_before_running_commands() {
        let root = temp_release_root("missing-manifest");
        let error = run_release_lane_stages(&root, ReleaseLane::Live, &release_plan(false), false)
            .expect_err("live lane should require the build manifest");

        assert!(error
            .to_string()
            .contains("requires the live-build artifact manifest"));
        fs::remove_dir_all(root).expect("remove temp release root");
    }

    #[cfg(unix)]
    #[test]
    fn live_lane_rejects_invalid_artifact_manifest_before_shared_integration() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_release_root("invalid-manifest");
        let manifest = root.join("dist/integration-runtime/manifest.json");
        fs::create_dir_all(manifest.parent().expect("manifest parent")).expect("create manifest");
        fs::write(&manifest, "invalid").expect("write invalid manifest");

        let bin = root.join("bin");
        fs::create_dir_all(&bin).expect("create fake command directory");
        let deno = bin.join("deno");
        fs::write(
            &deno,
            "#!/bin/sh
case \"$*\" in
  *--inventory-only*)
    test \"$(cat \"$PWD/dist/integration-runtime/manifest.json\")\" = invalid || exit 2
    printf '%s\\n' 'invalid live artifact manifest' >&2
    exit 1
    ;;
  *--prebuilt-only*)
    : > \"$PWD/shared-live-ran\"
    ;;
esac
exit 0
",
        )
        .expect("write fake deno");
        fs::set_permissions(&deno, fs::Permissions::from_mode(0o755))
            .expect("make fake deno executable");

        let previous_path = env::var_os("PATH");
        let mut paths = vec![bin];
        if let Some(path) = previous_path.as_ref() {
            paths.extend(env::split_paths(path));
        }
        env::set_var("PATH", env::join_paths(paths).expect("join PATH"));
        let result = run_release_lane_stages(&root, ReleaseLane::Live, &release_plan(false), false);
        match previous_path {
            Some(path) => env::set_var("PATH", path),
            None => env::remove_var("PATH"),
        }

        let error = result.expect_err("invalid manifest should stop the live lane");
        assert!(error
            .to_string()
            .contains("required live-build artifact manifest"));
        assert!(!root.join("shared-live-ran").exists());
        fs::remove_dir_all(root).expect("remove temp release root");
    }

    fn temp_release_root(name: &str) -> PathBuf {
        let root = env::temp_dir().join(format!(
            "trellis-release-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create temp release root");
        root
    }
}
