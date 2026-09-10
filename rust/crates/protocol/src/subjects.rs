use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    identifiers::{validate_logical_name, validate_version},
    ProtocolError,
};

/// Subjects derived for a set of communication surfaces.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedApiSubjects {
    /// RPC subjects keyed by logical name.
    pub rpc: BTreeMap<String, String>,
    /// Operation subjects keyed by logical name.
    pub operations: BTreeMap<String, String>,
    /// Event base and wildcard subjects keyed by logical name.
    pub events: BTreeMap<String, DerivedEventSubjects>,
    /// Feed subjects keyed by logical name.
    pub feeds: BTreeMap<String, String>,
}

/// Base, publish template, and wildcard subscription subjects for one event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedEventSubjects {
    /// Event subject before parameter tokens are appended.
    pub base: String,
    /// Publish subject template with JSON-pointer placeholders for parameters.
    pub template: String,
    /// Subscription subject with one wildcard per event parameter.
    pub wildcard: String,
}

/// Derive an RPC subject from its version and logical name.
///
/// # Errors
///
/// Returns [`ProtocolError::InvalidIdentifier`] for an invalid `vN` version or
/// logical surface name.
pub fn derive_rpc_subject(version: &str, logical_name: &str) -> Result<String, ProtocolError> {
    derive_subject("rpc", version, logical_name)
}

/// Derive an operation subject from its version and logical name.
///
/// # Errors
///
/// Returns [`ProtocolError::InvalidIdentifier`] for an invalid `vN` version or
/// logical surface name.
pub fn derive_operation_subject(
    version: &str,
    logical_name: &str,
) -> Result<String, ProtocolError> {
    derive_subject("operations", version, logical_name)
}

/// Derive an event base subject from its version and logical name.
///
/// # Errors
///
/// Returns [`ProtocolError::InvalidIdentifier`] for an invalid `vN` version or
/// logical surface name.
pub fn derive_event_subject(version: &str, logical_name: &str) -> Result<String, ProtocolError> {
    derive_subject("events", version, logical_name)
}

/// Derive an event subscription subject with one wildcard per parameter.
///
/// Parameter order is defined by the semantic API; this function appends one
/// wildcard token for each parameter without reordering it.
///
/// # Errors
///
/// Returns [`ProtocolError::InvalidIdentifier`] for an invalid `vN` version or
/// logical surface name.
pub fn derive_event_wildcard_subject(
    version: &str,
    logical_name: &str,
    parameter_count: usize,
) -> Result<String, ProtocolError> {
    let mut subject = derive_event_subject(version, logical_name)?;
    for _ in 0..parameter_count {
        subject.push_str(".*");
    }
    Ok(subject)
}

/// Derive a feed subject from its version and logical name.
///
/// # Errors
///
/// Returns [`ProtocolError::InvalidIdentifier`] for an invalid `vN` version or
/// logical surface name.
pub fn derive_feed_subject(version: &str, logical_name: &str) -> Result<String, ProtocolError> {
    derive_subject("feed", version, logical_name)
}

/// Return whether two tokenized event subject patterns can match the same subject.
///
/// `*` matches one token. Patterns with different token counts cannot overlap.
#[must_use]
pub fn event_patterns_overlap(left: &str, right: &str) -> bool {
    let left = left.split('.').collect::<Vec<_>>();
    let right = right.split('.').collect::<Vec<_>>();
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left == &"*" || right == "*" || left == &right)
}

fn derive_subject(
    family: &str,
    version: &str,
    logical_name: &str,
) -> Result<String, ProtocolError> {
    validate_version(version)?;
    validate_logical_name(logical_name)?;
    Ok(format!("{family}.{version}.{logical_name}"))
}

#[cfg(test)]
mod tests {
    use super::{derive_rpc_subject, event_patterns_overlap};

    #[test]
    fn subject_versions_use_canonical_positive_decimals() {
        assert_eq!(
            derive_rpc_subject("v1", "Documents.Get").unwrap(),
            "rpc.v1.Documents.Get"
        );
        assert_eq!(
            derive_rpc_subject("v10", "Documents.Get").unwrap(),
            "rpc.v10.Documents.Get"
        );
        assert!(derive_rpc_subject("v01", "Documents.Get").is_err());
        assert!(derive_rpc_subject("v00", "Documents.Get").is_err());
    }

    #[test]
    fn event_patterns_overlap_by_token() {
        assert!(event_patterns_overlap(
            "events.v1.Sites.Changed.*",
            "events.v1.Sites.Changed.eu"
        ));
        assert!(!event_patterns_overlap(
            "events.v1.Sites.Changed.*",
            "events.v1.Sites.Changed.eu.extra"
        ));
        assert!(!event_patterns_overlap(
            "events.v1.Sites.Changed.us",
            "events.v1.Sites.Changed.eu"
        ));
    }
}
