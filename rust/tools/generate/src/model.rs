use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeSource {
    Registry,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageTarget {
    Api,
    TypeScript,
    Cargo,
}

#[derive(Debug, Clone)]
pub struct ContractInput {
    pub api: Option<PathBuf>,
    pub participant: Option<PathBuf>,
    pub referenced_api: Vec<PathBuf>,
    pub source: Option<PathBuf>,
    pub image: Option<String>,
    pub source_export: String,
    pub image_api_path: String,
}
