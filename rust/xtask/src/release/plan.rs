use super::{ReleaseLane, INTEGRATION_LIVE_ARTIFACTS_MANIFEST};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct CommandSpec {
    pub(super) program: String,
    pub(super) args: Vec<String>,
}

impl CommandSpec {
    pub(super) fn new<S, I>(program: &str, args: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        Self {
            program: program.to_string(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }
}

pub(super) const fn release_lane_name(lane: ReleaseLane) -> &'static str {
    match lane {
        ReleaseLane::Static => "static",
        ReleaseLane::Rust => "rust",
        ReleaseLane::TypeScript => "typescript",
        ReleaseLane::LiveBuild => "live-build",
        ReleaseLane::Live => "live",
    }
}

pub(super) fn lane_command_specs(lane: ReleaseLane, keep_workdir: bool) -> Vec<CommandSpec> {
    match lane {
        ReleaseLane::Static => vec![
            CommandSpec::new("deno", ["fmt", "-c", "ts/deno.json", "--check"]),
            CommandSpec::new(
                "cargo",
                [
                    "fmt",
                    "--manifest-path",
                    "rust/Cargo.toml",
                    "--all",
                    "--check",
                ],
            ),
            CommandSpec::new(
                "cargo",
                [
                    "fmt",
                    "--manifest-path",
                    "rust/tools/generate/Cargo.toml",
                    "--check",
                ],
            ),
            CommandSpec::new(
                "cargo",
                ["fmt", "--manifest-path", "rust/xtask/Cargo.toml", "--check"],
            ),
            CommandSpec::new("actionlint", Vec::<String>::new()),
            CommandSpec::new(
                "deno",
                [
                    "check",
                    "-c",
                    "ts/deno.json",
                    "ts/packages/trellis/index.ts",
                    "ts/packages/trellis-svelte/src/index.ts",
                    "ts/packages/trellis-svelte/src/context.svelte.ts",
                ],
            ),
        ],
        ReleaseLane::Rust => vec![
            CommandSpec::new(
                "cargo",
                [
                    "clippy",
                    "--manifest-path",
                    "rust/Cargo.toml",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
            ),
            CommandSpec::new(
                "cargo",
                [
                    "check",
                    "--manifest-path",
                    "generated/packages/cargo-participants/jobs/Cargo.toml",
                ],
            ),
            CommandSpec::new(
                "cargo",
                [
                    "check",
                    "--manifest-path",
                    "rust/Cargo.toml",
                    "--package",
                    "trellis-protocol-wasm",
                    "--target",
                    "wasm32-unknown-unknown",
                ],
            ),
            CommandSpec::new(
                "cargo",
                [
                    "test",
                    "--manifest-path",
                    "rust/Cargo.toml",
                    "--workspace",
                    "--no-run",
                ],
            ),
            CommandSpec::new(
                "cargo",
                [
                    "test",
                    "--manifest-path",
                    "rust/Cargo.toml",
                    "--lib",
                    "-p",
                    "trellis-protocol",
                    "-p",
                    "trellis-contracts",
                    "-p",
                    "trellis-codegen-ts",
                    "-p",
                    "trellis-codegen-rust",
                    "-p",
                    "trellis-bootstrap",
                    "-p",
                    "trellis-local-bootstrap",
                    "-p",
                    "trellis-generate-runner",
                    "-p",
                    "trellis-cli",
                    "-p",
                    "trellis-local-nats",
                ],
            ),
            CommandSpec::new(
                "env",
                [
                    "RUSTDOCFLAGS=-D warnings",
                    "cargo",
                    "doc",
                    "--manifest-path",
                    "rust/Cargo.toml",
                    "--workspace",
                    "--no-deps",
                ],
            ),
            CommandSpec::new(
                "cargo",
                [
                    "test",
                    "--manifest-path",
                    "rust/Cargo.toml",
                    "--workspace",
                    "--doc",
                ],
            ),
            CommandSpec::new(
                "cargo",
                [
                    "package",
                    "--manifest-path",
                    "rust/Cargo.toml",
                    "--package",
                    "trellis-protocol",
                    "--allow-dirty",
                ],
            ),
            CommandSpec::new(
                "cargo",
                ["test", "--manifest-path", "rust/tools/generate/Cargo.toml"],
            ),
            CommandSpec::new(
                "cargo",
                ["test", "--manifest-path", "rust/xtask/Cargo.toml"],
            ),
            CommandSpec::new("cargo", ["test", "--manifest-path", "xtask/Cargo.toml"]),
        ],
        ReleaseLane::TypeScript => vec![
            CommandSpec::new("deno", ["task", "-c", "ts/deno.json", "packages:build:npm"]),
            CommandSpec::new(
                "deno",
                ["task", "-c", "ts/deno.json", "test:prepared:result"],
            ),
            CommandSpec::new(
                "deno",
                ["task", "-c", "ts/deno.json", "test:prepared:trellis"],
            ),
            CommandSpec::new(
                "deno",
                ["task", "-c", "ts/deno.json", "test:prepared:trellis-svelte"],
            ),
            CommandSpec::new(
                "deno",
                ["task", "-c", "ts/deno.json", "test:prepared:trellis-test"],
            ),
            CommandSpec::new(
                "deno",
                ["task", "-c", "ts/deno.json", "test:prepared:ui-tools"],
            ),
            CommandSpec::new(
                "deno",
                [
                    "task",
                    "-c",
                    "ts/deno.json",
                    "test:prepared:packaging:built",
                ],
            ),
            CommandSpec::new("bash", ["scripts/release-ts-dry-run.sh"]),
        ],
        ReleaseLane::LiveBuild => vec![CommandSpec::new(
            "deno",
            [
                "run",
                "-A",
                "-c",
                "ts/deno.json",
                "integration/live_runner.ts",
                "--build-only",
                "--artifacts-manifest",
                INTEGRATION_LIVE_ARTIFACTS_MANIFEST,
            ],
        )],
        ReleaseLane::Live => {
            let live = [
                "run",
                "-A",
                "-c",
                "ts/deno.json",
                "integration/live_runner.ts",
                "--prebuilt-only",
                "--artifacts-manifest",
                INTEGRATION_LIVE_ARTIFACTS_MANIFEST,
            ];
            if keep_workdir {
                let mut args = vec![
                    "TRELLIS_TEST_KEEP_WORKDIR=1".to_string(),
                    "deno".to_string(),
                ];
                args.extend(live.into_iter().map(str::to_string));
                vec![CommandSpec::new("env", args)]
            } else {
                vec![CommandSpec::new("deno", live)]
            }
        }
    }
}
