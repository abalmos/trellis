use std::collections::BTreeSet;

use super::{ReleaseLane, INTEGRATION_LIVE_ARTIFACTS_MANIFEST};

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum StageId {
    ReleaseMetadata,
    Prepare,
    DenoFormatting,
    RustWorkspaceFormatting,
    GeneratorFormatting,
    RustXtaskFormatting,
    WorkspaceClippy,
    GeneratedRustFacade,
    ProtocolWasm,
    Actionlint,
    TypeScriptCompile,
    NpmPackageBuild,
    PreparedResult,
    PreparedTrellis,
    PreparedTrellisSvelte,
    PreparedTrellisTest,
    PreparedUiTools,
    NpmPackagingSmoke,
    PackagePublicationDryRuns,
    WorkspaceTestCompileCoverage,
    CuratedPureRustTests,
    Rustdoc,
    Doctests,
    RustPackaging,
    GeneratorTests,
    RustXtaskTests,
    RootXtaskTests,
    LiveBuild,
    LiveArtifactValidation,
    SharedLiveIntegration,
}

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

    pub(super) fn stage(self, id: StageId) -> ReleaseStage {
        ReleaseStage {
            id,
            lane: release_lane_for_stage(id),
            command: self,
            dependencies: stage_dependencies(id),
            parallel_group: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ParallelGroup {
    PreparedJavaScript,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct ReleaseStage {
    pub(super) id: StageId,
    pub(super) lane: Option<ReleaseLane>,
    pub(super) command: CommandSpec,
    pub(super) dependencies: &'static [StageId],
    pub(super) parallel_group: Option<ParallelGroup>,
}

impl ReleaseStage {
    pub(super) fn parallel(mut self, group: ParallelGroup) -> Self {
        self.parallel_group = Some(group);
        self
    }
}

const NO_DEPENDENCIES: &[StageId] = &[];
const PREPARE_DEPENDENCY: &[StageId] = &[StageId::Prepare];
const NPM_BUILD_DEPENDENCY: &[StageId] = &[StageId::NpmPackageBuild];
const PREPARED_JAVASCRIPT_DEPENDENCIES: &[StageId] = &[
    StageId::PreparedResult,
    StageId::PreparedTrellis,
    StageId::PreparedTrellisSvelte,
    StageId::PreparedTrellisTest,
    StageId::PreparedUiTools,
];
const NPM_PACKAGING_SMOKE_DEPENDENCY: &[StageId] = &[StageId::NpmPackagingSmoke];
const LIVE_BUILD_DEPENDENCY: &[StageId] = &[StageId::LiveBuild];

const fn stage_dependencies(id: StageId) -> &'static [StageId] {
    match id {
        StageId::ReleaseMetadata => NO_DEPENDENCIES,
        StageId::Prepare => &[StageId::ReleaseMetadata],
        StageId::PreparedResult
        | StageId::PreparedTrellis
        | StageId::PreparedTrellisSvelte
        | StageId::PreparedTrellisTest
        | StageId::PreparedUiTools => NPM_BUILD_DEPENDENCY,
        StageId::NpmPackagingSmoke => PREPARED_JAVASCRIPT_DEPENDENCIES,
        StageId::PackagePublicationDryRuns => NPM_PACKAGING_SMOKE_DEPENDENCY,
        StageId::LiveArtifactValidation => LIVE_BUILD_DEPENDENCY,
        StageId::SharedLiveIntegration => &[StageId::LiveArtifactValidation],
        StageId::DenoFormatting
        | StageId::RustWorkspaceFormatting
        | StageId::GeneratorFormatting
        | StageId::RustXtaskFormatting
        | StageId::WorkspaceClippy
        | StageId::GeneratedRustFacade
        | StageId::ProtocolWasm
        | StageId::Actionlint
        | StageId::TypeScriptCompile
        | StageId::NpmPackageBuild
        | StageId::WorkspaceTestCompileCoverage
        | StageId::CuratedPureRustTests
        | StageId::Rustdoc
        | StageId::Doctests
        | StageId::RustPackaging
        | StageId::GeneratorTests
        | StageId::RustXtaskTests
        | StageId::RootXtaskTests
        | StageId::LiveBuild => PREPARE_DEPENDENCY,
    }
}

pub(super) fn release_lane_for_stage(stage: StageId) -> Option<ReleaseLane> {
    match stage {
        StageId::DenoFormatting
        | StageId::RustWorkspaceFormatting
        | StageId::GeneratorFormatting
        | StageId::RustXtaskFormatting
        | StageId::Actionlint
        | StageId::TypeScriptCompile => Some(ReleaseLane::Static),
        StageId::WorkspaceClippy
        | StageId::GeneratedRustFacade
        | StageId::ProtocolWasm
        | StageId::WorkspaceTestCompileCoverage
        | StageId::CuratedPureRustTests
        | StageId::Rustdoc
        | StageId::Doctests
        | StageId::RustPackaging
        | StageId::GeneratorTests
        | StageId::RustXtaskTests
        | StageId::RootXtaskTests => Some(ReleaseLane::Rust),
        StageId::PreparedResult
        | StageId::PreparedTrellis
        | StageId::PreparedTrellisSvelte
        | StageId::PreparedTrellisTest
        | StageId::PreparedUiTools
        | StageId::NpmPackageBuild
        | StageId::NpmPackagingSmoke
        | StageId::PackagePublicationDryRuns => Some(ReleaseLane::TypeScript),
        StageId::LiveBuild => Some(ReleaseLane::LiveBuild),
        StageId::LiveArtifactValidation | StageId::SharedLiveIntegration => Some(ReleaseLane::Live),
        StageId::ReleaseMetadata | StageId::Prepare => None,
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

pub(super) fn verify_command_specs(
    version: &str,
    since: &str,
    skip_integration: bool,
    keep_workdir: bool,
) -> Vec<ReleaseStage> {
    command_specs(Some((version, since)), skip_integration, keep_workdir)
}

pub(super) fn lane_command_specs(lane: ReleaseLane, keep_workdir: bool) -> Vec<ReleaseStage> {
    release_plan(keep_workdir)
        .into_iter()
        .filter(|stage| stage.lane == Some(lane))
        .collect()
}

pub(super) fn release_plan(keep_workdir: bool) -> Vec<ReleaseStage> {
    command_specs(Some(("0.0.0", "v0.0.0")), false, keep_workdir)
}

pub(super) fn validate_stage_order(stages: &[ReleaseStage]) -> Result<(), String> {
    validate_stage_order_with_completed(stages, &BTreeSet::new())
}

pub(super) fn validate_selected_stage_order(
    plan: &[ReleaseStage],
    lane: ReleaseLane,
    selected: &[ReleaseStage],
) -> Result<(), String> {
    validate_stage_order(plan)?;
    if selected.is_empty() {
        return Err(format!(
            "release lane {} has no selected stages",
            release_lane_name(lane)
        ));
    }

    let selected_ids = selected
        .iter()
        .map(|stage| stage.id)
        .collect::<BTreeSet<_>>();
    let mut external_dependencies = BTreeSet::new();
    for stage in selected {
        if stage.lane != Some(lane) {
            return Err(format!(
                "release lane {} selected stage {:?} from another lane",
                release_lane_name(lane),
                stage.id
            ));
        }
        let planned = plan
            .iter()
            .find(|planned| planned.id == stage.id)
            .ok_or_else(|| {
                format!(
                    "selected release stage {:?} is missing from the plan",
                    stage.id
                )
            })?;
        if planned.lane != Some(lane) {
            return Err(format!(
                "release lane {} selected stage {:?} from another lane",
                release_lane_name(lane),
                stage.id
            ));
        }
        if planned != stage {
            return Err(format!(
                "selected release stage {:?} does not match the plan",
                stage.id
            ));
        }
        for dependency in stage.dependencies {
            if selected_ids.contains(dependency) {
                continue;
            }
            let planned_dependency = plan
                .iter()
                .find(|planned| planned.id == *dependency)
                .ok_or_else(|| {
                    format!(
                        "release stage {:?} depends on missing stage {:?}",
                        stage.id, dependency
                    )
                })?;
            if planned_dependency.lane == Some(lane) {
                return Err(format!(
                    "release lane {} omits dependency {:?} of stage {:?}",
                    release_lane_name(lane),
                    dependency,
                    stage.id
                ));
            }
            external_dependencies.insert(*dependency);
        }
    }

    validate_stage_order_with_completed(selected, &external_dependencies)
}

fn validate_stage_order_with_completed(
    stages: &[ReleaseStage],
    completed_stages: &BTreeSet<StageId>,
) -> Result<(), String> {
    let mut stage_ids = BTreeSet::new();
    for stage in stages {
        if !stage_ids.insert(stage.id) {
            return Err(format!("duplicate release stage identity {:?}", stage.id));
        }
    }

    let mut completed = completed_stages.clone();
    for stage in stages {
        for dependency in stage.dependencies {
            if !stage_ids.contains(dependency) && !completed.contains(dependency) {
                return Err(format!(
                    "release stage {:?} depends on missing stage {:?}",
                    stage.id, dependency
                ));
            }
            if !completed.contains(dependency) {
                return Err(format!(
                    "release stage {:?} runs before dependency {:?}",
                    stage.id, dependency
                ));
            }
        }
        completed.insert(stage.id);
    }
    Ok(())
}

pub(super) fn release_lane_waves(
    stages: &[ReleaseStage],
    completed_stages: &[StageId],
) -> Result<Vec<Vec<ReleaseLane>>, String> {
    validate_stage_order(stages)?;

    let stage_ids = stages.iter().map(|stage| stage.id).collect::<BTreeSet<_>>();
    let mut completed = BTreeSet::new();
    for stage in completed_stages {
        if !stage_ids.contains(stage) {
            return Err(format!("completed release stage {:?} is missing", stage));
        }
        completed.insert(*stage);
    }

    let mut lanes = Vec::new();
    for stage in stages {
        let Some(lane) = stage.lane else {
            continue;
        };
        if !lanes.contains(&lane) {
            lanes.push(lane);
        }
    }

    let mut waves = Vec::new();
    while completed.len() < stages.len() {
        let mut ready = Vec::new();
        for lane in &lanes {
            let lane_stages = stages
                .iter()
                .filter(|stage| stage.lane == Some(*lane))
                .collect::<Vec<_>>();
            if !lane_stages
                .iter()
                .any(|stage| !completed.contains(&stage.id))
            {
                continue;
            }

            let mut earlier_in_lane = BTreeSet::new();
            let can_run = lane_stages.iter().all(|stage| {
                let can_run = completed.contains(&stage.id)
                    || stage.dependencies.iter().all(|dependency| {
                        completed.contains(dependency) || earlier_in_lane.contains(dependency)
                    });
                earlier_in_lane.insert(stage.id);
                can_run
            });
            if can_run {
                ready.push(*lane);
            }
        }

        if ready.is_empty() {
            return Err("release stage dependencies cannot produce another execution wave".into());
        }

        for lane in &ready {
            for stage in stages.iter().filter(|stage| stage.lane == Some(*lane)) {
                completed.insert(stage.id);
            }
        }
        waves.push(ready);
    }
    Ok(waves)
}

fn command_specs(
    metadata: Option<(&str, &str)>,
    skip_integration: bool,
    keep_workdir: bool,
) -> Vec<ReleaseStage> {
    let mut specs = Vec::new();
    if let Some((version, since)) = metadata {
        specs.extend([
            CommandSpec::new(
                "cargo",
                vec![
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
            )
            .stage(StageId::ReleaseMetadata),
            CommandSpec::new(
                "cargo",
                vec![
                    "run",
                    "--manifest-path",
                    "xtask/Cargo.toml",
                    "--",
                    "prepare",
                ],
            )
            .stage(StageId::Prepare),
        ]);
    }
    specs.extend([
        CommandSpec::new("deno", vec!["fmt", "-c", "ts/deno.json", "--check"])
            .stage(StageId::DenoFormatting),
        CommandSpec::new(
            "cargo",
            vec![
                "fmt",
                "--manifest-path",
                "rust/Cargo.toml",
                "--all",
                "--check",
            ],
        )
        .stage(StageId::RustWorkspaceFormatting),
        CommandSpec::new(
            "cargo",
            vec![
                "fmt",
                "--manifest-path",
                "rust/tools/generate/Cargo.toml",
                "--check",
            ],
        )
        .stage(StageId::GeneratorFormatting),
        CommandSpec::new(
            "cargo",
            vec!["fmt", "--manifest-path", "rust/xtask/Cargo.toml", "--check"],
        )
        .stage(StageId::RustXtaskFormatting),
        CommandSpec::new(
            "cargo",
            vec![
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
        )
        .stage(StageId::WorkspaceClippy),
        CommandSpec::new(
            "cargo",
            vec![
                "check",
                "--manifest-path",
                "generated/packages/cargo-participants/jobs/Cargo.toml",
            ],
        )
        .stage(StageId::GeneratedRustFacade),
        CommandSpec::new(
            "cargo",
            vec![
                "check",
                "--manifest-path",
                "rust/Cargo.toml",
                "--package",
                "trellis-protocol-wasm",
                "--target",
                "wasm32-unknown-unknown",
            ],
        )
        .stage(StageId::ProtocolWasm),
        CommandSpec::new("actionlint", Vec::<String>::new()).stage(StageId::Actionlint),
        CommandSpec::new(
            "deno",
            vec![
                "check",
                "-c",
                "ts/deno.json",
                "ts/packages/trellis/index.ts",
                "ts/packages/trellis-svelte/src/index.ts",
                "ts/packages/trellis-svelte/src/context.svelte.ts",
            ],
        )
        .stage(StageId::TypeScriptCompile),
        CommandSpec::new(
            "deno",
            vec!["task", "-c", "ts/deno.json", "packages:build:npm"],
        )
        .stage(StageId::NpmPackageBuild),
        CommandSpec::new(
            "deno",
            vec!["task", "-c", "ts/deno.json", "test:prepared:result"],
        )
        .stage(StageId::PreparedResult)
        .parallel(ParallelGroup::PreparedJavaScript),
        CommandSpec::new(
            "deno",
            vec!["task", "-c", "ts/deno.json", "test:prepared:trellis"],
        )
        .stage(StageId::PreparedTrellis)
        .parallel(ParallelGroup::PreparedJavaScript),
        CommandSpec::new(
            "deno",
            vec!["task", "-c", "ts/deno.json", "test:prepared:trellis-svelte"],
        )
        .stage(StageId::PreparedTrellisSvelte)
        .parallel(ParallelGroup::PreparedJavaScript),
        CommandSpec::new(
            "deno",
            vec!["task", "-c", "ts/deno.json", "test:prepared:trellis-test"],
        )
        .stage(StageId::PreparedTrellisTest)
        .parallel(ParallelGroup::PreparedJavaScript),
        CommandSpec::new(
            "deno",
            vec!["task", "-c", "ts/deno.json", "test:prepared:ui-tools"],
        )
        .stage(StageId::PreparedUiTools)
        .parallel(ParallelGroup::PreparedJavaScript),
        CommandSpec::new(
            "deno",
            vec![
                "task",
                "-c",
                "ts/deno.json",
                "test:prepared:packaging:built",
            ],
        )
        .stage(StageId::NpmPackagingSmoke),
        CommandSpec::new("bash", vec!["scripts/release-ts-dry-run.sh"])
            .stage(StageId::PackagePublicationDryRuns),
        CommandSpec::new(
            "cargo",
            vec![
                "test",
                "--manifest-path",
                "rust/Cargo.toml",
                "--workspace",
                "--no-run",
            ],
        )
        .stage(StageId::WorkspaceTestCompileCoverage),
        CommandSpec::new(
            "cargo",
            vec![
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
        )
        .stage(StageId::CuratedPureRustTests),
        CommandSpec::new(
            "env",
            vec![
                "RUSTDOCFLAGS=-D warnings",
                "cargo",
                "doc",
                "--manifest-path",
                "rust/Cargo.toml",
                "--workspace",
                "--no-deps",
            ],
        )
        .stage(StageId::Rustdoc),
        CommandSpec::new(
            "cargo",
            vec![
                "test",
                "--manifest-path",
                "rust/Cargo.toml",
                "--workspace",
                "--doc",
            ],
        )
        .stage(StageId::Doctests),
        CommandSpec::new(
            "cargo",
            vec![
                "package",
                "--manifest-path",
                "rust/Cargo.toml",
                "--package",
                "trellis-protocol",
                "--allow-dirty",
            ],
        )
        .stage(StageId::RustPackaging),
        CommandSpec::new(
            "cargo",
            vec!["test", "--manifest-path", "rust/tools/generate/Cargo.toml"],
        )
        .stage(StageId::GeneratorTests),
        CommandSpec::new(
            "cargo",
            vec!["test", "--manifest-path", "rust/xtask/Cargo.toml"],
        )
        .stage(StageId::RustXtaskTests),
        CommandSpec::new("cargo", vec!["test", "--manifest-path", "xtask/Cargo.toml"])
            .stage(StageId::RootXtaskTests),
    ]);

    if !skip_integration {
        specs.push(
            CommandSpec::new(
                "deno",
                vec![
                    "run",
                    "-A",
                    "-c",
                    "ts/deno.json",
                    "integration/live_runner.ts",
                    "--build-only",
                    "--artifacts-manifest",
                    INTEGRATION_LIVE_ARTIFACTS_MANIFEST,
                ],
            )
            .stage(StageId::LiveBuild),
        );
        specs.push(
            CommandSpec::new(
                "deno",
                vec![
                    "run",
                    "-A",
                    "-c",
                    "ts/deno.json",
                    "integration/live_runner.ts",
                    "--inventory-only",
                    "--prebuilt-only",
                    "--artifacts-manifest",
                    INTEGRATION_LIVE_ARTIFACTS_MANIFEST,
                ],
            )
            .stage(StageId::LiveArtifactValidation),
        );
        let mut live_args = vec![
            "run".to_string(),
            "-A".to_string(),
            "-c".to_string(),
            "ts/deno.json".to_string(),
            "integration/live_runner.ts".to_string(),
            "--prebuilt-only".to_string(),
            "--artifacts-manifest".to_string(),
            INTEGRATION_LIVE_ARTIFACTS_MANIFEST.to_string(),
            "--jobs".to_string(),
            "20".to_string(),
        ];
        if keep_workdir {
            live_args.insert(0, "deno".to_string());
            live_args.insert(0, "TRELLIS_TEST_KEEP_WORKDIR=1".to_string());
            specs.push(CommandSpec::new("env", live_args).stage(StageId::SharedLiveIntegration));
        } else {
            specs.push(CommandSpec::new("deno", live_args).stage(StageId::SharedLiveIntegration));
        }
    }

    specs
}
