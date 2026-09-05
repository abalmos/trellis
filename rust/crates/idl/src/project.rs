//! Trellis project API dependency manifest and exact-release lock models.

use std::{collections::BTreeMap, fs, path::Path};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

/// One local project or OCI canonical API dependency.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApiDependency {
    /// Accepted release versions.
    pub version: String,
    /// Relative path to a Trellis project root.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Named OCI registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
}

/// One named OCI registry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryConfig {
    /// Registry host and optional repository namespace.
    pub prefix: String,
}

/// A project's `trellis.toml` dependency model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectManifest {
    /// Manifest format version. Must be `1`.
    pub format: u32,
    /// Registry used when a package command does not name one.
    #[serde(rename = "default-registry", skip_serializing_if = "Option::is_none")]
    pub default_registry: Option<String>,
    /// Named OCI registries.
    #[serde(default)]
    pub registries: BTreeMap<String, RegistryConfig>,
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
        for (name, registry) in &self.registries {
            if name.is_empty() || registry.prefix.is_empty() {
                return Err(miette!("registry names and prefixes must not be empty"));
            }
            if registry
                .prefix
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
            {
                return Err(miette!(
                    "registry '{name}' prefix must not contain whitespace"
                ));
            }
        }
        if let Some(name) = &self.default_registry {
            if !self.registries.contains_key(name) {
                return Err(miette!("default registry '{name}' is not configured"));
            }
        }
        let mut lineages = BTreeMap::new();
        for (id, dependency) in &self.apis {
            trellis_protocol::validate_api_id(id)
                .map_err(|error| miette!("invalid API id '{id}': {error}"))?;
            VersionReq::parse(&dependency.version)
                .map_err(|error| miette!("invalid version requirement for API '{id}': {error}"))?;
            match (&dependency.path, &dependency.registry) {
                (Some(path), None) if !path.is_empty() && !Path::new(path).is_absolute() => {}
                (None, Some(registry)) if self.registries.contains_key(registry) => {}
                (None, Some(registry)) => {
                    return Err(miette!(
                        "registry '{registry}' for API '{id}' is not configured"
                    ));
                }
                _ => {
                    return Err(miette!(
                        "API '{id}' must specify exactly one of path or registry"
                    ));
                }
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
    /// Local project path copied from the manifest dependency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Named OCI registry copied from the manifest dependency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    /// Exact OCI manifest digest for remote dependencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oci_digest: Option<String>,
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
            match (&api.path, &api.registry, &api.oci_digest) {
                (Some(path), None, None) if !path.is_empty() && !Path::new(path).is_absolute() => {}
                (None, Some(registry), Some(digest))
                    if !registry.is_empty()
                        && digest.strip_prefix("sha256:").is_some_and(|hex| {
                            hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
                        }) => {}
                _ => return Err(miette!("locked API '{}' has an invalid source", api.id)),
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

/// Read and validate `trellis.lock`.
pub fn read_lock(path: &Path) -> Result<ProjectLock> {
    let value = toml::from_str::<ProjectLock>(&fs::read_to_string(path).into_diagnostic()?)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to parse {}", path.display()))?;
    value.validate()?;
    Ok(value)
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
