//! Native Trellis IDL generation and filesystem watch orchestration.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use miette::{miette, IntoDiagnostic, Result, WrapErr};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use trellis_codegen_rust::{
    GenerateRustParticipantFacadeOpts, GenerateRustSdkOpts, ParticipantAliasMapping,
    RustRuntimeDeps, RustRuntimeSource,
};
use trellis_codegen_ts::{GenerateTsSdkOpts, TsRuntimeDeps, TsRuntimeSource};
use trellis_idl::project::read_manifest;

use crate::cli::GenerateArgs;

const TRELLIS_DENO_JSON: &str = include_str!("../../../../ts/packages/trellis/deno.json");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeSource {
    Local,
    Registry,
}

fn detect_output_root(project_root: &Path) -> PathBuf {
    project_root
        .ancestors()
        .find(|directory| directory.join(".git").exists())
        .unwrap_or(project_root)
        .to_path_buf()
}

fn detect_runtime_source(output_root: &Path) -> RuntimeSource {
    if output_root.join("rust/Cargo.toml").exists()
        && output_root.join("ts/packages/trellis").exists()
    {
        RuntimeSource::Local
    } else {
        RuntimeSource::Registry
    }
}

fn sdk_output_stem(id: &str) -> String {
    if id == "trellis.core@v1" {
        "trellis-core".to_owned()
    } else {
        trellis_codegen_rust::default_sdk_stem(id)
    }
}

fn default_rust_crate_name_from_id(id: &str) -> String {
    trellis_codegen_rust::default_sdk_crate_name(id)
}

pub(crate) fn validate_output_identity(kind: &str, id: &str) -> Result<()> {
    miette::ensure!(
        !id.contains(['/', '\\']) && !id.contains("..") && !id.chars().any(char::is_whitespace),
        "{kind} id {id:?} cannot be used as a generated output name"
    );
    Ok(())
}

pub(crate) fn trellis_package_version() -> String {
    serde_json::from_str::<serde_json::Value>(TRELLIS_DENO_JSON)
        .expect("bundled Trellis Deno package manifest must be valid JSON")["version"]
        .as_str()
        .expect("bundled Trellis Deno package manifest must have a version")
        .to_owned()
}

fn rust_runtime_deps(
    source: RuntimeSource,
    version: String,
    repo_root: Option<PathBuf>,
) -> RustRuntimeDeps {
    RustRuntimeDeps {
        source: match source {
            RuntimeSource::Local => RustRuntimeSource::Local,
            RuntimeSource::Registry => RustRuntimeSource::Registry,
        },
        version,
        repo_root,
    }
}

fn ts_runtime_deps(
    source: RuntimeSource,
    version: String,
    repo_root: Option<PathBuf>,
) -> TsRuntimeDeps {
    TsRuntimeDeps {
        source: match source {
            RuntimeSource::Local => TsRuntimeSource::Local,
            RuntimeSource::Registry => TsRuntimeSource::Registry,
        },
        version,
        repo_root,
    }
}

fn format_generated_typescript_artifacts(path: &Path, repo_root: Option<&Path>) -> Result<()> {
    let Some(config) = repo_root
        .map(|root| root.join("ts/deno.json"))
        .filter(|path| path.is_file())
    else {
        return Ok(());
    };
    let output = std::process::Command::new("deno")
        .args(["fmt", "-c"])
        .arg(config)
        .arg(path)
        .output()
        .into_diagnostic()?;
    miette::ensure!(
        output.status.success(),
        "deno fmt failed for {}\nstdout:\n{}\nstderr:\n{}",
        path.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[derive(Debug)]
pub(crate) struct GenerationResult {
    pub(crate) generated: usize,
    pub(crate) owned_api_paths: Vec<PathBuf>,
}

/// Run native IDL generation once or in watch mode.
pub fn run(args: &GenerateArgs) -> Result<()> {
    let root = args.project.root.canonicalize().into_diagnostic()?;
    if args.watch {
        watch(&root)
    } else {
        generate_once(&root).map(|_| ())
    }
}

/// Compiles and generates one native Trellis project without network access.
pub(crate) fn generate_once(root: &Path) -> Result<GenerationResult> {
    let root = root.canonicalize().into_diagnostic()?;
    let compiled = trellis_idl::compile_project(&root)?;
    let trellis_root = root.join(".trellis");
    let has_ts = root.join("deno.json").is_file() || root.join("deno.jsonc").is_file();
    let has_rust = root.join("Cargo.toml").is_file();
    let output_root = detect_output_root(&root);
    let runtime_source = detect_runtime_source(&output_root);
    let runtime_repo_root = matches!(runtime_source, RuntimeSource::Local).then_some(output_root);
    let runtime_version = trellis_package_version();
    let owner_version = (has_rust && !compiled.participants.is_empty())
        .then(|| rust_project_version(&root))
        .transpose()?;
    let apis = compiled
        .apis
        .iter()
        .chain(&compiled.referenced_apis)
        .map(|(id, api)| (id.as_str(), api))
        .collect::<BTreeMap<_, _>>();
    let mut sdk_stems = BTreeMap::new();
    for id in apis.keys() {
        validate_output_identity("API", id)?;
        let stem = sdk_output_stem(id);
        if let Some(existing) = sdk_stems
            .insert(stem.clone(), id)
            .filter(|_| has_ts || has_rust)
        {
            return Err(miette!(
                "API SDK path '{stem}' collides between '{existing}' and '{id}'"
            ));
        }
    }
    let mut participant_stems = BTreeMap::new();
    for participant in &compiled.participants {
        validate_output_identity("participant", participant.id())?;
        let stem = sdk_output_stem(participant.id());
        if let Some(existing) = participant_stems
            .insert(stem.clone(), participant.id())
            .filter(|_| has_ts || has_rust)
        {
            let language = if has_ts { "ts" } else { "rust" };
            return Err(miette!(
                "participant SDK path '{}' collides between '{}' and '{}'",
                trellis_root
                    .join(language)
                    .join("participants")
                    .join(stem)
                    .display(),
                existing,
                participant.id()
            ));
        }
    }

    // Sibling staging preserves relative Cargo/Deno paths after publication.
    let staging = tempfile::Builder::new()
        .prefix(".trellis-stage-")
        .tempdir_in(&root)
        .into_diagnostic()?;
    let api_root = staging.path().join("artifacts/apis");
    let participant_root = staging.path().join("artifacts/participants");
    let rust_api_root = staging.path().join("rust/apis");
    let rust_participant_root = staging.path().join("rust/participants");
    let ts_api_root = staging.path().join("ts/apis");
    let ts_participant_root = staging.path().join("ts/participants");

    for (id, api) in &compiled.apis {
        eprintln!("{id} {}", api.digest().into_diagnostic()?);
        let api_path = api_root.join(format!("{id}.json"));
        write_if_changed(
            &api_path,
            format!("{}\n", api.canonical_json().map_err(protocol_error)?).as_bytes(),
        )?;
    }

    for (id, api) in &apis {
        let stem = sdk_output_stem(id);
        if has_ts {
            let out = ts_api_root.join(&stem);
            trellis_codegen_ts::generate_ts_sdk(
                api,
                &GenerateTsSdkOpts {
                    out_dir: out.clone(),
                    package_name: format!("@trellis-sdk/{}", id.split('@').next().unwrap_or(id)),
                    package_version: api.version().to_owned(),
                    runtime_deps: ts_runtime_deps(
                        runtime_source,
                        runtime_version.clone(),
                        runtime_repo_root.clone(),
                    ),
                },
            )
            .into_diagnostic()?;
            format_generated_typescript_artifacts(&out, runtime_repo_root.as_deref())?;
        }
        if has_rust {
            let out = rust_api_root.join(&stem);
            trellis_codegen_rust::generate_rust_sdk(
                api,
                &GenerateRustSdkOpts {
                    out_dir: out.clone(),
                    crate_name: default_rust_crate_name_from_id(id),
                    crate_version: api.version().to_owned(),
                    runtime_deps: rust_runtime_deps(
                        runtime_source,
                        runtime_version.clone(),
                        runtime_repo_root.clone(),
                    ),
                },
            )
            .into_diagnostic()?;
        }
    }
    for participant in &compiled.participants {
        let participant_path = participant_root.join(format!("{}.json", participant.id()));
        write_if_changed(
            &participant_path,
            format!(
                "{}\n",
                participant.canonical_json().map_err(protocol_error)?
            )
            .as_bytes(),
        )?;
        let value = participant.normalized_value().map_err(protocol_error)?;
        let implemented = value["implements"]
            .as_object()
            .and_then(|implements| implements.values().next())
            .and_then(|implementation| implementation["api"].as_str())
            .ok_or_else(|| miette!("participant '{}' has no implemented API", participant.id()))?;
        if has_ts {
            let out = ts_participant_root.join(sdk_output_stem(participant.id()));
            trellis_codegen_ts::generate_ts_participant(
                participant,
                apis[implemented],
                &apis,
                &out,
            )
            .into_diagnostic()?;
            format_generated_typescript_artifacts(&out, runtime_repo_root.as_deref())?;
        }
        if has_rust {
            let aliases = ["required", "optional"]
                .into_iter()
                .filter_map(|kind| value["uses"][kind].as_object())
                .flat_map(|uses| uses.iter())
                .map(|(alias, used)| {
                    let id = used["api"].as_str().expect("validated participant API use");
                    ParticipantAliasMapping {
                        alias: alias.clone(),
                        crate_name: default_rust_crate_name_from_id(id),
                        api: apis[id],
                        crate_path: rust_api_root.join(sdk_output_stem(id)),
                    }
                })
                .collect();
            let out = rust_participant_root.join(sdk_output_stem(participant.id()));
            trellis_codegen_rust::generate_rust_participant_facade(
                apis[implemented],
                participant,
                &GenerateRustParticipantFacadeOpts {
                    out_dir: out.clone(),
                    crate_name: format!(
                        "trellis-participant-{}",
                        sdk_output_stem(participant.id())
                    ),
                    crate_version: owner_version.clone().expect("Rust project version"),
                    runtime_deps: rust_runtime_deps(
                        runtime_source,
                        runtime_version.clone(),
                        runtime_repo_root.clone(),
                    ),
                    owned_sdk_crate_name: Some(default_rust_crate_name_from_id(implemented)),
                    owned_sdk_path: Some(rust_api_root.join(sdk_output_stem(implemented))),
                    alias_mappings: aliases,
                },
            )
            .into_diagnostic()?;
        }
    }

    fs::create_dir_all(&trellis_root).into_diagnostic()?;
    let backup = tempfile::Builder::new()
        .prefix(".trellis-backup-")
        .tempdir_in(&root)
        .into_diagnostic()?;
    let mut saved = Vec::new();
    let mut published = Vec::new();
    let publication = (|| -> std::io::Result<()> {
        for name in ["artifacts", "rust", "ts"] {
            let target = trellis_root.join(name);
            if target.exists() {
                fs::rename(&target, backup.path().join(name))?;
                saved.push(name);
            }
            if staging.path().join(name).exists() {
                fs::rename(staging.path().join(name), &target)?;
                published.push(name);
            }
        }
        Ok(())
    })();
    if let Err(error) = publication {
        let mut rollback_error = None;
        for name in published.into_iter().rev() {
            if let Err(error) = fs::rename(trellis_root.join(name), staging.path().join(name)) {
                rollback_error.get_or_insert(error);
            }
        }
        for name in saved.into_iter().rev() {
            if let Err(error) = fs::rename(backup.path().join(name), trellis_root.join(name)) {
                rollback_error.get_or_insert(error);
            }
        }
        if let Some(rollback_error) = rollback_error {
            let recovery = backup.keep();
            return Err(miette!("generation publication failed: {error}; rollback failed: {rollback_error}; previous outputs retained at {}", recovery.display()));
        }
        return Err(error).into_diagnostic();
    }
    Ok(GenerationResult {
        generated: compiled.apis.len()
            + compiled.referenced_apis.len()
            + compiled.participants.len(),
        owned_api_paths: compiled
            .apis
            .keys()
            .map(|id| {
                trellis_root
                    .join("artifacts/apis")
                    .join(format!("{id}.json"))
            })
            .collect(),
    })
}

/// Compiles and generates one native project without package installation or network access.
pub fn generate_project(root: &Path) -> Result<()> {
    generate_once(root).map(|_| ())
}

fn watch(root: &Path) -> Result<()> {
    let (sender, receiver) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(200), move |events| {
        let _ = sender.send(events);
    })
    .into_diagnostic()?;
    let mut watched = BTreeSet::new();
    debouncer
        .watcher()
        .watch(root, RecursiveMode::Recursive)
        .into_diagnostic()?;
    watched.insert(root.to_path_buf());
    refresh_watch_roots(&mut debouncer, root, &mut watched);
    if let Err(error) = generate_once(root) {
        eprintln!("{error:?}");
    }
    while let Ok(events) = receiver.recv() {
        match events {
            Ok(events) if events.iter().any(|event| relevant(&event.path)) => {
                refresh_watch_roots(&mut debouncer, root, &mut watched);
                if let Err(error) = generate_once(root) {
                    eprintln!("{error:?}");
                }
            }
            Ok(_) => {}
            Err(error) => eprintln!("{error}"),
        }
    }
    Ok(())
}

fn refresh_watch_roots(
    debouncer: &mut notify_debouncer_mini::Debouncer<
        notify_debouncer_mini::notify::RecommendedWatcher,
    >,
    root: &Path,
    watched: &mut BTreeSet<PathBuf>,
) {
    let result = read_manifest(&root.join("trellis.toml")).and_then(|manifest| {
        for path in manifest
            .apis
            .values()
            .filter_map(|dependency| dependency.path.as_deref())
        {
            let path = root.join(path).canonicalize().into_diagnostic()?;
            if watched.insert(path.clone()) {
                debouncer
                    .watcher()
                    .watch(&path, RecursiveMode::Recursive)
                    .into_diagnostic()?;
            }
        }
        Ok(())
    });
    if let Err(error) = result {
        eprintln!("{error:?}");
    }
}

fn relevant(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "trellis")
        || path
            .file_name()
            .is_some_and(|name| name == "trellis.toml" || name == "trellis.lock")
}

fn rust_project_version(root: &Path) -> Result<String> {
    for directory in root.ancestors() {
        let manifest_path = directory.join("Cargo.toml");
        let Ok(contents) = fs::read_to_string(&manifest_path) else {
            continue;
        };
        let manifest: toml::Value = toml::from_str(&contents)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to parse {}", manifest_path.display()))?;
        if directory == root {
            if let Some(version) = manifest
                .get("package")
                .and_then(|package| package.get("version"))
                .and_then(toml::Value::as_str)
            {
                return Ok(version.to_owned());
            }
        }
        if let Some(version) = manifest
            .get("workspace")
            .and_then(|workspace| workspace.get("package"))
            .and_then(|package| package.get("version"))
            .and_then(toml::Value::as_str)
        {
            return Ok(version.to_owned());
        }
    }
    Err(miette!(
        "Rust projects require a package or workspace package version"
    ))
}

fn write_if_changed(path: &Path, contents: &[u8]) -> Result<()> {
    if fs::read(path).ok().as_deref() == Some(contents) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }
    fs::write(path, contents).into_diagnostic()
}

fn protocol_error(error: trellis_protocol::ProtocolError) -> miette::Report {
    miette!(error.to_string())
}
