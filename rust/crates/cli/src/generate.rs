//! Native Trellis package generation and offline filesystem watch orchestration.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use miette::{miette, IntoDiagnostic, Result};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use trellis_idl::project::{read_manifest, ProjectManifest};

use crate::cli::GenerateArgs;

pub(crate) fn validate_output_identity(kind: &str, id: &str) -> Result<()> {
    miette::ensure!(
        !id.contains(['/', '\\']) && !id.contains("..") && !id.chars().any(char::is_whitespace),
        "{kind} id {id:?} cannot be used as a generated output name"
    );
    Ok(())
}

/// Run native IDL generation once or in watch mode.
pub fn run(args: &GenerateArgs) -> Result<()> {
    let root = args.project.root.canonicalize().into_diagnostic()?;
    if args.watch {
        watch(&root)
    } else {
        generate_project(&root)
    }
}

/// Compile current local sources and exact cached dependencies without network access.
pub fn generate_project(root: &Path) -> Result<()> {
    generate_once(root).map(|_| ())
}

pub(crate) fn generate_once(root: &Path) -> Result<usize> {
    let root = root.canonicalize().into_diagnostic()?;
    let manifest = read_manifest(&root.join("trellis.toml"))?;
    let compiled = crate::package::compile_project(&root, &manifest)?;
    generate_compiled(&root, &manifest, &compiled)
}

pub(crate) fn generate_compiled(
    root: &Path,
    manifest: &ProjectManifest,
    compiled: &trellis_idl::CompiledProject,
) -> Result<usize> {
    let has_rust = root.join("Cargo.toml").is_file();
    let has_ts = ["package.json", "deno.json", "deno.jsonc"]
        .iter()
        .any(|file| root.join(file).is_file());
    let config = manifest.generate.clone().unwrap_or_default();
    let languages = usize::from(has_rust) + usize::from(has_ts);
    miette::ensure!(
        config.output.is_none() || languages == 1,
        "[generate].output requires exactly one detected language"
    );
    if languages == 0 {
        return Ok(0);
    }
    let name = manifest
        .name
        .as_deref()
        .ok_or_else(|| miette!("trellis.toml requires name when generating a language package"))?;
    if has_rust {
        miette::ensure!(
            name.starts_with(|character: char| character.is_ascii_alphabetic() || character == '_')
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric()
                        || character == '_'
                        || character == '-'),
            "invalid Cargo package name '{name}'"
        );
    }
    if has_ts {
        let parts = name
            .strip_prefix('@')
            .map_or_else(|| vec![name], |scoped| scoped.split('/').collect());
        miette::ensure!(
            name.len() <= 214
                && parts.len() == if name.starts_with('@') { 2 } else { 1 }
                && parts.iter().all(|part| !part.is_empty()
                    && !part.starts_with(['.', '_'])
                    && part.chars().all(|character| character.is_ascii_lowercase()
                        || character.is_ascii_digit()
                        || matches!(character, '.' | '_' | '-'))),
            "invalid npm package name '{name}'"
        );
    }
    let mut outputs = Vec::new();
    for (language, detected, selected) in [
        ("rust", has_rust, config.rust.as_ref()),
        ("typescript", has_ts, config.typescript.as_ref()),
    ] {
        if !detected {
            continue;
        }
        miette::ensure!(languages != 2 || selected.is_some(), "multiple languages detected; configure [generate.rust].output and [generate.typescript].output");
        miette::ensure!(
            config.output.is_none() || selected.is_none(),
            "configure only one output for {language}"
        );
        let destination = selected
            .map(|selected| selected.output.as_str())
            .or(config.output.as_deref())
            .unwrap_or("trellis");
        miette::ensure!(
            !destination.is_empty(),
            "generated output must not be empty"
        );
        let destination = root.join(destination);
        // Resolve existing ancestors without following symlinks into user data.
        let mut resolved = PathBuf::new();
        for component in destination.components() {
            match component {
                std::path::Component::CurDir => continue,
                std::path::Component::ParentDir => {
                    resolved.pop();
                }
                _ => resolved.push(component.as_os_str()),
            }
            if let Ok(metadata) = fs::symlink_metadata(&resolved) {
                miette::ensure!(
                    !metadata.file_type().is_symlink() && metadata.is_dir(),
                    "generated output ancestor {} must be a real directory",
                    resolved.display()
                );
            }
        }
        for source in std::iter::once(root.to_path_buf()).chain(
            manifest
                .apis
                .values()
                .filter_map(|dependency| dependency.path.as_ref())
                .map(|path| root.join(path)),
        ) {
            let source = source.canonicalize().into_diagnostic()?;
            miette::ensure!(
                !source.starts_with(&resolved),
                "generated output {} contains a source project",
                resolved.display()
            );
            for protected in ["contracts", ".git", ".trellis", "trellis_modules"] {
                miette::ensure!(
                    !resolved.starts_with(source.join(protected)),
                    "generated output {} overlaps source or reserved state",
                    resolved.display()
                );
            }
        }
        for (_, previous) in &outputs {
            miette::ensure!(
                !resolved.starts_with(previous) && !Path::new(previous).starts_with(&resolved),
                "generated package outputs overlap"
            );
        }
        outputs.push((language, resolved));
    }
    let apis = compiled
        .apis
        .iter()
        .chain(&compiled.referenced_apis)
        .map(|(id, api)| (id.as_str(), api))
        .collect::<BTreeMap<_, _>>();
    for id in apis.keys() {
        validate_output_identity("API", id)?;
    }
    for participant in &compiled.participants {
        validate_output_identity("participant", participant.id())?;
    }

    let mut staged = Vec::new();
    let mut changes = Vec::new();
    for (language, destination) in &outputs {
        let parent = destination
            .parent()
            .ok_or_else(|| miette!("output requires a parent directory"))?;
        fs::create_dir_all(parent).into_diagnostic()?;
        let staging = tempfile::Builder::new()
            .prefix(".trellis-stage-")
            .tempdir_in(parent)
            .into_diagnostic()?;
        let fresh = staging.path().join("new");
        match *language {
            "rust" => trellis_codegen_rust::generate_rust_package(
                &apis,
                &compiled.participants,
                &fresh,
                name,
            )
            .into_diagnostic()?,
            _ => {
                trellis_codegen_ts::generate_ts_package(&apis, &compiled.participants, &fresh, name)
                    .into_diagnostic()?
            }
        }
        for (id, api) in &apis {
            let path = fresh.join("artifacts/apis").join(format!("{id}.json"));
            fs::create_dir_all(path.parent().expect("artifact parent")).into_diagnostic()?;
            fs::write(
                path,
                format!("{}\n", api.canonical_json().into_diagnostic()?),
            )
            .into_diagnostic()?;
        }
        for participant in &compiled.participants {
            let path = fresh
                .join("artifacts/participants")
                .join(format!("{}.json", participant.id()));
            fs::create_dir_all(path.parent().expect("artifact parent")).into_diagnostic()?;
            fs::write(
                path,
                format!("{}\n", participant.canonical_json().into_diagnostic()?),
            )
            .into_diagnostic()?;
        }
        let mut files = BTreeSet::new();
        collect_files(&fresh, &fresh, &mut files)?;
        let mut existing = BTreeSet::new();
        if destination.exists() {
            collect_files(destination, destination, &mut existing)?;
        }
        let mut package_owned = false;
        for manifest in ["Cargo.toml", "package.json"] {
            if existing.contains(Path::new(manifest)) {
                package_owned |= generated_file(
                    Path::new(manifest),
                    &fs::read(destination.join(manifest)).into_diagnostic()?,
                );
            }
        }
        for relative in files.union(&existing) {
            let old = destination.join(relative);
            let new = fresh.join(relative);
            let previous = if old.exists() {
                Some(fs::read(&old).into_diagnostic()?)
            } else {
                None
            };
            let owned = package_owned
                && previous
                    .as_ref()
                    .is_some_and(|bytes| generated_file(relative, bytes));
            if new.is_file() {
                miette::ensure!(
                    previous.is_none() || owned,
                    "refusing to overwrite unrelated file {}",
                    old.display()
                );
                let next = fs::read(&new).into_diagnostic()?;
                if previous.as_deref() == Some(next.as_slice()) {
                    continue;
                }
            } else if !owned {
                continue;
            }
            changes.push((
                old,
                new,
                staging.path().join("backup").join(relative),
                previous.is_some(),
            ));
        }
        staged.push(staging);
    }

    let mut saved = Vec::new();
    let mut published = Vec::new();
    let publication = (|| -> std::io::Result<()> {
        for (old, new, backup, exists) in &changes {
            if *exists {
                fs::create_dir_all(backup.parent().expect("backup parent"))?;
                fs::rename(old, backup)?;
                saved.push((old, backup));
            }
            if new.is_file() {
                fs::create_dir_all(old.parent().expect("output parent"))?;
                fs::rename(new, old)?;
                published.push((old, new));
            }
        }
        Ok(())
    })();
    if let Err(error) = publication {
        let mut rollback_error = None;
        for (old, new) in published.into_iter().rev() {
            if let Err(error) = fs::rename(old, new) {
                rollback_error.get_or_insert(error);
            }
        }
        for (old, backup) in saved.into_iter().rev() {
            if let Err(error) = fs::rename(backup, old) {
                rollback_error.get_or_insert(error);
            }
        }
        if let Some(rollback_error) = rollback_error {
            let recovery = staged
                .into_iter()
                .map(|stage| stage.keep().display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(miette!("generation publication failed: {error}; rollback failed: {rollback_error}; recoverable outputs retained at {recovery}"));
        }
        return Err(error).into_diagnostic();
    }
    Ok(outputs.len())
}

fn collect_files(root: &Path, directory: &Path, files: &mut BTreeSet<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();
        let kind = entry.file_type().into_diagnostic()?;
        miette::ensure!(
            !kind.is_symlink(),
            "generated output must not contain symlinks: {}",
            path.display()
        );
        if kind.is_dir() {
            if !matches!(
                entry.file_name().to_str(),
                Some("node_modules" | "target" | ".git")
            ) {
                collect_files(root, &path, files)?;
            }
        } else {
            miette::ensure!(
                kind.is_file(),
                "generated output contains a non-file: {}",
                path.display()
            );
            files.insert(
                path.strip_prefix(root)
                    .expect("output descendant")
                    .to_path_buf(),
            );
        }
    }
    Ok(())
}

fn generated_file(path: &Path, bytes: &[u8]) -> bool {
    if bytes.starts_with(b"// Generated by Trellis.")
        || bytes.starts_with(b"# Generated by Trellis.")
    {
        return true;
    }
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return false;
    };
    if path == Path::new("package.json") {
        return value["description"] == "Generated Trellis APIs and participants."
            && value["private"] == true
            && value["version"] == "0.0.0";
    }
    if path.parent() == Some(Path::new("artifacts/apis")) {
        return trellis_protocol::parse_api(&value)
            .is_ok_and(|api| path.file_stem().is_some_and(|stem| stem == api.id()));
    }
    if path.parent() == Some(Path::new("artifacts/participants")) {
        return trellis_protocol::parse_participant(&value).is_ok_and(|participant| {
            path.file_stem()
                .is_some_and(|stem| stem == participant.id())
        });
    }
    false
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
