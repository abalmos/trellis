use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::{ContractsError, LoadedApi, LoadedParticipant, API_FORMAT_V1};

/// Load an arbitrary JSON value from disk.
pub fn load_json_value(path: impl AsRef<Path>) -> Result<Value, ContractsError> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

/// Load one strict protocol-owned participant artifact for facade rendering.
pub fn load_participant_source(
    path: impl AsRef<Path>,
) -> Result<LoadedParticipant, ContractsError> {
    let path = path.as_ref();
    let raw_value = load_json_value(path)?;
    trellis_protocol::lint_participant_v1_authoring(&raw_value)?;
    let participant = trellis_protocol::parse_participant_v1(&raw_value)?;
    let value = participant.normalized_value()?;
    let canonical = participant.canonical_json()?;
    let digest = participant.digest()?;
    let manifest = serde_json::from_value(value.clone())?;
    Ok(LoadedParticipant {
        path: path.to_path_buf(),
        participant,
        render_model: manifest,
        value,
        canonical,
        digest,
    })
}

/// Load one strict protocol-owned API artifact for SDK rendering.
pub fn load_sdk_source(path: impl AsRef<Path>) -> Result<LoadedApi, ContractsError> {
    let path = path.as_ref();
    let raw_value = load_json_value(path)?;
    trellis_protocol::lint_api_v1_authoring(&raw_value)?;
    let api = trellis_protocol::parse_api_v1(&raw_value)?;
    let value = api.normalized_value()?;
    let canonical = api.canonical_json()?;
    let digest = api.digest()?;
    let subjects = api.derived_subjects()?;
    let mut render_model: crate::ApiRenderModel = serde_json::from_value(value.clone())?;
    for (name, error) in &mut render_model.errors {
        error.error_type.clone_from(name);
    }
    Ok(LoadedApi {
        path: path.to_path_buf(),
        value,
        api,
        render_model,
        subjects,
        canonical,
        digest,
    })
}

/// Collect native API artifact candidates from one directory.
pub fn source_paths_in_dir(dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, ContractsError> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !(entry.file_type()?.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "json")
            && path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| stem.contains('@')))
        {
            continue;
        }
        let value = load_json_value(&path)?;
        if value.get("format").and_then(Value::as_str) == Some(API_FORMAT_V1) {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}
