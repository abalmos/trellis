use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TestMatrix {
    pub(crate) cases: Vec<MatrixCase>,
}

impl TestMatrix {
    pub(crate) fn case_by_id(&self, id: &str) -> Option<&MatrixCase> {
        self.cases.iter().find(|case| case.id == id)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct MatrixCase {
    pub(crate) id: String,
    pub(crate) fixture: String,
    pub(crate) title: String,
    pub(crate) coverage: Vec<String>,
    pub(crate) description: String,
    pub(crate) scenario: Scenario,
    pub(crate) completion: MatrixCompletion,
    pub(crate) classification: RuntimeClassification,
    #[serde(default)]
    pub(crate) isolation_reason: Option<String>,
    #[serde(default)]
    pub(crate) pending: Option<PendingCase>,
    #[serde(default)]
    pub(crate) implementations: Option<MatrixImplementations>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MatrixCompletion {
    #[serde(default)]
    pub(crate) typescript: Option<CompletionStatus>,
    #[serde(default)]
    pub(crate) rust: Option<CompletionStatus>,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CompletionStatus {
    Implemented,
    Pending,
    Required,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RuntimeClassification {
    Shared,
    IsolatedProcess,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct PendingCase {
    reason: String,
    owner: String,
    intended_milestone: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MatrixImplementations {
    #[serde(default)]
    pub(crate) typescript: Option<TypeScriptMatrixImplementation>,
    #[serde(default)]
    pub(crate) rust: Option<RustMatrixImplementation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct TypeScriptMatrixImplementation {
    id: String,
    file: String,
    test_name: String,
    runtime: String,
}

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
pub(crate) struct RustMatrixImplementation {
    pub(crate) module: String,
    pub(crate) function: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Scenario {
    participants: Vec<ScenarioParticipant>,
    given: Vec<String>,
    when: Vec<String>,
    then: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioParticipant {
    name: String,
    kind: ScenarioParticipantKind,
    contract: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ScenarioParticipantKind {
    App,
    Agent,
    Service,
    Device,
    Admin,
    ControlPlane,
}

pub(crate) fn load_client_test_matrix() -> Result<TestMatrix, String> {
    load_matrix(
        &repo_root()?.join("integration/client-test-matrix.json"),
        true,
    )
}

pub(crate) fn load_rust_runtime_test_matrix() -> Result<TestMatrix, String> {
    load_matrix(
        &repo_root()?.join("integration/rust-runtime-test-matrix.json"),
        false,
    )
}

pub(crate) fn repo_root() -> Result<PathBuf, String> {
    let mut current = Path::new(env!("CARGO_MANIFEST_DIR"));
    loop {
        if current
            .join("integration/client-test-matrix.json")
            .is_file()
            && current
                .join("integration/rust-runtime-test-matrix.json")
                .is_file()
        {
            return Ok(current.to_path_buf());
        }
        current = current
            .parent()
            .ok_or_else(|| "could not locate Trellis repository root".to_string())?;
    }
}

fn load_matrix(path: &Path, client: bool) -> Result<TestMatrix, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let matrix: TestMatrix = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse {}: {error}", path.display()))?;
    validate_matrix(&matrix, client)?;
    Ok(matrix)
}

fn validate_matrix(matrix: &TestMatrix, client: bool) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let mut rust_tests = BTreeSet::new();
    for case in &matrix.cases {
        non_empty(&case.id, "case id")?;
        non_empty(&case.fixture, &format!("{} fixture", case.id))?;
        if !case.id.starts_with(&format!("{}.", case.fixture)) {
            return Err(format!(
                "matrix case {} must start with fixture prefix {}.",
                case.id, case.fixture
            ));
        }
        if !ids.insert(&case.id) {
            return Err(format!("duplicate matrix case id {}", case.id));
        }
        non_empty(&case.title, &format!("{} title", case.id))?;
        non_empty(&case.description, &format!("{} description", case.id))?;
        non_empty_strings(&case.coverage, &format!("{} coverage", case.id))?;
        validate_scenario(&case.id, &case.scenario)?;
        match case.classification {
            RuntimeClassification::Shared if case.isolation_reason.is_some() => {
                return Err(format!("shared case {} has an isolation reason", case.id));
            }
            RuntimeClassification::IsolatedProcess => non_empty(
                case.isolation_reason.as_deref().unwrap_or_default(),
                &format!("{} isolationReason", case.id),
            )?,
            RuntimeClassification::Shared => {}
        }

        let implementations = case.implementations.as_ref();
        let rust = implementations.and_then(|value| value.rust.as_ref());
        match case.completion.rust {
            Some(CompletionStatus::Implemented) => {
                let implementation = rust.ok_or_else(|| {
                    format!("implemented Rust case {} has no implementation", case.id)
                })?;
                if !rust_tests.insert(implementation.clone()) {
                    return Err(format!(
                        "Rust test {}::{} is registered more than once",
                        implementation.module, implementation.function
                    ));
                }
                if case.pending.is_some() {
                    return Err(format!("implemented case {} has pending metadata", case.id));
                }
            }
            Some(CompletionStatus::Pending | CompletionStatus::Required) => {
                if rust.is_some() {
                    return Err(format!(
                        "incomplete Rust case {} claims an implementation",
                        case.id
                    ));
                }
                validate_pending(case)?;
            }
            None if client => {
                return Err(format!("client case {} has no Rust status", case.id));
            }
            None => return Err(format!("runtime case {} has no Rust status", case.id)),
        }

        if client {
            if case.completion.typescript != Some(CompletionStatus::Implemented) {
                return Err(format!(
                    "client case {} is not implemented in TypeScript",
                    case.id
                ));
            }
            let typescript = implementations
                .and_then(|value| value.typescript.as_ref())
                .ok_or_else(|| {
                    format!("client case {} has no TypeScript implementation", case.id)
                })?;
            non_empty(&typescript.id, &format!("{} TypeScript id", case.id))?;
            if typescript.id != case.id {
                return Err(format!(
                    "client case {} has mismatched TypeScript id {}",
                    case.id, typescript.id
                ));
            }
            non_empty(&typescript.file, &format!("{} TypeScript file", case.id))?;
            non_empty(
                &typescript.test_name,
                &format!("{} TypeScript testName", case.id),
            )?;
            if typescript.runtime != "live-trellis" {
                return Err(format!(
                    "client case {} has unsupported TypeScript runtime {}",
                    case.id, typescript.runtime
                ));
            }
        } else if case.completion.typescript.is_some()
            || implementations
                .and_then(|value| value.typescript.as_ref())
                .is_some()
        {
            return Err(format!(
                "Rust runtime case {} claims a TypeScript implementation",
                case.id
            ));
        }
    }
    Ok(())
}

fn validate_pending(case: &MatrixCase) -> Result<(), String> {
    let pending = case
        .pending
        .as_ref()
        .ok_or_else(|| format!("incomplete case {} has no pending metadata", case.id))?;
    non_empty(&pending.reason, &format!("{} pending reason", case.id))?;
    non_empty(&pending.owner, &format!("{} pending owner", case.id))?;
    non_empty(
        &pending.intended_milestone,
        &format!("{} pending intendedMilestone", case.id),
    )
}

fn validate_scenario(id: &str, scenario: &Scenario) -> Result<(), String> {
    for (index, participant) in scenario.participants.iter().enumerate() {
        non_empty(
            &participant.name,
            &format!("{id} participant {} name", index + 1),
        )?;
        non_empty(
            &participant.contract,
            &format!("{id} participant {} contract", index + 1),
        )?;
        let _ = &participant.kind;
    }
    non_empty_strings(&scenario.given, &format!("{id} given"))?;
    non_empty_strings(&scenario.when, &format!("{id} when"))?;
    non_empty_strings(&scenario.then, &format!("{id} then"))
}

fn non_empty(value: &str, context: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{context} must not be empty"))
    } else {
        Ok(())
    }
}

fn non_empty_strings(values: &[String], context: &str) -> Result<(), String> {
    for (index, value) in values.iter().enumerate() {
        non_empty(value, &format!("{context} item {}", index + 1))?;
    }
    Ok(())
}
