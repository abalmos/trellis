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
};
use trellis_codegen_ts::GenerateTsSdkOpts;
use trellis_idl::project::read_manifest;

use crate::cli::GenerateArgs;

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

pub(crate) fn generate_once(root: &Path) -> Result<GenerationResult> {
    let root = root.canonicalize().into_diagnostic()?;
    let compiled = trellis_idl::compile_project(&root)?;
    let trellis_root = root.join(".trellis");
    let api_root = trellis_root.join("artifacts/apis");
    let participant_root = trellis_root.join("artifacts/participants");
    let rust_api_root = trellis_root.join("rust/apis");
    let rust_participant_root = trellis_root.join("rust/participants");
    let ts_api_root = trellis_root.join("ts/apis");
    let has_ts = root.join("deno.json").is_file() || root.join("deno.jsonc").is_file();
    let has_rust = root.join("Cargo.toml").is_file();
    let output_root = trellis_generation::artifacts::detect_output_root(&root);
    let runtime_source = trellis_generation::artifacts::detect_runtime_source(&output_root);
    let runtime_repo_root = matches!(
        runtime_source,
        trellis_generation::model::RuntimeSource::Local
    )
    .then_some(output_root);
    let runtime_version = trellis_generation::artifacts::trellis_package_version();
    let mut api_paths = BTreeSet::new();
    let mut ts_api_paths = BTreeSet::new();
    let mut rust_api_paths = BTreeSet::new();
    let mut sdk_stems = BTreeMap::new();
    for id in compiled.apis.keys().chain(compiled.referenced_apis.keys()) {
        let stem = trellis_generation::artifacts::sdk_output_stem(id);
        if let Some(existing) = sdk_stems.insert(stem.clone(), id) {
            return Err(miette!(
                "API SDK path '{stem}' collides between '{existing}' and '{id}'"
            ));
        }
    }

    for (id, api) in &compiled.apis {
        trellis_generation::planning::validate_output_identity("API", id)?;
        let api_path = api_root.join(format!("{id}.json"));
        write_if_changed(
            &api_path,
            format!("{}\n", api.canonical_json().map_err(protocol_error)?).as_bytes(),
        )?;
        api_paths.insert(api_path.clone());
    }

    let referenced = tempfile::tempdir().into_diagnostic()?;
    for (id, api) in &compiled.referenced_apis {
        trellis_generation::planning::validate_output_identity("API", id)?;
        write_if_changed(
            &referenced.path().join(format!("{id}.json")),
            api.canonical_json().map_err(protocol_error)?.as_bytes(),
        )?;
    }

    for (id, api) in compiled.apis.iter().chain(&compiled.referenced_apis) {
        let api_path = if compiled.apis.contains_key(id) {
            api_root.join(format!("{id}.json"))
        } else {
            referenced.path().join(format!("{id}.json"))
        };
        let stem = trellis_generation::artifacts::sdk_output_stem(id);
        if has_ts {
            let out = ts_api_root.join(&stem);
            trellis_codegen_ts::generate_ts_sdk(&GenerateTsSdkOpts {
                api_path: api_path.clone(),
                out_dir: out.clone(),
                package_name: format!("@trellis-sdk/{}", id.split('@').next().unwrap_or(id)),
                package_version: api.version().to_owned(),
                runtime_deps: trellis_generation::artifacts::ts_runtime_deps(
                    runtime_source,
                    runtime_version.clone(),
                    runtime_repo_root.clone(),
                ),
            })
            .into_diagnostic()?;
            trellis_generation::artifacts::format_generated_typescript_artifacts(
                &out,
                runtime_repo_root.as_deref(),
            )?;
            ts_api_paths.insert(out);
        }
        if has_rust {
            let out = rust_api_root.join(&stem);
            trellis_codegen_rust::generate_rust_sdk(&GenerateRustSdkOpts {
                api_path,
                out_dir: out.clone(),
                crate_name: trellis_generation::artifacts::default_rust_crate_name_from_id(id),
                crate_version: api.version().to_owned(),
                runtime_deps: trellis_generation::artifacts::rust_runtime_deps(
                    runtime_source,
                    runtime_version.clone(),
                    runtime_repo_root.clone(),
                ),
            })
            .into_diagnostic()?;
            rust_api_paths.insert(out);
        }
    }
    let owner_version = (has_rust && !compiled.participants.is_empty())
        .then(|| rust_project_version(&root))
        .transpose()?;
    let mut participant_paths = BTreeSet::new();
    let mut facade_paths = BTreeSet::new();
    for participant in &compiled.participants {
        trellis_generation::planning::validate_output_identity("participant", participant.id())?;
        let participant_path = participant_root.join(format!("{}.json", participant.id()));
        write_if_changed(
            &participant_path,
            format!(
                "{}\n",
                participant.canonical_json().map_err(protocol_error)?
            )
            .as_bytes(),
        )?;
        participant_paths.insert(participant_path.clone());
        if has_rust {
            let value = participant.normalized_value().map_err(protocol_error)?;
            let implemented = value["implements"]
                .as_object()
                .and_then(|implements| implements.values().next())
                .and_then(|implementation| implementation["api"].as_str())
                .ok_or_else(|| {
                    miette!("participant '{}' has no implemented API", participant.id())
                })?;
            let aliases = ["required", "optional"]
                .into_iter()
                .filter_map(|kind| value["uses"][kind].as_object())
                .flat_map(|uses| uses.iter())
                .map(|(alias, used)| {
                    let id = used["api"].as_str().expect("validated participant API use");
                    ParticipantAliasMapping {
                        alias: alias.clone(),
                        crate_name: trellis_generation::artifacts::default_rust_crate_name_from_id(
                            id,
                        ),
                        api_path: if compiled.apis.contains_key(id) {
                            api_root.join(format!("{id}.json"))
                        } else {
                            referenced.path().join(format!("{id}.json"))
                        },
                        crate_path: Some(
                            rust_api_root.join(trellis_generation::artifacts::sdk_output_stem(id)),
                        ),
                        cargo_dependency: None,
                    }
                })
                .collect();
            let out = rust_participant_root.join(trellis_generation::artifacts::sdk_output_stem(
                participant.id(),
            ));
            trellis_codegen_rust::generate_rust_participant_facade(
                &GenerateRustParticipantFacadeOpts {
                    api_path: if compiled.apis.contains_key(implemented) {
                        api_root.join(format!("{implemented}.json"))
                    } else {
                        referenced.path().join(format!("{implemented}.json"))
                    },
                    participant_path,
                    out_dir: out.clone(),
                    crate_name: format!(
                        "trellis-participant-{}",
                        trellis_generation::artifacts::sdk_output_stem(participant.id())
                    ),
                    crate_version: owner_version.clone().expect("Rust project version"),
                    runtime_deps: trellis_generation::artifacts::rust_runtime_deps(
                        runtime_source,
                        runtime_version.clone(),
                        runtime_repo_root.clone(),
                    ),
                    owned_sdk_crate_name: Some(
                        trellis_generation::artifacts::default_rust_crate_name_from_id(implemented),
                    ),
                    owned_sdk_path: Some(
                        rust_api_root
                            .join(trellis_generation::artifacts::sdk_output_stem(implemented)),
                    ),
                    alias_mappings: aliases,
                },
            )
            .into_diagnostic()?;
            facade_paths.insert(out);
        }
    }

    for (directory, current) in [
        (&api_root, &api_paths),
        (&participant_root, &participant_paths),
        (&ts_api_root, &ts_api_paths),
        (&rust_api_root, &rust_api_paths),
        (&rust_participant_root, &facade_paths),
    ] {
        prune(directory, current)?;
    }
    for directory in [
        trellis_root.join("artifacts"),
        trellis_root.join("rust"),
        trellis_root.join("ts"),
    ] {
        if directory.is_dir() && fs::read_dir(&directory).into_diagnostic()?.next().is_none() {
            fs::remove_dir(directory).into_diagnostic()?;
        }
    }
    let old_generated = trellis_root.join("generated");
    if old_generated.is_dir() {
        fs::remove_dir_all(old_generated).into_diagnostic()?;
    } else if old_generated.exists() {
        fs::remove_file(old_generated).into_diagnostic()?;
    }
    Ok(GenerationResult {
        generated: compiled.apis.len()
            + compiled.referenced_apis.len()
            + compiled.participants.len(),
        owned_api_paths: api_paths.into_iter().collect(),
    })
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

fn prune(directory: &Path, current: &BTreeSet<PathBuf>) -> Result<()> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Ok(());
    };
    for entry in entries {
        let path = entry.into_diagnostic()?.path();
        if !current.contains(&path) {
            if path.is_dir() {
                fs::remove_dir_all(path).into_diagnostic()?;
            } else {
                fs::remove_file(path).into_diagnostic()?;
            }
        }
    }
    if current.is_empty() {
        fs::remove_dir(directory).into_diagnostic()?;
    }
    Ok(())
}

fn protocol_error(error: trellis_protocol::ProtocolError) -> miette::Report {
    miette!(error.to_string())
}
