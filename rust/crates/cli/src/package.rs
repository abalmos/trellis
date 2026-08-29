//! Local path API dependency commands.

use std::{collections::BTreeMap, fs, path::Path};

use miette::{miette, IntoDiagnostic, Result, WrapErr};
use semver::{Version, VersionReq};
use serde::Serialize;

use crate::{
    cli::{AddArgs, OutputFormat, ProjectRootArgs, RmArgs},
    output,
    project::{
        read_lock, read_manifest, restore_project_files, write_lock, write_manifest_and_lock,
        ApiDependency, LockedApi, ProjectLock, ProjectManifest,
    },
};

#[derive(Debug, Serialize)]
struct PackageResult {
    installed_apis: usize,
    changed_dependencies: usize,
    generated_projects: usize,
}

pub fn add(format: OutputFormat, args: &AddArgs) -> Result<()> {
    let root = canonical_root(&args.project.root)?;
    let manifest_path = root.join("trellis.toml");
    let previous_manifest = fs::read(&manifest_path).into_diagnostic()?;
    let previous_lock = read_optional(&root.join("trellis.lock"))?;
    let mut manifest = read_manifest(&manifest_path)?;
    if args.api_json_path.is_absolute() {
        return Err(miette!(
            "API paths in trellis.toml must be relative to the project root"
        ));
    }
    let (id, release, _) = resolve_path_api(
        &root,
        "",
        &ApiDependency {
            version: "*".to_owned(),
            path: args.api_json_path.to_string_lossy().into_owned(),
        },
    )?;
    let requirement = args
        .version
        .clone()
        .unwrap_or_else(|| format!("^{release}"));
    VersionReq::parse(&requirement)
        .map_err(|error| miette!("invalid version requirement '{requirement}': {error}"))?;
    manifest.apis.insert(
        id.clone(),
        ApiDependency {
            version: requirement.clone(),
            path: args.api_json_path.to_string_lossy().into_owned(),
        },
    );
    let lock = resolve_lock(&root, &manifest)?;
    let result = commit_and_install(
        &root,
        &manifest,
        &lock,
        &previous_manifest,
        previous_lock.as_deref(),
    )?;
    print_result(format, &result, Some(format!("Added {id} {requirement}")))
}

pub fn remove(format: OutputFormat, args: &RmArgs) -> Result<()> {
    let root = canonical_root(&args.project.root)?;
    let previous_manifest = fs::read(root.join("trellis.toml")).into_diagnostic()?;
    let previous_lock = read_optional(&root.join("trellis.lock"))?;
    let mut manifest = read_manifest(&root.join("trellis.toml"))?;
    if manifest.apis.remove(&args.api_id).is_none() {
        return Err(miette!("API '{}' is not in trellis.toml", args.api_id));
    }
    let lock = resolve_lock(&root, &manifest)?;
    let result = commit_and_install(
        &root,
        &manifest,
        &lock,
        &previous_manifest,
        previous_lock.as_deref(),
    )?;
    print_result(format, &result, Some(format!("Removed {}", args.api_id)))
}

pub fn update(format: OutputFormat, args: &ProjectRootArgs) -> Result<()> {
    let root = canonical_root(&args.root)?;
    let manifest = read_manifest(&root.join("trellis.toml"))?;
    let previous_lock = read_optional(&root.join("trellis.lock"))?;
    let lock = resolve_lock(&root, &manifest)?;
    write_lock(&root.join("trellis.lock"), &lock)?;
    let result = match install_root(&root, &manifest, &lock) {
        Ok(result) => result,
        Err(error) => {
            restore_project_files(
                &root.join("trellis.toml"),
                &fs::read(root.join("trellis.toml")).into_diagnostic()?,
                &root.join("trellis.lock"),
                previous_lock.as_deref(),
            )?;
            return Err(error);
        }
    };
    print_result(format, &result, None)
}

pub fn install(format: OutputFormat, args: &ProjectRootArgs) -> Result<()> {
    let root = canonical_root(&args.root)?;
    let manifest = read_manifest(&root.join("trellis.toml"))?;
    let lock_path = root.join("trellis.lock");
    if !lock_path.exists() {
        return Err(miette!("trellis.lock is missing; run `trellis update`"));
    }
    let lock = read_lock(&lock_path)?;
    let result = install_root(&root, &manifest, &lock)?;
    print_result(format, &result, None)
}

fn canonical_root(root: &Path) -> Result<std::path::PathBuf> {
    root.canonicalize()
        .into_diagnostic()
        .wrap_err_with(|| format!("invalid project root {}", root.display()))
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).into_diagnostic(),
    }
}

fn commit_and_install(
    root: &Path,
    manifest: &ProjectManifest,
    lock: &ProjectLock,
    previous_manifest: &[u8],
    previous_lock: Option<&[u8]>,
) -> Result<PackageResult> {
    let manifest_path = root.join("trellis.toml");
    let lock_path = root.join("trellis.lock");
    write_manifest_and_lock(&manifest_path, manifest, &lock_path, lock)?;
    match install_root(root, manifest, lock) {
        Ok(result) => Ok(result),
        Err(error) => {
            restore_project_files(&manifest_path, previous_manifest, &lock_path, previous_lock)?;
            Err(error)
        }
    }
}

fn resolve_path_api(
    root: &Path,
    expected_id: &str,
    dependency: &ApiDependency,
) -> Result<(String, Version, String)> {
    let path = root.join(&dependency.path);
    let value: serde_json::Value = serde_json::from_slice(
        &fs::read(&path)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to read API artifact {}", path.display()))?,
    )
    .into_diagnostic()
    .wrap_err_with(|| format!("failed to parse API artifact {}", path.display()))?;
    let api = trellis_protocol::parse_api(&value).map_err(|error| miette!(error.to_string()))?;
    if !expected_id.is_empty() && api.id() != expected_id {
        return Err(miette!(
            "manifest API '{expected_id}' but path contains '{}'",
            api.id()
        ));
    }
    let version = Version::parse(api.version())
        .map_err(|error| miette!("invalid release version for '{}': {error}", api.id()))?;
    let requirement = VersionReq::parse(&dependency.version).map_err(|error| miette!(error))?;
    if !requirement.matches(&version) {
        return Err(miette!(
            "{} release {} does not satisfy {}",
            api.id(),
            version,
            dependency.version
        ));
    }
    let digest = api.digest().map_err(|error| miette!(error.to_string()))?;
    Ok((api.id().to_owned(), version, digest))
}

fn resolve_lock(root: &Path, manifest: &ProjectManifest) -> Result<ProjectLock> {
    let mut api = Vec::with_capacity(manifest.apis.len());
    for (id, dependency) in &manifest.apis {
        let (_, version, api_digest) = resolve_path_api(root, id, dependency)?;
        api.push(LockedApi {
            id: id.clone(),
            version: version.to_string(),
            api_digest,
            path: dependency.path.clone(),
        });
    }
    Ok(ProjectLock {
        format: 1,
        manifest_digest: manifest.digest()?,
        api,
    })
}

fn install_root(
    root: &Path,
    manifest: &ProjectManifest,
    lock: &ProjectLock,
) -> Result<PackageResult> {
    if lock.manifest_digest != manifest.digest()? {
        return Err(miette!(
            "trellis.toml changed since trellis.lock; run `trellis update`"
        ));
    }
    if lock.api.len() != manifest.apis.len() {
        return Err(miette!(
            "trellis.lock does not match trellis.toml; run `trellis update`"
        ));
    }
    for locked in &lock.api {
        trellis_generate::planning::validate_output_identity("API", &locked.id)?;
        let dependency = manifest
            .apis
            .get(&locked.id)
            .ok_or_else(|| miette!("locked API '{}' is absent from trellis.toml", locked.id))?;
        if dependency.path != locked.path {
            return Err(miette!(
                "locked path for '{}' does not match trellis.toml",
                locked.id
            ));
        }
        let (_, version, digest) = resolve_path_api(root, &locked.id, dependency)?;
        if version.to_string() != locked.version {
            return Err(miette!(
                "locked {} is {} but path now contains {}; run `trellis update`",
                locked.id,
                locked.version,
                version
            ));
        }
        if digest != locked.api_digest {
            return Err(miette!(
                "locked API digest does not match canonical artifact at {}",
                dependency.path
            ));
        }
    }

    let discovered = trellis_generate::discovery::discover_contracts(root)?;
    let has_ts = discovered
        .iter()
        .any(|item| item.language == trellis_generate::discovery::SourceLanguage::TypeScript);
    let has_rust = discovered
        .iter()
        .any(|item| item.language == trellis_generate::discovery::SourceLanguage::Rust);
    if has_ts {
        let config = [root.join("deno.json"), root.join("deno.jsonc")]
            .into_iter()
            .find(|path| path.is_file())
            .ok_or_else(|| {
                miette!("TypeScript contract source requires deno.json or deno.jsonc")
            })?;
        let contents = fs::read_to_string(config).into_diagnostic()?;
        if !contents.contains(".trellis/generated/ts/trellis-apis") {
            return Err(miette!(
                "add `.trellis/generated/ts/trellis-apis` to the root Deno `links` array"
            ));
        }
    }
    if has_rust {
        let mut package_stems = BTreeMap::new();
        for api in &lock.api {
            let stem = trellis_generate::artifacts::sdk_output_stem(&api.id);
            if let Some(existing) = package_stems.insert(stem.clone(), &api.id) {
                return Err(miette!(
                    "Rust SDK output '{stem}' collides between '{existing}' and '{}'",
                    api.id
                ));
            }
        }
        let manifest_path = root.join("Cargo.toml");
        let contents = fs::read_to_string(&manifest_path)
            .into_diagnostic()
            .wrap_err("Rust contract source requires a root Cargo.toml")?;
        let cargo: toml::Value = toml::from_str(&contents).into_diagnostic()?;
        let dependency = cargo
            .get("dependencies")
            .and_then(|value| value.get("trellis-apis"))
            .or_else(|| {
                cargo
                    .get("workspace")
                    .and_then(|value| value.get("dependencies"))
                    .and_then(|value| value.get("trellis-apis"))
            });
        let expected_path = ".trellis/generated/rust/trellis-apis";
        if dependency.and_then(|value| value.get("path"))
            != Some(&toml::Value::String(expected_path.to_owned()))
        {
            return Err(miette!(
                "add `trellis-apis = {{ path = \"{expected_path}\" }}` to the root Cargo.toml"
            ));
        }
    }
    let fingerprints = trellis_generate::artifacts::current_generator_fingerprints();
    let marker = trellis_protocol::digest_json(&serde_json::json!({
        "lock": lock,
        "typescript": has_ts,
        "rust": has_rust,
        "model": fingerprints.model,
        "tsCodegen": fingerprints.ts,
        "rustCodegen": fingerprints.rust,
        "runtimeVersion": trellis_generate::artifacts::trellis_package_version(),
    }))
    .map_err(|error| miette!(error.to_string()))?;
    let trellis_root = root.join(".trellis");
    let marker_path = trellis_root.join("install-digest");
    let dependencies_fresh = fs::read_to_string(&marker_path).ok().as_deref() == Some(&marker)
        && lock.api.iter().all(|api| {
            let materialized = trellis_root
                .join("apis")
                .join(&api.id)
                .join(&api.version)
                .join("trellis.api.json");
            let stem = api.id.split('@').next().unwrap_or(&api.id);
            let ts_out = has_ts.then(|| trellis_root.join("generated/ts/packages").join(stem));
            let rust_out = has_rust.then(|| {
                trellis_root
                    .join("generated/rust/packages")
                    .join(trellis_generate::artifacts::sdk_output_stem(&api.id))
            });
            trellis_generate::artifacts::installed_api_is_fresh(
                &api.id,
                &api.version,
                &api.api_digest,
                &materialized,
                ts_out.as_deref(),
                rust_out.as_deref(),
            )
        })
        && (!has_rust
            || trellis_root
                .join("generated/rust/trellis-apis/Cargo.toml")
                .is_file())
        && (!has_ts
            || trellis_root
                .join("generated/ts/trellis-apis/deno.json")
                .is_file());
    if dependencies_fresh {
        let generated = trellis_generate::commands::prepare::generate_project(root, &trellis_root)?;
        return Ok(PackageResult {
            installed_apis: lock.api.len(),
            changed_dependencies: 0,
            generated_projects: generated.generated,
        });
    }

    let backup_root = tempfile::Builder::new()
        .prefix(".trellis-install-backup-")
        .tempdir_in(root)
        .into_diagnostic()?;
    let backup = backup_root.path().join(".trellis");
    if trellis_root.exists() {
        fs::rename(&trellis_root, &backup).into_diagnostic()?;
    }
    let install_result = (|| {
        let changed_dependencies =
            stage_dependencies(root, lock, has_ts, has_rust, &trellis_root, &marker)?;
        let generated = trellis_generate::commands::prepare::generate_project(root, &trellis_root)?;
        Ok(PackageResult {
            installed_apis: lock.api.len(),
            changed_dependencies,
            generated_projects: generated.generated,
        })
    })();
    match install_result {
        Ok(result) => Ok(result),
        Err(error) => {
            remove_path(&trellis_root)?;
            if backup.exists() {
                fs::rename(&backup, &trellis_root)
                    .into_diagnostic()
                    .wrap_err("failed to restore the previous .trellis installation")?;
            }
            Err(error)
        }
    }
}

fn remove_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path).into_diagnostic()?;
    } else if path.exists() {
        fs::remove_file(path).into_diagnostic()?;
    }
    Ok(())
}

fn stage_dependencies(
    root: &Path,
    lock: &ProjectLock,
    has_ts: bool,
    has_rust: bool,
    trellis_root: &Path,
    marker: &str,
) -> Result<usize> {
    fs::create_dir_all(trellis_root).into_diagnostic()?;
    let staging = tempfile::Builder::new()
        .prefix(".install-")
        .tempdir_in(trellis_root)
        .into_diagnostic()?;
    let staged = staging.path();
    let mut modules = BTreeMap::new();
    for locked in &lock.api {
        let stem = locked.id.split('@').next().unwrap_or(&locked.id);
        let package_stem = trellis_generate::artifacts::sdk_output_stem(&locked.id);
        let api_out = staged
            .join("apis")
            .join(&locked.id)
            .join(&locked.version)
            .join("trellis.api.json");
        let ts_out = has_ts.then(|| staged.join("generated/ts/packages").join(stem));
        let rust_out = has_rust.then(|| staged.join("generated/rust/packages").join(&package_stem));
        trellis_generate::artifacts::generate_installed_api(
            &root.join(&locked.path),
            &api_out,
            ts_out.as_deref(),
            rust_out.as_deref(),
        )?;
        if has_rust {
            let module = stem.rsplit('.').next().unwrap_or(stem).replace('-', "_");
            if let Some(existing) = modules.insert(module.clone(), locked.id.clone()) {
                return Err(miette!(
                    "Rust API module name '{module}' collides between {existing} and {}",
                    locked.id
                ));
            }
        }
    }
    if has_rust {
        write_aggregate_crate(staged, &modules)?;
    }
    if has_ts {
        write_ts_aggregate(staged, lock)?;
    }
    fs::write(staged.join("install-digest"), marker).into_diagnostic()?;
    replace_managed_paths(
        staged,
        trellis_root,
        &["apis", "generated/ts", "generated/rust", "install-digest"],
    )?;
    Ok(lock.api.len())
}

fn replace_managed_paths(staged: &Path, root: &Path, relative_paths: &[&str]) -> Result<()> {
    let paths = relative_paths
        .iter()
        .map(|relative| {
            let destination = root.join(relative);
            let backup = destination.with_extension("trellis-install-old");
            (staged.join(relative), destination, backup)
        })
        .collect::<Vec<_>>();
    for (_, _, backup) in &paths {
        if backup.is_dir() {
            fs::remove_dir_all(backup).into_diagnostic()?;
        } else if backup.exists() {
            fs::remove_file(backup).into_diagnostic()?;
        }
    }
    for (_, destination, backup) in &paths {
        if destination.exists() {
            if let Err(error) = fs::rename(destination, backup) {
                for (_, previous_destination, previous_backup) in &paths {
                    if previous_backup.exists() {
                        let _ = fs::rename(previous_backup, previous_destination);
                    }
                }
                return Err(error).into_diagnostic();
            }
        }
    }
    for (source, destination, _) in &paths {
        if source.exists() {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).into_diagnostic()?;
            }
            if let Err(error) = fs::rename(source, destination) {
                for (_, moved_destination, backup) in paths.iter().rev() {
                    if moved_destination.is_dir() {
                        let _ = fs::remove_dir_all(moved_destination);
                    } else if moved_destination.exists() {
                        let _ = fs::remove_file(moved_destination);
                    }
                    if backup.exists() {
                        let _ = fs::rename(backup, moved_destination);
                    }
                }
                return Err(error).into_diagnostic();
            }
        }
    }
    for (_, _, backup) in &paths {
        if backup.is_dir() {
            let _ = fs::remove_dir_all(backup);
        } else if backup.exists() {
            let _ = fs::remove_file(backup);
        }
    }
    Ok(())
}

fn write_aggregate_crate(root: &Path, modules: &BTreeMap<String, String>) -> Result<()> {
    let aggregate = root.join("generated/rust/trellis-apis");
    fs::create_dir_all(aggregate.join("src")).into_diagnostic()?;
    let dependencies = modules
        .values()
        .map(|id| {
            let name = trellis_generate::artifacts::default_rust_crate_name_from_id(id);
            format!(
                "{name} = {{ path = \"../packages/{}\" }}\n",
                trellis_generate::artifacts::sdk_output_stem(id)
            )
        })
        .collect::<String>();
    fs::write(
        aggregate.join("Cargo.toml"),
        format!(
            "[package]\nname = \"trellis-apis\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[dependencies]\n{dependencies}"
        ),
    )
    .into_diagnostic()?;
    let modules = modules
        .iter()
        .map(|(module, id)| {
            let crate_name = trellis_generate::artifacts::default_rust_crate_name_from_id(id);
            format!(
                "pub mod {module} {{\n    pub use {}::*;\n}}\n",
                crate_name.replace('-', "_")
            )
        })
        .collect::<String>();
    fs::write(aggregate.join("src/lib.rs"), modules).into_diagnostic()?;
    Ok(())
}

fn write_ts_aggregate(root: &Path, lock: &ProjectLock) -> Result<()> {
    let aggregate = root.join("generated/ts/trellis-apis");
    fs::create_dir_all(&aggregate).into_diagnostic()?;
    let mut exports = serde_json::Map::new();
    for api in &lock.api {
        let stem = api.id.split('@').next().unwrap_or(&api.id);
        let file = format!("{stem}.ts");
        exports.insert(format!("./{stem}"), serde_json::json!(format!("./{file}")));
        fs::write(
            aggregate.join(&file),
            format!("export * from \"../packages/{stem}/mod.ts\";\n"),
        )
        .into_diagnostic()?;
    }
    fs::write(
        aggregate.join("deno.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "name": "@trellis/apis",
                "version": "0.0.0",
                "exports": exports,
                "publish": false,
            }))
            .into_diagnostic()?
        ),
    )
    .into_diagnostic()?;
    Ok(())
}

fn print_result(
    format: OutputFormat,
    result: &PackageResult,
    headline: Option<String>,
) -> Result<()> {
    if output::is_json(format) {
        output::print_json(result)
    } else {
        if let Some(headline) = headline {
            println!("{headline}");
        }
        if result.changed_dependencies == 0 && result.generated_projects == 0 {
            println!("Installed 0 changes");
        } else {
            println!("Installed {} API(s)", result.installed_apis);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_is_exact_and_preserves_previous_materialization_on_drift() {
        let root = tempfile::tempdir().unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../runtime/trellis.api.json");
        fs::copy(source, root.path().join("auth.json")).unwrap();
        let manifest = ProjectManifest {
            format: 1,
            apis: BTreeMap::from([(
                "trellis.auth@v1".to_owned(),
                ApiDependency {
                    version: "^1.0".to_owned(),
                    path: "auth.json".to_owned(),
                },
            )]),
        };
        let lock = resolve_lock(root.path(), &manifest).unwrap();
        install_root(root.path(), &manifest, &lock).unwrap();
        let installed = root
            .path()
            .join(".trellis/apis/trellis.auth@v1/1.0.0/trellis.api.json");
        let previous = fs::read(&installed).unwrap();

        let mut changed: serde_json::Value =
            serde_json::from_slice(&fs::read(root.path().join("auth.json")).unwrap()).unwrap();
        changed["version"] = serde_json::json!("1.0.1");
        fs::write(
            root.path().join("auth.json"),
            serde_json::to_vec_pretty(&changed).unwrap(),
        )
        .unwrap();
        let error = install_root(root.path(), &manifest, &lock).unwrap_err();
        assert!(error.to_string().contains("path now contains 1.0.1"));
        assert_eq!(fs::read(&installed).unwrap(), previous);

        changed["version"] = serde_json::json!("1.0.0");
        changed["schemas"]["AuthSessionsMeRequest"]["properties"]["extra"] =
            serde_json::json!({ "type": "string" });
        fs::write(
            root.path().join("auth.json"),
            serde_json::to_vec_pretty(&changed).unwrap(),
        )
        .unwrap();
        let error = install_root(root.path(), &manifest, &lock).unwrap_err();
        assert!(error.to_string().contains("digest does not match"));
        assert_eq!(fs::read(installed).unwrap(), previous);
    }

    #[test]
    fn add_update_and_remove_share_the_locked_installer() {
        let root = tempfile::tempdir().unwrap();
        crate::project::write_manifest(
            &root.path().join("trellis.toml"),
            &ProjectManifest {
                format: 1,
                apis: BTreeMap::new(),
            },
        )
        .unwrap();
        let api_path = root.path().join("auth.json");
        let mut api = serde_json::json!({
            "format": "trellis.api.v1",
            "id": "acme.auth@v1",
            "version": "1.4.2",
            "displayName": "Auth",
            "description": "Fixture API"
        });
        fs::write(&api_path, serde_json::to_vec_pretty(&api).unwrap()).unwrap();

        add(
            OutputFormat::Text,
            &AddArgs {
                api_json_path: Path::new("auth.json").to_path_buf(),
                version: None,
                project: ProjectRootArgs {
                    root: root.path().to_path_buf(),
                },
            },
        )
        .unwrap();
        let manifest = crate::project::read_manifest(&root.path().join("trellis.toml")).unwrap();
        assert_eq!(manifest.apis["acme.auth@v1"].version, "^1.4.2");
        let initial_lock = crate::project::read_lock(&root.path().join("trellis.lock")).unwrap();
        assert_eq!(initial_lock.api[0].version, "1.4.2");

        api["version"] = serde_json::json!("1.4.3");
        fs::write(&api_path, serde_json::to_vec_pretty(&api).unwrap()).unwrap();
        let project = ProjectRootArgs {
            root: root.path().to_path_buf(),
        };
        update(OutputFormat::Text, &project).unwrap();
        let updated_lock = crate::project::read_lock(&root.path().join("trellis.lock")).unwrap();
        assert_eq!(updated_lock.api[0].version, "1.4.3");
        assert_eq!(
            updated_lock.api[0].api_digest,
            initial_lock.api[0].api_digest
        );
        assert!(root
            .path()
            .join(".trellis/apis/acme.auth@v1/1.4.3/trellis.api.json")
            .is_file());
        assert!(!root
            .path()
            .join(".trellis/apis/acme.auth@v1/1.4.2")
            .exists());

        api["version"] = serde_json::json!("2.0.0");
        fs::write(&api_path, serde_json::to_vec_pretty(&api).unwrap()).unwrap();
        assert!(update(OutputFormat::Text, &project)
            .unwrap_err()
            .to_string()
            .contains("does not satisfy ^1.4.2"));
        assert_eq!(
            crate::project::read_lock(&root.path().join("trellis.lock")).unwrap(),
            updated_lock
        );

        remove(
            OutputFormat::Text,
            &RmArgs {
                api_id: "acme.auth@v1".to_owned(),
                project,
            },
        )
        .unwrap();
        assert!(
            crate::project::read_manifest(&root.path().join("trellis.toml"))
                .unwrap()
                .apis
                .is_empty()
        );
        assert!(crate::project::read_lock(&root.path().join("trellis.lock"))
            .unwrap()
            .api
            .is_empty());
        assert!(!root.path().join(".trellis/apis/acme.auth@v1").exists());
        assert!(api_path.is_file());

        let empty_lock = crate::project::read_lock(&root.path().join("trellis.lock")).unwrap();
        fs::remove_file(root.path().join("trellis.lock")).unwrap();
        assert!(install(
            OutputFormat::Text,
            &ProjectRootArgs {
                root: root.path().to_path_buf()
            }
        )
        .unwrap_err()
        .to_string()
        .contains("trellis.lock is missing"));
        crate::project::write_lock(&root.path().join("trellis.lock"), &empty_lock).unwrap();
        let mut changed_manifest =
            crate::project::read_manifest(&root.path().join("trellis.toml")).unwrap();
        changed_manifest.apis.insert(
            "acme.auth@v1".to_owned(),
            ApiDependency {
                version: "^1.4".to_owned(),
                path: "auth.json".to_owned(),
            },
        );
        crate::project::write_manifest(&root.path().join("trellis.toml"), &changed_manifest)
            .unwrap();
        assert!(install(
            OutputFormat::Text,
            &ProjectRootArgs {
                root: root.path().to_path_buf()
            }
        )
        .unwrap_err()
        .to_string()
        .contains("trellis.toml changed since trellis.lock"));
    }

    #[test]
    fn install_bootstraps_typescript_dependency_before_participant() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        let root = tempfile::tempdir().unwrap();
        let mut api = serde_json::json!({
            "format": "trellis.api.v1",
            "id": "acme.auth@v1",
            "version": "1.0.0",
            "displayName": "Auth",
            "description": "Fixture API",
            "schemas": {
                "Empty": { "type": "object", "properties": {}, "required": [] },
                "Session": {
                    "type": "object",
                    "properties": { "id": { "type": "string" } },
                    "required": ["id"]
                }
            },
            "rpc": {
                "Auth.Sessions.Me": {
                    "version": "v1",
                    "input": { "schema": "Empty" },
                    "output": { "schema": "Session" }
                }
            }
        });
        fs::write(
            root.path().join("auth.json"),
            serde_json::to_vec_pretty(&api).unwrap(),
        )
        .unwrap();
        fs::write(
            root.path().join("deno.json"),
            format!(
                r#"{{
  "links": [".trellis/generated/ts/trellis-apis"],
  "imports": {{
    "@qlever-llc/trellis": "file://{0}/ts/packages/trellis/index.ts",
    "@qlever-llc/trellis/": "file://{0}/ts/packages/trellis/",
    "@qlever-llc/trellis/contracts": "file://{0}/ts/packages/trellis/contracts.ts"
  }}
}}
"#,
                repo.display()
            ),
        )
        .unwrap();
        fs::write(
            root.path().join("package.json"),
            "{\"name\":\"consumer\",\"version\":\"1.0.0\"}\n",
        )
        .unwrap();
        fs::write(
            root.path().join("contract.ts"),
            r#"import { defineAppContract } from "@qlever-llc/trellis";
import { AuthSessionsMe } from "@trellis/apis/acme.auth";

export default defineAppContract(() => ({
  id: "acme.consumer@v1",
  apiId: "acme.consumer-api@v1",
  apiVersion: "1.0.0",
  displayName: "Consumer",
  description: "Consumer fixture",
  uses: [AuthSessionsMe],
}));
"#,
        )
        .unwrap();
        let manifest = ProjectManifest {
            format: 1,
            apis: BTreeMap::from([(
                "acme.auth@v1".to_owned(),
                ApiDependency {
                    version: "^1.0".to_owned(),
                    path: "auth.json".to_owned(),
                },
            )]),
        };
        let lock = resolve_lock(root.path(), &manifest).unwrap();

        install_root(root.path(), &manifest, &lock).unwrap();
        let participant: serde_json::Value = serde_json::from_slice(
            &fs::read(
                root.path()
                    .join(".trellis/generated/protocol/participants/acme.consumer@v1.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let used = participant["uses"]["required"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap();
        assert_eq!(used["api"], "acme.auth@v1");
        assert_eq!(used["apiDigest"], lock.api[0].api_digest);

        let warm = install_root(root.path(), &manifest, &lock).unwrap();
        assert_eq!(warm.changed_dependencies, 0);
        assert_eq!(warm.generated_projects, 0);

        let participant_path = root
            .path()
            .join(".trellis/generated/protocol/participants/acme.consumer@v1.json");
        let previous_participant = fs::read(&participant_path).unwrap();
        api["version"] = serde_json::json!("1.0.1");
        fs::write(
            root.path().join("auth.json"),
            serde_json::to_vec_pretty(&api).unwrap(),
        )
        .unwrap();
        let next_lock = resolve_lock(root.path(), &manifest).unwrap();
        let source = fs::read_to_string(root.path().join("contract.ts")).unwrap();
        fs::write(
            root.path().join("contract.ts"),
            format!("throw new Error(\"fixture failure\");\n{source}"),
        )
        .unwrap();
        assert!(install_root(root.path(), &manifest, &next_lock).is_err());
        assert!(root
            .path()
            .join(".trellis/apis/acme.auth@v1/1.0.0/trellis.api.json")
            .is_file());
        assert!(!root
            .path()
            .join(".trellis/apis/acme.auth@v1/1.0.1")
            .exists());
        assert_eq!(fs::read(participant_path).unwrap(), previous_participant);
    }

    #[test]
    fn install_bootstraps_rust_dependency_before_participant() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        let root = tempfile::tempdir().unwrap();
        let api = serde_json::json!({
            "format": "trellis.api.v1",
            "id": "trellis.jobs@v1",
            "version": "1.0.0",
            "displayName": "Auth",
            "description": "Fixture API",
            "schemas": {
                "Empty": { "type": "object", "properties": {}, "required": [] },
                "Session": {
                    "type": "object",
                    "properties": { "id": { "type": "string" } },
                    "required": ["id"]
                }
            },
            "rpc": {
                "Auth.Sessions.Me": {
                    "version": "v1",
                    "input": { "schema": "Empty" },
                    "output": { "schema": "Session" }
                }
            }
        });
        fs::write(
            root.path().join("auth.json"),
            serde_json::to_vec_pretty(&api).unwrap(),
        )
        .unwrap();
        fs::create_dir(root.path().join("contracts")).unwrap();
        fs::create_dir(root.path().join("src")).unwrap();
        fs::write(root.path().join("src/lib.rs"), "").unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            format!(
                "[package]\nname = \"consumer\"\nversion = \"1.0.0\"\nedition = \"2021\"\n\n[dependencies]\nserde_json = \"1\"\ntrellis-contracts = {{ path = \"{}/rust/crates/contracts\" }}\ntrellis-apis = {{ path = \".trellis/generated/rust/trellis-apis\" }}\n",
                repo.display()
            ),
        )
        .unwrap();
        fs::write(
            root.path().join("contracts/consumer.rs"),
            r#"use trellis_apis::jobs::{api::API_JSON, rpc::AuthSessionsMeRpc, API_ID};
use trellis_contracts::{use_contract, ApiArtifact, ApiBuilder, ContractArtifacts, ContractBuilder, ContractKind, ContractsError};

pub fn api_artifact() -> Result<ApiArtifact, ContractsError> {
    ApiBuilder::new(serde_json::json!({
        "format": "trellis.api.v1",
        "id": "acme.consumer-api@v1",
        "version": "1.0.0",
        "displayName": "Consumer",
        "description": "Consumer fixture"
    })).build()
}

pub fn contract_artifacts() -> Result<ContractArtifacts, ContractsError> {
    let _ = std::any::TypeId::of::<AuthSessionsMeRpc>();
    ContractBuilder::authoring("acme.consumer@v1", "acme.consumer-api@v1", "1.0.0", "Consumer", "Consumer fixture", ContractKind::App)
        .use_ref("jobs", use_contract(API_ID).with_rpc_call(["Auth.Sessions.Me"]))
        .referenced_api(API_ID, serde_json::from_str(API_JSON)?)
        .build()
}
"#,
        )
        .unwrap();
        let manifest = ProjectManifest {
            format: 1,
            apis: BTreeMap::from([(
                "trellis.jobs@v1".to_owned(),
                ApiDependency {
                    version: "^1.0".to_owned(),
                    path: "auth.json".to_owned(),
                },
            )]),
        };
        let lock = resolve_lock(root.path(), &manifest).unwrap();

        install_root(root.path(), &manifest, &lock).unwrap();
        let participant: serde_json::Value = serde_json::from_slice(
            &fs::read(
                root.path()
                    .join(".trellis/generated/protocol/participants/acme.consumer@v1.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            participant["uses"]["required"]["jobs"]["api"],
            "trellis.jobs@v1"
        );
        assert_eq!(
            participant["uses"]["required"]["jobs"]["apiDigest"],
            lock.api[0].api_digest
        );
        assert!(root
            .path()
            .join(".trellis/generated/rust/trellis-apis/src/lib.rs")
            .is_file());
        let own_sdk = fs::read_to_string(
            root.path()
                .join(".trellis/generated/packages/cargo/acme-consumer-api/Cargo.toml"),
        )
        .unwrap();
        assert!(own_sdk.contains(&format!(
            "trellis-rs = \"{}\"",
            trellis_generate::artifacts::trellis_package_version()
        )));
        let participant_facade = fs::read_to_string(
            root.path()
                .join(".trellis/generated/packages/cargo-participants/acme-consumer/Cargo.toml"),
        )
        .unwrap();
        assert!(participant_facade.contains(&format!(
            "trellis-rs = \"{}\"",
            trellis_generate::artifacts::trellis_package_version()
        )));
    }
}
