use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use miette::{miette, IntoDiagnostic, Result, WrapErr};

use super::plan::{lane_command_specs, release_lane_name, CommandSpec};
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
    if skip_integration {
        println!(
            "WARNING: --skip-integration was set; release verification is incomplete until the live suite passes."
        );
    }

    let before_prepare = working_tree_snapshot(repo_root)?;
    let total_started = Instant::now();
    run_checked_command(
        repo_root,
        &CommandSpec::new(
            "cargo",
            [
                "run",
                "--manifest-path",
                "xtask/Cargo.toml",
                "--",
                "release",
                "check-metadata",
                "--version",
                version,
                "--since",
                since,
            ],
        ),
        "release metadata verification failed",
    )?;
    run_checked_command(
        repo_root,
        &CommandSpec::new(
            "cargo",
            ["run", "--manifest-path", "xtask/Cargo.toml", "--", "prepare"],
        ),
        "repository preparation failed",
    )?;
    if working_tree_snapshot(repo_root)? != before_prepare {
        return Err(miette!(
            "repository preparation changed generated output; run prepare and commit the result"
        ));
    }

    let lanes = if skip_integration {
        vec![ReleaseLane::Static, ReleaseLane::Rust, ReleaseLane::TypeScript]
    } else {
        vec![
            ReleaseLane::Static,
            ReleaseLane::Rust,
            ReleaseLane::TypeScript,
            ReleaseLane::LiveBuild,
        ]
    };
    run_release_lanes_parallel(repo_root, &lanes, keep_workdir)?;
    if !skip_integration {
        run_release_lane(repo_root, ReleaseLane::Live, keep_workdir)?;
    }

    println!(
        "Release verification passed in {}.",
        format_elapsed(total_started.elapsed())
    );
    Ok(())
}

fn run_release_lanes_parallel(
    repo_root: &Path,
    lanes: &[ReleaseLane],
    keep_workdir: bool,
) -> Result<()> {
    std::thread::scope(|scope| {
        let handles = lanes
            .iter()
            .copied()
            .map(|lane| scope.spawn(move || run_release_lane(repo_root, lane, keep_workdir)))
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
    if lane == ReleaseLane::Live {
        let manifest = repo_root.join(INTEGRATION_LIVE_ARTIFACTS_MANIFEST);
        if !manifest.is_file() {
            return Err(miette!(
                "release lane live requires the live-build artifact manifest at {}; run `release lane live-build` first",
                manifest.display()
            ));
        }
    }

    let started = Instant::now();
    for command in lane_command_specs(lane, keep_workdir) {
        run_checked_command(repo_root, &command, "release lane command failed")?;
    }
    println!(
        "{} release lane passed in {}.",
        release_lane_name(lane),
        format_elapsed(started.elapsed())
    );
    Ok(())
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

pub(super) fn format_elapsed(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
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
