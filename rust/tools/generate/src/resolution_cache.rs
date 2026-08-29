use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::time::UNIX_EPOCH;

use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use trellis_contracts::{canonicalize_json, sha256_base64url, ContractBuilder, ContractKind};

use crate::contract_input::{
    rehydrate_cached_resolution, CachedNativeResolution, ResolvedNativeInput,
};
use crate::discovery::{DiscoveredContractSource, SourceLanguage};

const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CachedContractResolution {
    integrity_digest: String,
    schema_version: u32,
    resolution_fingerprint: String,
    projection_fingerprint: String,
    source_path: PathBuf,
    language: String,
    contract_id: String,
    contract_kind: ContractKind,
    api_digest: String,
    generated_api_digest: String,
    participant_digest: Option<String>,
    participant_id: Option<String>,
    protocol_participant_json: Option<String>,
    referenced_api_digests: BTreeMap<String, String>,
    local_contract_dependencies: Vec<String>,
    local_dependency_digests: BTreeMap<String, String>,
    warnings: Vec<String>,
    input_snapshot: InputSnapshot,
    owner_version: Option<String>,
    resolved_artifact_path: PathBuf,
    resolved_artifact_digest: String,
    resolved_artifact_len: u64,
    resolved_artifact_modified_ns: u128,
}

impl CachedContractResolution {
    fn refresh_integrity(&mut self) {
        self.integrity_digest.clear();
        self.integrity_digest =
            hex_sha256(&serde_json::to_vec(self).expect("cache entry must serialize"));
    }

    fn has_valid_integrity(&self) -> bool {
        let mut expected = self.clone();
        expected.refresh_integrity();
        self.integrity_digest == expected.integrity_digest
    }

    pub(crate) fn contract_id(&self) -> &str {
        &self.contract_id
    }

    pub(crate) fn contract_kind(&self) -> &ContractKind {
        &self.contract_kind
    }

    pub(crate) fn api_digest(&self) -> &str {
        &self.api_digest
    }

    pub(crate) fn generated_api_digest(&self) -> &str {
        &self.generated_api_digest
    }

    pub(crate) fn participant_digest(&self) -> Option<&str> {
        self.participant_digest.as_deref()
    }

    pub(crate) fn participant_id(&self) -> Option<&str> {
        self.participant_id.as_deref()
    }

    pub(crate) fn owner_version(&self) -> Option<&str> {
        self.owner_version.as_deref()
    }

    pub(crate) fn projection_is_current(&self) -> bool {
        self.projection_fingerprint == projection_fingerprint()
    }

    pub(crate) fn protocol_participant_is_fresh(&self, path: Option<&Path>) -> bool {
        match (&self.protocol_participant_json, path) {
            (Some(expected), Some(path)) => {
                fs::read_to_string(path).ok().as_deref() == Some(expected)
            }
            (None, None) => true,
            _ => false,
        }
    }

    pub(crate) fn emit_warnings(&self) {
        for warning in &self.warnings {
            eprintln!("WARNING {warning}");
        }
    }

    pub(crate) fn dependencies(&self) -> &[String] {
        &self.local_contract_dependencies
    }

    pub(crate) fn references_match(&self, current: &BTreeMap<String, String>) -> bool {
        self.referenced_api_digests
            .iter()
            .all(|(id, digest)| current.get(id).is_none_or(|current| current == digest))
            && self
                .local_dependency_digests
                .iter()
                .all(|(id, digest)| current.get(id) == Some(digest))
    }

    pub(crate) fn rehydrate(&self) -> miette::Result<ResolvedNativeInput> {
        let bytes = fs::read(&self.resolved_artifact_path).into_diagnostic()?;
        miette::ensure!(
            hex_sha256(&bytes) == self.resolved_artifact_digest,
            "cached resolution artifact failed integrity validation"
        );
        let cached = serde_json::from_slice::<CachedNativeResolution>(&bytes).into_diagnostic()?;
        let resolved = rehydrate_cached_resolution(&cached)?;
        miette::ensure!(
            resolved.api.digest == self.api_digest
                && resolved
                    .participant
                    .as_ref()
                    .map(|participant| participant.digest.as_str())
                    == self.participant_digest(),
            "cached canonical contract artifacts failed digest validation"
        );
        Ok(resolved)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputSnapshot {
    cacheable: bool,
    files: Vec<InputFileState>,
    directories: Vec<InputDirectoryState>,
    declared_inputs: Vec<DeclaredInputState>,
    environment: BTreeMap<String, Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputFileState {
    path: PathBuf,
    exists: bool,
    len: u64,
    modified_ns: u128,
    digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputDirectoryState {
    path: PathBuf,
    modified_ns: u128,
    entries_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeclaredInputState {
    pattern: String,
    matches: Vec<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
struct PrepareInputConfig {
    #[serde(default)]
    inputs: Vec<String>,
    #[serde(default)]
    env: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum CacheMissReason {
    Missing,
    InvalidSchema,
    InvalidFingerprint,
    InputChanged,
    Corrupt,
}

pub(crate) struct ResolutionCache {
    entries: PathBuf,
}

impl ResolutionCache {
    pub(crate) fn for_contract(contract: &DiscoveredContractSource) -> Self {
        let repository = repository_root(&contract.project_root);
        let base = env::var_os("TRELLIS_PREPARE_CACHE")
            .map(PathBuf::from)
            .or_else(|| {
                env::var_os("XDG_CACHE_HOME")
                    .map(PathBuf::from)
                    .map(|path| path.join("trellis/prepare"))
            })
            .or_else(|| {
                env::var_os("HOME")
                    .map(PathBuf::from)
                    .map(|path| path.join(".cache/trellis/prepare"))
            })
            .unwrap_or_else(|| repository.join("generated/.trellis-prepare-cache"));
        let namespace = sha256_base64url(repository.to_string_lossy().as_ref());
        Self {
            entries: base.join("repositories").join(namespace).join("contracts"),
        }
    }

    pub(crate) fn load(
        &self,
        contract: &DiscoveredContractSource,
    ) -> Result<CachedContractResolution, CacheMissReason> {
        let path = self.entry_path(contract);
        let bytes = fs::read(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                CacheMissReason::Missing
            } else {
                CacheMissReason::Corrupt
            }
        })?;
        let mut cached: CachedContractResolution =
            serde_json::from_slice(&bytes).map_err(|_| CacheMissReason::Corrupt)?;
        if cached.schema_version != SCHEMA_VERSION {
            return Err(CacheMissReason::InvalidSchema);
        }
        if !cached.has_valid_integrity() {
            return Err(CacheMissReason::Corrupt);
        }
        if cached.resolution_fingerprint != resolution_fingerprint() {
            return Err(CacheMissReason::InvalidFingerprint);
        }
        if cached.source_path != contract.source_path
            || cached.language != language_name(contract.language)
        {
            return Err(CacheMissReason::Corrupt);
        }
        let metadata =
            fs::metadata(&cached.resolved_artifact_path).map_err(|_| CacheMissReason::Corrupt)?;
        if metadata.len() != cached.resolved_artifact_len {
            return Err(CacheMissReason::Corrupt);
        }
        let resolved_modified_ns = modified_ns(&metadata);
        if resolved_modified_ns != cached.resolved_artifact_modified_ns {
            let bytes =
                fs::read(&cached.resolved_artifact_path).map_err(|_| CacheMissReason::Corrupt)?;
            if hex_sha256(&bytes) != cached.resolved_artifact_digest {
                return Err(CacheMissReason::Corrupt);
            }
            cached.resolved_artifact_modified_ns = resolved_modified_ns;
            cached.refresh_integrity();
            let _ = self.write_entry(contract, &cached);
        }
        let started = Instant::now();
        let (current, updated, stat_files, hashed_files) =
            snapshot_is_current(&mut cached.input_snapshot);
        crate::timings::input_snapshot(started.elapsed(), stat_files, hashed_files);
        if !current {
            return Err(CacheMissReason::InputChanged);
        }
        if updated {
            cached.refresh_integrity();
            let _ = self.write_entry(contract, &cached);
        }
        Ok(cached)
    }

    pub(crate) fn previous_participant_id(
        &self,
        contract: &DiscoveredContractSource,
    ) -> Option<String> {
        let bytes = fs::read(self.entry_path(contract)).ok()?;
        let entry: CachedContractResolution = serde_json::from_slice(&bytes).ok()?;
        (entry.source_path == contract.source_path && entry.has_valid_integrity())
            .then_some(entry.participant_id)
            .flatten()
    }

    pub(crate) fn store(
        &self,
        contract: &DiscoveredContractSource,
        resolved: &ResolvedNativeInput,
        contract_kind: &ContractKind,
        local_contract_dependencies: Vec<String>,
        current_api_digests: &BTreeMap<String, String>,
    ) -> miette::Result<CachedContractResolution> {
        let local_dependency_digests = local_contract_dependencies
            .iter()
            .filter_map(|id| {
                current_api_digests
                    .get(id)
                    .map(|digest| (id.clone(), digest.clone()))
            })
            .collect();
        let resolved_artifact = CachedNativeResolution::from_resolved(resolved);
        let resolved_artifact_bytes = serde_json::to_vec(&resolved_artifact).into_diagnostic()?;
        let resolved_artifact_path = self.resolved_path(contract);
        self.write_bytes(&resolved_artifact_path, &resolved_artifact_bytes)?;
        let resolved_artifact_metadata = fs::metadata(&resolved_artifact_path).into_diagnostic()?;
        let mut cached = CachedContractResolution {
            integrity_digest: String::new(),
            schema_version: SCHEMA_VERSION,
            resolution_fingerprint: resolution_fingerprint().to_string(),
            projection_fingerprint: projection_fingerprint().to_string(),
            source_path: contract.source_path.clone(),
            language: language_name(contract.language).to_string(),
            contract_id: resolved.api.render_model.id.clone(),
            contract_kind: contract_kind.clone(),
            api_digest: resolved.api.digest.clone(),
            generated_api_digest: crate::artifacts::native_api_digest(resolved)?,
            participant_digest: resolved
                .participant
                .as_ref()
                .map(|participant| participant.digest.clone()),
            participant_id: resolved
                .participant
                .as_ref()
                .map(|participant| participant.participant.id().to_string()),
            protocol_participant_json: protocol_participant_json(resolved)?,
            referenced_api_digests: resolved
                .referenced_apis
                .iter()
                .map(|api| (api.render_model.id.clone(), api.digest.clone()))
                .collect(),
            local_contract_dependencies,
            local_dependency_digests,
            warnings: crate::contract_input::forward_incompatible_public_schema_warnings(
                &resolved.api,
            ),
            input_snapshot: snapshot(contract)?,
            owner_version: resolved.owner_version.clone(),
            resolved_artifact_path,
            resolved_artifact_digest: hex_sha256(&resolved_artifact_bytes),
            resolved_artifact_len: resolved_artifact_metadata.len(),
            resolved_artifact_modified_ns: modified_ns(&resolved_artifact_metadata),
        };
        cached.refresh_integrity();
        self.write_entry(contract, &cached)?;
        Ok(cached)
    }

    pub(crate) fn update_dependencies(
        &self,
        contract: &DiscoveredContractSource,
        dependencies: Vec<String>,
        api_digests: &BTreeMap<String, String>,
    ) -> Option<CachedContractResolution> {
        let mut cached: CachedContractResolution =
            serde_json::from_slice(&fs::read(self.entry_path(contract)).ok()?).ok()?;
        if cached.schema_version != SCHEMA_VERSION
            || !cached.has_valid_integrity()
            || cached.resolution_fingerprint != resolution_fingerprint()
            || cached.source_path != contract.source_path
        {
            return None;
        }
        let mut dependency_digests = cached.local_dependency_digests.clone();
        dependency_digests.retain(|id, _| dependencies.contains(id));
        for id in &dependencies {
            if let Some(digest) = api_digests.get(id) {
                dependency_digests
                    .entry(id.clone())
                    .or_insert_with(|| digest.clone());
            }
        }
        if cached.local_contract_dependencies != dependencies
            || cached.local_dependency_digests != dependency_digests
        {
            cached.local_contract_dependencies = dependencies;
            cached.local_dependency_digests = dependency_digests;
            cached.refresh_integrity();
            self.write_entry(contract, &cached).ok()?;
        }
        Some(cached)
    }

    fn write_entry(
        &self,
        contract: &DiscoveredContractSource,
        cached: &CachedContractResolution,
    ) -> miette::Result<()> {
        self.write_bytes(
            &self.entry_path(contract),
            &serde_json::to_vec(cached).into_diagnostic()?,
        )?;
        Ok(())
    }

    fn write_bytes(&self, path: &Path, contents: &[u8]) -> miette::Result<()> {
        fs::create_dir_all(&self.entries).into_diagnostic()?;
        let mut temporary = NamedTempFile::new_in(&self.entries).into_diagnostic()?;
        temporary.write_all(contents).into_diagnostic()?;
        temporary.flush().into_diagnostic()?;
        temporary
            .persist(path)
            .map_err(|error| miette::miette!(error.error))?;
        Ok(())
    }

    fn entry_path(&self, contract: &DiscoveredContractSource) -> PathBuf {
        self.entries.join(format!(
            "{}.json",
            sha256_base64url(contract.source_path.to_string_lossy().as_ref())
        ))
    }

    fn resolved_path(&self, contract: &DiscoveredContractSource) -> PathBuf {
        self.entry_path(contract).with_extension("resolved.json")
    }
}

fn protocol_participant_json(resolved: &ResolvedNativeInput) -> miette::Result<Option<String>> {
    let Some(participant) = resolved.participant.as_ref() else {
        return Ok(None);
    };
    let referenced_apis = resolved
        .referenced_apis
        .iter()
        .map(|api| (api.render_model.id.clone(), api.value.clone()))
        .collect();
    let artifacts =
        ContractBuilder::from_native(resolved.api.value.clone(), participant.value.clone())
            .referenced_apis(referenced_apis)
            .build()
            .into_diagnostic()?;
    Ok(Some(format!(
        "{}\n",
        canonicalize_json(&artifacts.participant_value().into_diagnostic()?).into_diagnostic()?
    )))
}

fn resolution_fingerprint() -> &'static str {
    concat!(env!("TRELLIS_MODEL_FINGERPRINT"), ":cache-v1")
}

fn projection_fingerprint() -> &'static str {
    concat!(
        env!("TRELLIS_TS_CODEGEN_FINGERPRINT"),
        ":",
        env!("TRELLIS_NPM_PACKAGING_FINGERPRINT"),
        ":",
        env!("TRELLIS_RUST_CODEGEN_FINGERPRINT")
    )
}

fn repository_root(project_root: &Path) -> PathBuf {
    project_root
        .ancestors()
        .find(|path| path.join(".git").exists())
        .unwrap_or(project_root)
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf())
}

fn language_name(language: SourceLanguage) -> &'static str {
    match language {
        SourceLanguage::Protocol => "protocol",
        SourceLanguage::TypeScript => "typescript",
        SourceLanguage::Rust => "rust",
    }
}

fn snapshot(contract: &DiscoveredContractSource) -> miette::Result<InputSnapshot> {
    let mut paths = BTreeSet::new();
    let mut directories = BTreeSet::new();
    paths.insert(contract.source_path.clone());
    paths.insert(contract.manifest_path.clone());
    add_ancestor_inputs(contract, &mut paths);
    let cacheable = match contract.language {
        SourceLanguage::Protocol => {
            paths.insert(
                contract
                    .source_path
                    .with_file_name("trellis.participant.json"),
            );
            true
        }
        SourceLanguage::Rust => {
            paths.insert(crate::contract_input::resolve_rust_contract_source_path(
                &contract.source_path,
                "API",
            )?);
            add_rust_include_inputs(&mut paths)?
        }
        SourceLanguage::TypeScript => add_typescript_inputs(
            &contract.source_path,
            &contract.project_root,
            &mut paths,
            &mut directories,
        )?,
    };
    let config = prepare_input_config(contract)?;
    let declared_inputs =
        snapshot_declared_inputs(&contract.project_root, &config.inputs, &mut paths)?;
    let mut environment: BTreeMap<String, Option<String>> = config
        .env
        .into_iter()
        .map(|name| {
            let value = env::var(&name).ok();
            (name, value)
        })
        .collect();
    add_tool_inputs(contract, &mut paths, &mut environment);
    Ok(InputSnapshot {
        cacheable,
        files: paths
            .into_iter()
            .map(input_file_state)
            .collect::<miette::Result<_>>()?,
        directories: directories
            .into_iter()
            .map(input_directory_state)
            .collect::<miette::Result<_>>()?,
        declared_inputs,
        environment,
    })
}

fn add_rust_include_inputs(paths: &mut BTreeSet<PathBuf>) -> miette::Result<bool> {
    struct IncludeVisitor<'a> {
        source: &'a Path,
        includes: Vec<PathBuf>,
        cacheable: bool,
    }

    impl syn::visit::Visit<'_> for IncludeVisitor<'_> {
        fn visit_macro(&mut self, node: &syn::Macro) {
            if node.path.is_ident("include")
                || node.path.is_ident("include_str")
                || node.path.is_ident("include_bytes")
            {
                match syn::parse2::<syn::LitStr>(node.tokens.clone()) {
                    Ok(literal) => self.includes.push(
                        self.source
                            .parent()
                            .unwrap_or(Path::new("."))
                            .join(literal.value()),
                    ),
                    Err(_) => {
                        let tokens = node
                            .tokens
                            .to_string()
                            .chars()
                            .filter(|character| !character.is_whitespace())
                            .collect::<String>();
                        if !tokens.starts_with("concat!(env!(\"OUT_DIR\"),") {
                            self.cacheable = false;
                        }
                    }
                }
            }
            syn::visit::visit_macro(self, node);
        }

        fn visit_item_mod(&mut self, node: &syn::ItemMod) {
            if node.content.is_none() {
                let parent = self.source.parent().unwrap_or(Path::new("."));
                if let Some(path) = node
                    .attrs
                    .iter()
                    .find(|attribute| attribute.path().is_ident("path"))
                    .and_then(|attribute| attribute.parse_args::<syn::LitStr>().ok())
                {
                    self.includes.push(parent.join(path.value()));
                } else {
                    let name = node.ident.to_string();
                    for candidate in [
                        parent.join(format!("{name}.rs")),
                        parent.join(name).join("mod.rs"),
                    ] {
                        if candidate.is_file() {
                            self.includes.push(candidate);
                            break;
                        }
                    }
                }
            }
            syn::visit::visit_item_mod(self, node);
        }
    }

    let mut rust_sources = paths
        .iter()
        .filter(|path| {
            path.is_file() && path.extension().is_some_and(|extension| extension == "rs")
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    let mut cacheable = true;
    while let Some(source) = rust_sources.pop() {
        if !visited.insert(source.clone()) {
            continue;
        }
        let contents = fs::read_to_string(&source).into_diagnostic()?;
        let Ok(file) = syn::parse_file(&contents) else {
            cacheable = false;
            continue;
        };
        let mut visitor = IncludeVisitor {
            source: &source,
            includes: Vec::new(),
            cacheable: true,
        };
        syn::visit::Visit::visit_file(&mut visitor, &file);
        cacheable &= visitor.cacheable;
        for include in visitor.includes {
            if include
                .extension()
                .is_some_and(|extension| extension == "rs")
            {
                rust_sources.push(include.clone());
            }
            paths.insert(include);
        }
    }
    Ok(cacheable)
}

fn prepare_input_config(contract: &DiscoveredContractSource) -> miette::Result<PrepareInputConfig> {
    match contract.language {
        SourceLanguage::Rust => {
            let contents = fs::read_to_string(&contract.manifest_path).into_diagnostic()?;
            let manifest: toml::Value = toml::from_str(&contents).into_diagnostic()?;
            manifest
                .get("package")
                .and_then(|value| value.get("metadata"))
                .and_then(|value| value.get("trellis"))
                .and_then(|value| value.get("prepare"))
                .cloned()
                .map(toml::Value::try_into)
                .transpose()
                .into_diagnostic()
                .map(Option::unwrap_or_default)
        }
        SourceLanguage::TypeScript => {
            let Some(path) = contract
                .source_path
                .ancestors()
                .map(|directory| directory.join("package.json"))
                .find(|path| path.is_file())
            else {
                return Ok(PrepareInputConfig::default());
            };
            let value: serde_json::Value =
                serde_json::from_slice(&fs::read(path).into_diagnostic()?).into_diagnostic()?;
            value
                .get("trellis")
                .and_then(|value| value.get("prepare"))
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .into_diagnostic()
                .map(Option::unwrap_or_default)
        }
        SourceLanguage::Protocol => Ok(PrepareInputConfig::default()),
    }
}

fn snapshot_declared_inputs(
    project_root: &Path,
    patterns: &[String],
    paths: &mut BTreeSet<PathBuf>,
) -> miette::Result<Vec<DeclaredInputState>> {
    patterns
        .iter()
        .map(|pattern| {
            let pattern = if Path::new(pattern).is_absolute() {
                pattern.clone()
            } else {
                format!(
                    "{}/{}",
                    glob::Pattern::escape(&project_root.to_string_lossy()),
                    pattern
                )
            };
            let matches = declared_matches(&pattern)?;
            paths.extend(matches.iter().cloned());
            Ok(DeclaredInputState { pattern, matches })
        })
        .collect()
}

fn declared_matches(pattern: &str) -> miette::Result<Vec<PathBuf>> {
    let mut matches = glob::glob(pattern)
        .into_diagnostic()?
        .filter_map(Result::ok)
        .map(|path| path.canonicalize().unwrap_or(path))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    matches.sort();
    matches.dedup();
    Ok(matches)
}

fn add_ancestor_inputs(contract: &DiscoveredContractSource, paths: &mut BTreeSet<PathBuf>) {
    let repository = repository_root(&contract.project_root);
    let project_root = contract
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| contract.project_root.clone());
    for directory in project_root.ancestors() {
        for name in [
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain",
            "rust-toolchain.toml",
            "deno.json",
            "deno.jsonc",
            "deno.lock",
            "package.json",
            "package-lock.json",
            "pnpm-lock.yaml",
            "yarn.lock",
            "bun.lock",
            "bun.lockb",
        ] {
            paths.insert(directory.join(name));
        }
        if directory == repository {
            break;
        }
    }
}

fn add_tool_inputs(
    contract: &DiscoveredContractSource,
    paths: &mut BTreeSet<PathBuf>,
    environment: &mut BTreeMap<String, Option<String>>,
) {
    environment
        .entry("PATH".to_string())
        .or_insert_with(|| env::var("PATH").ok());
    let variables: &[(&str, &str)] = match contract.language {
        SourceLanguage::Rust => &[("RUSTC", "rustc")],
        SourceLanguage::TypeScript => &[
            ("TRELLIS_DENO_BIN", "deno"),
            ("TRELLIS_TSX_BIN", "tsx"),
            ("TRELLIS_NODE_BIN", "node"),
        ],
        SourceLanguage::Protocol => &[],
    };
    for (name, fallback) in variables {
        let configured = env::var(name).ok();
        environment.insert((*name).to_string(), configured.clone());
        if let Some(path) = executable_path(configured.as_deref().unwrap_or(fallback)) {
            paths.insert(path);
        }
    }
    if contract.language == SourceLanguage::TypeScript {
        for directory in contract.source_path.ancestors() {
            let candidate = directory.join("node_modules/.bin/tsx");
            if candidate.is_file() {
                paths.insert(candidate);
                break;
            }
        }
    }
    if contract.language == SourceLanguage::Rust {
        if let Some(path) = executable_path("cargo") {
            paths.insert(path);
        }
        for name in ["RUSTUP_TOOLCHAIN", "CARGO_HOME", "RUSTUP_HOME"] {
            environment.insert(name.to_string(), env::var(name).ok());
        }
    }
}

fn executable_path(binary: &str) -> Option<PathBuf> {
    let path = Path::new(binary);
    if path.components().count() > 1 {
        return path.is_file().then(|| path.to_path_buf());
    }
    env::split_paths(&env::var_os("PATH")?).find_map(|directory| {
        let path = directory.join(binary);
        path.is_file().then_some(path)
    })
}

fn add_typescript_inputs(
    source: &Path,
    project_root: &Path,
    paths: &mut BTreeSet<PathBuf>,
    directories: &mut BTreeSet<PathBuf>,
) -> miette::Result<bool> {
    let mut pending = vec![source.canonicalize().into_diagnostic()?];
    let mut visited = BTreeSet::new();
    let aliases = typescript_import_aliases(source)?;
    let mut conservative = aliases.is_none();
    while let Some(path) = pending.pop() {
        if !visited.insert(path.clone()) {
            continue;
        }
        paths.insert(path.clone());
        let contents = fs::read_to_string(&path).into_diagnostic()?;
        let dependencies = trellis_codegen_ts::typescript_module_dependencies(&contents);
        conservative |= dependencies.has_computed_dynamic_import || dependencies.has_parse_errors;
        for specifier in dependencies.specifiers {
            if let Some(resolved) = resolve_local_typescript_specifier(
                &path,
                &specifier,
                aliases.as_deref().unwrap_or_default(),
                paths,
            ) {
                if resolved.is_file()
                    && !is_generated_sdk_path(&resolved)
                    && !visited.contains(&resolved)
                {
                    pending.push(resolved);
                }
            } else if !is_known_external_typescript_specifier(&specifier) {
                conservative = true;
            }
        }
    }
    if conservative {
        collect_files(project_root, paths, directories, &|path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("ts" | "tsx" | "js" | "jsx" | "mts" | "mjs" | "cts" | "cjs" | "json")
            )
        })?;
    }
    Ok(true)
}

fn typescript_import_aliases(source: &Path) -> miette::Result<Option<Vec<(String, String)>>> {
    let config = source
        .ancestors()
        .flat_map(|directory| [directory.join("deno.json"), directory.join("deno.jsonc")])
        .find(|path| path.is_file());
    let Some(config) = config else {
        return Ok(Some(Vec::new()));
    };
    if config.extension().and_then(|extension| extension.to_str()) == Some("jsonc") {
        return Ok(None);
    }
    let Ok(value) =
        serde_json::from_slice::<serde_json::Value>(&fs::read(&config).into_diagnostic()?)
    else {
        return Ok(None);
    };
    let config_root = config.parent().unwrap_or(Path::new("."));
    let mut aliases = value
        .get("imports")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flat_map(|imports| imports.iter())
        .filter_map(|(specifier, target)| {
            target.as_str().map(|target| {
                let target = if target.starts_with("./") || target.starts_with("../") {
                    config_root.join(target).to_string_lossy().into_owned()
                } else {
                    target.to_string()
                };
                (specifier.clone(), target)
            })
        })
        .collect::<Vec<_>>();
    aliases.sort_by_key(|(specifier, _)| std::cmp::Reverse(specifier.len()));
    Ok(Some(aliases))
}

fn is_known_external_typescript_specifier(specifier: &str) -> bool {
    specifier.starts_with("npm:")
        || specifier.starts_with("jsr:")
        || specifier.starts_with("node:")
        || specifier.starts_with("http://")
        || specifier.starts_with("https://")
        || specifier.starts_with("@trellis-sdk/")
        || specifier.starts_with("@qlever-llc/trellis")
}

fn resolve_local_typescript_specifier(
    source: &Path,
    specifier: &str,
    aliases: &[(String, String)],
    paths: &mut BTreeSet<PathBuf>,
) -> Option<PathBuf> {
    let aliased = aliases.iter().find_map(|(prefix, target)| {
        if specifier == prefix {
            Some(target.clone())
        } else {
            prefix
                .ends_with('/')
                .then(|| specifier.strip_prefix(prefix))
                .flatten()
                .map(|suffix| format!("{target}{suffix}"))
        }
    });
    let specifier = aliased.as_deref().unwrap_or(specifier);
    let relative = Path::new(specifier)
        .is_absolute()
        .then(|| PathBuf::from(specifier))
        .or_else(|| specifier.strip_prefix("file://").map(PathBuf::from))
        .or_else(|| {
            (specifier.starts_with("./") || specifier.starts_with("../"))
                .then(|| source.parent().unwrap_or(Path::new(".")).join(specifier))
        })?;
    if relative.extension().is_some() {
        let relative = relative.canonicalize().unwrap_or(relative);
        paths.insert(relative.clone());
        return Some(relative);
    }
    let mut candidates = ["ts", "tsx", "js", "jsx", "mts", "mjs", "cts", "cjs"]
        .into_iter()
        .map(|extension| relative.with_extension(extension))
        .collect::<Vec<_>>();
    candidates.extend(
        ["ts", "tsx", "js", "jsx", "mts", "mjs", "cts", "cjs"]
            .into_iter()
            .map(|extension| relative.join(format!("index.{extension}"))),
    );
    let resolved = candidates
        .iter()
        .find(|candidate| candidate.is_file())
        .and_then(|candidate| candidate.canonicalize().ok());
    paths.extend(
        candidates
            .into_iter()
            .map(|candidate| candidate.canonicalize().unwrap_or(candidate)),
    );
    resolved
}

fn is_generated_sdk_path(path: &Path) -> bool {
    let mut components = path
        .components()
        .filter_map(|component| component.as_os_str().to_str());
    while let Some(component) = components.next() {
        if component == "generated"
            || (component == "sdk" && components.next() == Some("_generated"))
        {
            return true;
        }
    }
    false
}

fn collect_files(
    root: &Path,
    paths: &mut BTreeSet<PathBuf>,
    directories: &mut BTreeSet<PathBuf>,
    include: &impl Fn(&Path) -> bool,
) -> miette::Result<()> {
    directories.insert(root.to_path_buf());
    for entry in fs::read_dir(root).into_diagnostic()? {
        let path = entry.into_diagnostic()?.path();
        if path.is_dir() {
            if !matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some(".git" | ".worktrees" | "generated" | "node_modules" | "target")
            ) {
                collect_files(&path, paths, directories, include)?;
            }
        } else if include(&path) {
            paths.insert(path);
        }
    }
    Ok(())
}

fn input_file_state(path: PathBuf) -> miette::Result<InputFileState> {
    let Ok(metadata) = fs::metadata(&path) else {
        return Ok(InputFileState {
            path,
            exists: false,
            len: 0,
            modified_ns: 0,
            digest: String::new(),
        });
    };
    let contents = fs::read(&path).into_diagnostic()?;
    Ok(InputFileState {
        path,
        exists: true,
        len: metadata.len(),
        modified_ns: modified_ns(&metadata),
        digest: hex_sha256(&contents),
    })
}

fn input_directory_state(path: PathBuf) -> miette::Result<InputDirectoryState> {
    let metadata = fs::metadata(&path).into_diagnostic()?;
    Ok(InputDirectoryState {
        path: path.clone(),
        modified_ns: modified_ns(&metadata),
        entries_digest: directory_entries_digest(&path)?,
    })
}

fn directory_entries_digest(path: &Path) -> miette::Result<String> {
    let mut entries = fs::read_dir(path)
        .into_diagnostic()?
        .filter_map(Result::ok)
        .map(|entry| {
            let kind = entry
                .file_type()
                .map(|kind| {
                    if kind.is_dir() {
                        'd'
                    } else if kind.is_file() {
                        'f'
                    } else {
                        'o'
                    }
                })
                .unwrap_or('o');
            format!("{kind}:{}", entry.file_name().to_string_lossy())
        })
        .collect::<Vec<_>>();
    entries.sort();
    Ok(hex_sha256(entries.join("\n").as_bytes()))
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut digest, byte| {
            write!(digest, "{byte:02x}").expect("writing to a string cannot fail");
            digest
        })
}

fn snapshot_is_current(snapshot: &mut InputSnapshot) -> (bool, bool, usize, usize) {
    if !snapshot.cacheable {
        return (false, false, 0, 0);
    }
    let mut hashed = 0;
    let mut updated = false;
    if snapshot
        .environment
        .iter()
        .any(|(name, value)| env::var(name).ok().as_ref() != value.as_ref())
        || snapshot.declared_inputs.iter().any(|input| {
            declared_matches(&input.pattern)
                .map(|matches| matches != input.matches)
                .unwrap_or(true)
        })
    {
        return (false, false, 0, 0);
    }
    for (index, cached) in snapshot.files.iter_mut().enumerate() {
        let Ok(metadata) = fs::metadata(&cached.path) else {
            if cached.exists {
                return (false, updated, index + 1, hashed);
            }
            continue;
        };
        if !cached.exists {
            return (false, updated, index + 1, hashed);
        }
        if metadata.len() == cached.len && modified_ns(&metadata) == cached.modified_ns {
            continue;
        }
        hashed += 1;
        let Ok(contents) = fs::read(&cached.path) else {
            return (false, updated, index + 1, hashed);
        };
        if hex_sha256(&contents) != cached.digest {
            return (false, updated, index + 1, hashed);
        }
        cached.len = metadata.len();
        cached.modified_ns = modified_ns(&metadata);
        updated = true;
    }
    for cached in &mut snapshot.directories {
        let Ok(metadata) = fs::metadata(&cached.path) else {
            return (false, updated, snapshot.files.len(), hashed);
        };
        let modified_ns = modified_ns(&metadata);
        if modified_ns == cached.modified_ns {
            continue;
        }
        let Ok(entries_digest) = directory_entries_digest(&cached.path) else {
            return (false, updated, snapshot.files.len(), hashed);
        };
        if entries_digest != cached.entries_digest {
            return (false, updated, snapshot.files.len(), hashed);
        }
        cached.modified_ns = modified_ns;
        updated = true;
    }
    (
        true,
        updated,
        snapshot.files.len() + snapshot.directories.len(),
        hashed,
    )
}

fn modified_ns(metadata: &fs::Metadata) -> u128 {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_nanos())
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;

    fn rust_contract(root: &Path) -> DiscoveredContractSource {
        DiscoveredContractSource {
            project_root: root.to_path_buf(),
            manifest_path: root.join("Cargo.toml"),
            source_path: root.join("src/contract.rs"),
            language: SourceLanguage::Rust,
        }
    }

    fn typescript_contract(root: &Path) -> DiscoveredContractSource {
        DiscoveredContractSource {
            project_root: root.to_path_buf(),
            manifest_path: root.join("deno.json"),
            source_path: root.join("contract.ts"),
            language: SourceLanguage::TypeScript,
        }
    }

    fn fixture() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("src")).unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            root.path().join("src/contract.rs"),
            "pub fn api_artifact() {}\npub const VALUE: u8 = 1;\n",
        )
        .unwrap();
        root
    }

    #[test]
    fn unchanged_snapshot_reuses_file_digests() {
        let root = fixture();
        let mut snapshot = snapshot(&rust_contract(root.path())).unwrap();

        let (current, updated, _, hashed) = snapshot_is_current(&mut snapshot);

        assert!(current);
        assert!(!updated);
        assert_eq!(hashed, 0);
    }

    #[test]
    fn touched_unchanged_file_refreshes_metadata_without_invalidation() {
        let root = fixture();
        let contract = rust_contract(root.path());
        let mut snapshot = snapshot(&contract).unwrap();
        thread::sleep(Duration::from_millis(2));
        let contents = fs::read(&contract.source_path).unwrap();
        fs::write(&contract.source_path, contents).unwrap();

        let (current, updated, _, hashed) = snapshot_is_current(&mut snapshot);

        assert!(current);
        assert!(updated);
        assert_eq!(hashed, 1);
    }

    #[test]
    fn changed_file_invalidates_snapshot() {
        let root = fixture();
        let contract = rust_contract(root.path());
        let mut snapshot = snapshot(&contract).unwrap();
        fs::write(&contract.source_path, "pub const VALUE: u8 = 2;\n").unwrap();

        assert!(!snapshot_is_current(&mut snapshot).0);
    }

    #[test]
    fn rust_literal_include_changes_invalidate_snapshot() {
        let root = fixture();
        let contract = rust_contract(root.path());
        fs::write(
            &contract.source_path,
            "pub fn api_artifact() {}\nconst API: &str = include_str!(\"api.json\");\n",
        )
        .unwrap();
        fs::write(root.path().join("src/api.json"), "{\"value\":1}\n").unwrap();
        let mut snapshot = snapshot(&contract).unwrap();
        fs::write(root.path().join("src/api.json"), "{\"value\":2}\n").unwrap();

        assert!(!snapshot_is_current(&mut snapshot).0);
    }

    #[test]
    fn rust_computed_include_disables_cache() {
        let root = fixture();
        let contract = rust_contract(root.path());
        fs::write(
            &contract.source_path,
            "pub fn api_artifact() {}\nconst API: &str = include_str!(env!(\"API_PATH\"));\n",
        )
        .unwrap();

        assert!(!snapshot(&contract).unwrap().cacheable);
    }

    #[test]
    fn changed_rust_module_invalidates_snapshot() {
        let root = fixture();
        let contract = rust_contract(root.path());
        fs::write(
            &contract.source_path,
            "pub fn api_artifact() {}\nmod added;\n",
        )
        .unwrap();
        fs::write(
            root.path().join("src/added.rs"),
            "pub const ADDED: u8 = 1;\n",
        )
        .unwrap();
        let mut snapshot = snapshot(&contract).unwrap();
        fs::write(
            root.path().join("src/added.rs"),
            "pub const ADDED: u8 = 2;\n",
        )
        .unwrap();

        assert!(!snapshot_is_current(&mut snapshot).0);
    }

    #[test]
    fn added_declared_input_invalidates_snapshot() {
        let root = fixture();
        fs::create_dir(root.path().join("schemas")).unwrap();
        fs::write(root.path().join("schemas/one.json"), "{}\n").unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n\n[package.metadata.trellis.prepare]\ninputs = [\"schemas/*.json\"]\n",
        )
        .unwrap();
        let contract = rust_contract(root.path());
        let mut snapshot = snapshot(&contract).unwrap();
        fs::write(root.path().join("schemas/two.json"), "{}\n").unwrap();

        assert!(!snapshot_is_current(&mut snapshot).0);
    }

    #[test]
    fn declared_environment_change_invalidates_snapshot() {
        let root = fixture();
        let variable = format!("TRELLIS_CACHE_TEST_{}", std::process::id());
        fs::write(
            root.path().join("Cargo.toml"),
            format!(
                "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n\n[package.metadata.trellis.prepare]\nenv = [\"{variable}\"]\n"
            ),
        )
        .unwrap();
        std::env::set_var(&variable, "one");
        let contract = rust_contract(root.path());
        let mut snapshot = snapshot(&contract).unwrap();
        std::env::set_var(&variable, "two");

        assert!(!snapshot_is_current(&mut snapshot).0);
        std::env::remove_var(variable);
    }

    #[test]
    fn corrupt_cache_entry_is_a_miss() {
        let root = fixture();
        let contract = rust_contract(root.path());
        let cache = ResolutionCache {
            entries: root.path().join("cache"),
        };
        fs::create_dir_all(&cache.entries).unwrap();
        fs::write(cache.entry_path(&contract), "not json").unwrap();

        assert!(matches!(
            cache.load(&contract),
            Err(CacheMissReason::Corrupt)
        ));
    }

    #[test]
    fn typescript_import_map_source_changes_invalidate_snapshot() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("src")).unwrap();
        fs::write(
            root.path().join("deno.json"),
            "{\"imports\":{\"#local/\":\"./src/\"}}\n",
        )
        .unwrap();
        fs::write(
            root.path().join("contract.ts"),
            "import '#local/helper.ts';\n",
        )
        .unwrap();
        fs::write(
            root.path().join("src/helper.ts"),
            "export const value = 1;\n",
        )
        .unwrap();
        let mut snapshot = snapshot(&typescript_contract(root.path())).unwrap();
        fs::write(
            root.path().join("src/helper.ts"),
            "export const value = 2;\n",
        )
        .unwrap();

        assert!(!snapshot_is_current(&mut snapshot).0);
    }

    #[test]
    fn computed_import_uses_conservative_project_snapshot() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("deno.json"), "{}\n").unwrap();
        fs::write(
            root.path().join("contract.ts"),
            "await import(`./${name}.ts`);\n",
        )
        .unwrap();
        fs::write(root.path().join("possible.ts"), "export const value = 1;\n").unwrap();
        let mut snapshot = snapshot(&typescript_contract(root.path())).unwrap();
        fs::write(root.path().join("possible.ts"), "export const value = 2;\n").unwrap();

        assert!(!snapshot_is_current(&mut snapshot).0);
    }
}
