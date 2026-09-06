//! Project API dependency and publication commands.

use std::{collections::BTreeMap, fs, path::Path};

use miette::{miette, IntoDiagnostic, Result, WrapErr};
use semver::{Version, VersionReq};
use serde::Serialize;

use crate::{
    cli::{AddArgs, OutputFormat, ProjectRootArgs, PublishArgs, RmArgs},
    oci, output,
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

pub async fn add(format: OutputFormat, args: &AddArgs) -> Result<()> {
    let root = canonical_root(&args.project.root)?;
    let manifest_path = root.join("trellis.toml");
    let previous_manifest = fs::read(&manifest_path).into_diagnostic()?;
    let previous_lock = read_optional(&root.join("trellis.lock"))?;
    let mut manifest = read_manifest(&manifest_path)?;
    let source_path = Path::new(&args.source);
    let (id, release, path, registry) = if root.join(source_path).is_dir() {
        if source_path.is_absolute() {
            return Err(miette!(
                "API paths in trellis.toml must be relative to the project root"
            ));
        }
        let (id, release, _) = read_path_api(&root, "", &args.source)?;
        (id, release, Some(args.source.clone()), None)
    } else {
        trellis_protocol::validate_api_id(&args.source)
            .map_err(|error| miette!("invalid API id '{}': {error}", args.source))?;
        let registry = args
            .registry
            .clone()
            .or_else(|| manifest.default_registry.clone())
            .ok_or_else(|| miette!("remote add requires --registry or default-registry"))?;
        let config = manifest
            .registries
            .get(&registry)
            .ok_or_else(|| miette!("registry '{registry}' is not configured"))?;
        let requirement = args
            .version
            .as_deref()
            .map(VersionReq::parse)
            .transpose()
            .map_err(|error| miette!("invalid version requirement: {error}"))?;
        let release = select_remote_version(config, &args.source, requirement.as_ref()).await?;
        (args.source.clone(), release, None, Some(registry))
    };
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
            path,
            registry,
        },
    );
    let edited_manifest = edit_manifest_api(&previous_manifest, &id, manifest.apis.get(&id))?;
    let lock = resolve_lock(&root, &manifest).await?;
    let result = commit_and_install(
        &root,
        &manifest,
        &lock,
        &previous_manifest,
        previous_lock.as_deref(),
        Some(&edited_manifest),
    )
    .await?;
    print_result(format, &result, Some(format!("Added {id} {requirement}")))
}

pub async fn remove(format: OutputFormat, args: &RmArgs) -> Result<()> {
    let root = canonical_root(&args.project.root)?;
    let previous_manifest = fs::read(root.join("trellis.toml")).into_diagnostic()?;
    let previous_lock = read_optional(&root.join("trellis.lock"))?;
    let mut manifest = read_manifest(&root.join("trellis.toml"))?;
    if manifest.apis.remove(&args.api_id).is_none() {
        return Err(miette!("API '{}' is not in trellis.toml", args.api_id));
    }
    let edited_manifest = edit_manifest_api(&previous_manifest, &args.api_id, None)?;
    let lock = resolve_lock(&root, &manifest).await?;
    let result = commit_and_install(
        &root,
        &manifest,
        &lock,
        &previous_manifest,
        previous_lock.as_deref(),
        Some(&edited_manifest),
    )
    .await?;
    print_result(format, &result, Some(format!("Removed {}", args.api_id)))
}

pub async fn update(format: OutputFormat, args: &ProjectRootArgs) -> Result<()> {
    let root = canonical_root(&args.root)?;
    let manifest = read_manifest(&root.join("trellis.toml"))?;
    let previous_lock = read_optional(&root.join("trellis.lock"))?;
    let lock = resolve_lock(&root, &manifest).await?;
    write_lock(&root.join("trellis.lock"), &lock)?;
    let result = match install_root(&root, &manifest, &lock).await {
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

pub async fn install(format: OutputFormat, args: &ProjectRootArgs) -> Result<()> {
    let root = canonical_root(&args.root)?;
    let manifest = read_manifest(&root.join("trellis.toml"))?;
    let lock = project_lock(&root, &manifest)?;
    let result = install_root(&root, &manifest, &lock).await?;
    print_result(format, &result, None)
}

pub async fn publish(format: OutputFormat, args: &PublishArgs) -> Result<()> {
    let root = canonical_root(&args.project.root)?;
    let manifest = read_manifest(&root.join("trellis.toml"))?;
    let lock = project_lock(&root, &manifest)?;
    let (apis, _) = acquire_dependencies(&root, &manifest, &lock).await?;
    let compiled = trellis_idl::compile_project(&root, apis)?;
    let registry = args
        .registry
        .as_ref()
        .or(manifest.default_registry.as_ref())
        .ok_or_else(|| miette!("publish requires --registry or default-registry"))?;
    let config = manifest
        .registries
        .get(registry)
        .ok_or_else(|| miette!("registry '{registry}' is not configured"))?;
    if compiled.apis.is_empty() {
        return Err(miette!("project has no owned canonical APIs to publish"));
    }
    let mut checked = Vec::new();
    for candidate in compiled.apis.into_values() {
        let (version, existing_digest) = check_publication(config, &candidate).await?;
        let id = candidate.id().to_owned();
        checked.push((candidate, id, version.to_string(), existing_digest));
    }
    let mut published = Vec::new();
    for (candidate, id, version, existing_digest) in checked {
        let (digest, changed) = match existing_digest {
            Some(digest) => (digest, false),
            None => (oci::publish(config, &candidate).await?, true),
        };
        published.push((id, version, digest, changed));
    }
    if output::is_json(format) {
        output::print_json(&published)
    } else {
        for (id, version, digest, changed) in published {
            if changed {
                println!("Published {id} {version}");
            } else {
                println!("{id} {version} already published");
            }
            println!("{}@{digest}", oci::repository(config, &id)?);
        }
        Ok(())
    }
}

async fn check_publication(
    config: &crate::project::RegistryConfig,
    candidate: &trellis_protocol::ApiArtifact,
) -> Result<(Version, Option<String>)> {
    let version = Version::parse(candidate.version()).map_err(|error| miette!(error))?;
    let versions = oci::versions(config, candidate.id()).await?;
    if versions.binary_search(&version).is_ok() {
        let remote = oci::pull_tag(config, candidate.id(), &version).await?;
        if remote.manifest_digest != oci::artifact_digest(candidate)? {
            return Err(miette!(
                "release {} {} already exists with different content",
                candidate.id(),
                version
            ));
        }
        return Ok((version, Some(remote.manifest_digest)));
    }
    if let Some(previous_version) = versions.last() {
        if version <= *previous_version {
            // Releases are monotonic; add historical backfills only when required.
            return Err(miette!(
                "release {version} must be newer than {previous_version}"
            ));
        }
        let previous = oci::pull_tag(config, candidate.id(), previous_version).await?;
        let report = trellis_protocol::compare_api_replacement(&previous.api, candidate)
            .map_err(|error| miette!(error.to_string()))?;
        if !report.compatible {
            let issues = report
                .issues
                .iter()
                .map(|issue| issue.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(miette!("release {} {version} is not compatible with previous release {previous_version}: {issues}; use a new stable API identity such as @v2", candidate.id()));
        }
    }
    Ok((version, None))
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

fn project_lock(root: &Path, manifest: &ProjectManifest) -> Result<ProjectLock> {
    let outputs = crate::generate::output_paths(root, manifest)?;
    let path = root.join("trellis.lock");
    if read_optional(&path)?.is_none() {
        miette::ensure!(
            manifest.apis.is_empty() && outputs.is_empty(),
            "trellis.lock is missing; run `trellis update`"
        );
        return Ok(ProjectLock {
            format: 1,
            manifest_digest: manifest.digest()?,
            api: Vec::new(),
        });
    }
    read_lock(&path)
}

async fn commit_and_install(
    root: &Path,
    manifest: &ProjectManifest,
    lock: &ProjectLock,
    previous_manifest: &[u8],
    previous_lock: Option<&[u8]>,
    manifest_bytes: Option<&[u8]>,
) -> Result<PackageResult> {
    let manifest_path = root.join("trellis.toml");
    let lock_path = root.join("trellis.lock");
    write_manifest_and_lock(&manifest_path, manifest, manifest_bytes, &lock_path, lock)?;
    match install_root(root, manifest, lock).await {
        Ok(result) => Ok(result),
        Err(error) => {
            restore_project_files(&manifest_path, previous_manifest, &lock_path, previous_lock)?;
            Err(error)
        }
    }
}

fn edit_manifest_api(
    previous: &[u8],
    id: &str,
    dependency: Option<&ApiDependency>,
) -> Result<Vec<u8>> {
    let source = std::str::from_utf8(previous).into_diagnostic()?;
    let mut document = source.parse::<toml_edit::DocumentMut>().into_diagnostic()?;
    let apis = document
        .get_mut("apis")
        .and_then(toml_edit::Item::as_table_mut)
        .ok_or_else(|| miette!("trellis.toml must contain an [apis] table"))?;
    match dependency {
        Some(dependency) => {
            apis.insert(
                id,
                toml_edit::ser::to_document(dependency)
                    .into_diagnostic()?
                    .into_item(),
            );
        }
        None => {
            apis.remove(id);
        }
    }
    let bytes = document.to_string().into_bytes();
    let parsed: ProjectManifest = toml::from_slice(&bytes).into_diagnostic()?;
    parsed.validate()?;
    Ok(bytes)
}

fn resolve_path_api(
    root: &Path,
    expected_id: &str,
    dependency: &ApiDependency,
) -> Result<(String, Version, String)> {
    let path = dependency
        .path
        .as_deref()
        .ok_or_else(|| miette!("API '{expected_id}' is not a path dependency"))?;
    let (id, version, digest) = read_path_api(root, expected_id, path)?;
    let requirement = VersionReq::parse(&dependency.version).map_err(|error| miette!(error))?;
    if !requirement.matches(&version) {
        return Err(miette!(
            "{id} release {version} does not satisfy {}",
            dependency.version
        ));
    }
    Ok((id, version, digest))
}

fn read_path_api(
    root: &Path,
    expected_id: &str,
    dependency_path: &str,
) -> Result<(String, Version, String)> {
    let api = compile_path_api(root, expected_id, dependency_path)?;
    let version = Version::parse(api.version())
        .map_err(|error| miette!("invalid release version for '{}': {error}", api.id()))?;
    let digest = api.digest().map_err(|error| miette!(error.to_string()))?;
    Ok((api.id().to_owned(), version, digest))
}

fn compile_path_api(
    root: &Path,
    expected_id: &str,
    dependency_path: &str,
) -> Result<trellis_protocol::ApiArtifact> {
    let path = root.join(dependency_path);
    let project = trellis_idl::parse_project(&path)
        .wrap_err_with(|| format!("failed to parse API project {}", path.display()))?;
    let mut apis = trellis_idl::compile_apis(&project)?;
    let api = if expected_id.is_empty() {
        if apis.len() != 1 {
            return Err(miette!(
                "API project {} must declare exactly one API when added by path",
                path.display()
            ));
        }
        apis.pop_first().expect("one API was checked").1
    } else {
        apis.remove(expected_id).ok_or_else(|| {
            miette!(
                "manifest API '{expected_id}' is not declared by project {}",
                path.display()
            )
        })?
    };
    Ok(api)
}

async fn resolve_lock(root: &Path, manifest: &ProjectManifest) -> Result<ProjectLock> {
    let mut api = Vec::with_capacity(manifest.apis.len());
    for (id, dependency) in &manifest.apis {
        let (version, api_digest, oci_digest) = if let Some(registry) = &dependency.registry {
            let config = manifest
                .registries
                .get(registry)
                .ok_or_else(|| miette!("registry '{registry}' is not configured"))?;
            let requirement =
                VersionReq::parse(&dependency.version).map_err(|error| miette!(error))?;
            let version = select_remote_version(config, id, Some(&requirement)).await?;
            let pulled = oci::pull_tag(config, id, &version).await?;
            (
                version,
                pulled
                    .api
                    .digest()
                    .map_err(|error| miette!(error.to_string()))?,
                Some(pulled.manifest_digest),
            )
        } else {
            let (_, version, digest) = resolve_path_api(root, id, dependency)?;
            (version, digest, None)
        };
        api.push(LockedApi {
            id: id.clone(),
            version: version.to_string(),
            api_digest,
            path: dependency.path.clone(),
            registry: dependency.registry.clone(),
            oci_digest,
        });
    }
    Ok(ProjectLock {
        format: 1,
        manifest_digest: manifest.digest()?,
        api,
    })
}

async fn install_root(
    root: &Path,
    manifest: &ProjectManifest,
    lock: &ProjectLock,
) -> Result<PackageResult> {
    let (apis, changed_dependencies) = acquire_dependencies(root, manifest, lock).await?;
    let compiled = trellis_idl::compile_project(root, apis)?;
    let generated = crate::generate::generate_compiled(root, manifest, &compiled, false)?;
    Ok(PackageResult {
        installed_apis: lock.api.len(),
        changed_dependencies,
        generated_projects: generated,
    })
}

fn validate_lock(manifest: &ProjectManifest, lock: &ProjectLock) -> Result<()> {
    lock.validate()?;
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
        crate::generate::validate_output_identity("API", &locked.id)?;
        let dependency = manifest
            .apis
            .get(&locked.id)
            .ok_or_else(|| miette!("locked API '{}' is absent from trellis.toml", locked.id))?;
        if dependency.path != locked.path || dependency.registry != locked.registry {
            return Err(miette!(
                "locked path for '{}' does not match trellis.toml",
                locked.id
            ));
        }
        if !VersionReq::parse(&dependency.version)
            .into_diagnostic()?
            .matches(&Version::parse(&locked.version).into_diagnostic()?)
        {
            return Err(miette!(
                "locked API '{}' release {} does not satisfy {}; run `trellis update`",
                locked.id,
                locked.version,
                dependency.version
            ));
        }
    }
    Ok(())
}

async fn acquire_dependencies(
    root: &Path,
    manifest: &ProjectManifest,
    lock: &ProjectLock,
) -> Result<(BTreeMap<String, trellis_protocol::ApiArtifact>, usize)> {
    validate_lock(manifest, lock)?;
    let mut apis = path_dependencies(root, manifest, Some(lock))?;
    let mut acquired = 0;
    for locked in lock.api.iter().filter(|api| api.registry.is_some()) {
        let registry = locked
            .registry
            .as_ref()
            .expect("registry dependencies were filtered");
        let config = manifest
            .registries
            .get(registry)
            .ok_or_else(|| miette!("registry '{registry}' is not configured"))?;
        let digest = locked
            .oci_digest
            .as_deref()
            .ok_or_else(|| miette!("locked API '{}' has no OCI digest", locked.id))?;
        let pulled = match oci::read_locked(&locked.id, &locked.version, &locked.api_digest, digest)
        {
            Ok(pulled) => pulled,
            Err(_) => {
                let pulled = oci::pull_locked(
                    config,
                    &locked.id,
                    &locked.version,
                    &locked.api_digest,
                    digest,
                )
                .await?;
                acquired += 1;
                pulled
            }
        };
        apis.insert(locked.id.clone(), pulled.api);
    }
    Ok((apis, acquired))
}

fn path_dependencies(
    root: &Path,
    manifest: &ProjectManifest,
    exact_lock: Option<&ProjectLock>,
) -> Result<BTreeMap<String, trellis_protocol::ApiArtifact>> {
    let mut apis = BTreeMap::new();
    for (id, dependency) in &manifest.apis {
        let Some(path) = &dependency.path else {
            continue;
        };
        let api = compile_path_api(root, id, path)?;
        let version = Version::parse(api.version()).into_diagnostic()?;
        if !VersionReq::parse(&dependency.version)
            .into_diagnostic()?
            .matches(&version)
        {
            return Err(miette!(
                "{id} release {version} does not satisfy {}",
                dependency.version
            ));
        }
        if let Some(lock) = exact_lock {
            let locked = lock
                .api
                .iter()
                .find(|api| &api.id == id)
                .expect("validated lock membership");
            if api.version() != locked.version {
                return Err(miette!(
                    "locked API '{id}' is {}, but path now contains {}; run `trellis update`",
                    locked.version,
                    api.version()
                ));
            }
            if api.digest().into_diagnostic()? != locked.api_digest {
                return Err(miette!("locked API '{id}' digest does not match canonical path artifact; run `trellis update`"));
            }
        }
        apis.insert(id.clone(), api);
    }
    Ok(apis)
}

/// Compile current sources with direct local APIs and exact cached registry releases.
pub fn compile_project(
    root: &Path,
    manifest: &ProjectManifest,
) -> Result<trellis_idl::CompiledProject> {
    let mut apis = path_dependencies(root, manifest, None)?;
    if manifest
        .apis
        .values()
        .any(|dependency| dependency.registry.is_some())
    {
        let lock = read_lock(&root.join("trellis.lock"))
            .wrap_err("registry dependencies require trellis.lock; run `trellis update`")?;
        validate_lock(manifest, &lock)?;
        for locked in lock.api.iter().filter(|api| api.registry.is_some()) {
            let pulled = oci::read_locked(
                &locked.id,
                &locked.version,
                &locked.api_digest,
                locked.oci_digest.as_deref().expect("validated OCI lock"),
            )
            .map_err(|error| {
                miette!(
                    "cached API '{}' {} is unavailable: {error}; run `trellis install`",
                    locked.id,
                    locked.version
                )
            })?;
            apis.insert(locked.id.clone(), pulled.api);
        }
    }
    trellis_idl::compile_project(root, apis)
}

async fn select_remote_version(
    config: &crate::project::RegistryConfig,
    id: &str,
    requirement: Option<&VersionReq>,
) -> Result<Version> {
    select_version(oci::versions(config, id).await?, requirement).ok_or_else(|| {
        miette!(
            "no release of {id} satisfies {}",
            requirement.map_or("any version".to_owned(), ToString::to_string)
        )
    })
}

fn select_version(versions: Vec<Version>, requirement: Option<&VersionReq>) -> Option<Version> {
    versions
        .into_iter()
        .filter(|version| requirement.is_none_or(|requirement| requirement.matches(version)))
        .max()
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
    use std::{collections::BTreeMap, sync::Arc};

    use super::*;
    use axum::{
        body::{to_bytes, Body},
        extract::State,
        http::{header::HOST, Request, Response, StatusCode},
        routing::any,
        Router,
    };
    use registry_testkit::{RegistryConfig as TestRegistryConfig, RegistryServer};

    fn write_test_api(path: &Path, id: &str, version: &str, extra_field: bool) {
        fs::create_dir_all(path).unwrap();
        fs::write(
            path.join("contract.trellis"),
            format!(
                r#"api "{id}" {{
    version "{version}";
    display_name "Fixture API";
    description "Fixture API.";
    model Empty {{}}
    model Session {{ id: string; {} }}
    rpc "Auth.Sessions.Me" {{ version "v1"; input Empty; output Session; }}
}}
"#,
                if extra_field { "extra?: string;" } else { "" }
            ),
        )
        .unwrap();
    }

    fn write_test_project(root: &Path) {
        fs::write(
            root.join("contract.trellis"),
            "api \"test.project@v1\" { version \"1.0.0\"; display_name \"Test Project\"; description \"Package test project.\"; }\n",
        )
        .unwrap();
    }

    #[derive(Clone)]
    struct PagingRegistry {
        backend: String,
        tags: Arc<Vec<String>>,
        client: reqwest::Client,
    }

    async fn paging_registry(
        State(state): State<PagingRegistry>,
        request: Request<Body>,
    ) -> Response<Body> {
        if request.uri().path().ends_with("/tags/list") {
            let last = request
                .uri()
                .query()
                .and_then(|query| query.split('&').find_map(|part| part.strip_prefix("last=")));
            let start = last
                .and_then(|last| state.tags.iter().position(|tag| tag == last))
                .map_or(0, |position| position + 1);
            let tags = state.tags.iter().skip(start).take(1).collect::<Vec<_>>();
            return Response::builder()
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "name": "acme.orders-v1",
                        "tags": tags,
                    }))
                    .unwrap(),
                ))
                .unwrap();
        }

        let (parts, body) = request.into_parts();
        let body = match to_bytes(body, usize::MAX).await {
            Ok(body) => body,
            Err(_) => return Response::new(Body::empty()),
        };
        let mut forwarded = state
            .client
            .request(parts.method, format!("{}{}", state.backend, parts.uri));
        for (name, value) in &parts.headers {
            if name != HOST {
                forwarded = forwarded.header(name, value);
            }
        }
        match forwarded.body(body).send().await {
            Ok(response) => {
                let status = response.status();
                let headers = response.headers().clone();
                match response.bytes().await {
                    Ok(body) => {
                        let mut forwarded = Response::builder().status(status);
                        for (name, value) in headers {
                            if let Some(name) = name {
                                forwarded = forwarded.header(name, value);
                            }
                        }
                        forwarded.body(Body::from(body)).unwrap()
                    }
                    Err(_) => Response::builder()
                        .status(StatusCode::BAD_GATEWAY)
                        .body(Body::empty())
                        .unwrap(),
                }
            }
            Err(_) => Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::empty())
                .unwrap(),
        }
    }

    async fn start_paging_registry(
        backend_port: u16,
        tags: Vec<String>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new()
            .fallback(any(paging_registry))
            .with_state(PagingRegistry {
                backend: format!("http://127.0.0.1:{backend_port}"),
                tags: Arc::new(tags),
                client: reqwest::Client::new(),
            });
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (format!("127.0.0.1:{}", address.port()), task)
    }

    #[tokio::test]
    async fn paginated_releases_drive_update_and_publish_compatibility() {
        let _guard = crate::oci::TEST_ENV_LOCK.lock().await;
        let cache = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("TRELLIS_CACHE", cache.path()) };
        let backend = RegistryServer::new(TestRegistryConfig::memory())
            .await
            .unwrap();
        let tags = ["1.0.0", "1.1.0", "1.2.0"].map(str::to_owned).to_vec();
        let (prefix, proxy) = start_paging_registry(backend.port(), tags).await;
        let registry = crate::project::RegistryConfig { prefix };
        for version in ["1.0.0", "1.1.0"] {
            let api = trellis_protocol::parse_api(&serde_json::json!({
                "format": "trellis.api.v1",
                "id": "acme.orders@v1",
                "version": version,
                "displayName": "Orders",
                "description": "Orders API"
            }))
            .unwrap();
            crate::oci::publish(&registry, &api).await.unwrap();
        }
        let latest = trellis_protocol::parse_api(&serde_json::json!({
            "format": "trellis.api.v1",
            "id": "acme.orders@v1",
            "version": "1.2.0",
            "displayName": "Orders",
            "description": "Orders API",
            "schemas": {
                "Empty": { "type": "object", "properties": {}, "required": [] }
            },
            "rpc": {
                "Orders.Get": {
                    "version": "v1",
                    "input": { "schema": "Empty" },
                    "output": { "schema": "Empty" }
                }
            }
        }))
        .unwrap();
        crate::oci::publish(&registry, &latest).await.unwrap();

        let root = tempfile::tempdir().unwrap();
        write_test_project(root.path());
        crate::project::write_manifest(
            &root.path().join("trellis.toml"),
            &ProjectManifest {
                format: 1,
                name: None,
                generate: None,
                default_registry: Some("local".into()),
                registries: BTreeMap::from([("local".into(), registry.clone())]),
                apis: BTreeMap::from([(
                    "acme.orders@v1".into(),
                    ApiDependency {
                        version: "^1.0".into(),
                        path: None,
                        registry: Some("local".into()),
                    },
                )]),
            },
        )
        .unwrap();
        update(
            OutputFormat::Text,
            &ProjectRootArgs {
                root: root.path().to_path_buf(),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            crate::project::read_lock(&root.path().join("trellis.lock"))
                .unwrap()
                .api[0]
                .version,
            "1.2.0"
        );

        let drift = trellis_protocol::parse_api(&serde_json::json!({
            "format": "trellis.api.v1",
            "id": "acme.orders@v1",
            "version": "1.2.0",
            "displayName": "Changed Orders",
            "description": "Orders API"
        }))
        .unwrap();
        let error = check_publication(&registry, &drift).await.unwrap_err();
        assert!(error
            .to_string()
            .contains("already exists with different content"));

        let older = trellis_protocol::parse_api(&serde_json::json!({
            "format": "trellis.api.v1",
            "id": "acme.orders@v1",
            "version": "1.1.5",
            "displayName": "Orders",
            "description": "Orders API"
        }))
        .unwrap();
        let error = check_publication(&registry, &older).await.unwrap_err();
        assert!(error.to_string().contains("must be newer than 1.2.0"));

        let candidate = trellis_protocol::parse_api(&serde_json::json!({
            "format": "trellis.api.v1",
            "id": "acme.orders@v1",
            "version": "1.3.0",
            "displayName": "Orders",
            "description": "Orders API"
        }))
        .unwrap();
        let error = check_publication(&registry, &candidate).await.unwrap_err();
        assert!(error.to_string().contains("previous release 1.2.0"));
        proxy.abort();
        unsafe { std::env::remove_var("TRELLIS_CACHE") };
    }

    #[test]
    fn remote_version_selection_uses_standard_semver_prerelease_rules() {
        let versions = ["1.4.2", "1.5.0-rc.1", "1.5.0-rc.2"]
            .map(|version| Version::parse(version).unwrap())
            .to_vec();
        assert_eq!(
            select_version(
                versions.clone(),
                Some(&VersionReq::parse("^1.5.0-rc.1").unwrap())
            )
            .unwrap(),
            Version::parse("1.5.0-rc.2").unwrap()
        );
        assert_eq!(
            select_version(versions, Some(&VersionReq::parse("^1.4").unwrap())).unwrap(),
            Version::parse("1.4.2").unwrap()
        );
    }

    #[tokio::test]
    async fn remote_lock_installs_from_oci_and_global_cache() {
        let _guard = crate::oci::TEST_ENV_LOCK.lock().await;
        let root = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("TRELLIS_CACHE", cache.path()) };
        let server = RegistryServer::new(TestRegistryConfig::memory())
            .await
            .unwrap();
        let registry = crate::project::RegistryConfig {
            prefix: format!("127.0.0.1:{}", server.port()),
        };
        let api = trellis_protocol::parse_api(&serde_json::json!({
            "format": "trellis.api.v1",
            "id": "acme.orders@v1",
            "version": "1.4.2",
            "displayName": "Orders",
            "description": "Orders API"
        }))
        .unwrap();
        let api_digest = api.digest().unwrap();
        let oci_digest = crate::oci::publish(&registry, &api).await.unwrap();
        let manifest = ProjectManifest {
            format: 1,
            name: Some("cache-consumer".into()),
            generate: None,
            default_registry: Some("local".into()),
            registries: BTreeMap::from([("local".into(), registry)]),
            apis: BTreeMap::from([(
                api.id().into(),
                ApiDependency {
                    version: "^1.4".into(),
                    path: None,
                    registry: Some("local".into()),
                },
            )]),
        };
        let lock = ProjectLock {
            format: 1,
            manifest_digest: manifest.digest().unwrap(),
            api: vec![LockedApi {
                id: api.id().into(),
                version: api.version().into(),
                api_digest,
                path: None,
                registry: Some("local".into()),
                oci_digest: Some(oci_digest),
            }],
        };
        let write_project = |root: &Path| {
            fs::write(root.join("package.json"), "{}").unwrap();
            fs::write(
                root.join("contract.trellis"),
                "api \"acme.consumer@v1\" { version \"1.0.0\"; display_name \"Consumer\"; description \"Consumer API.\"; }\n",
            )
            .unwrap();
            write_manifest_and_lock(
                &root.join("trellis.toml"),
                &manifest,
                None,
                &root.join("trellis.lock"),
                &lock,
            )
            .unwrap();
        };
        write_project(root.path());

        let missing = crate::generate::generate_project(root.path())
            .unwrap_err()
            .to_string();
        assert!(missing.contains("trellis install"), "{missing}");
        let first = install_root(root.path(), &manifest, &lock).await.unwrap();
        assert_eq!(first.installed_apis, 1);
        let artifact = root
            .path()
            .join("trellis/artifacts/apis/acme.orders@v1.json");
        let previous = fs::read(&artifact).unwrap();
        drop(server);
        let docker_config = tempfile::tempdir().unwrap();
        fs::write(docker_config.path().join("config.json"), "{").unwrap();
        unsafe { std::env::set_var("DOCKER_CONFIG", docker_config.path()) };
        assert_eq!(
            install_root(root.path(), &manifest, &lock)
                .await
                .unwrap()
                .changed_dependencies,
            0
        );
        let second = tempfile::tempdir().unwrap();
        write_project(second.path());
        assert_eq!(
            install_root(second.path(), &manifest, &lock)
                .await
                .unwrap()
                .installed_apis,
            1
        );
        fs::remove_dir_all(root.path().join("trellis")).unwrap();
        let cached = install_root(root.path(), &manifest, &lock).await.unwrap();
        assert_eq!(cached.installed_apis, 1);
        assert_eq!(fs::read(&artifact).unwrap(), previous);
        assert!(!root.path().join(".trellis").exists());
        assert!(!second.path().join(".trellis").exists());
        crate::generate::generate_project(second.path()).unwrap();
        let cached_api = cache
            .path()
            .join("oci/sha256")
            .join(
                lock.api[0]
                    .oci_digest
                    .as_deref()
                    .unwrap()
                    .strip_prefix("sha256:")
                    .unwrap(),
            )
            .join("api.json");
        fs::write(cached_api, "{}").unwrap();
        let corrupt = crate::generate::generate_project(root.path())
            .unwrap_err()
            .to_string();
        assert!(corrupt.contains("digest mismatch"), "{corrupt}");
        assert_eq!(fs::read(&artifact).unwrap(), previous);
        unsafe { std::env::remove_var("DOCKER_CONFIG") };
        unsafe { std::env::remove_var("TRELLIS_CACHE") };
    }

    #[tokio::test]
    async fn install_is_exact_and_preserves_previous_package_on_drift() {
        let root = tempfile::tempdir().unwrap();
        write_test_project(root.path());
        fs::write(root.path().join("deno.json"), "{}").unwrap();
        let api_path = root.path().join("auth");
        write_test_api(&api_path, "trellis.auth@v1", "1.0.0", false);
        let manifest = ProjectManifest {
            format: 1,
            name: Some("local-consumer".into()),
            generate: None,
            default_registry: None,
            registries: BTreeMap::new(),
            apis: BTreeMap::from([(
                "trellis.auth@v1".to_owned(),
                ApiDependency {
                    version: "^1.0".to_owned(),
                    path: Some("auth".to_owned()),
                    registry: None,
                },
            )]),
        };
        let lock = resolve_lock(root.path(), &manifest).await.unwrap();
        crate::project::write_manifest(&root.path().join("trellis.toml"), &manifest).unwrap();
        crate::project::write_lock(&root.path().join("trellis.lock"), &lock).unwrap();
        install_root(root.path(), &manifest, &lock).await.unwrap();
        let generated = root
            .path()
            .join("trellis/artifacts/apis/trellis.auth@v1.json");
        let previous = fs::read(&generated).unwrap();

        write_test_api(&api_path, "trellis.auth@v1", "1.0.1", false);
        let error = install_root(root.path(), &manifest, &lock)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("path now contains 1.0.1"));
        assert_eq!(fs::read(&generated).unwrap(), previous);

        write_test_api(&api_path, "trellis.auth@v1", "1.0.0", true);
        let error = install_root(root.path(), &manifest, &lock)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("digest does not match"));
        assert_eq!(fs::read(generated).unwrap(), previous);
    }

    #[tokio::test]
    async fn add_update_and_remove_share_the_locked_installer() {
        let root = tempfile::tempdir().unwrap();
        write_test_project(root.path());
        fs::write(
            root.path().join("trellis.toml"),
            "# project dependencies\nformat = 1\n\n[apis]\n# keep this explanation\n",
        )
        .unwrap();
        let api_path = root.path().join("auth");
        write_test_api(&api_path, "acme.auth@v1", "1.4.2", false);

        add(
            OutputFormat::Text,
            &AddArgs {
                source: "auth".to_owned(),
                version: None,
                registry: None,
                project: ProjectRootArgs {
                    root: root.path().to_path_buf(),
                },
            },
        )
        .await
        .unwrap();
        let manifest = crate::project::read_manifest(&root.path().join("trellis.toml")).unwrap();
        assert_eq!(manifest.apis["acme.auth@v1"].version, "^1.4.2");
        assert!(fs::read_to_string(root.path().join("trellis.toml"))
            .unwrap()
            .contains("# keep this explanation"));
        let initial_lock = crate::project::read_lock(&root.path().join("trellis.lock")).unwrap();
        assert_eq!(initial_lock.api[0].version, "1.4.2");

        write_test_api(&api_path, "acme.auth@v1", "1.4.3", false);
        let project = ProjectRootArgs {
            root: root.path().to_path_buf(),
        };
        update(OutputFormat::Text, &project).await.unwrap();
        let updated_lock = crate::project::read_lock(&root.path().join("trellis.lock")).unwrap();
        assert_eq!(updated_lock.api[0].version, "1.4.3");
        assert_eq!(
            updated_lock.api[0].api_digest,
            initial_lock.api[0].api_digest
        );
        write_test_api(&api_path, "acme.auth@v1", "2.0.0", false);
        assert!(update(OutputFormat::Text, &project)
            .await
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
        .await
        .unwrap();
        assert!(
            crate::project::read_manifest(&root.path().join("trellis.toml"))
                .unwrap()
                .apis
                .is_empty()
        );
        assert!(fs::read_to_string(root.path().join("trellis.toml"))
            .unwrap()
            .contains("# keep this explanation"));
        assert!(crate::project::read_lock(&root.path().join("trellis.lock"))
            .unwrap()
            .api
            .is_empty());
        assert!(api_path.is_dir());

        let empty_lock = crate::project::read_lock(&root.path().join("trellis.lock")).unwrap();
        fs::remove_file(root.path().join("trellis.lock")).unwrap();
        install(
            OutputFormat::Text,
            &ProjectRootArgs {
                root: root.path().to_path_buf(),
            },
        )
        .await
        .unwrap();
        assert!(!root.path().join("trellis.lock").exists());
        crate::project::write_lock(&root.path().join("trellis.lock"), &empty_lock).unwrap();
        let mut changed_manifest =
            crate::project::read_manifest(&root.path().join("trellis.toml")).unwrap();
        changed_manifest.apis.insert(
            "acme.auth@v1".to_owned(),
            ApiDependency {
                version: "^1.4".to_owned(),
                path: Some("auth".to_owned()),
                registry: None,
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
        .await
        .unwrap_err()
        .to_string()
        .contains("trellis.toml changed since trellis.lock"));
    }

    #[tokio::test]
    async fn add_locks_a_prerelease_path_api() {
        let root = tempfile::tempdir().unwrap();
        write_test_project(root.path());
        crate::project::write_manifest(
            &root.path().join("trellis.toml"),
            &ProjectManifest {
                format: 1,
                name: None,
                generate: None,
                default_registry: None,
                registries: BTreeMap::new(),
                apis: BTreeMap::new(),
            },
        )
        .unwrap();
        write_test_api(
            &root.path().join("orders"),
            "acme.orders@v1",
            "1.5.0-rc.1",
            false,
        );

        add(
            OutputFormat::Text,
            &AddArgs {
                source: "orders".to_owned(),
                version: Some("^1.5.0-rc.1".to_owned()),
                registry: None,
                project: ProjectRootArgs {
                    root: root.path().to_path_buf(),
                },
            },
        )
        .await
        .unwrap();

        let lock = crate::project::read_lock(&root.path().join("trellis.lock")).unwrap();
        assert_eq!(lock.api[0].version, "1.5.0-rc.1");
        install(
            OutputFormat::Text,
            &ProjectRootArgs {
                root: root.path().to_path_buf(),
            },
        )
        .await
        .unwrap();
    }
}
