//! Parser and compiler for declarative Trellis IDL projects.

mod ast;
mod compile;
mod lexer;
mod parser;

use ast::{Project, Source};
use miette::{IntoDiagnostic, WrapErr};
use std::{collections::BTreeMap, path::Path};
use trellis_protocol::{ApiArtifact, ParticipantArtifact};

/// A parsed Trellis IDL project whose source model remains private.
#[derive(Debug)]
pub struct ParsedProject(Project);

/// Discover and parse the IDL source in a Trellis project root.
///
/// # Errors
///
/// Returns an error when neither supported source layout exists, both layouts
/// exist, source I/O fails, or the first source syntax error is encountered.
pub fn parse_project(root: &Path) -> miette::Result<ParsedProject> {
    let single = root.join("contract.trellis");
    let directory = root.join("contracts");
    let single_exists = single.is_file();
    let mut paths = if directory.is_dir() {
        let entries = std::fs::read_dir(&directory)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to read {}", directory.display()))?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to read {}", directory.display()))?;
        entries
            .into_iter()
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_file() && path.extension().is_some_and(|value| value == "trellis")
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    if single_exists && !paths.is_empty() {
        return Err(miette::miette!(
            "project {} contains both contract.trellis and contracts/*.trellis",
            root.display()
        ));
    }
    if single_exists {
        paths.push(single);
    }
    if paths.is_empty() {
        return Err(miette::miette!(
            "project {} contains neither contract.trellis nor contracts/*.trellis",
            root.display()
        ));
    }
    paths.sort();
    let sources = paths
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(&path)
                .into_diagnostic()
                .wrap_err_with(|| format!("failed to read {}", path.display()))?;
            Ok(Source { path, text })
        })
        .collect::<miette::Result<Vec<_>>>()?;
    parser::parse(sources).map(ParsedProject)
}

/// Compile and validate every API declared by a parsed project.
///
/// # Errors
///
/// Returns the first source-aware semantic or protocol validation error.
pub fn compile_apis(project: &ParsedProject) -> miette::Result<BTreeMap<String, ApiArtifact>> {
    compile::apis(&project.0)
}

/// Compile and resolve every participant against the supplied API artifacts.
///
/// # Errors
///
/// Returns the first source-aware semantic, protocol validation, or participant
/// resolution error.
pub fn compile_participants(
    project: &ParsedProject,
    apis: &BTreeMap<String, ApiArtifact>,
) -> miette::Result<Vec<ParticipantArtifact>> {
    compile::participants(&project.0, apis)
}

impl ParsedProject {
    /// Return the deterministically ordered IDL source paths.
    pub fn source_paths(&self) -> impl Iterator<Item = &Path> {
        self.0.sources.iter().map(|source| source.path.as_path())
    }
}
