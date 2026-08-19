use std::path::{Path, PathBuf};

#[cfg(test)]
use clap::Parser;
use clap::{Subcommand, ValueEnum};
use miette::{miette, Result};

mod changelog;
mod github;
mod plan;
mod runner;
mod versioning;

use changelog::{check_changelog, write_release_notes};
use github::run_pretag_check;
use runner::{run_release_lane, run_verify};
use versioning::{
    bump_versions, check_versions, display_repo_path, parse_release_tag, prepare_release,
    require_stable_version, version_base, write_github_env,
};

const RELEASE_JS_INTERNAL_NPM_VERSION_FILES: &[&str] = &[
    "ts/packages/trellis/scripts/build_npm.ts",
    "ts/packages/trellis-svelte/scripts/build_npm.ts",
    "ts/packages/trellis/tests/publishing_targets_test.ts",
];
const INTEGRATION_LIVE_ARTIFACTS_MANIFEST: &str = "dist/integration-runtime/manifest.json";

#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
pub(crate) enum ReleaseCommand {
    #[command(name = "check-versions")]
    CheckVersions,
    #[command(name = "prepare")]
    Prepare {
        #[arg(long)]
        tag: Option<String>,
    },
    #[command(name = "bump")]
    Bump {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },
    #[command(name = "changelog-check")]
    ChangelogCheck {
        #[arg(long)]
        version: String,
        #[arg(long)]
        since: Option<String>,
    },
    #[command(name = "write-notes")]
    WriteNotes {
        #[arg(long)]
        tag: String,
        #[arg(long)]
        output: PathBuf,
    },
    #[command(name = "check-metadata")]
    CheckMetadata {
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        since: Option<String>,
    },
    #[command(name = "pretag-check")]
    PretagCheck {
        #[arg(long)]
        tag: String,
        #[arg(long = "ref", default_value = "main", value_parser = normalize_git_ref)]
        git_ref: String,
    },
    #[command(name = "lane")]
    Lane {
        #[arg(value_enum)]
        lane: ReleaseLane,
    },
    #[command(name = "verify")]
    Verify {
        #[arg(long)]
        version: String,
        #[arg(long)]
        since: String,
        #[arg(long)]
        skip_integration: bool,
        #[arg(long)]
        keep_workdir: bool,
    },
}

fn normalize_git_ref(value: &str) -> std::result::Result<String, String> {
    if value.trim().is_empty() {
        Ok("main".to_string())
    } else {
        Ok(value.to_string())
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, ValueEnum)]
pub(crate) enum ReleaseLane {
    #[value(name = "static")]
    Static,
    #[value(name = "rust")]
    Rust,
    #[value(name = "typescript")]
    TypeScript,
    #[value(name = "live-build")]
    LiveBuild,
    #[value(name = "live")]
    Live,
}

#[cfg(test)]
#[derive(Debug, Parser)]
#[command(name = "release")]
struct ReleaseCli {
    #[command(subcommand)]
    command: ReleaseCommand,
}

#[cfg(test)]
pub(crate) fn parse_release_command<I>(args: I) -> Result<ReleaseCommand>
where
    I: Iterator<Item = String>,
{
    let argv = std::iter::once("release".to_string()).chain(args);
    let command = ReleaseCli::try_parse_from(argv)
        .map_err(|error| miette!("{error}"))?
        .command;
    validate_release_command(&command)?;
    Ok(command)
}

pub(crate) fn validate_release_command(command: &ReleaseCommand) -> Result<()> {
    match command {
        ReleaseCommand::PretagCheck { tag, .. } => {
            parse_release_tag(tag)?;
        }
        ReleaseCommand::Verify { version, since, .. } => {
            version_base(version)?;
            parse_release_tag(since)?;
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn run_release(repo_root: &Path, command: ReleaseCommand) -> Result<()> {
    match command {
        ReleaseCommand::CheckVersions => {
            let version = check_versions(repo_root)?;
            println!("All release-managed Trellis versions are {version}.");
            Ok(())
        }
        ReleaseCommand::Prepare { tag } => {
            let Some(tag) = tag.filter(|tag| !tag.trim().is_empty()) else {
                println!("release tag is not set; skipping release version preparation.");
                return Ok(());
            };
            let release = parse_release_tag(&tag)?;
            let changed = prepare_release(repo_root, &release)?;
            write_github_env("TRELLIS_RELEASE_VERSION", &release.version)?;
            write_github_env("TRELLIS_RELEASE_BASE_VERSION", &release.base_version)?;
            println!(
                "Prepared release version {} from tag {tag} in {} file(s).",
                release.version,
                changed.len()
            );
            for path in changed {
                println!("- {}", display_repo_path(repo_root, &path));
            }
            Ok(())
        }
        ReleaseCommand::Bump { from, to } => {
            require_stable_version(&from, "--from")?;
            require_stable_version(&to, "--to")?;
            let changed = bump_versions(repo_root, &from, &to)?;
            println!(
                "Bumped release-managed Trellis versions from {from} to {to} in {} file(s).",
                changed.len()
            );
            for path in changed {
                println!("- {}", display_repo_path(repo_root, &path));
            }
            Ok(())
        }
        ReleaseCommand::ChangelogCheck { version, since } => {
            check_changelog(repo_root, &version, since.as_deref())?;
            Ok(())
        }
        ReleaseCommand::WriteNotes { tag, output } => {
            let release = parse_release_tag(&tag)?;
            write_release_notes(repo_root, &release.version, &output)?;
            println!("Wrote release notes for {tag} to {}.", output.display());
            Ok(())
        }
        ReleaseCommand::CheckMetadata { version, since } => {
            let checked_version = check_versions(repo_root)?;
            if let Some(version) = version {
                let version_base = version_base(&version)?;
                if version != checked_version && version_base != checked_version {
                    return Err(miette!(
                        "requested release version {version} has base version {version_base}, but release-managed versions use {checked_version}"
                    ));
                }
                check_changelog(repo_root, &version, since.as_deref())?;
            }
            println!("Release metadata verification passed for {checked_version}.");
            println!(
                "Before publishing, run `release verify` locally or use the GitHub release gate."
            );
            Ok(())
        }
        ReleaseCommand::PretagCheck { tag, git_ref } => run_pretag_check(repo_root, &tag, &git_ref),
        ReleaseCommand::Lane { lane } => run_release_lane(repo_root, lane, false),
        ReleaseCommand::Verify {
            version,
            since,
            skip_integration,
            keep_workdir,
        } => run_verify(repo_root, &version, &since, skip_integration, keep_workdir),
    }
}

#[cfg(test)]
mod tests {
    use super::changelog::extract_changelog_section;
    use super::github::{pretag_dispatch_command, pretag_list_command, pretag_watch_command};
    use super::plan::lane_command_specs;
    use super::runner::{command_text, format_elapsed, working_tree_snapshot};
    use super::versioning::{
        collect_versions, prepare_release, rewrite_cargo_manifest_versions,
        rewrite_cargo_manifest_versions_for_release, rewrite_js_internal_npm_dependency_versions,
        rewrite_json_manifest_internal_jsr_dependency_versions, rewrite_json_manifest_version,
        version_base, ReleaseVersion,
    };
    use super::{parse_release_command, ReleaseCommand, ReleaseLane};
    use std::time::Duration;
    use std::fs;

    #[test]
    fn parse_release_bump_command() {
        let command = parse_release_command(
            ["bump", "--from", "0.8.2", "--to", "0.9.0"]
                .into_iter()
                .map(str::to_string),
        )
        .expect("parse release bump");
        assert_eq!(
            command,
            ReleaseCommand::Bump {
                from: "0.8.2".to_string(),
                to: "0.9.0".to_string(),
            }
        );
    }

    #[test]
    fn parse_release_prepare_command() {
        assert_eq!(
            parse_release_command(["prepare", "--tag", "v0.9.0-rc.1"].into_iter().map(str::to_string))
                .expect("parse release prepare"),
            ReleaseCommand::Prepare {
                tag: Some("v0.9.0-rc.1".to_string())
            }
        );
    }

    #[test]
    fn parse_release_pretag_check_defaults_ref() {
        assert_eq!(
            parse_release_command(["pretag-check", "--tag", "v0.9.0-rc.1"].into_iter().map(str::to_string))
                .expect("parse release pretag-check"),
            ReleaseCommand::PretagCheck {
                tag: "v0.9.0-rc.1".to_string(),
                git_ref: "main".to_string(),
            }
        );
    }

    #[test]
    fn parse_release_pretag_check_rejects_invalid_tag() {
        let error = parse_release_command(
            ["pretag-check", "--tag", "0.9.0"]
                .into_iter()
                .map(str::to_string),
        )
        .expect_err("pretag-check should reject invalid tag");
        assert!(error.to_string().contains("invalid release tag"));
    }

    #[test]
    fn parse_release_verify_command() {
        assert_eq!(
            parse_release_command(
                [
                    "verify",
                    "--version",
                    "0.9.0-rc.1",
                    "--since",
                    "v0.8.2",
                    "--skip-integration",
                    "--keep-workdir",
                ]
                .into_iter()
                .map(str::to_string),
            )
            .expect("parse release verify"),
            ReleaseCommand::Verify {
                version: "0.9.0-rc.1".to_string(),
                since: "v0.8.2".to_string(),
                skip_integration: true,
                keep_workdir: true,
            }
        );
    }

    #[test]
    fn parse_release_verify_requires_release_tag_since() {
        let error = parse_release_command(
            ["verify", "--version", "0.9.0", "--since", "0.8.2"]
                .into_iter()
                .map(str::to_string),
        )
        .expect_err("verify should reject non-tag since");
        assert!(error.to_string().contains("invalid release tag"));
    }

    #[test]
    fn parse_named_release_lane() {
        assert_eq!(
            parse_release_command(["lane", "live-build"].into_iter().map(str::to_owned)).unwrap(),
            ReleaseCommand::Lane {
                lane: ReleaseLane::LiveBuild,
            }
        );
    }

    #[test]
    fn pretag_check_command_specs_construct_gh_invocations() {
        assert_eq!(
            command_text(&pretag_dispatch_command("v0.9.0", "main")),
            "gh workflow run .github/workflows/release.yml --ref main -f tag=v0.9.0 -f publish=false"
        );
        assert_eq!(
            command_text(&pretag_list_command("main")),
            "gh run list --workflow .github/workflows/release.yml --event workflow_dispatch --branch main --limit 20 --json databaseId --jq '.[].databaseId'"
        );
        assert_eq!(
            command_text(&pretag_watch_command("12345")),
            "gh run watch 12345 --exit-status"
        );
    }

    #[test]
    fn release_lanes_are_direct_commands() {
        let live = lane_command_specs(ReleaseLane::Live, false);
        assert_eq!(live.len(), 1);
        let live = command_text(&live[0]);
        assert!(live.contains("integration/live_runner.ts --prebuilt-only"));
        assert!(!live.contains("--inventory-only"));
        assert!(!live.contains("--jobs 20"));

        let keep = command_text(&lane_command_specs(ReleaseLane::Live, true)[0]);
        assert!(keep.starts_with("env TRELLIS_TEST_KEEP_WORKDIR=1 deno "));

        let build = command_text(&lane_command_specs(ReleaseLane::LiveBuild, false)[0]);
        assert!(build.contains("integration/live_runner.ts --build-only"));
        assert!(lane_command_specs(ReleaseLane::Static, false)
            .iter()
            .any(|command| command.program == "actionlint"));
        assert!(lane_command_specs(ReleaseLane::Rust, false)
            .iter()
            .any(|command| command.args.first().is_some_and(|arg| arg == "clippy")));
    }

    #[test]
    fn release_timing_uses_minute_second_format() {
        assert_eq!(format_elapsed(Duration::from_secs(0)), "00:00");
        assert_eq!(format_elapsed(Duration::from_secs(125)), "02:05");
    }

    #[test]
    fn working_tree_snapshot_includes_untracked_file_contents() {
        let root = std::env::temp_dir().join(format!(
            "trellis-release-snapshot-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let git = |args: &[&str]| {
            assert!(std::process::Command::new("git")
                .args(args)
                .current_dir(&root)
                .status()
                .unwrap()
                .success());
        };
        git(&["init", "--quiet"]);
        git(&["config", "user.name", "Trellis Test"]);
        git(&["config", "user.email", "test@trellis.invalid"]);
        git(&["config", "commit.gpgsign", "false"]);
        fs::write(root.join("tracked"), "tracked").unwrap();
        git(&["add", "tracked"]);
        git(&["commit", "--quiet", "-m", "initial"]);
        fs::write(root.join("generated"), "before").unwrap();
        let before = working_tree_snapshot(&root).unwrap();
        fs::write(root.join("generated"), "after").unwrap();
        let after = working_tree_snapshot(&root).unwrap();
        assert_ne!(before, after);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rewrite_json_manifest_preserves_layout() {
        let original = "{\n  \"name\": \"@qlever-llc/trellis\",\n  \"version\": \"0.8.2\"\n}\n";
        let updated = rewrite_json_manifest_version(
            original,
            "0.8.2",
            "0.9.0",
            std::path::Path::new("deno.json"),
        )
        .expect("rewrite json version");
        assert_eq!(
            updated,
            "{\n  \"name\": \"@qlever-llc/trellis\",\n  \"version\": \"0.9.0\"\n}\n"
        );
    }

    #[test]
    fn rewrite_json_manifest_updates_internal_jsr_dependencies() {
        let original = "{\n  \"imports\": {\n    \"@qlever-llc/trellis\": \"jsr:@qlever-llc/trellis@^0.8.2\",\n    \"@qlever-llc/trellis/sdk/auth\": \"jsr:@qlever-llc/trellis@^0.8.2/sdk/auth\",\n    \"@std/path\": \"jsr:@std/path@^1.1.4\"\n  }\n}\n";
        let updated = rewrite_json_manifest_internal_jsr_dependency_versions(
            original,
            "0.8.2",
            "0.8.2-rc.1",
            std::path::Path::new("deno.json"),
        )
        .expect("rewrite jsr dependencies");
        assert!(updated.contains("@qlever-llc/trellis@^0.8.2-rc.1"));
        assert!(updated.contains("@std/path@^1.1.4"));
    }

    #[test]
    fn prepare_release_updates_internal_jsr_dependency_versions() {
        let root = temp_repo_root();
        let manifest = root.join("ts/packages/trellis-test/deno.json");
        fs::create_dir_all(manifest.parent().expect("manifest parent"))
            .expect("mkdir manifest parent");
        fs::write(
            &manifest,
            "{\n  \"name\": \"@qlever-llc/trellis-test\",\n  \"version\": \"0.8.2\",\n  \"imports\": {\n    \"@qlever-llc/trellis\": \"jsr:@qlever-llc/trellis@^0.8.2\"\n  }\n}\n",
        )
        .expect("write manifest");
        prepare_release(
            &root,
            &ReleaseVersion {
                version: "0.8.2-rc.1".to_string(),
                base_version: "0.8.2".to_string(),
            },
        )
        .expect("prepare release");
        assert!(fs::read_to_string(&manifest)
            .expect("read updated manifest")
            .contains("0.8.2-rc.1"));
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn rewrite_cargo_manifest_updates_workspace_and_internal_dependencies() {
        let original = "[workspace.package]\nversion = \"0.8.2\"\n\n[dependencies]\ntrellis-rs = { path = \"../trellis\", version = \"0.8.2\" }\nserde = { version = \"1.0\" }\n";
        let updated = rewrite_cargo_manifest_versions(
            original,
            "0.8.2",
            "0.9.0",
            std::path::Path::new("Cargo.toml"),
        )
        .expect("rewrite cargo versions");
        assert!(updated.contains("version = \"0.9.0\""));
        assert!(updated.contains("serde = { version = \"1.0\" }"));
    }

    #[test]
    fn rewrite_cargo_manifest_preserves_non_release_sentinel_version() {
        let original = "[package]\nname = \"trellis-sdk-console\"\nversion = \"0.0.0\"\n\n[dependencies]\ntrellis-rs = { path = \"../trellis\", version = \"0.8.2\" }\n";
        let updated = rewrite_cargo_manifest_versions(
            original,
            "0.8.2",
            "0.9.0",
            std::path::Path::new("Cargo.toml"),
        )
        .expect("rewrite cargo versions");
        assert!(updated.contains("version = \"0.0.0\""));
        assert!(updated.contains("version = \"0.9.0\""));
    }

    #[test]
    fn rewrite_cargo_manifest_for_release_updates_generated_sdk_dependencies() {
        let original = "[workspace.package]\nversion = \"0.8.2\"\n\n[dependencies]\ntrellis-rs = { path = \"../trellis\", version = \"0.8.2\" }\ntrellis-sdk-state = { path = \"../generated/packages/cargo/state\", version = \"0.8.2\" }\n";
        let updated = rewrite_cargo_manifest_versions_for_release(
            original,
            "0.8.2-rc.1",
            "0.8.2",
            std::path::Path::new("Cargo.toml"),
        )
        .expect("rewrite cargo release versions");
        assert!(updated.contains("0.8.2-rc.1"));
    }

    #[test]
    fn rewrite_js_internal_npm_dependency_versions_updates_build_scripts() {
        let original = "const dependencies = {\n  \"@qlever-llc/result\": \"^0.8.2\",\n  \"@qlever-llc/trellis\": \"~0.8.2\",\n};\n";
        let updated = rewrite_js_internal_npm_dependency_versions(
            original,
            "0.8.2",
            "0.9.0",
            std::path::Path::new("build_npm.ts"),
        )
        .expect("rewrite js internal npm dependencies");
        assert!(updated.contains("^0.9.0"));
        assert!(updated.contains("~0.9.0"));
    }

    #[test]
    fn collect_versions_skips_zero_version_apps() {
        let root = temp_repo_root();
        fs::create_dir_all(root.join("ts/packages/trellis")).expect("mkdir package");
        fs::create_dir_all(root.join("ts/apps/console")).expect("mkdir app");
        fs::create_dir_all(root.join("rust")).expect("mkdir rust");
        fs::write(
            root.join("ts/packages/trellis/deno.json"),
            "{\"version\":\"0.8.2\"}\n",
        )
        .expect("write package manifest");
        fs::write(
            root.join("ts/apps/console/deno.json"),
            "{\"version\":\"0.0.0\"}\n",
        )
        .expect("write app manifest");
        fs::write(
            root.join("rust/Cargo.toml"),
            "[workspace.package]\nversion = \"0.8.2\"\n",
        )
        .expect("write cargo manifest");
        let versions = collect_versions(&root).expect("collect versions");
        assert_eq!(versions.len(), 2);
        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn extract_changelog_section_finds_dated_heading() {
        let section = extract_changelog_section(
            "# Changelog\n\n## [0.9.0] - 2026-05-19\n\n### Added\n\n- Thing\n\n## [0.8.2]\n",
            "0.9.0",
        )
        .expect("extract changelog");
        assert!(section.contains("Thing"));
    }

    #[test]
    fn version_base_accepts_prerelease_versions() {
        assert_eq!(version_base("0.9.0-rc.1").expect("version base"), "0.9.0");
    }

    fn temp_repo_root() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "trellis-release-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        path
    }
}
