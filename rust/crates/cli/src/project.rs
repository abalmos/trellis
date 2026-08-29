//! Project API dependency manifest and exact-release lock models.

use std::{collections::BTreeMap, fs, path::Path};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

/// One local canonical API dependency.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApiDependency {
    /// Accepted release versions.
    pub version: String,
    /// Path to a canonical `trellis.api.v1` JSON artifact.
    pub path: String,
}

/// A project's `trellis.toml` dependency model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectManifest {
    /// Manifest format version. Must be `1`.
    pub format: u32,
    /// Dependencies keyed by stable API ID.
    #[serde(default)]
    pub apis: BTreeMap<String, ApiDependency>,
}

impl ProjectManifest {
    /// Validate manifest format and dependency declarations.
    pub fn validate(&self) -> Result<()> {
        if self.format != 1 {
            return Err(miette!("manifest format must equal 1"));
        }
        let mut lineages = BTreeMap::new();
        for (id, dependency) in &self.apis {
            trellis_protocol::validate_api_id(id)
                .map_err(|error| miette!("invalid API id '{id}': {error}"))?;
            VersionReq::parse(&dependency.version)
                .map_err(|error| miette!("invalid version requirement for API '{id}': {error}"))?;
            // ponytail: paths stay declarative here; install owns filesystem resolution.
            if dependency.path.is_empty() {
                return Err(miette!("path for API '{id}' must not be empty"));
            }
            if Path::new(&dependency.path).is_absolute() {
                return Err(miette!(
                    "path for API '{id}' must be relative to the project root"
                ));
            }
            let lineage = id.split('@').next().unwrap_or(id);
            if let Some(existing) = lineages.insert(lineage, id) {
                return Err(miette!(
                    "API lineage '{lineage}' collides between '{existing}' and '{id}'; install only one transport major"
                ));
            }
        }
        Ok(())
    }

    /// Compute the canonical semantic digest used by `trellis.lock`.
    pub fn digest(&self) -> Result<String> {
        self.validate()?;
        trellis_protocol::digest_json(&serde_json::to_value(self).into_diagnostic()?)
            .map_err(|error| miette!(error.to_string()))
    }
}

/// One exact API release pinned by `trellis.lock`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct LockedApi {
    /// Stable API identity.
    pub id: String,
    /// Exact API release version.
    pub version: String,
    /// Exact semantic API digest.
    pub api_digest: String,
    /// Path copied from the manifest dependency.
    pub path: String,
}

/// A project's exact-release `trellis.lock` model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ProjectLock {
    /// Lock format version. Must be `1`.
    pub format: u32,
    /// Semantic digest of the corresponding manifest.
    pub manifest_digest: String,
    /// Exact API releases.
    #[serde(default)]
    pub api: Vec<LockedApi>,
}

impl ProjectLock {
    /// Validate lock format and exact API release declarations.
    pub fn validate(&self) -> Result<()> {
        if self.format != 1 {
            return Err(miette!("lock format must equal 1"));
        }
        validate_digest("manifest-digest", &self.manifest_digest)?;
        let mut ids = BTreeMap::new();
        let mut lineages = BTreeMap::new();
        for api in &self.api {
            trellis_protocol::validate_api_id(&api.id)
                .map_err(|error| miette!("invalid API id '{}': {error}", api.id))?;
            Version::parse(&api.version)
                .map_err(|error| miette!("invalid version for API '{}': {error}", api.id))?;
            validate_digest("api-digest", &api.api_digest)?;
            if api.path.is_empty() {
                return Err(miette!("path for API '{}' must not be empty", api.id));
            }
            if Path::new(&api.path).is_absolute() {
                return Err(miette!(
                    "path for API '{}' must be relative to the project root",
                    api.id
                ));
            }
            if ids.insert(&api.id, ()).is_some() {
                return Err(miette!("duplicate locked API id '{}'", api.id));
            }
            let lineage = api.id.split('@').next().unwrap_or(&api.id);
            if let Some(existing) = lineages.insert(lineage, &api.id) {
                return Err(miette!(
                    "API lineage '{lineage}' collides between '{existing}' and '{}'; install only one transport major",
                    api.id
                ));
            }
        }
        Ok(())
    }
}

/// Read and validate `trellis.toml`.
pub fn read_manifest(path: &Path) -> Result<ProjectManifest> {
    let value = toml::from_str::<ProjectManifest>(&fs::read_to_string(path).into_diagnostic()?)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to parse {}", path.display()))?;
    value.validate()?;
    Ok(value)
}

/// Deterministically write `trellis.toml` when its contents changed.
pub fn write_manifest(path: &Path, manifest: &ProjectManifest) -> Result<()> {
    manifest.validate()?;
    write_atomic_if_changed(
        path,
        toml::to_string(manifest).into_diagnostic()?.as_bytes(),
    )
}

/// Read and validate `trellis.lock`.
pub fn read_lock(path: &Path) -> Result<ProjectLock> {
    let value = toml::from_str::<ProjectLock>(&fs::read_to_string(path).into_diagnostic()?)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to parse {}", path.display()))?;
    value.validate()?;
    Ok(value)
}

/// Deterministically write `trellis.lock` when its contents changed.
pub fn write_lock(path: &Path, lock: &ProjectLock) -> Result<()> {
    lock.validate()?;
    let mut sorted = lock.clone();
    sorted.api.sort_by(|left, right| left.id.cmp(&right.id));
    write_atomic_if_changed(path, toml::to_string(&sorted).into_diagnostic()?.as_bytes())
}

/// Replace `trellis.toml` and `trellis.lock` from fully validated values.
pub fn write_manifest_and_lock(
    manifest_path: &Path,
    manifest: &ProjectManifest,
    lock_path: &Path,
    lock: &ProjectLock,
) -> Result<()> {
    manifest.validate()?;
    lock.validate()?;
    let manifest_bytes = toml::to_string(manifest).into_diagnostic()?.into_bytes();
    let mut sorted_lock = lock.clone();
    sorted_lock
        .api
        .sort_by(|left, right| left.id.cmp(&right.id));
    let lock_bytes = toml::to_string(&sorted_lock)
        .into_diagnostic()?
        .into_bytes();
    let parent = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).into_diagnostic()?;
    let old_manifest = fs::read(manifest_path).ok();
    let mut manifest_temp = tempfile::NamedTempFile::new_in(parent).into_diagnostic()?;
    let mut lock_temp = tempfile::NamedTempFile::new_in(parent).into_diagnostic()?;
    std::io::Write::write_all(&mut manifest_temp, &manifest_bytes).into_diagnostic()?;
    std::io::Write::write_all(&mut lock_temp, &lock_bytes).into_diagnostic()?;
    manifest_temp.as_file().sync_all().into_diagnostic()?;
    lock_temp.as_file().sync_all().into_diagnostic()?;
    manifest_temp
        .persist(manifest_path)
        .map_err(|error| miette!(error))?;
    if let Err(error) = lock_temp.persist(lock_path) {
        match old_manifest {
            Some(bytes) => write_atomic_if_changed(manifest_path, &bytes)?,
            None => {
                let _ = fs::remove_file(manifest_path);
            }
        }
        return Err(miette!(error));
    }
    Ok(())
}

/// Restore the exact prior project-file bytes after a failed package mutation.
pub fn restore_project_files(
    manifest_path: &Path,
    manifest: &[u8],
    lock_path: &Path,
    lock: Option<&[u8]>,
) -> Result<()> {
    write_atomic_if_changed(manifest_path, manifest)?;
    match lock {
        Some(bytes) => write_atomic_if_changed(lock_path, bytes),
        None => {
            if lock_path.exists() {
                fs::remove_file(lock_path).into_diagnostic()?;
            }
            Ok(())
        }
    }
}

fn validate_digest(name: &str, value: &str) -> Result<()> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| miette!("{name} must be a base64url SHA-256 digest"))?;
    if bytes.len() != 32 {
        return Err(miette!("{name} must be a base64url SHA-256 digest"));
    }
    Ok(())
}

fn write_atomic_if_changed(path: &Path, bytes: &[u8]) -> Result<()> {
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).into_diagnostic()?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).into_diagnostic()?;
    std::io::Write::write_all(&mut temporary, bytes).into_diagnostic()?;
    temporary.as_file().sync_all().into_diagnostic()?;
    temporary.persist(path).map_err(|error| miette!(error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_models_validate_digest_and_round_trip_deterministically() {
        let first: ProjectManifest = toml::from_str(
            r#"
format = 1

[apis."trellis.jobs@v1"]
version = "^1.0"
path = "../apis/jobs.json"

[apis."acme.orders@v1"]
version = "^1.4"
path = "../apis/orders.json"
"#,
        )
        .unwrap();
        let reordered: ProjectManifest = toml::from_str(
            r#"format=1
[apis."acme.orders@v1"]
path="../apis/orders.json"
version="^1.4"
[apis."trellis.jobs@v1"]
path="../apis/jobs.json"
version="^1.0"
"#,
        )
        .unwrap();
        assert_eq!(first.digest().unwrap(), reordered.digest().unwrap());

        let mut invalid_manifest = first.clone();
        let dependency = invalid_manifest.apis.remove("acme.orders@v1").unwrap();
        invalid_manifest
            .apis
            .insert("acme.orders@1.4.2".into(), dependency);
        assert!(invalid_manifest.validate().is_err());
        invalid_manifest = first.clone();
        invalid_manifest
            .apis
            .get_mut("acme.orders@v1")
            .unwrap()
            .version = "banana".into();
        assert!(invalid_manifest.validate().is_err());
        invalid_manifest = first.clone();
        invalid_manifest
            .apis
            .get_mut("acme.orders@v1")
            .unwrap()
            .path = "/tmp/orders.json".into();
        assert!(invalid_manifest.validate().is_err());
        invalid_manifest = first.clone();
        invalid_manifest.apis.insert(
            "acme.orders@v2".into(),
            ApiDependency {
                version: "^2".into(),
                path: "../apis/orders-v2.json".into(),
            },
        );
        assert!(invalid_manifest.validate().is_err());

        let directory = tempfile::tempdir().unwrap();
        let manifest_path = directory.path().join("trellis.toml");
        write_manifest(&manifest_path, &first).unwrap();
        assert_eq!(read_manifest(&manifest_path).unwrap(), first);

        let digest = first.digest().unwrap();
        let lock = ProjectLock {
            format: 1,
            manifest_digest: digest.clone(),
            api: vec![
                LockedApi {
                    id: "trellis.jobs@v1".into(),
                    version: "1.0.0".into(),
                    api_digest: digest.clone(),
                    path: "../apis/jobs.json".into(),
                },
                LockedApi {
                    id: "acme.orders@v1".into(),
                    version: "1.4.2".into(),
                    api_digest: digest,
                    path: "../apis/orders.json".into(),
                },
            ],
        };
        let lock_path = directory.path().join("trellis.lock");
        write_lock(&lock_path, &lock).unwrap();
        assert_eq!(read_lock(&lock_path).unwrap().api[0].id, "acme.orders@v1");

        let mut duplicate = lock;
        duplicate.api.push(duplicate.api[0].clone());
        assert!(duplicate.validate().is_err());
        duplicate.api.pop();
        duplicate.api[0].version = "banana".into();
        assert!(duplicate.validate().is_err());
    }
}
