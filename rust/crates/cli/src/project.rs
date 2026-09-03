//! CLI project-file writers and shared manifest model exports.

use std::{fs, path::Path};

use miette::{miette, IntoDiagnostic, Result};

pub use trellis_idl::project::{
    read_lock, read_manifest, ApiDependency, LockedApi, ProjectLock, ProjectManifest,
    RegistryConfig,
};

/// Deterministically write `trellis.toml` when its contents changed.
pub fn write_manifest(path: &Path, manifest: &ProjectManifest) -> Result<()> {
    manifest.validate()?;
    write_atomic_if_changed(
        path,
        toml::to_string(manifest).into_diagnostic()?.as_bytes(),
    )
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
    manifest_source: Option<&[u8]>,
    lock_path: &Path,
    lock: &ProjectLock,
) -> Result<()> {
    manifest.validate()?;
    lock.validate()?;
    let manifest_bytes = manifest_source
        .map(ToOwned::to_owned)
        .unwrap_or(toml::to_string(manifest).into_diagnostic()?.into_bytes());
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
path = "../jobs"

[apis."acme.orders@v1"]
version = "^1.4"
path = "../orders"
"#,
        )
        .unwrap();
        let reordered: ProjectManifest = toml::from_str(
            r#"format=1
[apis."acme.orders@v1"]
path="../orders"
version="^1.4"
[apis."trellis.jobs@v1"]
path="../jobs"
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
            .path = Some("/tmp/orders".into());
        assert!(invalid_manifest.validate().is_err());
        invalid_manifest = first.clone();
        invalid_manifest.apis.insert(
            "acme.orders@v2".into(),
            ApiDependency {
                version: "^2".into(),
                path: Some("../orders-v2".into()),
                registry: None,
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
                    path: Some("../jobs".into()),
                    registry: None,
                    oci_digest: None,
                },
                LockedApi {
                    id: "acme.orders@v1".into(),
                    version: "1.4.2".into(),
                    api_digest: digest,
                    path: Some("../orders".into()),
                    registry: None,
                    oci_digest: None,
                },
            ],
        };
        let lock_path = directory.path().join("trellis.lock");
        write_lock(&lock_path, &lock).unwrap();
        assert_eq!(read_lock(&lock_path).unwrap().api[0].id, "acme.orders@v1");

        let remote: ProjectManifest = toml::from_str(
            r#"
format = 1
default-registry = "qlever"

[registries.qlever]
prefix = "ghcr.io/qlever-llc/trellis-apis"

[apis."acme.orders@v1"]
version = "^1.4"
registry = "qlever"
"#,
        )
        .unwrap();
        remote.validate().unwrap();
        let mut invalid_remote = remote.clone();
        invalid_remote.apis.get_mut("acme.orders@v1").unwrap().path = Some("../orders".into());
        assert!(invalid_remote.validate().is_err());
        ProjectLock {
            format: 1,
            manifest_digest: remote.digest().unwrap(),
            api: vec![LockedApi {
                id: "acme.orders@v1".into(),
                version: "1.4.2".into(),
                api_digest: first.digest().unwrap(),
                path: None,
                registry: Some("qlever".into()),
                oci_digest: Some(
                    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                        .into(),
                ),
            }],
        }
        .validate()
        .unwrap();

        let mut duplicate = lock;
        duplicate.api.push(duplicate.api[0].clone());
        assert!(duplicate.validate().is_err());
        duplicate.api.pop();
        duplicate.api[0].version = "banana".into();
        assert!(duplicate.validate().is_err());
    }
}
